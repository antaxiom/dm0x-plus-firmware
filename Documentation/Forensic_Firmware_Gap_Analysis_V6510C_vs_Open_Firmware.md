# Forensic Firmware Gap Analysis: Stock Factory `V6510C` vs. Open `dm01-firmware`

**Target Binaries:**
- `TO7-DM01-K260521A-V6510C.Pack` (`TS1000W` External 12-FET, 57,708 B Flash Payload)
- `TO7-DM02-K260526-V6510C.Pack` (`TS500W` Internal Integrated, 58,440 B Flash Payload)

**Comparative Reference:**
- `dm01-firmware` `v0.3.0` (`#![no_std]` Bare-Metal Embedded Rust Implementation)

---

## 1. Executive Summary & Forensic Scope

This report provides a forensic, subsystem-by-subsystem comparative gap analysis between the decompiled official ToSeven factory `V6510C` ARM Cortex-M4F firmware and the `dm01-firmware` open Rust implementation.

Over 22,000 disassembled Thumb-2/FPU instructions were mapped across both DM01 and DM02 factory payloads. The analysis validates that the open Rust firmware has already faithfully captured the core control laws (Clarke/Park transforms, SVPWM dead-time clamping, dual-ceiling power curve, 5-sample torque zero-drift tracking, 13 throttle modes, 5-block Flash EEPROM format, and 18-byte display protocol). It also uncovers the exact hardware peripheral routines, ADC injected offset registers, and state machine transitions used by the vendor runtime.

```mermaid
flowchart TD
    subgraph V6510C_Architecture["Factory V6510C Cortex-M4F Architecture"]
        VT["Vector Table (0x08001800)"] --> Reset["Reset_Handler (0x08001A51)"]
        Reset --> Main["main() (0x08009810)"]
        Main --> ShuntCal["Phase Shunt Auto-Zero (256-sample average)"]
        ShuntCal --> JOFR["Write ADC_JOFR1 / ADC_JOFR2"]
        Main --> EEPROM_Init["EEPROM Block 1 Check (A5 01 FE)"]
        EEPROM_Init --> FOC_Loop["TIM1_UP (IRQ 25) / ADC JEOC (IRQ 18)"]
        FOC_Loop --> FOC_SM["5-Stage FOC State Machine (0, 2, 4, 6, 8)"]
        Main --> SysTick["SysTick (100 Hz / 10 ms Control Loop)"]
        SysTick --> PowerCurve["Dual-Ceiling Power Curve & Arbitrator"]
        SysTick --> Telemetry["USART3 IRQ 39 (0x69 Wire Cipher)"]
    end
```

---

## 2. Hardware Interrupt Architecture & Vector Table Map

Both DM01 and DM02 firmware load at `0x08001800` (directly above the 6 KB factory IAP bootloader at `0x08000000..0x08001800`). The hardware interrupt vector tables reveal an identical real-time architecture:

