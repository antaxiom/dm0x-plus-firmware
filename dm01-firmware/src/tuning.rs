//! # Source-Level Customization & Tuning Profiles
//!
//! Replaces binary patching with clean, `#![no_std]`-compatible source-level modularity.
//!
//! Rather than hacking bytes in a proprietary binary blob, custom motor behaviors are modeled
//! directly in Rust using configurable profiles and strategies. The stock `V6510C` behavior
//! is preserved as [`TuningProfile::stock_v6510c()`].

/// Cadence scaling strategy for torque-sensor pedal assist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadenceScaling {
    /// Stock `V6510C` behavior: scales assist linearly relative to 60 RPM (`max(cadence, 5) / 60`).
    /// Power is cut in half at 30 RPM, and only reaches 100% at 60+ RPM.
    Stock60Rpm,
    /// Gear-safe progressive knee ramp: protects the internal blue nylon reduction gear from
    /// destructive low-RPM stall torque (P = tau * omega) while eliminating the sluggish 60 RPM
    /// penalty on steep climbs.
    ///
    /// Provides soft current limit below `protect_knee_rpm` (e.g. 18 RPM), then ramps up to
    /// 100% assist by `target_rpm` (e.g. 35 RPM).
    GearSafeKnee {
        min_cadence_rpm: u16,
        protect_knee_rpm: u16,
        protect_limit_q8: u16,
        target_rpm: u16,
    },
    /// Uncapped cadence: delivers 100% torque assist at any cadence above `min_cadence_rpm` (e.g. 5 RPM).
    /// Eliminates the low-cadence climbing penalty.
    ///
    /// WARNING: High electrical current at near-zero RPM produces immense mechanical stall torque
    /// that can strip the teeth off the stock blue nylon gear. Recommended only for aftermarket
    /// all-metal gear sets!
    Uncapped { min_cadence_rpm: u16 },
    /// Custom linear ramp: 0% at `ramp_start_rpm`, reaching 100% at `target_rpm`.
    LinearRamp {
        ramp_start_rpm: u16,
        target_rpm: u16,
    },
}

impl CadenceScaling {
    /// Computes the cadence scaling multiplier in fixed-point Q8 (256 = 1.0 = 100%).
    pub const fn compute_multiplier_q8(&self, cadence_rpm: u16) -> u16 {
        match self {
            Self::Stock60Rpm => {
                if cadence_rpm == 0 {
                    0
                } else {
                    let eff = if cadence_rpm < 5 { 5 } else { cadence_rpm };
                    if eff >= 60 {
                        256
                    } else {
                        ((eff as u32 * 256) / 60) as u16
                    }
                }
            }
            Self::GearSafeKnee {
                min_cadence_rpm,
                protect_knee_rpm,
                protect_limit_q8,
                target_rpm,
            } => {
                if cadence_rpm < *min_cadence_rpm {
                    0
                } else if cadence_rpm >= *target_rpm {
                    256
                } else if cadence_rpm < *protect_knee_rpm {
                    let min_rpm = *min_cadence_rpm as u32;
                    let knee_rpm = *protect_knee_rpm as u32;
                    let diff = knee_rpm.saturating_sub(min_rpm);
                    let span = if diff < 1 { 1 } else { diff };
                    let delta = (cadence_rpm as u32).saturating_sub(min_rpm);
                    let base_floor = (*protect_limit_q8 as u32) / 4;
                    let dynamic_range = (*protect_limit_q8 as u32) - base_floor;
                    (base_floor + (dynamic_range * delta) / span) as u16
                } else {
                    let knee_rpm = *protect_knee_rpm as u32;
                    let target = *target_rpm as u32;
                    let diff = target.saturating_sub(knee_rpm);
                    let span = if diff < 1 { 1 } else { diff };
                    let delta = (cadence_rpm as u32).saturating_sub(knee_rpm);
                    let start_q8 = *protect_limit_q8 as u32;
                    let range = 256u32.saturating_sub(start_q8);
                    (start_q8 + (range * delta) / span) as u16
                }
            }
            Self::Uncapped { min_cadence_rpm } => {
                if cadence_rpm >= *min_cadence_rpm {
                    256
                } else {
                    0
                }
            }
            Self::LinearRamp {
                ramp_start_rpm,
                target_rpm,
            } => {
                if cadence_rpm <= *ramp_start_rpm {
                    0
                } else if cadence_rpm >= *target_rpm {
                    256
                } else {
                    let span = (*target_rpm - *ramp_start_rpm) as u32;
                    let delta = (cadence_rpm - *ramp_start_rpm) as u32;
                    ((delta * 256) / span) as u16
                }
            }
        }
    }
}

/// Torque strain-gauge effort response curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorqueResponse {
    /// Stock `V6510C` linear response with `min(3 * StrengthValue, 30) * EAssist% / 1000`.
    StockLinear,
    /// Progressive exponential curve: gentle response for feathering at light pedal effort,
    /// ramping up exponentially for steep climbs.
    Progressive,
    /// High-sensitivity linear: reaches maximum motor assist with ~30% less rider foot force,
    /// ideal for relaxed urban commuting.
    HighSensitivity,
}

