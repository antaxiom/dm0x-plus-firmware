#![no_std]
#![no_main]
#![allow(clippy::missing_safety_doc)]

use core::panic::PanicInfo;
use dm01_firmware::config::MotorConfig;
use dm01_firmware::control::Dm01Controller;
use dm01_firmware::foc::FocCurrentLoop;
use dm01_firmware::hal::{Dm01PwmHardware, Dm01SensorsHardware, MotorSensorsHal, ThreePhasePwm};
use dm01_firmware::pac::HardwarePeripherals;

#[repr(C)]
pub union Vector {
    pub handler: unsafe extern "C" fn(),
    pub reset: unsafe extern "C" fn() -> !,
    pub sp: usize,
}

/// 16-entry Cortex-M core vector table matching ToSeven DM01 hardware.
#[link_section = ".vector_table"]
#[no_mangle]
pub static VECTOR_TABLE: [Vector; 16] = [
    // Vector 0: Initial Stack Pointer (MSP = 0x20005628)
    Vector { sp: 0x2000_5628 },
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

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    Reset_Handler();
}

#[no_mangle]
pub unsafe extern "C" fn Reset_Handler() -> ! {
    // 1. Enable Hardware Single-Precision FPU (CP10 and CP11)
    HardwarePeripherals::enable_fpu();

    // 2. Relocate Vector Table to Flash Application Base (0x08001800)
    HardwarePeripherals::set_vtor(0x0800_1800);

    // 3. Initialize Physical Hardware Peripherals
    let mut pwm = Dm01PwmHardware::new();
    pwm.init_hardware();

    let mut sensors = Dm01SensorsHardware::new();

    // 4. Initialize Motor Controller State Machine with Factory TS1000W configuration
    let config = MotorConfig::default_dm01();
    let mut controller = Dm01Controller::new(config);
    let mut foc = FocCurrentLoop::new(Dm01PwmHardware::DEFAULT_ARR);

    // Main real-time motor control super-loop
    loop {
        // Read slow-loop sensor inputs (strain gauge, throttle, battery, thermistors)
        let inputs = sensors.sample_control_inputs();

        // Step high-level controller state machine
        let output = controller.step_v6510c(&inputs);

        if output.cutoff_active || output.target_current_ma == 0 {
            pwm.disable_inverter();
        } else {
            pwm.enable_inverter();
            // Read instantaneous motor currents & Hall rotor state
            let currents = sensors.sample_phase_currents();
            let _hall = sensors.read_hall_bits();

            // Run FOC current loop step
            let duties = foc.step_isr(currents, 0, 0, output.target_current_ma as i32);
            pwm.set_phase_duties(duties);
        }

        // Simple idle loop
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub unsafe extern "C" fn SysTick_Handler() {
    // 1 ms system timer tick
}

#[no_mangle]
pub unsafe extern "C" fn Nmi_Handler() {
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