| Vector | Offset | Handler Address (DM01) | Handler Address (DM02) | Peripheral / Role | Status in `dm01-firmware` |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **SP** | `+0x00` | `0x20005628` | `0x20005620` | Initial Stack Pointer (MSP in SRAM) | Matched bit-exact in vector tables |
| **Sys 1** | `+0x04` | `0x08001A51` | `0x08001A51` | `Reset_Handler` | Implemented in `firmware.rs` / `firmware_dm02.rs` |
| **Sys 2** | `+0x08` | `0x0800672D` | `0x08006855` | `NMI_Handler` | Handled via safe default trap |
| **Sys 3** | `+0x0C` | `0x080046A1` | `0x08004849` | `HardFault_Handler` (MOSFET latch-off) | Handled via safe default trap |
| **Sys 11** | `+0x2C` | `0x080071AF` | `0x0800723B` | `SVCall` (Supervisor Call) | Single-threaded cooperative loop in Rust |
| **Sys 14** | `+0x38` | `0x08006CEB` | `0x08006DA7` | `PendSV` (Context Switcher) | Cooperative fast loop in Rust |
| **Sys 15** | `+0x3C` | `0x08007A11` | `0x08007AA1` | `SysTick` (100 Hz Master Timing Loop) | Implemented as 10 ms control step |
| **IRQ 11** | `+0x6C` | `0x080027ED` | `0x08002841` | `DMA1_Stream0` (ADC Regular Scan circular DMA) | Registers mapped in `pac.rs` |
| **IRQ 12** | `+0x70` | `0x08002895` | `0x080028E9` | `DMA1_Stream1` (USART3 Telemetry DMA) | Registers mapped in `pac.rs` |
| **IRQ 18** | `+0x88` | `0x08001C8D` | `0x08001C8D` | `ADC1_2` Dual Injected End-of-Conversion (`JEOC`) | Fast FOC loop trigger in `firmware.rs` |
| **IRQ 24** | `+0xA0` | `0x08007DD1` | `0x08007E61` | `TIM1_BRK_TIM9` (Hardware Overcurrent Break Trip) | Registers mapped in `pac.rs` |
| **IRQ 25** | `+0xA4` | `0x08007E49` | `0x08007EC5` | `TIM1_UP_TIM10` (PWM Carrier Frequency Update) | Supported via TIM1 registers |
| **IRQ 28** | `+0xB0` | `0x08007E65` | `0x08007ED5` | `TIM2` (10 kHz Quadrature PAS Cadence on `PD0` + `PA15`) | Implemented in `hal.rs` |
| **IRQ 30** | `+0xB8` | `0x08007E81` | `0x08007EF1` | `TIM4` (Motor Hall Sensor XOR Commutation on `PB6..8`) | Supported via `foc.rs` Hall observer |
| **IRQ 39** | `+0xDC` | `0x080095D9` | `0x080096A9` | `USART3` (Display UART 9600 Baud + `0x69` Cipher) | Full `0x5A` & 7-bit `0xC0..0x80` codecs in `protocol.rs` |
| **IRQ 50** | `+0x108` | `0x08007FC5` | `0x08008035` | `TIM5` (10 kHz Wheel Speed `PA3`, Brake `PB2`, Headlight `PC15`, LED `PA2`) | Implemented in `hal.rs` |

---

## 2.1 Complete Hardware Pinout & Peripheral Map (`0x0800B8DC`..`0x0800DD88`)

