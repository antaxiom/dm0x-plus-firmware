//! # Exact Decompiled V6510C Power Curve & Control Pipeline
//!
//! This module implements the bit-accurate integer math of the ToSeven DM01/DM02 `V6510C`
//! power curve pipeline in units matching the MCU firmware:
//! - Currents in **deci-amps (`dA`, `0.1 A`)**: e.g., `300 = 30.0 A`, `180 = 18.0 A`, `100 = 10.0 A`.
//! - Voltages in **millivolts (`mV`)** for sensors and **deci-volts (`dV`, `0.1 V`)** for battery bus (`500 = 50.0 V`).
//! - Speeds in **deci-km/h (`0.1 km/h`)**: e.g., `250 = 25.0 km/h`, `60 = 6.0 km/h`.
//! - Temperatures in **degrees Celsius (`°C`)**.
//!
//! # Compile-Time Tuning
//!
//! The former runtime `TuningProfile` / policy enums have been replaced by the constants
//! below. To customize a build, edit these values and the torque mapping table; there is no
//! longer a second code path for stock, uncapped, or commuter profiles.

/// Absolute hardware current limit (`400 dA = 40.0 A`) enforced by `V6510C`.
pub const HARDWARE_MAX_CURRENT_DA: u16 = 400;

/// Nominal full-current reference battery voltage (`500 dV = 50.0 V`).
/// Above `50.0 V`, maximum current is scaled by `500 / battery_dv` to hold constant max power.
pub const CONSTANT_POWER_REF_VOLT_DV: u16 = 500;

/// Torque mapping table applied to the normalized pedal effort (`0..=100%`).
///
/// The default is a 1:1 linear mapping, matching the stock `V6510C` response. Replace entries
/// to create progressive, high-sensitivity, or custom curves without a runtime profile.
pub const TORQUE_MAPPING_TABLE: [u8; 101] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100,
];

/// Fixed nominal torque-sensor span in millivolts.
///
/// The automatic `ZeroVol` tracker compensates for thermal drift, so the span stays constant.
/// `2000 mV` reaches 100% effort at the sensor's 3300 mV full scale even when `ZeroVol` has
/// drifted up to 1300 mV (`1300 + 2000 = 3300`).
pub const TORQUE_SENSOR_SPAN_MV: u16 = 2_000;

/// Minimum cadence for any torque assist. Below this the cadence multiplier is zero.
pub const CADENCE_MIN_RPM: u16 = 5;

/// Cadence at which the gear-safe knee reaches its protective current limit.
pub const CADENCE_KNEE_RPM: u16 = 18;

/// Protective current limit at the knee in Q8 (`102 / 256 = 40%`).
pub const CADENCE_KNEE_LIMIT_Q8: u16 = 102;

/// Cadence at which the gear-safe ramp reaches full (`256 / 256 = 100%`) assist.
pub const CADENCE_FULL_RPM: u16 = 35;

/// Number of stationary samples averaged into one `ZeroVol` start-stop cycle entry.
pub const ZERO_VOL_QUIET_TICKS: u16 = 50;

/// When true, the torque sensor is also clamped by the assist level's `Pwr%`.
/// The default `false` preserves the stock dual-ceiling split: torque uses the
/// global riding-mode ceiling while PAS and throttle use the level ceiling.
pub const TORQUE_USES_LEVEL_CEILING: bool = false;

/// Normal-riding speed soft-taper window in deci-km/h (`15 = 1.5 km/h`).
pub const SPEED_SOFT_TAPER_WINDOW_DECI_KPH: u16 = 15;

/// Current floor at the Low-Voltage Cutoff in percent (`0 = true 0.0 A cutoff`).
pub const LVC_CUSHION_FLOOR_PERCENT: u8 = 0;

/// Low-voltage reserve cushion window in deci-volts (`40 = 4.0 V`).
pub const LVC_CUSHION_WINDOW_DV: u16 = 40;

const _: () = assert!(CADENCE_KNEE_RPM > CADENCE_MIN_RPM);
const _: () = assert!(CADENCE_FULL_RPM > CADENCE_KNEE_RPM);
const _: () = assert!(CADENCE_KNEE_LIMIT_Q8 <= 256);
const _: () = assert!(TORQUE_SENSOR_SPAN_MV >= 1_000);

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

