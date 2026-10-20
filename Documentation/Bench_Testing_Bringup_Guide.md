# ToSeven DM01 / DM02 Hardware Bench Testing & Bring-up Runbook

## 1. Overview & Safety Rules

This runbook defines the official pre-flight bench testing protocol for **ToSeven DM01 (1000W / 750W)** and **DM02 (500W)** mid-drive motor controllers and displays prior to on-bike installation.

> [!CAUTION]
> ### CRITICAL SAFETY WARNING: BENCH TEST ONLY
> **OPEN-SOURCE FIRMWARE MUST NOT BE INSTALLED ANYWHERE OTHER THAN A CONTROLLED BENCH TEST.**
> Do NOT install open-source firmware on an assembled bicycle or attempt to ride with it at this stage. Open-source firmware is strictly intended for isolated bench bring-up, phase verification, and current-clamped diagnostics.
>
> **Bench Power Safety Rules:**
> 1. **Never use a high-discharge e-bike lithium battery for initial bench flashing.** E-bike batteries can deliver >100A instantly during a fault, vaporizing PCB traces and silicon. Always use a current-limited DC bench power supply.
> 2. **Current Limit Clamped to 1.0A:** During flashing and logic verification, strictly clamp the bench supply to **1.0A at 48.0V** (or 52.0V). Idle controller logic consumes only ~40–60 mA. A 1.0A clamp prevents MOSFET destruction in the event of phase shoot-through or improper gate drive.
> 3. **3.3V Logic Levels Only:** The Cortex-M4F controller UART lines operate at **3.3V TTL**. Connecting a 5V UART adapter directly or shorting to VBat will immediately destroy the MCU's USART3 transceiver pins.

---

## 2. Hardware & Tool Requirements

1. **Adjustable DC Bench Power Supply:** 0–60V, minimum 0–3A, with Constant Voltage (CV) and Constant Current (CC) limiting.
2. **USB-to-UART Serial Adapter:** High-speed FTDI (FT232R/FT232H), CP2102, or CH340 with **3.3V logic level jumper set**.
3. **ToSeven Programming Harness:**
   - 5-pin Julet/Higo connector (female) branching to USB-UART (TX, RX, GND) and power toggle (VBat, Key).
   - Or a test breakout clip connected to the main display harness.
4. **Digital Multimeter:** For measuring logic rail voltages and verifying zero ohms to ground.
5. **Host Computer:** Running macOS, Linux, or Windows with `toseven_pack` CLI and `DM+ Firmware Studio` GUI.

---

## 3. Wiring & Harness Pinout

### 5-Pin Display Connector Pinout (Julet / Higo Mini-B)
Looking into the connector face:

```
        Top Notch
       ┌─────────┐
    1 (●)       (●) 2
   3 (●)         (●) 4
         (●) 5
       └─────────┘
```

| Pin | Wire Color | Signal Name | Description | Bench Connection |
| :--- | :--- | :--- | :--- | :--- |
| **1** | Red | **VBat** | Full Battery Voltage (36V–58.8V) | Bench Supply **Positive (+)** |
| **2** | Blue / Orange | **Lock / Key-In** | Power Enable / Ignition line | Connected to **VBat** via SPST Toggle Switch |
| **3** | Black | **GND** | System Ground & Common Reference | Bench Supply **Negative (-)** AND USB-UART **GND** |
| **4** | Green | **Controller TX** | 3.3V UART Output from Controller | USB-UART **RXD** pin |
| **5** | Yellow | **Controller RX** | 3.3V UART Input to Controller | USB-UART **TXD** pin |

> [!WARNING]
> **Common Ground Reference:** Ensure the USB-UART **GND** is tied directly to Pin 3 (**GND**). Do NOT float the serial ground reference, or UART framing errors will occur.

---

## 4. Bench Power Supply Setup

1. **Preset Supply Before Connecting:**
   - Turn on the bench power supply with output **disabled**.
   - Set Voltage: **48.0 V** (or 52.0 V if testing a 14S setup).
   - Set Current Limit: **1.0 A** max.
