//! # Exact Decompiled V6510C Power Curve & Control Pipeline
//!
//! This module implements the bit-accurate integer math of the ToSeven DM01/DM02 `V6510C`
//! power curve pipeline in units matching the MCU firmware:
//! - Currents in **deci-amps (`dA`, `0.1 A`)**: e.g., `300 = 30.0 A`, `180 = 18.0 A`, `100 = 10.0 A`.
//! - Voltages in **millivolts (`mV`)** for sensors and **deci-volts (`dV`, `0.1 V`)** for battery bus (`500 = 50.0 V`).
//! - Speeds in **deci-km/h (`0.1 km/h`)**: e.g., `250 = 25.0 km/h`, `60 = 6.0 km/h`.
//! - Temperatures in **degrees Celsius (`°C`)**.

/// Absolute hardware current limit (`400 dA = 40.0 A`) enforced by `V6510C`.
pub const HARDWARE_MAX_CURRENT_DA: u16 = 400;

/// Nominal full-current reference battery voltage (`500 dV = 50.0 V`).
/// Above `50.0 V`, maximum current is scaled by `500 / battery_dv` to hold constant max power.
pub const CONSTANT_POWER_REF_VOLT_DV: u16 = 500;

/// Stage 1B: Continuous Linear Thermal Rollback (`0x0800373C` in DM01 V6510C).
///
/// Linearly scales the thermal current ceiling from `sport_max_da` (`300 dA = 30.0 A`) at
/// `temp_min_c` (`70°C`) down to `0 dA` at `temp_max_c` (`100°C`):
/// ```text
/// I_therm = I_sport - floor(((T - T_min) * I_sport) / (T_max - T_min))
/// ```
pub const fn compute_thermal_current_limit_da(
    temp_c: i16,
    temp_min_c: i16,
    temp_max_c: i16,
    sport_max_da: u16,
) -> u16 {
    if temp_c <= temp_min_c || temp_max_c <= temp_min_c {
        return sport_max_da;
    }
    if temp_c >= temp_max_c {
        return 0;
    }
    let delta_t = (temp_c - temp_min_c) as u32;
    let span_t = (temp_max_c - temp_min_c) as u32;
    let reduction = (delta_t * (sport_max_da as u32)) / span_t;
    sport_max_da.saturating_sub(reduction as u16)
}

/// Stage 1C: High-Voltage Constant-Power Current Cap (`0x080037F4` in DM01 V6510C).
///
/// Above `50.0 V` (`500 dV`), caps current to `floor(mode_max_da * 500 / battery_dv)`
/// so electrical input power never exceeds `50.0 V * mode_max_da` (`1500 W` at `30.0 A`).
pub const fn compute_voltage_capped_current_da(mode_max_da: u16, battery_dv: u16) -> u16 {
    let clamped_mode = if mode_max_da > HARDWARE_MAX_CURRENT_DA {
        HARDWARE_MAX_CURRENT_DA
    } else {
        mode_max_da
    };
    if battery_dv <= CONSTANT_POWER_REF_VOLT_DV {
        clamped_mode
    } else {
        (((clamped_mode as u32) * (CONSTANT_POWER_REF_VOLT_DV as u32)) / (battery_dv as u32)) as u16
    }
}

/// Stage 1 Global Mode Ceiling (`0x08003FD4` in DM01 V6510C).
///
/// Used as the current ceiling for the **Torque Sensor** curve.
/// Ignores the assist level's `Pwr%` setting and applies only the Riding Mode limit,
/// linear thermal rollback, and high-voltage constant-power cap.
pub const fn compute_global_mode_ceiling_da(
    mode_max_da: u16,
    sport_max_da: u16,
    temp_c: i16,
    temp_min_c: i16,
    temp_max_c: i16,
    battery_dv: u16,
) -> u16 {
    let mode_cap = if mode_max_da > HARDWARE_MAX_CURRENT_DA {
        HARDWARE_MAX_CURRENT_DA
    } else {
        mode_max_da
    };
    let therm_cap = compute_thermal_current_limit_da(temp_c, temp_min_c, temp_max_c, sport_max_da);
    let vbat_cap = compute_voltage_capped_current_da(mode_cap, battery_dv);

    let mut min_cap = mode_cap;
    if therm_cap < min_cap {
        min_cap = therm_cap;
    }
    if vbat_cap < min_cap {
        min_cap = vbat_cap;
    }
    min_cap
}

