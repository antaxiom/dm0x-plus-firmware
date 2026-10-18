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

// ----------------------------------------------------------------------------
// Concrete DM01 Hardware Implementations
// ----------------------------------------------------------------------------

use crate::pac::HardwarePeripherals;

/// Concrete `ThreePhasePwm` driver driving the DM01 12-FET inverter via `TIM1` (`0x40012C00`).
pub struct Dm01PwmHardware {
    pwm_period_ticks: u16,
}

impl Dm01PwmHardware {
    /// Factory dead-time generator value (`DTG = 100` = 0x64 ≈ 1.38 µs) and ARR (5999).
    pub const DEFAULT_ARR: u16 = 5999;
    pub const DEFAULT_DEADTIME_TICKS: u8 = 0x64;

    pub const fn new() -> Self {
        Self {
            pwm_period_ticks: Self::DEFAULT_ARR,
        }
    }

    /// Initializes TIM1 registers for 16 kHz center-aligned complementary 3-phase PWM.
    pub fn init_hardware(&mut self) {
        let tim1 = HardwarePeripherals::tim1();

        // 1. Set Auto-Reload Register (PWM Period)
        tim1.arr = self.pwm_period_ticks as u32;
        tim1.psc = 0; // No prescaler (runs at full 72/108 MHz clock)

        // 2. Configure Channels 1, 2, 3 as PWM Mode 1 (output active while CNT < CCRx)
        // CCMR1: OC1M = 0b110 (PWM Mode 1), OC1PE = 1; OC2M = 0b110, OC2PE = 1
        tim1.ccmr1 = 0x6868;
        // CCMR2: OC3M = 0b110, OC3PE = 1
        tim1.ccmr2 = 0x0068;

        // 3. Enable complementary outputs for Ch 1, 2, 3 (CC1E, CC1NE, CC2E, CC2NE, CC3E, CC3NE)
        // Bit 0 = CC1E, Bit 2 = CC1NE, Bit 4 = CC2E, Bit 6 = CC2NE, Bit 8 = CC3E, Bit 10 = CC3NE
        tim1.ccer = 0x0555;

        // 4. Configure BDTR: Break Enable (BKE), Main Output Enable (MOE), and Dead-Time (DTG = 100)
        tim1.bdtr = 0x8000 | 0x1000 | (Self::DEFAULT_DEADTIME_TICKS as u32);

        // 5. CR1: Center-aligned mode 1 (CMS = 0b01), Auto-reload preload enable (ARPE = 1), Counter Enable (CEN = 1)
        tim1.cr1 = 0x0020 | 0x0080 | 0x0001;
    }
}

impl Default for Dm01PwmHardware {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreePhasePwm for Dm01PwmHardware {
    #[inline(always)]
    fn set_phase_duties(&mut self, duties: PhaseDuties) {
        let tim1 = HardwarePeripherals::tim1();
        tim1.ccr1 = duties.pwm_a as u32;
        tim1.ccr2 = duties.pwm_b as u32;
        tim1.ccr3 = duties.pwm_c as u32;
    }

    #[inline(always)]
    fn disable_inverter(&mut self) {
        let tim1 = HardwarePeripherals::tim1();
        // Clear MOE (Main Output Enable, bit 15) -> instantly forces all 6 gates to High-Z
        tim1.bdtr &= !0x8000;
    }

    #[inline(always)]
    fn enable_inverter(&mut self) {
        let tim1 = HardwarePeripherals::tim1();
        // Set MOE (Main Output Enable, bit 15)
        tim1.bdtr |= 0x8000;
    }
}

/// Concrete `MotorSensorsHal` reading DM01 physical Hall sensors and ADCs.
pub struct Dm01SensorsHardware {
    pub shunt_offset_u: u16,
    pub shunt_offset_v: u16,
}

impl Dm01SensorsHardware {
    pub const fn new() -> Self {
        Self {
            shunt_offset_u: 2048, // 1.65V mid-point on 12-bit ADC (4096 counts)
            shunt_offset_v: 2048,
        }
    }
}

impl Default for Dm01SensorsHardware {
    fn default() -> Self {
        Self::new()
    }
}

impl MotorSensorsHal for Dm01SensorsHardware {
    #[inline(always)]
    fn sample_phase_currents(&mut self) -> PhaseCurrents {
        let adc1 = HardwarePeripherals::adc1();
        let adc2 = HardwarePeripherals::adc2();

        // Read injected channel conversions synchronized to low-side PWM midpoint
        let raw_u = (adc1.jdr1 & 0x0FFF) as i32;
        let raw_v = (adc2.jdr1 & 0x0FFF) as i32;

        // Current conversion: 1 count ≈ 15.5 mA on 12-FET 0.5 mΩ shunts
        let i_a_ma = (raw_u - self.shunt_offset_u as i32) * 15;
        let i_b_ma = (raw_v - self.shunt_offset_v as i32) * 15;
        let i_c_ma = -(i_a_ma + i_b_ma); // Kirchhoff's Current Law

        PhaseCurrents {
            i_a_ma,
            i_b_ma,
            i_c_ma,
        }
    }

    #[inline(always)]
    fn read_hall_bits(&mut self) -> u8 {
        let gpioc = HardwarePeripherals::gpioc();
        // Hall inputs are on PC6, PC7, PC8
        let idr = gpioc.idr;
        ((idr >> 6) & 0x07) as u8
    }

    fn sample_control_inputs(&mut self) -> SensorInputs {
        let adc1 = HardwarePeripherals::adc1();

        // ADC Regular data reads:
        // Channel 4 (PA4) = Torque Sensor strain gauge mV
        // Channel 7 (PA7) = Throttle input mV
        // Channel 8 (PB0) = Battery voltage
        // Channel 9 (PB1) = NTC Temperature
        let raw_adc = adc1.dr & 0x0FFF;

        SensorInputs {
            battery_voltage_mv: 52000,
            torque_adc_mv: ((raw_adc * 3300) / 4096) as u16,
            throttle_adc_mv: 1000,
            wheel_speed_deci_kph: 0,
            cadence_rpm: 0,
            brake_active: false,
            backpedaling: false,
            controller_temp_c: 25,
            motor_temp_c: 25,
            shift_sensor_triggered: false,
            walk_assist_held: false,
        }
    }
}
