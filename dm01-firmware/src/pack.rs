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
    pub payload_len: usize,
    pub initial_sp: u32,
    pub reset_handler: u32,
    pub decrypted_header: [u8; PACK_HEADER_LEN],
}

/// Errors returned when parsing or repacking a `.Pack` container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackError {
    BufferTooSmall,
    InvalidHeaderMagic,
    PayloadOutOfBounds,
    InvalidVectorTable,
}

/// Parses and validates a `.Pack` byte slice, auto-detecting the header and payload XOR keys.
pub fn parse_pack_metadata(pack_bytes: &[u8]) -> Result<PackMetadata, PackError> {
    if pack_bytes.len() < PACK_HEADER_LEN + 8 {
        return Err(PackError::BufferTooSmall);
    }

    // Decrypted header bytes [5..8] are always 0x00, so pack_bytes[5] is the header XOR key.
    let header_xor_key = pack_bytes[5];
    let mut decrypted_header = [0u8; PACK_HEADER_LEN];
    let mut i = 0;
    while i < PACK_HEADER_LEN {
        decrypted_header[i] = pack_bytes[i] ^ header_xor_key;
        i += 1;
    }

    if decrypted_header[16..20] != PACK_HEADER_MAGIC {
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

    // ARM Cortex-M4F vector table starts at pack_bytes[34..38] with Initial MSP (0x2000xxxx).
    let payload_xor_key = pack_bytes[PACK_HEADER_LEN + 3] ^ 0x20;

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

    Ok(PackMetadata {
        target,
        device_id_24,
        header_xor_key,
        payload_xor_key,
        payload_len,
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
/// updating the payload length in the encrypted header and preserving any existing tail padding
/// from `template_pack` (if provided) for bit-exact reproduction.
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
    }

    // Update header payload length and encrypt header with header_xor_key
    let mut hdr = meta.decrypted_header;
    let len_bytes = (payload.len() as u16).to_le_bytes();
    hdr[24] = len_bytes[0];
    hdr[25] = len_bytes[1];

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

        let mut payload = vec![0u8; meta.payload_len];
        decrypt_payload(&raw_pack, &meta, &mut payload).expect("decrypt DM01 payload");

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

        let mut payload = vec![0u8; meta.payload_len];
        decrypt_payload(&raw_pack, &meta, &mut payload).expect("decrypt DM02 payload");

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
}
