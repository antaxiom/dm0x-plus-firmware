//! # Display UART Communication Protocol & `0x69` Stream Cipher (`V6510C`)
//!
//! Implements the motor-controller-to-display UART protocol, including:
//! - The byte-wise `0x69` XOR stream cipher (`0x080095D8` RX and `0x0800966C` TX in `V6510C`)
//! - Live telemetry frame serialization/deserialization
//! - CRC-16 verification

use crate::config::RidingMode;
use crate::eeprom::crc16_toseven;
use crate::faults::FaultFlags;

/// Start-of-frame marker for UART telemetry and configuration frames.
pub const FRAME_SOF: u8 = 0x5A;

/// Frame type identifier for display-to-controller ride commands.
pub const FRAME_TYPE_COMMAND: u8 = 0x0A;

/// Stream XOR key applied to every byte sent or received over the Display UART (`0x080095D8` / `0x0800966C`).
pub const UART_STREAM_XOR_KEY: u8 = 0x69;

/// Encrypts or decrypts a UART byte buffer in-place using the `V6510C` `0x69` stream cipher.
pub fn xor_uart_stream_in_place(buf: &mut [u8]) {
    for b in buf.iter_mut() {
        *b ^= UART_STREAM_XOR_KEY;
    }
}

/// Real-time telemetry packet sent from the motor controller to the display / tuning app.
/// Includes all live diagnostic fields visible in the Display "Test > Drive" menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryFrame {
    pub battery_voltage_deci_v: u16,
    pub battery_soc_percent: u8,
    pub phase_current_deci_a: u16,
    pub wheel_speed_deci_kph: u16,
    pub cadence_rpm: u16,
    pub torque_sensor_mv: u16,
    pub throttle_mv: u16,
    pub controller_temp_c: i8,
    pub faults: FaultFlags,
}

impl TelemetryFrame {
    pub const ENCODED_LEN: usize = 18;
    pub const ENCODED_WITH_FAULTS_LEN: usize = 20;

    /// Serializes the telemetry frame into an 18-byte plaintext buffer with CRC-16 integrity check.
    /// If `self.faults` is non-empty, encodes frame subtype `0x11` with `faults.0` in bytes `13..15`
    /// (preserving full 18-byte wire compatibility).
    pub fn encode(&self, out: &mut [u8; 18]) {
        out[0] = FRAME_SOF;
        if self.faults.is_empty() {
            out[1] = 0x01; // Frame type: Live Telemetry (no active faults)
            out[13..15].copy_from_slice(&self.throttle_mv.to_le_bytes());
        } else {
            out[1] = 0x11; // Frame type: Live Telemetry with FaultFlags in [13..15]
            out[13..15].copy_from_slice(&self.faults.0.to_le_bytes());
        }
        out[2..4].copy_from_slice(&self.battery_voltage_deci_v.to_le_bytes());
        out[4] = self.battery_soc_percent;
        out[5..7].copy_from_slice(&self.phase_current_deci_a.to_le_bytes());
        out[7..9].copy_from_slice(&self.wheel_speed_deci_kph.to_le_bytes());
        out[9..11].copy_from_slice(&self.cadence_rpm.to_le_bytes());
        out[11..13].copy_from_slice(&self.torque_sensor_mv.to_le_bytes());
        out[15] = self.controller_temp_c as u8;
        let crc = crc16_toseven(&out[0..16]);
        out[16..18].copy_from_slice(&crc.to_le_bytes());
    }

    /// Serializes all telemetry fields (including both `throttle_mv` and `faults`) into a 20-byte plaintext buffer.
    pub fn encode_ext(&self, out: &mut [u8; 20]) {
        out[0] = FRAME_SOF;
        out[1] = 0x02; // Frame type: Extended Live Telemetry (with throttle_mv + faults)
        out[2..4].copy_from_slice(&self.battery_voltage_deci_v.to_le_bytes());
        out[4] = self.battery_soc_percent;
        out[5..7].copy_from_slice(&self.phase_current_deci_a.to_le_bytes());
        out[7..9].copy_from_slice(&self.wheel_speed_deci_kph.to_le_bytes());
        out[9..11].copy_from_slice(&self.cadence_rpm.to_le_bytes());
        out[11..13].copy_from_slice(&self.torque_sensor_mv.to_le_bytes());
        out[13..15].copy_from_slice(&self.throttle_mv.to_le_bytes());
        out[15] = self.controller_temp_c as u8;
        out[16..18].copy_from_slice(&self.faults.0.to_le_bytes());
        let crc = crc16_toseven(&out[0..18]);
        out[18..20].copy_from_slice(&crc.to_le_bytes());
    }

    /// Serializes the telemetry frame and applies the `0x69` UART stream XOR cipher for wire transmission.
    pub fn encode_encrypted_wire(&self, out: &mut [u8; 18]) {
        self.encode(out);
        xor_uart_stream_in_place(out);
    }

