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
> ### CRITICAL SAFETY WARNING: BENCH BRING-UP ONLY & BOOTLOADER RISK
> **DO NOT INSTALL EXPERIMENTAL OPEN-SOURCE MOTOR FIRMWARE UNLESS PREPARED FOR HARDWARE-LEVEL RECOVERY.**
>
> 1. **Potted Controller Vulnerability (Software Bootloader Lockout):**
>    The ToSeven motor controller uses a software In-Application Programming (IAP) bootloader stored in the first 6 KB of Flash (`0x08000000`). If custom application firmware encounters a hardware fault (such as a BusFault or HardFault during peripheral initialization) before initializing the UART command handler, **the controller cannot be commanded to reboot via software**.
>    - Because the physical controller is sealed in high-tensile polyurethane potting within an aluminum horseshoe casting, **recovering a locked controller requires physical depotting and direct hardware SWD flashing (ST-Link/J-Link)**.
>    - **Do NOT flash custom bare-metal firmware (`dm01_firmware` / `dm02_firmware`) onto a sealed controller that you are unwilling or unable to depot and attach to an SWD debugger.**
> 2. **Factory Stock Firmware & Display Tools Are Fully Safe:**
>    - Flashing official stock factory `.Pack` firmware containers (`TO7-DM01-*.Pack`, `TO7-DM02-*.Pack`) is verified safe and reversible.
>    - Unlocking and flashing display firmware dumps (`T24V`, `T154`, `LCD135C`, `M15`) is verified safe and operates over direct USB CDC ISP without touching the motor controller.
> 3. **Bench Power Safety Rules:**
>    - Never flash or test controllers using a high-discharge lithium bicycle battery. Always clamp an adjustable DC bench power supply to <= 1.0 A at 48 V.
>    - Operating custom open motor firmware on an actual bicycle is strictly prohibited at this stage.

---

## What Is Implemented Right Now (`v0.3.0-beta.8` - Release Beta)

Here is the verified functionality available in this repository:

1. **DM+ Firmware Studio Desktop GUI (`dm-plus-gui`):**
   - Native cross-platform desktop application built with Slint and Rust, running on Linux, macOS, and Windows.
   - Built-in embedded **DM01** (external 12-FET) and **DM02** (internal 6-FET) Open Firmware containers for single-click flashing without external files.
   - Automatic target hardware detection when loading custom `.Pack` files, locking the selector to prevent cross-flashing.
   - Fixed-height Adwaita compliance and electrical limit banners with standardized Libadwaita styling and zero layout jumping.
   - Interactive current limit controls with real-time visual safety warnings when exceeding stock factory ratings (> 30 A on DM01, > 18 A on DM02).
   - Live serial flashing with automatic port discovery, hotplug detection, and offline unlocked `.hex` export.
   - Pre-built binary packages generated for Linux x86_64, Windows x64, and macOS Universal (`.app`).

2. **Motor Controller IAP Flasher & Container Engine (`toseven_pack` & GUI):**
   - **Universal Container Support:** Automatic key derivation and payload decryption across all 15 official DM01 and DM02 factory releases.
   - **Robust Flashing Protocol Engine:** Full handshake state machine supporting dual-baud bootloader probing (38400 / 115200 baud), automated wakeup framing, paced sector streaming, and physical wire stream cipher masking.
   - **Direct ELF-to-.Pack Packaging:** Natively extracts loadable segments directly from Cortex-M ELF binaries into official 128 KB `.Pack` containers without requiring external `objcopy` utilities.
   - **Bit-Exact Verification:** Verifies container integrity and byte-exact roundtrips matching official vendor tooling.

3. **Display Lockout Patcher & ISP Flasher (`toseven_pack display`):**
   - Automatically parses Intel HEX display firmware dumps (`T24V`, `T154`, `LCD135C`, `M15`).
   - Detects and neutralizes all 6 restriction branch gates introduced in post-October 2024 factory firmware using clean-room ARM Thumb NOP patches.
   - Restores permanent access to `Speed&Power`, maximum current limits, and wheel circumference menus.
   - Flashes patched display firmware directly over USB CDC using the MegaHunt ISP bootloader protocol, or exports unlocked `.hex` files for offline flashing.

