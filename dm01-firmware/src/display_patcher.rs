//! # ToSeven Display Firmware Architecture & Lockout Patcher (`display_patcher`)
//!
//! Provides parsing, inspection, and clean-room unlocking for ToSeven e-bike display firmware
//! dumps in Intel HEX format (`T24V`, `T154`, `LCD135C`, `M15`).
//!
//! ## Post-October 2024 Advanced Menu Lockout
//! Beginning with firmware `Y241016` and continuing with `Y241225` and `Y250313`, ToSeven added
//! hardware restrictions to display firmwares:
//! - **Gate 1 (Speed & Power):** Requires motor controller version `CtlVer >= 0x1842` and authorization byte `0x51` (`'Q'`).
//! - **Gate 2 (Current Limits):** Requires `CtlVer >= 0x1842` and authorization byte `0x51`.
//! - **Gate 3 (Advanced Menu Navigation):** Denies entry to the Advanced Menu unless authorized and `CtlVer >= 0x1842`.
//!
//! This module scans the binary image, pinpoints all active lockout branch instructions across
//! all three gates, and replaces them with ARM Thumb `NOP` (`0x00BF`) instructions, permanently
//! unlocking full access for any motor controller.

#[cfg(feature = "std")]
use std::fmt;

/// Errors encountered while parsing or patching Intel HEX display firmware.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayError {
    InvalidRecord(usize),
    ChecksumMismatch {
        line: usize,
        expected: u8,
        found: u8,
    },
    AddressOutOfBounds(u32),
    EmptyFile,
    InvalidVectorTable,
    IoError(&'static str),
}

#[cfg(feature = "std")]
impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRecord(l) => write!(f, "Invalid Intel HEX record on line {l}"),
            Self::ChecksumMismatch {
                line,
                expected,
                found,
            } => {
                write!(f, "Checksum mismatch on line {line}: expected 0x{expected:02X}, found 0x{found:02X}")
            }
            Self::AddressOutOfBounds(addr) => write!(f, "Address 0x{addr:08X} out of bounds"),
            Self::EmptyFile => write!(f, "Empty Intel HEX file"),
            Self::InvalidVectorTable => write!(f, "Invalid vector table (not Cortex-M / MegaHunt)"),
            Self::IoError(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DisplayError {}

/// A single Intel HEX record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexRecord {
    pub addr: u16,
    pub rectype: u8,
    pub data: Vec<u8>,
}

impl HexRecord {
    /// Computes the Intel HEX two's complement modulo-256 checksum for this record.
    pub fn checksum(&self) -> u8 {
        let mut sum: u8 = (self.data.len() as u8)
            .wrapping_add((self.addr >> 8) as u8)
            .wrapping_add((self.addr & 0xFF) as u8)
            .wrapping_add(self.rectype);
        for &b in &self.data {
            sum = sum.wrapping_add(b);
        }
        (0u8).wrapping_sub(sum)
    }

    /// Formats this record as a standard Intel HEX record line (`:LLAAAATTDD...CC\r\n`).
    pub fn to_hex_line(&self) -> String {
        let mut s = String::with_capacity(11 + self.data.len() * 2 + 2);
        s.push(':');
        s.push_str(&format!(
            "{:02X}{:04X}{:02X}",
            self.data.len(),
            self.addr,
            self.rectype
        ));
        for &b in &self.data {
            s.push_str(&format!("{:02X}", b));
        }
        s.push_str(&format!("{:02X}\r\n", self.checksum()));
        s
    }
}

/// Parsed in-memory representation of an Intel HEX firmware file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntelHexFile {
    pub records: Vec<HexRecord>,
}

