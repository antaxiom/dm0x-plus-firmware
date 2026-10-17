//! # Periodic Motor Control Loop (`Dm01Controller`)
//!
//! Integrates all sensor inputs, riding modes (`Eco`/`City`/`Sport`), assist levels (`0..=8`, `A`, `C`, `R`),
//! and both the decompiled `V6510C` integer-math pipeline (`step_v6510c`) and the manual-specified
//! periodic control loop (`step`).

use crate::config::{
    AssistLevelConfig, AssistSelection, MotorConfig, PasControlMode, PasType, RidingMode,
    TorqueMode,
};
use crate::power_curve::{
    compute_global_mode_ceiling_da, compute_level_power_ceiling_da, compute_pas_current_da,
    PowerCurveArbitrator, ThrottleState as V6510cThrottleState,
    TorqueSensorState as V6510cTorqueState,
};
use crate::sensors::{parse_throttle_permille, TorqueSensorState};
use crate::thermal::ThermalMonitor;

/// Snapshot of physical sensor inputs sampled at each periodic control tick.
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
    pub motor_temp_c: i16,
}

/// Output command from the periodic motor control loop to the FOC current loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlOutput {
    /// Target phase/battery current magnitude in milliamps (mA).
    pub target_current_ma: u32,
    /// True if Special Level R (Reverse) is active.
    pub reverse_direction: bool,
    /// True if safety, brake, or shift cutoffs are actively inhibiting power.
    pub cutoff_active: bool,
    /// True if controller or motor thermal rollback is active.
    pub thermal_throttling: bool,
}

/// Periodic motor controller supporting both the exact decompiled `V6510C` pipeline
/// (`step_v6510c`) and the T24 Display Manual baseline (`step`).
pub struct Dm01Controller {
    pub config: MotorConfig,
    pub mode: RidingMode,
    pub assist: AssistSelection,
    pub torque_state: TorqueSensorState,
    pub thermal: ThermalMonitor,
    pub v6510c_torque: V6510cTorqueState,
    pub v6510c_throttle: V6510cThrottleState,
    pub v6510c_arbitrator: PowerCurveArbitrator,
    pas_keep_timer_ms: u16,
    shift_timer_ms: u16,
}

impl Dm01Controller {
    pub fn new(config: MotorConfig) -> Self {
        let zero_mv = config.torque_zero_vol_mv;
        let max_mv = config.torque_max_vol_mv;
        let start_mv = config.torque_startup_net_vol_mv;
        let th_start_mv = config.throttle_startup_vol_mv;
        Self {
            config,
            mode: RidingMode::City,
            assist: AssistSelection::Level(1),
            torque_state: TorqueSensorState::new(zero_mv),
            thermal: ThermalMonitor::new(),
            v6510c_torque: V6510cTorqueState::new(zero_mv, max_mv, start_mv),
            v6510c_throttle: V6510cThrottleState::new(th_start_mv, 3250),
            v6510c_arbitrator: PowerCurveArbitrator::new(),
            pas_keep_timer_ms: 0,
            shift_timer_ms: 0,
        }
    }

