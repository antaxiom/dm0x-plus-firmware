# Third-Party Notices & Licensing

## Project License
The **DM+ Firmware & Tooling Studio** (`dm01-firmware` and `dm-plus-gui`) source code in this repository is licensed under the **MIT License**.

## Third-Party Dependencies (`dm-plus-gui`)

### 1. Slint UI Toolkit (`slint` / `slint-build`)
- **Repository:** https://github.com/slint-ui/slint
- **License:** Dual-licensed under **Royalty-Free Desktop, Mobile, and Web Applications License** / **GNU GPLv3** / **Paid Commercial License**.
- **Usage in DM+:** Used by the optional desktop companion application `dm-plus-gui` as a free open-source desktop application under Slint's Royalty-Free Desktop License (and compatible with GPLv3 distribution). Note that the core `#![no_std]` motor firmware and CLI crate (`dm01-firmware`) has **zero** external runtime dependencies and does not link against Slint.

### 2. Rusty File Dialogs (`rfd`)
- **Repository:** https://github.com/PolyMeilex/rfd
- **License:** MIT License
- **Usage in DM+:** Native file picker dialogs in `dm-plus-gui`.

### 3. `libc` (Unix POSIX Serial Bindings)
- **Repository:** https://github.com/rust-lang/libc
- **License:** MIT OR Apache-2.0
- **Usage in DM+:** Standard POSIX `termios` and `ioctl` bindings for serial port transport on macOS and Linux.