impl IntelHexFile {
    /// Parses an Intel HEX formatted string into structured records.
    pub fn parse(text: &str) -> Result<Self, DisplayError> {
        let mut records = Vec::new();
        for (line_idx, raw_line) in text.lines().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = raw_line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !trimmed.starts_with(':') || trimmed.len() < 11 {
                return Err(DisplayError::InvalidRecord(line_num));
            }
            let hex_str = &trimmed[1..];
            let raw_bytes =
                hex_decode(hex_str).map_err(|_| DisplayError::InvalidRecord(line_num))?;
            let len = raw_bytes[0] as usize;
            if raw_bytes.len() != 5 + len {
                return Err(DisplayError::InvalidRecord(line_num));
            }
            let addr = u16::from_be_bytes([raw_bytes[1], raw_bytes[2]]);
            let rectype = raw_bytes[3];
            let data = raw_bytes[4..4 + len].to_vec();
            let found_chk = raw_bytes[4 + len];

            let rec = HexRecord {
                addr,
                rectype,
                data,
            };
            let expected_chk = rec.checksum();
            if found_chk != expected_chk {
                return Err(DisplayError::ChecksumMismatch {
                    line: line_num,
                    expected: expected_chk,
                    found: found_chk,
                });
            }
            records.push(rec);
            if rectype == 0x01 {
                // End of File record
                break;
            }
        }
        if records.is_empty() {
            return Err(DisplayError::EmptyFile);
        }
        Ok(Self { records })
    }

    /// Serializes all records back into an Intel HEX formatted string.
    pub fn to_hex_string(&self) -> String {
        let mut out = String::new();
        for rec in &self.records {
            out.push_str(&rec.to_hex_line());
        }
        out
    }

    /// Assembles all data records (`rectype == 0x00`) into a contiguous flat binary image.
    /// Returns `(base_address, binary_bytes)`.
    pub fn to_flat_binary(&self) -> Result<(u32, Vec<u8>), DisplayError> {
        let mut ext_base = 0u32;
        let mut min_addr = u32::MAX;
        let mut max_addr = 0u32;
        let mut data_recs = Vec::new();

        for rec in &self.records {
            match rec.rectype {
                0x04 => {
                    if rec.data.len() >= 2 {
                        ext_base = ((rec.data[0] as u32) << 24) | ((rec.data[1] as u32) << 16);
                    }
                }
                0x02 => {
                    if rec.data.len() >= 2 {
                        ext_base = (((rec.data[0] as u32) << 8) | (rec.data[1] as u32)) << 4;
                    }
                }
                0x00 => {
                    let full_addr = ext_base + (rec.addr as u32);
                    let end_addr = full_addr + rec.data.len() as u32;
                    if full_addr < min_addr {
                        min_addr = full_addr;
                    }
                    if end_addr > max_addr {
                        max_addr = end_addr;
                    }
                    data_recs.push((full_addr, &rec.data));
                }
                _ => {}
            }
        }

        if min_addr >= max_addr {
            return Err(DisplayError::EmptyFile);
        }

        let total_len = (max_addr - min_addr) as usize;
        let mut buffer = vec![0xFFu8; total_len];
        for (addr, data) in data_recs {
            let offset = (addr - min_addr) as usize;
            buffer[offset..offset + data.len()].copy_from_slice(data);
        }

        Ok((min_addr, buffer))
    }

    /// Patches a contiguous slice of bytes at `target_addr` inside the HEX records.
    pub fn patch_bytes(&mut self, target_addr: u32, new_bytes: &[u8]) -> Result<(), DisplayError> {
        for (i, &b) in new_bytes.iter().enumerate() {
            let byte_addr = target_addr + i as u32;
            let mut patched = false;
            let mut ext_base = 0u32;

            for rec in &mut self.records {
                match rec.rectype {
                    0x04 => {
                        if rec.data.len() >= 2 {
                            ext_base = ((rec.data[0] as u32) << 24) | ((rec.data[1] as u32) << 16);
                        }
                    }
                    0x02 => {
                        if rec.data.len() >= 2 {
                            ext_base = (((rec.data[0] as u32) << 8) | (rec.data[1] as u32)) << 4;
                        }
                    }
                    0x00 => {
                        let rec_full_addr = ext_base + (rec.addr as u32);
                        let rec_end = rec_full_addr + rec.data.len() as u32;
                        if byte_addr >= rec_full_addr && byte_addr < rec_end {
                            let rec_offset = (byte_addr - rec_full_addr) as usize;
                            rec.data[rec_offset] = b;
                            patched = true;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if !patched {
                return Err(DisplayError::AddressOutOfBounds(byte_addr));
            }
        }
        Ok(())
    }
}

/// Helper function to decode hex string into bytes.
fn hex_decode(s: &str) -> Result<Vec<u8>, ()> {
    if !s.len().is_multiple_of(2) {
        return Err(());
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    let chars = s.as_bytes();
    let mut i = 0;
    while i < chars.len() {
        let hi = hex_val(chars[i])?;
        let lo = hex_val(chars[i + 1])?;
        bytes.push((hi << 4) | lo);
        i += 2;
    }
    Ok(bytes)
}

fn hex_val(c: u8) -> Result<u8, ()> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(()),
    }
}

/// Description of an individual lockout patch site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockoutPatch {
    pub gate: u8,
    pub name: &'static str,
    pub description: &'static str,
    pub address: u32,
    pub original_bytes: [u8; 2],
    pub patched_bytes: [u8; 2],
}

/// Summary report of display lockout status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockoutStatus {
    /// No lockout gates found (legacy stock firmware, e.g. T24V-Y240527A, LCD135C, M15).
    LegacyUnlocked,
    /// Lockout gates detected and active.
    Locked { patches: Vec<LockoutPatch> },
    /// Lockout gates detected, but all have been patched with NOP (0x00BF).
    AlreadyUnlocked { gate_addresses: Vec<u32> },
}

