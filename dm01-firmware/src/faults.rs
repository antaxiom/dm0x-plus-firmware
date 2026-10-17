/// Bitmask of official controller and display fault flags documented on Pages 16-17
/// of the T24 Display Manual and Pages 33-34 of the DM01 User Manual v2.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FaultFlags(pub u16);

impl FaultFlags {
    pub const NONE: Self = Self(0);
    /// Communication encryption index number and controller inconsistency ("Protocol Dismatch").
    pub const PROTOCOL_DISMATCH: u16 = 1 << 0;
    /// No communication between display and drive unit controller ("Communication Interrupted").
    pub const COMMUNICATION_INTERRUPTED: u16 = 1 << 1;
    /// Controller memory hardware failure or parameter verification failure ("Controller memory Fail").
    pub const CONTROLLER_MEMORY_FAIL: u16 = 1 << 2;
    /// Throttle Hall sensor out of valid range ("Gallop Throttle Fail").
    pub const GALLOP_THROTTLE_FAIL: u16 = 1 << 3;
    /// Controller MOSFET bridge damaged or short-circuit fault ("Controller Mosfet Fail").
    pub const CONTROLLER_MOSFET_FAIL: u16 = 1 << 4;
    /// Motor rotation restricted due to abnormal resistance, stall, or phase overload ("Motor Blocked").
    pub const MOTOR_BLOCKED: u16 = 1 << 5;
    /// Wheel speed sensor signal missing during operation ("Speed Sensor Fail").
    pub const SPEED_SENSOR_FAIL: u16 = 1 << 6;
    /// Motor Hall sensor state invalid (`0b000` or `0b111`) ("Motor Hall Fail").
    pub const MOTOR_HALL_FAIL: u16 = 1 << 7;
    /// Controller temperature reached `MaxTemp(CTL)` ("Controller OverTemp").
    pub const CONTROLLER_OVER_TEMP: u16 = 1 << 8;
    /// Motor temperature reached `MaxTemp(MOT)` ("Motor OverTemp").
    pub const MOTOR_OVER_TEMP: u16 = 1 << 9;
    /// Torque sensor calibration timeout at startup or out-of-range voltage.
    pub const TORQUE_SENSOR_FAULT: u16 = 1 << 10;
    /// Battery undervoltage below `LowCutVoltage`.
    pub const LOW_VOLTAGE: u16 = 1 << 11;

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, flag: u16) -> bool {
        (self.0 & flag) != 0
    }

    pub fn insert(&mut self, flag: u16) {
        self.0 |= flag;
    }

    pub fn remove(&mut self, flag: u16) {
        self.0 &= !flag;
    }

    /// Returns true if any active fault requires shutting off motor phase drive.
    pub const fn inhibits_drive(self, speed_sensor_fail_unlimit: bool) -> bool {
        let mut critical = Self::PROTOCOL_DISMATCH
            | Self::CONTROLLER_MEMORY_FAIL
            | Self::GALLOP_THROTTLE_FAIL
            | Self::CONTROLLER_MOSFET_FAIL
            | Self::MOTOR_BLOCKED
            | Self::MOTOR_HALL_FAIL
            | Self::LOW_VOLTAGE;
        if !speed_sensor_fail_unlimit {
            critical |= Self::SPEED_SENSOR_FAIL;
        }
        (self.0 & critical) != 0
    }
}

/// Stateful fault detector monitoring startup torque calibration and runtime error conditions.
#[derive(Debug, Clone)]
pub struct FaultDetector {
    pub flags: FaultFlags,
    pub torque_calibrated: bool,
    calibration_elapsed_ms: u16,
    stall_timer_ms: u16,
    speed_sensor_missing_ms: u16,
}

impl FaultDetector {
    pub const fn new() -> Self {
        Self {
            flags: FaultFlags::NONE,
            torque_calibrated: false,
            calibration_elapsed_ms: 0,
            stall_timer_ms: 0,
            speed_sensor_missing_ms: 0,
        }
    }

