//! # Flash EEPROM Configuration Storage & ToSeven Nibble CRC-16
//!
//! Implements the exact 5-block (480-byte) Flash EEPROM layout stored at `0x0801FC00`,
//! including the `0xA5` byte-wise XOR cipher, the `A5 01 FE` header magic check,
//! the 16-entry nibble-table CRC-16 (`crc16_toseven`), and the bit-exact factory default
//! EEPROM images extracted from both the DM01 (`TS1000W`) and DM02 (`TS500W`) `V6510C` binaries.

/// Flash address where the 480-byte configuration image is persisted on the MCU.
pub const EEPROM_FLASH_BASE_ADDR: u32 = 0x0801_FC00;

/// Total size in bytes of the 5-block EEPROM configuration image (`0x1E0 = 480` bytes).
pub const EEPROM_TOTAL_LEN: usize = 480;

/// XOR key applied to every byte stored in Flash EEPROM (`0x0801FC00..0x0801FDDF`).
pub const EEPROM_XOR_KEY: u8 = 0xA5;

/// Sizes of each of the 5 independently CRC-protected blocks in the 480-byte EEPROM image.
pub const EEPROM_BLOCK_SIZES: [usize; 5] = [0x20, 0x40, 0x40, 0xA0, 0xA0];

/// Starting byte offsets of each of the 5 blocks within the 480-byte EEPROM image.
pub const EEPROM_BLOCK_OFFSETS: [usize; 5] = [0x000, 0x020, 0x060, 0x0A0, 0x140];

/// Expected 3-byte magic signature at the end of Block 1 (`0x05A..0x05D` = `Block1[0x3A..0x3D]`).
pub const EEPROM_BLOCK1_MAGIC: [u8; 3] = [0xA5, 0x01, 0xFE];

/// Exact 16-entry nibble lookup table for the ToSeven CRC-16 (`0x0800EE50` in DM01 V6510C).
pub const CRC16_TO7_TABLE: [u16; 16] = [
    0x0000, 0x1021, 0x2042, 0x3063, 0x4084, 0x50A5, 0x60C6, 0x70E7, 0x8108, 0x9129, 0xA14A, 0xB16B,
    0xC18C, 0xD1AD, 0xE1CE, 0xF1EF,
];

/// Computes the exact ToSeven 16-bit nibble-table CRC (`0x080021A4` in DM01 V6510C).
///
/// Uses polynomial `0x1021` with initial value `0x0000`, processing the high nibble
/// followed by the low nibble of each byte.
pub const fn crc16_toseven(data: &[u8]) -> u16 {
    let mut crc: u16 = 0x0000;
    let mut i = 0;
    while i < data.len() {
        let b = data[i];
        let hi_idx = (((crc >> 12) as u8) ^ (b >> 4)) & 0x0F;
        crc = (crc << 4) ^ CRC16_TO7_TABLE[hi_idx as usize];
        let lo_idx = (((crc >> 12) as u8) ^ (b & 0x0F)) & 0x0F;
        crc = (crc << 4) ^ CRC16_TO7_TABLE[lo_idx as usize];
        i += 1;
    }
    crc
}

