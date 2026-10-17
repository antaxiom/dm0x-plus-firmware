# How the ToSeven DM01 Determines Motor Power (Human-Readable Guide)

This guide explains **exactly how the ToSeven DM01 (firmware `V6510C`) calculates motor power and phase current**, translated directly from the decompiled motor controller firmware into plain engineering and rider terms—without memory addresses or assembly jargon.

---

## 1. The Big Picture: A Three-Stage Pipeline

Every control cycle, the DM01 motor controller determines how many Amps of current to send to the motor by running a **three-stage pipeline**:

```
   ┌──────────────────────────────────────────────────────────────────────┐
   │ STAGE 1: GLOBAL CEILINGS                                             │
   │  • Riding Mode Cap (Eco = 10A, City = 18A, Sport = 30A)              │
   │  • Thermal Taper (Linearly ramps down from 70°C to 100°C)            │
   │  • High-Voltage Power Cap (Caps constant wattage above 50.0V)        │
   └──────────────────┬───────────────────────────────────┬───────────────┘
                      │                                   │
         Ignores Level Pwr%                  Scales by Level Pwr%
      (Global Mode Ceiling)                  (Level Power Ceiling)
                      │                                   │
                      ▼                                   ▼
   ┌──────────────────────────────────┐ ┌─────────────────────────────────┐
   │ STAGE 2A: TORQUE SENSOR CURVE    │ │ STAGE 2B & 2C: CADENCE &        │
   │  1. Auto-Zero Drift Tracking     │ │                THROTTLE CURVES  │
   │  2. Pedal Force Smoothing Filter │ │  • Cadence (PAS): Scales from   │
   │  3. Strength × EAssist% Scaling  │ │    10 RPM up to PAS MaxSpeed    │
   │  4. Base Current + Start Kick    │ │    (60 RPM), capped by Pwr%     │
   │  5. Cadence Scaling (÷ 60 RPM)   │ │  • Throttle: Filtered Hall      │
   │  6. Clamped to Global Mode Cap   │ │    input, capped by Pwr%        │
   └──────────────────┬───────────────┘ └─────────────────┬───────────────┘
                      │                                   │
                      └─────────────────┬─────────────────┘
                                        ▼
   ┌──────────────────────────────────────────────────────────────────────┐
   │ STAGE 3: ARBITRATION, RAMPING & SAFETY TAPERS                        │
   │  1. Winning Demand = max(Torque Current, Cadence Current, Throttle)  │
   │  2. Smooth Ramp Limiter (±0.2 A per step)                            │
   │  3. Proportional Speed Limit Soft-Taper (1.5 km/h softening window)  │
   │  4. Low-Battery Reserve Cushion (3.0 V linear taper down to 25%)     │
   └──────────────────────────────────────────────────────────────────────┘
```

---

## 2. Stage 1: The Three Global Ceilings

Before looking at how hard you are pedaling or twisting the throttle, the controller calculates the maximum current the motor is allowed to draw right now. It combines three limits:

### A. Riding Mode Limit (`Eco`, `City`, `Sport`)
The display lets you choose between three overall riding modes. Each mode sets a base maximum current ceiling (capped at an absolute hardware ceiling of `40.0 A`):
- **Eco Mode:** `10.0 A` (default)
- **City Mode:** `18.0 A` (default)
- **Sport Mode:** `30.0 A` (default on DM01; `18.0 A` on DM02)

### B. Continuous Linear Thermal Rollback (`70°C` to `100°C`)
Unlike the user manual (which describes a sudden step drop at `100°C`), the actual `V6510C` firmware uses a **smooth linear thermal taper** between `MinTemp` (`70°C`) and `MaxTemp` (`100°C`):
- **Below `70°C`:** No thermal reduction.
- **Between `70°C` and `100°C`:** The thermal ceiling ramps linearly down from the **Sport Mode ceiling** (`30.0 A`) to `0.0 A`:
  $$\text{Thermal Ceiling} = \text{Sport Max Current} \times \left(1 - \frac{\text{Temp} - 70^\circ\text{C}}{100^\circ\text{C} - 70^\circ\text{C}}\right)$$
  *(At default settings, every `1°C` above `70°C` lowers the thermal ceiling by `1.0 A`.)*
- **At or above `100°C`:** The thermal ceiling reaches `0.0 A` until the controller cools below `100°C`.