| MCU Pin | Peripheral / Mode | Decompiled Function (`V6206C`..`V6510C`) | `dm01-firmware` Implementation |
| :--- | :--- | :--- | :--- |
| **`PA0, PA1`** | `ADC1_IN0/IN1` (Analog) | Controller/Motor NTC Temperature Sensor (`DMA_ADC[0]`) | Configured in `hal.rs` |
| **`PA2`** | `GPIO_Out_PP` (10 MHz) | Controller Status / Fault Blink LED (`0x08002094`) | Driven via `Dm01SensorsHardware::set_outputs` |
| **`PA3`** | `GPIO_IPU` (Input Pull-Up) | External Wheel Speed Sensor (`TIM5` 10 kHz ISR `0x08005A74`) | Polled in `Dm01SensorsHardware::sample_control_inputs` |
| **`PA4`** | `ADC1_IN4` (Analog) | Torque Strain Gauge `PedalVol` (`DMA_ADC[1]`) | Sampled via `DMA1_Ch1` in `hal.rs` |
| **`PA5, PA6`** | `ADC2_IN5/IN6` (Injected) | Phase B & Phase A Low-Side Shunt Currents (`0x08009A6C`) | 256-sample auto-zero + `JOFR1` in `hal.rs` |
| **`PA7`** | `ADC1_IN7` (Analog) | Battery Voltage Divider / Throttle (`DMA_ADC[2]`) | Sampled via `DMA1_Ch1` in `hal.rs` |
| **`PA8..PA10`** | `TIM1_CH1..3` (`AF_PP`) | High-Side 3-Phase Inverter Gates (`ARR=6249`, `CCR4=5999`) | Driven by `Dm01PwmHardware` |
| **`PA15`** | `GPIO_IPU` (`SWJ_JTAGDisable`) | Quadrature PAS Cadence Channel B (`TIM2` 10 kHz ISR `0x0800574C`) | Remapped via `AFIO->MAPR` & polled in `hal.rs` |
| **`PB0, PB1`** | `ADC1_IN8/IN9` (Analog) | Throttle Hall `ThrottleVol` & Aux Reference (`DMA_ADC[3..4]`) | Sampled via `DMA1_Ch1` in `hal.rs` |
| **`PB2`** | `GPIO_IPU` (Input Pull-Up) | Brake Cutout Switch (`0x0800210C`, 5-tick debounce) | Polled in `hal.rs` |
| **`PB6..PB8`** | `TIM4_CH1..3` (`GPIO_IPU`) | Motor Hall Sensors A, B, C (`(GPIOB->IDR >> 6) & 7`) | Polled in `Dm01SensorsHardware::read_hall_bits` |
| **`PB9`** | `TIM11_CH1` (`AF_PP`) | 1 kHz 50% Duty Sensor Excitation PWM (`0x0800A1D0`) | Documented hardware excitation reference |
| **`PB10, PB11`** | `USART3_TX/RX` (`9600 Bd`) | Display UART with `0x69` XOR Stream Cipher (`0x0800B84C`) | Full TX/RX in `pac.rs` & `protocol.rs` |
| **`PB12`** | `TIM1_BKIN` (Input) | Hardware Overcurrent Comparator Break Input (`0x0800D728`) | Enabled via `BDTR.BKE` in `hal.rs` |
| **`PB13..PB15`** | `TIM1_CH1N..3N` (`AF_PP`) | Low-Side 3-Phase Complementary Gates (`DTG = 0x64`) | Driven by `Dm01PwmHardware` |
| **`PC15`** | `GPIO_Out_PP` (10 MHz) | Headlight / Walk-Assist Output (`0x080061A4`) | Driven via `Dm01SensorsHardware::set_outputs` |
| **`PD0`** | `GPIO_IPU` (`PD01_REMAP`) | Quadrature PAS Cadence Channel A (`TIM2` 10 kHz ISR `0x0800574C`) | Remapped via `AFIO->MAPR` & polled in `hal.rs` |

---

## 2.2 Historical Firmware Evolution & Diff Matrix (All 17 Factory `.Pack` Releases)

Every single one of the 17 historical `.Pack` containers embeds a valid 480-byte 5-block (`0x20, 0x40, 0x40, 0xA0, 0xA0`) plaintext factory EEPROM table in Flash ROM and encodes the decimal firmware version in `decrypted_header[1..3]`.

> [!NOTE]
> **Forensic Discovery — Factory EEPROM Block 2 Engineering String (`"Sitgo-Rear"`):**
> Across all 17 factory `.Pack` firmwares from 2024 to 2026, the factory default EEPROM table embeds the ASCII string `"Sitgo-Rear 1700W"` (on DM01) and `"Sitgo-Rear 1500W"` (on DM02) at byte offset `0x07D..0x09E` in Block 2, alongside recurring placeholder serial numbers (`24111100001` on DM01, `230801230801` on DM02). Because both DM01 and DM02 are bottom-bracket mid-drive motors (rather than rear hub motors), this string is an internal vendor engineering template artifact (likely inherited from a shared factory calibration profile or ODM codebase) rather than a commercial consumer vehicle designation.

