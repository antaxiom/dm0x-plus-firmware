# Project Notes & Architecture Reference (`AGENTS.md`)

## Project Mission & Roadmap
This repository is a reverse-engineering, documentation, and `#![no_std]` Embedded Rust firmware recompilation project for **ToSeven (TO7)** mid-drive e-bike motors and displays:
1. **Phase 1 — DM01 (`TO7-DM01-K260521A-V6510C.Pack`):** Complete binary de-obfuscation, ARM Cortex-M4F disassembly, human-readable documentation of the power curve and FOC architecture, and functionally identical Rust recompilation (`dm01-firmware` `v0.3.0`).
2. **Phase 2 — DM02 (`TO7-DM02-K260526-V6510C.Pack`):** Shared `V6510C` architecture analysis, `.Pack` container decryption (`0xF7` header / `0xE2` payload), unified multi-target motor control, and `TS500W` factory EEPROM verification.
3. **Phase 3 — DM0X Displays (`T24V`, `T154`, `M15`, `LCD135C`) & Custom Firmware Enhancements:** Source-level tuning profiles (uncapped cadence, progressive torque curves, unified assist ceilings), open tuning tooling, and display firmware decompilation.

---

## Workspace Structure
- [`dm01-firmware/`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/Cargo.toml) — `#![no_std]`-compatible Rust firmware recompilation and `.Pack` tooling crate (`v0.3.0`):
  - [`src/pack.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/pack.rs) — `.Pack` firmware container parser, auto-key detector (`0xA8`/`0x67` for DM01, `0xF7`/`0xE2` for DM02), Cortex-M4F vector table validator, and bit-exact `.Pack` repacker.
  - [`src/eeprom.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/eeprom.rs) — 5-block (`0x20, 0x40, 0x40, 0xA0, 0xA0` = 480 bytes) Flash EEPROM configuration manager (`0x0801FC00`), `0xA5` XOR cipher, `A5 01 FE` magic check, 16-entry nibble CRC-16 (`crc16_toseven`), and embedded bit-exact DM01 (`TS1000W`) & DM02 (`TS500W`) factory EEPROM images.
  - [`src/power_curve.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/power_curve.rs) — Exact decompiled `V6510C` integer-math power curve pipeline:
    - Stage 1: Riding Mode ceilings (`Eco`/`City`/`Sport`), continuous linear thermal rollback (`70°C..100°C` anchored to Sport ceiling), high-voltage constant-power cap (above `50.0 V`), and the dual-ceiling split (**Global Mode Ceiling** for Torque vs. **Level Power Ceiling** `Pwr%` for PAS/Throttle).
    - Stage 2: Torque Sensor 5-sample auto-zero drift tracker (`4:1` IIR within `400..1000 mV`), configurable IIR pedal filter, `StrengthValue * EAssist%` effort multiplier, `CurrentOffset`, and `max(Cadence, 5) / 60` cadence scaling; Cadence (PAS) linear ramp; Throttle 8-tap filter & `200 mV/step` slew-rate limiter.
    - Stage 3: Arbitrator (`max(Torque, PAS, Throttle)`), `±0.2 A/step` ramp limiter, proportional speed soft-taper (`1.5 km/h` normal / `2.0 km/h` walk-assist), and `3.0 V` low-voltage reserve cushion down to `25%`.
  - [`src/tuning.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/tuning.rs) — **Source-Level Customization Profiles**:
    - `CadenceScaling`: `Stock60Rpm`, `Uncapped { min_cadence_rpm: 5 }` (eliminates low-cadence climbing penalty), or custom `LinearRamp`.
    - `TorqueResponse`: `StockLinear`, `Progressive` (quadratic for trail riding), or `HighSensitivity` (boosted for commuter riding).
    - `AssistCeilingPolicy`: `StockDualCeiling` (torque ignores assist level `Pwr%`) vs. `UnifiedLevelCeiling` (torque respects assist level limit).
    - Built-in presets: `stock_v6510c()`, `uncapped_cadence()`, `progressive_trail()`, `commuter_comfort()`.
  - [`src/foc.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/foc.rs) — Fixed-point `Q15` and Cortex-M4F single-precision FPU (`f32`) Clarke/Park transforms, `AutoCurrentPID`, flux weakening, Hall sensor + PLL velocity/angle observer, and 7-segment SVPWM with `[0.06, 0.94]` dead-time linear duty shaping.
  - [`src/protocol.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/protocol.rs) — Display UART protocol with `0x69` stream XOR cipher and ToSeven CRC-16 verification.
  - [`src/stock.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/stock.rs) — Pinned SHA-256 registry of known official factory `.Pack` releases.
  - [`src/sha256.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/sha256.rs) — Dependency-free `#![no_std]` SHA-256 digest engine.
  - [`src/config.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/config.rs), [`src/sensors.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/sensors.rs), [`src/control.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/control.rs), [`src/thermal.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/thermal.rs), [`src/battery.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/battery.rs), [`src/faults.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/faults.rs) — High-level controller state machines.
  - [`src/pac.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/pac.rs) — Direct memory-mapped peripheral register bindings (`TIM1`, `ADC1/2`, `USART3`, `GPIOA..D`, `SCB`).
  - [`src/hal.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/hal.rs) — Concrete hardware drivers (`Dm01PwmHardware` for 16 kHz complementary PWM with dead-time, `Dm01SensorsHardware`).
  - [`src/bin/firmware.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/bin/firmware.rs) — Standalone bare-metal Cortex-M4F firmware entry point, vector table (MSP `0x20005628`), and FOC control loop.
  - [`memory.ld`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/memory.ld) & [`build.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/build.rs) — Cortex-M4F Flash (`0x08001800`) / SRAM (`0x20000000`) linker script with automated cargo search configuration.
  - [`src/bin/toseven_pack.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/bin/toseven_pack.rs) — CLI utility (`toseven_pack`) to inspect, unpack, repack, create (directly from ELF or raw binary), objcopy, and verify `.Pack` firmware containers.
- [`blobs/`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/blobs/README.md) — External proprietary assets (.gitignore'd for clean-room legal safety):
  - `blobs/firmware/` (`dm01/`, `dm02/` stock `.Pack` files)
  - `blobs/displays/` (hex firmware dumps for `T24V`, `T154`, `M15`, `LCD135C`)
  - `blobs/apps/` (Android tuning APK)
  - `blobs/manuals/` (PDF user manuals)
  - Fully cataloged with SHA-256 hashes in [`blobs/README.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/blobs/README.md).
