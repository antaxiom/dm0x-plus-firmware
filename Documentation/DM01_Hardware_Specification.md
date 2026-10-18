# ToSeven DM01 Controller Hardware & Register Specification

This document provides the definitive, reverse-engineered hardware architecture and pinout specification for the **ToSeven DM01 (12-FET, 750W/1000W)** motor controller, extracted directly from the decompiled `V6510C` firmware binary (`TO7-DM01-K260521A-V6510C.Pack`).

---

## 1. Microcontroller Architecture & Memory Map

- **Core:** ARM Cortex-M4F with hardware single-precision Floating-Point Unit (FPU).
- **Silicon Family:** GigaDevice GD32F303 / STMicroelectronics STM32F303 / STM32F103 compatible.
- **Clock Configuration (`SystemInit` at `0x08007D0C`):**
  - System Core Clock: $72\text{ MHz}$ (or $108\text{ MHz}$ on GD32).
  - Coprocessor CP10/CP11 enabled via `SCB->CPACR` (`0xE000ED88 |= 0x00F00000`).
  - Vector Table Offset: `SCB->VTOR` (`0xE000ED08`) set to `0x08001800`.
- **Memory Boundaries:**
  - **Flash Bootloader:** `0x08000000..0x080017FF` ($6\text{ KB}$ factory IAP bootloader).
  - **Application Flash:** `0x08001800..0x0800F96C` ($57,708\text{ B}$ application firmware payload).
  - **Flash EEPROM:** `0x0801FC00..0x0801FFFF` ($1\text{ KB}$ page containing 5 blocks totaling $480\text{ B}$).
  - **Internal SRAM:** `0x20000000..0x20007FFF` ($32\text{ KB}$ / $48\text{ KB}$ SRAM).
  - **Initial Main Stack Pointer (MSP):** `0x20005628`.

---

## 2. Inverter Gate Driver & PWM Timer (`TIM1` at `0x40012C00`)

The 3-phase inverter uses 12 MOSFETs (2 in parallel per switch) driven by 6 complementary PWM channels with hardware dead-time generation:

| Inverter Switch | Phase | MCU Pin | Timer Channel | Register | Config / Mode |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **High-Side U** | Phase U | **`PA8`** | `TIM1_CH1` | `TIM1->CCR1` | Alternate Function Push-Pull (`50 MHz`) |
| **Low-Side U** | Phase U | **`PB13`** | `TIM1_CH1N` | `TIM1->CCR1` | Alternate Function Push-Pull (`50 MHz`) |
| **High-Side V** | Phase V | **`PA9`** | `TIM1_CH2` | `TIM1->CCR2` | Alternate Function Push-Pull (`50 MHz`) |
| **Low-Side V** | Phase V | **`PB14`** | `TIM1_CH2N` | `TIM1->CCR2` | Alternate Function Push-Pull (`50 MHz`) |
| **High-Side W** | Phase W | **`PA10`** | `TIM1_CH3` | `TIM1->CCR3` | Alternate Function Push-Pull (`50 MHz`) |
| **Low-Side W** | Phase W | **`PB15`** | `TIM1_CH3N` | `TIM1->CCR3` | Alternate Function Push-Pull (`50 MHz`) |
| **Emergency Break** | Fault | **`PB12`** | `TIM1_BKIN` | `TIM1->BDTR.BKE` | Hardware High-Z fault trip |

### Timer Operating Parameters:
- **PWM Frequency:** $16.0\text{ kHz}$ center-aligned mode (Counter Mode: `TIM_CounterMode_CenterAligned1`).
- **Dead-Time Generator (`TIM1->BDTR.DTG`):** `0x64` ($100$ clock ticks $\approx 1.388\text{ }\mu\text{s}$ at $72\text{ MHz}$).
- **Auto-Reload Register (`TIM1->ARR`):** `0x176F` ($5999$ counts).
- **ISR Rate:** `TIM1_UP_TIM10_IRQHandler` (Vector 41) triggered at $16\text{ kHz}$ to execute the Space Vector PWM (SVPWM) calculation and Clarke/Park transforms.

