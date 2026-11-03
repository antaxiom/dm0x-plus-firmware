//! # Battery Calibration Curve & Low-Voltage Cushion (`V6510C`)
//!
//! Implements the 6-point voltage-to-SoC calibration curve, transient sag filtering,
//! and the decompiled `V6510C` 3.0V low-voltage reserve cushion (`0x08005478`).

use crate::power_curve::apply_lvc_cushion_da;

/// Standard nominal battery voltage presets supported by the display and controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryPreset {
    V36,
    V48,
    V52,
    V60,
    V72,
    Custom,
}

/// 6-point voltage-to-SoC calibration curve in millivolts (`[0%, 20%, 40%, 60%, 80%, 100%]`).
/// Matches the hidden battery calibration table in the ToSeven Display Settings menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryCalibrationCurve {
    /// Voltage thresholds in millivolts for `[0%, 20%, 40%, 60%, 80%, 100%]`.
    pub points_mv: [u32; 6],
}

impl BatteryCalibrationCurve {
    /// Returns the documented linear voltage calibration table for a given battery preset.
    pub const fn from_preset(preset: BatteryPreset) -> Self {
        match preset {
            BatteryPreset::V36 => Self {
                points_mv: [30_000, 32_200, 34_400, 36_600, 38_800, 41_000],
            },
            BatteryPreset::V48 | BatteryPreset::Custom => Self {
                points_mv: [39_000, 41_900, 44_700, 47_600, 50_400, 53_300],
            },
            BatteryPreset::V52 => Self {
                points_mv: [42_000, 45_100, 48_200, 51_200, 54_300, 57_400],
            },
            BatteryPreset::V60 => Self {
                points_mv: [48_000, 51_500, 55_000, 58_600, 62_100, 65_600],
            },
            BatteryPreset::V72 => Self {
                points_mv: [62_000, 66_000, 70_000, 74_000, 78_000, 82_000],
            },
        }
    }

    /// Computes State of Charge (0..=100%) using piecewise-linear interpolation across the 6 calibration points.
    pub fn state_of_charge_percent(&self, voltage_mv: u32) -> u8 {
        if voltage_mv <= self.points_mv[0] {
            return 0;
        }
        if voltage_mv >= self.points_mv[5] {
            return 100;
        }

        for i in 0..5 {
            let v_low = self.points_mv[i];
            let v_high = self.points_mv[i + 1];
            if voltage_mv >= v_low && voltage_mv <= v_high {
                let base_pct = (i as u32) * 20;
                let span = v_high.saturating_sub(v_low).max(1);
                let frac = ((voltage_mv - v_low) * 20) / span;
                return (base_pct + frac).min(100) as u8;
            }
        }

        0
    }
}

/// Applies the `V6510C` 3.0V Low-Voltage Cushion (`0x08005478`) to a current target in milliamps.
pub const fn apply_lvc_cushion_ma(current_ma: u32, battery_mv: u32, low_cut_mv: u32) -> u32 {
    let current_da = (current_ma / 100) as u16;
    let bat_dv = (battery_mv / 100) as u16;
    let lvc_dv = (low_cut_mv / 100) as u16;
    (apply_lvc_cushion_da(current_da, bat_dv, lvc_dv) as u32) * 100
}

/// Battery telemetry filter that smooths transient voltage sag under load and tracks low-voltage cutoff hysteresis.
#[derive(Debug, Clone)]
pub struct BatteryMonitor {
    pub curve: BatteryCalibrationCurve,
    pub filtered_voltage_mv: u32,
    /// Battery internal pack resistance in milliohms for EggRider-grade voltage sag compensation (default: `150 mΩ`).
    /// If 0, dynamic sag compensation is disabled.
    pub internal_resistance_mohm: u16,
    pub low_voltage_tripped: bool,
    undervoltage_hold_ms: u16,
}

impl BatteryMonitor {
    /// Creates a battery monitor with default 150 mΩ pack resistance.
    pub fn new(preset: BatteryPreset, initial_voltage_mv: u32) -> Self {
        Self::with_resistance(preset, initial_voltage_mv, 150)
    }

    /// Creates a battery monitor with a specific internal pack resistance in milliohms.
    pub fn with_resistance(
        preset: BatteryPreset,
        initial_voltage_mv: u32,
        resistance_mohm: u16,
    ) -> Self {
        Self {
            curve: BatteryCalibrationCurve::from_preset(preset),
            filtered_voltage_mv: initial_voltage_mv,
            internal_resistance_mohm: resistance_mohm,
            low_voltage_tripped: false,
            undervoltage_hold_ms: 0,
        }
    }

