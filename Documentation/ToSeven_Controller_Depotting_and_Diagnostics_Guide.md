# ToSeven Controller Depotting & Hardware Diagnostics Guide
### Practical Teardown, Thermal Depotting, and Board-Level Triage for DM01 and DM02 Controllers

---

## 1. Material Overview: What You Are Dealing With

ToSeven mid-drive motor controllers (both DM01 and DM02) are potted inside an aluminum horseshoe casting using **dense, high-tensile Polyurethane (PU) elastomer** (~Shore A 70–90), rather than soft silicone gel or rock-hard epoxy.

* **Room-Temperature Characteristics:** Feels like automotive tire tread or hard roller-skate rubber. It has exceptionally high tear resistance and forms a strong adhesive bond with both the cast aluminum housing and the green PCB solder mask.
* **Thermal Characteristics:** Polyurethane experiences significant thermal softening between **80°C and 90°C (175°F to 195°F)**. In this temperature range, its polymer chains relax, tear strength drops dramatically, and its adhesive bond to the aluminum and PCB releases, allowing it to be sliced like warm cheese and peeled away in chunks.

---

## 2. Critical Safety Warnings & Rules

> [!CAUTION]
> **NEVER PRY POLYURETHANE COLD WITH METAL PICKS OR SCREWDRIVERS.**
> At room temperature, the tensile strength of cured polyurethane is higher than the shear strength of surface-mount solder joints. Prying cold rubber upward will rip 0402/0603 ceramic capacitors, resistors, and fine-pitch microcontroller pins directly off the circuit board, permanently destroying traces and solder pads.

> [!WARNING]
> **DO NOT USE ACETONE, BRAKE CLEANER, OR PAINT THINNER.**
> Cured polyurethane is chemically resistant to standard workshop solvents. Acetone will not dissolve cured PU; instead, it will dissolve the plastic insulating sleeve on the large bulk electrolytic capacitor, melt the white nylon JST sensor connectors, soften the PCB solder mask, and leave a sticky, gummy residue.

* **Ventilation:** Always work in a well-ventilated space when warming potting compounds. Keep temperatures strictly below 100°C (212°F) to avoid burning or releasing hazardous fumes.

---

## 3. Required Tools & Materials

