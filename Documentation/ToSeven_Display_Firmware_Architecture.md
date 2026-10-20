# ToSeven Display Firmware Architecture & Reverse Engineering Specification

## Executive Summary
This document provides the clean-room technical reverse-engineering specification of the display firmware architecture across **ToSeven (TO7)** and OEM **MinShine Electronics** (`cn.minshine.ebike`) mid-drive e-bike displays:
* **T24V:** Vertical high-contrast color display (`TO7-T24V-Y240527A.hex`, `TO7-T24V-Y241016.hex`, `TO7-T24V-Y241225.hex`).
* **LCD135C:** Landscape color display with integrated BLE 5.0 gateway (`TO7-LCD135C-Y240621A.hex`).
* **M15:** Compact minimalist handlebar display (`TO7-M15-Y250928.hex`).
* **T154:** Mid-sized high-resolution color display (`TO7-T154-Y250313.hex`).

All findings are based on binary analysis of official Intel HEX dumps, ARM Thumb-2 disassembly, and cross-verification with the controller In-Application Programming (IAP) flashing subsystem (`dm01-firmware/src/flasher.rs`).

---

## 1. Microcontroller & Memory Architecture

### 1.1 Hardware Platform (MegaHunt MHSCPU)
* **Target MCU Core:** ARM Cortex-M4F / Cortex-M3 (MegaHunt Security CPU series, MH1903 / MH2020C).
* **Clock & Peripherals:** Configured via MegaHunt SDK (`@..\SDK\Libraries\MHSCPU_Driver\src\mhscpu_uart.c`).
* **External Flash Memory:** Winbond Serial NOR Flash interfaced via SPI (auto-detected via JEDEC command `0x90`: `0xEF13` = W25Q80, `0xEF14` = W25Q16, `0xEF15` = W25Q32, `0xEF16` = W25Q64).

### 1.2 Memory Map
| Region | Base Address | End Address | Description |
| :--- | :--- | :--- | :--- |
| **Boot ROM / ISP Bootloader** | `0x01000000` | `0x01000FFF` | 4 KB on-chip MegaHunt ISP bootloader (communicates via `MH2020C_ISP.exe` at 115200 Baud). |
| **Application Flash** | `0x01001000` | `0x010D5FFF` | Main display application firmware (starts with ARM vector table). |
| **Internal SRAM** | `0x20000000` | `0x20010000` | 64 KB high-speed system RAM (initial MSP `0x2000B000..0x2000C8C8`). |
| **Display RX/TX Ring Buffers** | `0x200017D8` / `0x20001BD8` | — | Circular UART buffers for controller communication. |
| **Live Settings Cache** | `0x2000033C` | `0x200003DA` | 158-byte runtime configuration block (CRC-16-CCITT validated). |
| **Security & Password State** | `0x2000041C` | `0x20000438` | User PIN (`+0x06..+0x09`), Master PIN (`+0x13..+0x16`), and Unlock Flag (`+0x98`). |
| **UART Peripherals (APB)** | `0x40016000`, `0x4001D000` | — | Memory-mapped UART data, status, and baud rate divisor registers. |

### 1.3 Vector Table Signatures Across Tested Displays
Every official display dump begins at `0x01001000` with standard ARM Cortex-M vector entries:
```text
[0x01001000] Initial MSP
[0x01001004] Reset_Handler (Thumb address, bit 0 = 1)
```

| Display Model | Firmware Image | Image Size | Initial MSP | Reset_Handler | Advanced Menu Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **LCD135C** | `TO7-LCD135C-Y240621A.hex` | 872,180 bytes | `0x2000C8C8` | `0x01001D0D` | Fully Unlocked (0 lockout checks) |
| **M15** | `TO7-M15-Y250928.hex` | 628,772 bytes | `0x2000AF00` | `0x01010B01` | Fully Unlocked (0 lockout checks) |
| **T154** | `TO7-T154-Y250313.hex` | 403,184 bytes | `0x2000AF60` | `0x01015BE5` | Restricted (4 lockout checks) |
| **T24V (May 2024)** | `TO7-T24V-Y240527A.hex` | 174,200 bytes | `0x2000B008` | `0x0101726D` | Fully Unlocked (0 lockout checks) |
| **T24V (Oct 2024)** | `TO7-T24V-Y241016.hex` | 174,552 bytes | `0x2000B010` | `0x010173D1` | Restricted (4 lockout checks) |
| **T24V (Dec 2024)** | `TO7-T24V-Y241225.hex` | 174,716 bytes | `0x2000B018` | `0x01017475` | Restricted (4 lockout checks) |

---

