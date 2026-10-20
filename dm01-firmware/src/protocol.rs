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

    /// Evaluates pre-flight safety criteria from standard 18-byte telemetry.
    pub fn evaluate_pre_flight(&self) -> PreFlightTriageResult {
        let hall_valid = !self.faults.contains(FaultFlags::MOTOR_HALL_FAIL);
        let hall_code = if hall_valid { 1 } else { 0 };
        let torque_zero_valid = (400..=1000).contains(&self.torque_sensor_mv);
        let throttle_idle_valid = self.throttle_mv <= 1050;
        let temps_valid = (-20..=65).contains(&self.controller_temp_c);
        let all_passed = hall_valid && torque_zero_valid && throttle_idle_valid && temps_valid;

        PreFlightTriageResult {
            hall_code,
            hall_valid,
            torque_zero_mv: self.torque_sensor_mv,
            torque_zero_valid,
            throttle_idle_mv: self.throttle_mv,
            throttle_idle_valid,
            controller_temp_c: self.controller_temp_c,
            motor_temp_c: 25, // Not reported on 18-byte display stream
            temps_valid,
            all_passed,
        }
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

/// Diagnostic extended sensor frame (matching the 64-byte `SENSORDATA` packet decompiled from `GlobalVar$1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtendedSensorFrame {
    pub hall_bits: u8,
    pub shift_sensors: u8,
    pub cadence_rpm: u16,
    pub torque_sensor_mv: u16,
    pub throttle_mv: u16,
    pub assist_level: u8,
    pub motor_temp_c: i8,
    pub controller_temp_c: i8,
    pub motor_electrical_rpm: u16,
    pub battery_voltage_deci_v: u16,
    pub phase_current_deci_a: u16,
    pub bus_current_deci_a: u16,
    pub wheel_speed_deci_kph: u16,
    pub trip_deci_km: u16,
    pub lifetime_odo_deci_km: u32,
    pub headlight_on: bool,
}

/// Pre-flight sensor triage result for safe bench bring-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreFlightTriageResult {
    pub hall_code: u8,
    pub hall_valid: bool,
    pub torque_zero_mv: u16,
    pub torque_zero_valid: bool,
    pub throttle_idle_mv: u16,
    pub throttle_idle_valid: bool,
    pub controller_temp_c: i8,
    pub motor_temp_c: i8,
    pub temps_valid: bool,
    pub all_passed: bool,
}

impl ExtendedSensorFrame {
    pub const FRAME_LEN: usize = 64;

    /// Evaluates pre-flight safety criteria for bench testing.
    pub fn evaluate_pre_flight(&self) -> PreFlightTriageResult {
        let hall_code = self.hall_bits & 0b111;
        let hall_valid = (1..=6).contains(&hall_code);
        let torque_zero_valid = (400..=1000).contains(&self.torque_sensor_mv);
        let throttle_idle_valid = self.throttle_mv <= 1050;
        let temps_valid =
            (-20..=65).contains(&self.controller_temp_c) && (-20..=65).contains(&self.motor_temp_c);
        let all_passed = hall_valid && torque_zero_valid && throttle_idle_valid && temps_valid;

        PreFlightTriageResult {
            hall_code,
            hall_valid,
            torque_zero_mv: self.torque_sensor_mv,
            torque_zero_valid,
            throttle_idle_mv: self.throttle_mv,
            throttle_idle_valid,
            controller_temp_c: self.controller_temp_c,
            motor_temp_c: self.motor_temp_c,
            temps_valid,
            all_passed,
        }
    }

