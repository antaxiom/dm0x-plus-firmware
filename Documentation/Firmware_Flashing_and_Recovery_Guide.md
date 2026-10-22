# Firmware Flashing & Recovery Guide: Motor Controllers & Displays

This guide provides practical step-by-step instructions for flashing, un-bricking, and recovering **ToSeven DM01 / DM02 motor controllers** and **handlebar displays (T24V, T154, M15, LCD135C)** using the **DM+ Toolchain** (`toseven_pack` CLI and `DM+ Firmware Studio` GUI).

It adapts official ToSeven / MinShine procedures while removing proprietary Windows tool constraints, resolving anti-rollback ambiguities, and explaining how to safely recover controllers with corrupted firmware.

> [!WARNING]
> ### Disclaimer of Warranty & Equipment Damage Liability
> **THIS SOFTWARE AND DOCUMENTATION ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED.**
> Flashing custom, open-source, or modified firmware onto electric bicycle motor controllers and displays carries inherent electrical and mechanical risks. **Any damage, wear, overheating, component failure, blown MOSFETs, stripped primary reduction gears, battery damage, or permanent bricking of your motor controller, display, battery pack, wiring harness, or bicycle is solely and entirely your own responsibility.** The authors, contributors, and maintainers provide NO warranty and assume NO liability for any direct, indirect, incidental, special, or consequential damages resulting from the use or misuse of this software, tools, guides, or instructions.

---

## 1. Safety & Pre-Flight Rules

> [!CAUTION]
> ### CRITICAL SAFETY WARNING: BENCH TEST ONLY
> **OPEN-SOURCE FIRMWARE MUST NOT BE INSTALLED ANYWHERE OTHER THAN A CONTROLLED BENCH TEST.**
> Do NOT install open-source firmware on an assembled bicycle or attempt to ride with it at this stage. It is strictly intended for isolated bench bring-up, phase verification, and current-clamped diagnostics.
>
> ### Bench Power vs. Lithium Battery Safety
> 1. **Do not use a high-discharge e-bike lithium battery for flashing a corrupted controller.** A lithium battery can deliver >100A instantly if an internal FET state is stuck or if a short occurs.
> 2. **Current-Clamp to 1.0A:** Power the controller using an adjustable DC bench power supply set to **48.0V (or 52.0V) clamped strictly to 1.0A**.
>    - Normal controller idle logic draws **40 mA to 65 mA** at 48V.
>    - If current hits the 1.0A ceiling upon turning on the key switch, **cut power immediately**; this signals a shorted rail or blown FET.
> 3. **3.3V Logic Levels Only:** The Cortex-M4F serial port runs at **3.3V TTL**. Connecting a 5V adapter or shorting to $V_{\text{batt}}$ will permanently destroy the controller's UART transceiver.
> 4. **Common Ground:** Ensure the USB-to-UART adapter Ground (GND) is connected to Pin 3 (Black) of the Julet connector before powering up.

---

## 2. Hardware Wiring & Cable Pinouts

### 2.1 Motor Controller Programming Connector (5-Pin Julet / Higo Mini-B)
Looking into the connector face:

```
        Top Notch
       ┌─────────┐
    1 (●)       (●) 2
   3 (●)         (●) 4
         (●) 5
       └─────────┘
```

| Pin | Wire Color | Signal Name | Description | Flashing / Bench Connection |
| :--- | :--- | :--- | :--- | :--- |
| **1** | Red | **VBat** | Battery Voltage (36V–58.8V) | Bench Supply **Positive (+)** |
| **2** | Blue / Orange | **Lock / Key-In** | Ignition / Power-On trigger | Connect to **VBat** via toggle switch |
| **3** | Black | **GND** | System Ground & Common Reference | Bench Supply **Negative (-)** AND USB-UART **GND** |
| **4** | Green | **Controller TX** | 3.3V Serial Out from Controller | USB-UART **RXD** |
| **5** | Yellow | **Controller RX** | 3.3V Serial In to Controller | USB-UART **TXD** |

### 2.2 Display Programming Connector (USB-C)
ToSeven displays (`T24V`, `LCD135C`, `M15`, `T154`) feature a standard USB-C port that directly connects to your PC as a USB Full Speed CDC-ACM virtual COM port (`/dev/tty.usbmodem*` on macOS/Linux or `COM*` on Windows).

---

## 3. Flashing Motor Controller Firmware (`.Pack`)

You can flash using either the **DM+ Firmware Studio GUI** or the **`toseven_pack` CLI**.

### Method A: DM+ Firmware Studio GUI
1. Open the application:
   ```bash
   cargo run --manifest-path dm-plus-gui/Cargo.toml
   ```
