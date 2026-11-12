//! # Battery Calibration Curve, IR Compensation & Low-Voltage Cushion (`V6510C`)
//!
//! Implements the 6-point voltage-to-SoC calibration curve, automatic internal-resistance
//! tracking, a 60-second SoC averaging window, and the `V6510C` low-voltage reserve cushion
//! (`0x08005478`).
//!
//! The State-of-Charge estimate is display-only. Low-Voltage Cutoff (LVC) always evaluates the
//! raw terminal voltage so cell protection is never delayed by the averaging filter.

use crate::power_curve::apply_lvc_cushion_da;

/// Standard nominal battery voltage presets supported by the display and controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryPreset {
    /// 36V nominal (10S Li-ion, 42.0V peak, 31.5V empty).
    V36,
    /// 48V nominal (13S Li-ion, 54.6V peak, 40.95V empty). Default for DM01/DM02.
    V48,
    /// 52V nominal (14S Li-ion, 58.8V peak, 44.1V empty).
    V52,
    /// 60V nominal (16S Li-ion, 67.2V peak, 50.4V empty).
    ///
    /// # Hardware Warning
    /// Deprecated on stock DM01/DM02 hardware. While controller casings were originally marked "24-60V",
    /// 60V refers to the maximum operating ceiling (< 60V peak, i.e. 52V pack). A true 60V pack (67.2V peak)
    /// causes high-side/low-side MOSFET avalanche breakdown under high-current PWM switching inductive spikes.
    V60,
    /// 72V nominal (20S Li-ion, 84.0V peak, 63.0V empty).
    ///
    /// # Hardware Warning
    /// Strictly unsupported on stock DM01/DM02 hardware. Peak charging voltage (84.0V) instantly exceeds
    /// the 63V/80V bulk capacitor ratings and 80V MOSFET breakdown voltage, causing immediate catastrophic failure.
    V72,
    /// Custom user-defined voltage profile (defaults to 48V curve).
    Custom,
}

impl BatteryPreset {
    /// Stable EEPROM encoding. `0` is the 48V default so pre-existing custom EEPROM
    /// images (whose preset byte was zero) keep the original behavior.
    pub const fn to_u8(self) -> u8 {
        match self {
            BatteryPreset::V48 => 0,
            BatteryPreset::V36 => 1,
            BatteryPreset::V52 => 2,
            BatteryPreset::V60 => 3,
            BatteryPreset::V72 => 4,
            BatteryPreset::Custom => 5,
        }
    }

    /// Decodes a persisted EEPROM preset. Unknown values fall back to 48V.
    pub const fn from_u8(value: u8) -> Self {
        match value {
            1 => BatteryPreset::V36,
            2 => BatteryPreset::V52,
            3 => BatteryPreset::V60,
            4 => BatteryPreset::V72,
            5 => BatteryPreset::Custom,
            _ => BatteryPreset::V48,
        }
    }

    /// Nominal pack voltage in millivolts.
    pub const fn nominal_voltage_mv(self) -> u32 {
        match self {
            BatteryPreset::V36 => 36_000,
            BatteryPreset::V48 | BatteryPreset::Custom => 48_000,
            BatteryPreset::V52 => 52_000,
            BatteryPreset::V60 => 60_000,
            BatteryPreset::V72 => 72_000,
        }
    }

    /// Low-voltage cutoff for the preset in millivolts (`3.15 V` per Li-ion cell, rounded
    /// to the controller's 0.1 V grid). 48V keeps the factory `41.0 V` setting.
    pub const fn low_cut_voltage_mv(self) -> u32 {
        match self {
            BatteryPreset::V36 => 31_500,
            BatteryPreset::V48 | BatteryPreset::Custom => 41_000,
            BatteryPreset::V52 => 44_100,
            BatteryPreset::V60 => 50_400,
            BatteryPreset::V72 => 63_000,
        }
    }
}

/// 6-point voltage-to-SoC calibration curve in millivolts (`[0%, 20%, 40%, 60%, 80%, 100%]`).
/// Maps resting open-circuit voltage (OCV) to consumed Watt-hour capacity based on standard
/// Lithium-Ion (NMC/NCA) cell chemistry discharge curves (`[3.15V, 3.45V, 3.65V, 3.85V, 4.00V, 4.15V]` per cell).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryCalibrationCurve {
    /// Voltage thresholds in millivolts for `[0%, 20%, 40%, 60%, 80%, 100%]`.
    pub points_mv: [u32; 6],
}

