# Changelog

All notable changes to the **DM+ Open Firmware & Tooling Studio** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> [!NOTE]
> All releases in the `0.3.x` development branch are designated **Experimental Alpha** releases (`v0.3.0-alpha.1` .. `v0.3.0-alpha.9`) to reflect the early-stage, bench-only reverse engineering and firmware bring-up status. Operating custom motor firmware on an actual bicycle is strictly prohibited.

---

## [Unreleased]

### Added
- **Adaptive Zero-Point Torque Drift Tracking & Red Light Drift Lockout:**
  - Implemented decompiled `V6510C` (`0x08009034`) adaptive `ZeroVol` baseline drift tracking in `power_curve.rs`: samples during stationary stops (`cadence == 0`, `max_delta < 20 mV`) are hard-clamped to $[400\text{ mV},\; 1300\text{ mV}]$ and squeezed into a dynamic $\pm 200\text{ mV}$ band around the active baseline.
  - Added a 5-deep start-stop cycle history (`history_zero_volt: [u16; 5]`) updated with the 50-tick quiet sample average on each stop, computing `zero_vol_mv` as the 5-cycle mean clamped to $[400\text{ mV},\; 1300\text{ mV}]$.
  - Added single-shot cycle latch lockout (`zero_volt_tracking_disabled: bool`) that disables further tracking during an ongoing stop until crank movement (`cadence > 0`) is observed, eliminating the "Red Light Drift" bug where resting foot weight corrupts the resting baseline during long stops.
  - Added comprehensive unit tests verifying dynamic banding, low outlier upward pull, hard boundary clamping, and single-shot latch lockout.
- **EggRider-Grade Battery Voltage Sag Compensation (IR) & Hybrid Smoothing:**
  - Implemented dynamic internal resistance voltage sag cancellation in `BatteryMonitor` (`V_est_ocv = V_terminal + (I_load * R_pack) / 1000`), eliminating instantaneous 2V–5V voltage drops and preventing battery percentage yo-yoing under throttle and cadence acceleration.
  - Paired IR compensation with a light $\tau \approx 1.02\text{ s}$ Exponential Moving Average ($\alpha = 1/1024$ at 1 kHz) to filter inverter switching current ripple and electrochemical diffusion relaxation over $\approx 3\text{ seconds}$ without sluggish display lag.
  - Maintained physical terminal voltage monitoring on the 500 ms Low-Voltage Cutoff (LVC) debounce to safeguard battery cells from undervoltage.
  - Persisted `battery_resistance_mohm: u16` (default 150 mΩ) in Block 1 EEPROM zero padding (`0x04A + 8..10`) with automatic 16-entry CRC-16 recalculation.
  - Added *Battery Sag Compensation (IR)* slider (0–350 mΩ) to the DM+ Desktop Studio (`dm-plus-gui`), enabling flash-time EEPROM configuration and automated container checksum updates.
  - Embedded factory EEPROM images directly into bare-metal `dm01_firmware` and `dm02_firmware` Flash payloads, enabling automated self-commissioning on blank or unformatted MCU Flash Page 127.


## [v0.3.0-alpha.9] - 2026-10-25