/// Bit-exact 480-byte factory default EEPROM image extracted from `TO7-DM01-K260521A-V6510C.Pack` (`TS1000W`).
pub const DM01_V6510C_FACTORY_EEPROM: [u8; EEPROM_TOTAL_LEN] = [
    0x28, 0x00, 0x64, 0x00, 0x00, 0x00, 0x5C, 0x0D, 0x8A, 0x02, 0xFD, 0x00, 0x32, 0x00, 0xF7, 0x0B,
    0x58, 0x02, 0xE8, 0x03, 0x0A, 0x86, 0x00, 0x49, 0xF2, 0x59, 0x03, 0x00, 0x00, 0x00, 0x04, 0x14,
    0x54, 0x53, 0x31, 0x30, 0x30, 0x30, 0x57, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x6E, 0x19, 0x04, 0xBC, 0x02, 0x90, 0x01, 0xDE, 0x03, 0x01, 0x31, 0x38,
    0x11, 0x31, 0x30, 0x33, 0xF2, 0x03, 0xF2, 0x03, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA5, 0x01, 0xFE, 0x00, 0x4A, 0x6D,
    0x54, 0x53, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x32, 0x34, 0x31, 0x31, 0x31, 0x31,
    0x30, 0x30, 0x30, 0x30, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x53, 0x69, 0x74,
    0x67, 0x6F, 0x2D, 0x52, 0x65, 0x61, 0x72, 0x20, 0x31, 0x37, 0x30, 0x30, 0x57, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x79, 0xFD,
    0x04, 0x04, 0x02, 0xB9, 0x64, 0x64, 0x64, 0x00, 0x00, 0x00, 0x00, 0x37, 0x5C, 0x0A, 0xDC, 0x05,
    0x6E, 0x19, 0x32, 0x00, 0x1A, 0x00, 0x00, 0x28, 0x3C, 0x50, 0x14, 0x1E, 0x28, 0x00, 0x13, 0x26,
    0x45, 0x15, 0x46, 0x23, 0x00, 0x00, 0x64, 0x51, 0x42, 0x13, 0x26, 0x45, 0x01, 0x01, 0x05, 0x00,
    0x00, 0x96, 0x01, 0xEE, 0x02, 0xC8, 0x00, 0x1E, 0x00, 0x0F, 0x0A, 0x08, 0x02, 0x01, 0x00, 0x64,
    0xBC, 0x02, 0xC8, 0x1E, 0x05, 0x02, 0x05, 0x02, 0x01, 0x64, 0x00, 0x01, 0x00, 0x00, 0x00, 0x8C,
    0x0A, 0x00, 0x96, 0x00, 0x2C, 0x01, 0x14, 0x0A, 0x19, 0x04, 0x96, 0x00, 0x5E, 0x01, 0x80, 0x01,
    0xA6, 0x00, 0x99, 0x00, 0x01, 0x05, 0x58, 0x02, 0xF4, 0x01, 0xFE, 0x01, 0x7C, 0x02, 0x84, 0x02,
    0x92, 0x02, 0xA5, 0x02, 0xCA, 0x02, 0x68, 0x01, 0x2C, 0x01, 0x2C, 0x01, 0x5F, 0x01, 0x69, 0x01,
    0x76, 0x01, 0x89, 0x01, 0xA4, 0x01, 0xE0, 0x01, 0x90, 0x01, 0x90, 0x01, 0xC8, 0x01, 0xD5, 0x01,
    0xE6, 0x01, 0xFF, 0x01, 0x22, 0x02, 0x2C, 0x01, 0x64, 0x02, 0xFA, 0x00, 0x0A, 0x3C, 0x50, 0xBE,
    0x1A, 0x00, 0x52, 0x03, 0x1E, 0x00, 0x3C, 0x00, 0x02, 0x01, 0x96, 0x00, 0x14, 0xC8, 0x00, 0x1E,
    0x28, 0x3C, 0x50, 0x64, 0x02, 0x2C, 0x01, 0x01, 0x64, 0x01, 0x02, 0x01, 0x01, 0x00, 0x03, 0x00,
    0x93, 0x01, 0x01, 0x64, 0x00, 0x00, 0x0A, 0x01, 0x05, 0x00, 0x00, 0x00, 0x06, 0x06, 0x0F, 0x14,
    0x20, 0x28, 0x28, 0x37, 0x50, 0x1C, 0x0E, 0x14, 0x28, 0x3C, 0x50, 0x64, 0x64, 0x64, 0x64, 0x64,
    0x03, 0x0C, 0x01, 0x02, 0x0A, 0x28, 0x04, 0x64, 0x58, 0x02, 0x01, 0x07, 0x14, 0x05, 0x54, 0x0B,
    0x14, 0x05, 0x3C, 0x00, 0x01, 0x01, 0xC8, 0x00, 0xC8, 0x00, 0x02, 0x01, 0x01, 0x01, 0x00, 0xBC,
    0x02, 0xE4, 0x0C, 0x00, 0x64, 0x01, 0x1E, 0x00, 0x01, 0x00, 0x01, 0x00, 0x82, 0x00, 0x3C, 0x14,
    0x05, 0x3C, 0x0A, 0x05, 0x00, 0x01, 0x32, 0x00, 0x1E, 0x00, 0x0A, 0xC2, 0x01, 0x01, 0x0F, 0x01,
    0xFF, 0xFF, 0x00, 0x01, 0x12, 0x0A, 0x1E, 0x05, 0x0C, 0x00, 0x3C, 0x64, 0x00, 0x02, 0x00, 0x46,
    0x2E, 0x20, 0x0B, 0x0A, 0x28, 0x32, 0x00, 0x00, 0x01, 0x28, 0x08, 0x1E, 0xA0, 0x00, 0x69, 0x68,
];

