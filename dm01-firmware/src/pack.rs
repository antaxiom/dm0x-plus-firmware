//! # ToSeven `.Pack` Firmware Container Unpacker & Repacker
//!
//! Handles decoding, inspecting, and repacking ToSeven `.Pack` firmware containers
//! (`TO7-DM01-K260521A-V6510C.Pack` and `TO7-DM02-K260526-V6510C.Pack`).
//!
//! ## `.Pack` Container Layout (131,072 bytes = `0x20000`)
//! - **Bytes `0x0000..0x0021` (34 bytes):** Encrypted Header (`header_xor_key` stored at `pack[5]`:
//!   `0xA8` for DM01, `0xF7` for DM02).
//!   - Decrypted `header[0..4]`: 32-bit little-endian Device/Board ID (`0x81196EA4` for DM01, `0x81196EA3` for DM02; lower 24 bits `0x00196EA4` / `0x00196EA3`).
//!   - Decrypted `header[16..20]`: Magic signature `8A E6 40 F4` (`0xF440E68A`).
//!   - Decrypted `header[24..26]`: 16-bit little-endian firmware payload length in bytes (`0xE16C = 57,708` for DM01, `0xE448 = 58,440` for DM02).
//! - **Bytes `0x0022..0x0022 + payload_len`:** ARM Cortex-M4F Application Binary loaded at Flash `0x08001800`,
//!   XOR-encrypted with `payload_xor_key` (`0x67` for DM01, `0xE2` for DM02; auto-detected via `pack[0x22 + 3] ^ 0x20`
//!   since initial `MSP` at `0x08001800` is in SRAM `0x2000xxxx`).
//! - **Bytes `0x0022 + payload_len..0x20000`:** High-entropy tail padding to 128 KB (`131,072` bytes).

use crate::eeprom::crc16_toseven;

/// Total size of a standard ToSeven `.Pack` file (128 KB).
pub const PACK_FILE_SIZE: usize = 0x20000;

/// Length of the `.Pack` file header in bytes.
pub const PACK_HEADER_LEN: usize = 34;

/// Flash load address of the application firmware image on the ARM Cortex-M4F MCU.
pub const APP_FLASH_BASE_ADDR: u32 = 0x0800_1800;

/// Expected 4-byte magic inside the decrypted `.Pack` header at `header[16..20]`.
pub const PACK_HEADER_MAGIC: [u8; 4] = [0x8A, 0xE6, 0x40, 0xF4];

/// Known ToSeven motor hardware targets identified from `.Pack` headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorTarget {
    /// ToSeven DM01 (750W / 1000W external controller, `TS1000W`, device ID `0x00196EA4`).
    Dm01,
    /// ToSeven DM02 (250W / 350W / 500W internal controller, `TS500W`, device ID `0x00196EA3`).
    Dm02,
    /// Other/unknown ToSeven target with raw 24-bit device ID.
    Unknown(u32),
}

impl MotorTarget {
    pub const fn from_device_id(id24: u32) -> Self {
        match id24 {
            0x0019_6EA4 => Self::Dm01,
            0x0019_6EA3 => Self::Dm02,
            other => Self::Unknown(other),
        }
    }

    pub const fn default_keys(self) -> (u8, u8) {
        match self {
            Self::Dm01 => (0xA8, 0x67),
            Self::Dm02 => (0xF7, 0xE2),
            Self::Unknown(_) => (0xA8, 0x67),
        }
    }
}

/// Decoded metadata from the 34-byte `.Pack` header and ARM Cortex-M4F vector table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackMetadata {
    pub target: MotorTarget,
    pub device_id_24: u32,
    pub header_xor_key: u8,
    pub payload_xor_key: u8,
    pub uart_mask: u8,
    pub payload_len: usize,
    pub payload_checksum: u32,
    pub header_crc: u16,
    pub initial_sp: u32,
    pub reset_handler: u32,
    pub decrypted_header: [u8; PACK_HEADER_LEN],
}