### Added
- **Reverse Engineering Analysis & Support for October 23, 2026 Factory Releases (DM01 & DM02 V6510C):**
  - **New Firmware Cataloging & Verification:**
    - Decrypted, verified, and cataloged `TO7-DM01-K261023-V6510C.Pack` (SHA-256 `ff557c78825816d70e12dcecf83c9efba9fc4cb5908e4baefd60acee7d97fad2`) and `TO7-DM02-K261023-V6510C.Pack` (SHA-256 `efd792ce43f36dce30bf3a94a3e89bf2be0ee7f95b17b5b15fe32d119b2380de`).
    - Both packages strictly validate against ToSeven universal key derivation formulas: Header XOR key `(pack[32] + pack[33]) & 0xFF`, Payload XOR key `(dec_hdr[30] + dec_hdr[31]) & 0xFF`, and 32-bit cumulative byte sums.
  - **DM02 Architecture & Device ID Subtype (`0x0019_6EE4`):**
    - Identified new DM02 board/firmware identifier `0x0019_6EE4` (`Customer_Num: 0x196E`, `Device_Type: 0xE4`).
    - Updated `pack::MotorTarget::from_device_id()` to map `0x0019_6EE4` as a DM02 target.
    - Disassembled 16-byte code insertion sites (+4 bytes at `0x080067FA` adding an immediate field check before NMI handling, and +12 bytes at `0x08009253` adding a communication retry state loop before USART3).
    - Verified that DM02 factory EEPROM image (480 bytes, 5 blocks) remains 100% bit-exact identical to the May 2026 production baseline, shifting in Flash from `0x0800F142` to `0x0800F152` due to the code insertions.
  - **DM01 Architecture & Factory EEPROM Variations (+824 Bytes):**
    - Disassembled and traced DM01 payload code expansion (+824 bytes, 58,532 B vs. 57,708 B) and stack pointer relocation (MSP raised by 16 bytes from `0x20005628` to `0x20005638`).
    - Decoded 24 EEPROM byte modifications in the October 23 DM01 build, revealing:
      - Speed sensor magnet pulse count updated from 10 to 15 pulses/rev (`0x1D3`: `0x0A` -> `0x0F`).
      - Voltage threshold adjustments in Block 1 (`0x02BC` / 70.0V -> `0x0258` / 60.0V).
      - Multi-mode speed/power state selector updates in Block 3 and Block 4.
      - Embedded Block 1 model string updated to `TS\x11000W`, while retaining the internal vendor template string (`"Sitgo-Rear 1700W"` in Block 2 `0x07D..0x09E` and placeholder serial `24111100001` in `0x06A..0x07B`) present across all factory DM01 releases.
  - **Dynamic EEPROM Discovery & 17-Firmware Historical Compatibility (`V6206C`..`V6510C`):**
    - Upgraded `EepromImage::verify_all_blocks()` and `EepromImage::find_in_payload()` to accept both `A5 01 FE` (`V6212C`..`V6510C`) and `A5 FD 02` (`V6206C` / `V6303C`) Block 1 magic signatures (`raw[0x05A] == 0xA5 && (raw[0x05B] ^ raw[0x05C]) == 0xFF`), achieving **17/17 (100%)** embedded 480-byte EEPROM extraction across all historical `.Pack` releases.
    - Refined `parse_pack_metadata()` to auto-detect `MotorTarget::Dm01` (`TS1000W` / `TS\x11000W`) vs. `MotorTarget::Dm02` (`TS500W`) directly from the embedded EEPROM Block 1 model signature, resolving cross-version header `Device_ID` byte (`dec_hdr[0]`) variations across all 17 factory packs.
    - Added `PackMetadata::firmware_version_number()` returning the decimal firmware version (`6206`, `6212`, `6303`, `6306`, `6400`, `6510`) from `decrypted_header[1..3]`.
    - Added native `V6510C` 7-bit UART display protocol support (`0xC0 | cmd` .. `0x80 | crc_lo` with 3-byte packed `crc16_toseven` trailer, `V6510cDisplayCommandDecoder`, and `TelemetryFrame::encode_v6510c_*` builders) in `protocol.rs` and integrated it into `firmware.rs` and `firmware_dm02.rs`.
    - Updated `hal.rs` and `pac.rs` to match the decompiled `V6510C` GPIO/AFIO/Timer pinout (`AFIO->MAPR` `SWJ_JTAGDisable` + `PD01_REMAP`, `PD0` + `PA15` quadrature PAS cadence, `PA3` external wheel speed sensor, `PC15` headlight output, and `PA2` status/fault LED).
    - Updated `dm01-firmware` stock registry (`stock::KNOWN_RELEASES`) and universal container test suites to cover all 17 known factory releases with 100% bit-exact roundtrips.
