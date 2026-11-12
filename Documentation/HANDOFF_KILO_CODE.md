# Handoff: Antigravity (Gemini/Claude) → Kilo Code

Date: 2026-11-12. Repo: https://github.com/antaxiom/dm0x-plus-firmware (`main` @ `4484a3d`, clean tree, pushed).

## Where to start
1. Read `AGENTS.md`. It is the full technical reference (architecture, register maps, build commands, policies).
2. `.kilocode/rules/00-project.md` covers the hard rules (checks, commit/release policy, hardware limits).
3. `.kilocode/mcp.json` configures the REA reverse-engineering MCP server. It is pinned to `rea-agents@6.7.0`, matching the old Antigravity setup, and `HOPPER_LAUNCHER_PATH` points to Hopper in `~/Applications`. Change that path if Hopper lives somewhere else on the new machine. The REA skill and its reference guides were copied to `.kilocode/skills/reverse-engineer-anything/`.
4. `CHANGELOG.md` → `## [Unreleased]` lists everything merged since `v0.3.0-alpha.9`.

## Toolchain setup (new machine)
```bash
# Rust + embedded target
rustup toolchain install stable
rustup target add thumbv7em-none-eabihf
rustup component add clippy rustfmt

# Optional: disassembly / RE
brew install rizin arm-none-eabi-gcc   # or apt equivalents on Linux
npx -y rea-agents@latest doctor        # verify REA providers (Ghidra/Hopper/Rizin)

# Verify everything
cargo test --manifest-path dm01-firmware/Cargo.toml        # expect 89 passing (80 unit + 6 sim + 3 stock identity)
cargo build --manifest-path dm-plus-gui/Cargo.toml --release
```
Stock `.Pack` and display `.hex` files are **not** in git. Copy `blobs/` over from the old machine. SHA-256 hashes are listed in `blobs/README.md`, and some tests (stock identity, display patcher) need those files.

## Recent work (since alpha.9, unreleased)
| Commit | Change |
| :--- | :--- |
| `282ad9a` | Battery IR sag compensation (`V_ocv = V_term + I·R`, default 150 mΩ, stored in EEPROM Block 1 padding `0x04A+8..10`, with a GUI slider from 0 to 350 mΩ) and a τ≈1.02 s EMA (α=1/1024 at 1 kHz). Factory EEPROM images are now embedded in the firmware for self-commissioning. |
| `a09f547` | Adaptive `ZeroVol` torque drift tracking, checked against stock `V6510C` @ `0x08009034`. Samples are clamped to [400,1300] mV with a ±200 mV dynamic band. It keeps a 5-stop cycle history mean and latches once per stop to prevent red-light drift. Contributed by the community and cleaned up here. |
| `4484a3d` | Li-ion OCV SoC curves (3.15/3.45/3.65/3.85/4.00/4.15 V per cell). The 48V 0% point (40.95 V) matches the 41.0 V LVC. Added warnings for 60V/72V. |
| **Working tree (uncommitted)** | Community follow-up from Atak Snajperz: bounded automatic pack-resistance tracking (10 s window, `10..350 mΩ`, 3/4 EMA) and 60 s SoC boxcar averaging replace the 1.02 s EMA. Removed the entire `TuningProfile`/policy layer (`src/tuning.rs` deleted) in favor of compile-time `power_curve.rs` constants. Kept the gear-safe cadence ramp and 4.0 V true-zero LVC; switched torque to the linear mapping table and stock dual ceiling. Battery preset (36/48/52V) is now persisted in `DmPlusCustomConfig` at `0x04A+10`, restored at boot, and drives the SoC curve plus matching LVC. GUI and `toseven_pack create --battery` both patch the embedded image; `info` prints it. Tests: 89 firmware. |

## Open items / known gaps
- `README.md` "What Is Implemented Right Now" is current for the working-tree changes, but no release has been cut. The version header still reads "Unreleased, post-`v0.3.0-alpha.9`".
- No release has been cut for these changes. The user said: no new release yet.
- The design decisions on torque ADC resolution were discussed and kept as they are: mV scaling stays for parity with the stock firmware and display protocol; 8-bit quantization was rejected.
- All firmware is still unverified on hardware. Bench bring-up follows `Documentation/Bench_Testing_Bringup_Guide.md` (48V @ 1.0A current-limited supply).

## Working conventions the user expects
- Community contributions arrive as pasted code or attached files. Evaluate them against the stock disassembly, explain the trade-offs, ask before applying, then implement with tests.
- Be concise and technical. Prefer real reasoning over agreeing with the user.
- After implementing, ask "commit and push?" and only then commit and push. No tags.