/// Stage 1 Level Power Ceiling (`0x08003814` in DM01 V6510C).
///
/// Used as the current ceiling for **Cadence (PAS)** and **Throttle** curves.
/// Scales the Riding Mode ceiling by the assist level's `Pwr%` (`level_pwr_pct`),
/// then applies linear thermal rollback and the high-voltage constant-power cap.
pub const fn compute_level_power_ceiling_da(
    mode_max_da: u16,
    sport_max_da: u16,
    level_pwr_pct: u8,
    temp_c: i16,
    temp_min_c: i16,
    temp_max_c: i16,
    battery_dv: u16,
) -> u16 {
    let mode_cap = if mode_max_da > HARDWARE_MAX_CURRENT_DA {
        HARDWARE_MAX_CURRENT_DA
    } else {
        mode_max_da
    };
    let pwr_pct = if level_pwr_pct > 100 {
        100
    } else {
        level_pwr_pct as u32
    };
    let level_scaled_da = (((mode_cap as u32) * pwr_pct) / 100) as u16;
    let therm_cap = compute_thermal_current_limit_da(temp_c, temp_min_c, temp_max_c, sport_max_da);
    let vbat_cap = compute_voltage_capped_current_da(mode_cap, battery_dv);

    let mut min_cap = level_scaled_da;
    if therm_cap < min_cap {
        min_cap = therm_cap;
    }
    if vbat_cap < min_cap {
        min_cap = vbat_cap;
    }
    min_cap
}

/// Stage 2A: Stateful Torque Sensor Processor (`0x08009034`, `0x0800912C`, `0x08008024` in DM01 V6510C).
///
/// Implements:
/// 1. 12-bit ADC (`0..4095`) to millivolt (`0..3300 mV`) conversion.
/// 2. 5-sample moving delta window (`0x08009034`) for automatic resting `ZeroVol` thermal drift tracking
///    (`4:1` weighted average update when unloaded within `400..1000 mV` for 50 consecutive quiet ticks).
/// 3. Configurable IIR pedal force smoothing filter (`F = min(20 * filter_setting, 200)`).
/// 4. `StrengthValue * EAssist[level]` amplification and `CurrentOffset` starting kick.
/// 5. Linear cadence scaling relative to `60 RPM` (`max(cadence_rpm, 5) / 60`), clamped to `global_mode_ceiling_da`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorqueSensorState {
    /// Active resting zero-point voltage in millivolts (default `700 mV`).
    pub zero_vol_mv: u16,
    /// Maximum full-scale pedal voltage in millivolts (default `3300 mV`).
    pub max_vol_mv: u16,
    /// Minimum filtered net voltage above `zero_vol_mv` to trigger assist (default `50 mV`).
    pub startup_net_vol_mv: u16,
    /// Filtered net pedal voltage (`V_pedal - V_zero`) in millivolts.
    pub filtered_net_mv: u16,
    /// Normalized pedal effort percentage (`0..=100%`).
    pub torque_pct: u8,
    /// Ring buffer of the last 5 raw pedal voltage readings (in mV) for drift detection.
    history_mv: [u16; 5],
    history_idx: usize,
    quiet_ticks: u16,
}

impl TorqueSensorState {
    pub const fn new(zero_vol_mv: u16, max_vol_mv: u16, startup_net_vol_mv: u16) -> Self {
        Self {
            zero_vol_mv,
            max_vol_mv,
            startup_net_vol_mv,
            filtered_net_mv: 0,
            torque_pct: 0,
            history_mv: [zero_vol_mv; 5],
            history_idx: 0,
            quiet_ticks: 0,
        }
    }

