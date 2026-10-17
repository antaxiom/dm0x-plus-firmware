/// Maximum number of configurable assist levels supported by the display/controller.
pub const MAX_ASSIST_LEVELS: usize = 8;

/// Global riding mode selected via the Display 'I' button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RidingMode {
    Eco,
    City,
    Sport,
}

/// Speed limit enforcement behavior for PAS and Torque Sensor modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedLimitMode {
    /// Enforce the speed limit configured for the active assist level.
    SpeedLevel,
    /// Enforce the maximum speed limit across all active assist levels.
    SpeedMaxLevel,
    /// Ignore speed limit cutoffs (power still governed by assist level).
    NoSpeedLimit,
}

/// Torque sensor sensitivity profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorqueMode {
    /// Combines pedal pressure and cadence rotation before ramping power.
    Safe,
    /// Prioritizes immediate pedal pressure response to minimize re-engagement lag.
    Sensitive,
}

/// Per-level configuration for Speed, Power, and EAssist sensitivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssistLevelConfig {
    /// Target speed limit in 0.1 km/h units (e.g., 250 = 25.0 km/h).
    pub speed_limit_deci_kph: u16,
    /// Maximum power percentage (0..=100%) primarily governing PAS output ceiling.
    pub power_percent: u8,
    /// EAssist factor (0..=100%) governing initial torque multiplication and effort scaling.
    pub eassist_percent: u8,
}

/// Complete user-configurable parameter set for the DM01 controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorConfig {
    /// Number of active assist levels (typically 5, up to 8).
    pub level_count: u8,
    /// Per-level assist curve definitions.
    pub levels: [AssistLevelConfig; MAX_ASSIST_LEVELS],

    /// Maximum battery current in milliamps (mA) for Eco, City, and Sport modes.
    pub max_current_eco_ma: u32,
    pub max_current_city_ma: u32,
    pub max_current_sport_ma: u32,

    /// Nominal battery voltage in millivolts (e.g., 48_000 for 48V).
    pub nominal_voltage_mv: u32,
    /// Low-voltage cutoff threshold in millivolts (e.g., 39_000 for 48V).
    pub low_cut_voltage_mv: u32,

    /// Wheel circumference in millimeters (e.g., 2206 mm).
    pub wheel_circumference_mm: u16,
    /// Mechanical reduction ratio scaled by 100 (DM01 = 4027 for 40.27:1).
    pub reduction_ratio_x100: u16,

    /// Torque Sensor parameters
    pub torque_sensor_enabled: bool,
    pub torque_mode: TorqueMode,
    /// Idle resting voltage in mV (default 700 mV).
    pub torque_zero_vol_mv: u16,
    /// Voltage delta above idle required to trigger assist (default 200 mV).
    pub torque_startup_vol_mv: u16,
    /// Initial current offset in mA applied at torque trigger.
    pub torque_current_offset_ma: u32,
    /// Ramp filter constant (higher = smoother/slower ramp).
    pub torque_filter: u8,
    /// Torque gain multiplier (default 10).
    pub torque_strength_value: u8,
    /// Decay hold time in milliseconds when pedal force drops (default 2000 ms, max 5000 ms).
    pub torque_keep_time_ms: u16,
    pub torque_speed_mode: SpeedLimitMode,

    /// Cadence (PAS) parameters
    pub pas_enabled: bool,
    pub pas_startup_magnets: u8,
    /// Run-on keep time in milliseconds after pedaling stops (200..=1000 ms, default 600 ms).
    pub pas_keep_time_ms: u16,
    /// Cadence ramp scaling factor (default 10).
    pub pas_strength_rpm: u8,
    /// Maximum supported cadence in RPM before assist tapers off (DM01 default 160 RPM).
    pub pas_max_cadence_rpm: u16,
    /// Initial current offset in mA when PAS engages.
    pub pas_current_offset_ma: u32,
    pub pas_speed_mode: SpeedLimitMode,

    /// Throttle parameters
    pub throttle_enabled: bool,
    /// Operating mode 0..=12 governing PAS interaction and speed limits.
    pub throttle_speed_mode: u8,

    /// Shift sensor parameters
    pub shift_sensor_enabled: bool,
    /// Duration in ms to reduce power during a gear shift.
    pub shift_delay_ms: u16,
    /// Power percentage retained during shift (e.g., 0% or 10%).
    pub shift_power_percent: u8,

    /// Thermal rollback parameters in degrees Celsius.
    pub temp_min_c: i16,
    pub temp_max_c: i16,
}