    /// Serializes the diagnostic frame into the 64-byte wire buffer (matching decompiled offsets).
    pub fn encode(&self, out: &mut [u8; 64]) {
        out.fill(0);
        out[0] = 0x7E;
        out[1] = 0x40; // 64 bytes length
        out[2] = 0x3C;
        out[3] = 0xC3;
        out[4] = 0x00; // broadcast
        out[5] = 0x50; // cmd50 diagnostic response
        out[6] = 0x01;

        // Byte [26..28]: cadence
        out[26..28].copy_from_slice(&self.cadence_rpm.to_le_bytes());
        // Byte [29..31]: torque ADC
        out[29..31].copy_from_slice(&self.torque_sensor_mv.to_le_bytes());
        // Byte [31..33]: throttle ADC
        out[31..33].copy_from_slice(&self.throttle_mv.to_le_bytes());
        out[33] = self.assist_level;

        // Byte 44: digital bitmask (Hall 0..2, Shift 3..4)
        out[44] = (self.hall_bits & 0b111) | ((self.shift_sensors & 0b11) << 3);

        // Bytes 45..46: temperatures
        out[45] = self.motor_temp_c as u8;
        out[46] = self.controller_temp_c as u8;

        // Bytes 47..49: motor electrical RPM
        out[47..49].copy_from_slice(&self.motor_electrical_rpm.to_le_bytes());
        // Bytes 49..51: battery voltage
        out[49..51].copy_from_slice(&self.battery_voltage_deci_v.to_le_bytes());
        // Bytes 51..53: phase current
        out[51..53].copy_from_slice(&self.phase_current_deci_a.to_le_bytes());
        // Bytes 53..55: DC bus current
        out[53..55].copy_from_slice(&self.bus_current_deci_a.to_le_bytes());
        // Bytes 55..57: wheel speed
        out[55..57].copy_from_slice(&self.wheel_speed_deci_kph.to_le_bytes());
        // Bytes 57..59: trip
        out[57..59].copy_from_slice(&self.trip_deci_km.to_le_bytes());
        // Bytes 59..63: lifetime ODO
        out[59..63].copy_from_slice(&self.lifetime_odo_deci_km.to_le_bytes());
        out[63] = 0x7F; // EOF
    }

    /// Decodes a 64-byte diagnostic frame from raw bytes.
    pub fn decode(buf: &[u8; 64]) -> Option<Self> {
        if buf[0] != 0x7E || buf[2] != 0x3C || buf[3] != 0xC3 || buf[63] != 0x7F {
            return None;
        }
        let hall_bits = buf[44] & 0b111;
        let shift_sensors = (buf[44] >> 3) & 0b11;
        Some(Self {
            hall_bits,
            shift_sensors,
            cadence_rpm: u16::from_le_bytes([buf[26], buf[27]]),
            torque_sensor_mv: u16::from_le_bytes([buf[29], buf[30]]),
            throttle_mv: u16::from_le_bytes([buf[31], buf[32]]),
            assist_level: buf[33],
            motor_temp_c: buf[45] as i8,
            controller_temp_c: buf[46] as i8,
            motor_electrical_rpm: u16::from_le_bytes([buf[47], buf[48]]),
            battery_voltage_deci_v: u16::from_le_bytes([buf[49], buf[50]]),
            phase_current_deci_a: u16::from_le_bytes([buf[51], buf[52]]),
            bus_current_deci_a: u16::from_le_bytes([buf[53], buf[54]]),
            wheel_speed_deci_kph: u16::from_le_bytes([buf[55], buf[56]]),
            trip_deci_km: u16::from_le_bytes([buf[57], buf[58]]),
            lifetime_odo_deci_km: u32::from_le_bytes([buf[59], buf[60], buf[61], buf[62]]),
            headlight_on: (buf[44] & 0x80) != 0,
        })
    }
}

/// Real-time live parameter update frame (changes controller RAM without requiring a reboot).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveParamUpdateFrame {
    pub max_sport_current_a: u8,
    pub max_city_current_a: u8,
    pub max_eco_current_a: u8,
    pub speed_limit_kph: u8,
    pub throttle_mode: u8,
    pub assist_ceiling_policy: u8,
}

impl LiveParamUpdateFrame {
    pub const ENCODED_LEN: usize = 16;

    /// Serializes a live parameter tune request wrapped in standard `0x7E..0x7F` framing.
    pub fn encode(&self, out: &mut [u8; 16]) {
        out[0] = 0x7E;
        out[1] = 0x10; // 16 bytes length
        out[2] = 0x3C;
        out[3] = 0xC3;
        out[4] = 0x02; // Target: Motor Controller
        out[5] = 0x95; // Live Tune Command
        out[6] = 0x01; // Sequence
        out[7] = self.max_sport_current_a;
        out[8] = self.max_city_current_a;
        out[9] = self.max_eco_current_a;
        out[10] = self.speed_limit_kph;
        out[11] = self.throttle_mode;
        out[12] = self.assist_ceiling_policy;
        let crc = crc16_toseven(&out[0..13]);
        out[13..15].copy_from_slice(&crc.to_le_bytes());
        out[15] = 0x7F;
    }