2. In the **Motor Controller** tab:
   - Select your serial port from the dropdown (click **Refresh Ports** if needed).
   - Choose your **Firmware Source**:
     - *DM01 Open Firmware v0.3.0 (Built-in)*: Standard open firmware for DM01 external 12-FET.
     - *DM02 Open Firmware v0.3.0 (Built-in)*: Standard open firmware for DM02 internal 6-FET.
     - *Custom .Pack File (Manual Selection)*: Browse to select an official stock release (e.g. `TO7-DM01-K260521A-V6510C.Pack` or an older stock `.Pack`).
   - If using built-in firmware, select your tuning profile (e.g., *Gear-Safe Progressive*) and battery nominal pack. (Note: These options automatically disable if a custom `.Pack` is selected, as custom files have hardcoded limits).
3. **Verify:** Click **Test Flash (Dry Run)** to validate packet framing, container checksums, and vector tables without touching hardware.
4. **Flash:** Click **Flash Firmware**. The flasher executes the automated 5-step IAP engine:
   - **Step 1 (Wakeup):** Transmits the 21-byte encrypted wakeup packet (`0xC9, 0x77` masked with `0x69` cipher) at 9600 Baud with high/low nibble `UartMask` distribution (bytes 12-13) and hardware DTR reset pulse fallback to wake running application controllers into bootloader mode.
   - **Step 2 (Identification):** Probes the bootloader at 38400 Baud (vendor factory default, with 115200 fallback), receiving bootloader identification `0x02, 0xFD` (or `0x06`).
   - **Step 3 (Header):** Sends `0x06` ACK, pauses 10 ms, and transmits the 12-byte header frame containing program size, 32-bit checksum, `UartMask`, and baud rate index, receiving `0x06` ACK.
   - **Step 4 (Streaming):** Streams payload in 16-byte chunks with **Tier 2 wire stream XOR cipher (`b ^ UartMask`)**, matching the bootloader's on-the-fly hardware de-masking into Flash memory at `0x08001800`. Waits for sector commit `0x06` ACK every 1024 bytes (1 KB).
   - **Step 5 (Verification):** Verifies the MCU bootloader's cumulative 32-bit byte sum of Flash memory (`0x04` EOT on match; `0x15` NAK on failure) and automatically reboots the motor into the new application.

### Method B: Standalone CLI (`toseven_pack`)
```bash
# 1. Identify connected USB serial port
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- ports

# 2. Verify .Pack container integrity & vector table
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- verify "blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack"

# 3. Simulate framing & sector pacing (Dry Run)
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash "blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack" --dry-run

# 4. Live Flash over USB serial adapter to physical motor controller
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash /dev/tty.usbserial-110 "blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack"
```

---

## 4. Recovering a Controller with Corrupted Firmware

### 4.1 Is It Safe to Flash an Old Stock Version?
**Yes, it is safe**, provided you understand how the ToSeven bootloader behaves:

1. **The Bootloader Resides in Protected ROM:**
   - The ToSeven IAP bootloader is stored in protected on-chip ROM/system memory (`0x1FFFD800` or `0x08000000..0x080017FF`).
   - Firmware flashing only writes to application Flash starting at **`0x08001800`**.
   - Corrupted application firmware or an interrupted flash cannot overwrite or destroy the hardware bootloader.
2. **Pre-Flash Verification by `toseven_pack`:**
   - Our tool validates the Cortex-M4F vector table (initial MSP `0x2000xxxx` and Reset Handler `0x0800xxxx`) and container checksums before sending a single byte across the wire.
   - It will never write a truncated or misaligned binary payload.
3. **The Anti-Rollback Constraint (`CtlVer`):**
   - Official stock firmware containers include a controller version integer `CtlVer` in `header[0x03..0x04]`.
   - If your controller previously had a very recent firmware installed (e.g. `V6400C` from Dec 2024 or `V6510C`), the vendor bootloader enforces `CtlVer >= MCU_IAP_Ver`.
   - If you attempt to flash a stock `.Pack` whose `CtlVer` is lower than the version recorded in the MCU's bootloader, the bootloader will reject the 12-byte header with a negative response (`pack version is lower`).
   - **Crucial Safety Note:** If rejected by the bootloader, the bootloader aborts before erasing Flash sectors. It **will not** brick the controller; it simply declines to proceed.

### 4.2 Recovery Procedure & Critical Bootloader Warning

> [!CAUTION]
> ### The Potted Software-Bootloader Trap
> ToSeven controllers use an In-Application Programming (IAP) software bootloader located at `0x08000000..0x080017FF`. 
> - **The Normal Recovery Path:** When the application is running normally, sending the 21-byte wakeup frame (`0xC9 0x77` @ 9600 Baud) triggers a software reset (`SCB->AIRCR`) into the IAP bootloader.
> - **The Trap:** If custom experimental firmware encounters a BusFault or HardFault during startup (e.g. accessing an un-clocked peripheral) before entering its main loop, the MCU enters an infinite spinlock with interrupts masked. It **cannot** receive the 9600 Baud software wakeup command.
> - **Power-On Catch Limitations:** While the bootloader executes for ~200–400 ms on cold power-on, host USB-serial latency often exceeds this window. If the bootloader does not detect a valid handshake within this narrow window, it jumps straight into the corrupted application.
> - **Definitive Recovery:** If a controller becomes unresponsive to serial commands over the Julet harness, **recovery requires depotting the aluminum shell and flashing the MCU via hardware SWD (`SWDIO`/`SWCLK`) using an ST-Link/V2**. Do not flash experimental custom motor firmware on sealed controllers unless you have an SWD rig ready.