> **Interesting Firmware Quirk:** Because the thermal formula always starts tapering downward from the **Sport Mode ceiling (`30.0 A`)**, if you are riding in **Eco Mode (`10.0 A`)**, thermal throttling won't actually reduce your Eco power until the controller exceeds `90°C` (where the thermal ceiling finally drops below `10.0 A`).

### C. High-Voltage Constant-Power Cap (Above `50.0 V`)
To prevent over-powering the motor when using higher-voltage batteries (such as a freshly charged `48V` pack at `54.6V` or a `52V` pack at `58.8V`), the firmware treats **`50.0 V`** as the nominal full-current reference voltage:
- **At or below `50.0 V`:** Full Riding Mode current is allowed.
- **Above `50.0 V`:** The controller scales the current ceiling down by $\frac{50.0\text{ V}}{\text{Battery Voltage}}$:
  $$\text{Voltage-Capped Current} = \text{Mode Max Current} \times \frac{50.0\text{ V}}{\text{Battery Voltage}}$$
- **Why this matters:** In Sport Mode (`30.0 A`), electrical input power above `50.0 V` is held constant at **$50.0\text{ V} \times 30.0\text{ A} = 1500\text{ W}$**. For example, on a `52V` battery sitting at `58.0 V`, peak current is automatically capped at $30.0\text{ A} \times \frac{50.0}{58.0} = 25.8\text{ A}$ ($1496\text{ W}$). As the battery discharges toward `50.0 V`, the current limit smoothly rises to the full `30.0 A`.

---

## 3. The Crucial Split: Why `Pwr%` and `EAssist%` Confused Riders

In the display's Assist Level settings (Levels `1` through `5`), each level has two percentage settings:
- **`Pwr` (`%`):** Defaults to `20%, 40%, 60%, 80%, 100%`
- **`EAssist` (`%`):** Defaults to `30%, 40%, 60%, 80%, 100%`

Community riders and the ToSeven Wiki long noticed that changing `Pwr%` sometimes seemed to do nothing when pedaling with the torque sensor, while raising `EAssist%` in high assist levels seemed to hit an invisible ceiling unless you switched from `Eco` to `City` or `Sport`.

**The decompiled firmware reveals the exact reason:** the controller computes **two separate current ceilings** in Stage 1:

1. **The Global Mode Ceiling** (Used by the **Torque Sensor**):
   $$\text{Global Mode Ceiling} = \min(\text{Mode Limit}, \text{Thermal Ceiling}, \text{Voltage-Capped Current})$$
   **Notice that `Pwr%` is NOT in this formula!** When you ride using the **Torque Sensor**, the `Pwr%` column in the display menu is completely ignored. Instead, your assist level controls power exclusively through **`EAssist%`**.

2. **The Level Power Ceiling** (Used by the **Cadence / PAS Sensor** and **Throttle**):
   $$\text{Level Power Ceiling} = \min\left(\text{Mode Limit} \times \frac{\text{Pwr\%}}{100}, \text{Thermal Ceiling}, \text{Voltage-Capped Current}\right)$$
   When riding on **Cadence (PAS)** or **Throttle**, `Pwr%` directly caps your maximum current in that assist level, while `EAssist%` is ignored.

---

## 4. Stage 2A: How the Torque Sensor Curve Works

The DM01 has a strain-gauge torque sensor in the bottom bracket that outputs a voltage proportional to how hard you press on the pedals:
- **No pedal pressure (`ZeroVol`):** ~`700 mV` (`0.70 V`)
- **Maximum pedal force (`MaxVol`):** ~`3300 mV` (`3.30 V`)

The controller converts your pedal pressure into motor current in **five steps**:

### Step 1: Automatic Zero-Point Drift Tracking
Metal strain gauges drift slightly as the motor case warms up. To prevent "ghost pedaling" (where the motor pushes on its own) or sluggish response:
- Whenever you stop pedaling (`0 RPM`) and the pedal voltage stays completely steady (varying by less than `20 mV` across 5 consecutive readings over 50 cycles) inside a safe window (`400 mV` to `1000 mV`), the controller gently updates its resting zero point:
  $$\text{New ZeroVol} = 80\% \times \text{Old ZeroVol} + 20\% \times \text{Measured Resting Voltage}$$