/// Bit-exact 480-byte factory default EEPROM image extracted from `TO7-DM02-K260526-V6510C.Pack` (`TS500W`).
pub const DM02_V6510C_FACTORY_EEPROM: [u8; EEPROM_TOTAL_LEN] = [
    0x2C, 0x00, 0x64, 0x00, 0x00, 0x00, 0x5C, 0x0D, 0xBC, 0x02, 0xFD, 0x00, 0x32, 0x00, 0xF7, 0x0B,
    0x58, 0x02, 0xE8, 0x03, 0x0A, 0x86, 0x00, 0xF9, 0xF1, 0x63, 0x03, 0x00, 0x00, 0x00, 0xB6, 0xED,
    0x54, 0x53, 0x35, 0x30, 0x30, 0x57, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x6E, 0x19, 0x04, 0x90, 0x01, 0xB4, 0x00, 0x26, 0x02, 0x01, 0x31, 0x38,
    0x31, 0x31, 0x30, 0x33, 0xF2, 0x03, 0xF2, 0x03, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA5, 0x01, 0xFE, 0x00, 0xD7, 0x33,
    0x54, 0x53, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x32, 0x33, 0x30, 0x38, 0x30, 0x31,
    0x32, 0x33, 0x30, 0x38, 0x30, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x53, 0x69, 0x74,
    0x67, 0x6F, 0x2D, 0x52, 0x65, 0x61, 0x72, 0x20, 0x31, 0x35, 0x30, 0x30, 0x57, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x90, 0x03,
    0x04, 0x04, 0x02, 0xB9, 0x64, 0x64, 0x96, 0x00, 0x00, 0x00, 0x00, 0x23, 0x5A, 0x0A, 0xE8, 0x03,
    0x6E, 0x19, 0x85, 0x02, 0x1A, 0x00, 0x00, 0x32, 0x32, 0x32, 0x1E, 0x3C, 0x64, 0x00, 0x13, 0x26,
    0x45, 0x15, 0x46, 0x23, 0x00, 0x00, 0x10, 0x0C, 0x42, 0x13, 0x26, 0x45, 0x01, 0x01, 0x05, 0x00,
    0x00, 0x00, 0x00, 0x44, 0x02, 0xC8, 0x00, 0x1E, 0x00, 0x0F, 0x0A, 0x08, 0x80, 0x80, 0x00, 0x64,
    0xBC, 0x02, 0xC8, 0x1E, 0x05, 0x02, 0x05, 0x02, 0x01, 0x64, 0x00, 0x01, 0x00, 0x01, 0x00, 0x74,
    0x0E, 0x00, 0x96, 0x00, 0x2C, 0x01, 0x0D, 0x0A, 0x19, 0x04, 0x96, 0x00, 0x5E, 0x01, 0x80, 0x01,
    0x42, 0x00, 0x3F, 0x00, 0x00, 0x05, 0x58, 0x02, 0xF4, 0x01, 0xFE, 0x01, 0x7C, 0x02, 0x84, 0x02,
    0x92, 0x02, 0xA5, 0x02, 0xCA, 0x02, 0x68, 0x01, 0x2C, 0x01, 0x2C, 0x01, 0x5F, 0x01, 0x69, 0x01,
    0x76, 0x01, 0x89, 0x01, 0xA4, 0x01, 0xE0, 0x01, 0x90, 0x01, 0x90, 0x01, 0xC8, 0x01, 0xD5, 0x01,
    0xE6, 0x01, 0x00, 0x02, 0x22, 0x02, 0x2C, 0x01, 0x0A, 0x02, 0xE8, 0x03, 0x05, 0x64, 0x28, 0xC5,
    0x1A, 0x00, 0x26, 0x02, 0x1E, 0x00, 0x3C, 0x00, 0x02, 0x01, 0x96, 0x00, 0x0A, 0xC8, 0x00, 0x1E,
    0x32, 0x3C, 0x50, 0x64, 0x01, 0xB4, 0x00, 0x01, 0x64, 0x01, 0x02, 0x01, 0x01, 0x00, 0x05, 0x00,
    0x12, 0x02, 0x00, 0x64, 0x00, 0x00, 0x0A, 0x01, 0x05, 0x00, 0x00, 0x00, 0x06, 0x0A, 0x0F, 0x14,
    0x20, 0x28, 0x28, 0x28, 0x28, 0x1C, 0x0E, 0x19, 0x23, 0x32, 0x46, 0x64, 0x64, 0x64, 0x64, 0x64,
    0x03, 0x0C, 0x01, 0x01, 0x0A, 0x28, 0x05, 0x64, 0x58, 0x02, 0x01, 0x07, 0x14, 0x05, 0x54, 0x0B,
    0x02, 0x05, 0x1E, 0x00, 0x01, 0x00, 0xC8, 0x00, 0xC8, 0x00, 0x02, 0x01, 0x01, 0x01, 0x00, 0xBC,
    0x02, 0xE4, 0x0C, 0x00, 0x64, 0x01, 0x1E, 0x00, 0x01, 0x00, 0x01, 0x01, 0x69, 0x00, 0x3C, 0x14,
    0x05, 0x3C, 0x0A, 0x05, 0x00, 0x01, 0x1E, 0x00, 0x1E, 0x00, 0x0A, 0xC2, 0x01, 0x00, 0x12, 0x01,
    0xFF, 0xFF, 0x00, 0x01, 0x0C, 0x09, 0x1E, 0x05, 0x01, 0x03, 0x01, 0x02, 0x00, 0x02, 0x00, 0x46,
    0x2A, 0x20, 0x0B, 0x0F, 0x32, 0x32, 0x00, 0x00, 0x01, 0x28, 0x08, 0x1E, 0x3F, 0x00, 0xB7, 0xFE,
];

