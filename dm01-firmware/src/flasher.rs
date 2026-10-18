//! # ToSeven Motor Controller & Display Firmware Flashing Protocol (`flasher`)
//!
//! Clean-room implementation of the In-Application Programming (IAP) flashing protocol
//! reverse-engineered from the official MinShine / ToSeven Windows IAP utility (`TO7-G2 IAP Tool`)
//! and Android application (`cn.minshine.ebike`), as well as the MegaHunt ISP display protocol
//! (`MH2020C_ISP.exe`).
//!
//! ## Protocol Overview
//! 1. **Motor Controller IAP (UART):**
//!    - **Wakeup / Enter IAP:** A 21-byte packet sent at 9600 Baud with `0xC9, 0x77` sync bytes,
//!      7-bit big-endian encoded payload size, checksum, payload XOR key, customer ID, and ToSeven CRC-16,
//!      encrypted byte-wise with `XOR 0x69`.
//!    - **Handshake / Probe:** Flasher sends byte `0x02` at 115200 Baud; bootloader replies `0x02, 0xFD`.
//!    - **Header Handshake:** Flasher sends `0x06` (ACK), waits 10 ms, then transmits a 12-byte header frame:
//!      `[0xFF, 0xFF, ProgramSize (4B LE), Checksum (4B LE), PayloadKey (1B), BaudIndex (1B)]`.
//!      Bootloader validates and returns `0x06` ACK.
//!    - **Flash Programming:** Decrypted firmware payload streamed in 16-byte chunks.
//!      Every 1024 bytes (1 KB sector), flasher waits for `0x06` ACK from bootloader.
//!    - **Finalization:** Bootloader calculates final CRC and returns `0x04` (`EOT`).
//!
//! 2. **Display ISP (MegaHunt MH2020C / MH1903):**
//!    - **Framing:** `[0x02 (STX), CMD (1B), Length_LE (2B), Data (N bytes), CRC16_LE (2B)]`.
//!    - **CRC:** Standard CRC-16-CCITT (poly `0x1021`, init `0xFFFF`).

use crate::eeprom::crc16_toseven;

/// XOR cipher mask applied to all 21-byte command frames transmitted over the UART wire.
pub const UART_WIRE_MASK: u8 = 0x69;

/// 2-byte header synchronizer for controller IAP commands.
pub const WAKEUP_SYNC: [u8; 2] = [0xC9, 0x77];

/// Command ID for IAP Flash programming.
pub const CMD_IAP_PROGRAM: u8 = 0x5A;

/// Probe byte transmitted to check if MCU bootloader is active.
pub const BOOTLOADER_PROBE_BYTE: u8 = 0x02;

/// Response expected from MCU bootloader upon successful identification.
pub const BOOTLOADER_IDENT_REPLY: [u8; 2] = [0x02, 0xFD];

/// Single-byte ACK response sent/received during IAP handshakes.
pub const IAP_ACK_BYTE: u8 = 0x06;

/// Single-byte EOT (End of Transmission) response indicating flash verification succeeded.
pub const IAP_EOT_BYTE: u8 = 0x04;

/// Flash chunk size transmitted per write step (16 bytes).
pub const FLASH_CHUNK_SIZE: usize = 16;

/// Flash sector size requiring an ACK handshake from the bootloader (1024 bytes / 1 KB).
pub const FLASH_SECTOR_SIZE: usize = 1024;

/// Baud rate used for the 21-byte application-mode wakeup packet (9600 Baud).
pub const IAP_WAKEUP_BAUD: u32 = 9_600;

/// Baud rate used for bootloader probing, header exchange, and sector streaming (115200 Baud).
pub const IAP_BAUD_115200: u32 = 115_200;

/// MegaHunt display ISP STX synchronizer.
pub const DISPLAY_ISP_STX: u8 = 0x02;

