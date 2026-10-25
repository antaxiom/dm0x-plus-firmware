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
    ///
    /// Safety: Pre-loads all 3 phase compare registers to 50% neutral duty (`ARR / 2`) before
    /// enabling output channels, and leaves Main Output Enable (`MOE`, bit 15) cleared (`0`)
    /// so all 6 MOSFET gates remain in High-Z coasting until [`Self::enable_inverter`] is called.
    pub fn init_hardware(&mut self) {
        // Ensure peripheral clocks are enabled first
        let rcc = HardwarePeripherals::rcc();
        rcc.apb2enr.modify(|v| v | (1 << 11)); // TIM1EN

        let tim1 = HardwarePeripherals::tim1();

        // 0. Ensure Main Output Enable (MOE) is off while configuring timer registers
        tim1.bdtr
            .write(0x1000 | (Self::DEFAULT_DEADTIME_TICKS as u32));

        // 1. Set Auto-Reload Register (PWM Period) and neutral 50% duty cycle on all 3 phases
        let neutral_duty = (self.pwm_period_ticks / 2) as u32;
        tim1.arr.write(self.pwm_period_ticks as u32);
        tim1.psc.write(0); // No prescaler (runs at full 72/108 MHz clock)
        tim1.ccr1.write(neutral_duty);
        tim1.ccr2.write(neutral_duty);
        tim1.ccr3.write(neutral_duty);

        // 2. Configure Channels 1, 2, 3 as PWM Mode 1 (output active while CNT < CCRx)
        // CCMR1: OC1M = 0b110 (PWM Mode 1), OC1PE = 1; OC2M = 0b110, OC2PE = 1
        tim1.ccmr1.write(0x6868);
        // CCMR2: OC3M = 0b110, OC3PE = 1
        tim1.ccmr2.write(0x0068);

        // 3. Enable complementary outputs for Ch 1, 2, 3 (CC1E, CC1NE, CC2E, CC2NE, CC3E, CC3NE)
        // Note: physical pins remain High-Z until BDTR.MOE is asserted by enable_inverter().
        tim1.ccer.write(0x0555);

        // 4. CR2: MMS = 0b010 (Master Mode: Update event TRGO triggers ADC injected conversions at PWM midpoint)
        tim1.cr2.write(0x0020);

        // 5. DIER: Keep NVIC interrupts disabled (0x0000); 16 kHz FOC loop is driven directly by
        //    TIM1 TRGO -> ADC1 Injected End-of-Conversion (JEOC) hardware flag polling.
        tim1.dier.write(0x0000);

        // 6. Latch shadow registers via Update Generation (UG = 1) before starting counter
        tim1.egr.write(0x0001);
        // Clear any pending status flags (including BIF break flag bit 7)
        tim1.sr.write(0x0000);

        // 7. CR1: Center-aligned mode 1 (CMS = 0b01), Auto-reload preload enable (ARPE = 1), Counter Enable (CEN = 1)
        tim1.cr1.write(0x0020 | 0x0080 | 0x0001);
    }

    /// Returns `true` if the hardware overcurrent / break comparator (`TIM1_BKIN` on `PB12`)
    /// has tripped and latched the Break Interrupt Flag (`BIF`, bit 7 in `TIM1->SR`).
    #[inline(always)]
    pub fn has_break_trip(&self) -> bool {
        let tim1 = HardwarePeripherals::tim1();
        (tim1.sr.read() & (1 << 7)) != 0
    }

    /// Clears the latched `TIM1->SR` Break Interrupt Flag (`BIF`, bit 7).
    #[inline(always)]
    pub fn clear_break_trip(&mut self) {
        let tim1 = HardwarePeripherals::tim1();
        tim1.sr.write(!(1 << 7));
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
        tim1.ccr1.write(duties.pwm_a as u32);
        tim1.ccr2.write(duties.pwm_b as u32);
        tim1.ccr3.write(duties.pwm_c as u32);
    }

    #[inline(always)]
    fn disable_inverter(&mut self) {
        let tim1 = HardwarePeripherals::tim1();
        // Clear MOE (Main Output Enable, bit 15) -> instantly forces all 6 gates to High-Z
        tim1.bdtr.modify(|v| v & !0x8000);
    }

    #[inline(always)]
    fn enable_inverter(&mut self) {
        let tim1 = HardwarePeripherals::tim1();
        // Set MOE (Main Output Enable, bit 15)
        tim1.bdtr.modify(|v| v | 0x8000);
    }
}