/// Represents a decoded 480-byte EEPROM configuration image in RAM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EepromImage {
    pub raw: [u8; EEPROM_TOTAL_LEN],
}

impl EepromImage {
    /// Creates an `EepromImage` initialized with the factory defaults for DM01 (`TS1000W`).
    pub const fn default_dm01() -> Self {
        Self {
            raw: DM01_V6510C_FACTORY_EEPROM,
        }
    }

    /// Creates an `EepromImage` initialized with the factory defaults for DM02 (`TS500W`).
    pub const fn default_dm02() -> Self {
        Self {
            raw: DM02_V6510C_FACTORY_EEPROM,
        }
    }

    /// Decrypts a 480-byte Flash region (`0x0801FC00`) by XORing every byte with `0xA5`
    /// and validates both the Block 1 magic (`A5 01 FE`) and all 5 block CRC-16s.
    pub fn from_encrypted_flash(flash_bytes: &[u8; EEPROM_TOTAL_LEN]) -> Result<Self, usize> {
        let mut raw = [0u8; EEPROM_TOTAL_LEN];
        let mut i = 0;
        while i < EEPROM_TOTAL_LEN {
            raw[i] = flash_bytes[i] ^ EEPROM_XOR_KEY;
            i += 1;
        }
        let img = Self { raw };
        img.verify_all_blocks()?;
        Ok(img)
    }

    /// Recomputes and writes the CRC-16 checksum at the end of all 5 blocks,
    /// then encrypts the 480-byte buffer with `0xA5` for writing to MCU Flash (`0x0801FC00`).
    pub fn to_encrypted_flash(&mut self) -> [u8; EEPROM_TOTAL_LEN] {
        self.update_all_crcs();
        let mut out = [0u8; EEPROM_TOTAL_LEN];
        let mut i = 0;
        while i < EEPROM_TOTAL_LEN {
            out[i] = self.raw[i] ^ EEPROM_XOR_KEY;
            i += 1;
        }
        out
    }

    /// Reads and validates the 480-byte EEPROM image from physical MCU Flash Page 127 (`0x0801FC00`).
    /// Returns `Some(img)` if valid magic (`A5 01 FE`) and all 5 block CRCs pass, or `None` if unprogrammed/corrupt.
    ///
    /// # Safety
    /// Must only be executed on target hardware where `0x0801FC00` is mapped to internal Flash.
    pub unsafe fn read_from_flash() -> Option<Self> {
        let ptr = EEPROM_FLASH_BASE_ADDR as *const u8;
        let flash_slice = &*(ptr as *const [u8; EEPROM_TOTAL_LEN]);
        Self::from_encrypted_flash(flash_slice).ok()
    }