    /// Decodes a 16-byte live parameter tune request and verifies its checksum.
    pub fn decode(buf: &[u8; 16]) -> Option<Self> {
        if buf[0] != 0x7E || buf[2] != 0x3C || buf[3] != 0xC3 || buf[5] != 0x95 || buf[15] != 0x7F {
            return None;
        }
        let expected_crc = crc16_toseven(&buf[0..13]);
        let actual_crc = u16::from_le_bytes([buf[13], buf[14]]);
        if expected_crc != actual_crc {
            return None;
        }
        Some(Self {
            max_sport_current_a: buf[7],
            max_city_current_a: buf[8],
            max_eco_current_a: buf[9],
            speed_limit_kph: buf[10],
            throttle_mode: buf[11],
            assist_ceiling_policy: buf[12],
        })
    }
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

    #[test]
    fn test_extended_sensor_frame_roundtrip_and_pre_flight_triage() {
        let sensor_frame = ExtendedSensorFrame {
            hall_bits: 0b011, // Hall code 3
            shift_sensors: 0,
            cadence_rpm: 75,
            torque_sensor_mv: 715,
            throttle_mv: 840,
            assist_level: 3,
            motor_temp_c: 24,
            controller_temp_c: 26,
            motor_electrical_rpm: 1200,
            battery_voltage_deci_v: 485,
            phase_current_deci_a: 120,
            bus_current_deci_a: 65,
            wheel_speed_deci_kph: 240,
            trip_deci_km: 142,
            lifetime_odo_deci_km: 12850,
            headlight_on: false,
        };

        let mut buf = [0u8; 64];
        sensor_frame.encode(&mut buf);
        let decoded = ExtendedSensorFrame::decode(&buf).expect("decode valid sensor frame");
        assert_eq!(decoded.hall_bits, 3);
        assert_eq!(decoded.torque_sensor_mv, 715);
        assert_eq!(decoded.throttle_mv, 840);
        assert_eq!(decoded.controller_temp_c, 26);
        assert_eq!(decoded.battery_voltage_deci_v, 485);

        // Pre-flight check should pass!
        let triage = decoded.evaluate_pre_flight();
        assert!(
            triage.all_passed,
            "Clean sensor readings must pass pre-flight triage"
        );
        assert_eq!(triage.hall_code, 3);
        assert!(triage.hall_valid);
        assert!(triage.torque_zero_valid);
        assert!(triage.throttle_idle_valid);
        assert!(triage.temps_valid);

        // Corrupted throttle reading (e.g. 1500 mV stuck at idle) should fail pre-flight!
        let bad_throttle = ExtendedSensorFrame {
            throttle_mv: 1500,
            ..sensor_frame
        };
        let bad_triage = bad_throttle.evaluate_pre_flight();
        assert!(!bad_triage.all_passed);
        assert!(!bad_triage.throttle_idle_valid);
    }

    #[test]
    fn test_telemetry_frame_pre_flight_triage() {
        let nominal = TelemetryFrame {
            battery_voltage_deci_v: 480,
            battery_soc_percent: 85,
            phase_current_deci_a: 0,
            wheel_speed_deci_kph: 0,
            cadence_rpm: 0,
            torque_sensor_mv: 745,
            throttle_mv: 850,
            controller_temp_c: 24,
            faults: FaultFlags::NONE,
        };
        let res = nominal.evaluate_pre_flight();
        assert!(res.all_passed);
        assert!(res.hall_valid);
        assert!(res.torque_zero_valid);
        assert!(res.throttle_idle_valid);
        assert!(res.temps_valid);

        let hall_fault = TelemetryFrame {
            faults: FaultFlags(FaultFlags::MOTOR_HALL_FAIL),
            ..nominal
        };
        assert!(!hall_fault.evaluate_pre_flight().hall_valid);
        assert!(!hall_fault.evaluate_pre_flight().all_passed);
    }

    #[test]
    fn test_live_param_update_frame_roundtrip() {
        let update = LiveParamUpdateFrame {
            max_sport_current_a: 28,
            max_city_current_a: 16,
            max_eco_current_a: 10,
            speed_limit_kph: 32,
            throttle_mode: 7,
            assist_ceiling_policy: 1,
        };

        let mut buf = [0u8; 16];
        update.encode(&mut buf);
        let decoded = LiveParamUpdateFrame::decode(&buf).expect("valid update frame");
        assert_eq!(decoded, update);

        // Corrupt 1 byte in CRC
        buf[13] ^= 0x55;
        assert!(LiveParamUpdateFrame::decode(&buf).is_none());
    }
}