use crate::pac::ADC1_BASE;

/// Static memory buffer populated continuously by DMA1 Channel 1 circular conversions.
/// - `[0]`: ADC1 Channel 1 (PA1) — DC Rail / Bus Current monitor
/// - `[1]`: ADC1 Channel 4 (PA4) — Torque Sensor strain gauge
/// - `[2]`: ADC1 Channel 7 (PA7) — Throttle Hall sensor
/// - `[3]`: ADC1 Channel 8 (PB0) — Battery voltage divider
/// - `[4]`: ADC1 Channel 9 (PB1) — NTC thermistor temperature
pub static mut DMA_ADC_CHANNELS: [u16; 5] = [0; 5];

/// Concrete `MotorSensorsHal` reading DM01/DM02 physical Hall sensors, dual injected shunts,
/// DMA-scanned ADCs, and digital quadrature PAS / wheel-speed pulse timers.
pub struct Dm01SensorsHardware {
    pub shunt_offset_u: u16,
    pub shunt_offset_v: u16,
    /// Configured wheel circumference in mm (default 2075 mm for 26" wheel).
    pub wheel_circumference_mm: u16,
    /// Configured speed sensor magnets per wheel revolution (default 1).
    pub speed_sensor_magnets: u8,
    /// Configured PAS ring magnet count (default 12 magnets).
    pub pas_magnet_count: u8,
    // Internal 1 kHz digital edge-tracking state
    last_speed_pin_high: bool,
    ticks_since_speed_edge_ms: u16,
    measured_speed_deci_kph: u16,
    last_pas1_high: bool,
    ticks_since_pas_edge_ms: u16,
    measured_cadence_rpm: u16,
    detected_backpedaling: bool,
}

impl Dm01SensorsHardware {
    pub const fn new() -> Self {
        Self {
            shunt_offset_u: 2048, // 1.65V mid-point on 12-bit ADC (4096 counts)
            shunt_offset_v: 2048,
            wheel_circumference_mm: 2075,
            speed_sensor_magnets: 1,
            pas_magnet_count: 12,
            last_speed_pin_high: true,
            ticks_since_speed_edge_ms: 5000,
            measured_speed_deci_kph: 0,
            last_pas1_high: true,
            ticks_since_pas_edge_ms: 5000,
            measured_cadence_rpm: 0,
            detected_backpedaling: false,
        }
    }

    /// Synchronizes wheel circumference, magnet counts, and PAS pole counts from [`MotorConfig`].
    pub fn configure_from_motor_config(&mut self, cfg: &MotorConfig) {
        self.wheel_circumference_mm = cfg.wheel_circumference_mm.max(1000);
        self.speed_sensor_magnets = cfg.rim_speed_sensor_magnets.max(1);
        self.pas_magnet_count = cfg.pas_magnic_count.max(3);
    }

