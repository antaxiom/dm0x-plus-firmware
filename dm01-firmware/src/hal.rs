//! # Hardware Abstraction Layer (HAL) Traits
//!
//! Defines hardware-agnostic traits for the 3-phase PWM inverter timer, ADC/Hall sensor
//! peripherals, and Flash EEPROM configuration persistence.

use crate::config::MotorConfig;
use crate::control::SensorInputs;
use crate::foc::{PhaseCurrents, PhaseDuties};

/// Hardware Abstraction Layer (HAL) trait for the 3-phase inverter PWM peripheral.
pub trait ThreePhasePwm {
    /// Applies 3-phase duty cycles atomically at the next timer update event.
    fn set_phase_duties(&mut self, duties: PhaseDuties);
    /// Immediately disables all 6 gate-driver outputs (high-Z coasting) on fault or brake cutoff.
    fn disable_inverter(&mut self);
    /// Enables the gate-driver bridge.
    fn enable_inverter(&mut self);
}

/// Hardware Abstraction Layer (HAL) trait for high-speed ADC and Hall sensor sampling.
pub trait MotorSensorsHal {
    /// Reads instantaneous 3-phase shunt currents in milliamps (called at PWM ISR rate).
    fn sample_phase_currents(&mut self) -> PhaseCurrents;
    /// Reads the 3-bit Hall sensor state (`0b001..=0b110`).
    fn read_hall_bits(&mut self) -> u8;
    /// Samples slow-loop sensors (battery voltage, torque strain gauge, throttle, cadence, brakes, temp).
    fn sample_control_inputs(&mut self) -> SensorInputs;
}

/// Error returned when persisting configuration to non-volatile storage fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageError;

/// Non-volatile Flash / EEPROM storage trait for persisting user-configured `MotorConfig` profiles.
pub trait ConfigStorageHal {
    fn load_config(&mut self) -> Option<MotorConfig>;
    fn save_config(&mut self, config: &MotorConfig) -> Result<(), StorageError>;
}