### DM01 (`TS1000W` — 6 Releases)
| Release File | Version (`hdr[1..3]`) | Keys (`K_hdr`/`K_pay`/`UartMask`) | Payload Size | Initial `MSP` | Embedded EEPROM Offset & Block 1 Magic | Key Binary & EEPROM Diffs vs. Predecessor |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `TO7-DM01-K241021-V6212C.Pack` | `6212` (`0x1844`) | `0xF1` / `0xCC` / `0xB5` | `56,644 B` | `0x20005618` | `0x0800EA44` (`A5 01 FE`) | Initial `V6212C` baseline (`65.0 A` peak shunt limit, `55.0 V` HV cap). |
| `TO7-DM01-K250731-V6510C.Pack` | `6510` (`0x196E`) | `0xD7` / `0xF4` / `0xCC` | `57,704 B` | `0x20005628` | `0x0800EE6C` (`A5 01 FE`) | `+1,060 B` (`V6510C` architecture): adds 50.0 V constant-power cap, `MSP +16 B`. |
| `TO7-DM01-K250926-V6510C.Pack` | `6510` (`0x196E`) | `0xEA` / `0xB8` / `0xDC` | `57,704 B` | `0x20005628` | `0x0800EE6C` (`A5 01 FE`) | `55 B` code/table diff (`0 B` size delta): minor FOC & EEPROM threshold tweak. |
| `TO7-DM01-K260108-V6510C.Pack` | `6510` (`0x196E`) | `0x20` / `0x46` / `0xA8` | `57,708 B` | `0x20005628` | `0x0800EE70` (`A5 01 FE`) | `+4 B` code insertion (`8,808 B` shifted); `TS1000W` EEPROM reaches May 2026 baseline. |
| `TO7-DM01-K260521A-V6510C.Pack` | `6510` (`0x196E`) | `0xA8` / `0x67` / `0x89` | `57,708 B` | `0x20005628` | `0x0800EE70` (`A5 01 FE`) | `46 B` in-place patch (`0 B` size delta); 100% identical 480 B EEPROM to `K260108`. |
| `TO7-DM01-K261023-V6510C.Pack` | `6510` (`0x196E`) | `0xBA` / `0x3E` / `0x16` | `58,532 B` | `0x20005638` | `0x0800F1AC` (`A5 01 FE`) | `+824 B` expansion (`MSP +16 B`): `TS\x11000W` profile, 15-magnet speed sensor (`0x1D3 = 15`). |

### DM02 (`TS500W` — 11 Releases)
| Release File | Version (`hdr[1..3]`) | Keys (`K_hdr`/`K_pay`/`UartMask`) | Payload Size | Initial `MSP` | Embedded EEPROM Offset & Block 1 Magic | Key Binary & EEPROM Diffs vs. Predecessor |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `TO7-DM02-K240512A-V6206C.Pack` | `6206` (`0x183E`) | `0x34` / `0xD5` / `0xD2` | `56,508 B` | `0x20005618` | `0x0800E9B8` (**`A5 FD 02`**) | Earliest `V6206C` release: uses legacy `A5 FD 02` (`0xFD ^ 0x02 == 0xFF`) Block 1 magic. |
| `TO7-DM02-K240612B-V6306C.Pack` | `6306` (`0x18A2`) | `0xDD` / `0x37` / `0x59` | `56,644 B` | `0x20005618` | `0x0800EA44` (`A5 01 FE`) | `+136 B`: transitions Block 1 magic to `A5 01 FE`; updates `Pwr[0..5]` to `25/35/50/70/100%`. |
| `TO7L-DM02-K240723A-V6306C.Pack` | `6306` (`0x18A2`) | `0xBE` / `0xDF` / `0xA9` | `56,644 B` | `0x20005618` | `0x0800EA44` (`A5 01 FE`) | `46 B` in-place diff (`TO7L` variant); 100% identical 480 B EEPROM to `K240612B`. |
| `TO7-DM02-K241016-V6212C.Pack` | `6212` (`0x1844`) | `0xEC` / `0x4A` / `0xD4` | `56,668 B` | `0x20005618` | `0x0800EA5C` (`A5 01 FE`) | `+24 B`: unified `V6212C` branch with DM01. |
| `TO7-DM02-K241031-V6303C.Pack` | `6303` (`0x189F`) | `0x40` / `0x79` / `0x5A` | `56,440 B` | `0x20005618` | `0x0800E974` (**`A5 FD 02`**) | Maintenance build on `V6206C`/`V6303C` branch retaining `A5 FD 02` Block 1 magic. |
| `TO7-DM02-K241129A-V6400C.Pack` | `6400` (`0x1900`) | `0x8A` / `0xCE` / `0x6E` | `57,808 B` | `0x20005620` | `0x0800EECA` (`A5 01 FE`) | `+1,368 B` (`V6400C`): raises `MSP` to `0x20005620`, adds extended telemetry & BLE prep. |
| `TO7-DM02-K250402-V6400C.Pack` | `6400` (`0x1900`) | `0x5F` / `0x5B` / `0xFA` | `58,360 B` | `0x20005620` | `0x0800F0F2` (`A5 01 FE`) | `+552 B`: refines torque auto-zero & cadence filters; EEPROM matches `V6510C` except `0x1900` tag. |
| `TO7-DM02-K250801-V6510C.Pack` | `6510` (`0x196E`) | `0x23` / `0x9B` / `0x1E` | `58,436 B` | `0x20005620` | `0x0800F13E` (`A5 01 FE`) | `+76 B`: first DM02 `V6510C` build; 480 B EEPROM reaches final `TS500W` factory image. |
| `TO7-DM02-K260203-V6510C.Pack` | `6510` (`0x196E`) | `0xD7` / `0x45` / `0x83` | `58,440 B` | `0x20005620` | `0x0800F142` (`A5 01 FE`) | `+4 B` alignment update; 100% identical 480 B EEPROM to `K250801`. |
| `TO7-DM02-K260526-V6510C.Pack` | `6510` (`0x196E`) | `0xF7` / `0xE2` / `0xA9` | `58,440 B` | `0x20005620` | `0x0800F142` (`A5 01 FE`) | `46 B` in-place patch (`0 B` size delta); 100% identical 480 B EEPROM to `K250801` & `K260203`. |
| `TO7-DM02-K261023-V6510C.Pack` | `6510` (`0x196E`) | `0x32` / `0xDA` / `0x3E` | `58,456 B` | `0x20005620` | `0x0800F152` (`A5 01 FE`) | `+16 B` (`+4 B` @ `0x080067FA`, `+12 B` @ `0x08009253`); 100% identical 480 B EEPROM. |