## 2. Display UART Protocol & Wire Cipher (`0x69`)

### 2.1 Wire Stream Encryption
Communication between the Display and the Motor Controller is stream-encrypted using a byte-wise XOR cipher with key **`0x69`**.
* In `T24V` (`0x01015670`), every byte received over UART is immediately decrypted:
  ```assembly
  01015670: b510         push    {r4, lr}
  01015672: 4604         mov     r4, r0
  01015674: f084 0469    eor.w   r4, r4, #0x69       ; <--- Plaintext = Wire ^ 0x69
  01015678: 4842         ldr     r0, [pc, #0x108]
  010156ba: 540c         strb    r4, [r1, r0]        ; Store in ring buffer (0x20001BD8)
  ```
* Identical XOR masking is mirrored in the Motor Controller (`0x080095D8` RX / `0x0800966C` TX in `V6510C`) and implemented in [`dm01-firmware/src/protocol.rs`](../dm01-firmware/src/protocol.rs).

### 2.2 Controller EEPROM 5-Block Synchronization Protocol
When the display updates or restores configuration parameters, it synchronizes with the motor controller Flash EEPROM (`0x0801FC00`) by streaming all 5 blocks through procedure `0x0100CBFA`:
```
Display (T24V)                                     Motor Controller (Cortex-M4F)
--------------                                     ----------------------------
  |                                                             |
  |  1. Stream Block 1 (32 bytes = 0x20) @ 115200 Baud          |
  |------------------------------------------------------------>|
  |     [50 ms pacing delay]                                    |
  |  2. Stream Block 2 (32 bytes = 0x20)                        |
  |------------------------------------------------------------>|
  |     [50 ms pacing delay]                                    |
  |  3. Stream Block 4 (160 bytes = 0xA0)                       |
  |------------------------------------------------------------>|
  |     [50 ms pacing delay]                                    |
  |  4. Stream Block 2 (64 bytes = 0x40)                        |
  |------------------------------------------------------------>|
  |     [50 ms pacing delay]                                    |
  |  5. Stream Block 3 (64 bytes = 0x40)                        |
  |------------------------------------------------------------>|
  |     [Controller computes 16-entry nibble CRC-16 and saves]  |
```
Total transferred data equals **320 bytes**, matching the primary controller Flash EEPROM working image.

---

## 3. The 13 Decompiled Throttle Modes (0 to 12)
Reverse engineering of the display parameter table (`0x0102B834..0x0102B8E8` in `T24V`) revealed the official engineering designations for all 13 throttle behavior modes implemented in the motor controller:

| Mode ID | Display String | Stationary (Standstill) Behavior | While Pedaling (PAS Active) Behavior | Legal / Functional Classification |
| :---: | :--- | :--- | :--- | :--- |
| **0** | `0.Null-6kmh` | Disabled (0 power) | Walk assist ceiling: caps at 6 km/h | Strict EPAC / EU compliant |
| **1** | `01.Null-Level` | Disabled (0 power) | Speed ceiling matches current assist level | Commuter level-matched |
| **2** | `02.Null-Full` | Disabled (0 power) | Full motor speed/power regardless of assist level | Cadence-activated overtake |
| **3** | `03.6Kmh-6Kmh` | Walk assist only (6 km/h) | Caps at 6 km/h | Walk assist boost |
| **4** | `04.6Kmh-Level` | Walk assist only (6 km/h) | Speed ceiling matches current assist level | Mixed walk + assist |
| **5** | `05.6Kmh-Full` | Walk assist only (6 km/h) | Full motor speed/power | Push-start full throttle |
| **6** | `06.Level-Level` | Level speed from standstill | Speed ceiling matches current assist level | Instant takeoff (assist limited) |
| **7** | `07.Spd=Level-1` | Level speed minus 1 step | Speed ceiling matches Level - 1 | Stock factory default (V6400+) |
| **8** | `08.Full-Full` | Full power from standstill | Full motor speed/power | Unrestricted trail / off-road |
| **9** | `09.6kmh-Full` | 6 km/h start assist | Full motor speed/power | Soft ramp to full power |
| **10** | `10.Level-Full` | Level speed start assist | Full motor speed/power | Controlled launch to full power |
| **11** | `11.Full-Full` | Full power from standstill | Full motor speed/power | High-demand cargo / towing |
| **12** | `12.Scooter` | Pure throttle operation | Pedaling optional (no cadence required) | Pure electric moped / scooter |

---

## 4. Button Combos, Diagnostic Menus & Default PINs

