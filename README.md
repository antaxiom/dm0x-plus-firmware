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

# Create an official-format .Pack file from any raw ARM binary
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- create dm01 payload.bin output.Pack

# Cryptographically verify container integrity and bit-exact round-trip
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- verify <firmware.Pack>
```

---

## 🔍 Hardware Feasibility & Roadmap to Initial Release

### Can this compile into something that will run on the bike right now?
**The algorithmic and mathematical core is 100% complete, fully unit-tested, and verified against decompiled vendor assembly.**

However, **before flashing physical motor hardware**, the following embedded hardware steps are required:

```
[Phase 1: Algorithmic Core] ✅ (Done: FOC, Power Curves, EEPROM, Pack Tooling)
            │
[Phase 2: Hardware Binding] ⏳ (In Progress: STM32/GD32 TIM1, ADC, UART HAL)
            │
[Phase 3: Bench SWD Test]   ⚠️ (Required: Bench power supply & SWD unbrick recovery)
            │
[Phase 4: Public v1.0]      🎉 (Initial DM01 Public Release)
```

### What is Needed Before an Initial Public Release?

1. **MCU Register Peripheral Binding (PAC/HAL):**
   - The DM01 uses an ARM Cortex-M4F MCU (STM32/GD32 memory map) loaded at `0x08001800` above a 6 KB factory bootloader.
   - We must wire the hardware peripheral base addresses identified in our disassembly:
     - `TIM1` (`0x40012C00`): 6-channel complementary PWM with hardware dead-time generator.
     - `ADC1/ADC2` (`0x40012400` / `0x40012800`): Injected dual-channel sampling synchronized to PWM center-aligned counter for phase shunts.
     - `USART1/USART3` (`0x40013800` / `0x40004800`): Display UART communication.
     - `GPIO` (`0x40010800`..`0x40011400`): Hall sensor inputs and brake cutoffs.
2. **Linker Script & Vector Table Offset (`memory.x`):**
   - Relocate the Vector Table Offset Register (`SCB->VTOR = 0x08001800`) so hardware interrupts branch to our Rust handlers rather than the bootloader.
3. **Bench Verification with Current-Limited Power Supply (Safety Critical!):**
   - Inverter gate drivers must be tested on an isolated bench power supply (e.g., $48\text{ V} / 1.5\text{ A}$ limit) before connecting to a high-discharge battery pack.
   - *Why:* Any inverted GPIO polarity or insufficient dead-time will turn on the high-side and low-side FETs simultaneously, causing shoot-through that destroys the 12-FET power stage.
4. **SWD Hardware Recovery Route:**
   - Verify connection to the SWD test points (SWDIO / SWCLK / GND) on the controller PCB using an ST-Link or J-Link to dump the factory bootloader and provide a guaranteed unbrick path.

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
│   │   ├── protocol.rs      # Display UART 0x69 cipher protocol
│   │   ├── stock.rs         # Known official release registry
│   │   ├── sha256.rs        # Embedded no_std SHA-256 digest
│   │   └── bin/toseven_pack.rs # CLI packaging and inspection utility
│   └── tests/
│       └── stock_identity.rs # Bit-exact stock roundtrip & tuning tests
├── Documentation/
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