    /// Writes this EEPROM image to MCU Flash Page 127 (`0x0801FC00`), encrypting with `0xA5`
    /// and recalculating all 5 block CRCs.
    pub fn write_to_flash(&mut self) -> Result<(), crate::pac::FlashError> {
        let encrypted = self.to_encrypted_flash();
        let mut halfwords = [0u16; EEPROM_TOTAL_LEN / 2];
        let mut i = 0;
        while i < EEPROM_TOTAL_LEN / 2 {
            halfwords[i] = u16::from_le_bytes([encrypted[i * 2], encrypted[i * 2 + 1]]);
            i += 1;
        }

        crate::pac::HardwarePeripherals::flash_erase_page(EEPROM_FLASH_BASE_ADDR)?;
        crate::pac::HardwarePeripherals::flash_write_halfwords(EEPROM_FLASH_BASE_ADDR, &halfwords)
    }

    /// Verifies the Block 1 magic header (`A5 01 FE`) and the little-endian CRC-16
    /// at the end of each of the 5 blocks. Returns `Err(block_index)` if any block fails.
    pub fn verify_all_blocks(&self) -> Result<(), usize> {
        // Check Block 1 magic at offset 0x05A..0x05D
        if self.raw[0x05A..0x05D] != EEPROM_BLOCK1_MAGIC {
            return Err(1);
        }
        for block_idx in 0..5 {
            let start = EEPROM_BLOCK_OFFSETS[block_idx];
            let len = EEPROM_BLOCK_SIZES[block_idx];
            let payload_end = start + len - 2;
            let expected_crc = crc16_toseven(&self.raw[start..payload_end]);
            let stored_crc = u16::from_le_bytes([self.raw[payload_end], self.raw[payload_end + 1]]);
            if expected_crc != stored_crc {
                return Err(block_idx);
            }
        }
        Ok(())
    }

    /// Recomputes and updates the Block 1 magic (`A5 01 FE`) and the CRC-16 at the end of all 5 blocks.
    pub fn update_all_crcs(&mut self) {
        self.raw[0x05A..0x05D].copy_from_slice(&EEPROM_BLOCK1_MAGIC);
        for block_idx in 0..5 {
            let start = EEPROM_BLOCK_OFFSETS[block_idx];
            let len = EEPROM_BLOCK_SIZES[block_idx];
            let payload_end = start + len - 2;
            let crc = crc16_toseven(&self.raw[start..payload_end]);
            let crc_bytes = crc.to_le_bytes();
            self.raw[payload_end] = crc_bytes[0];
            self.raw[payload_end + 1] = crc_bytes[1];
        }
    }

    /// Returns the ASCII model string stored at the start of Block 1 (`0x020..0x028`, e.g. `"TS1000W"` or `"TS500W"`).
    pub fn model_name_bytes(&self) -> &[u8] {
        let slice = &self.raw[0x020..0x028];
        let mut len = 0;
        while len < slice.len() && slice[len] != 0 {
            len += 1;
        }
        &slice[..len]
    }

    /// Returns the default `Pwr[0..5]` assist level percentages stored in Block 4 (`0x177..0x17C`).
    pub fn assist_pwr_table(&self) -> [u8; 5] {
        [
            self.raw[0x177],
            self.raw[0x178],
            self.raw[0x179],
            self.raw[0x17A],
            self.raw[0x17B],
        ]
    }

    /// Updates the `Pwr[0..5]` assist level percentages in Block 4 and refreshes block CRCs.
    pub fn set_assist_pwr_table(&mut self, pwr: [u8; 5]) {
        self.raw[0x177..0x17C].copy_from_slice(&pwr);
        self.update_all_crcs();
    }

    /// Returns the `EAssist[0..5]` torque assist level percentages stored in Block 4 (`0x14F..0x154`).
    pub fn assist_eassist_table(&self) -> [u8; 5] {
        [
            self.raw[0x14F],
            self.raw[0x150],
            self.raw[0x151],
            self.raw[0x152],
            self.raw[0x153],
        ]
    }

