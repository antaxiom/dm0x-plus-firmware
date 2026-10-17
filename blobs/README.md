# Proprietary Assets & Binary Blobs (`blobs/`)

This directory is `.gitignore`d to maintain a **clean-room, legally unencumbered open-source repository** and avoid committing large binary files (such as the 156 MB Android APK or copyrighted OEM manuals) to Git.

All code in this repository (`dm01-firmware/`) is written from clean reverse-engineering analysis and does NOT require any proprietary binaries to compile.

---

## Directory Layout
```text
blobs/
├── apps/         # Android tuning APKs
├── displays/     # Display firmware dumps (.hex)
├── firmware/     # Motor controller .Pack containers
│   ├── dm01/     # DM01 750W/1000W stock packages
│   └── dm02/     # DM02 250W/350W/500W stock packages
├── manuals/      # Official PDF user guides and documentation
└── archives/     # Original vendor zip archives (if kept for backup)
```

---

## Catalog of Verified Assets & Checksums

| Asset | Path | SHA-256 Checksum | Description |
| :--- | :--- | :--- | :--- |
| **DM01 Stock Firmware** | `firmware/dm01/TO7-DM01-K260521A-V6510C.Pack` | `eb964a1787ba90f35965921353911c8a6e3083e1914cf422caeaa515f36ee579` | Official V6510C release (128 KB) |
| **DM02 Stock Firmware** | `firmware/dm02/TO7-DM02-K260526-V6510C.Pack` | `3ab150d10d3106080233d4f41e7df6a13f67154994138d831df67b1094ea1ca2` | Official V6510C release (128 KB) |
| **LCD135C Firmware** | `displays/TO7-LCD135C-Y240621A.hex` | `f0f9bcfbe7a39ba50b0709088658a2d1f057813ba58c14f6b0f1faecaa7ddb30` | LCD135C color display dump |
| **M15 Display Firmware** | `displays/TO7-M15-Y250928.hex` | `5e683f120150bcfd9c223c6237beee095a0a4c28f731174620f4ea4cbe7cf3db` | M15 display dump |
| **T154 Display Firmware** | `displays/TO7-T154-Y250313.hex` | `b33b0e9d6d3d9ec5db2be7833077e641ea55fa4ec47184499aa69b4e334df9df` | T154 display dump |
| **T24V Display Firmware** | `displays/TO7-T24V-Y241225.hex` | `60b459fcbe08e3eb6ad961cb054ceb84930164624389dfae2474fe01b33bfbcf` | T24 vertical display dump |
| **Tuning App** | `apps/TOSEVEN_Professional_Tuning_v1-1.09_Certified-2025-12-31.apk.1` | `75a7bcfb92d80d238cb20d40e4e58ea961445763fcb0e43a998bb3a0349a4f4d` | ToSeven tuning APK (156 MB) |
| **DM01 Manual** | `manuals/DM01-User-Manual-v2.4.pdf` | `e2a6d5483a967c7e39fc17ea66df981df8d8ec9215c0e1818d6a89c36fc9cb08` | User manual v2.4 (PDF) |
| **DM02 Manual** | `manuals/DM02-User-Manual-v2.4.pdf` | `d7ea6d400e9ec19ef4f85e486ea7be9f658e49ee07c7cb17c7689949ee1481b6` | User manual v2.4 (PDF) |
| **T24 Display Manual** | `manuals/T24-Display-Manual.pdf` | `8c1d5300e1cf065679dc6eb9d454659bbafe9a706591079366d2ce5aa91185aa` | T24 display manual (PDF) |
| **LCD135C Manual** | `manuals/LCD135C-Bluetooth-User-Guide.pdf` | `10bc93f2f8aee46067b5797f267756aebe547fcaebf3e691ba5f22f51f496155` | LCD135C manual (PDF) |

---

## Tooling Reference

The CLI tool `toseven_pack` searches the following default paths if specific arguments are not given:
- DM01 default: `blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack`
- DM02 default: `blobs/firmware/dm02/TO7-DM02-K260526-V6510C.Pack`

Or via environment variable:
```bash
export TOSEVEN_BLOBS_DIR="/path/to/external/storage/blobs"
```