- `Documentation/`
  - [`DM01_Power_Curve_Guide.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/Documentation/DM01_Power_Curve_Guide.md) — Human-readable guide explaining how the DM01 determines power without memory-index jargon.
  - [`Official_Specifications.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/Documentation/Official_Specifications.md) — Hardware specifications and official manual links.
  - [`Research_Downgrade_and_Update.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/Documentation/Research_Downgrade_and_Update.md) — Firmware update/downgrade and container notes.
  - `Miraheze_Wiki/` — Local archive of all 14 pages from the ToSeven Miraheze Wiki.

---

## Decompiled `V6510C` Binary Architecture Summary

### 1. Container & Encryption Layers
| Layer | DM01 (`TO7-DM01-K260521A-V6510C.Pack`) | DM02 (`TO7-DM02-K260526-V6510C.Pack`) |
| :--- | :--- | :--- |
| **Physical Rating** | External 12-FET (750W / 1000W) | Internal integrated (250W / 350W / 500W) |
| **Device ID (`header[0..3]`)** | `0x196EA4` | `0x196EA3` |
| **Header XOR Key (`pack[5]`)** | `0xA8` (34 bytes `0x00..0x21`) | `0xF7` (34 bytes `0x00..0x21`) |
| **Payload XOR Key** | `0x67` (`57,708` bytes `0x22..0xE18D`) | `0xE2` (`58,440` bytes `0x22..0xE469`) |
| **Flash Load Address** | `0x08001800..0x0800F96C` | `0x08001800..0x0800FC48` |
| **Initial MSP / `Reset_Handler`** | `0x20005628` / `0x08001A51` | `0x20005620` / `0x08001A51` |
| **Factory EEPROM Image (480 B)** | `0x0800EE70` (`"TS1000W"`) | `0x0800F142` (`"TS500W"`) |
| **Flash EEPROM Storage (`0x0801FC00`)** | 5 blocks (`0x20, 0x40, 0x40, 0xA0, 0xA0`), XOR `0xA5`, magic `A5 01 FE`, 16-entry nibble CRC-16 | Identical 5-block format |
| **Display UART Stream Cipher** | Byte-wise XOR `0x69` | Byte-wise XOR `0x69` |

### 2. Decompiled Power Curve Rules (Human-Readable Summary)
1. **Dual Current Ceilings (Why `Pwr%` and `EAssist%` Are Separate):**
   - **Global Mode Ceiling (Used by Torque Sensor):** $\min(\text{Mode Limit}, \text{Linear Thermal Ceiling}, \text{50V Constant-Power Cap})$. Ignores the assist level's `Pwr%` setting.
   - **Level Power Ceiling (Used by Cadence PAS & Throttle):** Scales the Mode Limit by `Pwr%` before applying the thermal and high-voltage caps.
2. **Linear Thermal Rollback (`70°C` to `100°C`):**
   - Tapers linearly from `100%` of the **Sport Mode ceiling (`30.0 A`)** at `70°C` down to `0.0 A` at `100°C`.
3. **High-Voltage Constant-Power Cap (Above `50.0 V`):**
   - Scales maximum current by $50.0\text{ V} / V_{\text{battery}}$ above `50.0 V` so peak electrical power stays constant ($1500\text{ W}$ in Sport Mode).
4. **Torque Sensor Curve:**
   - Tracks resting `ZeroVol` thermal drift via a 5-sample delta window (`< 20 mV` over 50 idle cycles) and updates `ZeroVol` with a `4:1` weighted average inside `400..1000 mV`.
   - Applies configurable IIR smoothing ($F = \min(20 \times \text{Filter}, 200)$), normalizes effort above `50 mV`, multiplies by $\min(3 \times \text{StrengthValue}, 30) \times \text{EAssist\%} / 1000$ (capped at `100%`), adds `CurrentOffset` (`0.5 A`), and scales linearly by crank cadence relative to **`60 RPM`** ($\max(\text{Cadence}, 5) / 60$), clamped to the **Global Mode Ceiling**.
5. **Arbitration, Smooth Ramping, Speed Soft-Taper & LVC Reserve Cushion:**
   - Selects $\max(\text{Torque}, \text{PAS}, \text{Throttle})$ when pedaling forward, ramps current at `±0.2 A/tick`, linearly tapers current across a `1.5 km/h` window below the speed limit, and linearly tapers current down to `25%` across a `3.0 V` reserve cushion above Low-Voltage Cutoff (`LVC`).

---

## Common Build & Verification Commands
```bash
# Run all unit and integration tests
cargo test --manifest-path dm01-firmware/Cargo.toml

# Check strict formatting and clippy lints
cargo fmt --manifest-path dm01-firmware/Cargo.toml -- --check
cargo clippy --manifest-path dm01-firmware/Cargo.toml --all-targets -- -D warnings

# Verify strict #![no_std] embedded compilation
cargo check --manifest-path dm01-firmware/Cargo.toml --no-default-features --lib

# Build standalone bare-metal Cortex-M4F ELF binary
cargo build --target thumbv7em-none-eabihf --manifest-path dm01-firmware/Cargo.toml --no-default-features --features embedded --bin dm01_firmware --release

# Extract raw Flash binary from ELF
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- objcopy dm01-firmware/target/thumbv7em-none-eabihf/release/dm01_firmware build/dm01_firmware.bin

# Package bare-metal firmware directly from ELF or raw binary into official 128 KB .Pack container
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- create dm01 dm01-firmware/target/thumbv7em-none-eabihf/release/dm01_firmware build/TO7-DM01-OpenFirmware-v0.3.0.Pack

# Inspect generated .Pack container
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- info build/TO7-DM01-OpenFirmware-v0.3.0.Pack

# Inspect and verify DM01 and DM02 stock .Pack containers
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- verify "blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack"
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- verify "blobs/firmware/dm02/TO7-DM02-K260526-V6510C.Pack"
```