4. **Clean-Room Motor Controller Architecture (`dm01-firmware`):**
   - 100% `#![no_std]` embedded Rust implementation of Field Oriented Control (FOC), Clarke/Park transforms, SVPWM duty generation with dead-time margins (6.0% to 94.0%), and PLL Hall velocity/angle observer.
   - Unified multi-target support for both **DM01** (external 12-FET, 30A Sport) and **DM02** (internal 6-FET, 18A Sport), with safe firmware ceilings (32.0 A max for DM01, 20.0 A max for DM02) and active UI/firmware warnings whenever exceeding stock factory ratings (> 30 A on DM01, > 18 A on DM02).
   - Dedicated bare-metal Cortex-M4F binary targets for both motors: `dm01_firmware` and `dm02_firmware`.
   - **Gear-Safe Progressive Profile (`GearSafeKnee`):** Soft assist (<= 30%) below 18 RPM protecting the blue nylon primary reduction gear against standing-start stomps, ramping to 100% assist at 35 RPM with progressive quadratic response.
   - **Unified Level Ceilings (`UnifiedLevelCeiling`):** Assist Level 1 strictly respects configured power limits rather than drawing full mode current under heavy pedaling.
   - **Automatic Dynamic Span Tracking (`AutoSpanTracking`):** Dynamic effort scaling that adapts to resting zero-point thermal drift, allowing riders to reach full assist across all riding temperatures.
   - **True Zero LVC Cutoff (`TrueZeroCutoff`):** Linear taper down to exact 0.0 A across a 3.0 V reserve window, eliminating sudden BMS cutouts under battery sag.
   - **Universal Sensorless Speed Fallback:** Computes static midpoint gear ratios from drivetrain sprocket teeth. When the physical wheel speed sensor disconnects or loses its magnet, dynamically estimates vehicle speed from cadence to maintain speedometer telemetry and speed governing.
   - **Hardware Safety & Low-Level Drivers:**
     - **Startup Shunt Calibration:** 256-sample synchronous startup calibration to cancel stationary phase current amplifier offset before inverter engagement.
     - **Non-Volatile Storage:** Flash configuration driver persisting runtime settings and display parameters across power cycles with CRC-16 integrity.
     - **Hardware Overcurrent Break Protection:** Comparator-triggered break input that immediately disables all 6 MOSFET gates in < 50 ns upon overcurrent trip.
     - **Hardware Watchdog (`IWDG`):** 500 ms safety trip window refreshed deterministically inside the 1 ms SysTick control loop.
     - **Automated Motor Self-Commissioning:** Safe, low-power automated self-commissioning procedure executing rotor alignment, Hall sequence check, electrical angle offset detection, and phase wiring validation.

5. **Physical Simulation & Verification Suite:**
   - 78 automated unit, integration, and physical simulation tests running in CI (69 unit tests, 6 physical simulation scenarios, 3 stock identity roundtrip tests).
   - 6 physical scenario simulations: steep hill takeoff, speed limit soft taper, continuous linear thermal rollback, battery sag / reserve cushion, SVPWM dead-time clamp verification, and hardware safety watchdog latency.
   - **Pre-Flight Sensor Triage (`toseven_pack diagnostic`):** Automated bench checkout evaluating Hall sensor validity, resting torque sensor zero-point drift, throttle idle voltage, and controller/motor thermals prior to applying main battery power.
   - **Live Parameter Tuning (`toseven_pack tune`):** High-speed dynamic parameter injection directly into controller memory over standard display UART, adopting new current and speed limits within <= 1 ms on the very next FOC tick with zero MCU reboot and zero Flash wear.
   - **Street-Legal Compliance & Tamper-Proof Lock (EN15194 / EPAC):**
     - **Street-Legal Default Mode:** Defaults to 25.0 km/h speed cutoff and 250W continuous power limit on boot. Holding **Brake Lever + Power-On** at startup temporarily disengages street-legal limits for that ride.
     - **Permanent Street-Legal Lock (Tamper-Proof):** Compile-time flag (`--features street_legal_lock`) or GUI EEPROM latch permanently locking the controller to EN15194 limits. Standalone throttle is capped to 6 km/h walk-assist, and the Brake+Power boot combo, display menu speed adjustments, and UART speed bypasses are permanently locked out and ignored.
   - **Decoupled Handlebar Tuning Engine:** Full preservation of all 13 factory throttle modes with on-trail profile switching via display proxies and handlebar button shortcuts.

6. **Native Serial Transport & Cross-Platform Parity:**
   - Unified native serial engine operating on macOS, Linux, and Windows without external script dependencies.
   - Complete modem line control (DTR and RTS) for bootloader reset and MegaHunt USB CDC-ACM display bootloader connection requirements.
   - Hardware and kernel FIFO queue flushing before baud rate transitions and probe sequences to purge leftover telemetry frames.

