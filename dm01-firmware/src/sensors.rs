//! # Strain-Gauge Torque Sensor & Hall Throttle Signal Conditioning
//!
//! Provides high-level sensor wrappers alongside the exact `V6510C` integer-math
//! [`crate::power_curve::TorqueSensorState`] and [`crate::power_curve::ThrottleState`] pipelines.

use crate::config::MotorConfig;

/// Minimum plausible raw voltage from the bottom-bracket strain gauge before flagging a broken wire.
pub const TORQUE_MIN_VALID_MV: u16 = 150;
/// Maximum full-scale Hall throttle output voltage in millivolts.
pub const THROTTLE_MAX_MV: u16 = 3250;

/// Stateful torque sensor processor with dynamic resting zero-point drift tracking (`V6510C`).
#[derive(Debug, Clone)]
pub struct TorqueSensorState {
    /// Active resting zero-point reference in millivolts (`700 mV` nominal).
    pub calibrated_zero_mv: u16,
    /// Filtered net pedal voltage in millivolts.
    pub filtered_net_mv: u16,
    /// Normalized torque demand (`0..=1000` permille).
    pub filtered_demand_permille: u16,
    /// Remaining keep-time hold in milliseconds.
    pub keep_timer_ms: u16,
    /// 5-sample ring buffer for V6510C dynamic zero-point drift detection (`0x08009034`).
    history_mv: [u16; 5],
    history_idx: usize,
    quiet_ticks: u16,
}

impl TorqueSensorState {
    pub const fn new(initial_zero_mv: u16) -> Self {
        Self {
            calibrated_zero_mv: initial_zero_mv,
            filtered_net_mv: 0,
            filtered_demand_permille: 0,
            keep_timer_ms: 0,
            history_mv: [initial_zero_mv; 5],
            history_idx: 0,
            quiet_ticks: 0,
        }
    }

    /// Sets the power-on zero reference recorded during startup calibration.
    pub fn set_power_on_zero(&mut self, zero_mv: u16) {
        self.calibrated_zero_mv = zero_mv;
        self.history_mv = [zero_mv; 5];
    }

    /// Updates the 5-sample dynamic zero-point drift tracker (`0x08009034` in DM01 V6510C)
    /// when the cranks are stationary (`cadence_rpm == 0`).
    pub fn step_zero_drift_tracking(&mut self, raw_torque_mv: u16, cadence_rpm: u16) {
        self.history_mv[self.history_idx] = raw_torque_mv;
        self.history_idx = (self.history_idx + 1) % 5;

        let mut min_v = self.history_mv[0];
        let mut max_v = self.history_mv[0];
        let mut i = 1;
        while i < 5 {
            if self.history_mv[i] < min_v {
                min_v = self.history_mv[i];
            }
            if self.history_mv[i] > max_v {
                max_v = self.history_mv[i];
            }
            i += 1;
        }

        if cadence_rpm == 0 && (max_v - min_v) < 20 && (400..=1000).contains(&raw_torque_mv) {
            self.quiet_ticks = self.quiet_ticks.saturating_add(1);
            if self.quiet_ticks >= 50 {
                // 4:1 weighted IIR update: floor((4 * old_zero + new_sample) / 5)
                self.calibrated_zero_mv =
                    (((self.calibrated_zero_mv as u32) * 4 + (raw_torque_mv as u32)) / 5) as u16;
                self.quiet_ticks = 0;
            }
        } else {
            self.quiet_ticks = 0;
        }
    }

    /// Computes normalized torque demand (`0..=1000` permille) from raw strain-gauge voltage
    /// using the V6510C IIR pedal smoothing filter and `StrengthValue`.
    pub fn update(&mut self, raw_torque_mv: u16, dt_ms: u16, config: &MotorConfig) -> u16 {
        if !config.torque_sensor_enabled || raw_torque_mv < TORQUE_MIN_VALID_MV {
            self.filtered_net_mv = 0;
            self.filtered_demand_permille = 0;
            return 0;
        }

        let zero_mv = self.calibrated_zero_mv;
        let net_torque_mv = raw_torque_mv.saturating_sub(zero_mv);

        // V6510C configurable IIR smoothing filter: F = min(20 * Filter, 200)
        let f_coeff = ((config.torque_filter as u32) * 20).min(200);
        self.filtered_net_mv = (((self.filtered_net_mv as u32) * f_coeff + (net_torque_mv as u32))
            / (f_coeff + 1)) as u16;

        let startup_net_mv = config.torque_startup_net_vol_mv;
        if self.filtered_net_mv >= startup_net_mv && config.torque_max_vol_mv > zero_mv {
            self.keep_timer_ms = config.torque_keep_time_ms.min(5000);
            let span = (config.torque_max_vol_mv - zero_mv) as u32;
            let torque_pct = (((self.filtered_net_mv as u32) * 100) / span).min(100);
            let s_scaled = ((config.torque_strength_value as u32) * 3).min(30);
            // Permille before EAssist% scaling
            self.filtered_demand_permille = ((torque_pct * s_scaled * 10) / 30).min(1000) as u16;
        } else if self.keep_timer_ms > 0 {
            self.keep_timer_ms = self.keep_timer_ms.saturating_sub(dt_ms);
            self.filtered_demand_permille =
                ((self.filtered_demand_permille as u32 * self.keep_timer_ms as u32)
                    / config.torque_keep_time_ms.max(1) as u32) as u16;
        } else {
            self.filtered_demand_permille = 0;
        }

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
