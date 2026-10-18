# Proprietary Assets & Binary Blobs (`blobs/`)

This directory is `.gitignore`d to maintain a **clean-room, legally unencumbered open-source repository** and avoid committing large binary files (such as the 156 MB Android APK, proprietary Windows utilities, or copyrighted OEM manuals) to Git.

All code in this repository (`dm01-firmware/`) is written from clean reverse-engineering analysis and does NOT require any proprietary binaries to compile.

---

## Directory Layout
```text
blobs/
├── apps/         # Android tuning APKs (cn.minshine.ebike)
├── displays/     # Display firmware dumps (.hex for MegaHunt MCU)
│   ├── lcd135c/  # LCD135C color display firmware
│   ├── m15/      # M15 display firmware
│   ├── t154/     # T154 display firmware
│   └── t24v/     # T24 vertical display firmware
├── firmware/     # Motor controller .Pack containers
│   ├── dm01/     # DM01 750W/1000W stock packages
│   └── dm02/     # DM02 250W/350W/500W stock packages
├── programming/  # Official vendor IAP & ISP flashing utilities
├── manuals/      # Official PDF user guides and documentation
└── archives/     # Original vendor zip/rar/7z archives (preserved for backup)
```

---

## Complete Catalog of Verified Assets & Checksums

### 1. DM01 Motor Controller Firmwares (`firmware/dm01/`)
| Release Name | File Path | SHA-256 Checksum | Description |
| :--- | :--- | :--- | :--- |
| **DM01 V6212C** | `firmware/dm01/TO7-DM01-K241021-V6212C.Pack` | `e81a9b49c4ed22a74da7e058138b83451b2ea1b468d577a138b27dca469a26d4` | V6212C release (Oct 21, 2024, 128 KB) |
| **DM01 V6510C** | `firmware/dm01/TO7-DM01-K250731-V6510C.Pack` | `0f9815339647faf9bcf46f58cae6d333285ddcfcc8abe8853ace9f631fd815e3` | V6510C release (Jul 31, 2025, 128 KB) |
| **DM01 V6510C** | `firmware/dm01/TO7-DM01-K250926-V6510C.Pack` | `3abd0a9ef915b2f40f40353bc82fcec730d4232f4891648d7b1b7ef04d30fbfd` | V6510C release (Sep 26, 2025, 128 KB) |
| **DM01 V6510C** | `firmware/dm01/TO7-DM01-K260108-V6510C.Pack` | `c0091ef7958928e6ccd684bf61a6e4b23a5d171db68a3e05892e7cf1d3b6a8ca` | V6510C release (Jan 08, 2026, 128 KB) |
| **DM01 V6510C (Latest)** | `firmware/dm01/TO7-DM01-K260521A-V6510C.Pack` | `eb964a1787ba90f35965921353911c8a6e3083e1914cf422caeaa515f36ee579` | Official V6510C latest production release (May 21, 2026, 128 KB) |

---

### 2. DM02 Motor Controller Firmwares (`firmware/dm02/`)
| Release Name | File Path | SHA-256 Checksum | Description |
| :--- | :--- | :--- | :--- |
| **DM02 V6206C** | `firmware/dm02/TO7-DM02-K240512A-V6206C.Pack` | `6b3abdddce50963cced5d205699fa6c71b84bec6e54940141803eeb8ae6051db` | V6206C release (May 12, 2024, 128 KB) |
| **DM02 V6306C** | `firmware/dm02/TO7-DM02-K240612B-V6306C.Pack` | `0b56ee38d790fde50930e7f1fcb76a080b2dc4b17063b262e311c4c10569f8fc` | V6306C release (Jun 12, 2024, 128 KB) |
| **DM02 V6212C** | `firmware/dm02/TO7-DM02-K241016-V6212C.Pack` | `983dc41b0f8d83a828bf2707a39b4a0c30c5a7846398a2d039d5790c084b048a` | V6212C release (Oct 16, 2024, 128 KB) |
| **DM02 V6303C** | `firmware/dm02/TO7-DM02-K241031-V6303C.Pack` | `4591837f809b2fa3f52a3c13083e8a5cc5d713f1b1d53cd49adaefb568ac112a` | V6303C release (Oct 31, 2024, 128 KB) |
| **DM02 V6400C** | `firmware/dm02/TO7-DM02-K241129A-V6400C.Pack` | `5088cac7b137128e29b672b366171501ca5fd8e8d428c99c0c032b0ec9e9d8ec` | V6400C release (Nov 29, 2024, 128 KB) |
| **DM02 V6400C** | `firmware/dm02/TO7-DM02-K250402-V6400C.Pack` | `c4fcead3e6239cc4b786f2d25601b22d9633af1fd2ef4863e2d333368507fd66` | V6400C release (Apr 02, 2025, 128 KB) |
| **DM02 V6510C** | `firmware/dm02/TO7-DM02-K250801-V6510C.Pack` | `36c61d4581b7042df1c05e3ff974a161d8b6ba8e5d93f612e11ab1c1b3f156ce` | V6510C release (Aug 01, 2025, 128 KB) |
| **DM02 V6510C** | `firmware/dm02/TO7-DM02-K260203-V6510C.Pack` | `7a75ab23c82fe918bc8086ec599cf2b1a9f976d66ab62daf5fce095b5f1b9035` | V6510C release (Feb 03, 2026, 128 KB) |
| **DM02 V6510C (Latest)** | `firmware/dm02/TO7-DM02-K260526-V6510C.Pack` | `3ab150d10d3106080233d4f41e7df6a13f67154994138d831df67b1094ea1ca2` | Official V6510C latest production release (May 26, 2026, 128 KB) |
| **TO7L-DM02 V6306C** | `firmware/dm02/TO7L-DM02-K240723A-V6306C.Pack` | `435aae3703971d92ee3a14342f0ae677491fb34fdc5f645bb919d41da40cb6a2` | V6306C release for TO7L series (Jul 23, 2024, 128 KB) |