    /// Converts a raw 12-bit ADC reading (`0..4095`) to millivolts (`0..3300 mV`).
    pub const fn adc12_to_mv(adc_raw: u16) -> u16 {
        let clamped = if adc_raw > 4095 { 4095 } else { adc_raw as u32 };
        ((clamped * 3300) / 4095) as u16
    }

    /// Updates automatic `ZeroVol` drift tracking (`0x08009034`) and the IIR pedal filter (`0x0800912C`).
    pub fn update_voltage(&mut self, pedal_mv: u16, cadence_rpm: u16, filter_setting: u8) {
        // 1. Record sample in 5-element ring buffer and compute max pairwise delta
        self.history_mv[self.history_idx] = pedal_mv;
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
        let max_delta = max_v - min_v;

        // 2. Dynamic zero-point drift tracking when cranks are stationary (0 RPM)
        // and signal is steady (delta < 20 mV) within the safe [400 mV, 1000 mV] window.
        if cadence_rpm == 0 && max_delta < 20 && (400..=1000).contains(&pedal_mv) {
            self.quiet_ticks = self.quiet_ticks.saturating_add(1);
            if self.quiet_ticks >= 50 {
                // 4:1 weighted IIR update: floor((4 * old_zero + new_sample) / 5)
                self.zero_vol_mv = (((self.zero_vol_mv as u32) * 4 + (pedal_mv as u32)) / 5) as u16;
                self.quiet_ticks = 0;
            }
        } else {
            self.quiet_ticks = 0;
        }

        // 3. Net pedal voltage and configurable IIR low-pass filter
        let raw_net_mv = pedal_mv.saturating_sub(self.zero_vol_mv);
        let f_coeff = ((filter_setting as u32) * 20).min(200);
        self.filtered_net_mv = (((self.filtered_net_mv as u32) * f_coeff + (raw_net_mv as u32))
            / (f_coeff + 1)) as u16;

        // 4. Startup deadband gate and normalization to 0..100%
        if self.filtered_net_mv < self.startup_net_vol_mv || self.max_vol_mv <= self.zero_vol_mv {
            self.torque_pct = 0;
        } else {
            let span = (self.max_vol_mv - self.zero_vol_mv) as u32;
            let pct = ((self.filtered_net_mv as u32) * 100) / span;
            self.torque_pct = pct.min(100) as u8;
        }
    }

    /// Computes the Torque Sensor target current in deci-amps (`0x08008024` in DM01 V6510C).
    #[allow(clippy::too_many_arguments)]
    pub fn compute_current_da(
        &mut self,
        pedal_mv: u16,
        cadence_rpm: u16,
        filter_setting: u8,
        strength_value: u8,
        level_eassist_pct: u8,
        current_offset_da: u16,
        mode_max_da: u16,
        global_mode_ceiling_da: u16,
    ) -> u16 {
        self.compute_current_da_with_profile(
            pedal_mv,
            cadence_rpm,
            filter_setting,
            strength_value,
            level_eassist_pct,
            current_offset_da,
            mode_max_da,
            global_mode_ceiling_da,
            global_mode_ceiling_da,
            &crate::tuning::TuningProfile::stock_v6510c(),
        )
    }