impl TorqueResponse {
    /// Applies the response curve to the normalized raw effort (0..1000). Returns scaled effort (0..1000).
    pub fn shape_effort(&self, raw_effort_per_thousand: u16) -> u16 {
        let effort = raw_effort_per_thousand.min(1000) as u32;
        match self {
            Self::StockLinear => effort as u16,
            Self::Progressive => {
                // Quadratic curve: effort^2 / 1000
                ((effort * effort) / 1000) as u16
            }
            Self::HighSensitivity => {
                // Boost low-effort response by 1.4x, clamped to 1000
                let boosted = (effort * 14) / 10;
                boosted.min(1000) as u16
            }
        }
    }
}

/// Assist level ceiling enforcement policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistCeilingPolicy {
    /// Stock `V6510C` dual-ceiling split:
    /// - Torque sensor ignores the assist level's `Pwr%` and uses the Global Mode Ceiling (e.g. 30A in Sport).
    /// - Throttle and Cadence (PAS) obey the assist level's `Pwr%`.
    StockDualCeiling,
    /// Unified ceiling:
    /// - Torque sensor current is ALSO constrained by the active assist level's `Pwr%`.
    /// - Gives distinct power limits to each assist level even in torque-sensing mode.
    UnifiedLevelCeiling,
}

/// Torque strain-gauge zero-drift and dynamic span scaling policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorqueSpanPolicy {
    /// Stock static MaxVol (e.g. 3300 mV). Span = MaxVol - ZeroVol.
    /// Subject to sensitivity variation as ZeroVol thermally drifts.
    StockStaticMaxVol,
    /// Automatic zero-point drift tracking with fixed nominal span (e.g. 2300 mV from 3000 mV - 700 mV).
    /// Maintains consistent pedal effort sensitivity invariant to thermal drift:
    /// EffectiveMaxVol = ZeroVol + nominal_span_mv.
    AutoSpanTracking { nominal_span_mv: u16 },
}

/// Battery Low-Voltage Cutoff (LVC) cushion policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LvcCushionPolicy {
    /// Stock V6510C behavior: 3.0V cushion tapering down to a 25% current floor before 0A cutoff.
    Stock25PctFloor,
    /// Proportional true zero cutoff: linearly tapers current down to exactly 0.0 A at LVC across `cushion_dv`
    /// (e.g. 40 dV = 4.0 V), preventing high current sag and abrupt BMS hard-trips.
    TrueZeroCutoff { cushion_dv: u16 },
}

/// Complete modular tuning profile for ToSeven DM01 and DM02 motors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuningProfile {
    pub name: &'static str,
    pub cadence_scaling: CadenceScaling,
    pub torque_response: TorqueResponse,
    pub ceiling_policy: AssistCeilingPolicy,
    pub span_policy: TorqueSpanPolicy,
    pub lvc_policy: LvcCushionPolicy,
    pub speed_soft_taper_window_0_1_kph: u16,
    pub thermal_rollback_start_c: i16,
    pub thermal_rollback_cutoff_c: i16,
}

impl TuningProfile {
    /// Exact stock `V6510C` firmware behavior reconstructed from deobfuscated assembly.
    pub const fn stock_v6510c() -> Self {
        Self {
            name: "Stock V6510C",
            cadence_scaling: CadenceScaling::Stock60Rpm,
            torque_response: TorqueResponse::StockLinear,
            ceiling_policy: AssistCeilingPolicy::StockDualCeiling,
            span_policy: TorqueSpanPolicy::StockStaticMaxVol,
            lvc_policy: LvcCushionPolicy::Stock25PctFloor,
            speed_soft_taper_window_0_1_kph: 15, // 1.5 km/h
            thermal_rollback_start_c: 70,        // 70°C
            thermal_rollback_cutoff_c: 100,      // 100°C
        }
    }

    /// Gear-safe progressive trail profile: protects the internal blue nylon gear from destructive
    /// low-RPM stall torque while delivering 100% assist at 35+ RPM, with auto-span drift tracking
    /// and true 0A LVC taper.
    pub const fn gear_safe_trail() -> Self {
        Self {
            name: "Gear-Safe Trail",
            cadence_scaling: CadenceScaling::GearSafeKnee {
                min_cadence_rpm: 5,
                protect_knee_rpm: 18,
                protect_limit_q8: 102, // 40%
                target_rpm: 35,
            },
            torque_response: TorqueResponse::Progressive,
            ceiling_policy: AssistCeilingPolicy::StockDualCeiling,
            span_policy: TorqueSpanPolicy::AutoSpanTracking {
                nominal_span_mv: 2300,
            },
            lvc_policy: LvcCushionPolicy::TrueZeroCutoff { cushion_dv: 40 },
            speed_soft_taper_window_0_1_kph: 30, // 3.0 km/h
            thermal_rollback_start_c: 75,
            thermal_rollback_cutoff_c: 105,
        }
    }