/// Gear-safe cadence multiplier in Q8 (`256 = 1.0 = 100%`).
///
/// The stock `V6510C` scales torque linearly to `60 RPM`, which still applies substantial
/// current at near-stall cadences where the nylon primary reduction gear is most vulnerable.
/// This ramp intentionally stays soft below the `18 RPM` knee (`<= 40%`), then reaches full
/// assist at `35 RPM`:
/// - `< 5 RPM`: `0`
/// - `5 RPM`: `25/256` (~10%)
/// - `18 RPM`: `102/256` (40%)
/// - `35+ RPM`: `256/256` (100%)
pub const fn cadence_multiplier_q8(cadence_rpm: u16) -> u16 {
    if cadence_rpm < CADENCE_MIN_RPM {
        return 0;
    }
    if cadence_rpm >= CADENCE_FULL_RPM {
        return 256;
    }
    if cadence_rpm < CADENCE_KNEE_RPM {
        let span = (CADENCE_KNEE_RPM - CADENCE_MIN_RPM) as u32;
        let delta = (cadence_rpm - CADENCE_MIN_RPM) as u32;
        let base_floor = CADENCE_KNEE_LIMIT_Q8 / 4;
        let dynamic_range = CADENCE_KNEE_LIMIT_Q8 - base_floor;
        (base_floor as u32 + (dynamic_range as u32 * delta) / span) as u16
    } else {
        let span = (CADENCE_FULL_RPM - CADENCE_KNEE_RPM) as u32;
        let delta = (cadence_rpm - CADENCE_KNEE_RPM) as u32;
        let range = 256u32 - CADENCE_KNEE_LIMIT_Q8 as u32;
        (CADENCE_KNEE_LIMIT_Q8 as u32 + (range * delta) / span) as u16
    }
}

