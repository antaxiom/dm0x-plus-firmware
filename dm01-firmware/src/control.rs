use crate::config::{MotorConfig, RidingMode, SpeedLimitMode, TorqueMode};
use crate::sensors::{parse_throttle_permille, TorqueSensorState};
use crate::thermal::ThermalMonitor;

/// Snapshot of physical sensor inputs sampled at each control tick.
#[derive(Debug, Clone, Copy)]
pub struct SensorInputs {
    pub battery_voltage_mv: u32,
    pub torque_adc_mv: u16,
    pub throttle_adc_mv: u16,
    pub cadence_rpm: u16,
    pub backpedaling: bool,
    pub wheel_speed_deci_kph: u16,
    pub brake_active: bool,
    pub shift_sensor_triggered: bool,
    pub walk_assist_held: bool,
    pub controller_temp_c: i16,
}

/// Output command from the high-level motor control loop to the FOC current loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlOutput {
    /// Target phase/battery current in milliamps (mA).
    pub target_current_ma: u32,
    /// True if safety or shift/brake cutoffs are actively inhibiting power.
    pub cutoff_active: bool,
    /// True if thermal rollback is active.
    pub thermal_throttling: bool,
}

/// Stateful periodic controller for the DM01 mid-drive motor.
pub struct Dm01Controller {
    pub config: MotorConfig,
    pub mode: RidingMode,
    pub assist_level: u8,
    pub torque_state: TorqueSensorState,
    pub thermal: ThermalMonitor,
    pas_keep_timer_ms: u16,
    shift_timer_ms: u16,
}

impl Dm01Controller {
    pub fn new(config: MotorConfig) -> Self {
        let zero_mv = config.torque_zero_vol_mv;
        Self {
            config,
            mode: RidingMode::City,
            assist_level: 1,
            torque_state: TorqueSensorState::new(zero_mv),
            thermal: ThermalMonitor::new(),
            pas_keep_timer_ms: 0,
            shift_timer_ms: 0,
        }
    }

