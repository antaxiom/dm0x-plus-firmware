/// Maximum number of configurable assist levels supported by the display/controller (`1..=8`).
pub const MAX_ASSIST_LEVELS: usize = 8;

/// Global riding mode selected via the Display 'I' (Info) button (Page 4 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RidingMode {
    Eco,
    City,
    Sport,
}

/// Active assistance gear selection, including special modes A (Auto), C (Climb), R (Reverse)
/// documented on Page 4 of the T24 Display Manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistSelection {
    /// Level 0: Electric assistance switched off.
    Off,
    /// Standard numerical level `1..=8`.
    Level(u8),
    /// Special Level A: Automatically adjusts power and motor speed according to bike speed.
    Auto,
    /// Special Level C: Strong power when using pedal sensor uphill.
    Climb,
    /// Special Level R: Motor runs in reverse (for cargo trikes).
    Reverse,
}

/// Motor gear configuration (`Motor > Gear`, Page 12 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorGearType {
    GearLess,
    Gear,
}

/// PAS directional discrimination type (`PAS > Type`, Page 12 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasType {
    Disable,
    /// Non-directional discrimination type 1.
    Low,
    /// Non-directional discrimination type 2.
    High,
    /// With directional discrimination type PAS.
    Auto,
}

/// PAS speed control mode (`PAS > ControlMode`, Page 12-13 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasControlMode {
    /// Maximum gear speed limit (`MaxBikeSpeed`).
    MaxBikeSpeed,
    /// Gear speed limit (`LevelSpeed`).
    LevelSpeed,
    /// PAS rotate speed corresponds to bicycle speed (`PAS speed * max_wheel_speed / max_pas_speed`).
    PedalSpeed1,
    /// PAS speed corresponds to bicycle speed, maximum speed limit.
    PedalSpeed2,
}

/// Torque sensor operating mode (`Torque Sensor > Mode`, Page 13 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorqueMode {
    /// Activated when `net_torque_mv > StartupNetVol` AND pedaling is active.
    Safe,
    /// Activated immediately when `net_torque_mv > StartupNetVol`.
    Sensitive,
}

/// Over-temperature current limiting mode (`TemperatureSensor > OverTemp`, Page 14 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverTempMode {
    /// Skip: unlimited power.
    Skip,
    /// Motor is limited to 30% power after overheating.
    Power30Percent,
    /// Motor is limited to 50% power after overheating.
    Power50Percent,
    /// Motor is limited to 70% power after overheating.
    Power70Percent,
}

/// Per-level configuration for `Speed&Power` (`1..=8`, Page 11 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssistLevelConfig {
    /// Target speed limit in 0.1 km/h units (e.g., 250 = 25.0 km/h).
    pub speed_limit_deci_kph: u16,
    /// Current limit percentage (`0..=100%`) of the maximum allowable working current.
    pub power_percent: u8,
    /// EAssist factor (`0..=100%`) governing torque sensor sensitivity per level.
    pub eassist_percent: u8,
}

/// Motor hardware and BLDC/FOC startup parameters (`Motor` menu, Page 12 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotorHardwareConfig {
    pub gear: MotorGearType,
    pub pole_pairs: u8,
    /// Reduction ratio scaled by 100 (DM01 = 4027 for 40.27:1).
    pub reduction_ratio_x100: u16,
    /// Hall installation angle error in degrees (`-30..=30`).
    pub angle_offset_deg: i8,
    /// Peak torque / sqrt(continuous power) (`MotorConstant`).
    pub motor_constant: u16,
    /// Start initial loading voltage (`0..=50%` of battery voltage).
    pub start_min_voltage_pct: u8,
    /// Start maximum loading voltage (`0..=70%` of battery voltage).
    pub start_max_voltage_pct: u8,
    /// Start initial speed (`1..=60` RPM).
    pub start_speed_rpm: u8,
    /// Maximum frequency of commutations for BLDC startup (`1..=240`).
    pub start_max_count: u8,
    /// 6-step Hall commutation table.
    pub commute_table: [u8; 6],
}