impl PackMetadata {
    /// Verifies that the cumulative 32-bit byte sum of `payload` matches the expected
    /// header checksum `((MCU_APP_ChkSumHigh << 16) | MCU_APP_ChkSumLow)`.
    pub fn verify_payload_checksum(&self, payload: &[u8]) -> bool {
        let mut sum = 0u32;
        let mut i = 0;
        while i < payload.len() {
            sum = sum.wrapping_add(payload[i] as u32);
            i += 1;
        }
        sum == self.payload_checksum
    }

    /// Returns the 16-bit OEM / Customer ID (`decrypted_header[1..3]`, e.g. `0x196E` = 6510 for V6510C).
    pub fn customer_num(&self) -> u16 {
        u16::from_le_bytes([self.decrypted_header[1], self.decrypted_header[2]])
    }
}

/// Errors returned when parsing or repacking a `.Pack` container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackError {
    BufferTooSmall,
    InvalidHeaderMagic,
    PayloadOutOfBounds,
    InvalidVectorTable,
}

/// Parses and validates a `.Pack` byte slice using ToSeven's universal key derivation
/// formulas verified across all official factory releases:
/// - `header_xor_key = (pack[32] + pack[33]) & 0xFF`
/// - `header_crc = decrypted_header[30..32]` (CRC-16-CCITT of `decrypted_header[0..30]`)
/// - `payload_xor_key = (decrypted_header[30] + decrypted_header[31]) & 0xFF`
/// - `payload_checksum = (decrypted_header[26..28] << 16) | decrypted_header[28..30]`
pub fn parse_pack_metadata(pack_bytes: &[u8]) -> Result<PackMetadata, PackError> {
    if pack_bytes.len() < PACK_HEADER_LEN + 8 {
        return Err(PackError::BufferTooSmall);
    }

    // Universal vendor formula: K_head = (pack[32] + pack[33]) % 256.
    // Fallback to pack_bytes[5] (since decrypted_header[5] is 0x00).
    let math_hdr_key = pack_bytes[32].wrapping_add(pack_bytes[33]);
    let header_xor_key = if math_hdr_key == pack_bytes[5] {
        math_hdr_key
    } else {
        pack_bytes[5]
    };

    let mut decrypted_header = [0u8; PACK_HEADER_LEN];
    let mut i = 0;
    while i < PACK_HEADER_LEN {
        decrypted_header[i] = pack_bytes[i] ^ header_xor_key;
        i += 1;
    }

    // The signature bytes at offset 17..19 are constant 0xE6, 0x40 across all ToSeven versions,
    // while byte 16 indicates build sequence (0x2D..0x8A) and byte 19 indicates package type (0xAA/0x00/0xF4).
    if decrypted_header[17..19] != [0xE6, 0x40] {
        return Err(PackError::InvalidHeaderMagic);
    }

    let device_id_24 = (decrypted_header[0] as u32)
        | ((decrypted_header[1] as u32) << 8)
        | ((decrypted_header[2] as u32) << 16);
    let target = MotorTarget::from_device_id(device_id_24);

    let payload_len = u16::from_le_bytes([decrypted_header[24], decrypted_header[25]]) as usize;
    if PACK_HEADER_LEN + payload_len > pack_bytes.len() {
        return Err(PackError::PayloadOutOfBounds);
    }

    let header_crc = u16::from_le_bytes([decrypted_header[30], decrypted_header[31]]);

    // Universal vendor formula: K_payload = (dec_hdr[30] + dec_hdr[31]) % 256.
    // Cortex-M4F vector table check: initial MSP (0x2000xxxx) at pack_bytes[34..38].
    let math_payload_key = decrypted_header[30].wrapping_add(decrypted_header[31]);
    let vec_payload_key = pack_bytes[PACK_HEADER_LEN + 3] ^ 0x20;
    let payload_xor_key = if math_payload_key == vec_payload_key {
        math_payload_key
    } else {
        vec_payload_key
    };

    let chk_hi = u16::from_le_bytes([decrypted_header[26], decrypted_header[27]]) as u32;
    let chk_lo = u16::from_le_bytes([decrypted_header[28], decrypted_header[29]]) as u32;
    let payload_checksum = (chk_hi << 16) | chk_lo;

    let mut vec_bytes = [0u8; 8];
    let mut j = 0;
    while j < 8 {
        vec_bytes[j] = pack_bytes[PACK_HEADER_LEN + j] ^ payload_xor_key;
        j += 1;
    }

    let initial_sp = u32::from_le_bytes([vec_bytes[0], vec_bytes[1], vec_bytes[2], vec_bytes[3]]);
    let reset_handler =
        u32::from_le_bytes([vec_bytes[4], vec_bytes[5], vec_bytes[6], vec_bytes[7]]);

    if (initial_sp & 0xFFFE_0000) != 0x2000_0000 || (reset_handler & 0xFF00_0000) != 0x0800_0000 {
        return Err(PackError::InvalidVectorTable);
    }

    let uart_mask = decrypted_header[0x17];

    Ok(PackMetadata {
        target,
        device_id_24,
        header_xor_key,
        payload_xor_key,
        uart_mask,
        payload_len,
        payload_checksum,
        header_crc,
        initial_sp,
        reset_handler,
        decrypted_header,
    })
}