- **Critical Flashing Warnings & Potted Bootloader Documentation:**
  - Added prominent warnings in [`README.md`](README.md), [`Firmware_Flashing_and_Recovery_Guide.md`](Documentation/Firmware_Flashing_and_Recovery_Guide.md), and [`Bench_Testing_Bringup_Guide.md`](Documentation/Bench_Testing_Bringup_Guide.md) explaining the software bootloader trap: if custom experimental firmware faults before initializing serial communication, software IAP recovery becomes impossible over the wire harness and requires physical depotting + SWD debugging.
  - Added multi-baud bootloader detection utility [`scripts/bootloader_catcher.py`](scripts/bootloader_catcher.py).

---

## [v0.3.0-alpha.8] - 2026-10-20

### Added
- **Automated Motor Self-Commissioning Engine (`calibration.rs`):**
  - Added low-power (1.5 A) bench commissioning procedure for DM01 and DM02 motors.
  - Implements 0° DC rotor alignment, 120° Hall commutation sequence verification, electrical offset angle calculation (`angle_offset_deg`), phase swap detection, and coil resistance sanity checks.
- **Pre-Flight Sensor Triage (`toseven_pack diagnostic`):**
  - Bench diagnostic checkout command evaluating Hall state validity, resting torque sensor zero-point drift (400..1000 mV), throttle idle ADC ($\le 1050\text{ mV}$ to eliminate runaway hazards), and controller/motor thermals prior to connecting high-current battery power.
- **Live Parameter Tuning (`toseven_pack tune`):**
  - High-speed dynamic parameter injection frame over Julet 5-pin UART.
  - Updates Sport/City/Eco currents, speed limits, and assist ceiling policies within $\le 1\text{ ms}$ on the next FOC tick with zero MCU reboot and zero Flash EEPROM wear.
- **Street-Legal Compliance & Tamper-Proof Lock (EN15194 / EPAC):**
  - Added `StreetLegalDefault` mode: defaults to 25.0 km/h speed cutoff and 250W continuous power limit on boot.
  - **Brake + Power-On Startup Combo:** Holding the brake lever while powering on the bike temporarily switches the controller into Off-Road mode for the active ride session.
  - Added `PermanentLock` mode: compile-time (`--features street_legal_lock`) or EEPROM latch permanently locking the controller to EN15194 limits, capping standalone throttle to 6 km/h walk-assist, and permanently rejecting all boot combos, UART bypasses, and display menu speed adjustments.
- **Controller Depotting & Diagnostic Guide:**
  - Added [`Documentation/ToSeven_Controller_Depotting_and_Diagnostics_Guide.md`](Documentation/ToSeven_Controller_Depotting_and_Diagnostics_Guide.md) covering non-destructive polyurethane potting release (80°C–85°C heat gun), PCB layout, MOSFET diode checks, and SWD pinouts.
- **Bluetooth Low Energy (BLE) & App Protocol Guide:**
  - Added [`Documentation/ToSeven_Bluetooth_and_App_Protocol.md`](Documentation/ToSeven_Bluetooth_and_App_Protocol.md) detailing GATT services (`0000FFF0..2`), `0x7E..0x7F` packet framing with `0x3C 0xC3` sync, diagnostic telemetry packets, and `CTLCFG` EEPROM reflection fields.
- **Mathematical FOC Verification Suite:**
  - Added rigorous unit tests for Clarke transform balanced 3-phase invariance, Park transform pure Q-axis torque alignment and roundtrips, and 120° Hall PLL observer edge detection.
  - Verified SVPWM dead-time clamp ($6.0\%$ to $94.0\%$) guaranteeing $\ge 3.75\text{ µs}$ low-side bootstrap capacitor recharge per 16 kHz cycle.

### Changed
- **Default Open-Firmware Tuning:**
  - Updated default profile (`gear_safe_trail()`) to use `UnifiedLevelCeiling`, ensuring Assist Level 1 strictly respects configured power limits rather than drawing full mode current under hard pedaling.
  - Enabled `GearSafeKnee` by default, protecting the blue nylon primary reduction gear against standing-start stomps below 18 RPM.
