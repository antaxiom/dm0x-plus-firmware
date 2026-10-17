use crate::faults::FaultFlags;

/// Start-of-frame marker for UART telemetry and configuration frames.
pub const FRAME_SOF: u8 = 0x5A;

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

    /// Serializes the telemetry frame into an 18-byte buffer with CRC-16-CCITT integrity check.
    pub fn encode(&self, out: &mut [u8; 18]) {
        out[0] = FRAME_SOF;
        out[1] = 0x01; // Frame type: Live Telemetry
        out[2..4].copy_from_slice(&self.battery_voltage_deci_v.to_le_bytes());
        out[4] = self.battery_soc_percent;
        out[5..7].copy_from_slice(&self.phase_current_deci_a.to_le_bytes());
        out[7..9].copy_from_slice(&self.wheel_speed_deci_kph.to_le_bytes());
        out[9..11].copy_from_slice(&self.cadence_rpm.to_le_bytes());
        out[11..13].copy_from_slice(&self.torque_sensor_mv.to_le_bytes());
        out[13..15].copy_from_slice(&self.throttle_mv.to_le_bytes());
        out[15] = self.controller_temp_c as u8;
        let crc = crc16_ccitt(&out[0..16]);
        out[16..18].copy_from_slice(&crc.to_le_bytes());
    }

    /// Decodes and verifies an 18-byte telemetry frame.
    pub fn decode(buf: &[u8; 18]) -> Option<Self> {
        if buf[0] != FRAME_SOF || buf[1] != 0x01 {
            return None;
        }
        let expected_crc = crc16_ccitt(&buf[0..16]);
        let actual_crc = u16::from_le_bytes([buf[16], buf[17]]);
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
            faults: FaultFlags::NONE,
        })
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
    fn test_telemetry_frame_roundtrip_and_crc() {
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
        let mut buf = [0u8; 18];
        frame.encode(&mut buf);

        let decoded = TelemetryFrame::decode(&buf).expect("valid CRC");
        assert_eq!(decoded, frame);

        // Corrupt 1 bit and verify rejection
        buf[5] ^= 0x01;
        assert!(TelemetryFrame::decode(&buf).is_none());
    }
}
