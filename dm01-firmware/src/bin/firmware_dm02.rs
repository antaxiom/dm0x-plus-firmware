#![no_std]
#![no_main]
#![allow(clippy::missing_safety_doc)]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};
use dm01_firmware::config::{AssistSelection, MotorConfig};
use dm01_firmware::control::Dm01Controller;
use dm01_firmware::foc::{FocCurrentLoop, HallPllObserver};
use dm01_firmware::hal::{Dm01PwmHardware, Dm01SensorsHardware, MotorSensorsHal, ThreePhasePwm};
use dm01_firmware::pac::HardwarePeripherals;
use dm01_firmware::protocol::{DisplayCommandDecoder, TelemetryFrame, UART_STREAM_XOR_KEY};

extern "C" {
    static _sidata: u32;
    static mut _sdata: u32;
    static mut _edata: u32;
    static mut _sbss: u32;
    static mut _ebss: u32;
}

#[repr(C)]
pub union Vector {
    pub handler: unsafe extern "C" fn(),
    pub reset: unsafe extern "C" fn() -> !,
    pub sp: usize,
}

/// 16-entry Cortex-M core vector table matching ToSeven DM02 hardware (`TS500W`).
#[link_section = ".vector_table"]
#[no_mangle]
pub static VECTOR_TABLE: [Vector; 16] = [
    // Vector 0: Initial Stack Pointer (MSP = 0x20005620 on DM02)
    Vector { sp: 0x2000_5620 },
    Vector {
        reset: Reset_Handler,
    },
    Vector {
        handler: Nmi_Handler,
    },
    Vector {
        handler: HardFault_Handler,
    },
    Vector {
        handler: Default_Handler,
    }, // MemManage
    Vector {
        handler: Default_Handler,
    }, // BusFault
    Vector {
        handler: Default_Handler,
    }, // UsageFault
    Vector {
        handler: Default_Handler,
    }, // Reserved
    Vector {
        handler: Default_Handler,
    }, // Reserved
    Vector {
        handler: Default_Handler,
    }, // Reserved
    Vector {
        handler: Default_Handler,
    }, // Reserved
    Vector {
        handler: Default_Handler,
    }, // SVCall
    Vector {
        handler: Default_Handler,
    }, // DebugMon
    Vector {
        handler: Default_Handler,
    }, // Reserved
    Vector {
        handler: Default_Handler,
    }, // PendSV
    Vector {
        handler: SysTick_Handler,
    }, // SysTick
];