impl Default for MotorConfig {
    /// Returns the documented DM01 48V (1000W) factory defaults.
    fn default() -> Self {
        Self {
            level_count: 5,
            levels: [
                AssistLevelConfig {
                    speed_limit_deci_kph: 60, // 6.0 km/h
                    power_percent: 20,
                    eassist_percent: 30,
                },
                AssistLevelConfig {
                    speed_limit_deci_kph: 150, // 15.0 km/h
                    power_percent: 35,
                    eassist_percent: 50,
                },
                AssistLevelConfig {
                    speed_limit_deci_kph: 200, // 20.0 km/h
                    power_percent: 50,
                    eassist_percent: 60,
                },
                AssistLevelConfig {
                    speed_limit_deci_kph: 320, // 32.0 km/h
                    power_percent: 70,
                    eassist_percent: 80,
                },
                AssistLevelConfig {
                    speed_limit_deci_kph: 400, // 40.0 km/h
                    power_percent: 100,
                    eassist_percent: 100,
                },
                AssistLevelConfig {
                    speed_limit_deci_kph: 400,
                    power_percent: 100,
                    eassist_percent: 100,
                },
                AssistLevelConfig {
                    speed_limit_deci_kph: 400,
                    power_percent: 100,
                    eassist_percent: 100,
                },
                AssistLevelConfig {
                    speed_limit_deci_kph: 400,
                    power_percent: 100,
                    eassist_percent: 100,
                },
            ],
            max_current_eco_ma: 10_000,   // 10.0 A
            max_current_city_ma: 18_000,  // 18.0 A
            max_current_sport_ma: 30_000, // 30.0 A
            nominal_voltage_mv: 48_000,
            low_cut_voltage_mv: 39_000,
            wheel_circumference_mm: 2206,
            reduction_ratio_x100: 4027, // 40.27 : 1
            torque_sensor_enabled: true,
            torque_mode: TorqueMode::Safe,
            torque_zero_vol_mv: 700,
            torque_startup_vol_mv: 200,
            torque_current_offset_ma: 2_000,
            torque_filter: 5,
            torque_strength_value: 10,
            torque_keep_time_ms: 2000,
            torque_speed_mode: SpeedLimitMode::SpeedMaxLevel,
            pas_enabled: true,
            pas_startup_magnets: 1,
            pas_keep_time_ms: 600,
            pas_strength_rpm: 10,
            pas_max_cadence_rpm: 160,
            pas_current_offset_ma: 5_000,
            pas_speed_mode: SpeedLimitMode::SpeedMaxLevel,
            throttle_enabled: true,
            throttle_speed_mode: 7,
            shift_sensor_enabled: true,
            shift_delay_ms: 300,
            shift_power_percent: 10,
            temp_min_c: 70,
            temp_max_c: 100,
        }
    }
}

impl MotorConfig {
    /// Returns the mode current limit in mA, scaled dynamically by battery voltage
    /// to maintain consistent power delivery across the discharge curve.
    pub fn mode_current_limit_ma(&self, mode: RidingMode, actual_battery_mv: u32) -> u32 {
        let base_ma = match mode {
            RidingMode::Eco => self.max_current_eco_ma,
            RidingMode::City => self.max_current_city_ma,
            RidingMode::Sport => self.max_current_sport_ma,
        };
        if actual_battery_mv == 0 {
            return base_ma;
        }
        // DM01 constant-power voltage compensation: I_adj = I_48V * 50V / V_actual
        // Capped at 120% of base current to protect MOSFETs at low state of charge.
        let reference_mv = self.nominal_voltage_mv.saturating_add(2_000);
        let scaled = (base_ma as u64 * reference_mv as u64) / (actual_battery_mv as u64);
        let max_allowed = (base_ma * 120) / 100;
        (scaled as u32).min(max_allowed)
    }

    /// Returns the active assist level configuration (1-indexed level; 0 = assist off).
    pub fn active_level(&self, level: u8) -> Option<&AssistLevelConfig> {
        if level == 0 || level > self.level_count {
            return None;
        }
        let idx = (level - 1) as usize;
        self.levels.get(idx)
    }