---

## 3. Analog-to-Digital Converters (`ADC1` / `ADC2` at `0x40012400` / `0x40012800`)

Dual synchronized ADCs operate in injected + regular scan mode:

### A. Injected Channels (Phase Shunt Currents — Synchronized to PWM Midpoint):
- **Trigger:** Synchronized to `TIM1` center-aligned update event (when all low-side MOSFETs are ON).
- **ADC Channels:** Dual Injected reads on `PA0` (Phase A Shunt) and `PA1` (Phase B Shunt), with Phase C derived via Kirchhoff's Current Law: $I_c = -(I_a + I_b)$.

### B. Regular Channels (Analog Inputs):
| ADC Channel | MCU Pin | Signal Source | Description | Voltage Range |
| :--- | :--- | :--- | :--- | :--- |
| **Channel 1** | `PA1` | Current Shunt / Bus | Total battery current monitor | $0..3.3\text{ V}$ |
| **Channel 4** | `PA4` | **Torque Strain Gauge** | Analog pedal force signal | $400..3000\text{ mV}$ (resting zero $\sim 750\text{ mV}$) |
| **Channel 7** | `PA7` | **Throttle Signal** | Hall-effect thumb / twist throttle | $800..3500\text{ mV}$ (deadband cutoff below $1000\text{ mV}$) |
| **Channel 8** | `PB0` | **Battery Voltage** | Resistor voltage divider | Scaled for $36\text{ V}..52\text{ V}$ packs (up to $60.0\text{ V}$) |
| **Channel 9** | `PB1` | **Controller / Motor Temp** | NTC Thermistor divider | $10\text{ k}\Omega$ NTC, rollback starts at $70^\circ\text{C}$ |

---

## 4. Display Communications (`USART3` at `0x40004800`)

The handlebar display communicates over a half-duplex / full-duplex UART bus using ToSeven's proprietary XOR cipher:

- **Peripheral:** `USART3`
- **Pins:**
  - **TX:** **`PB10`** (Alternate Function Push-Pull)
  - **RX:** **`PB11`** (Input with internal pull-up)
- **Baud Rate:** **`9600 Baud`** (`mov.w r0, #0x2580` = 9600)
- **Frame Format:** 8 Data Bits, No Parity, 1 Stop Bit (8-N-1)
- **Wire Encryption:** Byte-wise stream cipher XOR `0x69`
- **Packet Size:** 32-byte telemetry frame protected by 16-entry nibble CRC-16

---

## 5. Hall Sensors & Cadence Observer (`TIM4` at `0x40000800` & `GPIOC`)

- **Hall Sensor Inputs:** 3 digital inputs with $10\text{ k}\Omega$ pull-ups:
  - `PC6` (Hall U), `PC7` (Hall V), `PC8` (Hall W)
- **Angle Tracking:** Handled by `TIM4_IRQHandler` (Vector 46) and `HallPllObserver` in `src/foc.rs`.
- **Interpolation:** Tracks the elapsed time between sector state transitions (`0b001..=0b110`) and advances the estimated rotor electrical angle ($\theta_e$) in $60^\circ$ increments using a Phase-Locked Loop (PLL).

---

## 6. Non-Volatile Flash Configuration Storage (`0x0801FC00`)

The DM01 saves all user configurations, calibration zero voltages, and ride statistics to the final $1\text{ KB}$ Flash page (`0x0801FC00`):
- **Total Payload:** 5 structured blocks ($32 + 64 + 64 + 160 + 160 = 480$ bytes).
- **Encryption:** Byte-wise XOR `0xA5`.
- **Validation Header:** `A5 01 FE` at `0x0801FC5A`.
- **Block Integrity:** 16-bit CRC-16 stored little-endian at the end of each block.
- **Factory Model String:** `"TS1000W"` at `0x0800EE70`.