    /// Executes one control step (e.g., every 10 ms / 100 Hz) and computes the target current.
    pub fn step(&mut self, inputs: &SensorInputs, dt_ms: u16) -> ControlOutput {
        // 1. Immediate safety cutoffs: brake levers, backpedaling, or low-voltage cutoff
        if inputs.brake_active
            || inputs.backpedaling
            || inputs.battery_voltage_mv < self.config.low_cut_voltage_mv
        {
            self.pas_keep_timer_ms = 0;
            self.torque_state.keep_timer_ms = 0;
            return ControlOutput {
                target_current_ma: 0,
                cutoff_active: true,
                thermal_throttling: self.thermal.is_throttling,
            };
        }

        // 2. Shift sensor timer management
        if self.config.shift_sensor_enabled && inputs.shift_sensor_triggered {
            self.shift_timer_ms = self.config.shift_delay_ms;
        } else if self.shift_timer_ms > 0 {
            self.shift_timer_ms = self.shift_timer_ms.saturating_sub(dt_ms);
        }

        // 3. Walk assist mode (6.0 km/h cap)
        if inputs.walk_assist_held {
            let walk_current_ma = if inputs.wheel_speed_deci_kph < 60 {
                3_000 // Gentle 3A walk assist
            } else {
                0
            };
            return ControlOutput {
                target_current_ma: walk_current_ma,
                cutoff_active: false,
                thermal_throttling: self.thermal.is_throttling,
            };
        }

        // 4. Determine mode maximum current (with voltage sag compensation)
        let mode_max_ma = self
            .config
            .mode_current_limit_ma(self.mode, inputs.battery_voltage_mv);

        // 5. Update torque sensor state
        let torque_permille = self.torque_state.update(
            inputs.torque_adc_mv,
            inputs.cadence_rpm,
            dt_ms,
            &self.config,
        );

        // 6. Update PAS keep-time state
        let pas_active = if self.config.pas_enabled
            && inputs.cadence_rpm > 0
            && inputs.cadence_rpm <= self.config.pas_max_cadence_rpm
        {
            self.pas_keep_timer_ms = self.config.pas_keep_time_ms.clamp(200, 1000);
            true
        } else if self.pas_keep_timer_ms > 0 {
            self.pas_keep_timer_ms = self.pas_keep_timer_ms.saturating_sub(dt_ms);
            true
        } else {
            false
        };

        // 7. Compute pedal-assist demand from active level
        let mut pedal_current_ma = 0u32;
        if let Some(level_cfg) = self.config.active_level(self.assist_level) {
            let level_ceiling_ma = (mode_max_ma * level_cfg.power_percent as u32) / 100;

            if self.config.torque_sensor_enabled {
                // In Safe mode, require PAS rotation or high pedal force; in Sensitive mode, respond immediately
                let can_engage = match self.config.torque_mode {
                    TorqueMode::Safe => pas_active || torque_permille > 250,
                    TorqueMode::Sensitive => torque_permille > 0,
                };
                if can_engage && torque_permille > 0 {
                    // Combine EAssist base boost with proportional torque effort
                    let eassist_base_ma =
                        (level_ceiling_ma * level_cfg.eassist_percent as u32) / 100;
                    let dynamic_span_ma = level_ceiling_ma.saturating_sub(eassist_base_ma);
                    let raw_ma = self.config.torque_current_offset_ma
                        + eassist_base_ma
                        + (dynamic_span_ma * torque_permille as u32) / 1000;
                    pedal_current_ma = raw_ma.min(level_ceiling_ma);
                }
            } else if pas_active {
                // Cadence-only (PAS) mode scales with cadence up to level ceiling
                let cadence_factor = ((inputs.cadence_rpm as u32
                    * self.config.pas_strength_rpm.max(1) as u32)
                    * 10)
                    .min(1000);
                let raw_ma = self.config.pas_current_offset_ma
                    + ((level_ceiling_ma.saturating_sub(self.config.pas_current_offset_ma))
                        * cadence_factor)
                        / 1000;
                pedal_current_ma = raw_ma.min(level_ceiling_ma);
            }

            // Apply smooth speed limit tapering (2.0 km/h taper band instead of abrupt wall cutoff)
            let speed_mode = if self.config.torque_sensor_enabled {
                self.config.torque_speed_mode
            } else {
                self.config.pas_speed_mode
            };
            let speed_limit_deci_kph = match speed_mode {
                SpeedLimitMode::SpeedLevel => Some(level_cfg.speed_limit_deci_kph),
                SpeedLimitMode::SpeedMaxLevel => Some(self.config.max_level_speed_deci_kph()),
                SpeedLimitMode::NoSpeedLimit => None,
            };
            if let Some(limit) = speed_limit_deci_kph {
                pedal_current_ma =
                    apply_speed_taper(pedal_current_ma, inputs.wheel_speed_deci_kph, limit);
            }
        }

        // 8. Compute Throttle demand
        let mut throttle_current_ma = 0u32;
        let throttle_permille = parse_throttle_permille(inputs.throttle_adc_mv);
        if throttle_permille > 0 && self.assist_level > 0 {
            if !self.config.throttle_enabled {
                // When throttle is disabled in settings, it acts as a 6 km/h walk assist
                if inputs.wheel_speed_deci_kph < 60 {
                    throttle_current_ma = (mode_max_ma * throttle_permille as u32) / 4000;
                }
            } else {
                // Mode 7 default: 32 km/h throttle-only, 40 km/h when pedaling; Mode 12: unrestricted scooter mode
                let throttle_limit_deci_kph = match (self.config.throttle_speed_mode, pas_active) {
                    (3, _) => Some(60),        // 6.0 km/h EN15194 mode
                    (7, false) => Some(320),   // 32.0 km/h throttle-only
                    (7, true) => Some(400),    // 40.0 km/h with PAS
                    (12, _) => None,           // Scooter mode
                    (_, _) => Some(self.config.max_level_speed_deci_kph()),
                };
                let raw_throttle_ma = (mode_max_ma * throttle_permille as u32) / 1000;
                throttle_current_ma = match throttle_limit_deci_kph {
                    Some(limit) => {
                        apply_speed_taper(raw_throttle_ma, inputs.wheel_speed_deci_kph, limit)
                    }
                    None => raw_throttle_ma,
                };
            }
        }

        let mut combined_ma = pedal_current_ma.max(throttle_current_ma);

        // 9. Apply Shift Sensor power reduction if active
        if self.shift_timer_ms > 0 {
            combined_ma = (combined_ma * self.config.shift_power_percent as u32) / 100;
        }

        // 10. Apply Thermal Protection scaling
        let thermal_scale_permille = self
            .thermal
            .update_scale_permille(inputs.controller_temp_c, &self.config);
        combined_ma = (combined_ma * thermal_scale_permille as u32) / 1000;

        ControlOutput {
            target_current_ma: combined_ma,
            cutoff_active: self.shift_timer_ms > 0,
            thermal_throttling: self.thermal.is_throttling,
        }
    }
}