    /// Computes torque sensor current demand using a specified [`crate::tuning::TuningProfile`].
    #[allow(clippy::too_many_arguments)]
    pub fn compute_current_da_with_profile(
        &mut self,
        pedal_mv: u16,
        cadence_rpm: u16,
        filter_setting: u8,
        strength_value: u8,
        level_eassist_pct: u8,
        current_offset_da: u16,
        mode_max_da: u16,
        global_mode_ceiling_da: u16,
        level_power_ceiling_da: u16,
        profile: &crate::tuning::TuningProfile,
    ) -> u16 {
        self.update_voltage(pedal_mv, cadence_rpm, filter_setting);

        if self.torque_pct == 0 || cadence_rpm == 0 || level_eassist_pct == 0 {
            return 0;
        }

        // 1. Scale StrengthValue: S_scaled = min(3 * StrengthValue, 30)
        let s_scaled = ((strength_value as u32) * 3).min(30);

        // 2. Combined Effort Multiplier r4 = min(floor(floor(torque_pct * S_scaled * EAssist / 10) / 100), 100)
        let scaled_effort = ((self.torque_pct as u32) * s_scaled * (level_eassist_pct as u32)) / 10;
        let raw_pct = (scaled_effort / 100).min(100) as u16;

        let combined_effort_pct = match profile.torque_response {
            crate::tuning::TorqueResponse::StockLinear => raw_pct as u32,
            other => (other.shape_effort(raw_pct * 10) / 10) as u32,
        };

        // 3. Base Torque Current = floor(combined_effort_pct * mode_max_da / 100) + current_offset_da
        let mode_cap = mode_max_da.min(HARDWARE_MAX_CURRENT_DA) as u32;
        let base_current_da = ((combined_effort_pct * mode_cap) / 100) + (current_offset_da as u32);

        // 4. Cadence Scaling via profile
        let cadence_scaled_da = match profile.cadence_scaling {
            crate::tuning::CadenceScaling::Stock60Rpm => {
                let eff = (cadence_rpm.max(5)) as u32;
                ((base_current_da * eff) / 60) as u16
            }
            other => {
                let mult_q8 = other.compute_multiplier_q8(cadence_rpm) as u32;
                ((base_current_da * mult_q8) / 256) as u16
            }
        };

        // 5. Ceiling Enforcement Policy
        let ceiling = match profile.ceiling_policy {
            crate::tuning::AssistCeilingPolicy::StockDualCeiling => global_mode_ceiling_da,
            crate::tuning::AssistCeilingPolicy::UnifiedLevelCeiling => {
                global_mode_ceiling_da.min(level_power_ceiling_da)
            }
        };

        cadence_scaled_da.min(ceiling)
    }
}

/// Stage 2B: Cadence / PAS Power Curve (`0x080067D4` in DM01 V6510C).
///
/// 1. Clamps `cadence_rpm` to `[10, pas_max_speed_rpm]` (where `pas_max_speed_rpm` defaults to `60 RPM`).
/// 2. Computes `floor(global_mode_ceiling_da * clamped_cadence / pas_max_speed_rpm) + pas_current_offset_da`.
/// 3. Clamps the result to `level_power_ceiling_da` (`Pwr[level]`).
pub const fn compute_pas_current_da(
    cadence_rpm: u16,
    pas_active: bool,
    pas_max_speed_rpm: u16,
    pas_current_offset_da: u16,
    global_mode_ceiling_da: u16,
    level_power_ceiling_da: u16,
) -> u16 {
    if !pas_active || cadence_rpm == 0 || level_power_ceiling_da == 0 {
        return 0;
    }

    let max_cad = if pas_max_speed_rpm < 15 {
        60
    } else {
        pas_max_speed_rpm
    };
    let clamped_cad = if cadence_rpm < 10 {
        10
    } else if cadence_rpm > max_cad {
        max_cad
    } else {
        cadence_rpm
    };

    let raw_da = (((global_mode_ceiling_da as u32) * (clamped_cad as u32)) / (max_cad as u32))
        as u16
        + pas_current_offset_da;

    if raw_da > level_power_ceiling_da {
        level_power_ceiling_da
    } else {
        raw_da
    }
}

/// Stage 2C: Stateful Hall Throttle Processor (`0x08008EB0`, `0x080064A4`, `0x08006328` in DM01 V6510C).
///
/// Implements an 8-tap IIR filter, a `200 mV/step` slew-rate limiter, and the 13 `SpeedMode` rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThrottleState {
    pub filtered_mv: u16,
    pub slew_limited_mv: u16,
    pub startup_vol_mv: u16,
    pub max_vol_mv: u16,
}