    /// Power-on zero-point calibration check (DM01 User Manual Page 31).
    /// Records the resting zero reference when pedals are unloaded (`500..=1150 mV`, `0 RPM`) for 300 ms.
    pub fn step_startup_calibration(
        &mut self,
        torque_mv: u16,
        cadence_rpm: u16,
        dt_ms: u16,
    ) -> Option<u16> {
        if self.torque_calibrated {
            return Some(torque_mv);
        }

        if (500..=1150).contains(&torque_mv) && cadence_rpm == 0 {
            self.calibration_elapsed_ms = self.calibration_elapsed_ms.saturating_add(dt_ms);
            if self.calibration_elapsed_ms >= 300 {
                self.torque_calibrated = true;
                self.flags.remove(FaultFlags::TORQUE_SENSOR_FAULT);
                return Some(torque_mv);
            }
        } else {
            self.calibration_elapsed_ms = self.calibration_elapsed_ms.saturating_add(dt_ms);
            if self.calibration_elapsed_ms >= 3000 {
                self.flags.insert(FaultFlags::TORQUE_SENSOR_FAULT);
            }
        }
        None
    }

    /// Evaluates runtime faults on each periodic control tick.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_runtime(
        &mut self,
        commanded_current_ma: u32,
        rotor_rpm: u16,
        wheel_speed_deci_kph: u16,
        torque_mv: u16,
        throttle_mv: u16,
        hall_bits: u8,
        ctl_over_temp: bool,
        mot_over_temp: bool,
        low_voltage: bool,
        dt_ms: u16,
    ) -> FaultFlags {
        // 1. Hall sensor validity (`0b001..=0b110`)
        let masked_hall = hall_bits & 0b111;
        if masked_hall == 0b000 || masked_hall == 0b111 {
            self.flags.insert(FaultFlags::MOTOR_HALL_FAIL);
        } else {
            self.flags.remove(FaultFlags::MOTOR_HALL_FAIL);
        }

        // 2. Torque sensor plausibility (`100..=3450 mV`)
        if !(100..=3450).contains(&torque_mv) {
            self.flags.insert(FaultFlags::TORQUE_SENSOR_FAULT);
        } else if self.torque_calibrated {
            self.flags.remove(FaultFlags::TORQUE_SENSOR_FAULT);
        }

        // 3. Gallop Throttle Fail (`350..=3500 mV`)
        if !(350..=3500).contains(&throttle_mv) {
            self.flags.insert(FaultFlags::GALLOP_THROTTLE_FAIL);
        } else {
            self.flags.remove(FaultFlags::GALLOP_THROTTLE_FAIL);
        }

        // 4. Motor Blocked (stall / overload protection)
        if commanded_current_ma >= 12_000 && rotor_rpm == 0 {
            self.stall_timer_ms = self.stall_timer_ms.saturating_add(dt_ms);
            if self.stall_timer_ms >= 1500 {
                self.flags.insert(FaultFlags::MOTOR_BLOCKED);
            }
        } else {
            self.stall_timer_ms = 0;
        }

        // 5. Speed Sensor Fail
        if commanded_current_ma >= 5_000 && rotor_rpm > 500 && wheel_speed_deci_kph == 0 {
            self.speed_sensor_missing_ms = self.speed_sensor_missing_ms.saturating_add(dt_ms);
            if self.speed_sensor_missing_ms >= 8_000 {
                self.flags.insert(FaultFlags::SPEED_SENSOR_FAIL);
            }
        } else if wheel_speed_deci_kph > 0 {
            self.speed_sensor_missing_ms = 0;
            self.flags.remove(FaultFlags::SPEED_SENSOR_FAIL);
        }

        // 6. Over-temperature and undervoltage flags
        if ctl_over_temp {
            self.flags.insert(FaultFlags::CONTROLLER_OVER_TEMP);
        } else {
            self.flags.remove(FaultFlags::CONTROLLER_OVER_TEMP);
        }

        if mot_over_temp {
            self.flags.insert(FaultFlags::MOTOR_OVER_TEMP);
        } else {
            self.flags.remove(FaultFlags::MOTOR_OVER_TEMP);
        }

        if low_voltage {
            self.flags.insert(FaultFlags::LOW_VOLTAGE);
        } else {
            self.flags.remove(FaultFlags::LOW_VOLTAGE);
        }

        self.flags
    }
}
