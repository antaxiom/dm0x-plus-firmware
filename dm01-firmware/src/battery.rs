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

/// Battery telemetry filter that smooths transient voltage sag under load and tracks low-voltage cutoff hysteresis.
#[derive(Debug, Clone)]
pub struct BatteryMonitor {
    pub curve: BatteryCalibrationCurve,
    pub filtered_voltage_mv: u32,
    pub low_voltage_tripped: bool,
    undervoltage_hold_ms: u16,
}

impl BatteryMonitor {
    pub fn new(preset: BatteryPreset, initial_voltage_mv: u32) -> Self {
        Self {
            curve: BatteryCalibrationCurve::from_preset(preset),
            filtered_voltage_mv: initial_voltage_mv,
            low_voltage_tripped: false,
            undervoltage_hold_ms: 0,
        }
    }

    /// Updates filtered battery voltage and evaluates low-voltage cutoff with a 500ms debounce
    /// to avoid tripping on instantaneous inrush transients.
    pub fn update(&mut self, raw_voltage_mv: u32, low_cut_mv: u32, dt_ms: u16) -> u8 {
        if self.filtered_voltage_mv == 0 {
            self.filtered_voltage_mv = raw_voltage_mv;
        } else {
            // Exponential moving average (15/16 previous + 1/16 sample)
            self.filtered_voltage_mv =
                ((self.filtered_voltage_mv as u64 * 15 + raw_voltage_mv as u64) / 16) as u32;
        }

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
        // Brief 200ms sag below 39V should not trip cutoff
        mon.update(38_000, 39_000, 200);
        assert!(!mon.low_voltage_tripped);

        // Sustained 600ms sag below 39V trips cutoff
        mon.update(38_000, 39_000, 400);
        assert!(mon.low_voltage_tripped);

        // Rebounding above 40.5V clears cutoff
        mon.update(41_000, 39_000, 100);
        assert!(!mon.low_voltage_tripped);
    }
}