### 4.1 Button State Decoder (`0x0100EF00` & `0x0100F700`)
The display samples analog resistor-ladder button inputs through ADC debounce filters rather than discrete GPIO lines. The firmware reads `ADC_CH` and compares the raw voltage reading against calibrated windows (`AD=%4d, Cnt=%2d`):
* **`Up&Down` (`AD ≈ 1200 mV`):** Simultaneous hold of `+` and `-` buttons (enters settings menu after 2000 ms debounce).
* **`Up` (`AD ≈ 1650 mV`):** `+` button (increases assist level, increments value; long-press toggles headlight).
* **`Down&Info` (`AD ≈ 2100 mV`):** Simultaneous hold of `-` and `Info/Power` (resets trip odometer).
* **`Info` (`AD ≈ 2600 mV`):** `Power / Mode` button (toggles display modes: ODO, TRIP, TIME, MAX, AVG; short-press confirms menu selection).
* **`Down` (`AD ≈ 3100 mV`):** `-` button (decreases assist level, decrements value; long-press engages 6 km/h Walk Assist).
* **Idle (No Buttons Pressed):** VDD pull-up (`AD ≈ 3300 mV`).

### 4.2 Factory Key Combinations & Diagnostic Pages
* **Main Settings Menu:** Long-press **Power** to power on; as soon as the screen lights up, simultaneously hold **Power + Up** until the PIN entry prompt appears. Default user PIN is **`1234`**.
* **Advanced Settings Menu (Pre-October 2024 firmware):** In Main Settings, scroll to `"Unit"`, then hold **Up + Down** together for 3 seconds. Enter PIN **`1234`**.
* **Hidden Engineering Diagnostic Screen ("Test" / Live Sensor View):** Repeatedly press **Power** until `"Test"` appears at the bottom bar, then press **Up** to enter.
  * Shows live battery voltage (0.1V resolution), controller temperature (°C), live phase current (A), cadence (RPM), torque sensor output (mV), and throttle ADC (mV).
  * Real-time commutation observer view: displays active Hall sector sequence (`HALL=F-%d%d%d%d%d%d` forward / `HALL=B-%d%d%d%d%d%d` backward) and sensorless fallback observer (`NOHALL=F-...`).
  * Live sensor ratio line: `PAS=%d/THR=%d/TQS=%d` (Crank Cadence RPM, Throttle ADC mV, and Torque Strain Gauge mV).
* **Timer Reset:** Hold **Power + Down** simultaneously for 2 seconds.
* **Firmware Version Display:** Scroll bottom bar to `"Info"`, then press **Down**.

### 4.3 Complete Decompiled Engineering Parameter Tree
Forensic disassembly of the display firmware strings and menu dispatch tables (`0x0102B834..0x0102E500`) revealed the complete internal parameter tree exposed to authorized technicians:

1. **`Speed&Power` Sub-Menu:**
   - `Spd L1..L9`: Per-level speed limit in 1.0 km/h increments (`Spd L%d=%2d kmh`).
   - `pwr L1..L9`: Per-level power percentage cap (`pwr L%d=%3d %%`, scales mode current for Cadence PAS and Throttle).
   - `EAssist-1..9`: Per-level torque effort multiplier (`EAssist-%d=%3d %%`, multiplies human rider wattage).
2. **`Current PID` (FOC Tuning) Sub-Menu:**
   - `CurrentKp` / `CurrentKi`: Direct- and quadrature-axis current loop PI gains.
   - `SpeedKp` / `SpeedKi` / `Speed Filter`: Outer velocity control loop gains.
   - `FluxWeakVoltage` (`90..98%`): Field-weakening threshold voltage percentage.
   - `FluxWeakCurrent` (`0..10 A`): Demagnetizing d-axis flux-weakening current cap.
3. **`Motor & Electrical Limits` Sub-Menu:**
   - `MaxCurrentECO`, `MaxCurrentCITY`, `MaxCurrentSPORT`: Continuous current ceilings per riding mode.
   - `CurrentOffest(a)`: Torque startup initial step current (`0.5 A` stock default).
   - `LowCutVoltage` (`LVC`) / `StandardVoltge`: Low-voltage cutoff and battery pack selection (`24V`, `36V`, `48V`, `52V`).
   - `Capacity=0%..100%`: 5-point state-of-charge (SOC) voltage lookup table.
   - `StartMinVoltage%` / `StartSpeed(rpm)`: Minimum stall duty and initial rotation threshold.
   - `RimSpeedSensor` / `SpeedSensorFail`: Toggles external reed switch vs. universal sensorless speed estimation.
   - `BrakeLight`: Active-high vs. active-low brake light driver mode.
   - `AutoPowerOff`: Idle shutdown timeout in minutes (`0..10`).