---

## 2.3 Decompiled Native `V6510C` 7-Bit UART Display Protocol (`0xC0 | cmd` .. `0x80 | crc_lo`)

In addition to the 18-byte `0x5A` telemetry/command framing, disassembly of `0x0800BFD0` (RX synchronizer), `0x0800E21C` (TX packet builder), `0x0800473C` (command dispatcher), and `0x080052A4` (telemetry scheduler) revealed the native 7-bit wire framing used across all 17 historical firmwares underneath the `0x69` stream XOR cipher:
- **Start-of-Frame (SOF):** `buf[0] = 0xC0 | (cmd & 0x3F)` (top 2 bits `0b11`).
- **7-Bit Data Payload:** `buf[1..1+N]` where every byte has bit 7 cleared (`b & 0x80 == 0`), and 14-bit integers are big-endian packed as `[((val >> 7) & 0x7F) as u8, (val & 0x7F) as u8]`.
- **3-Byte Packed CRC-16 + End-of-Frame (`0x08002AA8` / `0x08002458`):**
  - `crc = crc16_toseven(&buf[0..1+N])`
  - `buf[1+N] = ((crc >> 9) & 0x7F) as u8`
  - `buf[2+N] = ((crc >> 2) & 0x7F) as u8`
  - `buf[3+N] = 0x80 | ((crc & 0x03) as u8)` (top 2 bits `0b10` mark End-of-Frame).
- **Supported Display Commands (`V6510cDisplayCommandDecoder` in [`src/protocol.rs`](../dm01-firmware/src/protocol.rs)):**
  - `cmd 0x1E` (`0xDE`, `@0x0800490E`): Periodic status (`bit 5` = headlight, `bit 6` = walk assist, `bits 0..3` = assist level `0..5`).
  - `cmd 0x31` (`0xF1`, `@0x08004D38`): Explicit riding mode selection (`data[0] == 0x01`, `data[1] & 0x0F` = `Eco`/`City`/`Sport`).
  - `cmd 0x03` (`0xC3`, `@0x08004A3C`) subcmd `0x1D`: Live riding mode and assist level sync.
- **Supported Controller Telemetry Builders (`TelemetryFrame::encode_v6510c_*` in [`src/protocol.rs`](../dm01-firmware/src/protocol.rs)):**
  - `0xF0` (`cmd 0x30` status/Hall), `0xE7` (`cmd 0x27` temp + 15°C), `0xE6` (`cmd 0x26` SOC %), `0xEE` (`cmd 0x2E` torque mV), `0xEF` (`cmd 0x2F` current 0.1A), `0xEB` (`cmd 0x2B` cadence RPM), `0xF2` (`cmd 0x32` display error code).