/// Special FOC vector control parameters (`FOC Menu`, Page 15 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocSpecialConfig {
    /// Motor internal resistance in milliohms (`mΩ`), measured by LCR.
    pub resistance_mohm: u16,
    /// Motor inductance in microhenries (`μH`), measured by LCR.
    pub inductance_uh: u16,
    /// Speed PID Kp value (`5..=200`).
    pub speed_kp: u16,
    /// Speed PID Ki value (`1..=100`).
    pub speed_ki: u16,
    /// Speed feedback smoothing value (`6..=120`).
    pub speed_filter: u8,
    /// Current loop PID Kp value (`20..=5000`).
    pub current_kp: u16,
    /// Current loop PID Ki value (`1..=500`).
    pub current_ki: u16,
    /// Automatically derive CurrentKp and CurrentKi from `resistance_mohm` and `inductance_uh`.
    pub auto_current_pid: bool,
    /// Field-weakening activation threshold (`90..=98%` of battery voltage).
    pub flux_weak_voltage_pct: u8,
    /// Field-weakening maximum demagnetizing current (`0..=10` A; `0` = disabled).
    pub flux_weak_current_a: u8,
}

/// Complete stock parameter set for the DM01 controller matching the official manuals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorConfig {
    /// Number of active assist levels (`1..=8`, factory default 5).
    pub level_count: u8,
    pub cruise_enabled: bool,
    pub slide_bike_enabled: bool,
    pub levels: [AssistLevelConfig; MAX_ASSIST_LEVELS],

    /// Controller current limits (`5A..=50A`, Page 11 of T24 Manual).
    pub max_current_eco_ma: u32,
    pub max_current_city_ma: u32,
    pub max_current_sport_ma: u32,

    /// Battery parameters (`20V..=60V` nominal, `15.0V..=55.0V` undervoltage).
    pub nominal_voltage_mv: u32,
    pub low_cut_voltage_mv: u32,

    /// Drivetrain & wheel parameters (`Device` menu, Page 10-11 of T24 Manual).
    pub wheel_circumference_mm: u16,
    pub rim_speed_sensor_magnets: u8,
    pub crankset_teeth: u8,
    pub cassette_max_teeth: u8,
    pub cassette_min_teeth: u8,
    pub speed_sensor_fail_unlimit: bool,

    /// Motor & FOC parameters.
    pub motor_hw: MotorHardwareConfig,
    pub foc_config: FocSpecialConfig,

    /// Torque Sensor parameters (Page 13-14 of T24 Manual).
    pub torque_sensor_enabled: bool,
    pub torque_mode: TorqueMode,
    /// Zero point voltage (`100..=2000 mV`, default `700 mV`).
    pub torque_zero_vol_mv: u16,
    /// Maximum output voltage (`1000..=3300 mV`, default `3300 mV`).
    pub torque_max_vol_mv: u16,
    /// Net starting voltage (`200..=2000 mV`, default `200 mV`).
    pub torque_startup_net_vol_mv: u16,
    /// Keep angle (`0..=180` degrees).
    pub torque_keep_angle_deg: u8,
    /// Minimum current bias (`0..=10 A`, in mA).
    pub torque_min_out_current_ma: u32,
    /// Torque sensor smooth signal filter (`0..=10`).
    pub torque_filter: u8,
    /// Assist strength = electric power / manpower (`0..=10`).
    pub torque_strength_value: u8,
    /// Torque keep time in ms (default `2000 ms`).
    pub torque_keep_time_ms: u16,

    /// PAS parameters (Page 12-13 of T24 Manual).
    pub pas_type: PasType,
    /// PAS magnet count (`3..=64`).
    pub pas_magnic_count: u8,
    /// PAS starter magnets number (`1..=6`).
    pub pas_startup_count: u8,
    /// Smooth speed filter (`1..=3`).
    pub pas_filter_count: u8,
    /// KeepTime (`200..=1000 ms`, default `600 ms`).
    pub pas_keep_time_ms: u16,
    /// Maximum cadence speed (`60..=200 rpm`, DM01 default `160 rpm`).
    pub pas_max_speed_rpm: u16,
    pub pas_control_mode: PasControlMode,
    pub pas_current_offset_ma: u32,

    /// Throttle parameters (Page 13 of T24 Manual).
    pub throttle_enabled: bool,
    /// Minimum voltage to start throttle (`800..=2000 mV`, default `800 mV`).
    pub throttle_startup_vol_mv: u16,
    /// Speed control mode (`0..=12`, factory default `7`).
    pub throttle_control_mode: u8,

    /// Shift sensor parameters.
    pub shift_sensor_enabled: bool,
    pub shift_delay_ms: u16,
    pub shift_power_percent: u8,

    /// TemperatureSensor parameters (Page 14 of T24 Manual).
    pub temp_ctl_enabled: bool,
    pub temp_ctl_over_mode: OverTempMode,
    pub temp_ctl_min_c: i16,
    pub temp_ctl_max_c: i16,

    pub temp_mot_enabled: bool,
    pub temp_mot_over_mode: OverTempMode,
    pub temp_mot_min_c: i16,
    pub temp_mot_max_c: i16,
}

