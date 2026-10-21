# Changelog

All notable changes to the **DM+ Open Firmware & Tooling Studio** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [v0.3.0-beta.8] - 2026-10-20

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

## [v0.3.0-beta.7] - 2026-10-18

### Added
- Apple HIG-compliant branding assets (high-resolution `.icns`, `.ico`, `.png`) across all desktop GUI builds.
- Native macOS dock icon integration and activation policy enforcement.

### Changed
- Standardized segmented pill switcher with fixed dimensions to prevent cross-tab layout resizing.
- Hardened integer arithmetic across all power curve stages to eliminate overflow risks under extreme battery voltages.

---

## [v0.3.0-beta.6] - 2026-10-18

### Added
- Windows `#![windows_subsystem = "windows"]` console suppression for clean GUI launching.
- 1200 ms dynamic serial port auto-refresh timer detecting hotplugged and disconnected USB-UART adapters.

---

## [v0.3.0-beta.5] - 2026-10-18

### Fixed
- Fixed Win32 COM port discovery supporting `QueryDosDeviceW`, registry probing across 64-bit sign-extended values, and `CreateFileW` fallback across `COM1..COM32`.
- Corrected non-blocking `COMMTIMEOUTS` and DCB baud rate line configuration for reliable IAP flashing on Windows.

---

## [v0.3.0-beta.4] - 2026-10-18

### Added
- Comprehensive hardware bring-up safety runbook ([`Documentation/Bench_Testing_Bringup_Guide.md`](Documentation/Bench_Testing_Bringup_Guide.md)) detailing current-limited bench power supply procedures.
- Initial native desktop studio (`dm-plus-gui`) built with Slint.

---

## [v0.3.0-beta.3] - 2026-10-18

### Added
- Community-informed torque curve enhancements:
  - `GearSafeKnee`: Soft assist below 18 RPM protecting the nylon primary reduction gear.
  - `AutoSpanTracking`: Dynamic zero-point drift compensation pinning deflection to 2.3 V.
  - `TrueZeroCutoff`: Linear current taper down to exact 0.0 A across a 3.0 V reserve window above LVC.

---

## [v0.3.0-beta.2] - 2026-10-18

### Added
- Embedded DM01 and DM02 open firmware containers directly in GUI assets.
- Automatic hardware target detection from `.Pack` headers.

---

## [v0.3.0-beta.1] - 2026-10-18

### Added
- Initial clean-room `#![no_std]` Rust recompilation of the ToSeven motor controller firmware (`dm01-firmware`).
- Complete IAP protocol reverse engineering and bit-exact `.Pack` container repacker (`toseven_pack`).
- MegaHunt ISP display flasher and clean-room NOP lockout patcher for `T24V`, `T154`, `LCD135C`, and `M15`.
