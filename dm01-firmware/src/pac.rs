//! # Minimal Peripheral Access Crate (PAC) for STM32/GD32 Cortex-M4F
//!
//! Provides direct, `#![no_std]` memory-mapped register definitions for the DM01
//! hardware peripherals (`TIM1`, `ADC1/2`, `USART3`, `GPIOA..GPIOD`, and `SCB`).

use core::cell::UnsafeCell;
use core::ptr::{read_volatile, write_volatile};

/// Transparent wrapper for a memory-mapped hardware register enforcing volatile reads and writes
/// without requiring exclusive (`&mut`) references that could alias in interrupts.
#[repr(transparent)]
pub struct VolatileReg<T: Copy>(UnsafeCell<T>);

// Safety: Hardware peripheral registers are accessed at fixed MMIO addresses via volatile ops.
unsafe impl<T: Copy> Sync for VolatileReg<T> {}

impl<T: Copy> VolatileReg<T> {
    #[inline(always)]
    pub const fn new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }

    /// Performs a volatile read from the hardware register.
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { read_volatile(self.0.get()) }
    }

    /// Performs a volatile write to the hardware register.
    #[inline(always)]
    pub fn write(&self, val: T) {
        unsafe {
            write_volatile(self.0.get(), val);
        }
    }

    /// Performs a volatile read-modify-write update on the hardware register.
    #[inline(always)]
    pub fn modify<F: FnOnce(T) -> T>(&self, f: F) {
        let cur = self.read();
        self.write(f(cur));
    }
}

pub const TIM1_BASE: usize = 0x4001_2C00;
pub const ADC1_BASE: usize = 0x4001_2400;
pub const ADC2_BASE: usize = 0x4001_2800;
pub const GPIOA_BASE: usize = 0x4001_0800;
pub const GPIOB_BASE: usize = 0x4001_0C00;
pub const GPIOC_BASE: usize = 0x4001_1000;
pub const GPIOD_BASE: usize = 0x4001_1400;
pub const AFIO_BASE: usize = 0x4001_0000;
pub const IWDG_BASE: usize = 0x4000_3000;
pub const USART3_BASE: usize = 0x4000_4800;
pub const RCC_BASE: usize = 0x4002_1000;
pub const DMA1_BASE: usize = 0x4002_0000;
pub const FLASH_BASE_REG: usize = 0x4002_2000;
pub const SCB_VTOR_ADDR: usize = 0xE000_ED08;
pub const SCB_AIRCR_ADDR: usize = 0xE000_ED0C;
pub const SCB_CPACR_ADDR: usize = 0xE000_ED88;
pub const SYST_CSR_ADDR: usize = 0xE000_E010;
pub const SYST_RVR_ADDR: usize = 0xE000_E014;
pub const SYST_CVR_ADDR: usize = 0xE000_E018;

/// STM32/GD32 Reset and Clock Control (`RCC`) register layout.
#[repr(C)]
pub struct RccRegisters {
    pub cr: VolatileReg<u32>,       // +0x00: Clock control
    pub cfgr: VolatileReg<u32>,     // +0x04: Clock configuration
    pub cir: VolatileReg<u32>,      // +0x08: Clock interrupt
    pub apb2rstr: VolatileReg<u32>, // +0x0C: APB2 peripheral reset
    pub apb1rstr: VolatileReg<u32>, // +0x10: APB1 peripheral reset
    pub ahbenr: VolatileReg<u32>,   // +0x14: AHB peripheral clock enable (DMA1, CRC, Flash)
    pub apb2enr: VolatileReg<u32>, // +0x18: APB2 peripheral clock enable (TIM1, ADC1/2, GPIOA..D, AFIO)
    pub apb1enr: VolatileReg<u32>, // +0x1C: APB1 peripheral clock enable (TIM4, USART3)
    pub bdcr: VolatileReg<u32>,    // +0x20: Backup domain control
    pub csr: VolatileReg<u32>,     // +0x24: Control/status register
}