---

### 3. Display Firmware Dumps (`displays/`)
| Display Model | File Path | SHA-256 Checksum | Description |
| :--- | :--- | :--- | :--- |
| **LCD135C** | `displays/lcd135c/TO7-LCD135C-Y240621A.hex` | `c529d8dcb5e65dbbbd5af874660c5e959f8c99a8823c56803bf51a2020343466` | LCD135C color display firmware dump |
| **M15** | `displays/m15/TO7-M15-Y250928.hex` | `924cc7dd259ef0db51affe048619d111f5e7be1dbf08ea98d50abb9ed2156aa8` | M15 display firmware dump |
| **T154** | `displays/t154/TO7-T154-Y250313.hex` | `827721dcea9d54b7145105f4250fd378afed662259e8a9508b122662944e0d7c` | T154 display firmware dump |
| **T24V (May 2024)** | `displays/t24v/TO7-T24V-Y240527A.hex` | `9f0130b6a0112ebd011e26d40501e59cc53511f3a314e54714989bc263bfe3f6` | T24 vertical display early dump |
| **T24V (Oct 2024)** | `displays/t24v/TO7-T24V-Y241016.hex` | `d3886f1238b2ff775ce7c5c3d9189549b60f979c6a3d94fe699149c97ad536b4` | T24 vertical display mid release |
| **T24V (Dec 2024)** | `displays/t24v/TO7-T24V-Y241225.hex` | `8e297200d6307f5cd8efc76e34712aa5d49e5032de9efed6b3822808fbb2d3d5` | T24 vertical display latest release |

---

### 4. Official Programming & Flashing Utilities (`programming/`)
| Utility | File Path | SHA-256 Checksum | Description |
| :--- | :--- | :--- | :--- |
| **TO7-G2 IAP Tool v5.2 (Factory)** | `programming/TO7-G2 IAP Tool Ver 5.2-CMD-20002-2025-3-27-Factory.exe` | `b6738dbd4df037eb8edd3bb0be0c182dd4b6d9e08fb48f4d8c9035687d8fd15c` | Official Delphi factory IAP flasher v5.2 (115200 Baud) |
| **TO7-G2 IAP Tool v5.1 (No Rollback)** | `programming/TO7-G2 IAP Tool Ver 5.1-20002-2-SKIP-VERSION-CHECK.exe` | `9075a0c70ab3ad9d5ee59c230f7c0782acc619f6171a2633f9aed08335f5b038` | Version check disabled utility for downgrade testing |
| **FOC IAP Tool v3.1** | `programming/FOC IAP Tool Ver 3.1-CMD-119-20002.exe` | `7298958cf20ae5c74e95ebbed74291d5f8588b0010273788b18938766929be5d` | Legacy Delphi flasher v3.1 (CMD 119) |
| **M4 FOC IAP Tool (C# / .NET)** | `programming/M4FOCIAP413CMD.exe` | `e3e443a6f30f2f91739433c629ba42598e45e45ed0f99a78662b7c60fe9ad9b9` | MinShine .NET flasher (38400 Baud) with complete debug PDB |
| **MegaHunt Display ISP Tool** | `programming/MH2020C_ISP.exe` | `c9d17cbf05933ae04f5115d19b11abe081dbdc455ce7f8f417a7bbc0b5bbaac4` | .NET ISP flasher for MegaHunt MH2020C / MH1903 displays |

---

### 5. Tuning Apps & Manuals
| Asset | Path | SHA-256 Checksum | Description |
| :--- | :--- | :--- | :--- |
| **Tuning App** | `apps/TOSEVEN_Professional_Tuning_v1-1.09_Certified-2025-12-31.apk.1` | `75a7bcfb92d80d238cb20d40e4e58ea961445763fcb0e43a998bb3a0349a4f4d` | ToSeven BLE tuning APK (156 MB) |
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