### Step 2: Pedal Force Smoothing (`Filter`) and Deadband (`StartupVol`)
1. The controller subtracts the resting zero point from the live sensor reading to get your **Net Pedal Voltage**:
   $$\text{Net Pedal Voltage} = \text{Live Voltage} - \text{ZeroVol}$$
2. It smooths this voltage through a low-pass filter controlled by the display's `Torque Filter` setting (`1` to `10`, default `5`). Higher filter numbers smooth out choppy pedal strokes over more samples; lower numbers make the motor react more Crisp/Instantaneously.
3. **Startup Deadband (`StartupVol`, default `50 mV`):** If your filtered Net Pedal Voltage is below `50 mV` (or if your crank cadence is `0 RPM`), the torque sensor outputs **`0 A`**.
4. Once above `50 mV`, it converts your pedal force into a **Pedal Effort Percentage (`0%` to `100%`)** across the sensor's full span (`MaxVol - ZeroVol`, which is `3300 - 700 = 2600 mV` by default):
   $$\text{Pedal Effort \%} = \frac{\text{Filtered Net Pedal Voltage}}{\text{MaxVol} - \text{ZeroVol}} \times 100$$

### Step 3: Amplification by `StrengthValue` and `EAssist%`
Next, the controller multiplies your **Pedal Effort %** by your torque sensitivity (`StrengthValue`, default `10`) and the current assist level's **`EAssist%`** (`30%` to `100%`), and caps the resulting **Combined Effort Multiplier** at **`100%`**:

$$\text{Effective Strength} = \min(3 \times \text{StrengthValue}, 30)$$

$$\text{Combined Effort \%} = \min\left(\frac{\text{Pedal Effort \%} \times \text{Effective Strength} \times \text{EAssist\%}}{1000}, 100\%\right)$$

> **Why `StrengthValue` Defaults to `10` (`Effective Strength = 30`):**
> At the default `StrengthValue = 10`, $\frac{\text{Effective Strength}}{10} = 3.0$.
> - In **Level 1 (`EAssist = 30%`)**, your Pedal Effort is multiplied by $3.0 \times 0.30 = 0.90$. Even if you stand on the pedals at `100%` force, Combined Effort reaches `90%`.
> - In **Level 5 (`EAssist = 100%`)**, your Pedal Effort is multiplied by $3.0 \times 1.00 = 3.00$. This means you only need **`33.3%` pedal force** (`~867 mV` above zero) to reach the **`100%` Combined Effort cap**! Pressing harder than `33.3%` in Level 5 cannot increase Combined Effort above `100%`—which is why the only way to get more peak power at that point is to raise the **Riding Mode Ceiling** (`Eco` $\to$ `City` $\to$ `Sport`).

### Step 4: Base Current and Starting Kick (`CurrentOffset`)
The **Combined Effort %** is applied to your active **Riding Mode Limit** (`10A`, `18A`, or `30A`), and the controller adds a small starting current boost (`Torque CurrentOffset`, default `0.5 A`) so the motor engages immediately without lag:
$$\text{Base Torque Current} = \left(\text{Mode Limit} \times \frac{\text{Combined Effort \%}}{100}\right) + \text{CurrentOffset}$$

### Step 5: The 60 RPM Cadence Scaling Rule
Finally, the DM01 scales your **Base Torque Current** by how fast you are spinning the cranks, using **`60 RPM`** as the `100%` reference cadence (with a minimum floor of `5 RPM` when starting from a stop):

$$\text{Torque Current} = \min\left(\text{Base Torque Current} \times \frac{\max(\text{Cadence RPM}, 5)}{60}, \text{Global Mode Ceiling}\right)$$

- **Below `60 RPM`:** At `30 RPM`, the motor delivers **half** (`30 / 60 = 50%`) of your Base Torque Current. This prevents the motor from dumping 30 Amps into a stalled drivetrain at very low crank speeds and encourages spinning in an efficient gear.
- **At `60 RPM`:** You get `100%` of your Base Torque Current.
- **Above `60 RPM`:** If your Base Torque Current was less than `100%` of the Mode Ceiling (for example, light pedaling at `90 RPM`), the $\frac{\text{Cadence}}{60}$ multiplier (`1.5×`) boosts motor current proportionally so power keeps up with your fast spin—all the way up to the **Global Mode Ceiling**!

