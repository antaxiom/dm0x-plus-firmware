//! # Motor & Display Configuration Tables (`DM01` & `DM02`)
//!
//! Defines the complete configuration parameters for ToSeven DM01 (`TS1000W`) and DM02 (`TS500W`)
//! mid-drive motors and DM0X displays (`T24`, `T154`, `M15`, `LCD135C`).

use crate::eeprom::EepromImage;

/// Maximum number of configurable assist levels supported by the display/controller (`1..=8`).
pub const MAX_ASSIST_LEVELS: usize = 8;

/// Global riding mode selected via the Display 'I' (Info) button (Page 4 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RidingMode {
    Eco,
    City,
    Sport,
}

impl RidingMode {
    pub const fn to_wire_u8(self) -> u8 {
        match self {
            RidingMode::City => 0,
            RidingMode::Eco => 1,
            RidingMode::Sport => 2,
        }
    }

    pub const fn from_wire_u8(val: u8) -> Self {
        match val {
            1 => RidingMode::Eco,
            2 => RidingMode::Sport,
            _ => RidingMode::City,
        }
    }
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
    /// Current limit percentage (`0..=100%`) applied to PAS and Throttle (`Pwr[level]`).
    pub power_percent: u8,
    /// EAssist factor (`0..=100%`) applied to Torque Sensor sensitivity (`EAssist[level]`).
    pub eassist_percent: u8,
}

/// Motor hardware and BLDC/FOC startup parameters (`Motor` menu, Page 12 of T24 Manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotorHardwareConfig {
    pub gear: MotorGearType,
    pub pole_pairs: u8,
    /// Reduction ratio scaled by 100 (DM01 = 4027 for 40.27:1, DM02 = 5300 for 53.00:1).
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

impl MotorHardwareConfig {
    pub const fn default_dm01() -> Self {
        Self {
            gear: MotorGearType::Gear,
            pole_pairs: 4,
            reduction_ratio_x100: 4027, // 40.27 : 1
            angle_offset_deg: 0,
            motor_constant: 100,
            start_min_voltage_pct: 10,
            start_max_voltage_pct: 35,
            start_speed_rpm: 15,
            start_max_count: 60,
            commute_table: [1, 3, 2, 6, 4, 5],
        }
    }

    pub const fn default_dm02() -> Self {
        Self {
            gear: MotorGearType::Gear,
            pole_pairs: 4,
            reduction_ratio_x100: 5300, // 53.00 : 1
            angle_offset_deg: 0,
            motor_constant: 100,
            start_min_voltage_pct: 10,
            start_max_voltage_pct: 35,
            start_speed_rpm: 15,
            start_max_count: 60,
            commute_table: [1, 3, 2, 6, 4, 5],
        }
    }
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

/// Stock factory continuous current ceiling for DM01 (30.0 A).
pub const DM01_STOCK_MAX_CURRENT_MA: u32 = 30_000;
/// Absolute maximum allowed current ceiling for DM01 (32.0 A).
pub const DM01_ABSOLUTE_MAX_CURRENT_MA: u32 = 32_000;

/// Stock factory continuous current ceiling for DM02 (18.0 A).
pub const DM02_STOCK_MAX_CURRENT_MA: u32 = 18_000;
/// Absolute maximum allowed current ceiling for DM02 (20.0 A).
pub const DM02_ABSOLUTE_MAX_CURRENT_MA: u32 = 20_000;

/// Complete parameter set for the DM01 / DM02 controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorConfig {
    /// Number of active assist levels (`1..=8`, factory default 5).
    pub level_count: u8,
    pub cruise_enabled: bool,
    pub slide_bike_enabled: bool,
    pub levels: [AssistLevelConfig; MAX_ASSIST_LEVELS],

    /// Controller current limits (`5A..=32A` in mA; V6510C DM01 defaults: 10A Eco, 18A City, 30A Sport; DM02: 8A, 12A, 18A).
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

