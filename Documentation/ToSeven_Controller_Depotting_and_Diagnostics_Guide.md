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

## 5. Non-Destructive Electrical Triage (Check Dead Controllers First)

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

## 6. Summary: Targeted Depotting vs. Full Strip

* **If repairing a blown MOSFET:** Only clear out the section along the outer aluminum heatsink rail over the 6 (or 12) power FETs. Leave the center logic potted to save time.
* **If diagnosing a non-responsive controller:** Expose the DC-DC buck converter circuit (near the inductor coil and large capacitor) and the square MCU in the center.
* **Always use heat (80°C–85°C) and wooden/nylon tools.** Never use brute mechanical force.
