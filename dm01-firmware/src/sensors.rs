use crate::config::MotorConfig;

/// Maximum torque sensor ADC voltage in millivolts (3.3V).
pub const TORQUE_MAX_MV: u16 = 3300;
/// Minimum valid torque sensor resting voltage in millivolts before fault detection.
pub const TORQUE_MIN_VALID_MV: u16 = 400;

/// Throttle resting and maximum voltages in millivolts.
pub const THROTTLE_REST_MV: u16 = 750;
pub const THROTTLE_MAX_MV: u16 = 3250;

/// Dynamic zero-point tracking state for the Wheatstone bridge strain-gauge torque sensor.
/// Compensates for thermal expansion drift when pedal pressure is released.
#[derive(Debug, Clone)]
pub struct TorqueSensorState {
    /// Dynamically tracked zero-point voltage in mV.
    pub dynamic_zero_mv: u16,
    /// Filtered torque demand in permille (0..=1000).
    pub filtered_demand_permille: u16,
    /// Remaining keep-time hold counter in milliseconds.
    pub keep_timer_ms: u16,
    /// Time spent with zero cadence and steady voltage for dynamic drift recalibration.
    pub idle_window_ms: u16,
}

impl TorqueSensorState {
    pub fn new(initial_zero_mv: u16) -> Self {
        Self {
            dynamic_zero_mv: initial_zero_mv,
            filtered_demand_permille: 0,
            keep_timer_ms: 0,
            idle_window_ms: 0,
        }
    }

    /// Updates dynamic zero-point calibration and computes normalized torque demand (0..=1000).
    pub fn update(
        &mut self,
        raw_torque_mv: u16,
        cadence_rpm: u16,
        dt_ms: u16,
        config: &MotorConfig,
    ) -> u16 {
        if !config.torque_sensor_enabled || raw_torque_mv < TORQUE_MIN_VALID_MV {
            self.filtered_demand_permille = 0;
            return 0;
        }

        // Dynamic zero-point drift tracking:
        // When the rider is not pedaling (cadence == 0) and raw voltage is near the resting band
        // (e.g., thermally drifted between 500 mV and 1250 mV), slowly slew dynamic_zero_mv
        // toward raw_torque_mv so ghost pedaling cannot occur.
        if cadence_rpm == 0 && raw_torque_mv <= 1250 {
            self.idle_window_ms = self.idle_window_ms.saturating_add(dt_ms);
            if self.idle_window_ms >= 500 {
                if raw_torque_mv > self.dynamic_zero_mv {
                    self.dynamic_zero_mv = self.dynamic_zero_mv.saturating_add(1);
                } else if raw_torque_mv < self.dynamic_zero_mv {
                    self.dynamic_zero_mv = self.dynamic_zero_mv.saturating_sub(1);
                }
                self.idle_window_ms = 0;
            }
        } else {
            self.idle_window_ms = 0;
        }

        let effective_zero = self.dynamic_zero_mv.max(config.torque_zero_vol_mv);
        let trigger_mv = effective_zero.saturating_add(config.torque_startup_vol_mv);

        let raw_demand_permille = if raw_torque_mv > trigger_mv {
            self.keep_timer_ms = config.torque_keep_time_ms.min(5000);
            let span = TORQUE_MAX_MV.saturating_sub(trigger_mv).max(1) as u32;
            let delta = (raw_torque_mv - trigger_mv) as u32;
            let scaled = (delta * 1000 * config.torque_strength_value.max(1) as u32) / (span * 10);
            scaled.min(1000) as u16
        } else if self.keep_timer_ms > 0 {
            self.keep_timer_ms = self.keep_timer_ms.saturating_sub(dt_ms);
            // Decay demand smoothly during keep_time window
            ((self.filtered_demand_permille as u32 * self.keep_timer_ms as u32)
                / config.torque_keep_time_ms.max(1) as u32) as u16
        } else {
            0
        };

        // First-order low-pass ramp filter governed by config.torque_filter
        let alpha_denom = (config.torque_filter as u32).max(1) + 1;
        self.filtered_demand_permille = (((self.filtered_demand_permille as u32
            * (alpha_denom - 1))
            + raw_demand_permille as u32)
            / alpha_denom) as u16;

        self.filtered_demand_permille
    }
}

/// Normalizes Hall throttle voltage into permille (0..=1000).
pub fn parse_throttle_permille(raw_throttle_mv: u16) -> u16 {
    if raw_throttle_mv <= THROTTLE_REST_MV {
        return 0;
    }
    let span = (THROTTLE_MAX_MV - THROTTLE_REST_MV) as u32;
    let delta = (raw_throttle_mv.min(THROTTLE_MAX_MV) - THROTTLE_REST_MV) as u32;
    ((delta * 1000) / span) as u16
}