/// Single DMA channel register set.
#[repr(C)]
pub struct DmaChannelRegisters {
    pub ccr: VolatileReg<u32>,   // +0x00: Configuration register
    pub cndtr: VolatileReg<u32>, // +0x04: Number of data transfers
    pub cpar: VolatileReg<u32>,  // +0x08: Peripheral address
    pub cmar: VolatileReg<u32>,  // +0x0C: Memory address
    pub _reserved: u32,          // +0x10: Padding to 20 bytes
}

/// STM32/GD32 DMA1 controller register layout (7 channels).
#[repr(C)]
pub struct Dma1Registers {
    pub isr: VolatileReg<u32>,    // +0x00: Interrupt status
    pub ifcr: VolatileReg<u32>,   // +0x04: Interrupt flag clear
    pub ch1: DmaChannelRegisters, // +0x08..+0x18 (ADC1 regular scan)
    pub ch2: DmaChannelRegisters, // +0x1C..+0x2C
    pub ch3: DmaChannelRegisters, // +0x30..+0x40
    pub ch4: DmaChannelRegisters, // +0x44..+0x54
    pub ch5: DmaChannelRegisters, // +0x58..+0x68
    pub ch6: DmaChannelRegisters, // +0x6C..+0x7C
    pub ch7: DmaChannelRegisters, // +0x80..+0x90
}

/// STM32/GD32 Advanced Control Timer (`TIM1`) register layout.
#[repr(C)]
pub struct Tim1Registers {
    pub cr1: VolatileReg<u32>,   // +0x00
    pub cr2: VolatileReg<u32>,   // +0x04
    pub smcr: VolatileReg<u32>,  // +0x08
    pub dier: VolatileReg<u32>,  // +0x0C
    pub sr: VolatileReg<u32>,    // +0x10
    pub egr: VolatileReg<u32>,   // +0x14
    pub ccmr1: VolatileReg<u32>, // +0x18
    pub ccmr2: VolatileReg<u32>, // +0x1C
    pub ccer: VolatileReg<u32>,  // +0x20
    pub cnt: VolatileReg<u32>,   // +0x24
    pub psc: VolatileReg<u32>,   // +0x28
    pub arr: VolatileReg<u32>,   // +0x2C
    pub rcr: VolatileReg<u32>,   // +0x30
    pub ccr1: VolatileReg<u32>,  // +0x34
    pub ccr2: VolatileReg<u32>,  // +0x38
    pub ccr3: VolatileReg<u32>,  // +0x3C
    pub ccr4: VolatileReg<u32>,  // +0x40
    pub bdtr: VolatileReg<u32>,  // +0x44
    pub dcr: VolatileReg<u32>,   // +0x48
    pub dmar: VolatileReg<u32>,  // +0x4C
}

/// STM32/GD32 GPIO register layout.
#[repr(C)]
pub struct GpioRegisters {
    pub crl: VolatileReg<u32>,  // +0x00: Pin 0..7 configuration
    pub crh: VolatileReg<u32>,  // +0x04: Pin 8..15 configuration
    pub idr: VolatileReg<u32>,  // +0x08: Input data register
    pub odr: VolatileReg<u32>,  // +0x0C: Output data register
    pub bsrr: VolatileReg<u32>, // +0x10: Bit set/reset register
    pub brr: VolatileReg<u32>,  // +0x14: Bit reset register
    pub lckr: VolatileReg<u32>, // +0x18: Port configuration lock
}

/// STM32/GD32 ADC register layout.
#[repr(C)]
pub struct AdcRegisters {
    pub sr: VolatileReg<u32>,    // +0x00
    pub cr1: VolatileReg<u32>,   // +0x04
    pub cr2: VolatileReg<u32>,   // +0x08
    pub smpr1: VolatileReg<u32>, // +0x0C
    pub smpr2: VolatileReg<u32>, // +0x10
    pub jofr1: VolatileReg<u32>, // +0x14
    pub jofr2: VolatileReg<u32>, // +0x18
    pub jofr3: VolatileReg<u32>, // +0x1C
    pub jofr4: VolatileReg<u32>, // +0x20
    pub htr: VolatileReg<u32>,   // +0x24
    pub ltr: VolatileReg<u32>,   // +0x28
    pub sqr1: VolatileReg<u32>,  // +0x2C
    pub sqr2: VolatileReg<u32>,  // +0x30
    pub sqr3: VolatileReg<u32>,  // +0x34
    pub jsqr: VolatileReg<u32>,  // +0x38
    pub jdr1: VolatileReg<u32>,  // +0x3C: Injected data register 1 (Phase A shunt)
    pub jdr2: VolatileReg<u32>,  // +0x40: Injected data register 2 (Phase B shunt)
    pub jdr3: VolatileReg<u32>,  // +0x44: Injected data register 3 (Phase C shunt)
    pub jdr4: VolatileReg<u32>,  // +0x48
    pub dr: VolatileReg<u32>,    // +0x4C: Regular data register
}