| Tool / Item | Purpose | Notes |
| :--- | :--- | :--- |
| **Heated Bed / Controlled Oven / Hot Plate** | Evenly heats aluminum housing | 3D printer bed (80°C–85°C) or toaster oven (80°C) is ideal. |
| **Hot Air Rework Station or Hair Dryer** | Spot heating during manual peeling | Set to 90°C–100°C; do not use high-temp industrial heat guns without temperature control. |
| **X-Acto Knife (#11 Blade) / Scalpel** | Slicing perimeter and top layer grids | Strictly for scoring and shallow cutting; never for levering against the board. |
| **Bamboo Skewers / Wooden Popsicle Sticks** | Non-marring prying and scraping | Wood is harder than warm PU but softer than copper traces and solder mask. |
| **Nylon Automotive Trim Spudger** | Wedge tool for perimeter release | Pliable plastic prevents gouging aluminum or PCB. |
| **99% Isopropyl Alcohol (IPA)** | Final film cleanup and residue release | Weakens boundary adhesion; safe for all electronics. |
| **Stiff Nylon Toothbrush & Soft Brass Brush** | Scrubbing IC pins and tight crevices | Brass is soft enough to protect copper/mask while clearing rubber between pins. |
| **Digital Multimeter (DMM)** | Electrical triage and diode testing | For checking MOSFETs and power rails. |

---

## 4. Step-by-Step Thermal Depotting Procedure

```
[Step 1: Aluminum Pre-Heat (80°C-85°C)] ──> [Step 2: Slice Metal Perimeter Bond]
                                                          │
                                                          ▼
[Step 4: Switch to Wood/Nylon near PCB] <── [Step 3: Shallow Grid Slicing (1-2mm)]
            │
            ▼
[Step 5: Detail Cleaning with IPA & Brass Brush]
```

### Step 1: Pre-Heat the Aluminum Shell (80°C to 85°C / 175°F to 185°F)
Because aluminum conducts heat far better than rubber, heating from the metal side warms the compound from the bottom up, weakening the adhesive bond where the PCB sits against the casing.
1. Place the controller with its flat aluminum bottom directly on a **3D printer heated bed set to 80°C–85°C**, or into an oven set to **80°C (175°F)**.
2. Allow it to heat soak for **15 to 20 minutes**.
3. *Verification:* Touch the black potting with a wooden stick; it should feel noticeably soft and pliable with slight give, rather than stiff like a cold tire.

### Step 2: Break the Perimeter Bond
1. While the controller is hot, use a fresh scalpel or X-Acto blade to trace the **entire inner perimeter wall of the aluminum horseshoe casting**.
2. Cut straight down along the aluminum edge to sever the bond between the polyurethane and the metal walls.
3. Insert a nylon spudger or wooden stick along the edge; the softened rubber should separate cleanly from the smooth aluminum. Relieving this perimeter bond significantly reduces internal mechanical tension.

### Step 3: Layered Grid Slicing (Top Layers)
1. Do not attempt to pry out the entire mass in a single block.
2. In open areas away from the wire bundles, score horizontal and vertical grid lines **1 mm to 2 mm deep** into the surface of the warm rubber.
3. Use a flat wooden stick or nylon pry tool to push and peel the warm cubes sideways. The rubber will tear away along your score lines.
4. If the rubber cools down and becomes tough again, reheat for 5–10 minutes.

### Step 4: Approaching the Circuit Board Surface
1. **Put all metal blades away** once you are within 1–2 mm of component bodies or the green PCB substrate.
2. Use sharpened bamboo skewers or carved wooden sticks to gently roll and push the warm rubber off component tops and between electrolytic capacitors.
3. Around the large bulk capacitor, gently work the stick around its base. Do not rock or twist the capacitor, as its lead legs can tear the internal PCB pads.

### Step 5: Final Cleanup Around IC Pins
1. Once the bulk rubber is removed, a thin translucent film will remain over the microcontroller and gate drivers.
2. Apply generous amounts of **99% Isopropyl Alcohol (IPA)** to the area.
3. Use a stiff nylon toothbrush or a **soft brass wire brush** to gently scrub along the direction of the IC pins. The alcohol will release the residual rubber film, leaving clean solder joints ready for inspection or probing.

---

## 5. High-Value Decompilation & Reverse-Engineering Data

For our open-source firmware and reverse-engineering project, physical access to a depotted controller PCB provides critical ground-truth data that cannot be 100% verified from disassembling `.Pack` containers alone.

If you expose the board, please inspect, photograph (under macro/magnification), and document the following targets:

### 1. MCU Part Number & Silicon Markings
* **Location:** The main square IC located near the center of the PCB (typically an LQFP-48 or LQFP-64 package).
* **Information Needed:** The full laser-etched text on the chip face:
  - Manufacturer: STMicroelectronics (`STM32`), GigaDevice (`GD32`), or Artery (`AT32`).
  - Part Number: e.g. `GD32F303RCT6`, `STM32F303CCT6`, or `STM32F103RFT6`.
  - Date code and silicon revision.
* **Why it matters:** Confirms the exact core clock ceiling (72 MHz on STM32 vs. 108–120 MHz on GD32), exact Flash sector layout for Page 127 EEPROM persistence, and hardware FPU capabilities.

### 2. Dumping the Factory IAP Bootloader (0x08000000..0x080017FF)
The official `.Pack` firmware containers only contain application code starting at Flash address `0x08001800`. The **first 6 KB (`0x08000000..0x080017FF`)** contains the factory In-Application Programming (IAP) bootloader, which has never been extracted directly from silicon.
* **Extraction via SWD:**
  Connect an ST-Link/V2 debugger to the SWD test points and run:
  ```bash
  # Dump the entire raw Flash memory (128 KB)
  st-flash read full_flash_dump_128k.bin 0x08000000 0x20000

  # Or dump specifically the 6 KB factory bootloader
  st-flash read factory_bootloader_6k.bin 0x08000000 0x1800
  ```
  *(Or via OpenOCD: `dump_image bootloader.bin 0x08000000 0x1800`)*
* **Why it matters:** Provides the bit-exact factory bootloader assembly, confirms whether Readout Protection (RDP) is disabled (`0xAA`) by default, and reveals any undocumented vendor diagnostic commands or recovery baud rates.

### 3. Phase Shunt Resistor Values & Op-Amp Markings
* **Location:** Heavy flat metal-strip or surface-mount resistors located immediately adjacent to the low-side MOSFET source legs and phase wire pads.
* **Information Needed:**
  - Laser marking on the shunts (e.g. `R002` = 2.0 mΩ, `R003` = 3.0 mΩ, `R005` = 5.0 mΩ, `1M0` = 1.0 mΩ).
  - Part number on the small 8-pin operational amplifier IC (if discrete, e.g. SGM8262, LM358, or if using MCU internal op-amps).
* **Why it matters:** Determines the exact Amperes-per-ADC-LSB conversion ratio for our Field Oriented Control (FOC) current measurement pipeline and hardware overcurrent trip comparator.

### 4. Power MOSFET & Gate Driver Part Numbers
* **Location:** The row of power transistors clamped against the aluminum heatsink casing (6 FETs on DM02, 12 FETs on DM01).
* **Information Needed:**
  - Part number on the MOSFET packages (e.g. CRST040N08N, NCE8580, IRFB4110).
  - Part number on the gate driver IC (typically a 16-pin or 28-pin SOIC/QFN, e.g. FD6288Q, EG2133, or discrete totem-pole pairs).
* **Why it matters:** Provides exact gate charge ($Q_g$) and switching speeds, allowing us to mathematically optimize PWM dead-time margins (currently clamped to 1.38 µs) and thermal dissipation.

### 5. Analog Resistor Divider Ratios
* **Battery Voltage Divider (PB0 / ADC Channel 8):** The pair of high-precision SMD resistors (one high-side connected to `B+`, one low-side to `GND`) feeding `PB0`. Reading their 3-digit/4-digit codes (e.g. `104` = 100 kΩ and `512` = 5.1 kΩ) provides the exact hardware attenuation constant.
* **Torque Sensor Input Filter (PA4 / ADC Channel 4):** Resistor and capacitor values on the analog strain gauge line entering the MCU, validating our software low-pass filter time constants.
* **Temperature NTC Divider (PB1 / ADC Channel 9):** Identifies the pull-up resistor value (typically 10 kΩ or 4.7 kΩ) used in the Steinhart-Hart thermistor lookup calculation.

### 6. SWD Debug Port Pinout & PCB Test Point Map
* **Location:** A cluster of circular gold/silver test pads on the board, often silk-screened with labels like `DIO`, `CLK`, `GND`, `3V3`, `TX`, `RX`, or `RST`.
* **Action:** Photograph the test pad cluster with clear surrounding component landmarks.
* **Why it matters:** Enables creating a definitive hardware unbricking and direct-SWD flashing guide, allowing developers and riders to revive dead controllers and flash custom firmware directly without the factory bootloader.

---

## 6. Non-Destructive Electrical Triage (Check Dead Controllers First)

Before spending hours depotting dead units, run these diagnostic checks with a standard Digital Multimeter (DMM):

### Test 1: External 3-Phase MOSFET Diode Test (0% Depotting Required)
This test can be performed immediately on any sealed controller using only the external wire harness:

1. Set your multimeter to **Diode Mode** (`->|-`).
2. **Check Low-Side MOSFETs:**
   - Connect the **Red (+) multimeter probe** to the main **Battery Negative terminal (`B-`)**.
   - Probe each of the three motor phase terminals (Phase U / Blue, Phase V / Green, Phase W / Yellow) with the **Black (-) probe**.
   - *Expected Result:* Each phase should show a standard body diode forward voltage drop of **0.40 V to 0.52 V**.
3. **Check High-Side MOSFETs:**
   - Connect the **Black (-) multimeter probe** to the main **Battery Positive terminal (`B+`)**.
   - Probe each of the three motor phase terminals (Blue, Green, Yellow) with the **Red (+) probe**.
   - *Expected Result:* Each phase should show **0.40 V to 0.52 V**.
4. **Reverse Check:** Swap probes (reverse bias); all combinations should read **OL** (Open Loop / Infinite resistance).

> [!IMPORTANT]
> **Diagnosing the Result:**
> - If any phase reads **`0.00 V` (dead short) or continuity beep in both directions:** One or more MOSFETs in that half-bridge are shorted and destroyed.
> - If all 6 readings are normal (~0.45 V), the power stage MOSFETs are intact; the fault lies in the logic power supply, gate driver, or microcontroller.

### Test 2: Low-Voltage Logic Rail Short-Circuit Test
On the partially depotted controller, locate the low-voltage power rails:
1. Switch your multimeter to **Resistance mode (Ω)**.
2. Measure resistance between the **3.3V logic rail** (or the VDD pin of the microcontroller) and **GND**.
   - **Healthy Reading:** Several kilohms (> 1 kΩ) or higher.
   - **Shorted Reading (< 20 Ω or dead short):** Either the internal step-down buck converter (often an XL7005A, LM5017, or MP4560 IC) has failed, or a voltage spike breached the logic rail and destroyed the microcontroller silicon.

### Test 3: SWD Silicon Verification Test
If the microcontroller is partially exposed:
1. Locate the 4-pin SWD programming cluster: `SWDIO`, `SWCLK`, `3.3V`, and `GND`.
2. Connect an inexpensive **ST-Link/V2** or **J-Link** debugger (SWDIO to SWDIO, SWCLK to SWCLK, GND to GND).
3. Power the 3.3V rail externally from the debugger's 3.3V output (with the main battery disconnected).
4. Run `st-info --probe` or STM32CubeProgrammer:
   - If the core is detected as **`Cortex-M4`** (Device ID `0x1BA01477` / STM32F303 or GD32F303), the processor is **alive and healthy**. The firmware payload can be dumped via SWD, and the controller is likely repairable by fixing the power stage or buck regulator.
   - If the debugger reports "Target not found" and the MCU chip gets warm to the touch under 3.3V power, the processor is fried.

---

## 7. Summary: Targeted Depotting vs. Full Strip

* **If repairing a blown MOSFET:** Only clear out the section along the outer aluminum heatsink rail over the 6 (or 12) power FETs. Leave the center logic potted to save time.
* **If diagnosing a non-responsive controller:** Expose the DC-DC buck converter circuit (near the inductor coil and large capacitor) and the square MCU in the center.
* **Always use heat (80°C–85°C) and wooden/nylon tools.** Never use brute mechanical force.
