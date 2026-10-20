# DM+ : Open Mid-Drive Firmware & Tooling Studio

[![CI](https://github.com/antaxiom/dm0x-plus-firmware/actions/workflows/ci.yml/badge.svg)](https://github.com/antaxiom/dm0x-plus-firmware/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B%20%7C%20%23!%5Bno__std%5D-orange.svg)](dm01-firmware/Cargo.toml)
[![Target](https://img.shields.io/badge/target-ARM%20Cortex--M4F-blue.svg)](https://developer.arm.com/Processors/Cortex-M4)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

An open-source `#![no_std]` Embedded Rust firmware recompilation, display unlocking toolchain, and cross-platform desktop studio for **ToSeven (TO7)** mid-drive e-bike motors (DM01, DM02) and handlebar displays (T24V, T154, M15, LCD135C).

> [!NOTE]
> **DM+** is the current working project name for this unified firmware and tooling ecosystem. Naming may evolve as the project expands to additional motor controllers and displays.

> [!WARNING]
> ### Disclaimer of Warranty & Equipment Damage Liability
> **THIS SOFTWARE AND FIRMWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED.**
> Flashing, configuring, or operating experimental, custom, or reverse-engineered firmware on electric bicycles, motors, controllers, and displays carries inherent electrical, mechanical, and safety risks. **Any damage to your equipment—including blown MOSFETs, thermal degradation, stripped primary reduction gears, damaged battery packs, blown BMS units, harness damage, electrical shorts, or permanently bricked motor controllers and displays—is solely and entirely your own responsibility.**
> No warranty is provided, whether express, implied, or statutory, including warranties of merchantability, fitness for a particular purpose, and non-infringement. Proceed strictly at your own risk.

> [!CAUTION]
> ### CRITICAL SAFETY WARNING: BENCH TESTING ONLY AT THIS STAGE
> **DO NOT INSTALL OPEN-SOURCE FIRMWARE ONTO AN ASSEMBLED ELECTRIC BICYCLE OR ATTEMPT TO RIDE WITH IT.**
> At this stage, the DM+ open-source motor controller firmware (`dm01_firmware` / `dm02_firmware`) is in active beta bring-up and **MUST NOT BE INSTALLED ANYWHERE OTHER THAN A CONTROLLED BENCH TEST ENVIRONMENT.**
> - **Bench Testing Exclusively:** Only operate controllers on an adjustable DC bench power supply strictly current-limited to $\le 1.0\text{ A}$ at $48\text{ V}$.
> - **Never Use a Lithium Battery for Initial Testing:** Do not test open-source firmware using a high-discharge lithium-ion battery pack.
> - **Oscilloscope & Phase Verification First:** Phase commutation, ADC calibration, shunt offset subtraction, and FOC loop timings must be verified on the bench before any mechanical load is applied.
> - **Flashing Factory Stock is Safe:** Flashing official stock `.Pack` files or patching display firmware (`.hex`) for menu unlock is fully supported. However, running custom open motor firmware on an actual bicycle is strictly prohibited at this stage.

---

## What Is Implemented Right Now (`v0.3.0-beta.7` - Release Beta)

Rather than relying on future promises, here is the verified functionality available today in this repository:

1. **DM+ Firmware Studio Desktop GUI (`dm-plus-gui`):**
   - Native cross-platform desktop application built with Slint and Rust.
   - Built-in embedded **DM01** and **DM02** Open Firmware containers (`128 KB` each) for instant, standalone flashing without requiring external files.
   - Automatic target hardware detection when a custom `.Pack` file is loaded (detecting DM01 vs. DM02 from container metadata), locking the hardware model selector to prevent mismatched flashing.
   - Intelligent parameter isolation: dynamically grays out and disables both the Tuning Profile card and the Battery & Electrical Limits card when a custom `.Pack` file is chosen, since compiled firmware packages contain fixed factory tables and cannot be reconfigured without compiling from source. All section titles remain consistently legible.
   - Standardized physical hardware model naming (`DM01 (External 12-FET)` and `DM02 (Integrated 6-FET)`), accurately reflecting that all power variants of DM01 (500W/750W/1000W) and DM02 (250W/350W/500W) share identical respective controller hardware and firmware.
   - Native system file pickers (`rfd`) with enforced extension filters (`.Pack` and `.hex`).
   - Balanced 2-column card geometry across both Motor Controller and Display tabs, eliminating empty whitespace padding and aligning the Flashing window dimensions and button styling identically across both views.
   - Dedicated "Save Patched .hex..." export button in the Display Patcher card to generate and save unlocked Intel HEX firmware offline without requiring serial hardware.
   - Synchronized ISP and Motor Flashing windows with uniform port selectors, dynamic 1200 ms background auto-refresh on device plug/unplug, manual "Refresh" trigger buttons, clear progress feedback, and non-blocking asynchronous serial engine with connection timeouts.
   - Rigid Adwaita-style segmented view switcher with mathematically locked tab widths (`min-width: 200px; max-width: 200px;` inside a `410px` pill container), completely eliminating horizontal jitter, text clipping, and tab resizing when toggling between regular and semibold font weights.
   - Clean professional header layout, with redundant branding logos removed from beside the title text to maintain a crisp, native titlebar appearance.
   - Official DM+ branding assets across all supported operating systems: Apple HIG-standard squircle icon (`AppIcon.icns`, `logo.ico`, `logo.png`, `logo.svg`, and Linux `.desktop`) featuring an $824 \times 824$ squircle centered on a $1024 \times 1024$ canvas, $185\text{px}$ corner radius, and subtle multi-layer drop shadows.
   - Dynamic macOS Dock icon integration: unbundled applications running via `cargo run` automatically paint the application dock icon and update `dockTile.setContentView:` with the squircle logo.
   - Native Windows subsystem configuration (`#![windows_subsystem = "windows"]`), completely eliminating unwanted background command prompt/terminal windows when launching `dm-plus-gui.exe`.
   - Comprehensive multi-tier Windows COM port discovery combining `QueryDosDeviceW`, registry `SERIALCOMM` enumeration with 64-bit handle sign extension, and direct `CreateFileW` probing fallback across `COM1..COM32` (detecting restricted or active virtual USB serial adapters via `ERROR_ACCESS_DENIED` and `ERROR_SHARING_VIOLATION`).
   - Pre-built binary packages generated for Linux x86_64, Windows x64, and macOS Universal (`.app`).

2. **Verified Motor Controller IAP Flasher & Container Engine (`toseven_pack` & GUI):**
   - **Dual-Tier Container & Wire Stream Cipher Architecture:**
     - **Tier 1 (Container Encryption):** Universal mathematical auto-key formulas verified across 100% of all 15 official stock releases:
       $$K_{\text{head}} = (\text{pack}[32] + \text{pack}[33]) \pmod{256}$$
       $$\text{Header CRC} = \text{crc16\_toseven}(\text{header}[0..30]) = \text{header}[30..32]$$
       $$K_{\text{payload}} = (\text{header}[30] + \text{header}[31]) \pmod{256}$$
       $$\text{Payload Checksum} = (\text{header}[26..28] \ll 16) \mid \text{header}[28..30]$$
     - **Tier 2 (Physical Wire Stream Cipher):** Physical UART stream masking during flash writing where every chunk byte is XORed with `UartMask` ($\text{byte}_{\text{wire}} = \text{byte}_{\text{plain}} \oplus \text{UartMask}$, parsed from container header offset `0x17`), matching the bootloader's on-the-fly hardware de-masking into Flash memory (`0x08001800`).
   - **Full Handshake Protocol Engine:**
     - **Step 1:** Sends 21-byte wakeup burst (`0xC9, 0x77` encrypted with symmetric `0x69` stream cipher) at 9600 Baud with high/low nibble `UartMask` distribution (bytes 12-13) and hardware DTR reset pulse fallback to wake running application controllers into IAP bootloader mode.
     - **Step 2:** Bootloader probe at 38400 Baud (vendor factory bootloader default) with automatic 115200 Baud fallback (`0x02` $\rightarrow$ `0x02, 0xFD` or `0x06`).
     - **Step 3:** 12-byte header handshake (`0x06` ACK $\rightarrow$ 10 ms delay $\rightarrow$ 12-byte header frame containing program size, 32-bit checksum, `UartMask`, and baud index $\rightarrow$ `0x06` ACK).
     - **Step 4:** Streams payload in 16-byte (`0x10`) chunks with inter-chunk pacing delay (~1 ms) and 1024-byte (1 KB) sector commit acknowledgments (`0x06` ACK).
     - **Step 5:** Bootloader cumulative 32-bit byte sum verification against Flash contents (`0x04` EOT on match; `0x15` NAK on failure), confirmed matching physical controller hardware.
   - Automatic hardware target detection (`Dm01` with `0xA8`/`0x67` vs. `Dm02` with `0xF7`/`0xE2`).
   - Bit-exact unpacking, verification, and repacking matching official vendor tools bit-for-bit.
   - Natively extracts loadable segments directly from Cortex-M ELF binaries and packages them into official 128 KB `.Pack` containers for both DM01 and DM02 without requiring host `objcopy`.

3. **Display Lockout Patcher & ISP Flasher (`toseven_pack display`):**
   - Automatically parses Intel HEX display firmware dumps (`T24V`, `T154`, `LCD135C`, `M15`).
   - Detects and neutralizes all 6 restriction branch gates introduced in the post-October 2024 factory firmware with ARM Thumb NOP (`0x00BF`) patches.
   - Restores permanent access to `Speed&Power`, maximum current limits, and wheel circumference menus.
   - Flashes patched display firmware directly over USB CDC using the MegaHunt ISP bootloader protocol.

4. **Clean-Room Motor Controller Architecture (`dm01-firmware`):**
   - 100% `#![no_std]` embedded Rust implementation of Field Oriented Control (FOC), Clarke/Park transforms, SVPWM duty generation with dead-time clamps ($[6.0\%, 94.0\%]$), and Hall observer.
   - Unified multi-target motor control support for both **DM01** (external 12-FET, 30A Sport, 40.27:1 reduction) and **DM02** (internal 6-FET, 18A Sport, 53.00:1 reduction).
   - Dedicated bare-metal Cortex-M4F binary targets for both motors: `dm01_firmware` (MSP `0x20005628`) and `dm02_firmware` (MSP `0x20005620`).
   - **Universal Sensorless Speed Fallback:** Computes static midpoint gear ratio $\text{ratio}_{\text{mid}} = \frac{1}{2}\left(\frac{\text{Crankset}}{\text{Min}} + \frac{\text{Crankset}}{\text{Max}}\right)$ from EEPROM Block 4 (`0x1D0..0x1D2`). When the physical wheel speed sensor disconnects, fails, or loses its magnet, dynamically estimates vehicle speed across all modes (torque, PAS, throttle) using $\text{Cadence} \times \text{ratio}_{\text{mid}} \times C_{\text{wheel}}$, preserving active display speedometer telemetry and speed soft-taper governing.
   - **Automatic Span Tracking (`AutoSpanTracking`):** Dynamic $V_{\text{max}} = V_{\text{zero}} + 2300\text{ mV}$ scaling pinning nominal deflection to $2.3\text{ V}$, eliminating thermal zero-point drift and matching realistic human pedal deflection (GitHub Issue #1).
   - **Gear-Safe Progressive Profile (`GearSafeKnee`):** Soft current ($\le 30\%$) below $18\text{ RPM}$ to protect the blue nylon primary reduction gear against standing-start stomps, ramping to 100% assist at $35\text{ RPM}$ with progressive quadratic modulation (GitHub Issue #2).
   - **True Zero LVC Cutoff (`TrueZeroCutoff`):** Linear taper down to exact $0.0\text{ A}$ at $V_{\text{LVC}}$ across the $3.0\text{ V}$ reserve cushion, replacing the stock 25% floor to eliminate sudden BMS disconnects under battery sag (GitHub Issue #3).
   - Source-level tuning profiles: gear-safe progressive, commuter comfort, uncapped cadence (metal gear only), progressive quadratic torque response, and unified level assist ceilings.
   - **Hardware Parity & Low-Level Drivers:**
     - **256-Sample Shunt Auto-Zero (`0x08009A6C`):** Synchronously averages 256 injected ADC conversions during startup while inverter outputs remain in High-Z (0A), calibrating stationary phase bias and programming `ADC1->jofr1` and `ADC2->jofr1`.
     - **Flash Page 127 Persistence Driver:** Unlocks, erases, and programs Page 127 (`0x0801FC00`), allowing runtime settings and display parameters to survive power cycles with verified 5-block XOR `0xA5` encryption and CRC-16 integrity.
     - **Hardware Overcurrent Break Protection:** Configures `TIM1_BDTR.BKE` break input on PB12 to immediately disable all 6 MOSFET gates in $< 50\text{ ns}$ upon comparator trip.
     - **Hardware Watchdog (`IWDG`):** 500 ms safety trip window refreshed deterministically inside the 1 ms SysTick control loop.
   - 5-block Flash EEPROM manager (`0x0801FC00`) with XOR cipher, magic headers, and nibble CRC-16, embedding bit-exact factory images for both `TS1000W` and `TS500W`.

5. **Physical Simulation & Verification Suite:**
   - 57 automated unit, integration, and end-to-end physical simulation tests running in CI (48 unit tests, 6 physical simulation scenarios, 3 stock identity roundtrip tests).
   - 6 physical scenario simulations: steep hill takeoff, speed limit soft taper, continuous linear thermal rollback, battery sag / reserve cushion, SVPWM dead-time clamp verification, and hardware safety watchdog latency.

6. **Native Serial Transport & Cross-Platform Parity:**
   - **Cross-Platform Driver Architecture:** Unified native serial engine (`PosixSerialPort` / `NativeSerialPort`) operating seamlessly on macOS, Linux, and Windows.
   - **Windows Serial Engine:** Multi-tier COM port enumeration querying Windows Registry (`HKLM\HARDWARE\DEVICEMAP\SERIALCOMM`), `QueryDosDeviceW` MS-DOS device mapping, and direct `COM1..COM64` probing. Complete Win32 `DCB` configuration, non-blocking `COMMTIMEOUTS` (immediate return), modem line control (`SETDTR`/`SETRTS`), buffer purges (`PurgeComm`), and universal `\\.\COMx` support for ports `COM1` through `COM255`.
   - **macOS & Linux Engine:** Direct POSIX `termios` raw 8N1 driver with automatic `/dev/tty.*` to `/dev/cu.*` callout node conversion, eliminating fragile external `stty` process spawning.
   - Dedicated `IOSSIOSPEED` custom-baud IOKit `ioctl` support on macOS for accurate line rates across all USB-serial converter chipsets (CH340, FTDI, CP2102).
   - Modem line state assertion (`TIOCMBIS` with `DTR` and `RTS`), satisfying the MegaHunt USB CDC-ACM display bootloader connection requirements and providing MCU reset pulse support.
   - Hardware and kernel FIFO queue flushing (`tcflush`) before baud rate transitions and probe sequences to purge leftover telemetry frames, ensuring 100% handshake parity with official Windows tools.
   - Robust STX (`0x02`) stream synchronization in MegaHunt ISP packet parser, gracefully discarding leading line noise.

7. **Safety & Bench Bring-up Runbook:**
   - Comprehensive bench bring-up guide ([`Documentation/Bench_Testing_Bringup_Guide.md`](Documentation/Bench_Testing_Bringup_Guide.md)) detailing Julet 5-pin programming pinout, SWD recovery points, and 48V current-limited bench power supply procedures.

---

## Clean-Room Reimplementation & AI-Assisted Engineering

### Methodology & Architecture
This project is a **100% clean-room reimplementation** written in memory-safe, modern `#![no_std]` Rust. 

- **Decompilation-Informed Architecture:** The algorithmic structure, register offsets, communication framing, and EEPROM layouts were uncovered through rigorous binary decompilation and static analysis (ARM Cortex-M4F disassemblies) of stock factory releases (including `V6510C`).
- **No Proprietary Code Reuse:** No vendor C code, proprietary assembly routines, or closed-source libraries are included or copied. Instead, every mathematical equation, lookup table, filter, and state machine was documented in human-readable terms and rewritten from scratch in idiomatic Rust with compile-time type safety.
- **Asset Isolation:** All proprietary manufacturer assets (`.Pack` containers, `.hex` display dumps, APK tuning apps, and vendor manuals) are strictly isolated in a `.gitignore`d directory and cataloged solely by SHA-256 hashes in [`blobs/README.md`](blobs/README.md).

### AI Ethics & Engineering Disclosure
This codebase was developed with the assistance of advanced generative AI models, specifically **Google Gemini** (via Antigravity) and **Anthropic Claude**:
- **Technical Role:** AI models were utilized as augmentative technical pair-programmers and analytical assistants to assist in reverse-engineering obfuscated ARM machine code, cross-referencing peripheral register maps against microcontrollers, translating decompiled math into safe Rust algorithms, and developing comprehensive simulation suites.
- **Automated Verification & Determinism:** The codebase relies on extensive automated regression tests, bit-exact roundtrips, cryptographic digests, and deterministic physical simulations to validate algorithms and data structures. However, it has not undergone an exhaustive line-by-line manual human audit of all decompiled routines; community testing, peer review, and bench validation remain essential steps.
- **Ethical Purpose & Interoperability:** This work is conducted strictly for interoperability, research, safety improvements, and consumer right-to-repair. The project aims to eliminate artificial software lockouts, correct safety deficiencies, and provide riders with full control over the hardware they legitimately own.

---

## Community Inquiries & Technical Feedback

We actively track feedback and technical inquiries from the e-bike community. Below are key technical topics raised regarding the core algorithms:

### 1. Torque Sensor Dynamic Range & Auto-Zero Drift (GitHub Issue #1)
- **Observation:** In physical testing, riders cannot mechanically deflect the DM01 torque strain gauge to the nominal $3.3\text{ V}$ ceiling (even steep hill climbing without motor assist yields a maximum of $\approx 2.7\text{ V}$, with typical hard efforts reaching $1.7\text{ V}$ to $2.0\text{ V}$). Furthermore, the resting zero-point voltage drifts thermally between $0.6\text{ V}$ and $0.9\text{ V}$.
- **Stock Limitation:** The stock `V6510C` firmware normalizes rider effort against a static $3300\text{ mV}$ ceiling ($3300\text{ mV} - 700\text{ mV} = 2600\text{ mV}$ span), meaning riders rarely utilize more than $40\%\text{--}50\%$ of the configured assist range without artificial amplification.
- **DM+ Implementation:** In DM+, rider effort is calculated strictly from the dynamic span:
  $$\Delta V_{\text{rider}} = V_{\text{applied}} - V_{\text{zero}}$$
  We have implemented `TorqueSpanPolicy::AutoSpanTracking { nominal_span_mv: 2300 }` where $V_{\text{max}}$ automatically shifts with the tracked resting zero-point:
  $$V_{\text{max}} = V_{\text{zero}} + 2300\text{ mV}$$
  Rider effort is computed as $\text{Effort} = \frac{V_{\text{applied}} - V_{\text{zero}}}{2300\text{ mV}}$, ensuring that a given pedal strain always delivers identical assist regardless of thermal drift, and allowing riders to reach 100% assist within realistic mechanical pedal deflection. Available in firmware and exposed via the `Commuter Comfort (Auto-Span)` preset.

### 2. Low-Cadence Power vs. Nylon Primary Reduction Gear (GitHub Issue #2)
- **Observation:** The DM01 internal primary reduction relies on a composite blue nylon helical gear. Delivering maximum current ($30\text{ A}$ / $\approx 1500\text{ W}$) at very low cadence ($< 18\text{ RPM}$) introduces extreme mechanical shear stress on the gear teeth, creating a severe risk of tooth stripping under standing-start pedal stomps.
- **Stock Limitation:** Stock firmware penalizes low-cadence climbing with a rigid $\frac{\text{Cadence}}{60\text{ RPM}}$ multiplier, cutting power by 60% at 25 RPM. While this protects the plastic gear, it causes riders to stall on steep trail inclines.
- **DM+ Implementation:** Power delivery is decoupled into selectable, transparent tuning profiles:
  - `GearSafeKnee` (**Default / Recommended**): Implements a soft-knee curve providing protective low assist ($\le 30\%$) below $18\text{ RPM}$ to shield gear teeth from standing starts, then ramps linearly to 100% assist at $35\text{ RPM}$ with full torque across the entire active pedaling band ($35\text{--}110+\text{ RPM}$).
  - `Stock60Rpm`: Retains the conservative stock linear ramp for riders seeking factory throttle and PAS behavior.
  - `UncappedCadence` (**Metal Gear Only**): Immediate full torque assist down to $5\text{ RPM}$ for riders running aftermarket steel/PEEK primary reduction gears. Clearly marked with red safety warnings in the GUI Studio.

### 3. Battery LVC Cushion & True Zero-Current Cutoff (GitHub Issue #3)
- **Observation:** The stock firmware features a $3.0\text{ V}$ reserve cushion above Low-Voltage Cutoff (LVC), but tapers current down to a $25\%$ floor rather than fully cutting off motor power. If an overloaded battery sags further, this $25\%$ residual draw can trigger a harsh, sudden BMS trip.
- **DM+ Implementation:** DM+ implements `LvcCushionPolicy::TrueZeroCutoff` with `apply_true_zero_lvc_cushion_da()`:
  - Above $V_{\text{LVC}} + 3.0\text{ V}$: 100% configured current limit is available.
  - Across the $3.0\text{ V}$ cushion ($V_{\text{LVC}} \le V \le V_{\text{LVC}} + 3.0\text{ V}$): Current linearly tapers from $100\%$ down to **exactly $0.0\text{ A}$** at $V_{\text{LVC}}$.
  - At or below $V_{\text{LVC}}$: PWM output is completely suppressed ($0\text{ A}$), preventing battery sag below cell manufacturer limits and eliminating abrupt BMS shutdowns under load.

### 4. Drivetrain Kinematics, Sprocket Sizes & Universal Sensorless Speed Fallback
- **Observation:** In the display settings menu under `Device`, options exist for `CranksetTeeth` (default 46T DM01 / 42T DM02), `CassetMaxTeeth` (default 32T), and `CassetMinTeeth` (default 11T). In stock firmware, altering these settings appears to have no effect during standard riding, prompting community questions regarding whether the controller is gear-aware or could implement dynamic shifting assistance.
- **Decompiled Findings & Kinematic Limitations:**
  - Decompilation of the `V6510C` Cortex-M4F binary (`0x0800625E`) revealed that the motor controller reads these three EEPROM parameters from Block 4 (`0x1D0..0x1D2`) to precompute a static midpoint gear ratio:
    $$\text{ratio}_{\text{mid}} = \frac{1}{2}\left(\frac{\text{Crankset}}{\text{CassetMin}} + \frac{\text{Crankset}}{\text{CassetMax}}\right)$$
    In stock firmware, this value was utilized strictly in `PedalSpeed` cadence modes.
  - Real-time dynamic gear ratio estimation ($\frac{\omega_{\text{wheel}}}{\omega_{\text{crank}}}$) is mechanically constrained on mid-drive bicycles:
    1. **Pulse Latency:** A standard 1-magnet spoke sensor provides only 1 pulse per wheel revolution. At low climbing speeds ($6\text{ km/h}$), each pulse arrives $\approx 1.4\text{ seconds}$ apart. By the time a ratio change is mathematically registered, the shift is already complete.
    2. **Dual Freewheel Decoupling:** Both the bottom bracket sprag clutch and the rear wheel cassette freehub decouple crank speed from wheel speed during coasting, ratcheting, or light pedaling.
    3. **Drivetrain Disturbances:** Chain slap, clutch derailleur spring bounce, cassette tooth step-changes, and full-suspension pedal kickback cause temporary ratio spikes. Triggering power cuts or shift detection on transient spikes would stall riders during critical climbs.
- **DM+ Implementation:** Rather than attempting fragile dynamic shift-guessing, DM+ utilizes these EEPROM sprocket parameters for **Universal Sensorless Speed Fallback**:
  - When the physical wheel speed sensor is operational, its reading governs speed limits and telemetry directly.
  - If the wheel speed sensor cable is severed, unplugged, or the spoke magnet is knocked out of alignment ($v_{\text{wheel}} = 0$), DM+ automatically falls back to:
    $$v_{\text{est}} = \text{Cadence} \times \text{ratio}_{\text{mid}} \times C_{\text{wheel}}$$
  - Operates across **all riding modes** (Torque, PAS, and Throttle), ensuring that handlebar display speedometers remain active and vehicle speed soft-taper governors continue protecting riders and legal compliance even during sensor hardware failure.
  - **SpeedSensorFail Bypass Support:** When riders intentionally choose `SpeedSensorFail = Speed Unlimit` (`0x1D7 == 0`, factory default), the open firmware respects this option and bypasses the speed governor cutoff when the physical wheel sensor reads 0 or is disconnected, while still reporting estimated kinematic speed to the display. If set to `Speed Limit` (`0x1D7 == 1`), the governor strictly enforces assist level speed cutoffs using estimated speed.

### 5. Decompiled Forensic Gap Analysis (`V6510C` vs. `dm01-firmware`)
- A comprehensive forensic gap analysis is available in the artifact documentation:
  - Vector table interrupt mapping across all active ISRs (ADC JEOC IRQ 18, TIM1 Break IRQ 24, TIM1 UP IRQ 25, TIM2 Hall IRQ 28, TIM4 Speed IRQ 30, USART3 IRQ 39).
  - Identification of STM32 hardware Injected Offset Registers (`ADC1->JOFR1` / `ADC2->JOFR1`) utilized by vendor firmware for zero-overhead phase shunt subtraction.
  - Decompilation of the 5-stage FOC rotor state machine (States 0, 2, 4, 6, 8) and inverter reset routines (`0x08006000`).
  - Wire protocol byte multiplexing between regular throttle (`0x01`) and active `FaultFlags` (`0x11`) driving all 12 display error messages.

---

## Detailed Documentation & Reference Guides

<details>
<summary><strong>Comparison: Stock Factory Firmware (V6510C) vs. DM+ Open Architecture</strong></summary>

<br>

| Feature / Subsystem | Stock Factory Firmware (`V6510C`) | DM+ Open Firmware (`v0.3.0`) | Benefit to Rider / Builder |
| :--- | :--- | :--- | :--- |
| **Low-Cadence Climbing Power** | Strict $\frac{\text{Cadence}}{60\text{ RPM}}$ scaling; assist is cut by **60%** at 25 RPM (only 2.5 A / 125 W delivered) | **Selectable Profiles:** Gear-Safe Progressive (default, protects nylon gear under 18 RPM, full assist at 35 RPM) or Uncapped Cadence (metal gear only) | Up to 2.5× more power on steep hill starts without gear damage |
| **Torque Assist Ceilings** | Dual-Ceiling Split: Torque sensor **ignores** assist level `Pwr%` and uses only Mode Max | **Unified Level Ceiling Policy:** Optional profile enforcing that torque assist respects assist level `Pwr%` | True fine-grained battery conservation and level-separated power control |
| **Pedal Effort Response** | Rigid linear effort multiplier | **Progressive Quadratic & High-Sensitivity Curves:** Gentle feathering at low pedal effort, exponential boost on steep climbs | Natural, intuitive MTB trail feel; less leg strain for urban commuters |
| **Takeoff & Standstill Decay** | Susceptible to startup shudder from cold Hall angle estimation | **Direct Physical Hall Sync + 100ms Decay:** Synchronizes instantly to physical Hall pins; decays to zero velocity when stationary | Smooth, instantaneous takeoff without jerky shudder or runaway assist |
| **Hardware Safety Watchdog** | Inactive or unmonitored during boot hangs | **Independent Hardware Watchdog (IWDG):** Active 500 ms window timer clocked by 40 kHz LSI oscillator, serviced in 1 ms SysTick | Failsafe reboot into safe High-Z (`MOE=0`) if firmware ever hangs or crashes |
| **MOSFET Shoot-Through Protection** | Proprietary timing table | **Guaranteed Dead-Time Clamp:** SVPWM duty cycles strictly clamped to $[6.0\%, 94.0\%]$ ($1.5\,\mu\text{s}$ margin) | Eliminates high-side/low-side FET shoot-through risk under heavy transients |
| **Emergency Cutoff Latency** | Software-filtered delay | **Instantaneous 0 ms High-Z Shutdown:** Brake lever (`PB12`), backpedaling, Hall fault (`0b000`/`0b111`), or gear shift sensor | Immediate motor disengagement for maximum rider safety |
| **Thermal Protection** | Sudden cliff or delayed step reduction | **Smooth Continuous Linear Rollback:** Continuous linear derating from 70°C down to 0.0 A at 100°C | No jarring sudden loss of power; keeps motor running smoothly while cooling |
| **Battery LVC Cushion** | Fixed 3.0V cushion (39V..42V) tapering down to 25% current floor | **True Zero Cutoff (TrueZeroCutoff):** Linear taper down to exact 0.0 A at LVC; completely suppresses PWM below LVC | Eliminates abrupt battery BMS shutdowns under heavy hill climbing or acceleration |
| **Multi-Voltage Adaptability** | Hardcoded 50.0V constant-power cap; poor 36V / 52V optimization | **Dynamic Voltage Anchoring & Auto-Detection:** Scales power ceiling relative to nominal pack voltage (36V, 48V, 52V) | Safe operation across all common e-bike battery voltages without bricking |
| **Display Compatibility** | Post-October 2024 menu lockouts lock users out of settings | **Automated Clean-Room Patcher (`toseven_pack display patch`):** NOPs out all 6 restriction gates on `T24V`, `T154`, `LCD135C`, `M15`, with live USB flashing or direct `.hex` export | Regain full access to `Speed&Power`, current limits, and wheel circumference |
| **Verification & Flashing Tool** | Windows-only closed-source executables (`IAP Tool 5.2`, `MH2020C_ISP`) | **Cross-Platform Open CLI (`toseven_pack`) & Desktop GUI:** Flash motor `.Pack` & display `.hex`, auto-detect ports, dry-run simulate, export patched hex | Open, scriptable, cross-platform (macOS/Linux/Windows) flashing and testing |

> [!TIP]
> For step-by-step flashing procedures, recovery of corrupted controllers, hardware pinouts, and display ISP unlocking instructions, consult the [`Documentation/Firmware_Flashing_and_Recovery_Guide.md`](Documentation/Firmware_Flashing_and_Recovery_Guide.md).

</details>

<details>
<summary><strong>DM+ Firmware Studio Desktop GUI & macOS Gatekeeper Guide</strong></summary>

<br>

**DM+ Firmware Studio** (`dm-plus-gui`) is a native desktop application built with [Slint](https://slint.dev/) and backed directly by the `dm01-firmware` crate.

```bash
# Launch the desktop studio from source
cargo run --manifest-path dm-plus-gui/Cargo.toml

# Package local release archives (CLI + GUI) into dist/
./scripts/package-release.sh
```

### Opening on macOS (Gatekeeper & Privacy Restrictions)

Because release binaries downloaded from GitHub are not signed with an Apple Developer Program certificate, macOS Gatekeeper may present a security prompt stating *"DM+ Firmware Studio.app cannot be opened because Apple cannot check it for malicious software"*.

To open the application on macOS:
1. **Finder Context Menu:** Right-click (or Control-click) `DM+ Firmware Studio.app` in Finder, select **Open**, and then click **Open** in the confirmation dialog.
2. **System Settings:** Open **System Settings > Privacy & Security**, scroll down to the **Security** section, and click **Open Anyway**.
3. **Terminal (Strip Quarantine):** Remove the quarantine attribute directly:
   ```bash
   xattr -cr "/path/to/DM+ Firmware Studio.app"
   ```
   For the standalone CLI binary:
   ```bash
   xattr -c toseven_pack && chmod +x toseven_pack
   ```

</details>

<details>
<summary><strong>CLI Tooling Reference (<code>toseven_pack</code>)</strong></summary>

<br>

```bash
# Inspect container metadata, vector table, and factory EEPROM
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- info <firmware.Pack>

# Decrypt firmware binary payload (57,708 bytes loaded at 0x08001800)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- unpack <firmware.Pack> payload.bin

# Build the standalone bare-metal Cortex-M4F ELF binary
cargo build --target thumbv7em-none-eabihf --manifest-path dm01-firmware/Cargo.toml --no-default-features --features embedded --bin dm01_firmware --release

# Extract raw Flash image from compiled ELF
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- objcopy dm01-firmware/target/thumbv7em-none-eabihf/release/dm01_firmware build/dm01_firmware.bin

# Package directly from ELF into official ToSeven .Pack firmware container
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- create dm01 dm01-firmware/target/thumbv7em-none-eabihf/release/dm01_firmware build/TO7-DM01-OpenFirmware-v0.3.0.Pack

# Cryptographically verify container integrity and bit-exact round-trip
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- verify <firmware.Pack>

# Simulate motor controller IAP packet framing and sector pacing (dry run)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash <firmware.Pack> --dry-run

# Flash motor controller firmware over USB-UART adapter to hardware
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash <port> <firmware.Pack>

# Inspect display firmware (identifies model, vector table, and lockout status)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display info <firmware.hex>

# Permanently unlock display firmware (removes post-Oct 2024 Advanced Menu lockouts)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display patch <input.hex> <output.hex>

# Flash display firmware with on-the-fly unlock (dry-run simulation)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display flash <firmware.hex> --unlock --dry-run

# Flash display firmware over USB CDC port to hardware with on-the-fly unlock
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display flash <port> <firmware.hex> --unlock

# Monitor live 0x69-encrypted UART telemetry (or run simulated ride demo)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- monitor --simulate
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- monitor <port> --baud 9600

# Run comparative 6-scenario ride simulation suite (Stock V6510C vs. Open Firmware)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- simulate

# List connected USB serial adapters and displays
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- ports
```

</details>

<details>
<summary><strong>Hardware Specifications, Pinouts & Bring-up Runbook</strong></summary>

<br>

### Extracted Hardware Architecture ([`Documentation/DM01_Hardware_Specification.md`](Documentation/DM01_Hardware_Specification.md))
- **Processor:** ARM Cortex-M4F with hardware single-precision FPU enabled (`SCB->CPACR`).
- **PWM Timer (`TIM1` at `0x40012C00`):** Center-aligned 16 kHz complementary 6-FET PWM (`ARR = 5999`, `DTG = 100` ≈ 1.38 µs dead-time). High-side: `PA8, PA9, PA10`. Low-side: `PB13, PB14, PB15`. Emergency Break input: `PB12`.
- **Analog Subsystem (`ADC1/ADC2`):** Dual injected simultaneous sequence for Shunt U (`PA0`), Shunt V (`PA1`), and derived Shunt W. Regular scanned channels: Bus voltage (`PB0`), Strain gauge torque sensor (`PA4`), Throttle (`PA7`), Motor thermistor (`PB1`).
- **UART (`USART3` at `0x40004800`):** Display bus running at 9600 Baud (8-N-1), TX on `PB10`, RX on `PB11`, with symmetric wire XOR `0x69` stream cipher.

### Safety Verification Before Physical On-Bike Testing
- **Bench Power Supply Limits:** Before connecting to a live high-current battery pack, inverter gate drivers must be tested on a bench power supply strictly current-clamped to $\le 1.0\text{ A}$ at $48\text{ V}$.
- **Complete Runbook:** Follow the step-by-step bring-up guide in [`Documentation/Bench_Testing_Bringup_Guide.md`](Documentation/Bench_Testing_Bringup_Guide.md) covering harness pinouts, SWD debug connections, and phased bring-up checks.

</details>

<details>
<summary><strong>Repository Directory Layout</strong></summary>

<br>

```text
├── dm01-firmware/           # Core #![no_std] Rust firmware crate & CLI
│   ├── src/
│   │   ├── power_curve.rs   # Exact V6510C integer power curve pipeline
│   │   ├── tuning.rs        # Source-level tuning profiles (uncapped cadence, progressive torque)
│   │   ├── foc.rs           # Clarke/Park, SVPWM, and PLL Hall observer
│   │   ├── eeprom.rs        # 5-block Flash EEPROM storage & CRC-16
│   │   ├── flasher.rs       # IAP wakeup, 12-byte header, chunk pacing & display ISP
│   │   ├── display_patcher.rs # Intel HEX parser & NOP lockout bypass
│   │   ├── pack.rs          # .Pack container parser, cipher, and repacker
│   │   ├── pac.rs           # Memory-mapped peripheral register bindings (TIM1, ADC, USART, GPIO)
│   │   ├── hal.rs           # Concrete DM01 hardware drivers (Dm01PwmHardware, Dm01SensorsHardware)
│   │   ├── protocol.rs      # Display UART 0x69 cipher protocol
│   │   ├── stock.rs         # Known official release registry (15 factory releases)
│   │   ├── sha256.rs        # Embedded no_std SHA-256 digest
│   │   ├── bin/
│   │   │   ├── firmware.rs  # Standalone bare-metal Cortex-M4F firmware entry point & vector table
│   │   │   └── toseven_pack.rs # CLI packaging, unpacking, verify, and flashing tool
│   │   └── memory.ld        # Flash (0x08001800) and SRAM (0x20000000) linker script
│   └── tests/
│       ├── simulation_suite.rs # 6 end-to-end comparative physical ride scenarios
│       └── stock_identity.rs   # Bit-exact stock roundtrip & tuning tests
├── dm-plus-gui/             # DM+ Firmware Studio desktop GUI (Slint)
│   ├── assets/              # Embedded standalone open firmware .Pack
│   ├── ui/
│   │   └── appwindow.slint  # Dark GTK4/Libadwaita desktop interface
│   └── src/
│       └── main.rs          # Asynchronous flashing threads, RFD file pickers, and serial integration
├── scripts/
│   └── package-release.sh   # Local release packaging script for current host OS
├── .github/
│   └── workflows/
│       ├── ci.yml           # Continuous integration (linter, unit tests, no_std, simulations)
│       └── release.yml      # Cross-platform release matrix (Linux, macOS Universal, Windows)
├── Documentation/
│   ├── Firmware_Flashing_and_Recovery_Guide.md # Step-by-step flashing & unbricking recovery runbook
│   ├── Bench_Testing_Bringup_Guide.md # Hardware bring-up runbook, Julet pinouts, bench setup
│   ├── ToSeven_IAP_Flashing_Protocol.md # Clean-room IAP UART and display ISP flashing spec
│   ├── ToSeven_Display_Firmware_Architecture.md # Display architecture, throttle modes, lockout bypass
│   ├── DM01_Hardware_Specification.md # Complete reverse-engineered register map and pinout
│   ├── DM01_Power_Curve_Guide.md # Comprehensive guide explaining power delivery
│   ├── Official_Specifications.md # Hardware limits, pinouts, and manual references
│   └── Research_Downgrade_and_Update.md # Container encryption & IAP bootloader notes
├── blobs/                   # External proprietary assets (.gitignore'd for clean-room legal safety)
│   └── README.md            # Verified SHA-256 asset catalog (25 verified files)
├── AGENTS.md                # Technical project reference & reverse engineering notes
└── README.md                # Project overview and roadmap (this file)
```

</details>

<details>
<summary><strong>Project Vision & Future Roadmap (Unified Mega-Firmware)</strong></summary>

<br>

Most e-bike firmware modifications force riders into a frustrating loop: disconnect the bike, connect a programming cable, run a proprietary flashing tool, adjust a setting, re-flash, test ride, and repeat.

Our long-term architectural goal is to build an open, unified motor and display ecosystem:
1. **Flash Once:** Flash the open motor controller firmware onto the motor once.
2. **Handlebar Control:** Configure motor behaviors directly through the on-bike display menu (`LCD135C`, `T24V`, `T154`, `M15`) or smartphone via Bluetooth.
3. **Dynamic Tuning via UART:** Settings are communicated over the ToSeven UART protocol and saved directly into Flash EEPROM (`0x0801FC00`).
4. **On-the-Fly Profile Switching:** Toggle between smooth commuting, progressive trail riding, or uncapped climbing power at the push of a handlebar button.

```
[Phase 1: Algorithmic Core]       [Done] (FOC, Power Curves, EEPROM, Pack Tooling)
            │
[Phase 2.1: Hardware Binding]     [Done] (Volatile PAC, Safe High-Z TIM1 PWM, Hall+PLL, memory.ld)
            │
[Phase 2.2: Standalone .Pack]     [Done] (Native ELF extraction, 128 KB .Pack, Live Serial & Monitor)
            │
[Phase 3: Bench Verification]     [In Progress] (ADC DMA scan sequencing + 48V current-limited bench test)
            │
[Phase 4: Public v1.0 Release]    [Planned] (Initial DM01 Open Firmware Release)
```

</details>

---

## Contributing & Community

Contributions are welcome from developers, e-bike builders, and riders! Key areas where help is needed:

1. **Hardware Bench Testing (Phase 3):**
   - Oscilloscope validation of gate driver dead-time and complementary PWM switching on bench power supplies ($\le 1.0\text{ A}$ current limit).
   - Validation of multi-channel ADC DMA sequencing (`Shunt U/V`, `Torque`, `Throttle`, `V_bat`, `Temp`).
2. **Display Firmware Verification:**
   - Testing unlocked `.hex` firmware on different display hardware revisions (`T24V`, `T154`, `M15`, `LCD135C`).
   - Extracting and documenting additional display models and Bluetooth module firmwares.
3. **DM02 Motor Controller Validation:**
   - Testing and confirming `.Pack` container repacking and EEPROM block formatting for DM02 (`TS500W`).
4. **Code Quality Standards:**
   - Retain strict `#![no_std]` embedded compatibility for core controller modules in `dm01-firmware`.
   - Ensure all code passes `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
   - Include unit tests or simulation scenarios covering any modified power curve or state machine logic.

---

## Clean-Room & Legal Notice

This project is a clean-room reverse-engineering, documentation, and open-source re-implementation effort created strictly for interoperability, research, safety, and educational purposes.

- No proprietary binaries, copyrighted vendor manuals, or closed-source code are distributed in this repository.
- All proprietary `.Pack` files, `.hex` display dumps, and vendor APKs remain the property of their respective owners and are isolated in `.gitignore`d storage as detailed in [`blobs/README.md`](blobs/README.md).
- **Assumption of Risk & Equipment Damage:** Flashing custom, open-source, or modified firmware to any motor controller or display carries inherent electrical and mechanical risks. **Any damage to your equipment—including motor controllers, displays, batteries, wiring, gears, or bicycles—is entirely your own responsibility.**
- **No Warranty:** This project is provided "AS IS", without warranty of any kind, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose, and non-infringement. In no event shall the authors or contributors be liable for any claim, damages, or other liability. See [LICENSE](LICENSE) for complete terms.