    /// Executes one exact decompiled `V6510C` control cycle (`0x08003FD4`, `0x08003814`,
    /// `0x08008024`, `0x080067D4`, `0x08008DA4`, `0x08006620`, `0x0800674C`, `0x08005478`).
    pub fn step_v6510c(&mut self, inputs: &SensorInputs) -> ControlOutput {
        let cutoff = inputs.brake_active
            || inputs.backpedaling
            || inputs.shift_sensor_triggered
            || inputs.battery_voltage_mv <= self.config.low_cut_voltage_mv;

        let level_cfg = match self.assist {
            AssistSelection::Off => {
                self.v6510c_arbitrator.ramped_current_da = 0;
                return ControlOutput {
                    target_current_ma: 0,
                    reverse_direction: false,
                    cutoff_active: cutoff,
                    thermal_throttling: inputs.controller_temp_c > self.config.temp_ctl_min_c,
                };
            }
            AssistSelection::Level(lvl) => self
                .config
                .active_level(lvl)
                .copied()
                .unwrap_or(self.config.levels[0]),
            AssistSelection::Auto => self.config.levels[2],
            AssistSelection::Climb => AssistLevelConfig {
                speed_limit_deci_kph: self.config.max_level_speed_deci_kph(),
                power_percent: 100,
                eassist_percent: 100,
            },
            AssistSelection::Reverse => AssistLevelConfig {
                speed_limit_deci_kph: 60,
                power_percent: 25,
                eassist_percent: 30,
            },
        };

        let mode_max_da = (match self.mode {
            RidingMode::Eco => self.config.max_current_eco_ma,
            RidingMode::City => self.config.max_current_city_ma,
            RidingMode::Sport => self.config.max_current_sport_ma,
        } / 100) as u16;
        let sport_max_da = (self.config.max_current_sport_ma / 100) as u16;
        let battery_dv = (inputs.battery_voltage_mv / 100) as u16;
        let lvc_dv = (self.config.low_cut_voltage_mv / 100) as u16;

        // 1. Stage 1 Ceilings: Global Mode Ceiling (Torque) vs Level Power Ceiling (PAS/Throttle)
        let global_ceil_da = compute_global_mode_ceiling_da(
            mode_max_da,
            sport_max_da,
            inputs.controller_temp_c,
            self.config.temp_ctl_min_c,
            self.config.temp_ctl_max_c,
            battery_dv,
        );
        let level_ceil_da = compute_level_power_ceiling_da(
            mode_max_da,
            sport_max_da,
            level_cfg.power_percent,
            inputs.controller_temp_c,
            self.config.temp_ctl_min_c,
            self.config.temp_ctl_max_c,
            battery_dv,
        );

        // 2. Stage 2A: Torque Sensor Curve (ignores Pwr[level], uses EAssist[level] and global_ceil_da)
        let torque_offset_da = (self.config.torque_min_out_current_ma / 100) as u16;
        let torque_da = if self.config.torque_sensor_enabled {
            self.v6510c_torque.compute_current_da(
                inputs.torque_adc_mv,
                inputs.cadence_rpm,
                self.config.torque_filter,
                self.config.torque_strength_value,
                level_cfg.eassist_percent,
                torque_offset_da,
                mode_max_da,
                global_ceil_da,
            )
        } else {
            0
        };

        // 3. Stage 2B: Cadence (PAS) Curve (capped by level_ceil_da = Pwr[level])
        let pas_active = self.config.pas_type != PasType::Disable
            && inputs.cadence_rpm > 0
            && !inputs.backpedaling;
        let pas_offset_da = (self.config.pas_current_offset_ma / 100) as u16;
        let pas_da = compute_pas_current_da(
            inputs.cadence_rpm,
            pas_active,
            self.config.pas_max_speed_rpm,
            pas_offset_da,
            global_ceil_da,
            level_ceil_da,
        );

        // 4. Stage 2C: Throttle Curve (capped by level_ceil_da = Pwr[level])
        let throttle_da = if self.config.throttle_enabled {
            self.v6510c_throttle
                .compute_current_da(inputs.throttle_adc_mv, level_ceil_da)
        } else {
            0
        };

        // 5. Stage 3: Arbitrator, ±0.2A/step Ramp, Speed Soft-Taper, and LVC 3.0V Cushion
        let pedaling_forward = inputs.cadence_rpm > 0 && !inputs.backpedaling;
        let speed_limit_deci_kph = if inputs.walk_assist_held {
            60
        } else {
            level_cfg.speed_limit_deci_kph
        };
        let final_da = self.v6510c_arbitrator.step(
            cutoff,
            pedaling_forward,
            torque_da,
            pas_da,
            throttle_da,
            inputs.wheel_speed_deci_kph,
            speed_limit_deci_kph,
            inputs.walk_assist_held,
            battery_dv,
            lvc_dv,
        );

        ControlOutput {
            target_current_ma: (final_da as u32) * 100,
            reverse_direction: self.assist == AssistSelection::Reverse,
            cutoff_active: cutoff,
            thermal_throttling: inputs.controller_temp_c > self.config.temp_ctl_min_c,
        }
    }