    /// Uncapped cadence profile: removes the 60 RPM penalty, delivering instant, full torque
    /// assist right from pedal start (5 RPM). Perfect for steep climbs and heavy loads.
    ///
    /// WARNING: Delivers full electrical power at near-zero RPM. Only recommended for aftermarket
    /// all-metal reduction gear sets!
    pub const fn uncapped_cadence() -> Self {
        Self {
            name: "Uncapped Cadence",
            cadence_scaling: CadenceScaling::Uncapped { min_cadence_rpm: 5 },
            torque_response: TorqueResponse::StockLinear,
            ceiling_policy: AssistCeilingPolicy::StockDualCeiling,
            span_policy: TorqueSpanPolicy::AutoSpanTracking {
                nominal_span_mv: 2300,
            },
            lvc_policy: LvcCushionPolicy::TrueZeroCutoff { cushion_dv: 40 },
            speed_soft_taper_window_0_1_kph: 15,
            thermal_rollback_start_c: 70,
            thermal_rollback_cutoff_c: 100,
        }
    }

    /// Progressive mountain bike / trail profile: natural progressive torque ramp, gear-safe
    /// cadence knee ramp, auto-span drift tracking, and true 0A LVC cutoff.
    pub const fn progressive_trail() -> Self {
        Self::gear_safe_trail()
    }

    /// Commuter profile: high sensitivity for relaxed pedaling, unified level ceilings,
    /// auto-span drift tracking, and true 0A LVC cutoff.
    pub const fn commuter_comfort() -> Self {
        Self {
            name: "Commuter Comfort",
            cadence_scaling: CadenceScaling::LinearRamp {
                ramp_start_rpm: 10,
                target_rpm: 45,
            },
            torque_response: TorqueResponse::HighSensitivity,
            ceiling_policy: AssistCeilingPolicy::UnifiedLevelCeiling,
            span_policy: TorqueSpanPolicy::AutoSpanTracking {
                nominal_span_mv: 2000,
            },
            lvc_policy: LvcCushionPolicy::TrueZeroCutoff { cushion_dv: 40 },
            speed_soft_taper_window_0_1_kph: 20, // 2.0 km/h
            thermal_rollback_start_c: 70,
            thermal_rollback_cutoff_c: 100,
        }
    }
}

impl Default for TuningProfile {
    fn default() -> Self {
        Self::stock_v6510c()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_vs_uncapped_cadence() {
        let stock = CadenceScaling::Stock60Rpm;
        let uncapped = CadenceScaling::Uncapped { min_cadence_rpm: 5 };

        // At 30 RPM (climbing in a high gear):
        // Stock delivers only 50% (128 / 256)
        assert_eq!(stock.compute_multiplier_q8(30), 128);
        // Uncapped delivers full 100% (256 / 256)
        assert_eq!(uncapped.compute_multiplier_q8(30), 256);

        // At 60 RPM: both deliver 100%
        assert_eq!(stock.compute_multiplier_q8(60), 256);
        assert_eq!(uncapped.compute_multiplier_q8(60), 256);

        // At 0 RPM: neither delivers assist
        assert_eq!(stock.compute_multiplier_q8(0), 0);
        assert_eq!(uncapped.compute_multiplier_q8(0), 0);
    }

    #[test]
    fn test_gear_safe_knee() {
        let knee = CadenceScaling::GearSafeKnee {
            min_cadence_rpm: 5,
            protect_knee_rpm: 18,
            protect_limit_q8: 102, // 40% (102/256)
            target_rpm: 35,
        };

        // Below 5 RPM: Cutoff
        assert_eq!(knee.compute_multiplier_q8(0), 0);
        assert_eq!(knee.compute_multiplier_q8(4), 0);

        // At 5 RPM: Protective floor (~10% = 25/256)
        assert_eq!(knee.compute_multiplier_q8(5), 25);

        // At 10 RPM: Still protected to shield nylon gear (~21% = 54/256)
        assert_eq!(knee.compute_multiplier_q8(10), 54);

        // At 18 RPM: Reaches protective knee limit (102/256 = 40%)
        assert_eq!(knee.compute_multiplier_q8(18), 102);

        // At 25 RPM: Delivers strong climbing power (~64% = 165/256)
        assert_eq!(knee.compute_multiplier_q8(25), 165);

        // At 35+ RPM: Full 100% assist (256/256)
        assert_eq!(knee.compute_multiplier_q8(35), 256);
        assert_eq!(knee.compute_multiplier_q8(60), 256);
    }

    #[test]
    fn test_torque_response_shaping() {
        let linear = TorqueResponse::StockLinear;
        let prog = TorqueResponse::Progressive;
        let sensitive = TorqueResponse::HighSensitivity;

        // Light effort (20% = 200/1000):
        assert_eq!(linear.shape_effort(200), 200);
        // Progressive softens light effort: 200^2 / 1000 = 40 (4%)
        assert_eq!(prog.shape_effort(200), 40);
        // High sensitivity boosts light effort: 200 * 1.4 = 280 (28%)
        assert_eq!(sensitive.shape_effort(200), 280);

        // Maximum effort (1000/1000): all reach 1000
        assert_eq!(linear.shape_effort(1000), 1000);
        assert_eq!(prog.shape_effort(1000), 1000);
        assert_eq!(sensitive.shape_effort(1000), 1000);
    }
}