    /// Decodes and verifies an 18-byte plaintext telemetry frame.
    pub fn decode(buf: &[u8; 18]) -> Option<Self> {
        if buf[0] != FRAME_SOF || (buf[1] != 0x01 && buf[1] != 0x11) {
            return None;
        }
        let expected_crc = crc16_toseven(&buf[0..16]);
        let actual_crc = u16::from_le_bytes([buf[16], buf[17]]);
        if expected_crc != actual_crc {
            return None;
        }
        let (throttle_mv, faults) = if buf[1] == 0x11 {
            (0, FaultFlags(u16::from_le_bytes([buf[13], buf[14]])))
        } else {
            (u16::from_le_bytes([buf[13], buf[14]]), FaultFlags::NONE)
        };
        Some(Self {
            battery_voltage_deci_v: u16::from_le_bytes([buf[2], buf[3]]),
            battery_soc_percent: buf[4],
            phase_current_deci_a: u16::from_le_bytes([buf[5], buf[6]]),
            wheel_speed_deci_kph: u16::from_le_bytes([buf[7], buf[8]]),
            cadence_rpm: u16::from_le_bytes([buf[9], buf[10]]),
            torque_sensor_mv: u16::from_le_bytes([buf[11], buf[12]]),
            throttle_mv,
            controller_temp_c: buf[15] as i8,
            faults,
        })
    }

    /// Decodes and verifies a 20-byte extended plaintext telemetry frame (`0x02`).
    pub fn decode_ext(buf: &[u8; 20]) -> Option<Self> {
        if buf[0] != FRAME_SOF || buf[1] != 0x02 {
            return None;
        }
        let expected_crc = crc16_toseven(&buf[0..18]);
        let actual_crc = u16::from_le_bytes([buf[18], buf[19]]);
        if expected_crc != actual_crc {
            return None;
        }
        Some(Self {
            battery_voltage_deci_v: u16::from_le_bytes([buf[2], buf[3]]),
            battery_soc_percent: buf[4],
            phase_current_deci_a: u16::from_le_bytes([buf[5], buf[6]]),
            wheel_speed_deci_kph: u16::from_le_bytes([buf[7], buf[8]]),
            cadence_rpm: u16::from_le_bytes([buf[9], buf[10]]),
            torque_sensor_mv: u16::from_le_bytes([buf[11], buf[12]]),
            throttle_mv: u16::from_le_bytes([buf[13], buf[14]]),
            controller_temp_c: buf[15] as i8,
            faults: FaultFlags(u16::from_le_bytes([buf[16], buf[17]])),
        })
    }

    /// Decrypts an 18-byte wire frame using the `0x69` UART XOR cipher and verifies its CRC-16.
    pub fn decode_encrypted_wire(wire_buf: &[u8; 18]) -> Option<Self> {
        let mut plain = *wire_buf;
        xor_uart_stream_in_place(&mut plain);
        Self::decode(&plain)
    }
}

/// Command frame sent from the handlebar display to the motor controller (`0x0A`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayCommandFrame {
    pub headlight_on: bool,
    pub walk_assist_active: bool,
    pub mode: RidingMode,
    pub assist_level: u8,
}

impl DisplayCommandFrame {
    pub const ENCODED_LEN: usize = 8;

    /// Serializes command frame into an 8-byte plaintext buffer with ToSeven CRC-16.
    pub fn encode(&self, out: &mut [u8; 8]) {
        out[0] = FRAME_SOF;
        out[1] = FRAME_TYPE_COMMAND;
        out[2] = self.headlight_on as u8;
        out[3] = self.walk_assist_active as u8;
        out[4] = self.mode.to_wire_u8();
        out[5] = self.assist_level;
        let crc = crc16_toseven(&out[0..6]);
        out[6..8].copy_from_slice(&crc.to_le_bytes());
    }

    /// Serializes command frame and applies 0x69 UART wire XOR cipher.
    pub fn encode_encrypted_wire(&self, out: &mut [u8; 8]) {
        self.encode(out);
        xor_uart_stream_in_place(out);
    }

    /// Decodes an 8-byte plaintext command frame and validates its CRC-16.
    pub fn decode(buf: &[u8; 8]) -> Option<Self> {
        if buf[0] != FRAME_SOF || buf[1] != FRAME_TYPE_COMMAND {
            return None;
        }
        let expected_crc = crc16_toseven(&buf[0..6]);
        let actual_crc = u16::from_le_bytes([buf[6], buf[7]]);
        if expected_crc != actual_crc {
            return None;
        }
        Some(Self {
            headlight_on: buf[2] != 0,
            walk_assist_active: buf[3] != 0,
            mode: RidingMode::from_wire_u8(buf[4]),
            assist_level: buf[5],
        })
    }

    /// Decrypts an 8-byte encrypted wire buffer and decodes the command frame.
    pub fn decode_encrypted_wire(wire_buf: &[u8; 8]) -> Option<Self> {
        let mut plain = *wire_buf;
        xor_uart_stream_in_place(&mut plain);
        Self::decode(&plain)
    }
}