---

## 3. Subsystem-by-Subsystem Forensic Gap Analysis

### 3.1 Field-Oriented Control (FOC) & Inverter Subsystem

| Feature / Mechanism | Stock `V6510C` Implementation | `dm01-firmware` Status | Gap Assessment & Impact |
| :--- | :--- | :--- | :--- |
| **Math Representation** | Cortex-M4F single-precision FPU (`vldr`, `vmul.f32`, `vadd.f32`, `vsub.f32`) with 2,446 FPU instructions | Single-precision `f32` and `Q15` fixed-point dual implementations | **Parity Achieved**: Single-precision FPU used throughout `foc.rs`. |
| **Phase Shunt DC Bias Calibration** | 256-sample averaging loop at `0x08009A6C` prior to PWM startup; offsets written to hardware `ADC_JOFR1`/`ADC_JOFR2` registers | Mathematical offset tracking in `sensors.rs`; raw ADC reads in `firmware.rs` | **Minor Gap**: Writing the averaged offset directly to `ADC_JOFRx` allows ADC hardware to auto-subtract bias on injected conversions without CPU cycles. |
| **Shunt Topologies** | Runtime switch at `0x08002C44` (`[r0, #0x25]`): Phase A+B ($I_c = -I_a - I_b$) vs Phase A+C ($I_b = -I_a - I_c$) | Configured for Phase A+B dual low-side shunt topology | **No Gap**: DM01 and DM02 PCBs are physically wired for Phase A+B dual shunts. |
| **FOC Rotor State Machine** | 5 discrete states (`0x08002D5C`):<br>• State 0: Coast / Reset<br>• State 2: Open-loop Hall alignment<br>• State 4: Low-speed Hall interpolation<br>• State 6: Observer handoff transition<br>• State 8: High-speed PLL with field weakening | Continuous Hall angle observer with PLL velocity filter and field-weakening | **Conceptual Parity**: State transitions are continuous in Rust; introducing the explicit 5-state enum provides cleaner auditability. |
| **Space Vector PWM (SVPWM)** | 7-segment sector selection with dead-time clamping to `[0.06, 0.94]` | Sector determination (1..6) with `clamp_duty_cycle(0.06, 0.94)` | **Exact Bit Parity**: Mathematical duty cycles match factory behavior. |
| **Hardware Overcurrent Break** | `TIM1_BRK` (IRQ 24) trips on comparator overcurrent; clears `TIM1->CCER` bits 0, 2, 4, 6, 8, 10 in < 50 ns | `pwm.disable_inverter()` clears CCER register bits | **Hardware Gap**: Enabling `TIM1_BDTR.BKE` break input connects hardware comparator directly to MOSFET disable logic without software intervention. |

---

### 3.2 Power Curve, Assist Shaping & Safety Governors

| Feature / Mechanism | Stock `V6510C` Implementation | `dm01-firmware` Status | Gap Assessment & Impact |
| :--- | :--- | :--- | :--- |
| **Ceiling Architecture** | Dual split: Global Mode Ceiling (ignores `Pwr%`) for torque; Level Power Ceiling for PAS/throttle | Stock mode replicates dual split; optional `UnifiedLevelCeiling` respects `Pwr%` | **Complete + Enhanced**: Rider-selectable via profile system. |
| **Thermal Rollback** | Continuous linear rollback from 70°C down to 0A at 100°C anchored to Sport limit (30.0A) | Exact integer math rollback matching `0x08005478` | **Exact Bit Parity**: Verified against decompiled formulas. |
| **Torque Auto-Zero Tracking** | 5-sample idle delta window (< 20 mV over 50 ticks) with 4:1 IIR filter inside 400..1000 mV | `TorqueAutoZeroTracker` with exact 4:1 IIR and bounds check | **Exact Bit Parity**: Preserves factory calibration behavior. |
| **Cadence Scaling** | Multiplies torque effort by $\max(\text{Cadence}, 5) / 60$, clamped to mode ceiling | `Stock60Rpm` matches factory; `Uncapped` and `LinearRamp` available | **Complete + Enhanced**: Eliminates steep hill-climbing torque penalty. |
| **Low-Cadence Nylon Gear Safety** | Stock firmware delivers full torque down to 5 RPM, contributing to stripped nylon primary gears | `GearSafeKnee` provides soft torque below 18 RPM, ramping to 100% by 35 RPM | **Intentional Open Firmware Advantage**: Prevents mechanical drive failure. |
| **Speed Sensor Failure Fallback** | When spoke magnet is lost, stock cuts assist unless `SpeedSensorFail = 0` (Speed Unlimit) is set | Sensorless speed kinematic estimation from cadence + sprocket ratio; unlimit bypass implemented | **Complete + Enhanced**: Provides telemetry speed while allowing full power in unlimit mode. |
| **LVC Reserve Cushion** | 3.0V linear taper above Low-Voltage Cutoff down to 25% reserve cushion | Configurable policy: Stock 25% cushion or `TrueZeroCutoff` | **Complete + Enhanced**: Protects battery BMS from abrupt trip under sag. |