2. **Verify Polarization:**
   - Confirm Red lead connects to Pin 1 (VBat).
   - Confirm Black lead connects to Pin 3 (GND).
   - Leave the Key toggle switch **OFF** (open).
3. **Enable Supply Output:**
   - Turn output ON. Current draw should be **0.000 A** with Key switch open.
4. **Engage Key Switch:**
   - Close the Key toggle switch (connecting Pin 2 to Pin 1).
   - Expected current draw: **40 mA to 65 mA** at 48.0V as the internal buck converter initializes and powers the Cortex-M4F MCU.
   - If current immediately hits the **1.0A limit**, immediately disconnect — this indicates a short circuit on the VBat line or a blown primary buck inductor.

---

## 5. Step-by-Step Bring-up & Verification Procedure

```mermaid
flowchart TD
    A["1. Current-Limited Bench Power (48V @ 1.0A)"] --> B["2. Engage Key Switch (40-65 mA idle)"]
    B --> C["3. Probe USB Serial Port (toseven_pack ports)"]
    C --> D["4. Dry-Run Verification (toseven_pack flash ... --dry-run)"]
    D --> E["5. Live IAP Flashing (toseven_pack flash ...)"]
    E --> F["6. Telemetry Verification (toseven_pack monitor ...)"]
    F --> G["7. Low-Current No-Load Throttle Spin (Clamped to 1.5A)"]
```

### Step 1: Detect Serial Adapter
Plug in the USB-UART adapter and list available ports:
```bash
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- ports
```
Expected output:
```
Detected 1 serial port(s):
  [1] /dev/tty.usbserial-110 (USB-Serial Controller)
```

### Step 2: Validate Firmware Container
Before flashing, verify the `.Pack` file integrity and vector table:
```bash
# Verify Open Firmware container
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- info build/TO7-DM01-OpenFirmware-v0.3.0.Pack
```
Expected output:
```
ToSeven .Pack Firmware Container:
  Target Device:      DM01 Mid-Drive Motor Controller (ID: 0x00196EA4)
  Target Rating:      External 12-FET Inverter (750W / 1000W)
  Flash Load Address: 0x08001800
  Initial MSP:        0x20005628
  Reset Handler:      0x08001A51
  Header Key:         0xA8
  Payload Key:        0x67
  Payload Size:       57,708 bytes
  Status:             Valid Cortex-M4F Vector Table & Checksums OK
```

### Step 3: Dry-Run Handshake & Sector Simulation
Run a safe dry-run simulation that validates the handshake packet encoding and sector chunking:
```bash
# Via CLI:
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash /dev/tty.usbserial-110 build/TO7-DM01-OpenFirmware-v0.3.0.Pack --dry-run

# Or via DM+ Firmware Studio GUI:
Click [ Test Flash (Dry Run) ]
```

### Step 4: Live IAP Firmware Flashing
Execute the live hardware flash:
```bash
# Via CLI:
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash /dev/tty.usbserial-110 build/TO7-DM01-OpenFirmware-v0.3.0.Pack

# Or via DM+ Firmware Studio GUI:
Click [ Flash Firmware ]
```
Expected sequence:
1. Flasher listens at 38400 Baud (vendor factory default, with 115200 fallback) for bootloader `IDENT` (`0x02, 0xFD` or `0x06`).
2. If in application mode, flasher transmits 21-byte encrypted wakeup frame at 9600 Baud (`0xC9, 0x77`, 7-bit encoded, `UartMask` high/low nibbles, ToSeven CRC-16, `0x69` cipher) and asserts DTR reset.
3. Flasher transmits `0x06` ACK, pauses 10 ms, and transmits 12-byte header frame (size, sum, `UartMask`, baud rate index), receiving `0x06` ACK.
4. Flasher streams payload in 16-byte chunks masked with Tier 2 wire stream XOR cipher (`b ^ UartMask`), receiving `0x06` ACK every 1024-byte (1 KB) sector.
5. Controller bootloader verifies cumulative 32-bit byte sum against Flash memory (`0x04` EOT on match; `0x15` NAK on failure) and automatically restarts into the application.