/// Smoothly tapers current over a 2.0 km/h (20 deci-kph) window approaching `limit_deci_kph`
/// to eliminate the harsh "hitting a wall" cutoff sensation.
fn apply_speed_taper(current_ma: u32, speed_deci_kph: u16, limit_deci_kph: u16) -> u32 {
    if speed_deci_kph >= limit_deci_kph {
        return 0;
    }
    let taper_start = limit_deci_kph.saturating_sub(20);
    if speed_deci_kph <= taper_start {
        return current_ma;
    }
    let remaining = (limit_deci_kph - speed_deci_kph) as u32;
    (current_ma * remaining) / 20
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_inputs() -> SensorInputs {
        SensorInputs {
            battery_voltage_mv: 50_000,
            torque_adc_mv: 700,
            throttle_adc_mv: 700,
            cadence_rpm: 0,
            backpedaling: false,
            wheel_speed_deci_kph: 150,
            brake_active: false,
            shift_sensor_triggered: false,
            walk_assist_held: false,
            controller_temp_c: 45,
        }
    }

    #[test]
    fn test_brake_and_backpedal_cutoff() {
        let mut ctrl = Dm01Controller::new(MotorConfig::default());
        ctrl.mode = RidingMode::Sport;
        ctrl.assist_level = 5;

        let mut inputs = default_inputs();
        inputs.torque_adc_mv = 2500;
        inputs.cadence_rpm = 80;
        inputs.brake_active = true;

        let out = ctrl.step(&inputs, 10);
        assert_eq!(out.target_current_ma, 0);
        assert!(out.cutoff_active);
    }

    #[test]
    fn test_dynamic_zero_drift_compensation_prevents_ghost_pedaling() {
        let mut ctrl = Dm01Controller::new(MotorConfig::default());
        ctrl.config.torque_mode = TorqueMode::Sensitive;
        ctrl.assist_level = 3;

        let mut inputs = default_inputs();
        // Simulate thermal drift raising idle torque voltage from 700mV to 950mV while not pedaling
        inputs.torque_adc_mv = 950;
        inputs.cadence_rpm = 0;

        for _ in 0..300 {
            ctrl.step(&inputs, 500);
        }
        assert_eq!(ctrl.torque_state.dynamic_zero_mv, 950);

        // Because dynamic zero tracked to 950mV, 950mV at rest produces 0A (no ghost pedaling)
        let out = ctrl.step(&inputs, 10);
        assert_eq!(out.target_current_ma, 0);
    }

    #[test]
    fn test_thermal_rollback_at_100c() {
        let mut ctrl = Dm01Controller::new(MotorConfig::default());
        ctrl.mode = RidingMode::Sport;
        ctrl.assist_level = 5;

        let mut inputs = default_inputs();
        inputs.throttle_adc_mv = 3250; // Full throttle
        inputs.controller_temp_c = 50;
        let normal = ctrl.step(&inputs, 10);

        inputs.controller_temp_c = 102; // Exceeds 100C MaxTemp
        let overheated = ctrl.step(&inputs, 10);

        assert!(overheated.thermal_throttling);
        assert_eq!(overheated.target_current_ma, (normal.target_current_ma * 30) / 100);
    }
}