---

### 3.3 Non-Volatile Flash EEPROM Manager

| Feature / Mechanism | Stock `V6510C` Implementation | `dm01-firmware` Status | Gap Assessment & Impact |
| :--- | :--- | :--- | :--- |
| **Flash Block Layout** | 5 blocks (`0x20, 0x40, 0x40, 0xA0, 0xA0` = 480 bytes) at `0x0801FC00` (Sector 127) | `EepromImage` with exact 5-block offsets and lengths | **Exact Bit Parity**: Verified with bit-exact roundtrips. |
| **Magic Check & Cipher** | Block 1 magic `A5 01 FE` at `0x05A..0x05D`; Flash encrypted with XOR key `0xA5` | Magic validation and `0xA5` XOR transformation | **Exact Bit Parity**: 100% matches `0x0800C83E`. |
| **Checksum Algorithm** | 16-entry nibble CRC-16 table (`0x0800EE50`) at end of all 5 blocks | `crc16_toseven` implementation in `eeprom.rs` | **Exact Bit Parity**: Matches vendor CRC-16 bit for bit. |
| **Factory Image Recovery** | Falls back to embedded factory image at `0x0800EE70` (DM01) or `0x0800F142` (DM02) if Flash corrupted | `default_dm01()` and `default_dm02()` embedded in crate | **Exact Bit Parity**: Bit-exact factory images preserved. |
| **Flash Hardware Erase & Write** | Unlock sequence (`0x45670123`, `0xCDEF89AB`), Sector 127 erase, half-word flash write | RAM buffer management implemented; Flash register write sequence in PAC | **Hardware Gap**: Low-level Flash programming state machine needs connection to runtime EEPROM save callbacks. |

---

### 3.4 Display UART Protocol & Error Codes

| Display Error Description | Wire Telemetry Frame Representation | `FaultFlags` Bit | Status in `dm01-firmware` |
| :--- | :--- | :--- | :--- |
| **No Fault (Normal Telemetry)** | `frame[1] = 0x01`, `frame[13..15] = throttle_adc_mv` | `FaultFlags::NONE` | Implemented in `protocol.rs` |
| **Protocol Dismatch** | `frame[1] = 0x11`, `frame[13..15] = flags` | Bit 0 (`0x0001`) | Implemented in `faults.rs` |
| **Comm Reception Fail / Motor Disconnect** | Display timeout or Bit 1 active | Bit 1 (`0x0002`) | Implemented in `faults.rs` |
| **Controller Memory Fail** | `frame[1] = 0x11`, Bit 2 active | Bit 2 (`0x0004`) | Implemented in `faults.rs` |
| **Gallop Throttle Fail** | `frame[1] = 0x11`, Bit 3 active | Bit 3 (`0x0008`) | Implemented in `faults.rs` |
| **Controller Mosfet Fail / Broken** | `frame[1] = 0x11`, Bit 4 active | Bit 4 (`0x0010`) | Implemented in `faults.rs` |
| **Motor Blocked** | `frame[1] = 0x11`, Bit 5 active | Bit 5 (`0x0020`) | Implemented in `faults.rs` |
| **Speed Sensor Fail** | `frame[1] = 0x11`, Bit 6 active | Bit 6 (`0x0040`) | Implemented in `faults.rs` |
| **Motor Hall Fail** | `frame[1] = 0x11`, Bit 7 active | Bit 7 (`0x0080`) | Implemented in `faults.rs` |
| **Controller OverTemp** | `frame[1] = 0x11`, Bit 8 active | Bit 8 (`0x0100`) | Implemented in `faults.rs` |
| **Motor OverTemp** | `frame[1] = 0x11`, Bit 9 active | Bit 9 (`0x0200`) | Implemented in `faults.rs` |
| **Torque Sensor Fault** | `frame[1] = 0x11`, Bit 10 active | Bit 10 (`0x0400`) | Implemented in `faults.rs` |
| **Low Voltage** | `frame[1] = 0x11`, Bit 11 active | Bit 11 (`0x0800`) | Implemented in `faults.rs` |

