# ToSeven Motor Controller & Display Firmware Flashing Protocol Specification

## Executive Summary
This document provides the definitive, clean-room technical specification of the firmware update and In-Application Programming (IAP) protocols used across **ToSeven (TO7)** mid-drive motor systems (engineered by OEM **MinShine Electronics**, `cn.minshine.ebike`).

It covers two distinct subsystems:
1. **Motor Controller IAP Protocol (ARM Cortex-M4F):** Used to flash `.Pack` containers to the DM01 and DM02 motor controllers over UART / BLE OTA.
2. **Display ISP Protocol (MegaHunt MH2020C / MH1903):** Used to flash `.hex` firmware dumps to ToSeven e-bike displays (`T24V`, `LCD135C`, `M15`, `T154`) over USB CDC-ACM.

All details in this specification have been reverse-engineered, validated against factory binaries (`TO7-G2 IAP Tool Ver 5.2`, `cn.minshine.ebike` Android tuning APK, and `MH2020C_ISP.exe`), and cross-verified with bit-identical Rust implementations in [`dm01-firmware`](../dm01-firmware/src/flasher.rs).

---

## Part 1: Motor Controller IAP Protocol (DM01 & DM02)

### 1. Physical & Serial Interface
* **Signaling:** 3.3V TTL UART (asynchronous serial).
* **Wire Pinout (Display/Programming Connector):**
  * `Pin 1 (Red):` Battery $V_{\text{batt}}$ (48V / 52V nominal)
  * `Pin 2 (Black):` Ground (GND)
  * `Pin 3 (Blue):` Key Lock / Power-On enable ($V_{\text{batt}}$ trigger)
  * `Pin 4 (Green):` Controller RX (connects to USB-UART TX)
  * `Pin 5 (Yellow):` Controller TX (connects to USB-UART RX)
* **Baud Rates:**
  * **Normal IAP Programming:** `115200 Baud` (8 data bits, 1 stop bit, no parity, `8-N-1`).
  * **Application Run Mode:** `9600 Baud` (or `19200 Baud` depending on display protocol).
  * **Recovery / Restart Trigger:** `9600 Baud` (used by the flasher to command a running MCU into IAP mode).
* **Hardware Reset Control:**
  * Flasher utilities toggle the USB-to-UART adapter's `DTR` / `RTS` lines with a **500 ms low pulse** (`0x483C20`) to reset the controller when the bootloader does not immediately respond.

---

### 2. Container Architecture & Encryption Keys (Mathematical Proof)
Every official ToSeven firmware release is packaged inside a **131,072-byte (128 KB)** `.Pack` container file. The container consists of a **34-byte encrypted header (`0x00..0x21`)** followed by the encrypted payload starting at offset **`0x22`**.

#### The Official Key Derivation Algorithm
Reverse engineering of procedure `0x00480548` in `TO7-G2 IAP Tool` revealed the exact mathematical formulas used to derive the XOR encryption keys:

$$\text{Header XOR Key} = (\text{pack}[0\text{x}20] + \text{pack}[0\text{x}21]) \pmod{256}$$

$$\text{Payload XOR Key} = (\text{decrypted\_header}[0\text{x}1E] + \text{decrypted\_header}[0\text{x}1F]) \pmod{256}$$