/// Comprehensive information extracted from a display firmware image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayInfo {
    pub base_address: u32,
    pub size_bytes: usize,
    pub initial_sp: u32,
    pub reset_handler: u32,
    pub identified_model: &'static str,
    pub build_timestamp: Option<String>,
    pub lockout_status: LockoutStatus,
}

/// Detects the lockout status in a flat binary image at `base_addr`.
pub fn detect_lockout_in_binary(data: &[u8], base_addr: u32) -> LockoutStatus {
    let mut active_patches = Vec::new();
    let mut already_patched_addresses = Vec::new();

    // -------------------------------------------------------------
    // Gate 1: Speed & Power Edit Permission
    // Version check: 41 f6 42 01 88 42 [0a db] (or 00 bf if unlocked)
    // Auth check:    90 f8 98 00 51 28 [05 d1] (or 00 bf if unlocked)
    // -------------------------------------------------------------
    let g1_ver_pat = [0x41, 0xF6, 0x42, 0x01, 0x88, 0x42];
    let g1_auth_pat = [0x90, 0xF8, 0x98, 0x00, 0x51, 0x28];

    if let Some(idx) = find_subslice(data, &g1_ver_pat) {
        if idx + 8 <= data.len() {
            let branch = [data[idx + 6], data[idx + 7]];
            let addr = base_addr + (idx + 6) as u32;
            if branch == [0x0A, 0xDB] {
                active_patches.push(LockoutPatch {
                    gate: 1,
                    name: "Gate 1 (Speed & Power CtlVer Check)",
                    description: "Bypasses CtlVer >= 0x1842 requirement for Speed & Power settings",
                    address: addr,
                    original_bytes: branch,
                    patched_bytes: [0x00, 0xBF],
                });
            } else if branch == [0x00, 0xBF] {
                already_patched_addresses.push(addr);
            }
        }
    }

    if let Some(idx) = find_subslice(data, &g1_auth_pat) {
        if idx + 8 <= data.len() {
            let branch = [data[idx + 6], data[idx + 7]];
            let addr = base_addr + (idx + 6) as u32;
            if branch == [0x05, 0xD1] {
                active_patches.push(LockoutPatch {
                    gate: 1,
                    name: "Gate 1 (Speed & Power Auth Check)",
                    description:
                        "Bypasses 0x51 authorization requirement for Speed & Power settings",
                    address: addr,
                    original_bytes: branch,
                    patched_bytes: [0x00, 0xBF],
                });
            } else if branch == [0x00, 0xBF] {
                already_patched_addresses.push(addr);
            }
        }
    }

    // -------------------------------------------------------------
    // Gate 2: Controller Current Limit Edit Permission
    // Version check: 41 f6 42 02 90 42 [08 db] (or 00 bf if unlocked)
    // Auth check:    90 f8 98 00 51 28 [03 d1] (or 00 bf if unlocked)
    // -------------------------------------------------------------
    let g2_ver_pat = [0x41, 0xF6, 0x42, 0x02, 0x90, 0x42];
    if let Some(idx) = find_subslice(data, &g2_ver_pat) {
        if idx + 8 <= data.len() {
            let branch = [data[idx + 6], data[idx + 7]];
            let addr = base_addr + (idx + 6) as u32;
            if branch == [0x08, 0xDB] {
                active_patches.push(LockoutPatch {
                    gate: 2,
                    name: "Gate 2 (Current Limit CtlVer Check)",
                    description: "Bypasses CtlVer >= 0x1842 requirement for Current Limit settings",
                    address: addr,
                    original_bytes: branch,
                    patched_bytes: [0x00, 0xBF],
                });
            } else if branch == [0x00, 0xBF] {
                already_patched_addresses.push(addr);
            }
        }
    }

    // For Gate 2 auth check: search second occurrence of g1_auth_pat
    if let Some(idx) = find_nth_subslice(data, &g1_auth_pat, 1) {
        if idx + 8 <= data.len() {
            let branch = [data[idx + 6], data[idx + 7]];
            let addr = base_addr + (idx + 6) as u32;
            if branch == [0x03, 0xD1] {
                active_patches.push(LockoutPatch {
                    gate: 2,
                    name: "Gate 2 (Current Limit Auth Check)",
                    description:
                        "Bypasses 0x51 authorization requirement for Current Limit settings",
                    address: addr,
                    original_bytes: branch,
                    patched_bytes: [0x00, 0xBF],
                });
            } else if branch == [0x00, 0xBF] {
                already_patched_addresses.push(addr);
            }
        }
    }

    // -------------------------------------------------------------
    // Gate 3: Advanced Menu Navigation Entry
    // Auth check:    90 f8 98 00 51 28 [XX d1] (0x11 or 0x12)
    // Version check: 41 f6 42 01 88 42 [XX db] (0x0B or 0x0C)
    // -------------------------------------------------------------
    if let Some(idx) = find_nth_subslice(data, &g1_auth_pat, 2) {
        if idx + 8 <= data.len() {
            let branch = [data[idx + 6], data[idx + 7]];
            let addr = base_addr + (idx + 6) as u32;
            if (branch[0] == 0x11 || branch[0] == 0x12) && branch[1] == 0xD1 {
                active_patches.push(LockoutPatch {
                    gate: 3,
                    name: "Gate 3 (Advanced Menu Auth Check)",
                    description:
                        "Unconditionally permits Advanced Menu entry without 0x51 authorization",
                    address: addr,
                    original_bytes: branch,
                    patched_bytes: [0x00, 0xBF],
                });
            } else if branch == [0x00, 0xBF] {
                already_patched_addresses.push(addr);
            }
        }
    }

    if let Some(idx) = find_nth_subslice(data, &g1_ver_pat, 1) {
        if idx + 8 <= data.len() {
            let branch = [data[idx + 6], data[idx + 7]];
            let addr = base_addr + (idx + 6) as u32;
            if (branch[0] == 0x0B || branch[0] == 0x0C) && branch[1] == 0xDB {
                active_patches.push(LockoutPatch {
                    gate: 3,
                    name: "Gate 3 (Advanced Menu CtlVer Check)",
                    description: "Unconditionally permits Advanced Menu entry with CtlVer < 0x1842",
                    address: addr,
                    original_bytes: branch,
                    patched_bytes: [0x00, 0xBF],
                });
            } else if branch == [0x00, 0xBF] {
                already_patched_addresses.push(addr);
            }
        }
    }

    if !active_patches.is_empty() {
        LockoutStatus::Locked {
            patches: active_patches,
        }
    } else if !already_patched_addresses.is_empty() {
        LockoutStatus::AlreadyUnlocked {
            gate_addresses: already_patched_addresses,
        }
    } else {
        LockoutStatus::LegacyUnlocked
    }
}