/// Copies `.data` from Flash to SRAM and zeroes `.bss` before running any Rust code.
#[inline(always)]
unsafe fn init_ram() {
    let mut src = &_sidata as *const u32;
    let mut dst = &raw mut _sdata;
    let end_data = &raw mut _edata;
    while dst < end_data {
        write_volatile(dst, read_volatile(src));
        dst = dst.add(1);
        src = src.add(1);
    }

    let mut bss = &raw mut _sbss;
    let end_bss = &raw mut _ebss;
    while bss < end_bss {
        write_volatile(bss, 0);
        bss = bss.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    Reset_Handler();
}

#[no_mangle]
pub unsafe extern "C" fn Reset_Handler() -> ! {
    // 0. Initialize SRAM (.data and .bss)
    init_ram();

    // 1. Enable Hardware Single-Precision FPU (CP10 and CP11)
    HardwarePeripherals::enable_fpu();

    // 2. Relocate Vector Table to Flash Application Base (0x08001800)
    HardwarePeripherals::set_vtor(0x0800_1800);

    // 3. Initialize Physical Hardware Peripherals (starts in High-Z with MOE=0)
    let mut pwm = Dm01PwmHardware::new();
    pwm.init_hardware();

    let mut sensors = Dm01SensorsHardware::new();
    sensors.init_hardware();
    // 3a. Perform 256-sample startup shunt auto-zero calibration (matching V6510C at 0x08009A6C)
    sensors.calibrate_shunts();

    // 4. Configure SysTick Timer for 1 ms deterministic tick at 72 MHz (72,000 clock cycles)
    HardwarePeripherals::init_systick(72_000);

    // 5. Initialize Hardware Watchdog (IWDG) with 500 ms safety trip window
    HardwarePeripherals::init_iwdg(500);

    // 6. Initialize DM02 Motor Controller State Machine (load from Flash Page 127 if available)
    let mut config = MotorConfig::default_dm02();
    if let Some(flash_img) = unsafe { dm01_firmware::eeprom::EepromImage::read_from_flash() } {
        config.apply_eeprom_image(&flash_img);
    }
    let angle_offset_deg = config.motor_hw.angle_offset_deg;
    let foc_cfg = config.foc_config;
    let mut controller = Dm01Controller::with_tuning(
        config,
        dm01_firmware::tuning::TuningProfile::gear_safe_trail(),
    );
    let mut observer = HallPllObserver::new();
    let mut foc = FocCurrentLoop::new(Dm01PwmHardware::DEFAULT_ARR);
    foc.configure_from_foc_menu(&foc_cfg);

    let adc1 = HardwarePeripherals::adc1();
    let mut current_target_ma: i32 = 0;
    let mut cutoff_active = false;
    let mut first_tick = true;
    let mut telemetry_tick: u16 = 0;
    let mut cmd_decoder = DisplayCommandDecoder::new();
    let mut display_walk_held = false;

    // Main real-time motor control loop:
    // - Slow loop (1 ms / 1 kHz): analog strain gauge, throttle, battery voltage, thermistors, brake
    // - Fast loop (16 kHz): synchronized to TIM1 TRGO injected shunt current conversions (ADC1 JEOC)
    loop {
        // 1. Slow 1 ms loop: sample control inputs and step high-level controller state machine
        if first_tick || HardwarePeripherals::systick_has_fired() {
            first_tick = false;

            // A. Feed Hardware Watchdog to prevent MCU reset
            HardwarePeripherals::feed_iwdg();

            // B. Poll USART3 RX non-blockingly for incoming display command frames
            while HardwarePeripherals::usart3_has_rx() {
                let wire_b = HardwarePeripherals::usart3_read_byte();
                let plain_b = wire_b ^ UART_STREAM_XOR_KEY;
                if let Some(cmd) = cmd_decoder.push_byte(plain_b) {
                    controller.mode = cmd.mode;
                    controller.assist = if cmd.assist_level == 0 {
                        AssistSelection::Off
                    } else {
                        AssistSelection::Level(cmd.assist_level)
                    };
                    display_walk_held = cmd.walk_assist_active;
                }
            }

            let mut inputs = sensors.sample_control_inputs();
            if display_walk_held {
                inputs.walk_assist_held = true;
            }

            let output = controller.step_v6510c(&inputs);
            current_target_ma = output.target_current_ma as i32;
            cutoff_active = output.cutoff_active;

            if cutoff_active || current_target_ma == 0 {
                pwm.disable_inverter();
                foc.reset();
            }

            // Periodically transmit stock 18-byte telemetry frame to display (20 Hz / 50 ms)
            telemetry_tick += 1;
            if telemetry_tick >= 50 {
                telemetry_tick = 0;
                let eff_speed = controller.effective_wheel_speed_deci_kph(&inputs);
                let frame = TelemetryFrame {
                    battery_voltage_deci_v: (inputs.battery_voltage_mv / 100) as u16,
                    battery_soc_percent: 100,
                    phase_current_deci_a: (current_target_ma.max(0) as u32 / 100) as u16,
                    wheel_speed_deci_kph: eff_speed,
                    cadence_rpm: inputs.cadence_rpm,
                    torque_sensor_mv: inputs.torque_adc_mv,
                    throttle_mv: inputs.throttle_adc_mv,
                    controller_temp_c: inputs.controller_temp_c as i8,
                    faults: dm01_firmware::faults::FaultFlags::NONE,
                };
                let mut tx_buf = [0u8; 18];
                frame.encode_encrypted_wire(&mut tx_buf);
                HardwarePeripherals::usart3_transmit(&tx_buf);
            }
        }

        // 2. Fast FOC loop: check if injected conversion completed (JEOC in ADC1->SR, bit 2)
        let sr = adc1.sr.read();
        if (sr & (1 << 2)) != 0 {
            // Clear JEOC flag by clearing bit 2
            adc1.sr.modify(|v| v & !(1 << 2));

            if !cutoff_active && current_target_ma > 0 {
                // Read instantaneous phase currents (low-side shunt midpoint) & Hall rotor state
                let currents = sensors.sample_phase_currents();
                let hall = sensors.read_hall_bits();

                if let Some(electrical_angle) = observer.step_isr(hall, angle_offset_deg) {
                    let id_target_ma = foc.compute_flux_weakening_id_ma(&foc_cfg);
                    let duties =
                        foc.step_isr(currents, electrical_angle, id_target_ma, current_target_ma);
                    pwm.set_phase_duties(duties);
                    pwm.enable_inverter();
                } else {
                    // Invalid Hall state (0b000 or 0b111): immediately disable inverter
                    pwm.disable_inverter();
                    foc.reset();
                }
            } else {
                pwm.disable_inverter();
                foc.reset();
            }
        }

        // Low-power spin wait between ADC conversion pulses
        core::hint::spin_loop();
    }
}

pub static mut SYSTEM_TICKS_MS: u32 = 0;

#[no_mangle]
pub unsafe extern "C" fn SysTick_Handler() {
    SYSTEM_TICKS_MS = SYSTEM_TICKS_MS.wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Nmi_Handler() {
    let mut pwm = Dm01PwmHardware::new();
    pwm.disable_inverter();
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub unsafe extern "C" fn HardFault_Handler() {
    // Hardware fault handler: immediately disable PWM inverter (forces high-Z coasting)
    let mut pwm = Dm01PwmHardware::new();
    pwm.disable_inverter();
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub unsafe extern "C" fn Default_Handler() {
    let mut pwm = Dm01PwmHardware::new();
    pwm.disable_inverter();
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Safety: on any panic, immediately shut down all 6 inverter gate drivers
    let mut pwm = Dm01PwmHardware::new();
    pwm.disable_inverter();
    loop {
        core::hint::spin_loop();
    }
}