---

## 5. Reverse Engineering the Post-October 2024 Advanced Menu Lockout

### 5.1 Root Cause Analysis
Beginning with firmware build `Y241016` (October 2024) on `T24V` and `T154`, ToSeven introduced a two-tier hardware lockout mechanism designed to prevent riders from accessing the **Speed & Power**, **Controller Current Limits**, and **Motor Calibration** menus.

Decompilation of procedure `0x01010E70` in `TO7-T24V-Y241225.hex` revealed the exact lockout logic:
```assembly
; Check Tier 1: Authorization byte in settings RAM
01010e70: 2c01         cmp     r4, #0x1
01010e72: d121         bne     0x1010eb8           ; Continue normal menu
01010e74: 488b         ldr     r0, [pc, #0x22c]    ; Load pointer to settings (0x2000033C)
01010e76: f890 0098    ldrb.w  r0, [r0, #0x98]     ; Load Authorization Flag
01010e7a: 2851         cmp     r0, #0x51           ; Check if authorized (0x51 = 'Q')
01010e7c: d112         bne     0x1010ea4           ; <--- LOCKOUT BRANCH: Skips Advanced Menu!

; Check Tier 2: Controller firmware version check
01010e7e: 488a         ldr     r0, [pc, #0x228]    ; Load controller status struct
01010e80: 8a00         ldrh    r0, [r0, #0x10]     ; Load Controller Version (CtlVer)
01010e82: f641 0142    movw    r1, #0x1842         ; Minimum required version (0x1842)
01010e86: 4288         cmp     r0, r1
01010e88: db0c         blt     0x1010ea4           ; <--- LOCKOUT BRANCH: Skips if CtlVer < 0x1842!
```

### 5.2 The 4 Lockout Inspection Points
Across the entire binary, the lockout byte `[settings + 0x98] == 0x51` is checked at exactly 4 critical gates:
1. **Gate 1 (`0x0100D406`):** Speed & Power edit permission (`bne +10`).
2. **Gate 2 (`0x0100E4A0`):** Controller current limit edit permission (`bne +6`).
3. **Gate 3 (`0x01010E7A`):** Advanced Menu navigation entry (`bne +36` & `blt +24`).
4. **Gate 4 (`0x01011390`):** Secret unlock through `RimSizeUnit` (`"inch or mm"`).

### 5.3 Stock OEM Unlock Mechanism (The "Secret Sequence")
In procedure `0x01011390`, the display evaluates the `RimSizeUnit` setting:
```assembly
010113a0: f1a0 0950    sub.w   r9, r0, #0x50
010113b2: f109 0050    add.w   r0, r9, #0x50       ; Value 1 ('mm') + 0x50 = 0x51 ('Q')
010113b8: f881 0098    strb.w  r0, [r1, #0x98]     ; Sets Authorization Flag to 0x51!
```
When `RimSizeUnit` is toggled from inches to **`mm`** (value `1`), the display stores `0x51` into offset `0x98`, satisfying Gate 1. However, if the motor controller's reported `CtlVer` is less than `0x1842`, Gate 2 still denies entry.

### 5.4 Permanent Clean-Room Patch (Zero-Lockout Modification)
To permanently remove all restrictions, prevent lockouts, and enable the Advanced Menu and all edit menus regardless of motor controller version, replace the conditional branches across all three restriction gates with ARM Thumb `NOP` instructions (`0xBF00` / bytes `[0x00, 0xBF]`):