/// Stage 2A: Stateful Torque Sensor Processor (`0x08009034`, `0x0800912C`, `0x08008024` in DM01 V6510C).
///
/// Implements:
/// 1. 12-bit ADC (`0..4095`) to millivolt (`0..3300 mV`) conversion.
/// 2. 5-sample moving delta window (`0x08009034`) for automatic resting `ZeroVol` thermal drift tracking.
///    Samples are clamped to `[400 mV, 1300 mV]`, then squeezed into `[zero_vol_mv − 200 mV,
///    zero_vol_mv + 200 mV]`, and accumulated over [`ZERO_VOL_QUIET_TICKS`] consecutive quiet
///    ticks. The resulting average becomes one entry in a 5-deep start-stop cycle history;
///    `ZeroVolMv` is the mean of that history clamped to `[400 mV, 1300 mV]`. A cycle completes
///    only once per start-stop sequence: tracking is disabled after an update and re-armed when
///    cadence returns above zero.
/// 3. Configurable IIR pedal force smoothing filter (`F = min(20 * filter_setting, 200)`).
/// 4. `StrengthValue * EAssist[level]` amplification and `CurrentOffset` starting kick.
/// 5. Gear-safe cadence scaling ([`cadence_multiplier_q8`]), clamped to the global mode ceiling.
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
    /// Number of samples accumulated in the current quiet window (start-stop cycle).
    quiet_ticks: u16,
    /// History of per-cycle averaged `ZeroVolMv` values from the last 5 start-stop cycles.
    /// `history_zero_volt[0]` is the newest entry.
    history_zero_volt: [u16; 5],
    /// Running sum of `clamped_pedal_mv` samples within the current quiet window.
    sum_pedal_mv: u32,
    /// Set after a cycle update completes; re-armed when `cadence_rpm > 0` is observed.
    zero_volt_tracking_disabled: bool,
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
            // Seed the cycle history with the initial zero so the very first
            // completed cycle averages against a sensible baseline instead of zeros.
            history_zero_volt: [zero_vol_mv; 5],
            sum_pedal_mv: 0,
            zero_volt_tracking_disabled: false,
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

        // 2. Adaptive zero-point drift tracking (only while cranks are stationary).
        //
        // While `cadence_rpm == 0` and the signal is stable (delta < 20 mV), each pedal
        // sample is clamped to a hard global window `[400 mV, 1300 mV]`, then further
        // squeezed into a tighter dynamic band `[zero_vol_mv − 200 mV, zero_vol_mv + 200 mV]`
        // around the current zero estimate. This biases any outlier samples toward the
        // most recent baseline instead of letting them drag the accumulated average.
        //
        // Once `ZERO_VOL_QUIET_TICKS` quiet ticks are collected, their average becomes one
        // entry in the 5-deep start-stop cycle history. `ZeroVolMv` is recomputed as the mean
        // of that history and clamped back into `[400 mV, 1300 mV]`. Tracking is then
        // disabled until the cranks move again, preventing a single long stop from
        // continuously dragging the zero point.
        if cadence_rpm == 0 {
            if !self.zero_volt_tracking_disabled && max_delta < 20 {
                if self.quiet_ticks < u16::MAX {
                    let mut clamped_pedal_mv: u32 = pedal_mv as u32;
                    // Hard global limits on any accepted sample.
                    clamped_pedal_mv = clamped_pedal_mv.clamp(400, 1300);
                    // Tighter dynamic band around the current zero estimate.
                    let upper = self.zero_vol_mv as u32 + 200;
                    let lower = (self.zero_vol_mv as u32).saturating_sub(200);
                    clamped_pedal_mv = clamped_pedal_mv.min(upper).max(lower);

                    self.sum_pedal_mv += clamped_pedal_mv;
                    self.quiet_ticks += 1;
                }

                if self.quiet_ticks >= ZERO_VOL_QUIET_TICKS {
                    // Shift the cycle history right by one (evicts the oldest entry).
                    for j in (1..5).rev() {
                        self.history_zero_volt[j] = self.history_zero_volt[j - 1];
                    }
                    // Insert this cycle's average as the newest entry.
                    self.history_zero_volt[0] =
                        (self.sum_pedal_mv / ZERO_VOL_QUIET_TICKS as u32) as u16;

                    // Mean of the last 5 start-stop cycles becomes the new resting zero.
                    let mut sum_zero = 0u32;
                    for j in 0..5 {
                        sum_zero += self.history_zero_volt[j] as u32;
                    }
                    // Hard global clamp on the recomputed zero.
                    self.zero_vol_mv = (sum_zero / 5).clamp(400, 1300) as u16;

                    // Reset accumulators and lock tracking until cranks move again.
                    self.sum_pedal_mv = 0;
                    self.quiet_ticks = 0;
                    self.zero_volt_tracking_disabled = true;
                }
            } else {
                // Signal too noisy or tracking already completed for this stop: reset.
                self.sum_pedal_mv = 0;
                self.quiet_ticks = 0;
            }
        } else {
            // Cranks are moving: reset accumulators and re-arm tracking for the next stop.
            self.sum_pedal_mv = 0;
            self.quiet_ticks = 0;
            self.zero_volt_tracking_disabled = false;
        }

        // 3. Net pedal voltage and configurable IIR low-pass filter
        let raw_net_mv = pedal_mv.saturating_sub(self.zero_vol_mv);
        let f_coeff = ((filter_setting as u32) * 20).min(200);
        self.filtered_net_mv = (((self.filtered_net_mv as u32) * f_coeff + (raw_net_mv as u32))
            / (f_coeff + 1)) as u16;

        // 4. Startup deadband gate and normalization to 0..100%
        if self.filtered_net_mv < self.startup_net_vol_mv {
            self.torque_pct = 0;
        } else {
            let span = TORQUE_SENSOR_SPAN_MV as u32;
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
        level_power_ceiling_da: u16,
    ) -> u16 {
        self.update_voltage(pedal_mv, cadence_rpm, filter_setting);

        if self.torque_pct > 100 {
            self.torque_pct = 100;
        }
        // Apply the compile-time response curve. The default table is linear.
        self.torque_pct = TORQUE_MAPPING_TABLE[self.torque_pct as usize];

        if self.torque_pct == 0 || cadence_rpm == 0 || level_eassist_pct == 0 {
            return 0;
        }

        // 1. Scale StrengthValue: S_scaled = min(3 * StrengthValue, 30)
        let s_scaled = ((strength_value as u32) * 3).min(30);

        // 2. Combined Effort Multiplier r4 = min(floor(floor(torque_pct * S_scaled * EAssist / 10) / 100), 100)
        let scaled_effort = ((self.torque_pct as u32) * s_scaled * (level_eassist_pct as u32)) / 10;
        let combined_effort_pct = (scaled_effort / 100).min(100);

        // 3. Base Torque Current = floor(combined_effort_pct * mode_max_da / 100) + current_offset_da
        let mode_cap = mode_max_da.min(HARDWARE_MAX_CURRENT_DA) as u32;
        let base_current_da = ((combined_effort_pct * mode_cap) / 100) + (current_offset_da as u32);

        // 4. Gear-safe cadence scaling (Q8: 256 = 100%)
        let cadence_scaled_da =
            ((base_current_da * cadence_multiplier_q8(cadence_rpm) as u32) / 256) as u16;

        // 5. Compile-time ceiling selection. Stock dual ceiling keeps torque on the global
        //    mode ceiling; enabling the constant makes torque respect the level's Pwr%.
        let ceiling: u16 = if TORQUE_USES_LEVEL_CEILING {
            global_mode_ceiling_da.min(level_power_ceiling_da)
        } else {
            global_mode_ceiling_da
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
/// current across a softening window ([`SPEED_SOFT_TAPER_WINDOW_DECI_KPH`] in normal riding,
/// `20 = 2.0 km/h` in walk assist):
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
    let window_deci_kph: u16 = if walk_assist_active {
        20
    } else {
        SPEED_SOFT_TAPER_WINDOW_DECI_KPH
    };
    let taper_start = speed_limit_deci_kph.saturating_sub(window_deci_kph);
    if bike_speed_deci_kph <= taper_start || window_deci_kph == 0 {
        return ramped_current_da;
    }
    let remaining_speed = (speed_limit_deci_kph - bike_speed_deci_kph) as u32;
    (((ramped_current_da as u32) * remaining_speed) / (window_deci_kph as u32)) as u16
}

const fn clamp_floor_pct(value: u8) -> u32 {
    if value > 100 {
        100
    } else {
        value as u32
    }
}

/// Stage 3B: Low-Voltage Cutoff (LVC) Reserve Cushion (`0x08005478` in DM01 V6510C).
///
/// Linearly interpolates current from [`LVC_CUSHION_FLOOR_PERCENT`] at the LVC up to 100% at
/// `lvc_dv + `[`LVC_CUSHION_WINDOW_DV`], and cuts off to `0 dA` at or below `lvc_dv`. The
/// default (`0%` floor, `40 dV = 4.0 V` window) is the true-zero taper: it reaches exactly
/// `0.0 A` at the cutoff instead of the stock 25% floor.
pub const fn apply_lvc_cushion_da(current_da: u16, battery_dv: u16, lvc_dv: u16) -> u16 {
    if battery_dv <= lvc_dv {
        return 0;
    }
    if LVC_CUSHION_WINDOW_DV == 0 {
        return current_da;
    }
    let margin_dv = battery_dv - lvc_dv;
    if margin_dv >= LVC_CUSHION_WINDOW_DV {
        return current_da;
    }
    let floor_pct = clamp_floor_pct(LVC_CUSHION_FLOOR_PERCENT);
    let window_dv = LVC_CUSHION_WINDOW_DV as u32;
    let num = floor_pct * window_dv + (100 - floor_pct) * (margin_dv as u32);
    ((current_da as u32 * num) / (100 * window_dv)) as u16
}

/// Stage 3 Complete Arbitrator (`0x08006620`, `0x0800674C`, `0x08005478` in DM01 V6510C).
///
/// Combines Torque, Cadence (PAS), and Throttle demands, applies the `±2 dA` (`±0.2 A`) per tick
/// slew-rate ramp limiter, proportional speed soft-taper, and LVC reserve cushion.
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

        // 1. Arbitration: max(Torque, PAS, Throttle) when pedaling forward
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

        // 3. Proportional speed limit soft-taper
        let speed_tapered_da = apply_speed_limit_soft_taper_da(
            self.ramped_current_da,
            bike_speed_deci_kph,
            speed_limit_deci_kph,
            walk_assist_active,
        );

        // 4. Low-voltage reserve cushion
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
    fn test_gear_safe_cadence_multiplier() {
        // Below 5 RPM: no assist
        assert_eq!(cadence_multiplier_q8(0), 0);
        assert_eq!(cadence_multiplier_q8(4), 0);
        // 5 RPM: protective floor (~10%)
        assert_eq!(cadence_multiplier_q8(5), 25);
        // 10 RPM: still protected (~21%)
        assert_eq!(cadence_multiplier_q8(10), 54);
        // 18 RPM: knee limit (40%)
        assert_eq!(cadence_multiplier_q8(18), 102);
        // 25 RPM: strong climbing power (~64%)
        assert_eq!(cadence_multiplier_q8(25), 165);
        // 35+ RPM: full assist
        assert_eq!(cadence_multiplier_q8(35), 256);
        assert_eq!(cadence_multiplier_q8(60), 256);
    }

    #[test]
    fn test_torque_sensor_worked_examples_and_cadence_scaling() {
        let mut ts = TorqueSensorState::new(700, 3300, 50);

        // City mode (180 dA), Level 2 (EAssist = 40%), Strength = 10,
        // CurrentOffset = 5 dA (0.5 A), 32% pedal force (700 + 650 = 1350 mV), 45 RPM, filter = 0.
        let current_da = ts.compute_current_da(1350, 45, 0, 10, 40, 5, 180, 180, 180);
        // Torque pct = 650 / 2000 = 32%
        // Combined effort = (32 * 30 * 40) / 1000 = 38%
        // Base current = (180 * 38 / 100) + 5 = 68 + 5 = 73 dA (7.3 A)
        // 45 RPM is above the 35 RPM knee -> 100% cadence multiplier
        assert_eq!(current_da, 73);

        // Sport mode (300 dA), Level 5 (EAssist = 100%), Strength = 10,
        // 65% pedal force (2000 mV), 60 RPM -> hits 100% effort cap, clamped to 300 dA (30.0 A)
        let climb_60rpm = ts.compute_current_da(2000, 60, 0, 10, 100, 5, 300, 300, 300);
        assert_eq!(climb_60rpm, 300);

        // At 30 RPM on a steep grade -> gear-safe multiplier is 210/256 (~82%)
        let climb_30rpm = ts.compute_current_da(2000, 30, 0, 10, 100, 5, 300, 300, 300);
        assert_eq!(climb_30rpm, 250);
    }

    #[test]
    fn test_torque_uses_stock_dual_ceiling_by_default() {
        let mut ts = TorqueSensorState::new(700, 3300, 50);
        let out = ts.compute_current_da(3300, 60, 0, 10, 100, 0, 300, 300, 200);
        if TORQUE_USES_LEVEL_CEILING {
            assert_eq!(out, 200);
        } else {
            assert_eq!(out, 300);
        }
    }

    #[test]
    fn test_torque_sensor_auto_zero_drift_tracking() {
        let mut ts = TorqueSensorState::new(700, 3300, 50);
        // Simulate thermal drift to 750 mV while unloaded (0 RPM) for 4 window-fill + 50 quiet samples
        for _ in 0..54 {
            ts.update_voltage(750, 0, 5);
        }
        // Cycle 1: 50 quiet ticks × 750 mV → history_zero_volt = [750, 700, 700, 700, 700]
        // ZeroVolMv = (750 + 4 × 700) / 5 = 710 mV
        assert_eq!(ts.zero_vol_mv, 710);
    }

    #[test]
    fn test_zero_tracking_clamps_sample_into_dynamic_band() {
        // Zero starts at 700 mV. Dynamic band is [500, 900].
        // Feed 1200 mV: hard clamp keeps 1200, dynamic min(1200, 900) = 900.
        let mut ts = TorqueSensorState::new(700, 3300, 50);
        for _ in 0..54 {
            ts.update_voltage(1200, 0, 0);
        }
        // Every sample clamped to 900 mV → history[0] = 900
        // New zero = (900 + 700*4) / 5 = 740
        assert_eq!(ts.zero_vol_mv, 740);
    }

    #[test]
    fn test_zero_tracking_pulls_low_outlier_upward() {
        // Zero starts at 700 mV. Dynamic band is [500, 900].
        // Feed 300 mV → hard clamp → 400 mV → dynamic max(400, 500) = 500 mV.
        let mut ts = TorqueSensorState::new(700, 3300, 50);
        for _ in 0..54 {
            ts.update_voltage(300, 0, 0);
        }
        // Every sample clamped to 500 mV → history[0] = 500
        // New zero = (500 + 700*4) / 5 = 660
        assert_eq!(ts.zero_vol_mv, 660);
    }

    #[test]
    fn test_zero_volt_result_is_hard_clamped() {
        // Zero starts at 1300 mV (upper limit). Dynamic band is [1100, 1500].
        // Feed 1400 mV: hard clamped to 1300 mV each cycle, stays at 1300.
        let mut ts = TorqueSensorState::new(1300, 3300, 50);
        for _ in 0..5 {
            for _ in 0..54 {
                ts.update_voltage(1400, 0, 0);
            }
            // Cadence > 0 re-arms for the next stop
            ts.update_voltage(1400, 60, 0);
        }
        assert_eq!(ts.zero_vol_mv, 1300);
        assert!(ts.zero_vol_mv <= 1300);
        assert!(ts.zero_vol_mv >= 400);
    }

    #[test]
    fn test_zero_tracking_latches_once_per_stop() {
        let mut ts = TorqueSensorState::new(700, 3300, 50);

        // 1. First stop: 54 ticks at 750 mV completes 1 cycle
        for _ in 0..54 {
            ts.update_voltage(750, 0, 0);
        }
        assert_eq!(ts.zero_vol_mv, 710);

        // 2. Continuous sitting at red light: another 100 ticks without moving cranks
        for _ in 0..100 {
            ts.update_voltage(750, 0, 0);
        }
        // Remains 710 mV because tracking latches off until cranks move!
        assert_eq!(ts.zero_vol_mv, 710);

        // 3. Cranks move (cadence > 0) -> re-arms tracking
        for _ in 0..5 {
            ts.update_voltage(750, 60, 0);
        }

        // 4. Second stop: 54 ticks at 750 mV updates cycle 2
        for _ in 0..54 {
            ts.update_voltage(750, 0, 0);
        }
        // history = [750, 750, 700, 700, 700], mean = 3600 / 5 = 720 mV
        assert_eq!(ts.zero_vol_mv, 720);
    }

    #[test]
    fn test_speed_soft_taper_and_lvc_cushion() {
        // Normal 25.0 km/h (250 deci-kph) limit has a 1.5 km/h (15 deci-kph) taper window [235..250]
        assert_eq!(apply_speed_limit_soft_taper_da(300, 230, 250, false), 300);
        // Halfway into the 1.5 km/h window (24.25 km/h -> 242 deci-kph: remaining 8/15)
        assert_eq!(apply_speed_limit_soft_taper_da(300, 242, 250, false), 160);
        assert_eq!(apply_speed_limit_soft_taper_da(300, 250, 250, false), 0);

        // LVC cushion: LVC = 41.0 V (410 dV), 4.0 V true-zero window [410..450 dV]
        assert_eq!(apply_lvc_cushion_da(300, 450, 410), 300); // >= 45.0V: 100%
        assert_eq!(apply_lvc_cushion_da(300, 430, 410), 150); // 43.0V (halfway): 50%
        assert_eq!(apply_lvc_cushion_da(300, 425, 410), 112); // 42.5V: 37.5% -> 112 dA
        assert_eq!(apply_lvc_cushion_da(300, 410, 410), 0); // <= 41.0V: 0 dA cutoff
    }

    #[test]
    fn test_lvc_cushion_floor_is_configurable() {
        // The default constant is a true-zero cutoff. This guards against accidental
        // reintroduction of the stock 25% floor without updating the documentation.
        assert_eq!(LVC_CUSHION_FLOOR_PERCENT, 0);
        assert_eq!(LVC_CUSHION_WINDOW_DV, 40);
        assert_eq!(apply_lvc_cushion_da(300, 411, 410), 7); // 2.5% of 300 = 7 dA
    }

    #[test]
    fn test_fixed_span_invariance() {
        // The fixed 2000 mV span keeps effort sensitivity invariant across ZeroVol drift.
        let mut cold_ts = TorqueSensorState::new(600, 3000, 50);
        let mut warm_ts = TorqueSensorState::new(900, 3000, 50);

        // Equal human foot force produces 1150 mV delta above resting zero:
        // Cold sensor (0.6V rest): 600 + 1150 = 1750 mV
        // Warm sensor (0.9V rest): 900 + 1150 = 2050 mV
        let cold_da = cold_ts.compute_current_da(1750, 60, 0, 10, 100, 0, 300, 300, 300);
        let warm_da = warm_ts.compute_current_da(2050, 60, 0, 10, 100, 0, 300, 300, 300);

        // Both must produce identical torque percentage (1150 / 2000 = 57%) and identical current!
        assert_eq!(cold_ts.torque_pct, 57);
        assert_eq!(warm_ts.torque_pct, 57);
        assert_eq!(
            cold_da, warm_da,
            "Fixed span must produce identical current across thermal drift"
        );
    }
}