#### Header Format (34 Bytes)
| Offset | Type | Field Name | Description |
| :--- | :--- | :--- | :--- |
| `0x00..0x01` | `u16` (LE) | `Customer_Num` | OEM / Customer ID code (`0x196E` = 6510, matching `V6510C`). |
| `0x02` | `u8` | `Device_Type` | Target controller identifier (`0xA4` = DM01, `0xA3` = DM02). |
| `0x03..0x04` | `u16` (LE) | `CtlVer` | Controller firmware version (e.g., `0x81BE`). Bootloader enforces `CtlVer >= MCU_IAP_Ver`. |
| `0x05..0x08` | `[u8; 4]` | `Padding` | Always zeroes (`0x00, 0x00, 0x00, 0x00`) in decrypted plaintext. |
| `0x09..0x0A` | `u16` (LE) | `IAP_CMD_Num` | Flashing tool command code (`0x4E22` = `20002` decimal). |
| `0x0B..0x0C` | `u16` (LE) | `ModelCode` | Internal hardware revision code. |
| `0x0D..0x0F` | `[u8; 3]` | `BuildDate` | Encoded date stamp. |
| `0x10..0x13` | `[u8; 4]` | `HeaderMagic` | Magic validation signature (`8A E6 40 F4`). |
| `0x14` | `u8` | `PackageType` | `0xAA` = Firmware + Settings (Flash EEPROM included); any other byte = Firmware Only. |
| `0x15..0x19` | — | `ProgramSize` | Split into two 16-bit LE words: `word[0x15..0x16] << 16 \| word[0x18..0x19]`. |
| `0x1A..0x1D` | `u32` (LE) | `PayloadChecksum` | Expected 32-bit arithmetic sum of all decrypted payload bytes: `word[0x1A..0x1B] << 16 \| word[0x1C..0x1D]`. |
| `0x1E..0x1F` | `[u8; 2]` | `PayloadKeySeed` | Seed bytes: `(header[0x1E] + header[0x1F]) & 0xFF == Payload XOR Key`. |
| `0x20..0x21` | `[u8; 2]` | `HeaderKeySeed` | Raw container seed bytes: `(raw[0x20] + raw[0x21]) & 0xFF == Header XOR Key`. |

#### Verified Stock Targets
* **DM01 (`TO7-DM01-K260521A-V6510C.Pack`):**
  * `raw[0x20..0x21]`: `0x0E, 0x9A` $\rightarrow 0\text{x}0E + 0\text{x}9A = \mathbf{0\text{x}A8}$ (Header Key).
  * `header[0x1E..0x1F]`: `0x51, 0x16` $\rightarrow 0\text{x}51 + 0\text{x}16 = \mathbf{0\text{x}67}$ (Payload Key).
  * `ProgramSize`: `57,708` bytes; `PayloadChecksum`: `0x0059E35F` (exact match).
* **DM02 (`TO7-DM02-K260526-V6510C.Pack`):**
  * `raw[0x20..0x21]`: `0x0D, 0xEA` $\rightarrow 0\text{x}0D + 0\text{x}EA = \mathbf{0\text{x}F7}$ (Header Key).
  * `header[0x1E..0x1F]`: `0x9D, 0x45` $\rightarrow 0\text{x}9D + 0\text{x}45 = \mathbf{0\text{x}E2}$ (Payload Key).
  * `ProgramSize`: `58,440` bytes; `PayloadChecksum`: `0x005AE79E` (exact match).

---

### 3. IAP Flashing Sequence & Packet Definitions

```
Flasher (PC / CLI)                                Controller Bootloader (MCU)
------------------                                ---------------------------
  |                                                            |
  |  [If MCU in App mode: Send 21B Wakeup @ 9600 + DTR Reset]  |
  |----------------------------------------------------------->|
  |                                                            |
  |  1. Probe @ 115200 Baud: [0x02]                            |
  |----------------------------------------------------------->|
  |                                                            |
  |  2. Identification Response: [0x02, 0xFD]                  |
  |<-----------------------------------------------------------|
  |                                                            |
  |  3. Send ACK: [0x06]                                       |
  |----------------------------------------------------------->|
  |     (Wait 10 ms)                                           |
  |  4. Transmit 12B Header Frame:                             |
  |     [0xFF, 0xFF, Size(4B), Sum(4B), Key(1B), Baud(1B)]     |
  |----------------------------------------------------------->|
  |                                                            |
  |  5. Header Accepted ACK: [0x06]                            |
  |<-----------------------------------------------------------|
  |                                                            |
  |  6. Stream Decrypted Payload in 16B Chunks                 |
  |     (Wait delay between chunks)                            |
  |----------------------------------------------------------->|
  |                                                            |
  |  7. Every 1024 Bytes (1 KB Sector): Sector ACK [0x06]      |
  |<-----------------------------------------------------------|
  |                                                            |
  |  8. Final Verification Complete: EOT [0x04]                |
  |<-----------------------------------------------------------|
  |                                                            |
  |  9. MCU Boots into New Firmware Application               |
  v                                                            v
```

#### Step 1: The 21-Byte Wakeup / Enter IAP Packet
When the controller is running its normal application firmware, the bootloader is not yet active. The flasher commands the application to restart into IAP bootloader mode by transmitting a **21-byte frame at 9600 Baud**, encrypted byte-by-byte with `XOR 0x69`:

```
Byte 0..1:   0xC9, 0x77                                (Wakeup Sync Header)
Byte 2..6:   ProgramSize (5 bytes, 7-bit Big-Endian)   (val >> 28, val >> 21, val >> 14, val >> 7, val & 0x7F)
Byte 7..11:  PayloadChecksum (5 bytes, 7-bit BE)       (sum >> 28, sum >> 21, sum >> 14, sum >> 7, sum & 0x7F)
Byte 12:     PayloadXorKey >> 4                        (High nibble)
Byte 13:     PayloadXorKey & 0x0F                      (Low nibble)
Byte 14:     0x5A                                      (CMD_IAP_PROGRAM)
Byte 15..17: Customer_Num (3 bytes, 7-bit BE)          (cust >> 14, cust >> 7, cust & 0x7F)
Byte 18..20: ToSeven CRC-16 (3 bytes, 7-bit BE)        (crc >> 14, crc >> 7, (crc & 0x7F) | 0x80)
```
* **Wire Encryption:** Each byte $B_i$ sent over the physical wire is masked as $B_i \oplus 0\text{x}69$.
* **Hardware Pulse:** Immediately following packet transmission, the flasher pulses the `DTR` line low for 500 ms to trigger an external MCU reset.

#### Step 2: Bootloader Handshake & Identification
1. The flasher opens the port at **115200 Baud** and transmits single byte `0x02` (`BOOTLOADER_PROBE_BYTE`).
2. The bootloader replies with two bytes: `0x02, 0xFD`.
3. Receipt of `0x02` and `0xFD` confirms that the MCU is in bootloader mode (`MCU Identified`).

#### Step 3: Header Handshake
1. The flasher transmits single byte `0x06` (`IAP_ACK_BYTE`).
2. The flasher pauses for **10 ms**.
3. The flasher transmits the **12-byte Header Frame**:
   ```
   [0x00..0x01]: 0xFF, 0xFF (Sync word)
   [0x02..0x05]: ProgramSize (4 bytes, Little-Endian u32)
   [0x06..0x09]: PayloadChecksum (4 bytes, Little-Endian u32)
   [0x0A]:       PayloadXorKey (e.g. 0x67 for DM01, 0xE2 for DM02)
   [0x0B]:       BaudIndex (0 = 38400, 1 = 57600, 2 = 115200 Baud)
   ```
4. The bootloader validates that `Customer_Num` matches the vendor mask and `CtlVer >= Current_MCU_Ver`.
5. On success, the bootloader responds with byte `0x06` (`ACK`).

#### Step 4: Flash Programming & Sector Pacing
1. The flasher streams the **decrypted plaintext binary** in **16-byte (`0x10`) chunks**.
2. Between 16-byte chunks, the flasher inserts an inter-chunk pacing delay (scaled according to baud rate: 50 µs for 115200 Baud, 150 µs for 57600 Baud, 300 µs for 38400 Baud).
3. **Sector Handshake:** Every **1024 bytes (1 KB)** written (or at the end of the payload), the flasher halts and waits for the bootloader to erase and program the Flash sector.
4. The bootloader returns byte `0x06` (`ACK`) once the 1 KB sector is committed to Flash memory.

#### Step 5: Finalization & Verification
1. Once all payload bytes have been transferred, the flasher waits for the bootloader to verify the full-image CRC.
2. If verification succeeds, the bootloader transmits byte `0x04` (`EOT`).
3. The flasher displays `Program sucessully` and commands the MCU to boot into application mode.

---

### 4. Root Cause of Official Anti-Rollback & Identification Errors
The ToSeven Miraheze Wiki documents a frequent user failure:
> *"MCU unidentified, try again or IAP version is lower or pack version is lower"*

Reverse engineering of the factory tool and APK validates the exact source of this error:
1. **Customer ID Mismatch:** If `Customer_Num` in the header does not match the bootloader's locked vendor mask (`0x196E`), the bootloader aborts before handshake completion (`MCU unidentified`).
2. **Version Anti-Rollback:** If `CtlVer` at `header[3..4]` is strictly less than the version integer recorded in the MCU's internal bootloader Flash, the bootloader rejects the 12-byte header frame and refuses to erase Flash (`pack version is lower`).
3. **Software + Settings Flag:** If `header[0x14]` is not `0xAA`, the tool reports `"Software Only"` and ignores any attached EEPROM block (`0x0801FC00`), preventing factory calibration restore.