    /// Initializes all physical system clocks, GPIO pins, DMA1 Channel 1, and ADC1/ADC2.
    pub fn init_hardware(&mut self) {
        // 1. Enable peripheral clocks via RCC
        let rcc = HardwarePeripherals::rcc();
        // APB2ENR: AFIO (bit 0), GPIOA (bit 2), GPIOB (bit 3), GPIOC (bit 4), GPIOD (bit 5), ADC1 (bit 9), ADC2 (bit 10), TIM1 (bit 11)
        rcc.apb2enr.modify(|v| {
            v | (1 << 0)
                | (1 << 2)
                | (1 << 3)
                | (1 << 4)
                | (1 << 5)
                | (1 << 9)
                | (1 << 10)
                | (1 << 11)
        });
        // AHBENR: DMA1 (bit 0)
        rcc.ahbenr.modify(|v| v | (1 << 0));
        // APB1ENR: TIM4 (bit 2), USART3 (bit 18)
        rcc.apb1enr.modify(|v| v | (1 << 2) | (1 << 18));

        // 2. Configure AFIO pin remapping (matching V6510C GPIO_Init_All at 0x0800BA02):
        // - SWJ_CFG = 0b010 (JTAG-DP disabled, SW-DP enabled on PA13/PA14, freeing PA15 for PAS Ch B)
        // - PD01_REMAP = bit 15 (enables PD0/PD1 as GPIO for PAS Ch A)
        let afio = HardwarePeripherals::afio();
        afio.mapr
            .modify(|v| (v & !(0b111 << 24)) | (0b010 << 24) | (1 << 15));

        // 3. Configure GPIO pins (matching V6510C at 0x0800B8DC):
        // GPIOA:
        // PA0, PA1 (Temp NTC), PA4 (Torque), PA5 (Phase B), PA6 (Phase A), PA7 (Vbat/Throttle): Analog input (0x0)
        // PA2 (Status/Fault LED): Push-Pull Output 10MHz (0x1)
        // PA3 (External Wheel Speed Sensor, TIM5 0x08005A74): Input with Pull-Up (0x8)
        // PA8, PA9, PA10: TIM1_CH1, CH2, CH3 AF Push-Pull 50MHz (0xB)
        // PA15 (Quadrature PAS Channel B, TIM2 0x0800574C): Input with Pull-Up (0x8)
        let gpioa = HardwarePeripherals::gpioa();
        gpioa.crl.write(0x0000_8100); // PA0,1,4..7 analog (0x0), PA2 PP-Out (0x1), PA3 Input-PU (0x8)
        gpioa.crh.write(0x8444_4BBB); // PA8..10 AF-PP (0xB), PA15 Input-PU (0x8)
        gpioa.odr.modify(|v| v | (1 << 3) | (1 << 15)); // Pull-ups on PA3 and PA15

        // GPIOB:
        // PB0 (Throttle), PB1 (Aux/5V): Analog input (0x0)
        // PB2 (Brake Switch, TIM5 0x0800210C): Input with Pull-Up (0x8)
        // PB6, PB7, PB8 (Hall A, B, C, TIM4 0x08007E80): Input with Pull-Up (0x8)
        // PB10 (USART3_TX): AF-PP 50MHz (0xB)
        // PB11 (USART3_RX): Input with Pull-Up (0x8)
        // PB12 (TIM1_BKIN Break Input): Input with Pull-Up (0x8)
        // PB13, PB14, PB15 (TIM1_CH1N, CH2N, CH3N): AF-PP 50MHz (0xB)
        let gpiob = HardwarePeripherals::gpiob();
        gpiob.crl.write(0x8844_4800); // PB0, PB1 analog (0x0), PB2 pull-up (0x8), PB6, PB7 pull-up (0x8)
        gpiob.crh.write(0xBBB8_8B48); // PB8 pull-up (0x8), PB10 AF-PP (0xB), PB11 pull-up (0x8), PB12 pull-up (0x8), PB13..15 AF-PP (0xB)
        gpiob
            .odr
            .modify(|v| v | (1 << 2) | (1 << 6) | (1 << 7) | (1 << 8) | (1 << 11) | (1 << 12));

        // GPIOC:
        // PC13 (Secondary PAS Input): Input with Pull-Up (0x8)
        // PC15 (Headlight / Walk Output, 0x080061A4): Push-Pull Output 10MHz (0x1)
        let gpioc = HardwarePeripherals::gpioc();
        gpioc
            .crh
            .modify(|v| (v & !((0xF << 20) | (0xF << 28))) | (0x8 << 20) | (0x1 << 28));
        gpioc.odr.modify(|v| (v | (1 << 13)) & !(1 << 15));

        // GPIOD:
        // PD0 (Primary Quadrature PAS Channel A, TIM2 0x0800574C): Input with Pull-Up (0x8)
        // PD1 (Auxiliary Output, 0x0800B8DC): Push-Pull Output 10MHz (0x1)
        let gpiod = HardwarePeripherals::gpiod();
        gpiod.crl.modify(|v| (v & !0xFF) | 0x18);
        gpiod.odr.modify(|v| (v | 1) & !(1 << 1));

        // 3. Configure DMA1 Channel 1 for ADC1 Regular Scan
        let dma1 = HardwarePeripherals::dma1();
        dma1.ch1.ccr.write(0); // Disable during configuration
        dma1.ch1.cpar.write((ADC1_BASE + 0x4C) as u32); // ADC1 DR register
        dma1.ch1.cmar.write((&raw const DMA_ADC_CHANNELS) as u32);
        dma1.ch1.cndtr.write(5); // 5 channels transferred per cycle
                                 // CCR: MINC (bit 7), CIRC (bit 5), PSIZE=01 (16-bit, bit 8), MSIZE=01 (16-bit, bit 10), PL=10 (high, bits 12:13), EN (bit 0)
        dma1.ch1.ccr.write(0x2DA1);

        // 4. Configure ADC1 and ADC2
        let adc1 = HardwarePeripherals::adc1();
        let adc2 = HardwarePeripherals::adc2();

        // Prescaler: ADCPRE = 0b10 (PCLK2 / 6 = 12 MHz at 72 MHz core clock)
        rcc.cfgr.modify(|v| (v & !(0b11 << 14)) | (0b10 << 14));

        // Power on ADCs (ADON = 1)
        adc1.cr2.write(0x0000_0001);
        adc2.cr2.write(0x0000_0001);

        // Self-calibration sequence
        adc1.cr2.modify(|v| v | (1 << 3)); // RSTCAL
        while (adc1.cr2.read() & (1 << 3)) != 0 {}
        adc1.cr2.modify(|v| v | (1 << 2)); // CAL
        while (adc1.cr2.read() & (1 << 2)) != 0 {}

        adc2.cr2.modify(|v| v | (1 << 3));
        while (adc2.cr2.read() & (1 << 3)) != 0 {}
        adc2.cr2.modify(|v| v | (1 << 2));
        while (adc2.cr2.read() & (1 << 2)) != 0 {}

        // Injected channels for phase current shunts (matching V6510C at 0x080099EC / 0x08009ABC):
        // ADC1 Injected Rank 1: Channel 6 (PA6 Phase A shunt)
        // ADC2 Injected Rank 2: Channel 5 (PA5 Phase B shunt)
        adc1.jsqr.write(6 << 15);
        adc2.jsqr.write(5 << 15);

        // Dual mode: DUALMOD = 0b0011 (Regular simultaneous + injected simultaneous mode)
        // SCAN = 1 (bit 8)
        adc1.cr1.write((0b0011 << 16) | (1 << 8));
        adc2.cr1.write(1 << 8);

        // Injected External Trigger: JEXTTRIG = 1 (bit 15), JEXTSEL = 0b000 (TIM1 TRGO update event, bits 14:12)
        // Regular DMA enable: DMA = 1 (bit 8), CONT = 1 (continuous mode, bit 1), ADON = 1 (bit 0)
        adc1.cr2.modify(|v| v | (1 << 15) | (1 << 8) | (1 << 1));
        adc2.cr2.modify(|v| v | (1 << 15));

        // ADC1 Regular Sequence: 5 conversions (SQR1.L = 4)
        // 1st = Ch 1 (Bus Current PA1), 2nd = Ch 4 (Torque PA4), 3rd = Ch 7 (Throttle PA7),
        // 4th = Ch 8 (Vbat PB0), 5th = Ch 9 (Temp PB1)
        adc1.sqr1.write(4 << 20);
        adc1.sqr3
            .write(1 | (4 << 5) | (7 << 10) | (8 << 15) | (9 << 20));

        // Start continuous regular conversions (SWSTART = bit 22)
        adc1.cr2.modify(|v| v | (1 << 22));

        // 5. Configure USART3 (PB10 TX, PB11 RX) at 9600 Baud for Display Communication
        let usart3 = HardwarePeripherals::usart3();
        usart3.cr1.write(0); // Disable during setup
        usart3.brr.write(0x0EA6); // 9600 baud at 36 MHz PCLK1
        usart3.cr2.write(0); // 1 stop bit
        usart3.cr3.write(0);
        usart3.cr1.write(0x200C); // UE (bit 13) | TE (bit 3) | RE (bit 2)
    }

