# DM+ Project Rules (Kilo Code)

The canonical project reference is [`AGENTS.md`](../../AGENTS.md) at the repository root. Read it before any task. It covers the mission, workspace layout, decompiled `V6510C` architecture, build commands, and documentation policy. This file only adds what Kilo needs on top.

## Hard rules
- Always call the project **DM+** (DM+ Firmware & Tooling Studio).
- Keep `dm01-firmware` buildable as `#![no_std]`, with integer/fixed-point math in control paths. No heap allocations in the firmware library.
- Every change must pass all of these before it is committed:
  ```bash
  cargo test --manifest-path dm01-firmware/Cargo.toml
  cargo fmt --manifest-path dm01-firmware/Cargo.toml -- --check
  cargo clippy --manifest-path dm01-firmware/Cargo.toml --all-targets -- -D warnings
  cargo check --manifest-path dm01-firmware/Cargo.toml --no-default-features --lib
  cargo build --target thumbv7em-none-eabihf --manifest-path dm01-firmware/Cargo.toml --no-default-features --features embedded --bin dm01_firmware --bin dm02_firmware --release
  ```
- Record user-visible changes under `## [Unreleased]` in `CHANGELOG.md`. Also update `AGENTS.md` and the relevant `Documentation/*.md` when behavior changes.
- Use conventional commits (`feat(scope): ...`, `fix(...)`, `docs(...)`, `ci(...)`), with a bullet list in the body.
- Only push to `origin main` when the user asks. Do **not** tag or cut a release unless the user explicitly says so. Pushing a tag triggers `.github/workflows/release.yml`.
- `README.md`: no emojis, no ASCII UI diagrams, use `<details>` for long tables. Never claim the code was reviewed line by line by a human; say it was built with AI assistance plus automated tests, and that community bench testing is needed.
- `blobs/` holds proprietary assets and is gitignored. Never commit anything from it.
- Running modified firmware on a real bicycle is prohibited. All releases are bench-only Experimental Alpha.

## Hardware facts to respect
- Supported packs: **36V (10S), 48V (13S), 52V (14S)**. 60V is deprecated (it causes MOSFET breakdown on stock hardware). 72V is unsupported (it destroys the 80V FETs and the 63V/80V capacitors). Do not expose 60V or 72V in the GUI.
- DM01 current is hard-capped at 32 A and DM02 at 20 A (see `config.rs`).

## Reverse engineering
- The REA MCP server is configured in `.kilocode/mcp.json` (`rea-agents@6.7.0`, with Hopper as the provider). Before any reverse-engineering task, read the `reverse-engineer-anything` skill in `.kilocode/skills/`.
- Stock packs live in `blobs/firmware/dm0{1,2}/`. Unpack them with `toseven_pack unpack`. The payload loads at `0x08001800` (Thumb-2, Cortex-M4F). You can also disassemble with `lldb`, `rizin`, or `arm-none-eabi-objdump`.
- When a community member proposes a change to stock behavior, check it against the stock disassembly first, then implement it with tests.