/// MegaHunt display ISP commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DisplayIspCmd {
    Handshake1 = 0x20,
    Handshake2 = 0x21,
    ChipInfo = 0x22,
    ReadFlashId = 0x3A,
    EraseFlash = 0x3D,
    WriteFlashOption = 0x3E,
    ProgramFirmware = 0x50,
    ChipUpdate = 0x58,
}

/// Errors occurring during the IAP flashing process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlasherError {
    Timeout,
    IdentificationFailed,
    HeaderRejected,
    SectorAckTimeout(usize),
    VerificationFailed,
    BufferTooSmall,
    InvalidPacket,
    UnexpectedResponse(u8),
}

/// Splits a 32-bit integer into 5 7-bit big-endian bytes (holding up to 35 bits).
/// Used by the official ToSeven IAP tool for parameter encoding.
pub fn encode_7bit_u32(val: u32) -> [u8; 5] {
    [
        ((val >> 28) & 0x7F) as u8,
        ((val >> 21) & 0x7F) as u8,
        ((val >> 14) & 0x7F) as u8,
        ((val >> 7) & 0x7F) as u8,
        (val & 0x7F) as u8,
    ]
}

/// Decodes a 5-byte 7-bit big-endian slice back into a 32-bit integer.
pub fn decode_7bit_u32(b: &[u8; 5]) -> u32 {
    ((b[0] as u32 & 0x7F) << 28)
        | ((b[1] as u32 & 0x7F) << 21)
        | ((b[2] as u32 & 0x7F) << 14)
        | ((b[3] as u32 & 0x7F) << 7)
        | (b[4] as u32 & 0x7F)
}

/// Splits a 16-bit integer into 3 7-bit big-endian bytes (holding up to 21 bits).
pub fn encode_7bit_u16(val: u16) -> [u8; 3] {
    [
        ((val >> 14) & 0x7F) as u8,
        ((val >> 7) & 0x7F) as u8,
        (val & 0x7F) as u8,
    ]
}

/// Constructs the 21-byte "Enter IAP Mode" packet sent to force the controller MCU into bootloader mode.
///
/// Layout (unencrypted 21 bytes):
/// - `[0..1]`: `0xC9, 0x77` (Sync)
/// - `[2..6]`: `program_size` (5 bytes, 7-bit big-endian)
/// - `[7..11]`: `payload_checksum` (5 bytes, 7-bit big-endian)
/// - `[12]`: `payload_xor_key >> 4` (high nibble)
/// - `[13]`: `payload_xor_key & 0x0F` (low nibble)
/// - `[14]`: `0x5A` (CMD_IAP_PROGRAM)
/// - `[15..17]`: `customer_num` (3 bytes, 7-bit big-endian)
/// - `[18..20]`: ToSeven CRC-16 over bytes 0..17 (split into 3 7-bit bytes, with byte 20 `|= 0x80`)
///
/// The entire 21-byte frame is then XORed with `0x69` before transmission.
pub fn build_wakeup_packet(
    program_size: u32,
    payload_checksum: u32,
    payload_xor_key: u8,
    customer_num: u16,
) -> [u8; 21] {
    let mut pkt = [0u8; 21];
    pkt[0] = WAKEUP_SYNC[0];
    pkt[1] = WAKEUP_SYNC[1];

    let sz_enc = encode_7bit_u32(program_size);
    pkt[2..7].copy_from_slice(&sz_enc);

    let sum_enc = encode_7bit_u32(payload_checksum);
    pkt[7..12].copy_from_slice(&sum_enc);

    pkt[12] = (payload_xor_key >> 4) & 0x0F;
    pkt[13] = payload_xor_key & 0x0F;
    pkt[14] = CMD_IAP_PROGRAM;

    let cust_enc = encode_7bit_u16(customer_num);
    pkt[15..18].copy_from_slice(&cust_enc);

    let crc = crc16_toseven(&pkt[0..18]);
    let crc_enc = encode_7bit_u16(crc);
    pkt[18] = crc_enc[0];
    pkt[19] = crc_enc[1];
    pkt[20] = crc_enc[2] | 0x80;

    // Apply wire encryption (XOR 0x69)
    let mut encrypted = [0u8; 21];
    let mut i = 0;
    while i < 21 {
        encrypted[i] = pkt[i] ^ UART_WIRE_MASK;
        i += 1;
    }
    encrypted
}