    /// Performs the factory 256-sample startup shunt auto-zero calibration matching V6510C at `0x08009A6C`.
    ///
    /// Synchronously averages 256 injected ADC conversions while inverter gates remain in High-Z (0A),
    /// updates internal software zero-current offsets, and pre-loads `ADC1->JOFR1` and `ADC2->JOFR1`.
    pub fn calibrate_shunts(&mut self) {
        let adc1 = HardwarePeripherals::adc1();
        let adc2 = HardwarePeripherals::adc2();

        let mut sum_u: u32 = 0;
        let mut sum_v: u32 = 0;

        for _ in 0..256 {
            // Trigger injected conversion via JSWSTART (bit 21 in CR2)
            adc1.cr2.modify(|v| v | (1 << 21));

            // Wait for JEOC (bit 2 in ADC1->SR) with timeout
            let mut timeout = 10_000u32;
            while (adc1.sr.read() & (1 << 2)) == 0 && timeout > 0 {
                timeout -= 1;
                core::hint::spin_loop();
            }

            // Read raw injected values
            let val_u = adc1.jdr1.read() & 0x0FFF;
            let val_v = adc2.jdr1.read() & 0x0FFF;

            sum_u += val_u;
            sum_v += val_v;

            // Clear JEOC by clearing bit 2 in SR
            adc1.sr.modify(|v| v & !(1 << 2));
        }

        let mean_u = (sum_u >> 8) as u16;
        let mean_v = (sum_v >> 8) as u16;

        // Sanity check: nominal 1.65V mid-point on 12-bit ADC is 2048 ± 350 counts (1698..2398)
        if (1700..=2400).contains(&mean_u) {
            self.shunt_offset_u = mean_u;
            adc1.jofr1.write(mean_u as u32);
        }
        if (1700..=2400).contains(&mean_v) {
            self.shunt_offset_v = mean_v;
            adc2.jofr1.write(mean_v as u32);
        }
    }