    /// Updates filtered battery voltage and evaluates low-voltage cutoff with a 500ms debounce
    /// to avoid tripping on instantaneous inrush transients.
    ///
    /// Implements Option 3 (Tuned Hybrid):
    /// 1. Instantaneous EggRider-grade IR sag cancellation:
    ///    `V_est_ocv = V_terminal + (I_load * R_pack)`
    /// 2. Light Exponential Moving Average (1023/1024 previous + 1/1024 sample):
    ///    At 1 kHz (dt = 1 ms), time constant τ = 1024 ms ≈ 1.02s (95% settling time ≈ 3.1s),
    ///    smoothing inverter current ripple and chemical diffusion relaxation without sluggish display lag.
    pub fn update(
        &mut self,
        raw_voltage_mv: u32,
        low_cut_mv: u32,
        load_current_ma: u32,
        dt_ms: u16,
    ) -> u8 {
        // 1. EggRider IR Voltage Sag Compensation
        let sag_mv =
            ((load_current_ma as u64 * self.internal_resistance_mohm as u64) / 1000) as u32;
        let est_ocv_mv = raw_voltage_mv.saturating_add(sag_mv);

        // 2. Light Exponential Moving Average (τ ≈ 1.02s at 1 kHz)
        if self.filtered_voltage_mv == 0 {
            self.filtered_voltage_mv = est_ocv_mv;
        } else {
            self.filtered_voltage_mv =
                (((self.filtered_voltage_mv as u64) * 1023 + est_ocv_mv as u64) / 1024) as u32;
        }

        // 3. Low-voltage cutoff debounce operates on actual terminal voltage to safeguard cells
        if raw_voltage_mv < low_cut_mv {
            self.undervoltage_hold_ms = self.undervoltage_hold_ms.saturating_add(dt_ms);
            if self.undervoltage_hold_ms >= 500 {
                self.low_voltage_tripped = true;
            }
        } else {
            self.undervoltage_hold_ms = 0;
            // Recover once voltage rebounds 1.5V above low_cut_mv
            if self.low_voltage_tripped && raw_voltage_mv >= low_cut_mv.saturating_add(1_500) {
                self.low_voltage_tripped = false;
            }
        }

        self.curve.state_of_charge_percent(self.filtered_voltage_mv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_48v_soc_interpolation() {
        let curve = BatteryCalibrationCurve::from_preset(BatteryPreset::V48);
        assert_eq!(curve.state_of_charge_percent(38_500), 0);
        assert_eq!(curve.state_of_charge_percent(39_000), 0);
        assert_eq!(curve.state_of_charge_percent(47_600), 60);
        assert_eq!(curve.state_of_charge_percent(53_300), 100);
        assert_eq!(curve.state_of_charge_percent(54_600), 100);
    }

    #[test]
    fn test_low_voltage_debounce_and_recovery() {
        let mut mon = BatteryMonitor::new(BatteryPreset::V48, 48_000);
        mon.update(38_000, 39_000, 0, 200);
        assert!(!mon.low_voltage_tripped);

        mon.update(38_000, 39_000, 0, 400);
        assert!(mon.low_voltage_tripped);

        mon.update(41_000, 39_000, 0, 100);
        assert!(!mon.low_voltage_tripped);
    }

    #[test]
    fn test_ir_sag_compensation_and_smoothing() {
        // 48V pack at 48.0V resting
        let mut mon = BatteryMonitor::with_resistance(BatteryPreset::V48, 48_000, 150);
        assert_eq!(mon.internal_resistance_mohm, 150);

        // Under 20A (20,000 mA) load with 150 mΩ pack resistance:
        // sag = (20,000 * 150) / 1000 = 3000 mV (3.0 V).
        // If terminal voltage drops by 3.0V to 45,000 mV:
        // est_ocv = 45,000 + 3,000 = 48,000 mV (zero drop in estimated OCV!)
        let initial_soc = mon.update(48_000, 39_000, 0, 1);
        let under_load_soc = mon.update(45_000, 39_000, 20_000, 1);
        assert_eq!(initial_soc, under_load_soc);

        // When IR compensation is disabled (0 mΩ), sag directly drags down filtered voltage
        let mut uncomp_mon = BatteryMonitor::with_resistance(BatteryPreset::V48, 48_000, 0);
        uncomp_mon.update(48_000, 39_000, 0, 1);
        // After 2000 ticks (2 seconds) of sagged 45.0V without IR compensation:
        for _ in 0..2000 {
            uncomp_mon.update(45_000, 39_000, 20_000, 1);
        }
        assert!(uncomp_mon.filtered_voltage_mv < 46_000);
    }
}
