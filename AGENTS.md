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
  - [`src/flasher.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/flasher.rs) — Motor controller IAP flashing state machine (38400 Baud factory bootloader default with 115200 fallback, 21-byte wakeup packet encoder at 9600 Baud with `uart_mask` nibbles, separated `0x06` prep + 12-byte header frame builder with `uart_mask`, 16-byte chunk with Tier 2 wire stream XOR cipher `b ^ uart_mask`, 1 KB sector pacing, and Step 5 EOT `0x04` / NAK `0x15` verification), and MegaHunt display ISP framing & session state machine (`0x02`, CMD, len, CRC-16-CCITT).
  - [`src/serial.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/serial.rs) — Native cross-platform serial transport supporting POSIX termios (macOS/Linux) with `IOSSIOSPEED` custom-baud ioctl and Win32 COM port engine (`QueryDosDeviceW`, 64-bit sign-extended registry `SERIALCOMM`, `CreateFileW` probe fallback across `COM1..COM32`, non-blocking `COMMTIMEOUTS`, DCB line configuration), hardware line assertions (`DTR`/`RTS`), live IAP/ISP session execution, and universal adapter enumeration (`/dev/cu.*`, `/dev/ttyUSB*`, `COM1..COM255`).
  - [`src/display_patcher.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/display_patcher.rs) — Display firmware Intel HEX parser, vector table validator, automated 3-gate post-October 2024 lockout detector, and ARM Thumb NOP (`0x00BF`) clean-room patcher for `T24V`, `T154`, `LCD135C`, and `M15`.
  - [`src/pac.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/pac.rs) — Direct memory-mapped peripheral register bindings (`TIM1`, `ADC1/2`, `USART3`, `GPIOA..D`, `FLASH`, `IWDG`, `SCB`) and embedded Flash controller drivers.
  - [`src/bin/firmware.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/bin/firmware.rs) & [`src/bin/firmware_dm02.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/bin/firmware_dm02.rs) — Standalone bare-metal Cortex-M4F open firmware targets (`dm01_firmware` MSP `0x20005628` and `dm02_firmware` MSP `0x20005620`), speaking standard 18-byte display protocol, using Page 127 Flash EEPROM persistence, 256-sample startup shunt auto-zero calibration, TIM1 break trip protection, with gear-safe progressive knee, sensorless speed fallback, smooth FOC, and thermal rollback.
  - [`memory.ld`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/memory.ld) & [`build.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/build.rs) — Cortex-M4F Flash (`0x08001800`) / SRAM (`0x20000000`) linker script with automated cargo search configuration.
  - [`src/bin/toseven_pack.rs`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm01-firmware/src/bin/toseven_pack.rs) — Unified CLI utility (`toseven_pack`) to inspect, unpack, repack, create (directly from ELF or raw binary), objcopy, verify, and flash motor controller `.Pack` containers, as well as inspect (`display info`), permanently unlock (`display patch`), and flash (`display flash [--unlock]`) display `.hex` firmware.
- [`dm-plus-gui/`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/dm-plus-gui/Cargo.toml) — Native cross-platform desktop studio (`dm-plus-gui`) built with Slint and Rust, featuring embedded DM01/DM02 open firmware containers, automatic model detection, fixed-height compliance and electrical banners, standardized Libadwaita corner radii, bidirectional slider reactivity, Windows `#![windows_subsystem = "windows"]` console suppression, 1200 ms dynamic port auto-refresh, manual refresh controls, and offline display HEX patch export.
- [`blobs/`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/blobs/README.md) — External proprietary assets (.gitignore'd for clean-room legal safety):
  - `blobs/firmware/` (`dm01/`, `dm02/` stock `.Pack` files)
  - `blobs/displays/` (hex firmware dumps for `T24V`, `T154`, `M15`, `LCD135C`)
  - `blobs/apps/` (Android tuning APK)
  - `blobs/programming/` (`TO7-G2 IAP Tool Ver 5.2.exe`, `MH2020C_ISP.exe`)
  - `blobs/manuals/` (PDF user manuals)
  - Fully cataloged with SHA-256 hashes in [`blobs/README.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/blobs/README.md).
- `Documentation/`
  - [`Firmware_Flashing_and_Recovery_Guide.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/Documentation/Firmware_Flashing_and_Recovery_Guide.md) — Step-by-step flashing, unbricking, and recovery runbook for DM01/DM02 motor controllers and displays.
  - [`Forensic_Firmware_Gap_Analysis_V6510C_vs_Open_Firmware.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/Documentation/Forensic_Firmware_Gap_Analysis_V6510C_vs_Open_Firmware.md) — Exhaustive comparative forensic analysis between stock factory `V6510C` Cortex-M4F binary disassembly and DM+ Open Firmware, mapping all 73 vector entries, ISR timings, FOC loops, and hardware parity achievements.
  - [`Bench_Testing_Bringup_Guide.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/Documentation/Bench_Testing_Bringup_Guide.md) — Hardware bring-up runbook, Julet 5-pin programming connector and harness pinouts, 48V @ 1.0A bench safety limits, and step-by-step verification protocol.
  - [`ToSeven_Controller_Depotting_and_Diagnostics_Guide.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/Documentation/ToSeven_Controller_Depotting_and_Diagnostics_Guide.md) — Teardown, thermal depotting (80°C–85°C polyurethane release), and board-level electrical triage (MOSFET diode check, power rails, SWD verification).
  - [`ToSeven_IAP_Flashing_Protocol.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/Documentation/ToSeven_IAP_Flashing_Protocol.md) — Definitive clean-room reverse engineering specification of the Motor Controller IAP protocol (handshake, 21-byte wakeup frame, 12-byte header, 1024-byte sector pacing, Tier 2 wire stream XOR cipher, mathematical proof of container XOR keys) and Display ISP protocol (MegaHunt MH2020C / MH1903).
  - [`ToSeven_Display_Firmware_Architecture.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/Documentation/ToSeven_Display_Firmware_Architecture.md) — Complete reverse-engineering specification of ToSeven displays (`T24V`, `LCD135C`, `M15`, `T154`), 13 throttle modes, button matrix, EEPROM sync, OTA flasher, and post-Oct 2024 menu lockout bypass.
  - [`ToSeven_Bluetooth_and_App_Protocol.md`](file:///Users/alouk/Documents/ToSeven%20Reverse%20Engineering/Documentation/ToSeven_Bluetooth_and_App_Protocol.md) — Reverse-engineering specification of the ToSeven mobile app, BLE GATT profile (`0000FFF0..2`), `0x7E..0x7F` packet framing with `0x3C 0xC3` sync, diagnostic telemetry packets, and `CTLCFG` EEPROM reflection fields.
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
| **Wire Flash Stream Cipher** | Byte-wise XOR `UartMask` (`header[0x17]`, e.g. `0x89` / `0xB5`) | Byte-wise XOR `UartMask` (`header[0x17]`, e.g. `0xA9`) |
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
# Run all unit, integration, and simulation tests (78 tests)
cargo test --manifest-path dm01-firmware/Cargo.toml

# Check strict formatting and clippy lints
cargo fmt --manifest-path dm01-firmware/Cargo.toml -- --check
cargo clippy --manifest-path dm01-firmware/Cargo.toml --all-targets -- -D warnings

# Verify strict #![no_std] embedded compilation
cargo check --manifest-path dm01-firmware/Cargo.toml --no-default-features --lib

# Build standalone bare-metal Cortex-M4F ELF binaries
cargo build --target thumbv7em-none-eabihf --manifest-path dm01-firmware/Cargo.toml --no-default-features --features embedded --bin dm01_firmware --bin dm02_firmware --release

# Extract raw Flash binary from ELF
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- objcopy target/thumbv7em-none-eabihf/release/dm01_firmware build/dm01_firmware.bin
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- objcopy target/thumbv7em-none-eabihf/release/dm02_firmware build/dm02_firmware.bin

# Package bare-metal firmware directly from ELF into official 128 KB .Pack containers
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- create dm01 target/thumbv7em-none-eabihf/release/dm01_firmware build/TO7-DM01-OpenFirmware-v0.3.0.Pack
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- create dm02 target/thumbv7em-none-eabihf/release/dm02_firmware build/TO7-DM02-OpenFirmware-v0.3.0.Pack

# Inspect generated .Pack containers
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- info build/TO7-DM01-OpenFirmware-v0.3.0.Pack
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- info build/TO7-DM02-OpenFirmware-v0.3.0.Pack

# Inspect and verify DM01 and DM02 stock .Pack containers
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- verify "blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack"
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- verify "blobs/firmware/dm02/TO7-DM02-K260526-V6510C.Pack"

# Dry-run simulate IAP packet framing and handshake
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash "blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack" --dry-run
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash "blobs/firmware/dm02/TO7-DM02-K260526-V6510C.Pack" --dry-run

# Live flash firmware over USB-UART adapter (bench power supply recommended)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash /dev/tty.usbserial-110 build/TO7-DM01-OpenFirmware-v0.3.0.Pack

# Inspect display firmware (model, vector table, and lockout status)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display info blobs/displays/t24v/TO7-T24V-Y241225.hex

# Permanently unlock display firmware (neutralizes all 6 restriction branch gates)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display patch blobs/displays/t24v/TO7-T24V-Y241225.hex build/TO7-T24V-Y241225-Unlocked.hex

# Flash display firmware with on-the-fly unlock (dry-run simulation)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display flash blobs/displays/t24v/TO7-T24V-Y241225.hex --unlock --dry-run

# Live flash display firmware over USB CDC port with on-the-fly unlock
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display flash /dev/tty.usbmodem110 blobs/displays/t24v/TO7-T24V-Y241225.hex --unlock

# Pre-flight sensor triage checkout (bench safety check)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- diagnostic --simulate
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- diagnostic /dev/tty.usbserial-110

# Live parameter tuning (SRAM injection without MCU reboot or EEPROM wear)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- tune --sport 28 --city 18 --eco 10 --speed 32 --simulate
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- tune /dev/tty.usbserial-110 --sport 28 --city 18 --eco 10 --speed 32

# Monitor live 0x69-encrypted UART telemetry (or run simulated ride demo)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- monitor --simulate
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- monitor /dev/tty.usbserial-110 --baud 9600

# Run complete comparative simulation suite (Stock V6510C vs. Open Firmware)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- simulate
cargo test --manifest-path dm01-firmware/Cargo.toml --test simulation_suite -- --nocapture

# List connected USB serial adapters
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- ports

# Package local host release artifacts (CLI & GUI) into dist/
./scripts/package-release.sh
```

---

## Documentation & `README.md` Maintenance Policy
To prevent `README.md` from falling behind or drifting out of sync with code and community discussions, all agents must adhere to the following rules:

1. **Brand Identity:** Always refer to the overall open project as **DM+** (e.g., *DM+ Firmware & Tooling Studio*), noting that naming may evolve as support broadens.
2. **Current Implementation First:** Keep the "What Is Implemented Right Now" section up to date with actual working features. Long-term goals belong in the foldable "Project Vision & Future Roadmap" section.
3. **Foldable Structure:** Use HTML `<details><summary>` blocks for large tables, register maps, and extensive CLI references to preserve clean scannability.
4. **No Emojis & No ASCII Interface Maps:** Keep `README.md` clean and professional; do not re-introduce decorative emojis or fragile ASCII UI drawings.
5. **AI Ethics & Accuracy:** Never claim comprehensive manual human line-by-line review of decompiled code. Clearly state the use of AI assistance (Gemini / Claude) along with automated tests, bit-exact roundtrips, and simulation verification, emphasizing the need for community bench testing.
6. **Community Inquiries:** When addressing community feedback (such as torque sensor span calculation, nylon primary reduction gear protection, or battery LVC true 0A cutoff), keep the technical rationale in the "Community Inquiries & Technical Feedback" section aligned with ongoing developments.
7. **Always Synchronize Before Pushes/Tags:** Always update `README.md` to reflect all implemented features, test counts, UI additions, and tag versions prior to creating git tags or pushing to remotes.
8. **Prominent Liability & Warranty Disclaimers:** Maintain clear disclaimers that the software and firmware are provided "AS IS" with NO WARRANTY, and any damage to equipment is solely the user's responsibility.