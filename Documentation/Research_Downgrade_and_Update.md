# Firmware Update & Downgrade Research (public sources only)

Sources: `Documentation/Miraheze_Wiki/Firmware.md`, `Troubleshooting.md`, `General_FAQs.md`, ToSeven public update instructions.
Scope: documented behavior only. No analysis of vendor binaries or installer internals.

## Documented facts
- **One-way updates.** The wiki states that after installing DM01 `K241216-V6400C` (Dec 2024) or later, and DM02 firmware from Nov 2024, "it is not possible to roll back to an older version."
- **Documented error on rollback:** `MCU unidentified, try again or IAP version is lower or pack version is lower`. The wiki reports no known fix.
  - The wording names three conditions: MCU not identified, IAP (in-application-programming loader) version lower, pack version lower. The public text does not say which one applies. Treat it as unknown.
- **Update tool versions matter.** Several IAP tool releases exist (Dec 2023 v3.1, Aug 2024, Dec 2024 v5.1, Mar 2025 v5.2). A mismatch between tool and controller gives `connect fail`. Installing an intermediate version sometimes helps.
- **Display firmware (Dec 2024 / Jan 2025) locks out the Advanced menu** and is "unlikely to be reversible."
- **250W EN15194 DM02:** may not be restorable to the street-legal firmware after other firmware is installed.
- **Factory defaults changed** in newer firmware: hard-set amps, 32 km/h throttle-only and 40 km/h pedalling caps.
- **Programming cable:** faulty cables shipped Apr-Jul 2025 cause `Program Fail`. Recoverable with a good cable.

## Implications for our firmware (design requirements)
1. Our package must install on units that already have the latest stock firmware. This is the "pack version is lower" risk, so our version identifiers must not be lower than stock.
2. Our package must not raise any monotonic version or anti-rollback marker beyond what stock requires, so users can flash stock back.
3. A tested stock-restore path must be documented before any release.
4. Config storage must be forward and backward compatible, with a version tag and safe defaults.

## Open questions (need user-provided info or official vendor answers)
- DM01 controller MCU part number (not found in public sources so far).
- Which of the three error conditions causes the rollback refusal. Best route: ask ToSeven support or Discord, or test on a sacrificial unit.
- Whether ToSeven publishes any developer or SDK documentation for the IAP protocol.

## Other
- `analysis/` (untracked) holds a scratch extraction made earlier. It should be deleted and is not part of the project.
