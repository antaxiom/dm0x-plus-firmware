use crate::config::MotorConfig;

/// Minimum valid torque sensor resting voltage in millivolts before sensor fault detection.
pub const TORQUE_MIN_VALID_MV: u16 = 100;
/// Maximum Hall throttle ADC voltage in millivolts (3.3V full scale).
pub const THROTTLE_MAX_MV: u16 = 3250;

/// Stock torque sensor state machine matching the DM01L power-on zero-point calibration
/// (DM01 User Manual Page 31 & T24 Display Manual Pages 13-14).
#[derive(Debug, Clone)]
pub struct TorqueSensorState {
    /// Zero-point reference voltage captured at power-on calibration (`ZeroVol(mv)`).
    pub calibrated_zero_mv: u16,
    /// Smoothed torque signal output in permille (`0..=1000`).
    pub filtered_demand_permille: u16,
    /// Remaining keep-time hold counter in milliseconds.
    pub keep_timer_ms: u16,
}

impl TorqueSensorState {
    pub fn new(initial_zero_mv: u16) -> Self {
        Self {
            calibrated_zero_mv: initial_zero_mv,
            filtered_demand_permille: 0,
            keep_timer_ms: 0,
        }
    }

    /// Sets the power-on zero reference recorded during the startup calibration procedure.
    pub fn set_power_on_zero(&mut self, zero_mv: u16) {
        self.calibrated_zero_mv = zero_mv;
    }

    /// Computes normalized torque demand (`0..=1000` permille) from raw strain-gauge voltage.
    /// Net starting voltage formula (T24 Manual Page 13-14):
    /// `net_torque_mv = raw_torque_mv - zero_point_voltage`
    /// Activated when `net_torque_mv > StartupNetVol(mv)`.
    pub fn update(&mut self, raw_torque_mv: u16, dt_ms: u16, config: &MotorConfig) -> u16 {
        if !config.torque_sensor_enabled || raw_torque_mv < TORQUE_MIN_VALID_MV {
            self.filtered_demand_permille = 0;
            return 0;
        }

        let zero_mv = self.calibrated_zero_mv.max(config.torque_zero_vol_mv);
        let net_torque_mv = raw_torque_mv.saturating_sub(zero_mv);
        let startup_net_mv = config.torque_startup_net_vol_mv;

        let raw_demand_permille = if net_torque_mv > startup_net_mv {
            self.keep_timer_ms = config.torque_keep_time_ms.min(5000);
            let active_delta = (net_torque_mv - startup_net_mv) as u32;
            let max_span = config
                .torque_max_vol_mv
                .saturating_sub(zero_mv.saturating_add(startup_net_mv))
                .max(1) as u32;
            let scaled =
                (active_delta * 1000 * config.torque_strength_value as u32) / (max_span * 10);
            scaled.min(1000) as u16
        } else if self.keep_timer_ms > 0 {
            self.keep_timer_ms = self.keep_timer_ms.saturating_sub(dt_ms);
            ((self.filtered_demand_permille as u32 * self.keep_timer_ms as u32)
                / config.torque_keep_time_ms.max(1) as u32) as u16
        } else {
            0
        };

        // First-order smoothing filter (`Filter` parameter 0..=10)
        let alpha_denom = (config.torque_filter as u32) + 1;
        self.filtered_demand_permille = (((self.filtered_demand_permille as u32
            * (alpha_denom - 1))
            + raw_demand_permille as u32)
            / alpha_denom) as u16;

        self.filtered_demand_permille
    }
}

/// Normalizes Hall throttle voltage into permille (`0..=1000`) using configurable `StartupVol(mv)`
/// (`800mV..=2000mV`, T24 Display Manual Page 13).
pub fn parse_throttle_permille(raw_throttle_mv: u16, startup_vol_mv: u16) -> u16 {
    let start_mv = startup_vol_mv.clamp(700, 2000);
    if raw_throttle_mv <= start_mv {
        return 0;
    }
    let span = THROTTLE_MAX_MV.saturating_sub(start_mv).max(1) as u32;
    let delta = (raw_throttle_mv.min(THROTTLE_MAX_MV) - start_mv) as u32;
    ((delta * 1000) / span) as u16
}