/// Zero-allocation sliding-window synchronizer for parsing incoming DisplayCommandFrames
/// byte-by-byte from USART3 RX in real-time embedded environments.
#[derive(Debug, Clone)]
pub struct DisplayCommandDecoder {
    buf: [u8; 8],
    len: usize,
}

impl Default for DisplayCommandDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayCommandDecoder {
    pub const fn new() -> Self {
        Self {
            buf: [0; 8],
            len: 0,
        }
    }

    pub fn reset(&mut self) {
        self.len = 0;
    }

    /// Feeds one already-decrypted plaintext byte (e.g. `raw_rx_byte ^ 0x69`).
    /// Returns `Some(DisplayCommandFrame)` when a valid complete frame is received.
    pub fn push_byte(&mut self, b: u8) -> Option<DisplayCommandFrame> {
        if self.len == 0 {
            if b == FRAME_SOF {
                self.buf[0] = b;
                self.len = 1;
            }
            return None;
        }

        if self.len == 1 {
            if b == FRAME_TYPE_COMMAND {
                self.buf[1] = b;
                self.len = 2;
            } else if b == FRAME_SOF {
                self.buf[0] = b;
                self.len = 1;
            } else {
                self.len = 0;
            }
            return None;
        }

        self.buf[self.len] = b;
        self.len += 1;

        if self.len == 8 {
            self.len = 0;
            DisplayCommandFrame::decode(&self.buf)
        } else {
            None
        }
    }
}

/// Computes CRC-16-CCITT (polynomial 0x1021, initial 0xFFFF).
pub const fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    let mut i = 0;
    while i < data.len() {
        crc ^= (data[i] as u16) << 8;
        let mut bit = 0;
        while bit < 8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
            bit += 1;
        }
        i += 1;
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_frame_roundtrip_and_wire_cipher() {
        let frame = TelemetryFrame {
            battery_voltage_deci_v: 524,
            battery_soc_percent: 88,
            phase_current_deci_a: 145,
            wheel_speed_deci_kph: 315,
            cadence_rpm: 82,
            torque_sensor_mv: 1450,
            throttle_mv: 700,
            controller_temp_c: 54,
            faults: FaultFlags::NONE,
        };
        let mut wire = [0u8; 18];
        frame.encode_encrypted_wire(&mut wire);
        assert_eq!(wire[0], FRAME_SOF ^ UART_STREAM_XOR_KEY);

        let decoded = TelemetryFrame::decode_encrypted_wire(&wire).expect("valid encrypted frame");
        assert_eq!(decoded, frame);

        // Corrupt 1 bit on the wire and verify rejection
        wire[5] ^= 0x01;
        assert!(TelemetryFrame::decode_encrypted_wire(&wire).is_none());
    }

    #[test]
    fn test_telemetry_frame_with_faults_roundtrip() {
        let fault_frame = TelemetryFrame {
            battery_voltage_deci_v: 430,
            battery_soc_percent: 12,
            phase_current_deci_a: 0,
            wheel_speed_deci_kph: 0,
            cadence_rpm: 0,
            torque_sensor_mv: 750,
            throttle_mv: 0,
            controller_temp_c: 92,
            faults: FaultFlags(FaultFlags::CONTROLLER_OVER_TEMP | FaultFlags::LOW_VOLTAGE),
        };
        let mut wire = [0u8; 18];
        fault_frame.encode_encrypted_wire(&mut wire);
        let decoded = TelemetryFrame::decode_encrypted_wire(&wire).expect("valid fault frame");
        assert_eq!(decoded, fault_frame);

        // Extended 20-byte frame preserves both non-zero throttle_mv and faults
        let ext_frame = TelemetryFrame {
            throttle_mv: 1250,
            ..fault_frame
        };
        let mut ext_buf = [0u8; 20];
        ext_frame.encode_ext(&mut ext_buf);
        let ext_decoded = TelemetryFrame::decode_ext(&ext_buf).expect("valid ext frame");
        assert_eq!(ext_decoded, ext_frame);
    }

    #[test]
    fn test_display_command_frame_roundtrip_and_decoder() {
        let cmd = DisplayCommandFrame {
            headlight_on: true,
            walk_assist_active: false,
            mode: RidingMode::Sport,
            assist_level: 4,
        };
        let mut wire = [0u8; 8];
        cmd.encode_encrypted_wire(&mut wire);
        assert_eq!(wire[0], FRAME_SOF ^ UART_STREAM_XOR_KEY);

        let decoded =
            DisplayCommandFrame::decode_encrypted_wire(&wire).expect("valid command frame");
        assert_eq!(decoded, cmd);

        // Test streaming decoder with noise prefix
        let mut decoder = DisplayCommandDecoder::new();
        let noise = [0xFF, 0x00, 0x5A, 0x99]; // False start
        for &b in &noise {
            assert!(decoder.push_byte(b).is_none());
        }

        let mut plain = [0u8; 8];
        cmd.encode(&mut plain);
        let mut result = None;
        for &b in &plain {
            if let Some(f) = decoder.push_byte(b) {
                result = Some(f);
            }
        }
        assert_eq!(result, Some(cmd));
    }
}
