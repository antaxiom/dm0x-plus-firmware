# ToSeven Bluetooth Low Energy (BLE) & Mobile App Protocol Specification

## Executive Summary
This document provides the clean-room forensic reverse-engineering specification of the ToSeven (Minshine) mobile tuning application (`TOSEVEN_Professional_Tuning_v1-1.09_Certified-2025-12-31.apk.1`), its Bluetooth Low Energy (BLE) GATT profile, framing protocol, real-time diagnostic telemetry packet structure, and Flash EEPROM (`CTLCFG`) synchronization mechanism.

---

## 1. BLE GATT Profile & Radio Architecture

### 1.1 Service & Characteristic UUIDs
The companion app scans for and establishes connection with ToSeven Bluetooth-enabled displays (`T24V`, `LCD135C`, `M15`, `T154`) and standalone BLE dongles using the standard 16-bit vendor GATT profile:

| Entity | UUID | Properties | Description |
| :--- | :--- | :--- | :--- |
| **Vendor Service** | `0000FFF0-0000-1000-8000-00805F9B34FB` | Primary Service | Main ToSeven Mid-Drive GATT Service |
| **Write Characteristic (TX)** | `0000FFF1-0000-1000-8000-00805F9B34FB` | Write / Write Without Response | Mobile App -> Display / Motor Controller |
| **Notify Characteristic (RX)** | `0000FFF2-0000-1000-8000-00805F9B34FB` | Notify / Read | Display / Motor Controller -> Mobile App |

- **MTU Size:** Negotiated up to `247` bytes (fallback `23` bytes).
- **Advertisement Local Name:** Matches display model name prefix (e.g. `TO7-T24V-XXXX`, `TO7-DM01-XXXX`, `TO7-DM02-XXXX`).

---

## 2. Framing & Packet Wire Protocol

All communication between the Android app and the bike controller uses framed packets wrapped with `0x7E` (SOF) and `0x7F` (EOF), protected by synchronization headers and checksums.

### 2.1 Packet Format
```text
+------+------+------+------+------+------+------+------+---------------+--------+------+
| 0x7E | LEN  | 0x3C | 0xC3 | ADDR | CMD  | SEQ  | SUB  | PAYLOAD DATA  | CHKSUM | 0x7F |
+------+------+------+------+------+------+------+------+---------------+--------+------+
  [0]    [1]    [2]    [3]    [4]    [5]    [6]    [7]     [8 .. N-2]      [N-1]    [N]
```

- **`0x7E` (SOF):** Start of Frame delimiter.
- **`LEN`:** Total length of the frame including headers and delimiters.
- **`0x3C 0xC3`:** Synchronization Magic Word (`60`, `195`). Every valid packet contains this 2-byte signature.
- **`ADDR`:** Target device address (`0x00` = Broadcast, `0x01` = Display, `0x02` = Motor Controller).
- **`CMD`:** Command ID.
- **`SEQ`:** Packet sequence counter.
- **`PAYLOAD DATA`:** Command-specific payload.
- **`CHKSUM`:** 8-bit checksum: `Sum(bytes[1..N-2]) & 0xFF` (or ToSeven nibble CRC-16 depending on command).
- **`0x7F` (EOF):** End of Frame delimiter.

---

## 3. Core Command Catalog

### 3.1 Polling & Telemetry Requests
The mobile app runs a background timer thread (`GlobalVar$1`) that alternates between two high-frequency polling requests:

1. **`cmd3` (General Ride Telemetry Poll):**
   ```text
   7E 03 3C C3 00 0C 01 8C 52 01 ED 7F
   ```
   - Requests real-time speed, battery voltage, current, assist gear, odometer, and trip time.

2. **`cmd50` (Engineering Sensor Diagnostic Poll):**
   ```text
   7E 32 3C C3 00 0C 01 BB 52 02 4B 7F
   ```
   - Requests raw ADC sensor voltages (Torque mV, Throttle mV, Hall states, temperatures).

### 3.2 Real-Time Control Commands
- **Increase Assist Level (`incBtnClick`):**
  ```text
  7E 0B 3C C3 00 0D 01 95 80 [TargetGear] [CRC] 7F
  ```
- **Decrease Assist Level (`decBtnClick`):**
  ```text
  7E 0B 3C C3 00 0D 01 95 [TargetGear] [CRC] 7F
  ```
- **Lock / Anti-Theft Engage (`btnLock`):**
  ```text
  7E 15 3C C3 00 0D 01 9F 5A A5 02 C0 7F
  ```
  - Sends the `5A A5 02 C0` anti-theft immobilizer payload. The motor controller disables PWM gate drive immediately and enters low-power sleep.
- **Password Query (`getPwdHex`):**
  ```text
  7E 01 3C C3 00 11 01 8F [PasswordChars] [CRC] 7F
  ```
- **Password Set (`getSetPwdHex`):**
  ```text
  7E 1E 3C C3 00 11 01 AC [NewPasswordChars] [CRC] 7F
  ```

---

## 4. Extended Sensor Telemetry Packet (`SENSORDATA`)

When the controller responds to `cmd50`, it returns a 64-byte diagnostic frame that the Android app unpacks into `cn.minshine.ebike.SensorData`:

| Byte Offset | Data Type | Field Name | Unit / Scaling | Description |
| :--- | :--- | :--- | :--- | :--- |
| **`[26..28]`** | `uint24` | `PedalSensor` | RPM / tick delta | Crank cadence measured by optical/Hall PAS disc. |
| **`[29..30]`** | `uint16` (LE) | `TorqueSensor` | Millivolts (mV) | Raw ADC voltage from bottom bracket strain gauge (`400..3300 mV`). |
| **`[31..32]`** | `uint16` (LE) | `Throttle` | Millivolts (mV) | Raw ADC voltage from handlebar throttle (`800..3300 mV`). |
| **`[33]`** | `uint8` | `Gear` | Assist Level | Active assist level (`0..=5` or `0..=8`). |
| **`[44]`** | `uint8` (Bitmask) | `HallSiftSensor` | Binary flags | Discrete hardware digital inputs: |
| | | *Bit 0:* | Hall A state | Motor rotor Hall sensor A (`PB6`). |
| | | *Bit 1:* | Hall B state | Motor rotor Hall sensor B (`PB7`). |
| | | *Bit 2:* | Hall C state | Motor rotor Hall sensor C (`PB8`). |
| | | *Bit 3:* | Shift Sensor 1 | Shifting cut-off switch 1. |
| | | *Bit 4:* | Shift Sensor 2 | Shifting cut-off switch 2. |
| **`[45]`** | `int8` | `MotorTemp` | Degrees Celsius (°C) | Internal motor winding thermistor (`NTC 10K`). |
| **`[46]`** | `int8` | `ControllerTemp`| Degrees Celsius (°C) | MOSFET heatsink thermistor (`PB1`). |
| **`[47..48]`** | `uint16` (LE) | `MotorSpeed` | Electrical RPM (ERPM) | Motor rotor electrical rotational speed. |
| **`[49..50]`** | `uint16` (LE) | `Voltage` | 0.1 V ($V \times 10$) | Battery terminal voltage under load. |
| **`[51..52]`** | `uint16` (LE) | `PhaseCurrent` | 0.1 A ($A \times 10$) | Peak inverter phase current ($I_a / I_b$). |
| **`[53..54]`** | `uint16` (LE) | `Current` | 0.1 A ($A \times 10$) | DC rail battery current ($I_{dc}$). |
| **`[55..56]`** | `uint16` (LE) | `Speed` | 0.1 km/h | Speedometer reading from wheel rim magnet (`PA15`). |
| **`[57..58]`** | `uint16` (LE) | `Trip` | 0.1 km | Trip distance since last reset. |
| **`[59..62]`** | `uint32` (LE) | `Kms` (ODO) | 0.1 km | Cumulative lifetime vehicle odometer. |
| **`[63]`** | `uint8` | `LightStatu` | Bit 0: Light ON/OFF | Active headlight relay status. |

---

## 5. Flash EEPROM Object (`CTLCFG`) Serialization

The Android app represents the motor controller's entire 480-byte Flash Page 127 configuration block as a Java class: `cn.minshine.ebike.CTLCFG`.

- **Field Structure:** Contains 187 fields (`setting_0` through `setting_186`).
- **Ordering Annotation:** Uses `@FiledOrderAnnotation(order = X)` on each field to guarantee strict sequential binary layout identical to the C `struct` compiled into the ARM Cortex-M4F firmware.
- **Key Settings Mapping:**
  - `setting_74`: Max current limit / Sport power ceiling (integer Amperes).
  - `setting_75`: Assist level count selector (`3`, `5`, or `9` levels).
  - `setting_80`: Shift sensor cutoff toggle (enabled/disabled).
  - `setting_117`: Unit selector (`0` = Metric km/h, `1` = Imperial mph).
  - `setting_125`: Wheel diameter selector in 0.1-inch increments (e.g. `275` = 27.5").
  - `setting_126`: Speed limit bypass / European 25 km/h limiter toggle.
  - `setting_130`: Cruise control enable switch.
  - `setting_16..setting_20`: Assist level power tables (`Pwr%` 0..100%).
  - `setting_106..setting_107`: Assist level effort multipliers (`EAssist%` 0..100%).
  - `setting_67`: 2D calibration matrix for torque sensor voltage curve.

---

## 6. Over-The-Air (OTA) Firmware Flashing Protocol

The Android app embeds a complete OTA bootloader client (`cn.minshine.ebike.FirmwareActivity` and `cn.minshine.ebike.utils.IAP_HEAD_STR`):

1. **Handshake:** Sends `Send_IAPack_ToFlash_Cmd` with `IAP_HEAD_STR`:
   - Validates `CtlHardwareNum` against connected motor hardware.
   - Verifies `ProgramSize`, `ProgramChkSum`, and `UartMask`.
2. **Chunk Pacing:** Slices `.Pack` payload into 16-byte chunks streamed over BLE characteristic `0000FFF1`.
3. **Sector Verification:** Pauses every 1024 bytes (1 sector) for Flash write and CRC confirmation.
4. **Completion:** Issues reboot command upon complete Flash write.