    /// Executes one stock control step (e.g., 100 Hz / 10 ms) matching the T24 Display Manual tables.
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
                reverse_direction: false,
                cutoff_active: true,
                thermal_throttling: self.thermal.ctl_overheated || self.thermal.mot_overheated,
            };
        }

        // 2. Shift sensor interruption timer
        if self.config.shift_sensor_enabled && inputs.shift_sensor_triggered {
            self.shift_timer_ms = self.config.shift_delay_ms;
        } else if self.shift_timer_ms > 0 {
            self.shift_timer_ms = self.shift_timer_ms.saturating_sub(dt_ms);
        }

        // 3. Walk assistance (6.0 km/h cap, T24 Manual Page 4)
        if inputs.walk_assist_held {
            let walk_current_ma = if inputs.wheel_speed_deci_kph < 60 {
                3_000
            } else {
                0
            };
            return ControlOutput {
                target_current_ma: walk_current_ma,
                reverse_direction: false,
                cutoff_active: false,
                thermal_throttling: self.thermal.ctl_overheated || self.thermal.mot_overheated,
            };
        }

        // 4. Resolve active assist level or special mode (0, 1..=8, Auto, Climb, Reverse)
        let mut reverse_direction = false;
        let resolved_level: Option<AssistLevelConfig> = match self.assist {
            AssistSelection::Off => None,
            AssistSelection::Level(lvl) => self.config.active_level(lvl).copied(),
            AssistSelection::Auto => {
                let idx = if inputs.wheel_speed_deci_kph < 120 {
                    1
                } else if inputs.wheel_speed_deci_kph < 220 {
                    3
                } else {
                    self.config.level_count
                };
                self.config.active_level(idx).copied()
            }
            AssistSelection::Climb => Some(AssistLevelConfig {
                speed_limit_deci_kph: self.config.max_level_speed_deci_kph(),
                power_percent: 100,
                eassist_percent: 100,
            }),
            AssistSelection::Reverse => {
                reverse_direction = true;
                Some(AssistLevelConfig {
                    speed_limit_deci_kph: 60,
                    power_percent: 25,
                    eassist_percent: 30,
                })
            }
        };

        let level_cfg = match resolved_level {
            Some(cfg) => cfg,
            None => {
                return ControlOutput {
                    target_current_ma: 0,
                    reverse_direction: false,
                    cutoff_active: false,
                    thermal_throttling: self.thermal.ctl_overheated || self.thermal.mot_overheated,
                };
            }
        };

        // 5. Mode maximum current ceiling (compensated for battery voltage)
        let mode_max_ma = self
            .config
            .mode_current_limit_ma(self.mode, inputs.battery_voltage_mv);

        // 6. Per-level current ceiling (`Power%` of mode maximum current)
        let level_ceiling_ma = (mode_max_ma * level_cfg.power_percent as u32) / 100;

        // 7. Determine PAS active state (`PAS working` vs `PAS not working`)
        if self.config.pas_type != PasType::Disable
            && inputs.cadence_rpm > 0
            && !inputs.backpedaling
        {
            self.pas_keep_timer_ms = self.config.pas_keep_time_ms.clamp(200, 1000);
        } else if self.pas_keep_timer_ms > 0 {
            self.pas_keep_timer_ms = self.pas_keep_timer_ms.saturating_sub(dt_ms);
        }
        let pas_active = self.pas_keep_timer_ms > 0;

        // 8. Compute Pedal (Torque + PAS) demand
        self.torque_state
            .step_zero_drift_tracking(inputs.torque_adc_mv, inputs.cadence_rpm);
        let torque_permille = self
            .torque_state
            .update(inputs.torque_adc_mv, dt_ms, &self.config);
        let torque_allowed = match self.config.torque_mode {
            TorqueMode::Safe => pas_active,
            TorqueMode::Sensitive => true,
        };

        let mut pedal_current_ma = 0u32;
        if self.config.torque_sensor_enabled && torque_allowed && torque_permille > 0 {
            let scaled_torque = (torque_permille as u32 * level_cfg.eassist_percent as u32) / 100;
            let raw_ma = self.config.torque_min_out_current_ma
                + ((mode_max_ma.saturating_sub(self.config.torque_min_out_current_ma))
                    * scaled_torque)
                    / 1000;
            pedal_current_ma = raw_ma.min(mode_max_ma);
        } else if pas_active {
            let max_cad = self.config.pas_max_speed_rpm.max(60) as u32;
            let cadence_factor = ((inputs.cadence_rpm as u32 * 1000) / max_cad).min(1000);
            let raw_ma = self.config.pas_current_offset_ma
                + ((level_ceiling_ma.saturating_sub(self.config.pas_current_offset_ma))
                    * cadence_factor)
                    / 1000;
            pedal_current_ma = raw_ma.min(level_ceiling_ma);
        }

        // Enforce PAS / Pedal Speed Limit (T24 Manual Pages 12-13)
        let pedal_speed_limit_deci_kph = match self.config.pas_control_mode {
            PasControlMode::MaxBikeSpeed => self.config.max_level_speed_deci_kph(),
            PasControlMode::LevelSpeed => level_cfg.speed_limit_deci_kph,
            PasControlMode::PedalSpeed1 | PasControlMode::PedalSpeed2 => {
                let max_wheel = self.config.max_level_speed_deci_kph() as u32;
                let max_pas = self.config.pas_max_speed_rpm.max(60) as u32;
                ((inputs.cadence_rpm as u32 * max_wheel) / max_pas).min(max_wheel) as u16
            }
        };
        if inputs.wheel_speed_deci_kph >= pedal_speed_limit_deci_kph {
            pedal_current_ma = 0;
        }

        // 9. Compute Throttle demand using the exact 0..=12 ControlMode table (T24 Manual Page 13)
        let mut throttle_current_ma = 0u32;
        let throttle_permille =
            parse_throttle_permille(inputs.throttle_adc_mv, self.config.throttle_startup_vol_mv);
        if throttle_permille > 0 {
            if !self.config.throttle_enabled {
                if inputs.wheel_speed_deci_kph < 60 {
                    throttle_current_ma = (mode_max_ma * throttle_permille as u32) / 4000;
                }
            } else {
                let max_gear_spd = self.config.max_level_speed_deci_kph();
                let lvl_spd = level_cfg.speed_limit_deci_kph;

                let throttle_spec: Option<(Option<u16>, bool)> =
                    match (self.config.throttle_control_mode, pas_active) {
                        (0, false) | (1, false) | (2, false) => None,
                        (0, true) => Some((Some(60), false)),
                        (1, true) => Some((Some(lvl_spd), false)),
                        (2, true) => Some((Some(max_gear_spd), false)),
                        (3, _) => Some((Some(60), false)),
                        (4, false) => Some((Some(60), false)),
                        (4, true) => Some((Some(lvl_spd), false)),
                        (5, false) => Some((Some(60), false)),
                        (5, true) => Some((Some(max_gear_spd), false)),
                        (6, _) => Some((Some(lvl_spd), false)),
                        (7, false) => Some((Some(lvl_spd.max(320)), false)),
                        (7, true) => Some((Some(max_gear_spd), false)),
                        (8, false) => Some((Some(lvl_spd), true)),
                        (8, true) => Some((Some(max_gear_spd), true)),
                        (9, false) => Some((Some(60), false)),
                        (9, true) => Some((Some(max_gear_spd), true)),
                        (10, false) => Some((Some(lvl_spd), false)),
                        (10, true) => Some((Some(max_gear_spd), true)),
                        (11, _) => Some((Some(max_gear_spd), true)),
                        (12, _) => Some((None, true)),
                        _ => Some((Some(max_gear_spd), false)),
                    };

                if let Some((opt_limit, full2_power)) = throttle_spec {
                    let ceiling = if full2_power {
                        mode_max_ma
                    } else {
                        level_ceiling_ma
                    };
                    let within_speed = match opt_limit {
                        Some(limit) => inputs.wheel_speed_deci_kph < limit,
                        None => true,
                    };
                    if within_speed {
                        throttle_current_ma = (ceiling * throttle_permille as u32) / 1000;
                    }
                }
            }
        }

        let mut combined_ma = pedal_current_ma.max(throttle_current_ma);

        // 10. Shift sensor reduction
        if self.shift_timer_ms > 0 {
            combined_ma = (combined_ma * self.config.shift_power_percent as u32) / 100;
        }

        // 11. Thermal protection rollback (CTL and MOT sensors)
        let thermal_scale_permille = self.thermal.update_scale_permille(
            inputs.controller_temp_c,
            inputs.motor_temp_c,
            &self.config,
        );
        combined_ma = (combined_ma * thermal_scale_permille as u32) / 1000;

        ControlOutput {
            target_current_ma: combined_ma,
            reverse_direction,
            cutoff_active: self.shift_timer_ms > 0,
            thermal_throttling: self.thermal.ctl_overheated || self.thermal.mot_overheated,
        }
    }
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
            motor_temp_c: 50,
        }
    }

    #[test]
    fn test_brake_and_backpedal_cutoff() {
        let mut ctrl = Dm01Controller::new(MotorConfig::default());
        ctrl.mode = RidingMode::Sport;
        ctrl.assist = AssistSelection::Level(5);

        let mut inputs = default_inputs();
        inputs.torque_adc_mv = 2500;
        inputs.cadence_rpm = 80;
        inputs.brake_active = true;

        let out = ctrl.step(&inputs, 10);
        assert_eq!(out.target_current_ma, 0);
        assert!(out.cutoff_active);

        let out_v6510c = ctrl.step_v6510c(&inputs);
        assert_eq!(out_v6510c.target_current_ma, 0);
        assert!(out_v6510c.cutoff_active);
    }

    #[test]
    fn test_stock_throttle_modes_0_to_12() {
        let mut ctrl = Dm01Controller::new(MotorConfig::default());
        ctrl.mode = RidingMode::Sport;
        ctrl.assist = AssistSelection::Level(2);

        let mut inputs = default_inputs();
        inputs.throttle_adc_mv = 3250;
        inputs.cadence_rpm = 0;
        inputs.wheel_speed_deci_kph = 100;

        ctrl.config.throttle_control_mode = 0;
        assert_eq!(ctrl.step(&inputs, 10).target_current_ma, 0);

        ctrl.config.throttle_control_mode = 3;
        assert_eq!(ctrl.step(&inputs, 10).target_current_ma, 0);

        ctrl.config.throttle_control_mode = 12;
        inputs.wheel_speed_deci_kph = 450;
        assert!(ctrl.step(&inputs, 10).target_current_ma > 25_000);
    }

    #[test]
    fn test_thermal_rollback_at_100c() {
        let mut ctrl = Dm01Controller::new(MotorConfig::default());
        ctrl.mode = RidingMode::Sport;
        ctrl.assist = AssistSelection::Level(5);

        let mut inputs = default_inputs();
        inputs.throttle_adc_mv = 3250;
        inputs.controller_temp_c = 50;
        let normal = ctrl.step(&inputs, 10);

        inputs.controller_temp_c = 102;
        let overheated = ctrl.step(&inputs, 10);

        assert!(overheated.thermal_throttling);
        assert_eq!(
            overheated.target_current_ma,
            (normal.target_current_ma * 30) / 100
        );
    }
}