    /// Torque Sensor parameters (Page 13-14 of T24 Manual & V6510C binary).
    pub torque_sensor_enabled: bool,
    pub torque_mode: TorqueMode,
    /// Zero point voltage (`400..=1000 mV`, default `700 mV`).
    pub torque_zero_vol_mv: u16,
    /// Maximum output voltage (`1000..=3300 mV`, default `3300 mV`).
    pub torque_max_vol_mv: u16,
    /// Net starting voltage (`50..=2000 mV`, V6510C default `50 mV`).
    pub torque_startup_net_vol_mv: u16,
    /// Keep angle (`0..=180` degrees).
    pub torque_keep_angle_deg: u8,
    /// Starting current offset (`CurrentOffset`, V6510C default `500 mA = 0.5 A`).
    pub torque_min_out_current_ma: u32,
    /// Torque sensor smooth signal filter (`0..=10`, default `5`).
    pub torque_filter: u8,
    /// Assist strength (`0..=10`, default `10`).
    pub torque_strength_value: u8,
    /// Torque keep time in ms (default `200 ms`).
    pub torque_keep_time_ms: u16,

    /// PAS parameters (Page 12-13 of T24 Manual & V6510C binary).
    pub pas_type: PasType,
    /// PAS magnet count (`3..=64`).
    pub pas_magnic_count: u8,
    /// PAS starter magnets number (`1..=6`).
    pub pas_startup_count: u8,
    /// Smooth speed filter (`1..=3`).
    pub pas_filter_count: u8,
    /// KeepTime (`200..=1000 ms`, default `300 ms`).
    pub pas_keep_time_ms: u16,
    /// Reference cadence speed (`PAS_MaxSpeed`, V6510C default `60 rpm`).
    pub pas_max_speed_rpm: u16,
    pub pas_control_mode: PasControlMode,
    /// PAS starting current offset (`PAS_CurrentOffset`, V6510C default `1000 mA = 1.0 A`).
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

    /// TemperatureSensor parameters (Page 14 of T24 Manual & V6510C linear rollback).
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
    /// Returns the DM01 (`TS1000W`) `V6510C` factory configuration.
    fn default() -> Self {
        Self::default_dm01()
    }
}