    /// Updates the `EAssist[0..5]` torque assist level percentages in Block 4 and refreshes block CRCs.
    pub fn set_assist_eassist_table(&mut self, eassist: [u8; 5]) {
        self.raw[0x14F..0x154].copy_from_slice(&eassist);
        self.update_all_crcs();
    }

    /// Returns the crankset (chainring) tooth count stored in Block 4 (`0x1D0`, e.g. 46T for DM01, 42T for DM02).
    pub fn crankset_teeth(&self) -> u8 {
        self.raw[0x1D0]
    }

    /// Sets the crankset (chainring) tooth count in Block 4 (`0x1D0`) and refreshes block CRCs.
    pub fn set_crankset_teeth(&mut self, teeth: u8) {
        self.raw[0x1D0] = teeth;
        self.update_all_crcs();
    }

    /// Returns the largest cassette sprocket tooth count stored in Block 4 (`0x1D1`, e.g. 32T).
    pub fn cassette_max_teeth(&self) -> u8 {
        self.raw[0x1D1]
    }

    /// Sets the largest cassette sprocket tooth count in Block 4 (`0x1D1`) and refreshes block CRCs.
    pub fn set_cassette_max_teeth(&mut self, teeth: u8) {
        self.raw[0x1D1] = teeth;
        self.update_all_crcs();
    }

    /// Returns the smallest cassette sprocket tooth count stored in Block 4 (`0x1D2`, e.g. 11T).
    pub fn cassette_min_teeth(&self) -> u8 {
        self.raw[0x1D2]
    }

    /// Sets the smallest cassette sprocket tooth count in Block 4 (`0x1D2`) and refreshes block CRCs.
    pub fn set_cassette_min_teeth(&mut self, teeth: u8) {
        self.raw[0x1D2] = teeth;
        self.update_all_crcs();
    }

    /// Returns the wheel diameter in inches stored at the start of Block 4 (`0x140`, e.g. 26 inches).
    pub fn wheel_diameter_inches(&self) -> u8 {
        self.raw[0x140]
    }

    /// Sets the wheel diameter in inches in Block 4 (`0x140`) and refreshes block CRCs.
    pub fn set_wheel_diameter_inches(&mut self, inches: u8) {
        self.raw[0x140] = inches;
        self.update_all_crcs();
    }

    /// Returns true if SpeedSensorFail is set to `Speed Unlimit` (stored as `0` at `0x1D7` in Block 4).
    pub fn speed_sensor_fail_unlimit(&self) -> bool {
        self.raw[0x1D7] == 0
    }

    /// Sets the SpeedSensorFail setting in Block 4 (`0x1D7`, `0` = Speed Unlimit, `1` = Speed Limit).
    pub fn set_speed_sensor_fail_unlimit(&mut self, unlimit: bool) {
        self.raw[0x1D7] = if unlimit { 0 } else { 1 };
        self.update_all_crcs();
    }

    /// Computes the static midpoint gear ratio:
    /// `ratio_mid = (Crankset / MinTeeth + Crankset / MaxTeeth) / 2`
    /// scaled by 256 (fixed-point Q8, where 256 = 1.0).
    pub fn midpoint_gear_ratio_q8(&self) -> u16 {
        let crank = self.crankset_teeth() as u32;
        let min_t = (self.cassette_min_teeth() as u32).max(1);
        let max_t = (self.cassette_max_teeth() as u32).max(min_t);
        let ratio_high = (crank * 256) / min_t;
        let ratio_low = (crank * 256) / max_t;
        ((ratio_high + ratio_low) / 2) as u16
    }

    /// Estimates vehicle speed in 0.1 km/h units from crank cadence using the midpoint gear ratio
    /// and wheel diameter in inches.
    pub fn estimate_speed_deci_kph(&self, cadence_rpm: u16) -> u16 {
        let inches = self.wheel_diameter_inches();
        let eff_inches = if inches == 0 { 26 } else { inches };
        let circ_mm = (eff_inches as u64 * 798) / 10;
        let ratio_q8 = self.midpoint_gear_ratio_q8() as u64;
        let num = (cadence_rpm as u64) * ratio_q8 * circ_mm * 3;
        ((num + 640_000) / 1_280_000) as u16
    }