/// Constructs the 12-byte Header packet transmitted to the bootloader immediately following handshake.
///
/// Layout:
/// - `[0..1]`: `0xFF, 0xFF`
/// - `[2..5]`: `program_size` (Little-Endian `u32`)
/// - `[6..9]`: `payload_checksum` (Little-Endian `u32`)
/// - `[10]`: `payload_xor_key`
/// - `[11]`: `baud_index` (0 = 38400, 1 = 57600, 2 = 115200)
pub fn build_header_packet(
    program_size: u32,
    payload_checksum: u32,
    payload_xor_key: u8,
    baud_index: u8,
) -> [u8; 12] {
    let mut pkt = [0u8; 12];
    pkt[0] = 0xFF;
    pkt[1] = 0xFF;
    pkt[2..6].copy_from_slice(&program_size.to_le_bytes());
    pkt[6..10].copy_from_slice(&payload_checksum.to_le_bytes());
    pkt[10] = payload_xor_key;
    pkt[11] = baud_index;
    pkt
}

/// Computes the 8-bit additive sum (modulo 256) of a byte slice.
pub fn calculate_checksum8(bytes: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    for &b in bytes {
        sum = sum.wrapping_add(b);
    }
    sum
}

/// Computes the 16-bit CRC-16-CCITT (poly `0x1021`, init `0xFFFF`) used by MegaHunt displays.
#[inline]
pub const fn calculate_crc16_ccitt(bytes: &[u8]) -> u16 {
    crate::protocol::crc16_ccitt(bytes)
}

/// Constructs a MegaHunt Display ISP packet frame.
///
/// Layout:
/// - `[0]`: `0x02` (STX)
/// - `[1]`: `cmd`
/// - `[2..3]`: `payload.len()` (Little-Endian `u16`)
/// - `[4..4+L-1]`: Payload data
/// - `[4+L..5+L]`: CRC-16-CCITT over bytes 0 through 3+L (Little-Endian `u16`)
pub fn build_display_isp_packet(
    cmd: u8,
    payload: &[u8],
    out: &mut [u8],
) -> Result<usize, FlasherError> {
    let total_len = 4 + payload.len() + 2;
    if out.len() < total_len {
        return Err(FlasherError::BufferTooSmall);
    }
    out[0] = DISPLAY_ISP_STX;
    out[1] = cmd;
    let len_u16 = payload.len() as u16;
    out[2..4].copy_from_slice(&len_u16.to_le_bytes());
    out[4..4 + payload.len()].copy_from_slice(payload);

    let crc = calculate_crc16_ccitt(&out[0..4 + payload.len()]);
    out[4 + payload.len()..total_len].copy_from_slice(&crc.to_le_bytes());
    Ok(total_len)
}

/// Parses and validates a MegaHunt Display ISP packet frame from a raw buffer.
pub fn parse_display_isp_packet(packet: &[u8]) -> Result<(u8, &[u8]), FlasherError> {
    if packet.len() < 6 {
        return Err(FlasherError::BufferTooSmall);
    }
    if packet[0] != DISPLAY_ISP_STX {
        return Err(FlasherError::InvalidPacket);
    }
    let cmd = packet[1];
    let len = u16::from_le_bytes([packet[2], packet[3]]) as usize;
    if packet.len() < 4 + len + 2 {
        return Err(FlasherError::BufferTooSmall);
    }
    let expected_crc = calculate_crc16_ccitt(&packet[0..4 + len]);
    let actual_crc = u16::from_le_bytes([packet[4 + len], packet[5 + len]]);
    if expected_crc != actual_crc {
        return Err(FlasherError::VerificationFailed);
    }
    Ok((cmd, &packet[4..4 + len]))
}