If the controller application is non-booting but the bootloader can be intercepted, follow this sequence:

1. Connect the USB-to-UART adapter to the PC and the 5-pin Julet connector.
2. Ensure the bench power supply is set to **48V @ 1.0A limit**, with the Key toggle switch **OFF**.
3. Launch the flashing tool in terminal:
   ```bash
   cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- flash /dev/tty.usbserial-110 <path_to_stock_pack>
   ```
4. While the flasher is actively probing (`Probing bootloader at 38400 Baud (0x02)...`), **toggle the Key switch ON**.
   - Upon initial power-on, the MCU hardware bootloader executes first and listens for the `0x02` probe before jumping to application code.
   - The flasher will immediately catch the bootloader response (`0x02, 0xFD`), lock the session, and stream the clean firmware image.
5. Watch the sector progress bar (`1024-byte` sectors). Once `Verification complete!` appears, power cycle the Key switch.

---

## 5. Flashing & Permanently Unlocking Displays (`.hex`)

ToSeven displays (`T24V`, `LCD135C`, `M15`, `T154`) use MegaHunt ARM microcontrollers with an internal USB CDC ISP bootloader.

### 5.1 Entering Display ISP Mode
1. Disconnect the display from the bike harness.
2. Connect the display to the computer using a high-quality USB-C data cable.
3. If the display does not automatically enter ISP mode:
   - Hold the **Power + Down** buttons simultaneously while plugging in the USB-C cable.
   - Keep holding for 2–3 seconds until the display enters bootloader mode.
   - On macOS/Linux, verify the port enumerates as `/dev/tty.usbmodem*`. On Windows, verify `COM*`.

### 5.2 Inspecting & Neutralizing Lockout Gates

Recent stock display firmwares (post-October 2024, such as `TO7-T24V-Y241225.hex` or `TO7-T154-Y240927`) contain up to 6 restriction branch gates that lock users out of the Advanced Settings menu (`Speed&Power`, current limits, and wheel circumference).

```bash
# 1. Inspect lockout status
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display info blobs/displays/t24v/TO7-T24V-Y241225.hex

# 2. Permanently unlock and save a patched .hex file offline
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display patch blobs/displays/t24v/TO7-T24V-Y241225.hex build/TO7-T24V-Unlocked.hex
```

### 5.3 Flashing Display Firmware
You can flash directly with automatic on-the-fly unlocking:
```bash
# Flash display with on-the-fly unlock over USB
cargo run --manifest-path dm01-firmware/Cargo.toml --bin toseven_pack -- display flash /dev/tty.usbmodem110 blobs/displays/t24v/TO7-T24V-Y241225.hex --unlock
```
*Note: In the `DM+ Firmware Studio` GUI, open the **Display Flasher & Patcher** tab, select your `.hex` file, ensure **Auto-Neutralize Lockouts** is checked, and click **Flash Display Firmware**.*

---

## 6. Troubleshooting Common Flashing Errors

| Error Message / Symptom | Root Cause | Solution |
| :--- | :--- | :--- |
| **`Connection timed out: No response from controller`** | RX/TX reversed, GND missing, or MCU stuck in application mode. | Swap RX and TX wires. Verify common GND. Power cycle the Key switch while the flasher is actively probing. |
| **`pack version is lower`** | Controller previously ran newer stock firmware with a higher `CtlVer` anti-rollback counter. | Use a newer stock `.Pack` (such as `V6510C`), or use DM+ Open Firmware which is configured to bypass anti-rollback rejection. |
| **`MCU unidentified`** | The `.Pack` container's `Customer_Num` does not match the vendor ID (`0x196E`), or cable was noisy. | Check `.Pack` file integrity with `toseven_pack info`. Verify cable connection and reduce electrical noise. |
| **Current immediately hits 1.0A clamp on power-up** | Shorted $V_{\text{batt}}$ rail or blown primary buck converter / FET. | Turn off supply immediately. Inspect harness pins with a digital multimeter for shorts between Pin 1 ($V_{\text{batt}}$) and Pin 3 (GND). |
| **Display disconnects mid-flash** | Low-quality USB-C charge-only cable or USB hub brownout. | Use a direct motherboard USB port with a known data cable. The display can be re-flashed without bricking. |