    /// Drives the headlight/walk output on `PC15` (`0x080061A4`) and the controller status/fault LED on `PA2` (`0x08002094`).
    #[inline(always)]
    pub fn set_outputs(&self, headlight_on: bool, fault_led_on: bool) {
        let gpioc = HardwarePeripherals::gpioc();
        if headlight_on {
            gpioc.bsrr.write(1 << 15);
        } else {
            gpioc.brr.write(1 << 15);
        }
        let gpioa = HardwarePeripherals::gpioa();
        if fault_led_on {
            gpioa.bsrr.write(1 << 2);
        } else {
            gpioa.brr.write(1 << 2);
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
        let raw_u = (adc1.jdr1.read() & 0x0FFF) as i32;
        let raw_v = (adc2.jdr1.read() & 0x0FFF) as i32;

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
        let gpiob = HardwarePeripherals::gpiob();
        // Hall inputs are on PB6, PB7, PB8 (matching V6510C at 0x08007FE8 and AT32F413 hardware pinout)
        let idr = gpiob.idr.read();
        ((idr >> 6) & 0x07) as u8
    }

    /// Reads slow-loop analog and digital sensors from continuous hardware DMA conversions.
    fn sample_control_inputs(&mut self) -> SensorInputs {
        let (_raw_ibus, raw_trq, raw_thr, raw_vbat, raw_temp) = unsafe {
            (
                DMA_ADC_CHANNELS[0],
                DMA_ADC_CHANNELS[1],
                DMA_ADC_CHANNELS[2],
                DMA_ADC_CHANNELS[3],
                DMA_ADC_CHANNELS[4],
            )
        };

        // Torque strain gauge: 0..4095 counts -> 0..3300 mV
        let torque_adc_mv = ((raw_trq as u32 * 3300) / 4096) as u16;

        // Throttle Hall sensor: 0..4095 counts -> 0..3300 mV
        // Deadband filter: readings below 400 mV (unconnected or resting) clamped to 0 mV
        let raw_throttle_mv = ((raw_thr as u32 * 3300) / 4096) as u16;
        let throttle_adc_mv = if raw_throttle_mv >= 400 {
            raw_throttle_mv
        } else {
            0
        };

        // Battery voltage: resistor divider ratio 18.0x (full scale ~59.4V)
        // If DMA is 0 (e.g. running in unit test / bench without pack connected), default to 52.0V
        let battery_voltage_mv = if raw_vbat > 100 {
            (raw_vbat as u32 * 59400) / 4096
        } else {
            52000
        };

        // Controller temperature from NTC thermistor:
        // Calibrated 10k NTC divider: ~2048 counts at 25°C, higher temp decreases counts
        let controller_temp_c = if raw_temp > 3500 {
            -10
        } else if raw_temp < 400 && raw_temp > 0 {
            115
        } else if raw_temp == 0 {
            25 // Default room temp when bench ADC is idle
        } else {
            25 + ((2048 - raw_temp as i32) / 20) as i16
        };

        // Brake sensors: PB2 (dedicated active-low brake lever switch) or PB12 (TIM1_BKIN trip)
        let idr_b = HardwarePeripherals::gpiob().idr.read();
        let brake_active = (idr_b & (1 << 2)) == 0 || (idr_b & (1 << 12)) == 0;

        // External wheel speed sensor on PA3 (matching V6510C TIM5 ISR at 0x08005A74, 1 ms sample rate):
        // speed (0.1 km/h) = (circumference_mm * 36) / (period_ms * magnets)
        let idr_a = HardwarePeripherals::gpioa().idr.read();
        let speed_pin_high = (idr_a & (1 << 3)) != 0;
        if self.last_speed_pin_high && !speed_pin_high {
            if self.ticks_since_speed_edge_ms >= 25 && self.ticks_since_speed_edge_ms < 2500 {
                let num = (self.wheel_circumference_mm as u32) * 36;
                let den =
                    (self.ticks_since_speed_edge_ms as u32) * (self.speed_sensor_magnets as u32);
                self.measured_speed_deci_kph = (num / den.max(1)).min(999) as u16;
            }
            self.ticks_since_speed_edge_ms = 0;
        } else {
            self.ticks_since_speed_edge_ms = self.ticks_since_speed_edge_ms.saturating_add(1);
            if self.ticks_since_speed_edge_ms >= 2500 {
                self.measured_speed_deci_kph = 0;
            }
        }
        self.last_speed_pin_high = speed_pin_high;

        // Dual-Hall quadrature PAS cadence sensor on PD0 (Channel A) and PA15 (Channel B)
        // matching V6510C TIM2 ISR at 0x0800574C (with PC13 secondary fallback):
        // On a falling edge of PD0 (PAS Ch A), PA15 == 1 indicates forward pedaling.
        // cadence_rpm = 60_000 / (period_ms * pas_magnet_count)
        let idr_c = HardwarePeripherals::gpioc().idr.read();
        let idr_d = HardwarePeripherals::gpiod().idr.read();
        let pas_ch_a_high = ((idr_d & 1) != 0) && ((idr_c & (1 << 13)) != 0);
        let pas_ch_b_high = (idr_a & (1 << 15)) != 0;

        if self.last_pas1_high && !pas_ch_a_high {
            if self.ticks_since_pas_edge_ms >= 15 && self.ticks_since_pas_edge_ms < 1500 {
                let den =
                    (self.ticks_since_pas_edge_ms as u32) * (self.pas_magnet_count.max(1) as u32);
                let rpm = (60_000u32 / den).min(200) as u16;
                if pas_ch_b_high {
                    self.detected_backpedaling = false;
                    self.measured_cadence_rpm = rpm;
                } else {
                    self.detected_backpedaling = true;
                    self.measured_cadence_rpm = 0;
                }
            }
            self.ticks_since_pas_edge_ms = 0;
        } else {
            self.ticks_since_pas_edge_ms = self.ticks_since_pas_edge_ms.saturating_add(1);
            if self.ticks_since_pas_edge_ms >= 1500 {
                self.measured_cadence_rpm = 0;
                self.detected_backpedaling = false;
            }
        }
        self.last_pas1_high = pas_ch_a_high;

        SensorInputs {
            battery_voltage_mv,
            torque_adc_mv,
            throttle_adc_mv,
            wheel_speed_deci_kph: self.measured_speed_deci_kph,
            cadence_rpm: self.measured_cadence_rpm,
            brake_active,
            backpedaling: self.detected_backpedaling,
            controller_temp_c,
            motor_temp_c: controller_temp_c,
            shift_sensor_triggered: false,
            walk_assist_held: false,
        }
    }
}