- **Hard Current Ceilings & Over-Stock Warnings:**
  - Enforced strict hard current ceilings: 32.0 A absolute cap for DM01 and 20.0 A absolute cap for DM02.
  - Added active UI and firmware warnings whenever current exceeds factory continuous ratings (> 30 A on DM01, > 18 A on DM02).
- **Desktop Studio (`dm-plus-gui`):**
  - Streamlined "Target Hardware & Legal Compliance" card by removing redundant runtime tuning dropdowns, recovering ~150px of vertical height.
  - Added fixed-height Adwaita compliance and electrical banners (56px, `clip: true`), eliminating layout overlap and vertical jumping.
  - Standardized all corner radii to Libadwaita design tokens (12px cards, 8px banners/buttons/alert containers, 9px/6px segmented switcher).
  - Implemented bidirectional Slider reactivity (`value <=> root.max-current-a`), ensuring immediate track repainting and thumb positioning when switching motor targets.
- **Documentation Overhaul:**
  - Streamlined `README.md` by stripping raw hex addresses, memory offsets, and internal math formulas, bringing immediate focus to core user-facing features and safety runbooks.
  - Integrated VESC architecture comparison and 3-pillar pre-flight verification protocol into `Documentation/Bench_Testing_Bringup_Guide.md`.

---

## [v0.3.0-alpha.7] - 2026-10-18

### Added
- Apple HIG-compliant branding assets (high-resolution `.icns`, `.ico`, `.png`) across all desktop GUI builds.
- Native macOS dock icon integration and activation policy enforcement.

### Changed
- Standardized segmented pill switcher with fixed dimensions to prevent cross-tab layout resizing.
- Hardened integer arithmetic across all power curve stages to eliminate overflow risks under extreme battery voltages.

---

## [v0.3.0-alpha.6] - 2026-10-18

### Added
- Windows `#![windows_subsystem = "windows"]` console suppression for clean GUI launching.
- 1200 ms dynamic serial port auto-refresh timer detecting hotplugged and disconnected USB-UART adapters.

---

## [v0.3.0-alpha.5] - 2026-10-18

### Fixed
- Fixed Win32 COM port discovery supporting `QueryDosDeviceW`, registry probing across 64-bit sign-extended values, and `CreateFileW` fallback across `COM1..COM32`.
- Corrected non-blocking `COMMTIMEOUTS` and DCB baud rate line configuration for reliable IAP flashing on Windows.

---

## [v0.3.0-alpha.4] - 2026-10-18

### Added
- Comprehensive hardware bring-up safety runbook ([`Documentation/Bench_Testing_Bringup_Guide.md`](Documentation/Bench_Testing_Bringup_Guide.md)) detailing current-limited bench power supply procedures.
- Initial native desktop studio (`dm-plus-gui`) built with Slint.

---

## [v0.3.0-alpha.3] - 2026-10-18

### Added
- Community-informed torque curve enhancements:
  - `GearSafeKnee`: Soft assist below 18 RPM protecting the nylon primary reduction gear.
  - `AutoSpanTracking`: Dynamic zero-point drift compensation pinning deflection to 2.3 V.
  - `TrueZeroCutoff`: Linear current taper down to exact 0.0 A across a 3.0 V reserve window above LVC.

---

## [v0.3.0-alpha.2] - 2026-10-18

### Added
- Embedded DM01 and DM02 open firmware containers directly in GUI assets.
- Automatic hardware target detection from `.Pack` headers.

---

## [v0.3.0-alpha.1] - 2026-10-18

### Added
- Initial clean-room `#![no_std]` Rust recompilation of the ToSeven motor controller firmware (`dm01-firmware`).
- Complete IAP protocol reverse engineering and bit-exact `.Pack` container repacker (`toseven_pack`).
- MegaHunt ISP display flasher and clean-room NOP lockout patcher for `T24V`, `T154`, `LCD135C`, and `M15`.