/// STM32/GD32 USART register layout.
#[repr(C)]
pub struct UsartRegisters {
    pub sr: VolatileReg<u32>,   // +0x00: Status register
    pub dr: VolatileReg<u32>,   // +0x04: Data register
    pub brr: VolatileReg<u32>,  // +0x08: Baud rate register
    pub cr1: VolatileReg<u32>,  // +0x0C: Control register 1
    pub cr2: VolatileReg<u32>,  // +0x10: Control register 2
    pub cr3: VolatileReg<u32>,  // +0x14: Control register 3
    pub gtpr: VolatileReg<u32>, // +0x18: Guard time and prescaler
}

/// STM32/GD32 Independent Watchdog (`IWDG`) register layout.
#[repr(C)]
pub struct IwdgRegisters {
    pub kr: VolatileReg<u32>,  // +0x00: Key register
    pub pr: VolatileReg<u32>,  // +0x04: Prescaler register
    pub rlr: VolatileReg<u32>, // +0x08: Reload register
    pub sr: VolatileReg<u32>,  // +0x0C: Status register
}

/// STM32/GD32 embedded Flash memory controller register layout.
#[repr(C)]
pub struct FlashRegisters {
    pub acr: VolatileReg<u32>,     // +0x00: Access control register
    pub keyr: VolatileReg<u32>,    // +0x04: Key register (unlock sequence: 0x45670123, 0xCDEF89AB)
    pub optkeyr: VolatileReg<u32>, // +0x08: Option byte key register
    pub sr: VolatileReg<u32>, // +0x0C: Status register (BSY=bit 0, PGERR=bit 2, WRPRTERR=bit 4, EOP=bit 5)
    pub cr: VolatileReg<u32>, // +0x10: Control register (PG=bit 0, PER=bit 1, MER=bit 2, STRT=bit 6, LOCK=bit 7)
    pub ar: VolatileReg<u32>, // +0x14: Address register (page erase address)
    pub _reserved: u32,       // +0x18
    pub obr: VolatileReg<u32>, // +0x1C: Option byte register
    pub wrpr: VolatileReg<u32>, // +0x20: Write protection register
}

/// Error conditions encountered during embedded Flash controller operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashError {
    /// Flash memory page is write-protected (`WRPRTERR`).
    WriteProtection,
    /// Flash programming or erase alignment fault (`PGERR`).
    ProgrammingError,
}

/// Hardware pointer accessors returning shared references to volatile register blocks.
pub struct HardwarePeripherals;