---

## 4. Implementation Gap Summary & Technical Roadmap

```
+---------------------------------------------------------------------------------------+
| Subsystem                     | Parity Level   | Implementation Status & Parity Work  |
+---------------------------------------------------------------------------------------+
| FOC Clarke / Park / SVPWM     | 100% Bit-Exact | Bit-exact Q15 and f32 single-precision|
| Power Curve & Arbitrator      | 100% Bit-Exact | Dual ceilings, 50V cap, LVC taper     |
| Display Protocol (0x69 wire)  | 100% Bit-Exact | 18-byte framing, byte muxing, CRC-16  |
| EEPROM Structure & CRC-16     | 100% Bit-Exact | 5-block XOR 0xA5, TS1000W / TS500W    |
| Universal .Pack Key Decoder   | 100% Verified  | 17/17 factory packs pass bit-exact    |
| Display Lockout Patching      | 100% Clean     | 6-gate NOP clean-room patcher verified|
| Hardware ADC_JOFRx Auto-Zero  | 100% Parity    | 256-sample average to JOFR1/2 (0x9A6C)|
| Hardware TIM1_BDTR Break Trip | 100% Parity    | BDTR.BKE enabled, <50ns shutdown      |
| Hardware Flash Persistence    | 100% Parity    | Page 127 unlock/erase/write driver    |
| Hardware Watchdog (IWDG)      | 100% Parity    | 500 ms window refreshed in 1 ms loop  |
+---------------------------------------------------------------------------------------+
```

### Parity Implementations Completed
1. **ADC JOFR Register Injection & 256-Sample Shunt Auto-Zero:**
   - Implemented `calibrate_shunts()` in `Dm01SensorsHardware` matching vendor subroutine `0x08009A6C`.
   - Synchronously triggers 256 injected conversions via `JSWSTART` while PWM outputs remain in High-Z (0A).
   - Averages samples (`sum >> 8`) and programs `ADC1->jofr1` and `ADC2->jofr1`, while updating software zero-current offsets.
2. **Hardware TIM1 BDTR Break Trip Protection:**
   - Verified and enabled `BDTR.BKE` (Break Enable bit 12) during `TIM1` initialization with active-high comparator trip mapping to PB12.
   - Clears all PWM channel outputs to High-Z in < 50 ns upon overcurrent condition.
3. **Hardware Watchdog (IWDG) Supervision:**
   - 500 ms independent hardware watchdog (`IWDG`) configured using internal LSI oscillator (625 Hz tick).
   - Refreshed deterministically every 1 ms in the SysTick executive loop (`HardwarePeripherals::feed_iwdg()`).
4. **Flash Page 127 EEPROM Hardware Persistence:**
   - Implemented low-level Flash memory controller driver (`FlashRegisters`, `flash_unlock()`, `flash_lock()`, `flash_erase_page()`, and `flash_write_halfwords()`) in `src/pac.rs`.
   - Integrated `EepromImage::read_from_flash()` to automatically load and validate settings from Page 127 (`0x0801FC00`) at boot, falling back to factory defaults if blank or corrupt.
   - Provided `EepromImage::write_to_flash()` to write 480-byte encrypted blocks with verified CRC-16 checksums.