/// Detects the lockout status in an `IntelHexFile`.
pub fn detect_lockout(hex_file: &IntelHexFile) -> Result<LockoutStatus, DisplayError> {
    let (base_addr, bin) = hex_file.to_flat_binary()?;
    Ok(detect_lockout_in_binary(&bin, base_addr))
}

/// Applies all available lockout unlock patches to an `IntelHexFile`.
/// Returns the list of applied patches.
pub fn patch_display(hex_file: &mut IntelHexFile) -> Result<Vec<LockoutPatch>, DisplayError> {
    let status = detect_lockout(hex_file)?;
    match status {
        LockoutStatus::Locked { patches } => {
            for patch in &patches {
                hex_file.patch_bytes(patch.address, &patch.patched_bytes)?;
            }
            Ok(patches)
        }
        LockoutStatus::AlreadyUnlocked { .. } => Ok(Vec::new()),
        LockoutStatus::LegacyUnlocked => Ok(Vec::new()),
    }
}

/// Inspects a display firmware image and extracts architecture metadata, model, and lockout status.
pub fn inspect_display(hex_file: &IntelHexFile) -> Result<DisplayInfo, DisplayError> {
    let (base_address, bin) = hex_file.to_flat_binary()?;
    if bin.len() < 8 {
        return Err(DisplayError::InvalidVectorTable);
    }
    let initial_sp = u32::from_le_bytes([bin[0], bin[1], bin[2], bin[3]]);
    let reset_handler = u32::from_le_bytes([bin[4], bin[5], bin[6], bin[7]]);

    // Validate ARM Cortex-M initial SP and reset vector (Thumb bit set)
    if (initial_sp & 0xFF00_0000) != 0x2000_0000 || (reset_handler & 1) == 0 {
        return Err(DisplayError::InvalidVectorTable);
    }

    // Identify display model from ASCII strings in binary
    let model = if find_subslice(&bin, b"T24V").is_some() || find_subslice(&bin, b"t24v").is_some()
    {
        "ToSeven T24V (MH2020C)"
    } else if find_subslice(&bin, b"T154").is_some() || find_subslice(&bin, b"t154").is_some() {
        "ToSeven T154 (MH2020C)"
    } else if find_subslice(&bin, b"135C").is_some() || find_subslice(&bin, b"LCD135").is_some() {
        "ToSeven LCD135C (MH2020C / BLE)"
    } else if find_subslice(&bin, b"M15").is_some() || find_subslice(&bin, b"m15").is_some() {
        "ToSeven M15 (MH2020C)"
    } else {
        "Generic ToSeven MegaHunt Display"
    };

    // Extract build timestamp if available (e.g. Y241225, Y241016, Y250313)
    let build_timestamp = extract_timestamp(&bin);
    let lockout_status = detect_lockout_in_binary(&bin, base_address);

    Ok(DisplayInfo {
        base_address,
        size_bytes: bin.len(),
        initial_sp,
        reset_handler,
        identified_model: model,
        build_timestamp,
        lockout_status,
    })
}

