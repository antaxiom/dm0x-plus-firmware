/// Bitmask of active controller fault flags reported to the display (flashing yellow triangle).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FaultFlags(pub u16);

impl FaultFlags {
    pub const NONE: Self = Self(0);
    /// Rotor stall, phase overcurrent, or loss of synchronous stator/rotor lock ("Motor Blocked").
    pub const MOTOR_BLOCKED: u16 = 1 << 0;
    /// Torque sensor out of range (< 400 mV or > 3500 mV) or startup calibration timeout.
    pub const TORQUE_SENSOR_FAULT: u16 = 1 << 1;
    /// Throttle Hall sensor disconnected (< 300 mV) or shorted (> 3600 mV).
    pub const THROTTLE_FAULT: u16 = 1 << 2;
    /// Controller temperature reached or exceeded MaxTemp threshold.
    pub const OVER_TEMPERATURE: u16 = 1 << 3;
    /// Battery voltage dropped below LowCutVoltage.
    pub const LOW_VOLTAGE: u16 = 1 << 4;
    /// Wheel speed sensor signal missing during sustained motor drive.
    pub const SPEED_SENSOR_FAIL: u16 = 1 << 5;
    /// UART communication timeout between display and controller ("Communication Interrupted").
    pub const COMM_INTERRUPTED: u16 = 1 << 6;

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

    /// Returns true if any active fault requires immediate motor current shutdown.
    pub const fn inhibits_drive(self, speed_sensor_fail_limits_power: bool) -> bool {
        let mut critical = Self::MOTOR_BLOCKED | Self::THROTTLE_FAULT | Self::LOW_VOLTAGE;
        if speed_sensor_fail_limits_power {
            critical |= Self::SPEED_SENSOR_FAIL;
        }
        (self.0 & critical) != 0
    }
}

/// Stateful fault detector monitoring stall conditions, sensor plausibility, and startup calibration.
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

    /// Runs the power-on torque sensor zero-point calibration check ("Calibrating, don't touch pedal").
    /// Succeeds when torque voltage rests cleanly within `500..=1150 mV` with zero cadence for 300 ms.
    /// Times out after 3000 ms if the rider stands on the pedals or the strain gauge is stuck.
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

    /// Evaluates runtime hardware protection conditions on each periodic control tick.
    pub fn evaluate_runtime(
        &mut self,
        commanded_current_ma: u32,
        rotor_rpm: u16,
        wheel_speed_deci_kph: u16,
        torque_mv: u16,
        throttle_mv: u16,
        over_temp: bool,
        low_voltage: bool,
        dt_ms: u16,
    ) -> FaultFlags {
        // 1. Torque sensor electrical plausibility (400..=3450 mV)
        if !(400..=3450).contains(&torque_mv) {
            self.flags.insert(FaultFlags::TORQUE_SENSOR_FAULT);
        } else if self.torque_calibrated {
            self.flags.remove(FaultFlags::TORQUE_SENSOR_FAULT);
        }

        // 2. Throttle electrical plausibility (350..=3500 mV)
        if !(350..=3500).contains(&throttle_mv) {
            self.flags.insert(FaultFlags::THROTTLE_FAULT);
        } else {
            self.flags.remove(FaultFlags::THROTTLE_FAULT);
        }

        // 3. Motor Blocked (stall protection):
        // If phase/battery current > 12A while rotor RPM is 0 for > 1500 ms, trip MOTOR_BLOCKED
        if commanded_current_ma >= 12_000 && rotor_rpm == 0 {
            self.stall_timer_ms = self.stall_timer_ms.saturating_add(dt_ms);
            if self.stall_timer_ms >= 1500 {
                self.flags.insert(FaultFlags::MOTOR_BLOCKED);
            }
        } else {
            self.stall_timer_ms = 0;
        }

        // 4. Speed sensor failure detection:
        // If motor is actively spinning (rotor_rpm > 500) with > 5A for > 8 seconds but wheel speed is 0
        if commanded_current_ma >= 5_000 && rotor_rpm > 500 && wheel_speed_deci_kph == 0 {
            self.speed_sensor_missing_ms = self.speed_sensor_missing_ms.saturating_add(dt_ms);
            if self.speed_sensor_missing_ms >= 8_000 {
                self.flags.insert(FaultFlags::SPEED_SENSOR_FAIL);
            }
        } else if wheel_speed_deci_kph > 0 {
            self.speed_sensor_missing_ms = 0;
            self.flags.remove(FaultFlags::SPEED_SENSOR_FAIL);
        }

        // 5. Thermal and Low-Voltage flags
        if over_temp {
            self.flags.insert(FaultFlags::OVER_TEMPERATURE);
        } else {
            self.flags.remove(FaultFlags::OVER_TEMPERATURE);
        }

        if low_voltage {
            self.flags.insert(FaultFlags::LOW_VOLTAGE);
        } else {
            self.flags.remove(FaultFlags::LOW_VOLTAGE);
        }

        self.flags
    }
}