| Gate & Check | File Target | Virtual Address | Stock Branch | Replacement | Effect |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Gate 1: Speed & Power CtlVer** | `T24V-Y241225` | `0x0100D3FE` | `0A DB` (`blt +20`) | `00 BF` (`nop`) | Bypasses `CtlVer >= 0x1842` for Speed & Power |
| **Gate 1: Speed & Power Auth** | `T24V-Y241225` | `0x0100D408` | `05 D1` (`bne +10`) | `00 BF` (`nop`) | Bypasses `0x51` auth check for Speed & Power |
| **Gate 2: Current Limit CtlVer** | `T24V-Y241225` | `0x0100E498` | `08 DB` (`blt +16`) | `00 BF` (`nop`) | Bypasses `CtlVer >= 0x1842` for Current Limits |
| **Gate 2: Current Limit Auth** | `T24V-Y241225` | `0x0100E4A2` | `03 D1` (`bne +6`) | `00 BF` (`nop`) | Bypasses `0x51` auth check for Current Limits |
| **Gate 3: Advanced Menu Auth** | `T24V-Y241225` | `0x01010E7C` | `12 D1` (`bne +36`) | `00 BF` (`nop`) | Unconditionally unlocks Advanced Menu entry |
| **Gate 3: Advanced Menu CtlVer** | `T24V-Y241225` | `0x01010E88` | `0C DB` (`blt +24`) | `00 BF` (`nop`) | Unlocks Advanced Menu with older/custom controllers |
| **Gate 1: Speed & Power CtlVer** | `T154-Y250313` | `0x0100A2FA` | `0A DB` (`blt +20`) | `00 BF` (`nop`) | Bypasses `CtlVer >= 0x1842` for Speed & Power |
| **Gate 1: Speed & Power Auth** | `T154-Y250313` | `0x0100A304` | `05 D1` (`bne +10`) | `00 BF` (`nop`) | Bypasses `0x51` auth check for Speed & Power |
| **Gate 2: Current Limit CtlVer** | `T154-Y250313` | `0x0100C06C` | `08 DB` (`blt +16`) | `00 BF` (`nop`) | Bypasses `CtlVer >= 0x1842` for Current Limits |
| **Gate 2: Current Limit Auth** | `T154-Y250313` | `0x0100C076` | `03 D1` (`bne +6`) | `00 BF` (`nop`) | Bypasses `0x51` auth check for Current Limits |
| **Gate 3: Advanced Menu Auth** | `T154-Y250313` | `0x0100FB26` | `11 D1` (`bne +34`) | `00 BF` (`nop`) | Unconditionally unlocks Advanced Menu entry |
| **Gate 3: Advanced Menu CtlVer** | `T154-Y250313` | `0x0100FB32` | `0B DB` (`blt +22`) | `00 BF` (`nop`) | Unlocks Advanced Menu with older/custom controllers |

---

## 6. Integrated OTA Flasher in `LCD135C`

Decompilation of `TO7-LCD135C-Y240621A.hex` revealed an embedded In-Application Programming engine named **`Controller Programer V1.0`**:
* **Mobile BLE Reception:** The display connects over Bluetooth Low Energy (device name `"MyeBike"`), accepts `.Pack` update chunks from the Android app (`cn.minshine.ebike`), and stages them in external SPI Flash memory (`"DO NOT POWER OFF DISPLAY - Pack:%d/%d"`).
* **Autonomous Controller Flashing:** The display executes the entire IAP state machine autonomously:
  1. `1.Connectting` (sends `0x02` probe at 115200 Baud).
  2. `2.MCU Identify` (validates `0x02, 0xFD` response).
  3. `3.Start Progaming` (streams 12-byte header frame and 16-byte decrypted firmware chunks).
  4. Verifies gate driver hardware (`"PowerChip: 6288 6287"`).
  5. Completes with verification checksum (`"Sucess"` / `"Verify Fail"`).
* **Display Programming Hotkeys:**
  * `Long <POWER>`: Execute flashing sequence.
  * `Long <UP + DOWN>`: Abort and exit.

This discovery proves that **any ToSeven display can be programmed to serve as an autonomous, computer-free motor controller flasher**.

---

## 7. Unified Open-Source Tooling (`toseven_pack display`)

All display firmware operations are consolidated into the open-source `toseven_pack` CLI utility:

```bash
# 1. Inspect display firmware (model, vector table, and lockout status)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display info blobs/displays/t24v/TO7-T24V-Y241225.hex

# 2. Standalone unlock patcher (outputs unlocked .hex with recomputed checksums)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display patch blobs/displays/t24v/TO7-T24V-Y241225.hex build/TO7-T24V-Y241225-Unlocked.hex

# 3. Flasher with on-the-fly unlock (dry-run simulation)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display flash blobs/displays/t24v/TO7-T24V-Y241225.hex --unlock --dry-run

# 4. Live hardware flashing via USB CDC port
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display flash /dev/tty.usbmodem110 blobs/displays/t24v/TO7-T24V-Y241225.hex --unlock
```

---

## 8. Roadmap To-Do: All-in-One GUI Patcher & Flasher

The architectural foundation is established for a cross-platform desktop graphical interface (**ToSeven Control Center**):
1. **Frontend Architecture:** Tauri 2.0 (Rust backend with light web UI) or Slint (native UI).
2. **Unified Flashing & Patching:**
   - Auto-detects connected USB ports (`MegaHunt CDC USB` for displays, `FTDI/CH340` for controllers).
   - Drag-and-drop firmware loading with visual lockout detection.
   - One-click "Unlock & Flash" for displays.
   - Interactive motor controller tuning sliders with direct `.Pack` generation and flashing.
3. **Safety Verification:** Built-in guidance for 48V current-limited bench power supply verification before flashing.

