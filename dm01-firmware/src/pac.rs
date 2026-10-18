//! # Minimal Peripheral Access Crate (PAC) for STM32/GD32 Cortex-M4F
//!
//! Provides direct, `#![no_std]` memory-mapped register definitions for the DM01
//! hardware peripherals (`TIM1`, `ADC1/2`, `USART3`, `GPIOA..GPIOD`, and `SCB`).

use core::ptr::{read_volatile, write_volatile};

pub const TIM1_BASE: usize = 0x4001_2C00;
pub const ADC1_BASE: usize = 0x4001_2400;
pub const ADC2_BASE: usize = 0x4001_2800;
pub const GPIOA_BASE: usize = 0x4001_0800;
pub const GPIOB_BASE: usize = 0x4001_0C00;
pub const GPIOC_BASE: usize = 0x4001_1000;
pub const GPIOD_BASE: usize = 0x4001_1400;
pub const AFIO_BASE: usize = 0x4001_0000;
pub const USART3_BASE: usize = 0x4000_4800;
pub const FLASH_BASE_REG: usize = 0x4002_2000;
pub const SCB_VTOR_ADDR: usize = 0xE000_ED08;
pub const SCB_CPACR_ADDR: usize = 0xE000_ED88;

/// STM32/GD32 Advanced Control Timer (`TIM1`) register layout.
#[repr(C)]
pub struct Tim1Registers {
    pub cr1: u32,   // +0x00
    pub cr2: u32,   // +0x04
    pub smcr: u32,  // +0x08
    pub dier: u32,  // +0x0C
    pub sr: u32,    // +0x10
    pub egr: u32,   // +0x14
    pub ccmr1: u32, // +0x18
    pub ccmr2: u32, // +0x1C
    pub ccer: u32,  // +0x20
    pub cnt: u32,   // +0x24
    pub psc: u32,   // +0x28
    pub arr: u32,   // +0x2C
    pub rcr: u32,   // +0x30
    pub ccr1: u32,  // +0x34
    pub ccr2: u32,  // +0x38
    pub ccr3: u32,  // +0x3C
    pub ccr4: u32,  // +0x40
    pub bdtr: u32,  // +0x44
    pub dcr: u32,   // +0x48
    pub dmar: u32,  // +0x4C
}

/// STM32/GD32 GPIO register layout.
#[repr(C)]
pub struct GpioRegisters {
    pub crl: u32,  // +0x00: Pin 0..7 configuration
    pub crh: u32,  // +0x04: Pin 8..15 configuration
    pub idr: u32,  // +0x08: Input data register
    pub odr: u32,  // +0x0C: Output data register
    pub bsrr: u32, // +0x10: Bit set/reset register
    pub brr: u32,  // +0x14: Bit reset register
    pub lckr: u32, // +0x18: Port configuration lock
}

/// STM32/GD32 ADC register layout.
#[repr(C)]
pub struct AdcRegisters {
    pub sr: u32,    // +0x00
    pub cr1: u32,   // +0x04
    pub cr2: u32,   // +0x08
    pub smpr1: u32, // +0x0C
    pub smpr2: u32, // +0x10
    pub jofr1: u32, // +0x14
    pub jofr2: u32, // +0x18
    pub jofr3: u32, // +0x1C
    pub jofr4: u32, // +0x20
    pub htr: u32,   // +0x24
    pub ltr: u32,   // +0x28
    pub sqr1: u32,  // +0x2C
    pub sqr2: u32,  // +0x30
    pub sqr3: u32,  // +0x34
    pub jsqr: u32,  // +0x38
    pub jdr1: u32,  // +0x3C: Injected data register 1 (Phase A shunt)
    pub jdr2: u32,  // +0x40: Injected data register 2 (Phase B shunt)
    pub jdr3: u32,  // +0x44: Injected data register 3 (Phase C shunt)
    pub jdr4: u32,  // +0x48
    pub dr: u32,    // +0x4C: Regular data register
}

/// STM32/GD32 USART register layout.
#[repr(C)]
pub struct UsartRegisters {
    pub sr: u32,   // +0x00: Status register
    pub dr: u32,   // +0x04: Data register
    pub brr: u32,  // +0x08: Baud rate register
    pub cr1: u32,  // +0x0C: Control register 1
    pub cr2: u32,  // +0x10: Control register 2
    pub cr3: u32,  // +0x14: Control register 3
    pub gtpr: u32, // +0x18: Guard time and prescaler
}

/// Safe hardware pointer accessors.
pub struct HardwarePeripherals;

impl HardwarePeripherals {
    #[inline(always)]
    pub fn tim1() -> &'static mut Tim1Registers {
        unsafe { &mut *(TIM1_BASE as *mut Tim1Registers) }
    }

    #[inline(always)]
    pub fn gpioa() -> &'static mut GpioRegisters {
        unsafe { &mut *(GPIOA_BASE as *mut GpioRegisters) }
    }

    #[inline(always)]
    pub fn gpiob() -> &'static mut GpioRegisters {
        unsafe { &mut *(GPIOB_BASE as *mut GpioRegisters) }
    }

    #[inline(always)]
    pub fn gpioc() -> &'static mut GpioRegisters {
        unsafe { &mut *(GPIOC_BASE as *mut GpioRegisters) }
    }

    #[inline(always)]
    pub fn adc1() -> &'static mut AdcRegisters {
        unsafe { &mut *(ADC1_BASE as *mut AdcRegisters) }
    }

    #[inline(always)]
    pub fn adc2() -> &'static mut AdcRegisters {
        unsafe { &mut *(ADC2_BASE as *mut AdcRegisters) }
    }

    #[inline(always)]
    pub fn usart3() -> &'static mut UsartRegisters {
        unsafe { &mut *(USART3_BASE as *mut UsartRegisters) }
    }

    /// Sets the Cortex-M Vector Table Offset Register (`SCB->VTOR`).
    #[inline(always)]
    pub fn set_vtor(vtor_addr: u32) {
        unsafe {
            write_volatile(SCB_VTOR_ADDR as *mut u32, vtor_addr);
        }
    }

    /// Enables full access to Coprocessor 10 and 11 (Hardware FPU).
    #[inline(always)]
    pub fn enable_fpu() {
        unsafe {
            let cpacr = read_volatile(SCB_CPACR_ADDR as *const u32);
            write_volatile(SCB_CPACR_ADDR as *mut u32, cpacr | 0x00F0_0000);
        }
    }
}