impl MotorConfig {
    /// Exact factory default configuration for the ToSeven DM01 (`TS1000W`, `V6510C`).
    pub const fn default_dm01() -> Self {
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
                    power_percent: 40,
                    eassist_percent: 40,
                },
                AssistLevelConfig {
                    speed_limit_deci_kph: 200, // 20.0 km/h
                    power_percent: 60,
                    eassist_percent: 60,
                },
                AssistLevelConfig {
                    speed_limit_deci_kph: 320, // 32.0 km/h
                    power_percent: 80,
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
            max_current_eco_ma: 10_000,   // 10.0 A (100 dA)
            max_current_city_ma: 18_000,  // 18.0 A (180 dA)
            max_current_sport_ma: 30_000, // 30.0 A (300 dA)
            nominal_voltage_mv: 48_000,
            low_cut_voltage_mv: 41_000,
            wheel_circumference_mm: 2075,
            rim_speed_sensor_magnets: 1,
            crankset_teeth: 46,
            cassette_max_teeth: 32,
            cassette_min_teeth: 11,
            speed_sensor_fail_unlimit: true,
            motor_hw: MotorHardwareConfig {
                gear: MotorGearType::Gear,
                pole_pairs: 4,
                reduction_ratio_x100: 4027, // 40.27 : 1
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
            torque_startup_net_vol_mv: 50,
            torque_keep_angle_deg: 90,
            torque_min_out_current_ma: 500, // 0.5 A (5 dA)
            torque_filter: 5,
            torque_strength_value: 10,
            torque_keep_time_ms: 200,
            pas_type: PasType::Auto,
            pas_magnic_count: 12,
            pas_startup_count: 1,
            pas_filter_count: 2,
            pas_keep_time_ms: 300,
            pas_max_speed_rpm: 60,
            pas_control_mode: PasControlMode::MaxBikeSpeed,
            pas_current_offset_ma: 1_000, // 1.0 A (10 dA)
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

    /// Exact factory default configuration for the ToSeven DM02 (`TS500W`, `V6510C`).
    pub const fn default_dm02() -> Self {
        let mut cfg = Self::default_dm01();
        cfg.max_current_eco_ma = 8_000;
        cfg.max_current_city_ma = 12_000;
        cfg.max_current_sport_ma = 18_000;
        cfg.motor_hw.reduction_ratio_x100 = 5300; // 53.00 : 1
        cfg.shift_sensor_enabled = false;
        cfg.crankset_teeth = 42; // TS500W factory default
        cfg.cassette_max_teeth = 32;
        cfg.cassette_min_teeth = 11;
        cfg.wheel_circumference_mm = 2075;
        cfg.levels[0].power_percent = 25;
        cfg.levels[0].eassist_percent = 30;
        cfg.levels[1].power_percent = 35;
        cfg.levels[1].eassist_percent = 50;
        cfg.levels[2].power_percent = 50;
        cfg.levels[2].eassist_percent = 60;
        cfg.levels[3].power_percent = 70;
        cfg.levels[3].eassist_percent = 80;
        cfg.levels[4].power_percent = 100;
        cfg.levels[4].eassist_percent = 100;
        cfg
    }

    /// Synchronizes `Pwr[0..5]`, `EAssist[0..5]`, sprockets, and wheel size from a decoded 480-byte `EepromImage`.
    pub fn apply_eeprom_image(&mut self, eeprom: &EepromImage) {
        let pwr = eeprom.assist_pwr_table();
        let eassist = eeprom.assist_eassist_table();
        let mut i = 0;
        while i < 5 {
            self.levels[i].power_percent = pwr[i];
            self.levels[i].eassist_percent = eassist[i];
            i += 1;
        }
        let crank = eeprom.crankset_teeth();
        if crank > 0 {
            self.crankset_teeth = crank;
        }
        let max_t = eeprom.cassette_max_teeth();
        if max_t > 0 {
            self.cassette_max_teeth = max_t;
        }
        let min_t = eeprom.cassette_min_teeth();
        if min_t > 0 {
            self.cassette_min_teeth = min_t;
        }
        let inches = eeprom.wheel_diameter_inches();
        if inches > 0 {
            self.wheel_circumference_mm = ((inches as u32 * 798) / 10) as u16;
        }
        self.speed_sensor_fail_unlimit = eeprom.speed_sensor_fail_unlimit();
    }

    /// Computes the midpoint gear ratio in fixed-point Q8 (256 = 1.0) from configured sprockets:
    /// `ratio_mid = (Crankset / MinTeeth + Crankset / MaxTeeth) / 2`.
    pub fn midpoint_gear_ratio_q8(&self) -> u16 {
        let crank = (self.crankset_teeth as u32).max(1);
        let min_t = (self.cassette_min_teeth as u32).max(1);
        let max_t = (self.cassette_max_teeth as u32).max(min_t);
        let ratio_high = (crank * 256) / min_t;
        let ratio_low = (crank * 256) / max_t;
        ((ratio_high + ratio_low) / 2) as u16
    }

    /// Estimates vehicle speed in 0.1 km/h units from crank cadence using the midpoint gear ratio
    /// and configured wheel circumference in mm.
    pub fn estimate_speed_deci_kph(&self, cadence_rpm: u16) -> u16 {
        let circ_mm = (self.wheel_circumference_mm as u64).max(1000);
        let ratio_q8 = self.midpoint_gear_ratio_q8() as u64;
        let num = (cadence_rpm as u64) * ratio_q8 * circ_mm * 3;
        ((num + 640_000) / 1_280_000) as u16
    }

    /// Returns the mode current limit in mA, applying the V6510C constant-power cap above 50.0 V,
    /// bounded by the absolute hardware limit of 32.0 A.
    pub fn mode_current_limit_ma(&self, mode: RidingMode, actual_battery_mv: u32) -> u32 {
        let base_ma = match mode {
            RidingMode::Eco => self.max_current_eco_ma,
            RidingMode::City => self.max_current_city_ma,
            RidingMode::Sport => self.max_current_sport_ma,
        }
        .min(DM01_ABSOLUTE_MAX_CURRENT_MA);
        if actual_battery_mv <= 50_000 {
            base_ma
        } else {
            ((base_ma as u64 * 50_000) / (actual_battery_mv as u64)) as u32
        }
    }

    /// Enforces hard current ceilings: 32.0 A for DM01 and 20.0 A for DM02.
    ///
    /// Clamps `max_current_sport_ma`, `max_current_city_ma`, and `max_current_eco_ma` so they
    /// never exceed the hardware-enforced limits (and ensures minimum of 5.0 A).
    ///
    /// Returns `true` if current is within factory stock ratings (<=30A for DM01, <=18A for DM02),
    /// or `false` if exceeding stock ratings (indicating an over-stock warning is warranted).
    pub fn enforce_current_limits(&mut self, is_dm02: bool) -> bool {
        let max_allowed = if is_dm02 {
            DM02_ABSOLUTE_MAX_CURRENT_MA
        } else {
            DM01_ABSOLUTE_MAX_CURRENT_MA
        };
        let stock_max = if is_dm02 {
            DM02_STOCK_MAX_CURRENT_MA
        } else {
            DM01_STOCK_MAX_CURRENT_MA
        };

        self.max_current_sport_ma = self.max_current_sport_ma.clamp(5_000, max_allowed);
        self.max_current_city_ma = self
            .max_current_city_ma
            .clamp(5_000, self.max_current_sport_ma);
        self.max_current_eco_ma = self
            .max_current_eco_ma
            .clamp(5_000, self.max_current_city_ma);

        self.max_current_sport_ma <= stock_max
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dm01_default_within_stock_limits() {
        let mut cfg = MotorConfig::default_dm01();
        assert_eq!(cfg.max_current_sport_ma, 30_000);
        let within_stock = cfg.enforce_current_limits(false);
        assert!(
            within_stock,
            "DM01 stock default (30A) must be within stock limits"
        );
        assert_eq!(cfg.max_current_sport_ma, 30_000);
    }

    #[test]
    fn test_dm02_default_within_stock_limits() {
        let mut cfg = MotorConfig::default_dm02();
        assert_eq!(cfg.max_current_sport_ma, 18_000);
        let within_stock = cfg.enforce_current_limits(true);
        assert!(
            within_stock,
            "DM02 stock default (18A) must be within stock limits"
        );
        assert_eq!(cfg.max_current_sport_ma, 18_000);
    }

    #[test]
    fn test_dm01_hard_cap_32a_and_warning() {
        let mut cfg = MotorConfig::default_dm01();
        // Request 35A (exceeds 32A hard cap)
        cfg.max_current_sport_ma = 35_000;
        let within_stock = cfg.enforce_current_limits(false);
        assert!(
            !within_stock,
            "35A exceeds stock 30A rating and must flag warning"
        );
        assert_eq!(
            cfg.max_current_sport_ma, 32_000,
            "Must be clamped to 32.0 A hard cap"
        );

        // Request 31A (within 32A hard cap, but exceeds 30A stock)
        cfg.max_current_sport_ma = 31_000;
        let within_stock = cfg.enforce_current_limits(false);
        assert!(
            !within_stock,
            "31A exceeds stock 30A rating and must flag warning"
        );
        assert_eq!(
            cfg.max_current_sport_ma, 31_000,
            "31A is allowed under 32A hard cap"
        );
    }

    #[test]
    fn test_dm02_hard_cap_20a_and_warning() {
        let mut cfg = MotorConfig::default_dm02();
        // Request 25A (exceeds 20A hard cap)
        cfg.max_current_sport_ma = 25_000;
        let within_stock = cfg.enforce_current_limits(true);
        assert!(
            !within_stock,
            "25A exceeds stock 18A rating and must flag warning"
        );
        assert_eq!(
            cfg.max_current_sport_ma, 20_000,
            "Must be clamped to 20.0 A hard cap"
        );

        // Request 19A (within 20A hard cap, but exceeds 18A stock)
        cfg.max_current_sport_ma = 19_000;
        let within_stock = cfg.enforce_current_limits(true);
        assert!(
            !within_stock,
            "19A exceeds stock 18A rating and must flag warning"
        );
        assert_eq!(
            cfg.max_current_sport_ma, 19_000,
            "19A is allowed under 20A hard cap"
        );
    }

    #[test]
    fn test_mode_current_limit_hard_cap() {
        let mut cfg = MotorConfig::default_dm01();
        cfg.max_current_sport_ma = 50_000; // Malformed setting
                                           // At 48V (<= 50V), it must never exceed 32.0 A
        let limited = cfg.mode_current_limit_ma(RidingMode::Sport, 48_000);
        assert_eq!(limited, 32_000);
    }
}