---

## Part 2: Display ISP Protocol (MegaHunt MH2020C / MH1903)

ToSeven displays (`T24V`, `LCD135C`, `M15`, `T154`) use MegaHunt Microelectronics ARM microcontrollers (such as the `MH2020C` or `MH1903`). These displays do not use `.Pack` files; they are flashed directly with **Intel HEX (`.hex`)** dumps using the MegaHunt In-System Programming (ISP) protocol.

### 1. Physical Interface
* **Connection:** Virtual COM Port via USB Full Speed (FS Mode CDC-ACM, VID/PID `MEGAHUNT1901` or `MH1903 ROM BOOT`).
* **Baud Rate:** Auto-negotiated over USB CDC (virtual serial).

### 2. Packet Framing
All communications to the MegaHunt bootloader follow a fixed framing standard:

```
[0x00]:      0x02 (STX - Start of Transmission)
[0x01]:      Command ID (1 byte)
[0x02..0x03]: Payload Length L (2 bytes, Little-Endian u16)
[0x04..3+L]:  Payload Data (L bytes)
[4+L..5+L]:   CRC-16-CCITT (2 bytes, Little-Endian u16)
```

#### CRC-16-CCITT Algorithm
The CRC is calculated over all bytes from index `0` through `3 + L` (STX, CMD, Length, and Payload Data):
* **Polynomial:** `0x1021` ($x^{16} + x^{12} + x^5 + 1$)
* **Initial Value:** `0xFFFF`
* **Bit Direction:** MSB first (standard CCITT / XMODEM variant)
* **Byte Order in Frame:** Little-endian (`[crc & 0xFF, (crc >> 8) & 0xFF]`).

### 3. Command Dictionary
| Command ID | Enum Name | Description |
| :--- | :--- | :--- |
| `0x20` | `HANDSHAKE1` | Primary bootloader synchronizer. |
| `0x21` | `HANDSHAKE2` | Secondary bootloader synchronizer (`CBootHANDSHAKE`). |
| `0x22` | `CHIP_INFO` | Reads chip silicon ID, serial number (`RPacketChipSN`), and ROM version. |
| `0x3A` | `READ_FLASH_ID` | Reads SPI Flash manufacturer and JEDEC ID (`RPacketFlashID`). |
| `0x3D` | `ERASE_FLASH` | Erases full chip or specified Flash sector range (`WPacketEraserFlash`). |
| `0x3E` | `WRITE_OPTION` | Programs Flash configuration options and Quad-SPI parameters (`WPacketFlashOption`). |
| `0x50` | `PROGRAM_FW` | Programs a contiguous block of display firmware (`WPacketFirmware`). |
| `0x58` | `CHIP_UPDATE` | Finalizes programming and commands display reboot (`WPacketChipUpdate`). |

---

## Part 3: Open-Source Rust Implementation

The ToSeven flashing protocol is implemented directly in `#![no_std]` Rust in the `dm01-firmware` crate:
* **Protocol Module:** [`src/flasher.rs`](../dm01-firmware/src/flasher.rs)
  * `build_wakeup_packet()`: Bit-exact 21-byte wakeup frame builder with 7-bit big-endian encoding, ToSeven CRC-16, and XOR 0x69 cipher.
  * `build_header_packet()`: 12-byte header frame builder with modulo-256 additive checksum.
  * `build_display_isp_packet()` / `parse_display_isp_packet()`: Complete MegaHunt ISP framing with CRC-16-CCITT.
  * `IapSession`: Non-blocking state machine driver for UART and BLE OTA flashing.
* **CLI Utility:** [`toseven_pack flash`](../dm01-firmware/src/bin/toseven_pack.rs)
  * Simulates or executes the full IAP flashing pipeline:
    ```bash
    # Dry-run validation of .Pack container and framing
    cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack --dry-run

    # Live flashing to motor controller via USB-UART adapter
    cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash /dev/tty.usbserial-110 blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack
    ```

> [!IMPORTANT]
> **Bench Power Supply Safety Rule:**
> When flashing custom or recompiled firmware to an e-bike motor controller for the first time, always power the controller through a **current-limited bench power supply** (e.g. 48V / 52V capped at 0.5A to 1.0A) rather than a full e-bike lithium battery pack. This guarantees hardware protection against potential short circuits or incorrect FET switching dead-times.