impl ThrottleState {
    pub const fn new(startup_vol_mv: u16, max_vol_mv: u16) -> Self {
        Self {
            filtered_mv: startup_vol_mv,
            slew_limited_mv: startup_vol_mv,
            startup_vol_mv,
            max_vol_mv,
        }
    }

    /// Updates the 8-tap IIR filter and `200 mV/step` slew-rate limiter (`0x08008EB0` & `0x080064A4`).
    pub fn step_voltage(&mut self, raw_throttle_mv: u16) -> u16 {
        // 8-tap IIR filter: floor((7 * prev + raw) / 8)
        self.filtered_mv = (((self.filtered_mv as u32) * 7 + (raw_throttle_mv as u32)) / 8) as u16;

        // Slew-rate limiter: max ±200 mV per control tick
        if self.filtered_mv > self.slew_limited_mv + 200 {
            self.slew_limited_mv += 200;
        } else if self.slew_limited_mv > self.filtered_mv + 200 {
            self.slew_limited_mv -= 200;
        } else {
            self.slew_limited_mv = self.filtered_mv;
        }
        self.slew_limited_mv
    }

    /// Computes Throttle target current in deci-amps (`0x08008DA4` in DM01 V6510C).
    pub fn compute_current_da(&mut self, raw_throttle_mv: u16, level_power_ceiling_da: u16) -> u16 {
        let v_mv = self.step_voltage(raw_throttle_mv);
        let start_mv = self.startup_vol_mv.max(500);
        if v_mv <= start_mv || self.max_vol_mv <= start_mv {
            return 0;
        }
        let clamped_v = v_mv.min(self.max_vol_mv);
        let net_v = (clamped_v - start_mv) as u32;
        let span_v = (self.max_vol_mv - start_mv) as u32;
        (((level_power_ceiling_da as u32) * net_v) / span_v) as u16
    }
}

/// Stage 3A: Proportional Speed Limit Soft-Taper (`0x0800674C` in DM01 V6510C).
///
/// Instead of an abrupt hard cutoff at `speed_limit_deci_kph`, linearly tapers the commanded
/// current across a softening window (`15 = 1.5 km/h` in normal riding, `20 = 2.0 km/h` in walk assist):
/// - Below `speed_limit - window`: 100% of `ramped_current_da`
/// - Inside `[speed_limit - window, speed_limit]`: linear taper to `0 dA`
/// - Above `speed_limit`: `0 dA`
pub const fn apply_speed_limit_soft_taper_da(
    ramped_current_da: u16,
    bike_speed_deci_kph: u16,
    speed_limit_deci_kph: u16,
    walk_assist_active: bool,
) -> u16 {
    if speed_limit_deci_kph == 0 {
        return ramped_current_da; // Unrestricted speed limit (e.g. Throttle Mode 12)
    }
    if bike_speed_deci_kph >= speed_limit_deci_kph {
        return 0;
    }
    let window_deci_kph: u16 = if walk_assist_active { 20 } else { 15 };
    let taper_start = speed_limit_deci_kph.saturating_sub(window_deci_kph);
    if bike_speed_deci_kph <= taper_start {
        return ramped_current_da;
    }
    let remaining_speed = (speed_limit_deci_kph - bike_speed_deci_kph) as u32;
    (((ramped_current_da as u32) * remaining_speed) / (window_deci_kph as u32)) as u16
}