/// Decrypts the application firmware payload from `pack_bytes` into `out_payload`.
/// `out_payload` must be at least `meta.payload_len` bytes long.
pub fn decrypt_payload(
    pack_bytes: &[u8],
    meta: &PackMetadata,
    out_payload: &mut [u8],
) -> Result<usize, PackError> {
    if out_payload.len() < meta.payload_len || pack_bytes.len() < PACK_HEADER_LEN + meta.payload_len
    {
        return Err(PackError::BufferTooSmall);
    }
    let src = &pack_bytes[PACK_HEADER_LEN..PACK_HEADER_LEN + meta.payload_len];
    let mut i = 0;
    while i < meta.payload_len {
        out_payload[i] = src[i] ^ meta.payload_xor_key;
        i += 1;
    }
    Ok(meta.payload_len)
}

/// Repacks a decrypted firmware `payload` into a `.Pack` container buffer (`out_pack`),
/// updating the payload length, cumulative 32-bit checksum, and header CCITT CRC,
/// preserving any existing tail padding from `template_pack` (if provided) for bit-exact reproduction.
pub fn repack_firmware(
    meta: &PackMetadata,
    payload: &[u8],
    template_pack: Option<&[u8]>,
    out_pack: &mut [u8; PACK_FILE_SIZE],
) -> Result<(), PackError> {
    if PACK_HEADER_LEN + payload.len() > PACK_FILE_SIZE {
        return Err(PackError::PayloadOutOfBounds);
    }

    if let Some(tpl) = template_pack {
        if tpl.len() == PACK_FILE_SIZE {
            out_pack.copy_from_slice(tpl);
        }
    } else {
        // When generating a brand-new container from scratch without a template, set bytes 32..33
        // such that (out_pack[32] + out_pack[33]) % 256 == meta.header_xor_key.
        out_pack[32] = 0x0E;
        out_pack[33] = meta.header_xor_key.wrapping_sub(0x0E);
    }

    // Update header payload length, checksum, and CRC
    let mut hdr = meta.decrypted_header;
    let len_bytes = (payload.len() as u16).to_le_bytes();
    hdr[24] = len_bytes[0];
    hdr[25] = len_bytes[1];

    let sum = payload
        .iter()
        .fold(0u32, |acc, &b| acc.wrapping_add(b as u32));
    hdr[26..28].copy_from_slice(&((sum >> 16) as u16).to_le_bytes());
    hdr[28..30].copy_from_slice(&(sum as u16).to_le_bytes());

    let crc = crc16_toseven(&hdr[0..30]);
    hdr[30..32].copy_from_slice(&crc.to_le_bytes());

    let mut i = 0;
    while i < PACK_HEADER_LEN {
        out_pack[i] = hdr[i] ^ meta.header_xor_key;
        i += 1;
    }

    // Encrypt firmware payload with payload_xor_key
    let mut j = 0;
    while j < payload.len() {
        out_pack[PACK_HEADER_LEN + j] = payload[j] ^ meta.payload_xor_key;
        j += 1;
    }

    Ok(())
}