/// High-level lifecycle state machine for the ToSeven motor controller IAP flashing sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IapSessionState {
    /// Initial idle state.
    Idle,
    /// Probing for existing bootloader (`0x02` at 115200 Baud).
    Probing,
    /// MCU application active; sending 21-byte wakeup packet at 9600 Baud + toggling reset.
    Awakening,
    /// Bootloader acknowledged (`0x02, 0xFD`); sending ACK and 12-byte header.
    Connected,
    /// Header accepted; streaming 16-byte chunks of decrypted payload.
    Flashing {
        bytes_sent: usize,
        sector_bytes: usize,
        total_bytes: usize,
    },
    /// Waiting for 1 KB sector ACK (`0x06`).
    AwaitingSectorAck { sector_index: usize },
    /// All bytes sent; waiting for final verification EOT (`0x04`).
    AwaitingVerification,
    /// Flashing succeeded.
    Complete,
    /// Flashing failed with error.
    Failed(FlasherError),
}

/// Simulated session driver and packet tracker for clean unit testing and execution.
pub struct IapSession {
    pub state: IapSessionState,
    pub program_size: u32,
    pub payload_checksum: u32,
    pub payload_xor_key: u8,
    pub customer_num: u16,
    pub baud_index: u8,
}

impl IapSession {
    pub fn new(
        program_size: u32,
        payload_checksum: u32,
        payload_xor_key: u8,
        customer_num: u16,
        baud_index: u8,
    ) -> Self {
        Self {
            state: IapSessionState::Idle,
            program_size,
            payload_checksum,
            payload_xor_key,
            customer_num,
            baud_index,
        }
    }

    /// Generates the initial probe byte.
    pub fn start_probe(&mut self) -> u8 {
        self.state = IapSessionState::Probing;
        BOOTLOADER_PROBE_BYTE
    }

    /// Generates the 21-byte wakeup packet if probe timed out.
    pub fn wakeup_packet(&mut self) -> [u8; 21] {
        self.state = IapSessionState::Awakening;
        build_wakeup_packet(
            self.program_size,
            self.payload_checksum,
            self.payload_xor_key,
            self.customer_num,
        )
    }

    /// Consumes incoming bytes and transitions state.
    pub fn handle_byte(&mut self, b: u8) {
        match self.state {
            IapSessionState::Probing | IapSessionState::Awakening => {
                if b == BOOTLOADER_IDENT_REPLY[1] {
                    self.state = IapSessionState::Connected;
                }
            }
            IapSessionState::Connected => {
                if b == IAP_ACK_BYTE {
                    self.state = IapSessionState::Flashing {
                        bytes_sent: 0,
                        sector_bytes: 0,
                        total_bytes: self.program_size as usize,
                    };
                }
            }
            IapSessionState::AwaitingSectorAck { sector_index } => {
                if b == IAP_ACK_BYTE {
                    let completed_bytes =
                        ((sector_index + 1) * FLASH_SECTOR_SIZE).min(self.program_size as usize);
                    if completed_bytes >= self.program_size as usize {
                        self.state = IapSessionState::AwaitingVerification;
                    } else {
                        self.state = IapSessionState::Flashing {
                            bytes_sent: completed_bytes,
                            sector_bytes: 0,
                            total_bytes: self.program_size as usize,
                        };
                    }
                }
            }
            IapSessionState::AwaitingVerification => {
                if b == IAP_EOT_BYTE {
                    self.state = IapSessionState::Complete;
                } else {
                    self.state = IapSessionState::Failed(FlasherError::VerificationFailed);
                }
            }
            _ => {}
        }
    }
}