7. **Documentation & Protocol Specifications:**
   - Hardware pinout runbooks ([`Documentation/Bench_Testing_Bringup_Guide.md`](Documentation/Bench_Testing_Bringup_Guide.md), [`Documentation/ToSeven_Controller_Depotting_and_Diagnostics_Guide.md`](Documentation/ToSeven_Controller_Depotting_and_Diagnostics_Guide.md)).
   - Display architecture and lockout bypass specification ([`Documentation/ToSeven_Display_Firmware_Architecture.md`](Documentation/ToSeven_Display_Firmware_Architecture.md)).
   - Bluetooth Low Energy (BLE) and mobile app protocol specification ([`Documentation/ToSeven_Bluetooth_and_App_Protocol.md`](Documentation/ToSeven_Bluetooth_and_App_Protocol.md)).

---

## Clean-Room Reimplementation & AI-Assisted Engineering

### Methodology & Architecture
This project is a **clean-room reimplementation** written in memory-safe, modern `#![no_std]` Rust:

- **Decompilation-Informed Architecture:** Register maps, communication framing, state machines, and configuration layouts were documented through static analysis of stock factory firmware binaries.
- **No Proprietary Code Reuse:** No vendor C code, proprietary assembly routines, or closed-source libraries are included. Every algorithm, filter, and state machine was documented and rewritten from scratch in idiomatic Rust.
- **Asset Isolation:** All proprietary manufacturer assets (`.Pack` containers, `.hex` display dumps, APK tuning apps, and vendor manuals) are strictly isolated in a `.gitignore`d directory and cataloged solely by SHA-256 hashes in [`blobs/README.md`](blobs/README.md).

### AI Assistance & Engineering Disclosure
This codebase was developed with the assistance of advanced generative AI models (Google Gemini and Anthropic Claude):
- **Technical Role:** AI models were utilized as pair-programmers to assist in analyzing obfuscated ARM machine code, cross-referencing peripheral register maps, translating decompiled math into safe Rust algorithms, and developing automated simulation tests.
- **Verification & Determinism:** The codebase relies on automated regression tests, bit-exact roundtrips, cryptographic digests, and deterministic physical simulations. It has not undergone an exhaustive line-by-line manual human audit of all decompiled routines; community testing, peer review, and bench validation remain essential steps.
- **Purpose:** This work is conducted strictly for interoperability, research, safety improvements, and consumer right-to-repair.

---

## Community Inquiries & Technical Feedback

Below are concise summaries of core technical solutions addressing community feedback on the stock firmware:

### 1. Torque Sensor Dynamic Range & Zero-Point Drift
- **The Issue:** Physical strain gauge deflection during normal hard pedaling produces a signal far below the nominal ceiling, and resting zero-point voltages drift with motor temperature. Stock firmware normalizes against a rigid fixed ceiling, preventing riders from reaching full assist without excessive artificial multipliers.
- **The DM+ Solution:** Auto-Span Tracking dynamically pins the assist span relative to the tracked resting zero-point. Assist sensitivity remains predictable across thermal drift, allowing riders to achieve full motor assist with natural pedal force.

### 2. Low-Cadence Power Delivery vs. Nylon Primary Gear Protection
- **The Issue:** Delivering maximum torque at near-zero cadence shears teeth off the internal blue nylon reduction gear during standing-start pedal stomps. Stock firmware protected the gear by penalizing all low-cadence riding with a severe linear ramp, causing riders to stall on steep hills.
- **The DM+ Solution:** The default **Gear-Safe Progressive Profile** softens assist below 18 RPM to shield the nylon gear, then ramps smoothly to 100% assist at 35 RPM. Full climbing power is unlocked across the active pedaling band (35–110+ RPM) without risking gear failure. An uncapped profile is available for aftermarket metal gear builds.

### 3. Battery Low-Voltage Cutoff (LVC) Cushion & True Zero Cutoff
- **The Issue:** Stock firmware maintains a residual current floor at low voltage rather than cutting power entirely. Under heavy hill climbing, this remaining draw causes sagging batteries to trip their BMS into sudden, jarring total shutdown.
- **The DM+ Solution:** True Zero Cutoff smoothly tapers current down to exact 0.0 A across a 3.0 V reserve window above the low-voltage cutoff. Power is completely suppressed at the cutoff threshold, protecting battery cells and preventing unexpected BMS disconnections.