/// Extracts the raw contiguous Flash payload from an ELF32 binary (little-endian ARM)
/// linked for execution starting at `APP_FLASH_BASE_ADDR` (0x08001800).
#[cfg(feature = "std")]
pub fn extract_flash_from_elf(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.len() < 52 || &data[0..4] != b"\x7fELF" {
        return Err("Not a valid ELF binary");
    }
    // Check 32-bit (data[4] == 1) and little-endian (data[5] == 1)
    if data[4] != 1 || data[5] != 1 {
        return Err("ELF is not 32-bit little-endian ARM");
    }

    let phoff = u32::from_le_bytes(data[0x1C..0x20].try_into().unwrap()) as usize;
    let phentsize = u16::from_le_bytes(data[0x2A..0x2C].try_into().unwrap()) as usize;
    let phnum = u16::from_le_bytes(data[0x2C..0x2E].try_into().unwrap()) as usize;

    let mut segments = Vec::new();
    let mut min_addr = u32::MAX;
    let mut max_addr = 0u32;

    for i in 0..phnum {
        let entry_offset = phoff + i * phentsize;
        if entry_offset + 32 > data.len() {
            break;
        }
        let seg_bytes = &data[entry_offset..entry_offset + 32];
        let p_type = u32::from_le_bytes(seg_bytes[0..4].try_into().unwrap());
        let p_offset = u32::from_le_bytes(seg_bytes[4..8].try_into().unwrap()) as usize;
        let p_paddr = u32::from_le_bytes(seg_bytes[12..16].try_into().unwrap());
        let p_filesz = u32::from_le_bytes(seg_bytes[16..20].try_into().unwrap()) as usize;

        // PT_LOAD = 1
        if p_type == 1 && p_filesz > 0 {
            // Flash load segments (e.g. 0x08001800..0x08080000)
            if (APP_FLASH_BASE_ADDR..0x0808_0000).contains(&p_paddr) {
                let seg_end = p_paddr + p_filesz as u32;
                if p_paddr < min_addr {
                    min_addr = p_paddr;
                }
                if seg_end > max_addr {
                    max_addr = seg_end;
                }
                segments.push((p_paddr, p_offset, p_filesz));
            }
        }
    }

    if segments.is_empty() {
        return Err("No PT_LOAD segments found in Flash region (>= 0x08001800)");
    }

    let base = APP_FLASH_BASE_ADDR;
    let total_size = (max_addr - base) as usize;
    let mut flash_image = vec![0xFFu8; total_size];

    for (paddr, offset, filesz) in segments {
        let dst_start = (paddr - base) as usize;
        let dst_end = dst_start + filesz;
        let src_end = offset + filesz;
        if src_end <= data.len() && dst_end <= flash_image.len() {
            flash_image[dst_start..dst_end].copy_from_slice(&data[offset..src_end]);
        }
    }

    Ok(flash_image)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::eeprom::{EepromImage, EEPROM_TOTAL_LEN};
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_dm01_pack_unpack_and_bit_exact_repack() {
        let candidates = [
            "../blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack",
            "blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack",
        ];
        let pack_path = candidates.iter().map(Path::new).find(|p| p.exists());
        let Some(pack_path) = pack_path else {
            return;
        };
        let raw_pack = fs::read(pack_path).expect("read DM01 .Pack");
        assert_eq!(raw_pack.len(), PACK_FILE_SIZE);

        let meta = parse_pack_metadata(&raw_pack).expect("valid DM01 Pack metadata");
        assert_eq!(meta.target, MotorTarget::Dm01);
        assert_eq!(meta.header_xor_key, 0xA8);
        assert_eq!(meta.payload_xor_key, 0x67);
        assert_eq!(meta.payload_len, 57_708);
        assert_eq!(meta.initial_sp, 0x2000_5628);
        assert_eq!(meta.reset_handler, 0x0800_1A51);

        assert_eq!(meta.header_crc, 0x1651);
        assert_eq!(meta.payload_checksum, 0x0059_E35F);

        let mut payload = vec![0u8; meta.payload_len];
        decrypt_payload(&raw_pack, &meta, &mut payload).expect("decrypt DM01 payload");
        assert!(meta.verify_payload_checksum(&payload));

        // Verify the embedded 480-byte factory EEPROM table at 0x0800EE70 inside the decrypted binary
        let eeprom_offset = (0x0800_EE70 - APP_FLASH_BASE_ADDR) as usize;
        let eeprom_slice: &[u8; EEPROM_TOTAL_LEN] = payload
            [eeprom_offset..eeprom_offset + EEPROM_TOTAL_LEN]
            .try_into()
            .unwrap();
        let eeprom = EepromImage { raw: *eeprom_slice };
        assert_eq!(eeprom.verify_all_blocks(), Ok(()));
        assert_eq!(eeprom, EepromImage::default_dm01());

        // Verify bit-for-bit identical round-trip repack
        let mut repacked = [0u8; PACK_FILE_SIZE];
        repack_firmware(&meta, &payload, Some(&raw_pack), &mut repacked)
            .expect("repack DM01 firmware");
        assert_eq!(&repacked[..], &raw_pack[..]);
    }

    #[test]
    fn test_dm02_pack_unpack_and_bit_exact_repack() {
        let candidates = [
            "../blobs/firmware/dm02/TO7-DM02-K260526-V6510C.Pack",
            "blobs/firmware/dm02/TO7-DM02-K260526-V6510C.Pack",
        ];
        let pack_path = candidates.iter().map(Path::new).find(|p| p.exists());
        let Some(pack_path) = pack_path else {
            return;
        };
        let raw_pack = fs::read(pack_path).expect("read DM02 .Pack");
        assert_eq!(raw_pack.len(), PACK_FILE_SIZE);

        let meta = parse_pack_metadata(&raw_pack).expect("valid DM02 Pack metadata");
        assert_eq!(meta.target, MotorTarget::Dm02);
        assert_eq!(meta.header_xor_key, 0xF7);
        assert_eq!(meta.payload_xor_key, 0xE2);
        assert_eq!(meta.payload_len, 58_440);
        assert_eq!(meta.initial_sp, 0x2000_5620);
        assert_eq!(meta.reset_handler, 0x0800_1A51);
        assert_eq!(meta.header_crc, 0x459D);
        assert_eq!(meta.payload_checksum, 0x005A_E79E);

        let mut payload = vec![0u8; meta.payload_len];
        decrypt_payload(&raw_pack, &meta, &mut payload).expect("decrypt DM02 payload");
        assert!(meta.verify_payload_checksum(&payload));

        // Verify the embedded 480-byte factory EEPROM table at 0x0800F142 inside the decrypted binary
        let eeprom_offset = (0x0800_F142 - APP_FLASH_BASE_ADDR) as usize;
        let eeprom_slice: &[u8; EEPROM_TOTAL_LEN] = payload
            [eeprom_offset..eeprom_offset + EEPROM_TOTAL_LEN]
            .try_into()
            .unwrap();
        let eeprom = EepromImage { raw: *eeprom_slice };
        assert_eq!(eeprom.verify_all_blocks(), Ok(()));
        assert_eq!(eeprom, EepromImage::default_dm02());

        // Verify bit-for-bit identical round-trip repack
        let mut repacked = [0u8; PACK_FILE_SIZE];
        repack_firmware(&meta, &payload, Some(&raw_pack), &mut repacked)
            .expect("repack DM02 firmware");
        assert_eq!(&repacked[..], &raw_pack[..]);
    }

    #[test]
    fn test_all_15_factory_packs_universal_formulas() {
        let search_dirs = ["blobs/firmware", "../blobs/firmware"];
        let mut found_count = 0;

        for base in &search_dirs {
            let base_path = Path::new(base);
            if !base_path.exists() {
                continue;
            }
            for model_dir in &["dm01", "dm02"] {
                let dir = base_path.join(model_dir);
                if !dir.exists() {
                    continue;
                }
                let entries = fs::read_dir(dir).expect("read firmware dir");
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("Pack") {
                        let raw = fs::read(&path).expect("read .Pack");
                        assert_eq!(raw.len(), PACK_FILE_SIZE, "File size of {:?}", path);

                        let meta = parse_pack_metadata(&raw)
                            .unwrap_or_else(|e| panic!("Failed parsing {:?}: {:?}", path, e));

                        // 1. Mathematical Header Key formula: (pack[32] + pack[33]) % 256
                        let math_hdr_key = raw[32].wrapping_add(raw[33]);
                        assert_eq!(
                            meta.header_xor_key, math_hdr_key,
                            "Header key formula mismatch for {:?}",
                            path
                        );

                        // 2. Header CCITT CRC: crc16_toseven(decrypted_header[0..30])
                        let computed_crc = crc16_toseven(&meta.decrypted_header[0..30]);
                        assert_eq!(
                            meta.header_crc, computed_crc,
                            "Header CRC mismatch for {:?}",
                            path
                        );

                        // 3. Mathematical Payload Key formula: (dec_hdr[30] + dec_hdr[31]) % 256
                        let math_pay_key =
                            meta.decrypted_header[30].wrapping_add(meta.decrypted_header[31]);
                        assert_eq!(
                            meta.payload_xor_key, math_pay_key,
                            "Payload key formula mismatch for {:?}",
                            path
                        );

                        // 4. Decrypt and verify 32-bit cumulative byte sum
                        let mut payload = vec![0u8; meta.payload_len];
                        decrypt_payload(&raw, &meta, &mut payload).expect("decrypt payload");
                        assert!(
                            meta.verify_payload_checksum(&payload),
                            "Payload 32-bit checksum mismatch for {:?}",
                            path
                        );

                        found_count += 1;
                    }
                }
            }
        }

        if found_count > 0 {
            assert!(
                found_count >= 15,
                "Expected at least 15 factory packs, found {}",
                found_count
            );
        }
    }

    #[test]
    fn test_extract_flash_from_elf_synthetic() {
        // Construct minimal valid ELF32
        let mut elf = vec![0u8; 128];
        elf[0..4].copy_from_slice(b"\x7fELF");
        elf[4] = 1; // 32-bit
        elf[5] = 1; // little endian
        elf[6] = 1; // version
                    // e_phoff at 0x1C = 0x34
        elf[0x1C..0x20].copy_from_slice(&0x34u32.to_le_bytes());
        // e_phentsize at 0x2A = 32
        elf[0x2A..0x2C].copy_from_slice(&32u16.to_le_bytes());
        // e_phnum at 0x2C = 1
        elf[0x2C..0x2E].copy_from_slice(&1u16.to_le_bytes());

        // Program header at 0x34 (32 bytes):
        // p_type = 1 (PT_LOAD)
        elf[0x34..0x38].copy_from_slice(&1u32.to_le_bytes());
        // p_offset = 0x60
        elf[0x38..0x3C].copy_from_slice(&0x60u32.to_le_bytes());
        // p_vaddr = 0x08001800
        elf[0x3C..0x40].copy_from_slice(&0x0800_1800u32.to_le_bytes());
        // p_paddr = 0x08001800
        elf[0x40..0x44].copy_from_slice(&0x0800_1800u32.to_le_bytes());
        // p_filesz = 16
        elf[0x44..0x48].copy_from_slice(&16u32.to_le_bytes());
        // p_memsz = 16
        elf[0x48..0x4C].copy_from_slice(&16u32.to_le_bytes());

        // Payload data at 0x60..0x70
        elf[0x60..0x64].copy_from_slice(&0x2000_5628u32.to_le_bytes()); // MSP
        elf[0x64..0x68].copy_from_slice(&0x0800_1851u32.to_le_bytes()); // Reset

        let flash = extract_flash_from_elf(&elf).expect("extract synthetic ELF");
        assert_eq!(flash.len(), 16);
        assert_eq!(&flash[0..4], &0x2000_5628u32.to_le_bytes());
        assert_eq!(&flash[4..8], &0x0800_1851u32.to_le_bytes());
    }
}