/// Stage 3B: Low-Voltage Cutoff (LVC) 3.0V Reserve Cushion (`0x08005478` in DM01 V6510C).
///
/// When `battery_dv` enters the `30 dV` (`3.0 V`) window above `lvc_dv`, linearly tapers
/// current from `100%` at `lvc_dv + 30` down to `25%` at `lvc_dv + 1`, and cuts off to `0 dA`
/// at or below `lvc_dv`.
pub const fn apply_lvc_cushion_da(current_da: u16, battery_dv: u16, lvc_dv: u16) -> u16 {
    if battery_dv <= lvc_dv {
        return 0;
    }
    let margin_dv = battery_dv - lvc_dv;
    if margin_dv >= 30 {
        return current_da;
    }
    // Scale factor = 0.25 + 0.75 * (margin_dv / 30) = (30 + 3 * margin_dv) / 120
    let num = 30 + 3 * (margin_dv as u32);
    (((current_da as u32) * num) / 120) as u16
}

/// Stage 3 Complete Arbitrator (`0x08006620`, `0x0800674C`, `0x08005478` in DM01 V6510C).
///
/// Combines Torque, Cadence (PAS), and Throttle demands, applies the `±2 dA` (`±0.2 A`) per tick
/// slew-rate ramp limiter, proportional speed soft-taper, and LVC 3.0V reserve cushion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerCurveArbitrator {
    pub ramped_current_da: u16,
}