### 4. Drivetrain Kinematics & Sensorless Speed Fallback
- **The Issue:** In stock firmware, drivetrain tooth count settings have no functional effect, and losing the spoke magnet or wheel sensor cable completely disables display speedometers and speed limiters.
- **The DM+ Solution:** DM+ uses configured chainring and cassette sprocket tooth counts to calculate midpoint gear ratios. If the physical wheel sensor signal is lost, the controller dynamically estimates vehicle speed from cadence to keep display speedometers functioning and maintain smooth speed governing. Detailed forensic comparison and interrupt architecture are documented in [`Documentation/Forensic_Firmware_Gap_Analysis_V6510C_vs_Open_Firmware.md`](Documentation/Forensic_Firmware_Gap_Analysis_V6510C_vs_Open_Firmware.md).

---

## Detailed Documentation & Reference Guides

<details>
<summary><strong>Comparison: Stock Factory Firmware vs. DM+ Open Architecture</strong></summary>

<br>

| Feature / Subsystem | Stock Factory Firmware | DM+ Open Firmware (`v0.3.0`) | Benefit to Rider / Builder |
| :--- | :--- | :--- | :--- |
| **Low-Cadence Climbing Power** | Strict linear scaling; assist cut by 60% at 25 RPM | **Selectable Profiles:** Gear-Safe Progressive (default, <= 30% under 18 RPM, full assist at 35 RPM) or Uncapped Cadence (metal gear only) | More power on steep hill starts without nylon gear damage |
| **Torque Assist Ceilings** | Dual-Ceiling Split: Torque sensor ignores assist level `Pwr%` and uses Mode Max | **Unified Level Ceiling Policy:** Assist Level 1 strictly respects configured `Pwr%` limit | Predictable power scaling and battery conservation across assist levels |
| **Pedal Effort Response** | Rigid linear effort multiplier | **Progressive Quadratic & High-Sensitivity Curves:** Gentle feathering at low pedal effort, progressive boost on steep climbs | Natural trail feel; reduced leg fatigue for urban commuting |
| **Takeoff & Standstill Decay** | Susceptible to startup shudder from cold Hall angle estimation | **Direct Physical Hall Sync + 100ms Decay:** Synchronizes to physical Hall pins; decays to zero velocity when stationary | Smooth takeoff without shudder or runaway assist |
| **Hardware Safety Watchdog** | Inactive or unmonitored during boot hangs | **Independent Hardware Watchdog (IWDG):** Active 500 ms window timer clocked by independent oscillator | Failsafe reboot into safe High-Z shutdown if firmware hangs |
| **MOSFET Shoot-Through Protection** | Proprietary timing table | **Guaranteed Dead-Time Clamp:** SVPWM duty cycles strictly clamped to 6.0% to 94.0% | Eliminates high-side/low-side FET shoot-through risk under transients |
| **Emergency Cutoff Latency** | Software-filtered delay | **Instantaneous Shutdown (< 50 ns hardware trip):** Brake lever, backpedaling, Hall fault, or gear shift sensor | Immediate motor disengagement for maximum rider safety |
| **Thermal Protection** | Sudden cliff or delayed step reduction | **Smooth Continuous Linear Rollback:** Continuous linear derating from 70°C down to 0.0 A at 100°C | Smooth power reduction while cooling without sudden cutouts |
| **Battery LVC Cushion** | Fixed 3.0V cushion tapering down to 25% current floor | **True Zero Cutoff (TrueZeroCutoff):** Linear taper down to exact 0.0 A at LVC; completely suppresses PWM below LVC | Eliminates abrupt battery BMS shutdowns under heavy hill climbing |
| **Multi-Voltage Adaptability** | Hardcoded constant-power cap; fixed 48V tuning | **Dynamic Voltage Anchoring & Auto-Detection:** Scales power ceiling relative to nominal pack voltage (36V, 48V, 52V) | Safe operation across all common e-bike battery voltages |
| **Display Compatibility** | Post-October 2024 menu lockouts prevent modifying settings | **Automated Clean-Room Patcher (`toseven_pack display patch`):** Neutralizes all 6 restriction gates on `T24V`, `T154`, `LCD135C`, `M15`, with live USB flashing or direct `.hex` export | Regain access to `Speed&Power`, current limits, and wheel circumference |
| **Verification & Flashing Tool** | Windows-only closed-source vendor executables | **Cross-Platform Open CLI (`toseven_pack`) & Desktop GUI:** Flash motor `.Pack` & display `.hex`, auto-detect ports, dry-run simulate, export patched hex | Open, scriptable, cross-platform (macOS/Linux/Windows) flashing and testing |

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
# Inspect container metadata, vector table, and factory configuration
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- info <firmware.Pack>

