# DM+ : Open Mid-Drive Firmware & Tooling Studio

[![CI](https://github.com/antaxiom/dm0x-plus-firmware/actions/workflows/ci.yml/badge.svg)](https://github.com/antaxiom/dm0x-plus-firmware/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B%20%7C%20%23!%5Bno__std%5D-orange.svg)](dm01-firmware/Cargo.toml)
[![Target](https://img.shields.io/badge/target-ARM%20Cortex--M4F-blue.svg)](https://developer.arm.com/Processors/Cortex-M4)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

An open-source `#![no_std]` Embedded Rust firmware recompilation, display unlocking toolchain, and cross-platform desktop studio for **ToSeven (TO7)** mid-drive e-bike motors (DM01, DM02) and handlebar displays (T24V, T154, M15, LCD135C).

> [!NOTE]
> **DM+** is the working project name for this unified firmware and tooling ecosystem. Naming may evolve as the project expands to additional motor controllers and displays.

> [!WARNING]
> ### Disclaimer of Warranty & Equipment Damage Liability
> **THIS SOFTWARE AND FIRMWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED.**
> Flashing, configuring, or operating experimental, custom, or reverse-engineered firmware on electric bicycles, motors, controllers, and displays carries inherent electrical, mechanical, and safety risks. **Any damage to your equipment—including blown MOSFETs, thermal degradation, stripped primary reduction gears, damaged battery packs, blown BMS units, harness damage, electrical shorts, or permanently bricked motor controllers and displays—is solely and entirely your own responsibility.**
> No warranty is provided, whether express, implied, or statutory, including warranties of merchantability, fitness for a particular purpose, and non-infringement. Proceed strictly at your own risk.

> [!CAUTION]
> ### CRITICAL SAFETY WARNING: BENCH TESTING ONLY AT THIS STAGE
> **DO NOT INSTALL OPEN-SOURCE FIRMWARE ONTO AN ASSEMBLED ELECTRIC BICYCLE OR ATTEMPT TO RIDE WITH IT.**
> At this stage, the DM+ open-source motor controller firmware (`dm01_firmware` / `dm02_firmware`) is in active beta bring-up and **MUST NOT BE INSTALLED ANYWHERE OTHER THAN A CONTROLLED BENCH TEST ENVIRONMENT.**
> - **Bench Testing Exclusively:** Only operate controllers on an adjustable DC bench power supply strictly current-limited to <= 1.0 A at 48 V.
> - **Never Use a Lithium Battery for Initial Testing:** Do not test open-source firmware using a high-discharge lithium-ion battery pack.
> - **Oscilloscope & Phase Verification First:** Phase commutation, ADC calibration, shunt offset subtraction, and FOC loop timings must be verified on the bench before any mechanical load is applied.
> - **Flashing Factory Stock is Safe:** Flashing official stock `.Pack` files or patching display firmware (`.hex`) for menu unlock is fully supported. However, running custom open motor firmware on an actual bicycle is strictly prohibited at this stage.

---

## What Is Implemented Right Now (`v0.3.0-beta.7` - Release Beta)

Here is the verified functionality available in this repository:

1. **DM+ Firmware Studio Desktop GUI (`dm-plus-gui`):**
   - Native cross-platform desktop application built with Slint and Rust, running on Linux, macOS, and Windows.
   - Built-in embedded **DM01** and **DM02** Open Firmware containers (128 KB each) for single-click flashing without external files.
   - Automatic target hardware detection when a custom `.Pack` file is loaded, locking the hardware selector to prevent cross-flashing between DM01 (external 12-FET) and DM02 (internal 6-FET).
   - Parameter isolation: automatically grays out source-level tuning and battery limit controls when flashing pre-compiled factory `.Pack` files.
   - Live serial flashing with automatic port discovery, plug/unplug detection, and offline patched `.hex` export.
   - Apple HIG-compliant assets across all platforms and proper native window/dock integration.
   - Pre-built binary packages generated for Linux x86_64, Windows x64, and macOS Universal (`.app`).

2. **Motor Controller IAP Flasher & Container Engine (`toseven_pack` & GUI):**
   - **Dual-Tier Encryption & Wire Stream Cipher:**
     - **Tier 1 (Container Encryption):** Verified auto-key derivation across all 15 official stock releases:
       $$K_{\text{head}} = (\text{pack}[32] + \text{pack}[33]) \pmod{256}$$
       $$K_{\text{payload}} = (\text{header}[30] + \text{header}[31]) \pmod{256}$$
       Header CRC-16 at offset 30, payload 32-bit checksum at offset 26.
     - **Tier 2 (Physical Wire Stream Cipher):** Byte-wise XOR masking during flash streaming:
       $$B_{\text{wire}} = B_{\text{plain}} \oplus \text{UartMask}$$
       Matching bootloader de-masking into Flash memory (`0x08001800`).
   - **Full Handshake Protocol Engine:**
     - **Step 1:** Sends 21-byte wakeup burst (`0x69` XOR stream cipher) at 9600 baud with high/low nibble `UartMask` distribution and DTR reset pulse fallback.
     - **Step 2:** Bootloader probe at 38400 baud (vendor factory default) with automatic 115200 baud fallback (`0x02` -> `0x02, 0xFD` or `0x06`).
     - **Step 3:** 12-byte header handshake (program size, 32-bit checksum, `UartMask`, baud index).
     - **Step 4:** Streams payload in 16-byte chunks with pacing delay and 1024-byte (1 KB) sector commit acknowledgments (`0x06` ACK).
     - **Step 5:** Bootloader cumulative 32-bit byte sum verification against Flash contents (`0x04` EOT on match; `0x15` NAK on failure).
   - Automatic hardware target detection (`DM01` with `0xA8`/`0x67` vs. `DM02` with `0xF7`/`0xE2`).
   - Bit-exact unpacking, verification, and repacking matching official vendor tools bit-for-bit.
   - Natively extracts loadable segments directly from Cortex-M ELF binaries into official 128 KB `.Pack` containers without requiring external `objcopy`.

3. **Display Lockout Patcher & ISP Flasher (`toseven_pack display`):**
   - Automatically parses Intel HEX display firmware dumps (`T24V`, `T154`, `LCD135C`, `M15`).
   - Detects and neutralizes all 6 restriction branch gates introduced in post-October 2024 factory firmware using ARM Thumb NOP (`0x00BF`) patches.
   - Restores permanent access to `Speed&Power`, maximum current limits, and wheel circumference menus.
   - Flashes patched display firmware directly over USB CDC using the MegaHunt ISP bootloader protocol.

4. **Clean-Room Motor Controller Architecture (`dm01-firmware`):**
   - 100% `#![no_std]` embedded Rust implementation of Field Oriented Control (FOC), Clarke/Park transforms, SVPWM duty generation with dead-time clamps (6.0% to 94.0%), and Hall observer.
   - Unified multi-target support for both **DM01** (external 12-FET, 30A Sport, 40.27:1 reduction) and **DM02** (internal 6-FET, 18A Sport, 53.00:1 reduction), with hard safety ceilings (32.0 A max for DM01, 20.0 A max for DM02) and active UI/firmware warnings whenever exceeding stock factory ratings (> 30 A on DM01, > 18 A on DM02).
   - Dedicated bare-metal Cortex-M4F binary targets for both motors: `dm01_firmware` (MSP `0x20005628`) and `dm02_firmware` (MSP `0x20005620`).
   - **Automatic Dynamic Span Tracking (`AutoSpanTracking`):** Dynamic scaling pinning nominal deflection to 2.3 V, adapting to thermal zero-point drift and matching realistic human pedal effort.
   - **Gear-Safe Progressive Profile (`GearSafeKnee`):** Soft current (<= 30%) below 18 RPM to protect the blue nylon primary reduction gear against standing-start stomps, ramping to 100% assist at 35 RPM with progressive quadratic modulation.
   - **True Zero LVC Cutoff (`TrueZeroCutoff`):** Linear taper down to exact 0.0 A at cutoff voltage across the 3.0 V reserve window, eliminating abrupt BMS disconnects under battery sag.
   - **Universal Sensorless Speed Fallback:** Computes static midpoint gear ratio from EEPROM Block 4. When the physical wheel speed sensor disconnects or loses its magnet, dynamically estimates vehicle speed from cadence to maintain speedometer telemetry and speed governing.
   - **Hardware Parity & Low-Level Drivers:**
     - **256-Sample Shunt Auto-Zero:** Synchronously averages 256 injected ADC conversions during startup while inverter outputs remain in High-Z (0A), calibrating stationary phase bias into injected offset registers.
     - **Flash Page 127 Persistence Driver:** Unlocks, erases, and programs Page 127 (`0x0801FC00`), allowing runtime settings and display parameters to survive power cycles with verified 5-block XOR `0xA5` encryption and CRC-16 integrity.
     - **Hardware Overcurrent Break Protection:** Configures `TIM1_BDTR.BKE` break input on PB12 to immediately disable all 6 MOSFET gates in < 50 ns upon comparator trip.
     - **Hardware Watchdog (`IWDG`):** 500 ms safety trip window refreshed deterministically inside the 1 ms SysTick control loop.
   - 5-block Flash EEPROM manager (`0x0801FC00`) with XOR cipher, magic headers, and nibble CRC-16, embedding bit-exact factory images for both `TS1000W` and `TS500W`.

5. **Physical Simulation & Verification Suite:**
   - 67 automated unit, integration, and end-to-end physical simulation tests running in CI (58 unit tests, 6 physical simulation scenarios, 3 stock identity roundtrip tests).
   - 6 physical scenario simulations: steep hill takeoff, speed limit soft taper, continuous linear thermal rollback, battery sag / reserve cushion, SVPWM dead-time clamp verification, and hardware safety watchdog latency.

6. **Native Serial Transport & Cross-Platform Parity:**
   - Unified native serial engine operating on macOS, Linux, and Windows without external script dependencies.
   - Complete modem line control (DTR and RTS) for bootloader reset and MegaHunt USB CDC-ACM display bootloader connection requirements.
   - Hardware and kernel FIFO queue flushing before baud rate transitions and probe sequences to purge leftover telemetry frames.

7. **Safety & Bench Bring-up Runbook:**
   - Comprehensive bench bring-up guide ([`Documentation/Bench_Testing_Bringup_Guide.md`](Documentation/Bench_Testing_Bringup_Guide.md)) detailing Julet 5-pin programming pinout, SWD recovery points, and 48V current-limited bench power supply procedures.

---

## Clean-Room Reimplementation & AI-Assisted Engineering

### Methodology & Architecture
This project is a **clean-room reimplementation** written in memory-safe, modern `#![no_std]` Rust:

- **Decompilation-Informed Architecture:** Register offsets, communication framing, state machines, and EEPROM layouts were documented through static analysis of stock factory firmware binaries (including `V6510C`).
- **No Proprietary Code Reuse:** No vendor C code, proprietary assembly routines, or closed-source libraries are included. Every algorithm, filter, and state machine was documented and rewritten from scratch in idiomatic Rust.
- **Asset Isolation:** All proprietary manufacturer assets (`.Pack` containers, `.hex` display dumps, APK tuning apps, and vendor manuals) are strictly isolated in a `.gitignore`d directory and cataloged solely by SHA-256 hashes in [`blobs/README.md`](blobs/README.md).

### AI Assistance & Engineering Disclosure
This codebase was developed with the assistance of advanced generative AI models (Google Gemini and Anthropic Claude):
- **Technical Role:** AI models were utilized as pair-programmers to assist in analyzing obfuscated ARM machine code, cross-referencing peripheral register maps, translating decompiled math into safe Rust algorithms, and developing automated simulation tests.
- **Verification & Determinism:** The codebase relies on automated regression tests, bit-exact roundtrips, cryptographic digests, and deterministic physical simulations. It has not undergone an exhaustive line-by-line manual human audit of all decompiled routines; community testing, peer review, and bench validation remain essential steps.
- **Purpose:** This work is conducted strictly for interoperability, research, safety improvements, and consumer right-to-repair.

---

## Community Inquiries & Technical Feedback

Below are technical analyses of community-raised topics regarding the motor controller algorithms:

### 1. Torque Sensor Dynamic Range & Zero-Point Drift
- **Observation:** In physical testing, riders cannot mechanically deflect the DM01 torque strain gauge to the nominal 3.3 V ceiling; typical hard pedal efforts reach 1.7 V to 2.0 V, with extreme climbing rarely exceeding 2.7 V. Furthermore, the resting zero-point voltage drifts thermally between 0.6 V and 0.9 V.
- **Stock Limitation:** Stock `V6510C` firmware normalizes rider effort against a static 3300 mV ceiling (a 2600 mV span above 700 mV zero), meaning riders rarely utilize more than 40% to 50% of the configured assist range without artificial amplification.
- **DM+ Implementation:** Rider effort is calculated from dynamic deflection:
  $$\Delta V_{\text{rider}} = V_{\text{applied}} - V_{\text{zero}}$$
  With `TorqueSpanPolicy::AutoSpanTracking { nominal_span_mv: 2300 }`, the effective ceiling shifts dynamically with the tracked zero-point:
  $$V_{\text{max}} = V_{\text{zero}} + 2300\text{ mV}$$
  $$\text{Effort} = \frac{V_{\text{applied}} - V_{\text{zero}}}{2300\text{ mV}}$$
  This ensures consistent assist sensitivity across thermal drift and allows riders to reach 100% assist within realistic mechanical pedal deflection.

### 2. Low-Cadence Power Delivery vs. Nylon Primary Gear Protection
- **Observation:** The DM01 internal primary reduction relies on a composite blue nylon helical gear. Delivering maximum current (30 A / ~1500 W) at very low cadence (< 18 RPM) introduces high mechanical shear stress on gear teeth during standing-start pedal stomps.
- **Stock Limitation:** Stock firmware penalizes low-cadence climbing with a linear cadence multiplier (`Cadence / 60 RPM`), cutting power by 60% at 25 RPM. While this protects the plastic gear, it causes riders to stall on steep inclines.
- **DM+ Implementation:** Power delivery is decoupled into selectable tuning profiles:
  - `GearSafeKnee` (**Default / Recommended**): Limits assist to <= 30% below 18 RPM to shield gear teeth from standing starts, then ramps to 100% assist at 35 RPM with full torque across the active pedaling band (35-110+ RPM).
  - `Stock60Rpm`: Retains the conservative stock linear ramp for factory throttle and PAS behavior.
  - `UncappedCadence` (**Metal Gear Only**): Immediate full torque assist down to 5 RPM for riders with aftermarket steel/PEEK primary reduction gears.

### 3. Battery Low-Voltage Cutoff (LVC) Cushion & 0A Protection
- **Observation:** Stock firmware features a 3.0 V reserve cushion above Low-Voltage Cutoff (LVC), but tapers current down to a 25% floor rather than cutting power completely. If an overloaded battery sags further, this residual draw can trigger an abrupt BMS shutdown.
- **DM+ Implementation:** `LvcCushionPolicy::TrueZeroCutoff` provides a smooth linear taper down to true zero:
  - Above $V_{\text{LVC}} + 3.0\text{ V}$: 100% configured current limit is available.
  - Across the 3.0 V cushion ($V_{\text{LVC}} \le V \le V_{\text{LVC}} + 3.0\text{ V}$): Current linearly tapers from 100% down to **exactly 0.0 A** at $V_{\text{LVC}}$.
  - At or below $V_{\text{LVC}}$: PWM output is completely suppressed (0 A), preventing battery sag below cell limits and eliminating abrupt BMS shutdowns under load.

### 4. Drivetrain Kinematics & Kinematic Speed Fallback
- **Observation:** Display settings menus provide `CranksetTeeth`, `CassetMaxTeeth`, and `CassetMinTeeth`. In stock firmware, altering these settings has no effect during standard torque-sensing riding.
- **Decompiled Findings:** Stock `V6510C` firmware reads these three EEPROM parameters from Block 4 (`0x1D0..0x1D2`) to precompute a static midpoint gear ratio:
  $$\text{ratio}_{\text{mid}} = \frac{1}{2}\left(\frac{\text{Crankset}}{\text{CassetMin}} + \frac{\text{Crankset}}{\text{CassetMax}}\right)$$
  Real-time dynamic gear ratio estimation ($\omega_{\text{wheel}} / \omega_{\text{crank}}$) is constrained on mid-drives due to 1-pulse-per-revolution sensor latency (~1.4 s between pulses at 6 km/h) and freewheel decoupling during coasting.
- **DM+ Implementation:** DM+ uses these EEPROM sprocket parameters for **Universal Sensorless Speed Fallback**:
  - When the physical wheel speed sensor is operational, its reading governs speed limits and telemetry directly.
  - If the wheel speed sensor cable is damaged or the spoke magnet is lost ($v_{\text{wheel}} = 0$), DM+ automatically estimates speed:
    $$v_{\text{est}} = \text{Cadence} \times \text{ratio}_{\text{mid}} \times C_{\text{wheel}}$$
  - Operates across all riding modes (Torque, PAS, and Throttle), keeping display speedometers active and speed soft-taper governors functioning during sensor hardware failure.
  - **SpeedSensorFail Option:** When configured to `Speed Unlimit` (`0x1D7 == 0`, factory default), the speed governor cutoff is bypassed if the physical sensor reads 0, while still reporting estimated speed to the display. If set to `Speed Limit` (`0x1D7 == 1`), the governor strictly enforces assist level speed cutoffs using estimated speed.

### 5. Hardware Gap Analysis (Stock V6510C vs. DM+)
- Vector table interrupt mapping across all active ISRs (ADC JEOC IRQ 18, TIM1 Break IRQ 24, TIM1 UP IRQ 25, TIM2 Hall IRQ 28, TIM4 Speed IRQ 30, USART3 IRQ 39).
- Utilization of STM32 hardware Injected Offset Registers (`ADC1->JOFR1` / `ADC2->JOFR1`) for zero-overhead phase shunt subtraction.
- Implementation of the 5-stage FOC rotor state machine and inverter reset routines (`0x08006000`).
- Wire protocol byte multiplexing between regular throttle (`0x01`) and active `FaultFlags` (`0x11`) driving display error messages.

---

## Detailed Documentation & Reference Guides

<details>
<summary><strong>Comparison: Stock Factory Firmware (V6510C) vs. DM+ Open Architecture</strong></summary>

<br>

| Feature / Subsystem | Stock Factory Firmware (`V6510C`) | DM+ Open Firmware (`v0.3.0`) | Benefit to Rider / Builder |
| :--- | :--- | :--- | :--- |
| **Low-Cadence Climbing Power** | Strict linear scaling (`Cadence / 60 RPM`); assist cut by 60% at 25 RPM | **Selectable Profiles:** Gear-Safe Progressive (default, <= 30% under 18 RPM, full assist at 35 RPM) or Uncapped Cadence (metal gear only) | More power on steep hill starts without nylon gear damage |
| **Torque Assist Ceilings** | Dual-Ceiling Split: Torque sensor ignores assist level `Pwr%` and uses only Mode Max | **Unified Level Ceiling Policy:** Optional profile enforcing that torque assist respects assist level `Pwr%` | Predictable power scaling and battery conservation across assist levels |
| **Pedal Effort Response** | Rigid linear effort multiplier | **Progressive Quadratic & High-Sensitivity Curves:** Gentle feathering at low pedal effort, progressive boost on steep climbs | Natural trail feel; reduced leg fatigue for urban commuting |
| **Takeoff & Standstill Decay** | Susceptible to startup shudder from cold Hall angle estimation | **Direct Physical Hall Sync + 100ms Decay:** Synchronizes to physical Hall pins; decays to zero velocity when stationary | Smooth takeoff without shudder or runaway assist |
| **Hardware Safety Watchdog** | Inactive or unmonitored during boot hangs | **Independent Hardware Watchdog (IWDG):** Active 500 ms window timer clocked by 40 kHz LSI oscillator, serviced in 1 ms SysTick | Failsafe reboot into safe High-Z (`MOE=0`) if firmware hangs |
| **MOSFET Shoot-Through Protection** | Proprietary timing table | **Guaranteed Dead-Time Clamp:** SVPWM duty cycles strictly clamped to 6.0% to 94.0% (1.5 µs margin) | Eliminates high-side/low-side FET shoot-through risk under transients |
| **Emergency Cutoff Latency** | Software-filtered delay | **Instantaneous 0 ms High-Z Shutdown:** Brake lever (`PB12`), backpedaling, Hall fault (`0b000`/`0b111`), or gear shift sensor | Immediate motor disengagement for maximum rider safety |
| **Thermal Protection** | Sudden cliff or delayed step reduction | **Smooth Continuous Linear Rollback:** Continuous linear derating from 70°C down to 0.0 A at 100°C | Smooth power reduction while cooling without sudden cutouts |
| **Battery LVC Cushion** | Fixed 3.0V cushion tapering down to 25% current floor | **True Zero Cutoff (TrueZeroCutoff):** Linear taper down to exact 0.0 A at LVC; completely suppresses PWM below LVC | Eliminates abrupt battery BMS shutdowns under heavy hill climbing |
| **Multi-Voltage Adaptability** | Hardcoded 50.0V constant-power cap; fixed 48V tuning | **Dynamic Voltage Anchoring & Auto-Detection:** Scales power ceiling relative to nominal pack voltage (36V, 48V, 52V) | Safe operation across all common e-bike battery voltages |
| **Display Compatibility** | Post-October 2024 menu lockouts prevent modifying settings | **Automated Clean-Room Patcher (`toseven_pack display patch`):** NOPs out all 6 restriction gates on `T24V`, `T154`, `LCD135C`, `M15`, with live USB flashing or direct `.hex` export | Regain access to `Speed&Power`, current limits, and wheel circumference |
| **Verification & Flashing Tool** | Windows-only closed-source executables (`IAP Tool 5.2`, `MH2020C_ISP`) | **Cross-Platform Open CLI (`toseven_pack`) & Desktop GUI:** Flash motor `.Pack` & display `.hex`, auto-detect ports, dry-run simulate, export patched hex | Open, scriptable, cross-platform (macOS/Linux/Windows) flashing and testing |

> [!TIP]
> For step-by-step flashing procedures, recovery of corrupted controllers, hardware pinouts, and display ISP unlocking instructions, consult [`Documentation/Firmware_Flashing_and_Recovery_Guide.md`](Documentation/Firmware_Flashing_and_Recovery_Guide.md).

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
- **Bench Power Supply Limits:** Before connecting to a live high-current battery pack, inverter gate drivers must be tested on a bench power supply strictly current-clamped to <= 1.0 A at 48 V.
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
│   ├── ToSeven_Controller_Depotting_and_Diagnostics_Guide.md # Thermal depotting, polyurethane release & board triage
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
1. **Flash Once:** Flash open motor controller firmware onto the motor once.
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
   - Oscilloscope validation of gate driver dead-time and complementary PWM switching on bench power supplies (<= 1.0 A current limit).
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
