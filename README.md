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

## 🚀 Key Features Built & Verified

- **100% Decompiled V6510C Integer Power Curve Pipeline (`src/power_curve.rs`):**
  - **Dual Current Ceilings:** Models the stock split between the Global Mode Ceiling (used by torque sensing) and Level Power Ceilings (used by cadence and throttle).
  - **Continuous Thermal Rollback:** Linear derating from Sport ceiling ($30.0\text{ A}$) at $70^\circ\text{C}$ to $0\text{ A}$ at $100^\circ\text{C}$.
  - **High-Voltage Constant-Power Cap:** Preserves $1500\text{ W}$ peak power above $50.0\text{ V}$.
  - **Strain-Gauge Drift Tracking:** 5-sample auto-zero drift compensation with $4:1$ IIR filtering within $400..1000\text{ mV}$.
  - **Arbitration & Soft Tapers:** $\max(\text{Torque}, \text{PAS}, \text{Throttle})$, $\pm0.2\text{ A/step}$ current ramp limiter, $1.5\text{ km/h}$ speed soft-taper, and $3.0\text{ V}$ Low-Voltage Cutoff (LVC) cushion.
- **Source-Level Customization Engine (`src/tuning.rs`):**
  - **Uncapped Cadence:** Eliminates the stock $60\text{ RPM}$ penalty ($\max(\text{cadence}, 5) / 60$) that cuts climbing power in half at $30\text{ RPM}$.
  - **Progressive Torque Response:** Quadratic / exponential strain response for natural, explosive trail riding.
  - **Unified Level Ceilings:** Optional policy forcing torque assist to respect the assist level's `Pwr%` limit.
- **High-Performance Field-Oriented Control (`src/foc.rs`):**
  - Fixed-point Q15 and Cortex-M4F single-precision FPU Clarke/Park transforms.
  - Dual PI current loops ($I_d / I_q$), flux weakening, and PLL Hall angle observer.
  - 7-segment Space Vector PWM (SVPWM) with $[0.06, 0.94]$ dead-time linear duty shaping.
- **Flash EEPROM Manager (`src/eeprom.rs`):**
  - Complete 5-block (480-byte) Flash manager (`0x0801FC00`) with $0\text{xA5}$ XOR obfuscation, `A5 01 FE` magic verification, and 16-entry nibble CRC-16 checks.
- **Official `.Pack` Container Tooling (`toseven_pack` CLI):**
  - Inspect (`info`), extract (`unpack`), rebuild (`repack`), generate from scratch (`create`), and cryptographically verify (`verify`) official vendor update packages.
  - Bit-exact round-trip verification matches official stock containers down to the last byte.

---

## 🛠️ Quickstart & Tooling Guide

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (stable, 1.75+)

### Running Unit & Integration Tests
```bash
# Run all 23 unit tests and container integration tests
cargo test --manifest-path dm01-firmware/Cargo.toml

# Check formatting and strict clippy lints
cargo fmt --manifest-path dm01-firmware/Cargo.toml -- --check
cargo clippy --manifest-path dm01-firmware/Cargo.toml --all-targets -- -D warnings

# Verify strict #![no_std] bare-metal compatibility
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

# Package directly from ELF or raw binary into an official ToSeven .Pack firmware container
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- create dm01 dm01-firmware/target/thumbv7em-none-eabihf/release/dm01_firmware build/TO7-DM01-OpenFirmware-v0.3.0.Pack

# Inspect the generated .Pack container
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- info build/TO7-DM01-OpenFirmware-v0.3.0.Pack

# Cryptographically verify container integrity and bit-exact round-trip
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- verify <firmware.Pack>
```

---

## 🔍 Hardware Feasibility & Roadmap to Initial Release

### Can this compile into something that will run on the bike right now?
**Yes! The bare-metal Cortex-M4F firmware compiles cleanly into a standalone ELF binary and packages directly into an official ToSeven `.Pack` file ready for flashing.**

```
[Phase 1: Algorithmic Core]       ✅ (Done: FOC, Power Curves, EEPROM, Pack Tooling)
            │
[Phase 2.1: Hardware Binding]     ✅ (Done: PAC, HAL, TIM1 Complementary PWM, ADC, UART, memory.ld)
            │
[Phase 2.2: Standalone .Pack]     ✅ (Done: Native ELF extraction and 128 KB .Pack generation)
            │
[Phase 3: Bench SWD Verification] ⏳ (Next: Isolated 48V current-limited bench power supply test)
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

- **Bench Power Supply with Current Limiter:**
  - Before flashing to a motor connected to a high-current battery pack, inverter gate drivers must be tested on a bench supply limited to $\le 1.5\text{ A}$ at $48\text{ V}$.
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
│   │   ├── pack.rs          # .Pack container parser, cipher, and repacker
│   │   ├── pac.rs           # Memory-mapped peripheral register bindings (TIM1, ADC, USART, GPIO)
│   │   ├── hal.rs           # Concrete DM01 hardware drivers (Dm01PwmHardware, Dm01SensorsHardware)
│   │   ├── protocol.rs      # Display UART 0x69 cipher protocol
│   │   ├── stock.rs         # Known official release registry
│   │   ├── sha256.rs        # Embedded no_std SHA-256 digest
│   │   ├── bin/
│   │   │   ├── firmware.rs  # Standalone bare-metal Cortex-M4F firmware entry point & vector table
│   │   │   └── toseven_pack.rs # CLI packaging, unpacking, and direct ELF extraction tool
│   │   └── memory.ld        # Flash (0x08001800) and SRAM (0x20000000) linker script
│   └── tests/
│       └── stock_identity.rs # Bit-exact stock roundtrip & tuning tests
├── Documentation/
│   ├── DM01_Hardware_Specification.md # Complete reverse-engineered register map and pinout
│   ├── DM01_Power_Curve_Guide.md # Comprehensive guide explaining power delivery
│   ├── Official_Specifications.md # Hardware limits, pinouts, and manual references
│   └── Research_Downgrade_and_Update.md # Container encryption & IAP bootloader notes
├── blobs/                   # External proprietary assets (.gitignore'd for clean-room legal safety)
│   └── README.md            # Verified SHA-256 asset catalog
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