impl Default for PowerCurveArbitrator {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerCurveArbitrator {
    pub const fn new() -> Self {
        Self {
            ramped_current_da: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        brake_or_cutoff: bool,
        pedaling_forward: bool,
        torque_current_da: u16,
        pas_current_da: u16,
        throttle_current_da: u16,
        bike_speed_deci_kph: u16,
        speed_limit_deci_kph: u16,
        walk_assist_active: bool,
        battery_dv: u16,
        lvc_dv: u16,
    ) -> u16 {
        if brake_or_cutoff || battery_dv <= lvc_dv {
            self.ramped_current_da = 0;
            return 0;
        }

        // 1. Arbitrate winning demand: max(Torque, PAS, Throttle) when pedaling forward
        let winning_demand_da = if pedaling_forward {
            torque_current_da
                .max(pas_current_da)
                .max(throttle_current_da)
        } else {
            throttle_current_da
        };

        // 2. Smooth current ramp limiter: ±2 dA (±0.2 A) per tick
        if winning_demand_da > self.ramped_current_da + 2 {
            self.ramped_current_da += 2;
        } else if self.ramped_current_da > winning_demand_da + 2 {
            self.ramped_current_da -= 2;
        } else {
            self.ramped_current_da = winning_demand_da;
        }

        // 3. Proportional speed limit soft-taper (1.5 km/h normal / 2.0 km/h walk-assist)
        let speed_tapered_da = apply_speed_limit_soft_taper_da(
            self.ramped_current_da,
            bike_speed_deci_kph,
            speed_limit_deci_kph,
            walk_assist_active,
        );

        // 4. Low-Voltage 3.0V reserve cushion
        apply_lvc_cushion_da(speed_tapered_da, battery_dv, lvc_dv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_thermal_rollback_anchored_to_sport() {
        // Below 70°C -> full 300 dA (30.0 A)
        assert_eq!(compute_thermal_current_limit_da(65, 70, 100, 300), 300);
        assert_eq!(compute_thermal_current_limit_da(70, 70, 100, 300), 300);
        // At 85°C (halfway between 70°C and 100°C) -> 150 dA (15.0 A)
        assert_eq!(compute_thermal_current_limit_da(85, 70, 100, 300), 150);
        // At 95°C -> 50 dA (5.0 A)
        assert_eq!(compute_thermal_current_limit_da(95, 70, 100, 300), 50);
        // At or above 100°C -> 0 dA
        assert_eq!(compute_thermal_current_limit_da(100, 70, 100, 300), 0);
    }

    #[test]
    fn test_high_voltage_constant_power_cap() {
        // At 48.0 V (480 dV) and 50.0 V (500 dV) -> full 300 dA (30.0 A)
        assert_eq!(compute_voltage_capped_current_da(300, 480), 300);
        assert_eq!(compute_voltage_capped_current_da(300, 500), 300);
        // At 58.0 V (580 dV, fully charged 52V battery) -> floor(300 * 500 / 580) = 258 dA (25.8 A = 1496 W)
        assert_eq!(compute_voltage_capped_current_da(300, 580), 258);
    }

    #[test]
    fn test_dual_ceilings_torque_ignores_pwr_pct() {
        // City mode (180 dA = 18.0 A), Level 2 (Pwr = 40%), 50°C, 49.0 V (490 dV)
        let global_ceil = compute_global_mode_ceiling_da(180, 300, 50, 70, 100, 490);
        let level_ceil = compute_level_power_ceiling_da(180, 300, 40, 50, 70, 100, 490);

        assert_eq!(global_ceil, 180); // 18.0 A for Torque Sensor
        assert_eq!(level_ceil, 72); // 7.2 A (40% of 18.0 A) for PAS & Throttle
    }

    #[test]
    fn test_torque_sensor_worked_examples_and_cadence_scaling() {
        let mut ts = TorqueSensorState::new(700, 3300, 50);

        // Example A from Guide: City mode (180 dA), Level 2 (EAssist = 40%), Strength = 10,
        // CurrentOffset = 5 dA (0.5 A), 25% pedal force (700 + 650 = 1350 mV), 45 RPM, filter = 0 (instant)
        let current_da = ts.compute_current_da(1350, 45, 0, 10, 40, 5, 180, 180);
        // Combined effort = (25 * 30 * 40) / 1000 = 30%
        // Base current = (180 * 30 / 100) + 5 = 54 + 5 = 59 dA (5.9 A)
        // Cadence scaled at 45 RPM = floor(59 * 45 / 60) = 44 dA (4.4 A)
        assert_eq!(current_da, 44);

        // Example B from Guide: Sport mode (300 dA), Level 5 (EAssist = 100%), Strength = 10,
        // 50% pedal force (2000 mV), 60 RPM -> hits 100% effort cap, clamped to 300 dA (30.0 A)
        let climb_60rpm = ts.compute_current_da(2000, 60, 0, 10, 100, 5, 300, 300);
        assert_eq!(climb_60rpm, 300);

        // At 30 RPM on steep grade -> floor(305 * 30 / 60) = 152 dA (15.2 A)
        let climb_30rpm = ts.compute_current_da(2000, 30, 0, 10, 100, 5, 300, 300);
        assert_eq!(climb_30rpm, 152);
    }

    #[test]
    fn test_torque_sensor_auto_zero_drift_tracking() {
        let mut ts = TorqueSensorState::new(700, 3300, 50);
        // Simulate thermal drift to 750 mV while unloaded (0 RPM) for 4 window-fill + 50 quiet samples
        for _ in 0..54 {
            ts.update_voltage(750, 0, 5);
        }
        // 4:1 weighted update: (4 * 700 + 750) / 5 = 710 mV
        assert_eq!(ts.zero_vol_mv, 710);
    }

    #[test]
    fn test_speed_soft_taper_and_lvc_cushion() {
        // Normal 25.0 km/h (250 deci-kph) limit has a 1.5 km/h (15 deci-kph) taper window [235..250]
        assert_eq!(apply_speed_limit_soft_taper_da(300, 230, 250, false), 300);
        // Halfway into the 1.5 km/h window (24.25 km/h -> 242 deci-kph: remaining 8/15)
        assert_eq!(apply_speed_limit_soft_taper_da(300, 242, 250, false), 160);
        assert_eq!(apply_speed_limit_soft_taper_da(300, 250, 250, false), 0);

        // LVC cushion: LVC = 41.0 V (410 dV), 3.0 V window [410..440 dV]
        assert_eq!(apply_lvc_cushion_da(300, 445, 410), 300); // > 44.0V: 100%
        assert_eq!(apply_lvc_cushion_da(300, 425, 410), 187); // 42.5V (halfway): 62.5% of 300 = 187 dA
        assert_eq!(apply_lvc_cushion_da(300, 410, 410), 0); // <= 41.0V: 0 dA cutoff
    }
}