# Decrypt firmware binary payload from .Pack container
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

# Pre-flight sensor triage for bench testing (evaluates Halls, torque zero, throttle idle, temps)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- diagnostic --simulate
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- diagnostic <port>

# Live parameter tuning via UART (instant runtime injection, zero reboot, zero Flash wear)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- tune --sport 28 --city 18 --eco 10 --speed 32 --simulate
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- tune <port> --sport 28 --city 18 --eco 10 --speed 32

# Monitor live encrypted UART telemetry (or run simulated ride demo)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- monitor --simulate
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- monitor <port> --baud 9600

# Run comparative 6-scenario ride simulation suite (Stock vs. Open Firmware)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- simulate

# List connected USB serial adapters and displays
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- ports
```

</details>

<details>
<summary><strong>Hardware Architecture, Pinouts & Bring-up Runbook</strong></summary>

<br>

### Hardware Architecture Overview ([`Documentation/DM01_Hardware_Specification.md`](Documentation/DM01_Hardware_Specification.md))
- **Processor:** 32-bit ARM Cortex-M4F microcontroller with hardware single-precision floating-point unit (FPU).
- **Inverter Drive:** Center-aligned 16 kHz complementary 6-FET PWM with 1.4 µs dead-time margins and comparator-driven emergency break input.
- **Analog Subsystem:** Dual-channel simultaneous ADC sampling for Phase Shunts U/V, bus voltage, analog strain gauge torque, throttle, and motor/controller thermistors.
- **Display Communication:** Bidirectional 9600 Baud bus running over standard Julet 5-pin harness with symmetric stream encryption.

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
│   │   ├── power_curve.rs   # Exact integer power curve pipeline & arbitration
│   │   ├── tuning.rs        # Source-level tuning profiles (uncapped cadence, progressive torque)
│   │   ├── foc.rs           # Clarke/Park, SVPWM, and PLL Hall observer
│   │   ├── eeprom.rs        # Flash EEPROM storage manager & CRC-16
│   │   ├── flasher.rs       # IAP wakeup, header framing, chunk pacing & display ISP
│   │   ├── display_patcher.rs # Intel HEX parser & clean-room NOP lockout bypass
│   │   ├── pack.rs          # .Pack container parser, cipher, and repacker
│   │   ├── pac.rs           # Memory-mapped peripheral register bindings
│   │   ├── hal.rs           # Concrete DM01 hardware drivers
│   │   ├── protocol.rs      # Display UART protocol & wire cipher
│   │   ├── stock.rs         # Known official release registry (15 factory releases)
│   │   ├── sha256.rs        # Embedded no_std SHA-256 digest
│   │   ├── bin/
│   │   │   ├── firmware.rs  # Standalone bare-metal Cortex-M4F firmware entry point & vector table
│   │   │   └── toseven_pack.rs # CLI packaging, unpacking, verify, and flashing tool
│   │   └── memory.ld        # Flash and SRAM linker script for bare-metal targets
│   └── tests/
│       ├── simulation_suite.rs # 6 end-to-end comparative physical ride scenarios
│       └── stock_identity.rs   # Bit-exact stock roundtrip & tuning tests
├── dm-plus-gui/             # DM+ Firmware Studio desktop GUI (Slint)
│   ├── assets/              # Embedded standalone open firmware .Pack containers
│   ├── ui/
│   │   └── appwindow.slint  # Dark Libadwaita desktop interface
│   └── src/
│       └── main.rs          # Asynchronous flashing threads, file pickers, and serial integration
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
<summary><strong>Project Vision & Future Roadmap (Unified Ecosystem)</strong></summary>

<br>

Most e-bike firmware modifications force riders into a frustrating loop: disconnect the bike, connect a programming cable, run a proprietary flashing tool, adjust a setting, re-flash, test ride, and repeat.

Our long-term architectural goal is to build an open, unified motor and display ecosystem:
1. **Flash Once:** Flash open motor controller firmware onto the motor once.
2. **Handlebar Control:** Configure motor behaviors directly through the on-bike display menu (`LCD135C`, `T24V`, `T154`, `M15`) or smartphone via Bluetooth.
3. **Dynamic Tuning via UART:** Settings are communicated over the ToSeven UART protocol and saved directly into Flash EEPROM.
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