impl HardwarePeripherals {
    #[inline(always)]
    pub fn tim1() -> &'static Tim1Registers {
        unsafe { &*(TIM1_BASE as *const Tim1Registers) }
    }

    #[inline(always)]
    pub fn gpioa() -> &'static GpioRegisters {
        unsafe { &*(GPIOA_BASE as *const GpioRegisters) }
    }

    #[inline(always)]
    pub fn gpiob() -> &'static GpioRegisters {
        unsafe { &*(GPIOB_BASE as *const GpioRegisters) }
    }

    #[inline(always)]
    pub fn gpioc() -> &'static GpioRegisters {
        unsafe { &*(GPIOC_BASE as *const GpioRegisters) }
    }

    #[inline(always)]
    pub fn gpiod() -> &'static GpioRegisters {
        unsafe { &*(GPIOD_BASE as *const GpioRegisters) }
    }

    #[inline(always)]
    pub fn adc1() -> &'static AdcRegisters {
        unsafe { &*(ADC1_BASE as *const AdcRegisters) }
    }

    #[inline(always)]
    pub fn adc2() -> &'static AdcRegisters {
        unsafe { &*(ADC2_BASE as *const AdcRegisters) }
    }

    #[inline(always)]
    pub fn usart3() -> &'static UsartRegisters {
        unsafe { &*(USART3_BASE as *const UsartRegisters) }
    }

    #[inline(always)]
    pub fn rcc() -> &'static RccRegisters {
        unsafe { &*(RCC_BASE as *const RccRegisters) }
    }

    #[inline(always)]
    pub fn dma1() -> &'static Dma1Registers {
        unsafe { &*(DMA1_BASE as *const Dma1Registers) }
    }

    #[inline(always)]
    pub fn iwdg() -> &'static IwdgRegisters {
        unsafe { &*(IWDG_BASE as *const IwdgRegisters) }
    }

    /// Initializes and starts the Independent Watchdog (IWDG).
    /// Uses 40 kHz internal low-speed oscillator (LSI) with prescaler 64 (~625 Hz tick).
    /// `timeout_ms` (e.g. 500 ms) sets the watchdog trip window before resetting the MCU.
    pub fn init_iwdg(timeout_ms: u32) {
        let iwdg = Self::iwdg();
        // 1. Enable register write access
        iwdg.kr.write(0x5555);
        // 2. Set prescaler to 64 (PR = 0b100) -> 40 kHz / 64 = 625 Hz (~1.6 ms/count)
        iwdg.pr.write(0b100);
        // 3. Set reload register (e.g. 500 ms * 625 / 1000 ≈ 312 counts, clamped to 0xFFF)
        let reload = ((timeout_ms * 625) / 1000).clamp(1, 0xFFF);
        iwdg.rlr.write(reload);
        // 4. Reload counter
        iwdg.kr.write(0xAAAA);
        // 5. Start watchdog
        iwdg.kr.write(0xCCCC);
    }

    /// Feeds / refreshes the Independent Watchdog (IWDG) counter to prevent MCU reset.
    #[inline(always)]
    pub fn feed_iwdg() {
        let iwdg = Self::iwdg();
        iwdg.kr.write(0xAAAA);
    }

    /// Sets the Cortex-M Vector Table Offset Register (`SCB->VTOR`).
    #[inline(always)]
    pub fn set_vtor(vtor_addr: u32) {
        unsafe {
            write_volatile(SCB_VTOR_ADDR as *mut u32, vtor_addr);
        }
    }

    /// Triggers a software system reset via the Cortex-M Application Interrupt and Reset Control Register (AIRCR).
    #[inline(always)]
    pub fn system_reset() -> ! {
        unsafe {
            // VECTKEY (0x05FA) | SYSRESETREQ (bit 2)
            write_volatile(SCB_AIRCR_ADDR as *mut u32, 0x05FA_0004);
            loop {
                core::hint::spin_loop();
            }
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

    /// Configures the Cortex-M SysTick timer for periodic ticks (e.g. 1 ms at 72 MHz).
    #[inline(always)]
    pub fn init_systick(reload_ticks: u32) {
        unsafe {
            // Disable counter, set reload value, clear current value
            write_volatile(SYST_CSR_ADDR as *mut u32, 0);
            write_volatile(SYST_RVR_ADDR as *mut u32, reload_ticks.saturating_sub(1));
            write_volatile(SYST_CVR_ADDR as *mut u32, 0);
            // CLKSOURCE (bit 2) | TICKINT (bit 1) | ENABLE (bit 0) = 0x07
            write_volatile(SYST_CSR_ADDR as *mut u32, 0x0007);
        }
    }

    /// Polls whether SysTick has counted down to zero since last read (bit 16 COUNTFLAG).
    #[inline(always)]
    pub fn systick_has_fired() -> bool {
        unsafe {
            let csr = read_volatile(SYST_CSR_ADDR as *const u32);
            (csr & (1 << 16)) != 0
        }
    }

    /// Transmits a byte buffer over USART3 with TXE polling.
    #[inline(always)]
    pub fn usart3_transmit(bytes: &[u8]) {
        let usart = Self::usart3();
        for &b in bytes {
            while (usart.sr.read() & (1 << 7)) == 0 {
                core::hint::spin_loop();
            }
            usart.dr.write(b as u32);
        }
    }

    #[inline(always)]
    pub fn flash() -> &'static FlashRegisters {
        unsafe { &*(FLASH_BASE_REG as *const FlashRegisters) }
    }

    /// Flash unlock key 1 matching STM32/GD32 hardware (`0x45670123`).
    pub const FLASH_KEY1: u32 = 0x4567_0123;
    /// Flash unlock key 2 matching STM32/GD32 hardware (`0xCDEF89AB`).
    pub const FLASH_KEY2: u32 = 0xCDEF_89AB;

    /// Unlocks the Flash memory controller if locked.
    pub fn flash_unlock() {
        let flash = Self::flash();
        if (flash.cr.read() & (1 << 7)) != 0 {
            flash.keyr.write(Self::FLASH_KEY1);
            flash.keyr.write(Self::FLASH_KEY2);
        }
    }

    /// Locks the Flash memory controller.
    pub fn flash_lock() {
        let flash = Self::flash();
        flash.cr.modify(|v| v | (1 << 7));
    }

    /// Erases a Flash page (e.g. Page 127 at `0x0801_FC00`).
    pub fn flash_erase_page(page_addr: u32) -> Result<(), FlashError> {
        let flash = Self::flash();
        Self::flash_unlock();

        // Wait while BSY
        while (flash.sr.read() & 1) != 0 {
            core::hint::spin_loop();
        }

        // Clear any previous error flags (EOP=bit 5, WRPRTERR=bit 4, PGERR=bit 2)
        flash.sr.write((1 << 5) | (1 << 4) | (1 << 2));

        // Set PER bit (bit 1) and specify page address in AR
        flash.cr.write(1 << 1);
        flash.ar.write(page_addr);
        // Set STRT bit (bit 6) to begin erase
        flash.cr.modify(|v| v | (1 << 6));

        // Wait while BSY
        while (flash.sr.read() & 1) != 0 {
            core::hint::spin_loop();
        }

        let sr = flash.sr.read();
        flash.cr.write(0); // Clear PER
        Self::flash_lock();

        if (sr & (1 << 4)) != 0 {
            Err(FlashError::WriteProtection)
        } else if (sr & (1 << 2)) != 0 {
            Err(FlashError::ProgrammingError)
        } else {
            Ok(())
        }
    }

    /// Programs a buffer of 16-bit halfwords to Flash at `dest_addr`.
    pub fn flash_write_halfwords(dest_addr: u32, data: &[u16]) -> Result<(), FlashError> {
        let flash = Self::flash();
        Self::flash_unlock();

        for (idx, &hw) in data.iter().enumerate() {
            // Wait while BSY
            while (flash.sr.read() & 1) != 0 {
                core::hint::spin_loop();
            }

            // Set PG bit (bit 0)
            flash.cr.write(1 << 0);

            let target = (dest_addr as usize + idx * 2) as *mut u16;
            unsafe {
                write_volatile(target, hw);
            }

            // Wait while BSY
            while (flash.sr.read() & 1) != 0 {
                core::hint::spin_loop();
            }

            let sr = flash.sr.read();
            if (sr & ((1 << 4) | (1 << 2))) != 0 {
                flash.cr.write(0);
                Self::flash_lock();
                return if (sr & (1 << 4)) != 0 {
                    Err(FlashError::WriteProtection)
                } else {
                    Err(FlashError::ProgrammingError)
                };
            }
        }

        flash.cr.write(0);
        Self::flash_lock();
        Ok(())
    }

    /// Polls whether a byte is waiting in the USART3 receive data register (RXNE, bit 5).
    #[inline(always)]
    pub fn usart3_has_rx() -> bool {
        let usart = Self::usart3();
        (usart.sr.read() & (1 << 5)) != 0
    }

    /// Reads one byte from USART3 (and clears the RXNE flag).
    #[inline(always)]
    pub fn usart3_read_byte() -> u8 {
        let usart = Self::usart3();
        (usart.dr.read() & 0xFF) as u8
    }
}