---

## 5. Stage 2B: How the Cadence (PAS) Curve Works

Alongside the torque sensor, the controller also runs a **Cadence (PAS) Curve** based purely on crank rotation speed (`Cadence RPM`) after the cranks have passed the startup magnet count (`StartupCount`):

1. **Clamp Cadence (`10 RPM` to `PAS MaxSpeed`):**
   The controller clamps your cadence between a minimum of `10 RPM` and the configured `PAS MaxSpeed` (default `60 RPM`).
2. **Linear Ramp + Offset:**
   It scales the **Global Mode Ceiling** linearly by your cadence ratio and adds `PAS CurrentOffset` (default `1.0 A`):
   $$\text{Raw PAS Current} = \left(\text{Global Mode Ceiling} \times \frac{\text{Clamped Cadence}}{\text{PAS MaxSpeed}}\right) + \text{PAS CurrentOffset}$$
3. **Cap at Level Power Ceiling (`Pwr%`):**
   Unlike the Torque Sensor, the PAS current is strictly capped by the current assist level's **Level Power Ceiling** (`Pwr%`):
   $$\text{PAS Current} = \min(\text{Raw PAS Current}, \text{Level Power Ceiling})$$

---

## 6. Stage 2C: How the Throttle Curve Works

When you press the thumb or twist throttle (`~700 mV` resting up to `3300 mV` full throttle):

1. **Smoothing & Slew-Rate Limiting:**
   - The raw throttle voltage is smoothed with an 8-sample moving filter.
   - To prevent jerky wheelies if you snap the throttle wide open, a **slew-rate limiter** restricts the internal throttle voltage from rising or falling faster than **`200 mV` per control tick**.
2. **Current Ceiling (`Pwr%`):**
   - By default, full throttle scales up to the active assist level's **Level Power Ceiling** (`Pwr%`), or up to the full Riding Mode ceiling in specific unrestricted `Throttle SpeedMode` settings (`Modes 8–12`).
3. **13 Operating Modes (`Throttle SpeedMode 0–12`):**
   - Depending on which `SpeedMode` (`0` through `12`) is selected in the display, the controller chooses whether the throttle works when you are not pedaling, whether it is limited to `6 km/h` (walk assist), the assist level's speed limit (`Spd`), the bike's global top speed limit, or unrestricted (`Mode 12`).

---

## 7. Stage 3: Arbitration, Smooth Ramping, Speed Soft-Taper & Battery Reserve

Once the Torque, Cadence (PAS), and Throttle demands are calculated, the **Central Arbitrator** combines them and applies three final smoothing and protection layers:

### 1. Input Arbitration (`max(Torque, PAS, Throttle)`)
- **If Brakes are pulled or Back-pedaling is detected:** Target current is immediately set to **`0.0 A`**.
- **If Pedaling Forward:** The controller takes whichever demand is **highest**:
  $$\text{Requested Current} = \max(\text{Torque Current}, \text{PAS Current}, \text{Throttle Current})$$
- **If Not Pedaling (Coasting):** Only the **Throttle Current** (or Walk Assist) is used.

### 2. Smooth Current Ramp (`±0.2 A` per step)
Instead of jumping instantaneously from `0 A` to `30 A`, the controller ramps the active current target up or down by at most **`0.2 A` (`2` deci-amps) per control cycle**, giving the drivetrain a smooth, mechanical feel and protecting the chain and gears from shock loads.

### 3. Proportional Speed Limit Soft-Taper (`1.5 km/h` Window)
Many e-bikes hit an annoying "brick wall" when reaching the speed limit, cutting power to zero and surging back on as you slow down. The DM01 avoids this using a **proportional speed soft-taper**:
- Let $V_{\text{limit}}$ be your active speed limit (e.g., `25.0 km/h`).
- The controller defines a **`1.5 km/h` softening band** below the speed limit (`23.5 km/h` to `25.0 km/h` in normal riding, or a `2.0 km/h` band in `6.0 km/h` walk-assist mode).
- **Below `23.5 km/h`:** You get `100%` of your requested current.
- **Between `23.5 km/h` and `25.0 km/h`:** Current tapers down linearly from `100%` to `0%`:
  $$\text{Speed-Tapered Current} = \text{Ramped Current} \times \frac{V_{\text{limit}} - V_{\text{bike}}}{1.5\text{ km/h}}$$