/// High-level lifecycle state machine for the MegaHunt display ISP flashing sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayIspSessionState {
    Idle,
    Handshake1,
    Handshake2,
    ReadingChipInfo,
    ReadingFlashId,
    ErasingFlash,
    WritingOption,
    Programming {
        bytes_sent: usize,
        total_bytes: usize,
    },
    Finalizing,
    Complete,
    Failed(FlasherError),
}

/// Simulated session driver and packet tracker for MegaHunt display ISP flashing.
pub struct DisplayIspSession {
    pub state: DisplayIspSessionState,
    pub base_addr: u32,
    pub total_size: usize,
}

impl DisplayIspSession {
    pub fn new(base_addr: u32, total_size: usize) -> Self {
        Self {
            state: DisplayIspSessionState::Idle,
            base_addr,
            total_size,
        }
    }

    pub fn build_handshake1(&mut self, out: &mut [u8]) -> Result<usize, FlasherError> {
        self.state = DisplayIspSessionState::Handshake1;
        build_display_isp_packet(DisplayIspCmd::Handshake1 as u8, &[], out)
    }

    pub fn build_handshake2(&mut self, out: &mut [u8]) -> Result<usize, FlasherError> {
        self.state = DisplayIspSessionState::Handshake2;
        build_display_isp_packet(DisplayIspCmd::Handshake2 as u8, &[], out)
    }

    pub fn build_chip_info(&mut self, out: &mut [u8]) -> Result<usize, FlasherError> {
        self.state = DisplayIspSessionState::ReadingChipInfo;
        build_display_isp_packet(DisplayIspCmd::ChipInfo as u8, &[], out)
    }

    pub fn build_read_flash_id(&mut self, out: &mut [u8]) -> Result<usize, FlasherError> {
        self.state = DisplayIspSessionState::ReadingFlashId;
        build_display_isp_packet(DisplayIspCmd::ReadFlashId as u8, &[], out)
    }

    pub fn build_erase_flash(
        &mut self,
        start_addr: u32,
        end_addr: u32,
        out: &mut [u8],
    ) -> Result<usize, FlasherError> {
        self.state = DisplayIspSessionState::ErasingFlash;
        let mut payload = [0u8; 8];
        payload[0..4].copy_from_slice(&start_addr.to_le_bytes());
        payload[4..8].copy_from_slice(&end_addr.to_le_bytes());
        build_display_isp_packet(DisplayIspCmd::EraseFlash as u8, &payload, out)
    }

    pub fn build_write_option(
        &mut self,
        option_flags: u32,
        out: &mut [u8],
    ) -> Result<usize, FlasherError> {
        self.state = DisplayIspSessionState::WritingOption;
        let payload = option_flags.to_le_bytes();
        build_display_isp_packet(DisplayIspCmd::WriteFlashOption as u8, &payload, out)
    }

    pub fn build_program_chunk(
        &mut self,
        addr: u32,
        chunk: &[u8],
        out: &mut [u8],
    ) -> Result<usize, FlasherError> {
        let total_payload_len = 4 + chunk.len();
        let total_frame_len = 4 + total_payload_len + 2;
        if out.len() < total_frame_len {
            return Err(FlasherError::BufferTooSmall);
        }
        out[0] = DISPLAY_ISP_STX;
        out[1] = DisplayIspCmd::ProgramFirmware as u8;
        let len_u16 = total_payload_len as u16;
        out[2..4].copy_from_slice(&len_u16.to_le_bytes());
        out[4..8].copy_from_slice(&addr.to_le_bytes());
        out[8..8 + chunk.len()].copy_from_slice(chunk);
        let crc = calculate_crc16_ccitt(&out[0..8 + chunk.len()]);
        out[8 + chunk.len()..total_frame_len].copy_from_slice(&crc.to_le_bytes());

        self.state = DisplayIspSessionState::Programming {
            bytes_sent: (addr - self.base_addr) as usize + chunk.len(),
            total_bytes: self.total_size,
        };
        Ok(total_frame_len)
    }

