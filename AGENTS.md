# Project Notes & Architecture Reference (`AGENTS.md`)

## Workspace Structure
- `dm01-firmware/` — Clean-room `no_std` Embedded Rust DM01 motor controller firmware crate:
  - `src/config.rs` — User-configurable parameters (`MotorConfig`) & community tuning presets (`preset_en15194`, `preset_jonas_dm01`, `preset_eight_level_progressive`)
  - `src/sensors.rs` — Strain-gauge torque sensor with dynamic zero-point thermal drift compensation (`TorqueSensorState`) and Hall throttle parser
  - `src/battery.rs` — 6-point voltage-to-SoC calibration curve (`36V`/`48V`/`52V`/`60V`/`72V`) and debounced low-voltage cutoff (`BatteryMonitor`)
  - `src/thermal.rs` — Controller thermal derating and `100°C` 30% rollback (`ThermalMonitor`)
  - `src/faults.rs` — Startup torque calibration timeout, `MotorBlocked` stall detection, and sensor plausibility checks (`FaultDetector`)
  - `src/control.rs` — Periodic control loop (`Dm01Controller`) combining PAS, torque, throttle modes, shift cutoffs, and smooth speed tapering
  - `src/foc.rs` — Fixed-point `Q15` Clarke/Park transforms, 4th-order parabolic trig, anti-windup PI controllers, Hall angle decoder, and 7-segment SVPWM (`FocCurrentLoop`)
  - `src/protocol.rs` — UART telemetry frame serializer/deserializer with CRC-16-CCITT verification (`TelemetryFrame`)
  - `src/hal.rs` — Hardware abstraction traits (`ThreePhasePwm`, `MotorSensorsHal`, `ConfigStorageHal`)
- `DM01 Sources/` — DM01 firmware packages
- `DM02 Sources/` — DM02 firmware packages
- `Display Sources/` — Display firmware packages (`T24V`, `T154`, `M15`, `LCD135C`)
- `Documentation/`
  - `Official_Specifications.md` — Hardware specifications and official manual links scraped from `to7motor.com`
  - `Miraheze_Wiki/` — Complete local archive of all 14 pages from the ToSeven Miraheze Wiki (`Main_Page`, `Firmware`, `Display_Settings`, `General_FAQs`, `Installation_and_bike_fit`, `Chains__chainrings_and_chain_lines`, `Accessories`, `Batteries`, `Repairs_and_maintenance`, `Troubleshooting`, `Example_Setups`, `Other_resources`, `Sellers`, `Easter_egg`)
  - `Manuals/` — Official PDF user manuals

---

## Hardware & Control Parameters Summary

### DM01 vs. DM02 Key Differences
| Feature | DM01 (750W / 1000W) | DM02 (250W / 350W / 500W) |
| :--- | :--- | :--- |
| **Controller Location** | External | Internal |
| **Reduction Ratio** | 40.27 : 1 | 53 : 1 |
| **Max Torque** | 160 N·m | 90 N·m |
| **Max Cadence** | 160 RPM | 100 RPM |
| **Default Current (Sport, 48V)** | 30 A | 14.2 A (up to 18 A on earlier revisions) |
| **Shift Sensor Support** | Hardware connector included | Software back-pedal / torque drop detection |

### Sensor & Control Subsystems
1. **Torque Sensor:**
   - Strain gauge (Wheatstone bridge) on the bottom bracket mechanism.
   - Idle resting voltage: `600–700 mV` (`0.6–0.7 V`), scaling up to `3300 mV` (`3.3 V`) under maximum pedal force.
   - Configurable parameters include `ZeroVol(mV)`, `StartupVol(mV)`, `CurrentOffset(A)`, `Filter` (ramp time), `StrengthValue`, `SpeedMode`, and `KeepTime(ms)`.
   - Newer firmware revisions implement dynamic zero-point tracking when pedal pressure is released to compensate for thermal voltage drift.
2. **Cadence / PAS Sensor:**
   - Configurable parameters include `StartupCount`, `KeepTime(ms)` (200–1000 ms), `Strength(rpm)`, `SpeedMode`, `MaxSpeed(rpm)`, and `CurrentOffset(A)`.
3. **Throttle:**
   - Hall-effect input (`0.7 V` resting to `3.3 V` full throttle).
   - Supports 13 operating modes (`SpeedMode 0–12`) governing interaction between pedaling state (PAS active vs. inactive) and speed/power limits.
4. **Thermal Protection:**
   - Controller temperature sensor (`MinTemp` default `70°C`, `MaxTemp` default `100°C`).
   - Above `100°C`, power rolls back to ~30% until temperature drops below threshold.
5. **Assist Modes & Levels:**
   - 3 global current modes (`Eco`, `City`, `Sport`) defining `MaxCurrent` limits.
   - 5 (configurable up to 8) assist levels defining per-level `Spd`, `Pwr` (%), and `EAssist` (%) curves.

---

## Firmware Language & Architecture Notes (Rust vs. C)

### Embedded C
- **Strengths:** Direct compatibility with vendor motor-control SDKs, fixed-point `IQmath` libraries, and existing open-source e-bike firmware architectures (e.g., VESC, TSDZ2 OSF).
- **Best For:** Rapidly bringing up vendor-specific FOC libraries or hardware-specific peripheral blocks when community Rust PAC/HAL support for a specific microcontroller is incomplete.

### Embedded Rust (`no_std`)
- **Strengths:** Compile-time safety for state machines (assist level transitions, thermal rollback, fault states, calibration routines), zero-cost concurrency auditing via **RTIC (Real-Time Interrupt-driven Concurrency)**, and fixed-point arithmetic via the `fixed` crate.
- **Recommended Hybrid/Rust Architecture:**
  - **Hard Real-Time Layer (High-Priority ISRs):** ADC sampling, Clarke/Park transforms, PI current controllers, and Space Vector PWM (SVPWM) updates running deterministically at PWM frequency.
  - **Control Loop Layer (Medium-Priority Periodic Task, e.g., 100–1000 Hz):** Torque sensor filtering, dynamic zero-point drift compensation, cadence calculation, ramp limiting, and thermal derating.
  - **Communication & UI Layer (Low-Priority Task):** UART packet framing, CRC verification, display telemetry, and parameter persistence in flash/EEPROM.

