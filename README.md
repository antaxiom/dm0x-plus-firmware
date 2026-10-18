# DM+ : Open Mid-Drive Firmware & Tooling Studio

[![CI](https://github.com/antaxiom/dm0x-plus-firmware/actions/workflows/ci.yml/badge.svg)](https://github.com/antaxiom/dm0x-plus-firmware/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B%20%7C%20%23!%5Bno__std%5D-orange.svg)](dm01-firmware/Cargo.toml)
[![Target](https://img.shields.io/badge/target-ARM%20Cortex--M4F-blue.svg)](https://developer.arm.com/Processors/Cortex-M4)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

An open-source `#![no_std]` Embedded Rust firmware recompilation, display unlocking toolchain, and cross-platform desktop studio for **ToSeven (TO7)** mid-drive e-bike motors (DM01, DM02) and handlebar displays (T24V, T154, M15, LCD135C).

> [!NOTE]
> **DM+** is the current working project name for this unified firmware and tooling ecosystem. Naming may evolve as the project expands to additional motor controllers and displays.

---

## What Is Implemented Right Now (`v0.3.0-beta.2`)

Rather than relying on future promises, here is the verified functionality available today in this repository:

1. **DM+ Firmware Studio Desktop GUI (`dm-plus-gui`):**
   - Native cross-platform desktop application built with Slint and Rust.
   - Built-in embedded DM01 Open Firmware container (`128 KB`) for instant, standalone flashing.
   - Native system file pickers (`rfd`) with enforced extension filters (`.Pack` and `.hex`).
   - Non-blocking asynchronous flashing engine with serial connection timeouts.
   - Pre-built binary packages generated for Linux x86_64, Windows x64, and macOS Universal (`.app`).

2. **Display Lockout Patcher & ISP Flasher (`toseven_pack display`):**
   - Automatically parses Intel HEX display firmware dumps (`T24V`, `T154`, `LCD135C`, `M15`).
   - Detects and neutralizes all 6 restriction branch gates introduced in the post-October 2024 factory firmware with ARM Thumb NOP (`0x00BF`) patches.
   - Restores permanent access to `Speed&Power`, maximum current limits, and wheel circumference menus.
   - Flashes patched display firmware directly over USB CDC using the MegaHunt ISP bootloader protocol.

3. **Motor Controller `.Pack` Container Engine (`toseven_pack`):**
   - Automatic XOR key detector (`0xA8`/`0x67` for DM01, `0xF7`/`0xE2` for DM02).
   - Bit-exact unpacking and repacking verified against official factory releases.
   - Natively extracts loadable segments directly from Cortex-M ELF binaries and packages them into official 128 KB `.Pack` containers without requiring host `objcopy`.

4. **Clean-Room Motor Controller Architecture (`dm01-firmware`):**
   - 100% `#![no_std]` embedded Rust implementation of Field Oriented Control (FOC), Clarke/Park transforms, SVPWM duty generation with dead-time clamps ($[6.0\%, 94.0\%]$), and Hall observer.
   - Complete 3-stage power curve pipeline modeled after the decompiled `V6510C` integer math.
   - Source-level tuning profiles: uncapped cadence, progressive quadratic torque response, and unified level assist ceilings.
   - 5-block Flash EEPROM manager (`0x0801FC00`) with XOR cipher, magic headers, and nibble CRC-16.
   - Standalone bare-metal Cortex-M4F binary (`dm01_firmware`) with dedicated vector table, peripheral register mappings (`pac.rs`), and safe High-Z gate-driver hardware drivers (`hal.rs`).

5. **Physical Simulation & Verification Suite:**
   - 45 automated unit, integration, and end-to-end physical simulation tests running in CI.
   - 6 physical scenario simulations: steep hill takeoff, speed limit soft taper, continuous linear thermal rollback, battery sag / reserve cushion, SVPWM dead-time clamp verification, and hardware safety watchdog latency.

6. **Safety & Bench Bring-up Runbook:**
   - Comprehensive bench bring-up guide ([`Documentation/Bench_Testing_Bringup_Guide.md`](Documentation/Bench_Testing_Bringup_Guide.md)) detailing Julet harness pinouts, SWD recovery points, and 48V current-limited bench power supply procedures.

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
- **DM+ Approach:** In DM+, rider effort is calculated strictly from the dynamic span:
  $$\Delta V_{\text{rider}} = V_{\text{applied}} - V_{\text{zero}}$$
  We are evaluating auto-span calibration where $V_{\text{max}}$ automatically shifts with the tracked resting zero-point:
  $$V_{\text{max}} = V_{\text{zero}} + V_{\text{span}}$$
  allowing riders to reach 100% assist at a realistic mechanical pedal force.

### 2. Low-Cadence Power vs. Nylon Primary Reduction Gear (GitHub Issue #2)
- **Observation:** The DM01 internal primary reduction relies on a composite blue nylon helical gear. Delivering maximum current ($30\text{ A}$ / $\approx 1500\text{ W}$) at very low cadence ($< 20\text{ RPM}$) introduces extreme mechanical shear stress on the gear teeth, creating a risk of tooth stripping under heavy load.
- **Stock Limitation:** Stock firmware penalizes low-cadence climbing with a rigid $\frac{\text{Cadence}}{60\text{ RPM}}$ multiplier, cutting power by 60% at 25 RPM. While this protects the plastic gear, it causes riders to stall on steep trail inclines.
- **DM+ Approach:** Power delivery is decoupled into selectable, transparent tuning profiles:
  - `Stock60Rpm`: Retains the conservative stock linear ramp for maximum mechanical longevity.
  - `GearSafeKnee`: Provides full torque response down to $25\text{ RPM}$, with a protective soft torque ceiling below $20\text{ RPM}$ to shield the nylon gear from dead-stop shock loads.
  - `Uncapped`: Delivers 100% requested torque assist down to $5\text{ RPM}$ for riders running reinforced all-metal gear sets or specialized applications.

### 3. Battery LVC Cushion & True Zero-Current Cutoff (GitHub Issue #3)
- **Observation:** The stock firmware features a $3.0\text{ V}$ reserve cushion above Low-Voltage Cutoff (LVC), but tapers current down to a $25\%$ floor rather than fully cutting off motor power. If an overloaded battery sags further, this $25\%$ residual draw can trigger a harsh, sudden BMS trip.
- **DM+ Approach:** DM+ implements a true taper to $0.0\text{ A}$ anchored to the user's configured $0\%$ battery voltage threshold. Power tapers smoothly across the reserve band down to an absolute zero current cutoff at LVC, preventing cell over-discharge and sudden BMS shutdowns while riding.

---

## Detailed Documentation & Reference Guides

<details>
<summary><strong>Comparison: Stock Factory Firmware (V6510C) vs. DM+ Open Architecture</strong></summary>

<br>

| Feature / Subsystem | Stock Factory Firmware (`V6510C`) | DM+ Open Firmware (`v0.3.0`) | Benefit to Rider / Builder |
| :--- | :--- | :--- | :--- |
| **Low-Cadence Climbing Power** | Strict $\frac{\text{Cadence}}{60\text{ RPM}}$ scaling; assist is cut by **60%** at 25 RPM (only 2.5 A / 125 W delivered) | **Selectable Profiles:** Gear-safe knee or uncapped cadence delivering full assist down to target RPM | Up to 2.5× more power on steep hill starts without stalling |
| **Torque Assist Ceilings** | Dual-Ceiling Split: Torque sensor **ignores** assist level `Pwr%` and uses only Mode Max | **Unified Level Ceiling Policy:** Optional profile enforcing that torque assist respects assist level `Pwr%` | True fine-grained battery conservation and level-separated power control |
| **Pedal Effort Response** | Rigid linear effort multiplier | **Progressive Quadratic & High-Sensitivity Curves:** Gentle feathering at low pedal effort, exponential boost on steep climbs | Natural, intuitive MTB trail feel; less leg strain for urban commuters |
| **Takeoff & Standstill Decay** | Susceptible to startup shudder from cold Hall angle estimation | **Direct Physical Hall Sync + 100ms Decay:** Synchronizes instantly to physical Hall pins; decays to zero velocity when stationary | Smooth, instantaneous takeoff without jerky shudder or runaway assist |
| **Hardware Safety Watchdog** | Inactive or unmonitored during boot hangs | **Independent Hardware Watchdog (IWDG):** Active 500 ms window timer clocked by 40 kHz LSI oscillator, serviced in 1 ms SysTick | Failsafe reboot into safe High-Z (`MOE=0`) if firmware ever hangs or crashes |
| **MOSFET Shoot-Through Protection** | Proprietary timing table | **Guaranteed Dead-Time Clamp:** SVPWM duty cycles strictly clamped to $[6.0\%, 94.0\%]$ ($1.5\,\mu\text{s}$ margin) | Eliminates high-side/low-side FET shoot-through risk under heavy transients |
| **Emergency Cutoff Latency** | Software-filtered delay | **Instantaneous 0 ms High-Z Shutdown:** Brake lever (`PB12`), backpedaling, Hall fault (`0b000`/`0b111`), or gear shift sensor | Immediate motor disengagement for maximum rider safety |
| **Thermal Protection** | Sudden cliff or delayed step reduction | **Smooth Continuous Linear Rollback:** Continuous linear derating from 70°C down to 0.0 A at 100°C | No jarring sudden loss of power; keeps motor running smoothly while cooling |
| **Battery LVC Cushion** | Fixed 3.0V cushion (39V..42V) tapering down to 25% current floor | **Cell-Aware Proportional Cushion & True 0A Taper:** Linear taper down to 0.0 A at LVC | Prevents abrupt battery BMS shutdowns under heavy hill climbing or acceleration |
| **Multi-Voltage Adaptability** | Hardcoded 50.0V constant-power cap; poor 36V / 52V optimization | **Dynamic Voltage Anchoring & Auto-Detection:** Scales power ceiling relative to nominal pack voltage (36V, 48V, 52V) | Safe operation across all common e-bike battery voltages without bricking |
| **Display Compatibility** | Post-October 2024 menu lockouts lock users out of settings | **Automated Clean-Room Patcher (`toseven_pack display patch`):** NOPs out all 6 restriction gates on `T24V`, `T154`, `LCD135C`, `M15` | Regain full access to `Speed&Power`, current limits, and wheel circumference |
| **Verification & Flashing Tool** | Windows-only closed-source executables (`IAP Tool 5.2`, `MH2020C_ISP`) | **Cross-Platform Open CLI (`toseven_pack`) & Desktop GUI:** Flash motor `.Pack` & display `.hex`, auto-detect ports, dry-run simulate | Open, scriptable, cross-platform (macOS/Linux/Windows) flashing and testing |

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
- Flashing custom firmware to any motor controller carries inherent electrical and mechanical risks. Use at your own risk.