    /// Returns the custom Open Firmware configuration stored in Block 1 zero padding (`0x04A..0x05A`),
    /// or `None` if unconfigured / stock defaults.
    pub fn custom_config(&self) -> Option<DmPlusCustomConfig> {
        let off = DM_PLUS_CUSTOM_CONFIG_OFFSET;
        if self.raw[off] == DM_PLUS_CUSTOM_MAGIC[0] && self.raw[off + 1] == DM_PLUS_CUSTOM_MAGIC[1]
        {
            Some(DmPlusCustomConfig {
                tuning_profile: self.raw[off + 2],
                ceiling_policy: self.raw[off + 3],
                nylon_gear_knee_rpm: self.raw[off + 4],
                lvc_cushion_deci_v: self.raw[off + 5],
                thermal_rollback_start_c: self.raw[off + 6],
                street_legal_mode: StreetLegalMode::from_u8(self.raw[off + 7]),
            })
        } else {
            None
        }
    }

    /// Writes custom Open Firmware configuration into Block 1 zero padding (`0x04A..0x05A`)
    /// and refreshes block CRC-16 checksums.
    pub fn set_custom_config(&mut self, cfg: &DmPlusCustomConfig) {
        let off = DM_PLUS_CUSTOM_CONFIG_OFFSET;
        self.raw[off] = DM_PLUS_CUSTOM_MAGIC[0];
        self.raw[off + 1] = DM_PLUS_CUSTOM_MAGIC[1];
        self.raw[off + 2] = cfg.tuning_profile;
        self.raw[off + 3] = cfg.ceiling_policy;
        self.raw[off + 4] = cfg.nylon_gear_knee_rpm;
        self.raw[off + 5] = cfg.lvc_cushion_deci_v;
        self.raw[off + 6] = cfg.thermal_rollback_start_c;
        self.raw[off + 7] = cfg.street_legal_mode as u8;
        // Zero reserved bytes [off+8..off+16]
        let mut r = off + 8;
        while r < off + 16 {
            self.raw[r] = 0;
            r += 1;
        }
        self.update_all_crcs();
    }
}

/// Offset within EEPROM image where custom DM+ configuration is persisted (`Block 1: 0x04A..0x05A`).
pub const DM_PLUS_CUSTOM_CONFIG_OFFSET: usize = 0x04A;

/// Magic signature identifying active DM+ custom configuration ('D', '+').
pub const DM_PLUS_CUSTOM_MAGIC: [u8; 2] = [0x44, 0x2B];

/// Street-legal compliance mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StreetLegalMode {
    /// Unrestricted / Off-Road: Limits configurable up to hardware rating (30A DM01, 18A DM02).
    Unrestricted = 0,
    /// Street-Legal Default: Defaults to 25 km/h and 250W continuous, toggleable via button combo.
    StreetLegalDefault = 1,
    /// Permanent Street-Legal Lock (EN15194 / EPAC):
    /// Strictly enforces 25.0 km/h cutoff and 250W continuous limit.
    /// Standalone throttle capped to 6.0 km/h walk-assist.
    /// Display menu and UART speed/power overrides are permanently rejected.
    PermanentLock = 2,
}

impl StreetLegalMode {
    pub const fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::StreetLegalDefault,
            2 => Self::PermanentLock,
            _ => Self::Unrestricted,
        }
    }
}

/// Custom Open Firmware configuration persisted in previously unused EEPROM Block 1 zero padding (`0x04A..0x05A`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DmPlusCustomConfig {
    /// Tuning profile selector:
    /// 0: Stock V6510C
    /// 1: GearSafe Progressive (Default)
    /// 2: Uncapped Cadence
    /// 3: Progressive Trail
    /// 4: Commuter Comfort
    pub tuning_profile: u8,
    /// Assist ceiling policy:
    /// 0: StockDualCeiling (Torque uses full Mode Current, ignoring Pwr%)
    /// 1: UnifiedLevelCeiling (Torque respects assist level Pwr%)
    pub ceiling_policy: u8,
    /// Nylon primary reduction gear protection knee RPM (default: 18 RPM).
    /// If 0, nylon gear protection is disabled.
    pub nylon_gear_knee_rpm: u8,
    /// Low-voltage cutoff linear cushion in 0.1V units (default: 30 = 3.0V).
    pub lvc_cushion_deci_v: u8,
    /// Thermal rollback initiation temperature in °C (default: 70°C).
    pub thermal_rollback_start_c: u8,
    /// Street-legal compliance mode.
    pub street_legal_mode: StreetLegalMode,
}

