# ToSeven DM01 Open Firmware (`dm01-firmware`)

[![CI](https://github.com/alouk/toseven-firmware/actions/workflows/ci.yml/badge.svg)](.github/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B%20%7C%20%23!%5Bno__std%5D-orange.svg)](dm01-firmware/Cargo.toml)
[![Target](https://img.shields.io/badge/target-ARM%20Cortex--M4F-blue.svg)](https://developer.arm.com/Processors/Cortex-M4)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

An open-source, `#![no_std]` Embedded Rust firmware recompilation, container toolchain, and customization engine for **ToSeven (TO7) DM01** mid-drive e-bike motors (with DM02 and DM0X displays on the roadmap).

---

## 🎯 The Vision: "Mega Firmware" + Smart Display Ecosystem

Most e-bike firmware mods force riders into a frustrating cycle: disconnect the bike, plug in a programming cable, run a proprietary PC flashing utility, tweak a number, re-flash, test ride, and repeat.

**Our goal is to build an open, unified "Mega Firmware" paired with updated display firmware:**
1. **Flash Once:** Flash the open motor controller firmware onto the DM01 once.
2. **Handlebar Control:** Configure all motor behaviors directly through the on-bike display menu (`LCD135C`, `T24V`, `T154`, `M15`) or smartphone via Bluetooth.
3. **Dynamic Tuning via UART:** Settings are communicated over the ToSeven UART protocol and saved directly into Flash EEPROM (`0x0801FC00`).
4. **On-the-Fly Profile Switching:** Toggle between smooth commuting, progressive trail riding, or uncapped climbing power at the push of a handlebar button.

---

## ✨ Improvements & Advancements in Open Firmware (`v0.3.0`)

Compared to proprietary factory firmware (`V6510C`), the open `#![no_std]` firmware introduces major architectural, safety, and ride-feel enhancements while maintaining 100% hardware compatibility:

| Feature / Subsystem | Stock Factory Firmware (`V6510C`) | Open Firmware (`v0.3.0`) | Benefit to Rider / Builder |
| :--- | :--- | :--- | :--- |
| **Low-Cadence Climbing Power** | Strict $\frac{\text{Cadence}}{60\text{ RPM}}$ scaling; assist is cut by **60%** at 25 RPM (only 2.5 A / 125 W delivered) | **Uncapped Cadence Profile:** Delivers 100% requested torque assist at any cadence $\ge 5\text{ RPM}$ (6.2 A / 310 W at 25 RPM) | **2.5× more power on steep hill starts** from a dead stop without stalling or standing on pedals |
| **Torque Assist Ceilings** | Dual-Ceiling Split: Torque sensor **ignores** assist level `Pwr%` and uses only Mode Max | **Unified Level Ceiling Policy:** Optional profile enforcing that torque assist respects assist level `Pwr%` | True fine-grained battery conservation and level-separated power control |
| **Pedal Effort Response** | Rigid linear effort multiplier | **Progressive Quadratic & High-Sensitivity Curves:** Gentle feathering at low pedal effort, exponential boost on steep climbs | Natural, intuitive MTB trail feel; less leg strain for urban commuters |
| **Takeoff & Standstill Decay** | Susceptible to startup shudder from cold Hall angle estimation | **Direct Physical Hall Sync + 100ms Decay:** Synchronizes instantly to physical Hall pins; decays to zero velocity when stationary | Smooth, instantaneous takeoff without jerky shudder or runaway assist |
| **Hardware Safety Watchdog** | Inactive or unmonitored during boot hangs | **Independent Hardware Watchdog (IWDG):** Active 500 ms window timer clocked by 40 kHz LSI oscillator, serviced in 1 ms SysTick | Failsafe reboot into safe High-Z (`MOE=0`) if firmware ever hangs or crashes |
| **MOSFET Shoot-Through Protection** | Proprietary timing table | **Guaranteed Dead-Time Clamp:** SVPWM duty cycles strictly clamped to $[6.0\%, 94.0\%]$ ($1.5\,\mu\text{s}$ margin) | Eliminates high-side/low-side FET shoot-through risk under heavy transients |
| **Emergency Cutoff Latency** | Software-filtered delay | **Instantaneous 0 ms High-Z Shutdown:** Brake lever (`PB12`), backpedaling, Hall fault (`0b000`/`0b111`), or gear shift sensor | Immediate motor disengagement for maximum rider safety |
| **Thermal Protection** | Sudden cliff or delayed step reduction | **Smooth Continuous Linear Rollback:** Continuous linear derating from 70°C down to 0.0 A at 100°C | No jarring sudden loss of power; keeps motor running smoothly while cooling |
| **Battery LVC Cushion** | Fixed 3.0V cushion (39V..42V) regardless of pack chemistry | **Cell-Aware Proportional Cushion & Sag Filter:** 3.0V linear taper down to 25% current before hard BMS cutoff | Prevents abrupt battery BMS shutdowns under heavy hill climbing or acceleration |
| **Multi-Voltage Adaptability** | Hardcoded 50.0V constant-power cap; poor 36V / 52V optimization | **Dynamic Voltage Anchoring & Auto-Detection:** Scales power ceiling relative to nominal pack voltage (36V, 48V, 52V) | Safe operation across all common e-bike battery voltages without bricking |
| **Display Compatibility** | Post-October 2024 menu lockouts lock users out of settings | **Automated Clean-Room Patcher (`toseven_pack display patch`):** NOPs out all 6 restriction gates on `T24V`, `T154`, `LCD135C`, `M15` | Regain full access to `Speed&Power`, current limits, and wheel circumference |
| **Verification & Flashing Tool** | Windows-only closed-source executables (`IAP Tool 5.2`, `MH2020C_ISP`) | **Cross-Platform Open CLI (`toseven_pack`):** Flash motor `.Pack` & display `.hex`, auto-detect ports, dry-run simulate, live telemetry | Open, scriptable, cross-platform (macOS/Linux/Windows) flashing and testing |

---

## 🛠️ Quickstart & Tooling Guide

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (stable, 1.75+)

### Running Unit & Integration Tests
```bash
# Run all 45 unit, integration, and physical simulation tests
cargo test --workspace

# Run physical simulation suite (6 end-to-end scenarios)
cargo test --manifest-path dm01-firmware/Cargo.toml --test simulation_suite -- --nocapture

# Check formatting and strict clippy lints across workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Verify strict #![no_std] bare-metal embedded compatibility
cargo check --manifest-path dm01-firmware/Cargo.toml --no-default-features --lib
```

### Using the `toseven_pack` Container Tool
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

# Inspect the generated .Pack container
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- info build/TO7-DM01-OpenFirmware-v0.3.0.Pack

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

# Launch DM+ Firmware Studio Desktop GUI (Slint)
cargo run --manifest-path dm-plus-gui/Cargo.toml
```

---

## 🖥️ DM+ Firmware Studio (Slint Desktop GUI)

**DM+ Firmware Studio** (`dm-plus-gui`) is a native, cross-platform graphical desktop application built with [Slint](https://slint.dev/) and powered directly by the `dm01-firmware` engine:

```bash
# Launch the desktop studio
cargo run --manifest-path dm-plus-gui/Cargo.toml

# Package local release archives (CLI + GUI) into dist/
./scripts/package-release.sh
```

### Multi-Platform Release Toolchain
- **Automated Matrix CI/CD (`.github/workflows/release.yml`)**: Builds native binaries for **Linux (x86_64)**, **macOS Universal (Apple Silicon + Intel)**, and **Windows (x64)** without local cross-compilation overhead.
- **Local Host Packager (`scripts/package-release.sh`)**: Builds and bundles the current host's CLI archive and Desktop app (`.app` on macOS, `.tar.gz` on Linux) into `dist/` with SHA-256 checksums.

```
┌────────────────────────────────────────────────────────────────────────┐
│                   ToSeven Control Center (GUI Blueprint)              │
├──────────────────────────────┬─────────────────────────────────────────┤
│  🎯 Device Selection         │  ⚡ Motor Controller Tuning (.Pack)    │
│  [●] DM01 / DM02 Controller  │  - Cadence Scaling: [ Uncapped ▾ ]      │
│  [ ] T24V / T154 Display     │  - Torque Response: [ Progressive ▾ ]   │
│  Port: [/dev/tty.usbserial▾] │  - Assist Ceiling:  [ Unified Level ▾ ] │
│  Status: Connected (115200)  │  - Max Current:     [ 30.0 A          ] │
├──────────────────────────────┼─────────────────────────────────────────┤
│  🖥️ Display Firmware (.hex)  │  📊 Real-Time Diagnostic Telemetry      │
│  File: TO7-T24V-Y241225.hex  │  - Battery: 52.4 V | Current: 0.0 A     │
│  Status: 6 Lockout Gates     │  - Temp: 28.5 °C   | Cadence: 0 RPM     │
│  [✓] Auto-Unlock Advanced    │  - Throttle: 0.95V | Hall: [H1 H2 H3]   │
│      Menu On Flash           │                                         │
├──────────────────────────────┴─────────────────────────────────────────┤
│  [ Inspect Image ]      [ Patch .hex ]      [ Flash Firmware ]         │
└────────────────────────────────────────────────────────────────────────┘
```

### Architectural Design & Decoupling
1. **Core Decoupling:** All business logic, container ciphers, Intel HEX manipulations, and serial state machines reside in `dm01-firmware` (`display_patcher.rs`, `flasher.rs`, `serial.rs`, `pack.rs`).
2. **GUI Framework Architecture:**
   - **Slint (Selected):** Pure Rust declarative UI framework providing native styling, sub-millisecond startup, minimal binary footprint, and seamless cross-platform desktop integration directly backed by the `dm01-firmware` crate.
3. **Core GUI Features Planned:**
   - **Auto-Discovery:** Automatically scans USB devices for vendor IDs (MegaHunt USB CDC `MH1903 ROM BOOT`, FTDI/CH340 serial converters).
   - **One-Click Unlock:** Drag and drop any locked display `.hex` file, click "Unlock & Flash", and immediately regain full Advanced Menu access.
   - **Visual Power Curve Editor:** Interactive curve sliders visualizing torque ramp, cadence scaling, and thermal rollback.
   - **Bench Safety Interlock:** Guides the user through 48V current-limited bench power supply setup before committing writes.

---

## 🔍 Hardware Feasibility & Roadmap to Initial Release

### Can this compile into something that will run on the bike right now?
**Display firmware unlocking & flashing (`toseven_pack display patch/flash`) and stock motor `.Pack` flashing are ready for hardware today.** The custom bare-metal Cortex-M4F motor firmware (`dm01_firmware`) also compiles cleanly into a standalone ELF and packages into an official 128 KB `.Pack` container, with `TIM1` booting safely in High-Z (`MOE = 0`, 50% neutral duty). However, **do not flash the custom bare-metal motor firmware onto a live bike yet**: multi-channel DMA/ADC scan sequencing, `TIM1_UP` ISR timing, and UART ISR wiring in `hal.rs` / `firmware.rs` still require verification on a **48V current-limited bench power supply** in Phase 3.

```
[Phase 1: Algorithmic Core]       ✅ (Done: FOC, Power Curves, EEPROM, Pack Tooling)
            │
[Phase 2.1: Hardware Binding]     ✅ (Done: Volatile PAC, Safe High-Z TIM1 PWM, Hall+PLL, memory.ld)
            │
[Phase 2.2: Standalone .Pack]     ✅ (Done: Native ELF extraction, 128 KB .Pack, Live Serial & Monitor)
            │
[Phase 3: Bench Verification]     ⏳ (Next: ADC DMA scan sequencing + 48V current-limited bench test)
            │
[Phase 4: Public v1.0 Release]    🎉 (Initial DM01 Open Firmware Release)
```

### What was completed in Phase 2?

1. **Hardware Pin & Register Extraction ([`Documentation/DM01_Hardware_Specification.md`](Documentation/DM01_Hardware_Specification.md)):**
   - 100% extracted from decompiled vendor assembly without depotting silicone potting compound.
   - Core: ARM Cortex-M4F with hardware single-precision FPU enabled (`SCB->CPACR`).
   - `TIM1` (`0x40012C00`): Center-aligned 16 kHz complementary 6-FET PWM (`ARR = 5999`, `DTG = 100` ≈ 1.38 µs dead-time). High-side: `PA8, PA9, PA10`. Low-side: `PB13, PB14, PB15`. Break: `PB12`.
   - `ADC1/ADC2` (`0x40012400` / `0x40012800`): Dual injected sequence for Shunts U (`PA0`), V (`PA1`), and derived W. Regular channels: Shunt/Bus (`PA1`), Torque strain gauge (`PA4`), Throttle (`PA7`), Battery voltage (`PB0`), Temperature (`PB1`).
   - `USART3` (`0x40004800`): Display UART at **9600 Baud** (8-N-1), TX on `PB10`, RX on `PB11`, wire XOR `0x69`.
2. **Peripheral Access Crate (`pac.rs`) & HAL (`hal.rs`):**
   - Implemented zero-cost register structs and safe hardware accessors.
   - `Dm01PwmHardware`: Configures `TIM1` for 16 kHz complementary PWM with dead-time and provides atomic duty updates and emergency High-Z shutoff.
   - `Dm01SensorsHardware`: Samples ADC channels and Hall rotor bits.
3. **Linker Script (`memory.ld`) & Vector Table:**
   - Vector table placed at Flash origin `0x08001800` above the 6 KB factory bootloader with `SCB->VTOR` relocation.
   - Initial MSP set to `0x20005628`, matching stock ToSeven DM01 specifications.
4. **Native ELF Extraction & `.Pack` Packaging:**
   - `toseven_pack` natively extracts loadable Flash segments directly from Cortex-M ELF binaries without external host tools (`objcopy`).

### Safety Verification Before Physical On-Bike Testing

- **Bench Testing & Hardware Bring-up Runbook:**
  - Follow the complete safety runbook in [`Documentation/Bench_Testing_Bringup_Guide.md`](Documentation/Bench_Testing_Bringup_Guide.md).
  - Before flashing to a motor connected to a high-current battery pack, inverter gate drivers must be tested on a bench power supply strictly clamped to $\le 1.0\text{ A}$ at $48\text{ V}$.
  - *Rationale:* Ensures gate driver timing and complementary PWM prevent shoot-through without stressing FETs.
- **SWD Recovery:**
  - Maintain an ST-Link/J-Link connection to SWDIO/SWCLK/GND on the controller PCB to guarantee unbrick recovery.

---

## 📁 Repository Structure

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

---

## ⚖️ Clean-Room & Legal Notice

This project is a clean-room reverse-engineering, documentation, and open-source re-implementation effort created strictly for interoperability, research, and educational purposes.

- No proprietary binaries, copyrighted vendor manuals, or closed-source code are distributed in this repository.
- All proprietary `.Pack` files, `.hex` display dumps, and vendor APKs remain the property of their respective owners and are isolated in `.gitignore`d storage as detailed in [`blobs/README.md`](blobs/README.md).
- Flashing custom firmware to any motor controller carries inherent electrical and mechanical risks. Use at your own risk.

---

## 🤝 Contributing

Contributions are welcome! Please ensure all code additions:
1. Retain strict `#![no_std]` embedded compatibility for core controller modules.
2. Pass `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`.
3. Include unit tests covering any new power curve or state machine logic.