impl Default for MotorConfig {
    /// Official DM01 48V (1000W) stock factory configuration.
    fn default() -> Self {
        Self {
            level_count: 5,
            cruise_enabled: false,
            slide_bike_enabled: false,
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
            rim_speed_sensor_magnets: 1,
            crankset_teeth: 46,
            cassette_max_teeth: 36,
            cassette_min_teeth: 11,
            speed_sensor_fail_unlimit: false,
            motor_hw: MotorHardwareConfig {
                gear: MotorGearType::Gear,
                pole_pairs: 4,
                reduction_ratio_x100: 4027, // 40.27:1
                angle_offset_deg: 0,
                motor_constant: 100,
                start_min_voltage_pct: 10,
                start_max_voltage_pct: 35,
                start_speed_rpm: 15,
                start_max_count: 60,
                commute_table: [1, 3, 2, 6, 4, 5],
            },
            foc_config: FocSpecialConfig {
                resistance_mohm: 45,
                inductance_uh: 120,
                speed_kp: 40,
                speed_ki: 10,
                speed_filter: 20,
                current_kp: 512,
                current_ki: 64,
                auto_current_pid: true,
                flux_weak_voltage_pct: 95,
                flux_weak_current_a: 0,
            },
            torque_sensor_enabled: true,
            torque_mode: TorqueMode::Safe,
            torque_zero_vol_mv: 700,
            torque_max_vol_mv: 3300,
            torque_startup_net_vol_mv: 200,
            torque_keep_angle_deg: 90,
            torque_min_out_current_ma: 2_000,
            torque_filter: 5,
            torque_strength_value: 10,
            torque_keep_time_ms: 2000,
            pas_type: PasType::Auto,
            pas_magnic_count: 12,
            pas_startup_count: 1,
            pas_filter_count: 2,
            pas_keep_time_ms: 600,
            pas_max_speed_rpm: 160,
            pas_control_mode: PasControlMode::MaxBikeSpeed,
            pas_current_offset_ma: 5_000,
            throttle_enabled: true,
            throttle_startup_vol_mv: 800,
            throttle_control_mode: 7,
            shift_sensor_enabled: true,
            shift_delay_ms: 300,
            shift_power_percent: 10,
            temp_ctl_enabled: true,
            temp_ctl_over_mode: OverTempMode::Power30Percent,
            temp_ctl_min_c: 70,
            temp_ctl_max_c: 100,
            temp_mot_enabled: false,
            temp_mot_over_mode: OverTempMode::Power30Percent,
            temp_mot_min_c: 80,
            temp_mot_max_c: 110,
        }
    }
}

impl MotorConfig {
    /// Returns the mode current limit in mA, compensated by battery voltage (`I_48V * 50V / V_bat`).
    pub fn mode_current_limit_ma(&self, mode: RidingMode, actual_battery_mv: u32) -> u32 {
        let base_ma = match mode {
            RidingMode::Eco => self.max_current_eco_ma,
            RidingMode::City => self.max_current_city_ma,
            RidingMode::Sport => self.max_current_sport_ma,
        };
        if actual_battery_mv == 0 {
            return base_ma;
        }
        let reference_mv = self.nominal_voltage_mv.saturating_add(2_000);
        let scaled = (base_ma as u64 * reference_mv as u64) / (actual_battery_mv as u64);
        let max_allowed = (base_ma * 120) / 100;
        (scaled as u32).min(max_allowed)
    }

    /// Returns the active assist level configuration (`1..=level_count`).
    pub fn active_level(&self, level: u8) -> Option<&AssistLevelConfig> {
        if level == 0 || level > self.level_count {
            return None;
        }
        let idx = (level - 1) as usize;
        self.levels.get(idx)
    }

    /// Returns the maximum configured speed limit across all active levels in 0.1 km/h.
    pub fn max_level_speed_deci_kph(&self) -> u16 {
        let count = (self.level_count as usize).min(MAX_ASSIST_LEVELS);
        self.levels[..count]
            .iter()
            .map(|l| l.speed_limit_deci_kph)
            .max()
            .unwrap_or(400)
    }
}