impl BatteryCalibrationCurve {
    /// Returns the chemistry-accurate Li-ion OCV calibration table for a given battery preset.
    ///
    /// Each curve scales proportionally with consumed Watt-hours across the NMC/NCA discharge plateau:
    /// - `0%`: 3.15 V/cell (safe reserve floor matching controller 41.0V LVC)
    /// - `20%`: 3.45 V/cell (onset of steep waterfall knee)
    /// - `40%`: 3.65 V/cell (lower nominal plateau)
    /// - `60%`: 3.85 V/cell (upper nominal plateau)
    /// - `80%`: 4.00 V/cell
    /// - `100%`: 4.15 V/cell (resting full charge float)
    pub const fn from_preset(preset: BatteryPreset) -> Self {
        match preset {
            BatteryPreset::V36 => Self {
                points_mv: [31_500, 34_500, 36_500, 38_500, 40_000, 41_500],
            },
            BatteryPreset::V48 | BatteryPreset::Custom => Self {
                points_mv: [40_950, 44_850, 47_450, 50_050, 52_000, 53_950],
            },
            BatteryPreset::V52 => Self {
                points_mv: [44_100, 48_300, 51_100, 53_900, 56_000, 58_100],
            },
            BatteryPreset::V60 => Self {
                points_mv: [50_400, 55_200, 58_400, 61_600, 64_000, 66_400],
            },
            BatteryPreset::V72 => Self {
                points_mv: [63_000, 69_000, 73_000, 77_000, 80_000, 83_000],
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

/// Applies the low-voltage reserve cushion to a current target in milliamps.
pub const fn apply_lvc_cushion_ma(current_ma: u32, battery_mv: u32, low_cut_mv: u32) -> u32 {
    let current_da = (current_ma / 100) as u16;
    let bat_dv = (battery_mv / 100) as u16;
    let lvc_dv = (low_cut_mv / 100) as u16;
    (apply_lvc_cushion_da(current_da, bat_dv, lvc_dv) as u32) * 100
}

/// Resistance-estimation window: 100 samples at 100 ms = 10 seconds.
const RESISTANCE_WINDOW_SAMPLES: usize = 100;
/// Sample the instantaneous pack voltage/current every 100 ms.
const RESISTANCE_SAMPLE_INTERVAL_MS: u16 = 100;
/// Recompute the resistance estimate once per second.
const RESISTANCE_ESTIMATE_INTERVAL_MS: u16 = 1_000;
/// Reject estimates above the GUI's supported 350 mΩ range.
const MAX_AUTO_RESISTANCE_MOHM: u16 = 350;
/// Low-current bin: samples at or below 2 A.
const LOW_CURRENT_THRESHOLD_MA: u16 = 2_000;
/// High-current bin: samples at or above 8 A.
const HIGH_CURRENT_THRESHOLD_MA: u16 = 8_000;
/// Require at least this many samples in each bin before estimating.
const MIN_RESISTANCE_SAMPLES: u64 = 4;
/// Require at least this much current swing to produce a usable estimate.
const MIN_CURRENT_SWING_MA: u64 = 5_000;

/// SoC display averaging window in seconds.
const SOC_AVERAGE_WINDOW_SECONDS: usize = 60;
/// Push one averaged OCV sample into the SoC window per second.
const SOC_SAMPLE_INTERVAL_MS: u16 = 1_000;

const fn clamp_u16(value: u32) -> u16 {
    if value > u16::MAX as u32 {
        u16::MAX
    } else {
        value as u16
    }
}

/// Battery telemetry filter that tracks pack resistance, smooths voltage sag, and evaluates
/// low-voltage cutoff hysteresis.
///
/// # Automatic Resistance Tracking
///
/// When `auto_track_resistance` is enabled, the monitor keeps a 10-second window of
/// voltage/current samples. Once the window contains at least [`MIN_RESISTANCE_SAMPLES`]
/// low-current (`<= 2 A`) and high-current (`>= 8 A`) samples, it estimates
/// `R = ΔV / ΔI`, clamps the result to `[10, 350] mΩ`, and slowly averages it into
/// `internal_resistance_mohm`. A starting value of `0 mΩ` disables both compensation and
/// tracking, preserving the documented "0 disables" behavior.
///
/// # SoC Averaging
///
/// The IR-compensated open-circuit estimate is averaged over a 60-second boxcar before the
/// SoC curve is evaluated. This prevents the displayed percentage from jumping under load.
/// LVC still uses the raw terminal voltage.
#[derive(Debug, Clone)]
pub struct BatteryMonitor {
    pub curve: BatteryCalibrationCurve,
    pub filtered_voltage_mv: u32,
    /// Battery internal pack resistance in milliohms (default: `150 mΩ`).
    /// If `0`, dynamic sag compensation and automatic tracking are disabled.
    pub internal_resistance_mohm: u16,
    /// Enables automatic resistance estimation. Initialized from `internal_resistance_mohm != 0`.
    pub auto_track_resistance: bool,
    pub low_voltage_tripped: bool,
    undervoltage_hold_ms: u16,

    resistance_voltage_mv: [u16; RESISTANCE_WINDOW_SAMPLES],
    resistance_current_ma: [u16; RESISTANCE_WINDOW_SAMPLES],
    resistance_index: usize,
    resistance_count: usize,
    resistance_sample_accum_ms: u16,
    resistance_estimate_accum_ms: u16,

    soc_history_mv: [u16; SOC_AVERAGE_WINDOW_SECONDS],
    soc_index: usize,
    soc_sum_mv: u32,
    soc_sample_accum_ms: u16,
}

impl BatteryMonitor {
    /// Creates a battery monitor with default 150 mΩ pack resistance and automatic tracking enabled.
    pub fn new(preset: BatteryPreset, initial_voltage_mv: u32) -> Self {
        Self::with_resistance(preset, initial_voltage_mv, 150)
    }

    /// Creates a battery monitor with a specific internal pack resistance in milliohms.
    /// A value of `0` disables both IR compensation and automatic tracking.
    pub fn with_resistance(
        preset: BatteryPreset,
        initial_voltage_mv: u32,
        resistance_mohm: u16,
    ) -> Self {
        let initial_v = clamp_u16(initial_voltage_mv);
        Self {
            curve: BatteryCalibrationCurve::from_preset(preset),
            filtered_voltage_mv: initial_voltage_mv,
            internal_resistance_mohm: resistance_mohm,
            auto_track_resistance: resistance_mohm != 0,
            low_voltage_tripped: false,
            undervoltage_hold_ms: 0,
            resistance_voltage_mv: [0; RESISTANCE_WINDOW_SAMPLES],
            resistance_current_ma: [0; RESISTANCE_WINDOW_SAMPLES],
            resistance_index: 0,
            resistance_count: 0,
            resistance_sample_accum_ms: 0,
            resistance_estimate_accum_ms: 0,
            soc_history_mv: [initial_v; SOC_AVERAGE_WINDOW_SECONDS],
            soc_index: 0,
            soc_sum_mv: initial_v as u32 * SOC_AVERAGE_WINDOW_SECONDS as u32,
            soc_sample_accum_ms: 0,
        }
    }

    /// Updates the resistance estimate, SoC average, and low-voltage cutoff state.
    ///
    /// `dt_ms` is the time since the previous call. The firmware calls this from the 1 ms
    /// control loop, but the estimator uses accumulated time so slower callers work too.
    pub fn update(
        &mut self,
        raw_voltage_mv: u32,
        low_cut_mv: u32,
        load_current_ma: u32,
        dt_ms: u16,
    ) -> u8 {
        let sample_voltage_mv = clamp_u16(raw_voltage_mv);
        let sample_current_ma = clamp_u16(load_current_ma);

        self.update_resistance_estimate(sample_voltage_mv, sample_current_ma, dt_ms);

        // 1. EggRider-style IR voltage sag compensation: V_ocv = V_terminal + I * R
        let sag_mv =
            ((load_current_ma as u64 * self.internal_resistance_mohm as u64) / 1_000) as u32;
        let est_ocv_mv = raw_voltage_mv.saturating_add(sag_mv);

        // 2. Average the compensated OCV over 60 seconds before evaluating SoC.
        self.update_soc_average(est_ocv_mv, dt_ms);

        // 3. Low-voltage cutoff debounce operates on actual terminal voltage to safeguard cells.
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

    fn update_soc_average(&mut self, est_ocv_mv: u32, dt_ms: u16) {
        self.soc_sample_accum_ms = self.soc_sample_accum_ms.saturating_add(dt_ms);
        if self.soc_sample_accum_ms >= SOC_SAMPLE_INTERVAL_MS {
            self.soc_sample_accum_ms = 0;
            let sample = clamp_u16(est_ocv_mv);
            self.soc_sum_mv =
                self.soc_sum_mv - self.soc_history_mv[self.soc_index] as u32 + sample as u32;
            self.soc_history_mv[self.soc_index] = sample;
            self.soc_index = (self.soc_index + 1) % SOC_AVERAGE_WINDOW_SECONDS;
        }
        self.filtered_voltage_mv = self.soc_sum_mv / SOC_AVERAGE_WINDOW_SECONDS as u32;
    }

    fn update_resistance_estimate(&mut self, voltage_mv: u16, current_ma: u16, dt_ms: u16) {
        if !self.auto_track_resistance {
            return;
        }

        self.resistance_sample_accum_ms = self.resistance_sample_accum_ms.saturating_add(dt_ms);
        if self.resistance_sample_accum_ms >= RESISTANCE_SAMPLE_INTERVAL_MS {
            self.resistance_sample_accum_ms = 0;
            self.resistance_voltage_mv[self.resistance_index] = voltage_mv;
            self.resistance_current_ma[self.resistance_index] = current_ma;
            self.resistance_index = (self.resistance_index + 1) % RESISTANCE_WINDOW_SAMPLES;
            if self.resistance_count < RESISTANCE_WINDOW_SAMPLES {
                self.resistance_count += 1;
            }
        }

        self.resistance_estimate_accum_ms = self.resistance_estimate_accum_ms.saturating_add(dt_ms);
        if self.resistance_estimate_accum_ms < RESISTANCE_ESTIMATE_INTERVAL_MS {
            return;
        }
        self.resistance_estimate_accum_ms = 0;

        let mut low_count = 0u64;
        let mut high_count = 0u64;
        let mut low_voltage_sum = 0u64;
        let mut high_voltage_sum = 0u64;
        let mut low_current_sum = 0u64;
        let mut high_current_sum = 0u64;

        for idx in 0..self.resistance_count {
            let current_ma = self.resistance_current_ma[idx] as u64;
            let voltage_mv = self.resistance_voltage_mv[idx] as u64;
            if current_ma <= LOW_CURRENT_THRESHOLD_MA as u64 {
                low_count += 1;
                low_voltage_sum += voltage_mv;
                low_current_sum += current_ma;
            } else if current_ma >= HIGH_CURRENT_THRESHOLD_MA as u64 {
                high_count += 1;
                high_voltage_sum += voltage_mv;
                high_current_sum += current_ma;
            }
        }

        if low_count < MIN_RESISTANCE_SAMPLES || high_count < MIN_RESISTANCE_SAMPLES {
            return;
        }

        let low_voltage_avg = low_voltage_sum / low_count;
        let high_voltage_avg = high_voltage_sum / high_count;
        let low_current_avg = low_current_sum / low_count;
        let high_current_avg = high_current_sum / high_count;

        let delta_v_mv = low_voltage_avg.saturating_sub(high_voltage_avg);
        let delta_i_ma = high_current_avg.saturating_sub(low_current_avg);
        if delta_v_mv == 0 || delta_i_ma < MIN_CURRENT_SWING_MA {
            return;
        }

        // V(mV) / I(mA) * 1000 = R(mΩ)
        let estimate_mohm =
            ((delta_v_mv * 1_000) / delta_i_ma).clamp(10, MAX_AUTO_RESISTANCE_MOHM as u64) as u16;

        // Slow 3/4 EMA: the 10-second window already averages 100 samples, so this mainly
        // rejects occasional bad estimates without making the tracker sluggish.
        let averaged = (self.internal_resistance_mohm as u32 * 3 + estimate_mohm as u32) / 4;
        self.internal_resistance_mohm = averaged.min(MAX_AUTO_RESISTANCE_MOHM as u32) as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_48v_soc_interpolation() {
        let curve = BatteryCalibrationCurve::from_preset(BatteryPreset::V48);
        assert_eq!(curve.state_of_charge_percent(40_000), 0);
        assert_eq!(curve.state_of_charge_percent(40_950), 0);
        assert_eq!(curve.state_of_charge_percent(47_450), 40);
        assert_eq!(curve.state_of_charge_percent(50_050), 60);
        assert_eq!(curve.state_of_charge_percent(52_000), 80);
        assert_eq!(curve.state_of_charge_percent(53_950), 100);
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
        // Fixed 150 mΩ compensation with automatic tracking disabled.
        let mut mon = BatteryMonitor::with_resistance(BatteryPreset::V48, 48_000, 150);
        mon.auto_track_resistance = false;

        // Fill the 60 s window at rest.
        for _ in 0..60 {
            mon.update(48_000, 39_000, 0, 1_000);
        }
        assert_eq!(mon.filtered_voltage_mv, 48_000);

        // 20 A load: 150 mΩ * 20 A = 3.0 V sag. Terminal 45.0 V + 3.0 V = 48.0 V OCV.
        for _ in 0..60 {
            mon.update(45_000, 39_000, 20_000, 1_000);
        }
        assert_eq!(mon.filtered_voltage_mv, 48_000);

        // Without IR compensation, the same sag pulls the average down to 45.0 V.
        let mut uncompensated = BatteryMonitor::with_resistance(BatteryPreset::V48, 48_000, 0);
        for _ in 0..60 {
            uncompensated.update(45_000, 39_000, 20_000, 1_000);
        }
        assert!(uncompensated.filtered_voltage_mv < 46_000);
    }

    #[test]
    fn test_auto_resistance_tracking() {
        let mut mon = BatteryMonitor::with_resistance(BatteryPreset::V48, 48_000, 150);
        assert!(mon.auto_track_resistance);

        // 5 s at rest, then 5 s at 20 A with a 150 mΩ pack (3.0 V sag).
        for _ in 0..5_000 {
            mon.update(48_000, 39_000, 0, 1);
        }
        for _ in 0..5_000 {
            mon.update(45_000, 39_000, 20_000, 1);
        }

        assert!(
            (140..=160).contains(&mon.internal_resistance_mohm),
            "expected ~150 mΩ, got {}",
            mon.internal_resistance_mohm
        );
    }

    #[test]
    fn test_auto_resistance_disabled_at_zero() {
        let mut mon = BatteryMonitor::with_resistance(BatteryPreset::V48, 48_000, 0);
        assert!(!mon.auto_track_resistance);

        for _ in 0..5_000 {
            mon.update(48_000, 39_000, 0, 1);
        }
        for _ in 0..5_000 {
            mon.update(45_000, 39_000, 20_000, 1);
        }

        assert_eq!(mon.internal_resistance_mohm, 0);
    }

    #[test]
    fn test_auto_resistance_is_bounded() {
        let mut mon = BatteryMonitor::with_resistance(BatteryPreset::V48, 48_000, 150);

        // 5 s at rest, then 8 s at 20 A with an implied 1100 mΩ (22 V sag).
        for _ in 0..5_000 {
            mon.update(48_000, 39_000, 0, 1);
        }
        for _ in 0..8_000 {
            mon.update(26_000, 39_000, 20_000, 1);
        }

        assert!(mon.internal_resistance_mohm <= MAX_AUTO_RESISTANCE_MOHM);
        assert!(
            mon.internal_resistance_mohm > 250,
            "estimator should converge toward the 350 mΩ clamp, got {}",
            mon.internal_resistance_mohm
        );
    }

    #[test]
    fn test_soc_average_window() {
        let mut mon = BatteryMonitor::with_resistance(BatteryPreset::V48, 48_000, 0);

        // Halfway through the 60 s window: 30 samples at 44.0 V, 30 at 48.0 V.
        for _ in 0..30 {
            mon.update(44_000, 39_000, 0, 1_000);
        }
        assert_eq!(mon.filtered_voltage_mv, 46_000);

        // Full window: 60 samples at 44.0 V.
        for _ in 0..30 {
            mon.update(44_000, 39_000, 0, 1_000);
        }
        assert_eq!(mon.filtered_voltage_mv, 44_000);
    }

    #[test]
    fn test_battery_preset_eeprom_encoding() {
        // 0 must remain the 48V default for old EEPROM images whose preset byte was zero.
        assert_eq!(BatteryPreset::V48.to_u8(), 0);
        assert_eq!(BatteryPreset::from_u8(0), BatteryPreset::V48);
        assert_eq!(BatteryPreset::from_u8(1), BatteryPreset::V36);
        assert_eq!(BatteryPreset::from_u8(2), BatteryPreset::V52);
        assert_eq!(BatteryPreset::from_u8(0xFF), BatteryPreset::V48);

        for preset in [
            BatteryPreset::V36,
            BatteryPreset::V48,
            BatteryPreset::V52,
            BatteryPreset::V60,
            BatteryPreset::V72,
            BatteryPreset::Custom,
        ] {
            assert_eq!(BatteryPreset::from_u8(preset.to_u8()), preset);
        }

        assert_eq!(BatteryPreset::V36.nominal_voltage_mv(), 36_000);
        assert_eq!(BatteryPreset::V52.nominal_voltage_mv(), 52_000);
        assert_eq!(BatteryPreset::V36.low_cut_voltage_mv(), 31_500);
        assert_eq!(BatteryPreset::V48.low_cut_voltage_mv(), 41_000);
        assert_eq!(BatteryPreset::V52.low_cut_voltage_mv(), 44_100);
    }
}