- **At or above `25.0 km/h`:** Current smoothly reaches `0.0 A`.

### 4. Low-Battery Reserve Cushion (`3.0 V` Window Above LVC)
To prevent sudden battery BMS shutoffs when your battery is nearly empty, the DM01 creates a **`3.0 V` soft-landing cushion** above your configured **Low Voltage Cutoff (`LVC`)** (for example, between `44.0 V` and `41.0 V` if `LVC = 41.0 V`):
- **More than `3.0 V` above `LVC`:** `100%` of current is allowed.
- **Within `3.0 V` above `LVC`:** Maximum current linearly tapers from **`100%` down to `25%`**:
  $$\text{LVC Taper Factor} = 25\% + 75\% \times \frac{V_{\text{battery}} - V_{\text{LVC}}}{3.0\text{ V}}$$
- **At or below `LVC`:** Motor drive shuts off completely (`0.0 A`) to protect the battery cells.

---

## 8. Real-World Worked Examples

### Example A: Moderate Pedaling in City Mode, Assist Level 2
- **Settings:** `City Mode` (`18.0 A` max), **Level 2** (`Pwr = 40%`, `EAssist = 40%`), `StrengthValue = 10`, `CurrentOffset = 0.5 A`, `48V` battery at `49.0 V`, controller at `45°C`.
- **Rider Input:** Pressing pedals at **`25%` effort** (`1350 mV`) while spinning at **`45 RPM`** at `18 km/h` (below the `25 km/h` speed limit).
- **Step-by-Step Calculation:**
  1. **Ceilings:**
     - Global Mode Ceiling = **`18.0 A`** (`City Mode`, below `70°C`, below `50.0 V`).
     - Level Power Ceiling (`Pwr = 40%`) = $18.0\text{ A} \times 40\% =$ **`7.2 A`**.
  2. **Torque Sensor Current:**
     - $\text{Effective Strength} = 3 \times 10 = 30$.
     - $\text{Combined Effort \%} = \frac{25 \times 30 \times 40}{1000} = 30\%$.
     - $\text{Base Current} = (18.0\text{ A} \times 30\%) + 0.5\text{ A} = 5.4\text{ A} + 0.5\text{ A} = 5.9\text{ A}$.
     - **Cadence Scaling (`45 RPM`):** $5.9\text{ A} \times \frac{45}{60} =$ **`4.4 A`**.
  3. **Cadence (PAS) Current:**
     - Raw PAS = $(18.0\text{ A} \times \frac{45}{60}) + 1.0\text{ A} = 13.5\text{ A} + 1.0\text{ A} = 14.5\text{ A}$.
     - Capped by Level Power Ceiling (`7.2 A`) $\implies$ **`7.2 A`** (if PAS assist is enabled alongside torque, or `0 A` in pure torque mode).

### Example B: Steep Hill Climb in Sport Mode, Assist Level 5
- **Settings:** `Sport Mode` (`30.0 A` max), **Level 5** (`EAssist = 100%`), `StrengthValue = 10`, `CurrentOffset = 0.5 A`.
- **Rider Input:** Standing on the pedals at **`50%` effort** (`2000 mV`) at **`60 RPM`**.
- **Step-by-Step Calculation:**
  1. $\text{Combined Effort \%} = \min\left(\frac{50 \times 30 \times 100}{1000}, 100\%\right) = \min(150\%, 100\%) =$ **`100%`**.
  2. $\text{Base Current} = (30.0\text{ A} \times 100\%) + 0.5\text{ A} = 30.5\text{ A}$.
  3. **Cadence Scaling (`60 RPM`):** $30.5\text{ A} \times \frac{60}{60} = 30.5\text{ A}$.
  4. **Clamped to Global Mode Ceiling (`30.0 A`):** The motor delivers the full **`30.0 A`** (`~1500 W`).
  - *What if the rider's cadence drops to **`30 RPM`** on the steep grade?*
    Because of the $\frac{\text{Cadence}}{60}$ rule, $30.5\text{ A} \times \frac{30}{60} =$ **`15.2 A`**. Downshifting the bicycle's rear derailleur so crank cadence stays near `60 RPM` immediately unlocks the full **`30.0 A`**!