    pub fn build_chip_update(&mut self, out: &mut [u8]) -> Result<usize, FlasherError> {
        self.state = DisplayIspSessionState::Finalizing;
        build_display_isp_packet(DisplayIspCmd::ChipUpdate as u8, &[], out)
    }

    pub fn handle_response(&mut self, resp_cmd: u8) {
        match (self.state, resp_cmd) {
            (DisplayIspSessionState::Handshake1, 0x20) => {
                self.state = DisplayIspSessionState::Handshake2;
            }
            (DisplayIspSessionState::Handshake2, 0x21) => {
                self.state = DisplayIspSessionState::ReadingChipInfo;
            }
            (DisplayIspSessionState::ReadingChipInfo, 0x22) => {
                self.state = DisplayIspSessionState::ReadingFlashId;
            }
            (DisplayIspSessionState::ReadingFlashId, 0x3A) => {
                self.state = DisplayIspSessionState::ErasingFlash;
            }
            (DisplayIspSessionState::ErasingFlash, 0x3D) => {
                self.state = DisplayIspSessionState::WritingOption;
            }
            (DisplayIspSessionState::WritingOption, 0x3E) => {
                self.state = DisplayIspSessionState::Programming {
                    bytes_sent: 0,
                    total_bytes: self.total_size,
                };
            }
            (DisplayIspSessionState::Finalizing, 0x58) => {
                self.state = DisplayIspSessionState::Complete;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_7bit_encoding_and_decoding() {
        let val: u32 = 57708;
        let enc = encode_7bit_u32(val);
        let dec = decode_7bit_u32(&enc);
        assert_eq!(val, dec);

        let checksum: u32 = 0x0059E35F;
        let enc_sum = encode_7bit_u32(checksum);
        let dec_sum = decode_7bit_u32(&enc_sum);
        assert_eq!(checksum, dec_sum);
    }

    #[test]
    fn test_dm01_wakeup_packet_roundtrip() {
        // DM01 parameters from official TO7-DM01-K260521A-V6510C.Pack
        let p_size: u32 = 57708;
        let p_sum: u32 = 0x0059E35F;
        let p_key: u8 = 0x67;
        let cust: u16 = 0x196E;

        let encrypted = build_wakeup_packet(p_size, p_sum, p_key, cust);
        assert_eq!(encrypted.len(), 21);

        // Decrypt with UART_WIRE_MASK (0x69)
        let mut decrypted = [0u8; 21];
        for i in 0..21 {
            decrypted[i] = encrypted[i] ^ UART_WIRE_MASK;
        }

        // Verify sync bytes
        assert_eq!(decrypted[0..2], WAKEUP_SYNC);
        // Verify size
        let decoded_sz = decode_7bit_u32(&[
            decrypted[2],
            decrypted[3],
            decrypted[4],
            decrypted[5],
            decrypted[6],
        ]);
        assert_eq!(decoded_sz, p_size);

        // Verify checksum
        let decoded_sum = decode_7bit_u32(&[
            decrypted[7],
            decrypted[8],
            decrypted[9],
            decrypted[10],
            decrypted[11],
        ]);
        assert_eq!(decoded_sum, p_sum);

        // Verify XOR key nibbles
        assert_eq!(decrypted[12], (p_key >> 4) & 0x0F);
        assert_eq!(decrypted[13], p_key & 0x0F);
        assert_eq!(decrypted[14], CMD_IAP_PROGRAM);

        // Verify CRC
        let crc = crc16_toseven(&decrypted[0..18]);
        let crc_enc = encode_7bit_u16(crc);
        assert_eq!(decrypted[18], crc_enc[0]);
        assert_eq!(decrypted[19], crc_enc[1]);
        assert_eq!(decrypted[20], crc_enc[2] | 0x80);
    }

    #[test]
    fn test_dm01_header_packet_construction() {
        let p_size: u32 = 57708;
        let p_sum: u32 = 0x0059E35F;
        let p_key: u8 = 0x67;
        let baud_idx: u8 = 2; // 115200

        let header = build_header_packet(p_size, p_sum, p_key, baud_idx);
        assert_eq!(header[0..2], [0xFF, 0xFF]);
        assert_eq!(
            u32::from_le_bytes([header[2], header[3], header[4], header[5]]),
            p_size
        );
        assert_eq!(
            u32::from_le_bytes([header[6], header[7], header[8], header[9]]),
            p_sum
        );
        assert_eq!(header[10], p_key);
        assert_eq!(header[11], baud_idx);

        let chk8 = calculate_checksum8(&header);
        assert!(chk8 > 0);
    }

    #[test]
    fn test_display_isp_packet_framing_and_crc() {
        let payload = [0x11, 0x22, 0x33, 0x44];
        let mut buf = [0u8; 32];
        let len =
            build_display_isp_packet(DisplayIspCmd::EraseFlash as u8, &payload, &mut buf).unwrap();
        assert_eq!(len, 10); // 4 hdr + 4 data + 2 crc
        assert_eq!(buf[0], DISPLAY_ISP_STX);
        assert_eq!(buf[1], DisplayIspCmd::EraseFlash as u8);

        let (parsed_cmd, parsed_data) = parse_display_isp_packet(&buf[0..len]).unwrap();
        assert_eq!(parsed_cmd, DisplayIspCmd::EraseFlash as u8);
        assert_eq!(parsed_data, &payload);
    }

    #[test]
    fn test_display_isp_session_lifecycle() {
        let mut session = DisplayIspSession::new(0x0100_1000, 1024);
        assert_eq!(session.state, DisplayIspSessionState::Idle);

        let mut buf = [0u8; 512];
        let len1 = session.build_handshake1(&mut buf).unwrap();
        assert_eq!(len1, 6);
        assert_eq!(session.state, DisplayIspSessionState::Handshake1);

        session.handle_response(0x20);
        assert_eq!(session.state, DisplayIspSessionState::Handshake2);

        let len2 = session.build_handshake2(&mut buf).unwrap();
        assert_eq!(len2, 6);
        session.handle_response(0x21);
        assert_eq!(session.state, DisplayIspSessionState::ReadingChipInfo);

        let len3 = session.build_chip_info(&mut buf).unwrap();
        assert_eq!(len3, 6);
        session.handle_response(0x22);
        assert_eq!(session.state, DisplayIspSessionState::ReadingFlashId);

        let len4 = session.build_read_flash_id(&mut buf).unwrap();
        assert_eq!(len4, 6);
        session.handle_response(0x3A);
        assert_eq!(session.state, DisplayIspSessionState::ErasingFlash);

        let len_erase = session
            .build_erase_flash(0x0100_1000, 0x0100_1400, &mut buf)
            .unwrap();
        assert_eq!(len_erase, 14); // 4 hdr + 8 data + 2 crc
        session.handle_response(0x3D);
        assert_eq!(session.state, DisplayIspSessionState::WritingOption);

        let len_opt = session.build_write_option(0, &mut buf).unwrap();
        assert_eq!(len_opt, 10); // 4 hdr + 4 data + 2 crc
        session.handle_response(0x3E);
        assert_eq!(
            session.state,
            DisplayIspSessionState::Programming {
                bytes_sent: 0,
                total_bytes: 1024,
            }
        );

        let chunk = [0xAA; 256];
        let len_prog = session
            .build_program_chunk(0x0100_1000, &chunk, &mut buf)
            .unwrap();
        assert_eq!(len_prog, 266); // 4 hdr + 4 addr + 256 data + 2 crc
        assert_eq!(
            session.state,
            DisplayIspSessionState::Programming {
                bytes_sent: 256,
                total_bytes: 1024,
            }
        );

        let len_upd = session.build_chip_update(&mut buf).unwrap();
        assert_eq!(len_upd, 6);
        session.handle_response(0x58);
        assert_eq!(session.state, DisplayIspSessionState::Complete);
    }
}