impl Default for DmPlusCustomConfig {
    fn default() -> Self {
        Self {
            tuning_profile: 1, // GearSafe Progressive default
            ceiling_policy: 1, // Unified Level Ceiling default
            nylon_gear_knee_rpm: 18,
            lvc_cushion_deci_v: 30, // 3.0V
            thermal_rollback_start_c: 70,
            street_legal_mode: StreetLegalMode::Unrestricted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dm01_and_dm02_factory_eeprom_crcs_are_valid() {
        let dm01 = EepromImage::default_dm01();
        assert_eq!(dm01.verify_all_blocks(), Ok(()));
        assert_eq!(dm01.model_name_bytes(), b"TS1000W");
        assert_eq!(dm01.assist_pwr_table(), [20, 40, 60, 80, 100]);
        assert_eq!(dm01.assist_eassist_table(), [30, 40, 60, 80, 100]);
        assert_eq!(dm01.crankset_teeth(), 46);
        assert_eq!(dm01.cassette_max_teeth(), 32);
        assert_eq!(dm01.cassette_min_teeth(), 11);
        assert_eq!(dm01.wheel_diameter_inches(), 26);
        assert!(dm01.speed_sensor_fail_unlimit());
        assert_eq!(dm01.midpoint_gear_ratio_q8(), 719);
        assert_eq!(dm01.estimate_speed_deci_kph(60), 210); // 21.0 km/h
        assert_eq!(dm01.estimate_speed_deci_kph(0), 0);

        let mut mod_dm01 = dm01.clone();
        mod_dm01.set_speed_sensor_fail_unlimit(false);
        assert!(!mod_dm01.speed_sensor_fail_unlimit());
        assert_eq!(mod_dm01.verify_all_blocks(), Ok(()));

        let dm02 = EepromImage::default_dm02();
        assert_eq!(dm02.verify_all_blocks(), Ok(()));
        assert_eq!(dm02.model_name_bytes(), b"TS500W");
        assert_eq!(dm02.assist_pwr_table(), [25, 35, 50, 70, 100]);
        assert_eq!(dm02.assist_eassist_table(), [30, 50, 60, 80, 100]);
        assert_eq!(dm02.crankset_teeth(), 42);
        assert_eq!(dm02.cassette_max_teeth(), 32);
        assert_eq!(dm02.cassette_min_teeth(), 11);
        assert_eq!(dm02.wheel_diameter_inches(), 26);
        assert!(dm02.speed_sensor_fail_unlimit());
        assert_eq!(dm02.midpoint_gear_ratio_q8(), 656);
        assert_eq!(dm02.estimate_speed_deci_kph(60), 191); // 19.1 km/h
    }

    #[test]
    fn test_dm_plus_custom_config_persistence_and_crc() {
        let mut dm01 = EepromImage::default_dm01();
        assert_eq!(dm01.custom_config(), None); // Stock has no custom config

        let custom = DmPlusCustomConfig {
            tuning_profile: 1, // GearSafe
            ceiling_policy: 1, // Unified
            nylon_gear_knee_rpm: 18,
            lvc_cushion_deci_v: 30,
            thermal_rollback_start_c: 72,
            street_legal_mode: StreetLegalMode::PermanentLock,
        };
        dm01.set_custom_config(&custom);
        assert_eq!(dm01.verify_all_blocks(), Ok(()));
        assert_eq!(dm01.custom_config(), Some(custom));

        // Test encryption round-trip
        let flash = dm01.to_encrypted_flash();
        let decrypted = EepromImage::from_encrypted_flash(&flash).expect("decrypted valid");
        assert_eq!(decrypted.custom_config(), Some(custom));
    }

    #[test]
    fn test_eeprom_flash_xor_encryption_and_tamper_detection() {
        let mut dm01 = EepromImage::default_dm01();
        let mut flash = dm01.to_encrypted_flash();

        let recovered = EepromImage::from_encrypted_flash(&flash).expect("valid flash image");
        assert_eq!(recovered, dm01);

        // Corrupt a byte in Block 3 and verify that block 3 fails CRC check
        flash[0x0B0] ^= 0x01;
        assert_eq!(EepromImage::from_encrypted_flash(&flash), Err(3));
    }
}