    /// Returns the highest configured speed limit across all active levels in 0.1 km/h.
    pub fn max_level_speed_deci_kph(&self) -> u16 {
        let count = (self.level_count as usize).min(MAX_ASSIST_LEVELS);
        self.levels[..count]
            .iter()
            .map(|l| l.speed_limit_deci_kph)
            .max()
            .unwrap_or(400)
    }

    /// Preset: EN15194 street-legal pedelec profile (25.0 km/h assist cap, 6.0 km/h walk throttle).
    pub fn preset_en15194() -> Self {
        let mut cfg = Self::default();
        cfg.levels[0] = AssistLevelConfig {
            speed_limit_deci_kph: 180,
            power_percent: 25,
            eassist_percent: 20,
        };
        cfg.levels[1] = AssistLevelConfig {
            speed_limit_deci_kph: 250,
            power_percent: 40,
            eassist_percent: 40,
        };
        cfg.levels[2] = AssistLevelConfig {
            speed_limit_deci_kph: 250,
            power_percent: 60,
            eassist_percent: 60,
        };
        cfg.levels[3] = AssistLevelConfig {
            speed_limit_deci_kph: 250,
            power_percent: 80,
            eassist_percent: 80,
        };
        cfg.levels[4] = AssistLevelConfig {
            speed_limit_deci_kph: 250,
            power_percent: 100,
            eassist_percent: 100,
        };
        cfg.pas_speed_mode = SpeedLimitMode::SpeedLevel;
        cfg.torque_speed_mode = SpeedLimitMode::SpeedLevel;
        cfg.throttle_speed_mode = 3; // 6 km/h cut-off
        cfg
    }

    /// Preset: Jonas's DM01 responsive torque-sensor profile (high dynamic range, fast keep-time decay).
    pub fn preset_jonas_dm01() -> Self {
        let mut cfg = Self::default();
        cfg.levels[0] = AssistLevelConfig {
            speed_limit_deci_kph: 450,
            power_percent: 46,
            eassist_percent: 20,
        };
        cfg.levels[1] = AssistLevelConfig {
            speed_limit_deci_kph: 450,
            power_percent: 54,
            eassist_percent: 32,
        };
        cfg.levels[2] = AssistLevelConfig {
            speed_limit_deci_kph: 450,
            power_percent: 62,
            eassist_percent: 44,
        };
        cfg.levels[3] = AssistLevelConfig {
            speed_limit_deci_kph: 450,
            power_percent: 70,
            eassist_percent: 56,
        };
        cfg.levels[4] = AssistLevelConfig {
            speed_limit_deci_kph: 600,
            power_percent: 82,
            eassist_percent: 68,
        };
        cfg.shift_power_percent = 10;
        cfg.pas_keep_time_ms = 200;
        cfg.torque_mode = TorqueMode::Sensitive;
        cfg.torque_keep_time_ms = 200;
        cfg.torque_startup_vol_mv = 300;
        cfg
    }

    /// Preset: 8-Level progressive single-mode profile ("One Mode to Rule Them All").
    pub fn preset_eight_level_progressive() -> Self {
        let mut cfg = Self::default();
        cfg.level_count = 8;
        cfg.levels = [
            AssistLevelConfig { speed_limit_deci_kph: 150, power_percent: 11, eassist_percent: 6 },
            AssistLevelConfig { speed_limit_deci_kph: 250, power_percent: 21, eassist_percent: 14 },
            AssistLevelConfig { speed_limit_deci_kph: 300, power_percent: 30, eassist_percent: 20 },
            AssistLevelConfig { speed_limit_deci_kph: 350, power_percent: 40, eassist_percent: 30 },
            AssistLevelConfig { speed_limit_deci_kph: 350, power_percent: 50, eassist_percent: 40 },
            AssistLevelConfig { speed_limit_deci_kph: 350, power_percent: 60, eassist_percent: 50 },
            AssistLevelConfig { speed_limit_deci_kph: 400, power_percent: 80, eassist_percent: 60 },
            AssistLevelConfig { speed_limit_deci_kph: 500, power_percent: 100, eassist_percent: 100 },
        ];
        cfg
    }
}