fn extract_timestamp(data: &[u8]) -> Option<String> {
    // Search for patterns like 'Y' followed by 6 digits: 'Y241225', 'Y250313'
    for i in 0..data.len().saturating_sub(7) {
        if data[i] == b'Y'
            && data[i + 1].is_ascii_digit()
            && data[i + 2].is_ascii_digit()
            && data[i + 3].is_ascii_digit()
            && data[i + 4].is_ascii_digit()
            && data[i + 5].is_ascii_digit()
            && data[i + 6].is_ascii_digit()
        {
            if let Ok(s) = std::str::from_utf8(&data[i..i + 7]) {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn find_nth_subslice(haystack: &[u8], needle: &[u8], nth: usize) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let mut count = 0;
    let mut pos = 0;
    while pos + needle.len() <= haystack.len() {
        if &haystack[pos..pos + needle.len()] == needle {
            if count == nth {
                return Some(pos);
            }
            count += 1;
            pos += needle.len();
        } else {
            pos += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn find_blob(rel: &str) -> Option<PathBuf> {
        let parent = format!("../{rel}");
        for candidate in [parent.as_str(), rel] {
            let p = Path::new(candidate);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
        None
    }

    /// Builds a self-contained Intel HEX image in memory containing a valid MH2020C vector table
    /// (`0x01001000`) and all 3 post-Oct 2024 lockout gate signatures so CI (where `blobs/` is
    /// `.gitignore`'d) still tests 100% of `IntelHexFile`, `inspect_display`, `detect_lockout`,
    /// and `patch_display`.
    fn build_synthetic_locked_hex() -> String {
        let mut bin = vec![0xFFu8; 0x2000];
        // Vector 0: Initial SP = 0x2000_B018; Vector 1: Reset_Handler = 0x0101_7475
        bin[0..4].copy_from_slice(&0x2000_B018u32.to_le_bytes());
        bin[4..8].copy_from_slice(&0x0101_7475u32.to_le_bytes());
        bin[0x40..0x44].copy_from_slice(b"T24V");
        bin[0x50..0x57].copy_from_slice(b"Y241225");

        let g1_ver_pat = [0x41, 0xF6, 0x42, 0x01, 0x88, 0x42];
        let g1_auth_pat = [0x90, 0xF8, 0x98, 0x00, 0x51, 0x28];
        let g2_ver_pat = [0x41, 0xF6, 0x42, 0x02, 0x90, 0x42];

        // Gate 1: CtlVer check [0x0A, 0xDB] + Auth check #0 [0x05, 0xD1]
        let mut pos = 0x0100;
        bin[pos..pos + 6].copy_from_slice(&g1_ver_pat);
        bin[pos + 6..pos + 8].copy_from_slice(&[0x0A, 0xDB]);
        pos += 0x20;
        bin[pos..pos + 6].copy_from_slice(&g1_auth_pat);
        bin[pos + 6..pos + 8].copy_from_slice(&[0x05, 0xD1]);

        // Gate 2: CtlVer check [0x08, 0xDB] + Auth check #1 [0x03, 0xD1]
        pos += 0x20;
        bin[pos..pos + 6].copy_from_slice(&g2_ver_pat);
        bin[pos + 6..pos + 8].copy_from_slice(&[0x08, 0xDB]);
        pos += 0x20;
        bin[pos..pos + 6].copy_from_slice(&g1_auth_pat);
        bin[pos + 6..pos + 8].copy_from_slice(&[0x03, 0xD1]);

        // Gate 3: Auth check #2 [0x12, 0xD1] + CtlVer check #1 [0x0C, 0xDB]
        pos += 0x20;
        bin[pos..pos + 6].copy_from_slice(&g1_auth_pat);
        bin[pos + 6..pos + 8].copy_from_slice(&[0x12, 0xD1]);
        pos += 0x20;
        bin[pos..pos + 6].copy_from_slice(&g1_ver_pat);
        bin[pos + 6..pos + 8].copy_from_slice(&[0x0C, 0xDB]);

        // Encode as Intel HEX at base 0x0100_1000
        let mut records = Vec::new();
        records.push(HexRecord {
            addr: 0,
            rectype: 0x04, // Extended Linear Address
            data: vec![0x01, 0x00],
        });
        for (i, chunk) in bin.chunks(16).enumerate() {
            records.push(HexRecord {
                addr: (0x1000 + i * 16) as u16,
                rectype: 0x00, // Data
                data: chunk.to_vec(),
            });
        }
        records.push(HexRecord {
            addr: 0,
            rectype: 0x01, // EOF
            data: Vec::new(),
        });
        IntelHexFile { records }.to_hex_string()
    }

    #[test]
    fn test_synthetic_intel_hex_lockout_and_patch() {
        let raw_hex = build_synthetic_locked_hex();
        let mut hex_file = IntelHexFile::parse(&raw_hex).expect("parse synthetic hex");

        let info = inspect_display(&hex_file).expect("inspect synthetic display");
        assert_eq!(info.base_address, 0x0100_1000);
        assert_eq!(info.initial_sp, 0x2000_B018);
        assert_eq!(info.reset_handler, 0x0101_7475);
        assert_eq!(info.identified_model, "ToSeven T24V (MH2020C)");

        let status = detect_lockout(&hex_file).expect("detect lockout");
        let LockoutStatus::Locked { ref patches } = status else {
            panic!("Expected synthetic hex to be Locked, got {status:?}");
        };
        assert_eq!(patches.len(), 6);

        let applied = patch_display(&mut hex_file).expect("apply patches");
        assert_eq!(applied.len(), 6);

        let after = detect_lockout(&hex_file).expect("detect after patch");
        assert!(matches!(after, LockoutStatus::AlreadyUnlocked { .. }));

        // Round-trip through string serialization
        let serialized = hex_file.to_hex_string();
        let reparsed = IntelHexFile::parse(&serialized).expect("reparse");
        assert_eq!(detect_lockout(&reparsed).unwrap(), after);
    }

    #[test]
    fn test_intel_hex_roundtrip_t24v() {
        let Some(path) = find_blob("blobs/displays/t24v/TO7-T24V-Y241225.hex") else {
            return;
        };
        let content = fs::read_to_string(path).expect("read hex");
        let hex_file = IntelHexFile::parse(&content).expect("parse hex");
        assert!(!hex_file.records.is_empty());

        let (base, bin) = hex_file.to_flat_binary().expect("flat binary");
        assert_eq!(base, 0x0100_1000);
        assert!(bin.len() > 100_000);

        let info = inspect_display(&hex_file).expect("inspect");
        assert_eq!(info.identified_model, "ToSeven T24V (MH2020C)");
        assert_eq!(info.initial_sp, 0x2000_B018);
        assert_eq!(info.reset_handler, 0x0101_7475);
    }

    #[test]
    fn test_lockout_detection_and_patching_t24v_y241225() {
        let Some(path) = find_blob("blobs/displays/t24v/TO7-T24V-Y241225.hex") else {
            return;
        };
        let content = fs::read_to_string(path).expect("read hex");
        let mut hex_file = IntelHexFile::parse(&content).expect("parse hex");

        let status = detect_lockout(&hex_file).expect("detect");
        let LockoutStatus::Locked { patches } = status else {
            panic!(
                "Expected TO7-T24V-Y241225 to be Locked, found: {:?}",
                status
            );
        };
        assert_eq!(patches.len(), 6, "Expected 6 active lockout branch patches");

        // Verify Gate 3 patches match reverse-engineered addresses
        let g3_auth = patches
            .iter()
            .find(|p| p.gate == 3 && p.name.contains("Auth"))
            .unwrap();
        assert_eq!(g3_auth.address, 0x0101_0E7C);
        assert_eq!(g3_auth.original_bytes, [0x12, 0xD1]);
        assert_eq!(g3_auth.patched_bytes, [0x00, 0xBF]);

        let g3_ver = patches
            .iter()
            .find(|p| p.gate == 3 && p.name.contains("CtlVer"))
            .unwrap();
        assert_eq!(g3_ver.address, 0x0101_0E88);
        assert_eq!(g3_ver.original_bytes, [0x0C, 0xDB]);
        assert_eq!(g3_ver.patched_bytes, [0x00, 0xBF]);

        // Apply patches
        let applied = patch_display(&mut hex_file).expect("patch");
        assert_eq!(applied.len(), 6);

        // Verify new status is AlreadyUnlocked
        let new_status = detect_lockout(&hex_file).expect("detect after patch");
        match new_status {
            LockoutStatus::AlreadyUnlocked { ref gate_addresses } => {
                assert_eq!(gate_addresses.len(), 6);
            }
            other => panic!("Expected AlreadyUnlocked, got {:?}", other),
        }

        // Serialized hex must still be valid and reparseable
        let serialized = hex_file.to_hex_string();
        let reparsed = IntelHexFile::parse(&serialized).expect("reparse patched hex");
        assert_eq!(detect_lockout(&reparsed).expect("detect"), new_status);
    }

    #[test]
    fn test_lockout_detection_and_patching_t154() {
        let Some(path) = find_blob("blobs/displays/t154/TO7-T154-Y250313.hex") else {
            return;
        };
        let content = fs::read_to_string(path).expect("read hex");
        let mut hex_file = IntelHexFile::parse(&content).expect("parse hex");

        let status = detect_lockout(&hex_file).expect("detect");
        let LockoutStatus::Locked { patches } = status else {
            panic!(
                "Expected TO7-T154-Y250313 to be Locked, found: {:?}",
                status
            );
        };
        assert_eq!(patches.len(), 6);

        let g3_auth = patches
            .iter()
            .find(|p| p.gate == 3 && p.name.contains("Auth"))
            .unwrap();
        assert_eq!(g3_auth.address, 0x0100_FB26);
        assert_eq!(g3_auth.original_bytes, [0x11, 0xD1]);

        let g3_ver = patches
            .iter()
            .find(|p| p.gate == 3 && p.name.contains("CtlVer"))
            .unwrap();
        assert_eq!(g3_ver.address, 0x0100_FB32);
        assert_eq!(g3_ver.original_bytes, [0x0B, 0xDB]);

        patch_display(&mut hex_file).expect("patch");
        assert!(matches!(
            detect_lockout(&hex_file).expect("detect"),
            LockoutStatus::AlreadyUnlocked { .. }
        ));
    }

    #[test]
    fn test_legacy_firmware_reports_unlocked() {
        let legacy_paths = [
            "blobs/displays/t24v/TO7-T24V-Y240527A.hex",
            "blobs/displays/lcd135c/TO7-LCD135C-Y240621A.hex",
            "blobs/displays/m15/TO7-M15-Y250928.hex",
        ];
        for rel in legacy_paths {
            let Some(path) = find_blob(rel) else {
                continue;
            };
            let content = fs::read_to_string(&path).expect("read hex");
            let hex_file = IntelHexFile::parse(&content).expect("parse hex");
            let status = detect_lockout(&hex_file).expect("detect");
            assert_eq!(
                status,
                LockoutStatus::LegacyUnlocked,
                "Expected legacy unlocked for {}",
                rel
            );
        }
    }
}