### Step 5: Live Telemetry Verification
After flashing completes, the controller begins broadcasting 18-byte telemetry frames at 9600 Baud:
```bash
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- monitor /dev/tty.usbserial-110 --baud 9600
```
Expected output:
```
[MONITOR] Listening on /dev/tty.usbserial-110 (9600 Baud, XOR 0x69 cipher)...
Frame #001: 48.1 V | 0.0 A (0 W) | Temp: 24.2 °C | Speed: 0.0 km/h | Cadence: 0 RPM | Faults: None [OK]
Frame #002: 48.1 V | 0.0 A (0 W) | Temp: 24.2 °C | Speed: 0.0 km/h | Cadence: 0 RPM | Faults: None [OK]
```

### Step 6: No-Load Motor Spin Test
1. Set bench power supply current limit to **2.5 A**.
2. Connect throttle or apply a 1.2V–3.5V analog signal to the throttle pin.
3. Observe motor spin:
   - Motor should ramp smoothly up to maximum RPM without cogging or hesitation.
   - Current draw at full speed no-load should be **0.8A to 1.4A** at 48V.
   - If current hits the 2.5A ceiling with grinding noise, release throttle immediately and verify Hall sensor wire sequence.

---

## 6. Display Bench Flashing & ISP Lockout Bypass

ToSeven displays (`T24V`, `T154`, `LCD135C`, `M15`) use a MegaHunt MH1903/MH2020C microcontroller equipped with an internal USB CDC ISP bootloader:

1. **Connect Display over USB:**
   - Plug the display directly into the host computer via its USB-C port.
   - Hold the **Power + Down** buttons while plugging in to force ISP bootloader mode.
   - The device will enumerate as `/dev/tty.usbmodem*` (macOS/Linux) or `COM*` (Windows).
2. **Inspect and Neutralize Lockout Gates:**
   ```bash
   # Inspect model and lockout status
   cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display info blobs/displays/t24v/TO7-T24V-Y241225.hex

   # Permanently patch lockout branch gates
   cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display patch blobs/displays/t24v/TO7-T24V-Y241225.hex build/TO7-T24V-Unlocked.hex
   ```
3. **Flash Display with Auto-Unlock:**
   ```bash
   cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display flash /dev/tty.usbmodem110 blobs/displays/t24v/TO7-T24V-Y241225.hex --unlock
   ```

---

## 7. Troubleshooting & Recovery

| Symptom | Probable Cause | Corrective Action |
| :--- | :--- | :--- |
| **No serial response / Timeout waiting for IDENT** | RX/TX reversed or baud rate mismatch | Swap USB-UART RX and TX. Verify Ground pin is connected. Power cycle controller using Key switch while flasher is probing. |
| **Current immediately hits 1.0A clamp on power-up** | Reverse polarity or shorted VBat rail | Cut power immediately. Check harness wiring with DMM. Verify resistance between Pin 1 and Pin 3 exceeds 10 kΩ. |
| **Telemetry receives framing errors / garbage bytes** | Unconnected GND or baud rate mismatch | Verify baud rate is 9600 Baud for telemetry (flasher uses 115200 Baud). Confirm USB-UART GND is firmly soldered to Pin 3. |
| **Flash completes but motor does not turn** | Brake cutoff engaged or throttle voltage out of range | Check telemetry frame fault flags in `monitor`. Ensure brake sensors are disconnected or normally open. |
| **Total bootloader brick (no IDENT after interrupted flash)** | Flash vector table corrupted | The ToSeven hardware bootloader resides in protected ROM (`0x1FFFD800` or `0x08000000..0x080017FF`) and cannot be erased by IAP writes. Power cycle the Key switch while launching `toseven_pack flash` to catch the initial ROM boot sequence. |
