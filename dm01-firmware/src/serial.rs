//! # Live Serial Transport & Telemetry Stream Decoder (`std`)
//!
//! Zero-dependency POSIX / CDC serial transport layer, end-to-end live flashing drivers
//! for ToSeven motor controllers (IAP) and MegaHunt displays (ISP), and real-time `0x69`
//! UART stream-cipher telemetry decoder.
//!
//! ## Architecture
//! - [`SerialTransport`] — Hardware-agnostic byte-stream trait supporting dynamic baud-rate
//!   switching (`9600` $\rightarrow$ `115200` Baud for motor controller wakeup) and timed reads.
//! - [`PosixSerialPort`] — Concrete serial driver for macOS (`/dev/cu.usbserial-*`, `/dev/tty.usbmodem*`)
//!   and Linux (`/dev/ttyUSB*`, `/dev/ttyACM*`) using POSIX `termios` (`stty`) raw 8N1 mode
//!   and non-blocking file descriptor polling.
//! - [`MockSerialPort`] — Deterministic in-memory transport simulating a real motor controller
//!   bootloader or MegaHunt display ROM for automated integration tests.
//! - [`execute_motor_iap_flash`] — Live motor controller flashing state machine (115200 probe,
//!   9600 Baud 21-byte wakeup fallback, 12-byte header + modulo-256 checksum, 16-byte chunks,
//!   1024-byte sector `0x06` ACKs, and final `0x04` EOT verification).
//! - [`execute_display_isp_flash`] — Live MegaHunt display ISP flashing state machine (Handshake1/2,
//!   ChipInfo, ReadFlashId, EraseFlash, WriteOption, 256-byte CRC-16-CCITT chunks, and ChipUpdate).
//! - [`TelemetryStreamDecoder`] — Sliding-window `0x69` stream-cipher frame synchronizer that
//!   extracts both verified [`TelemetryFrame`] packets and raw stock 32-byte ToSeven UART frames.

use crate::faults::FaultFlags;
use crate::flasher::{
    build_header_packet, build_wakeup_packet, calculate_checksum8, parse_display_isp_packet,
    DisplayIspCmd, DisplayIspSession, FlasherError, BOOTLOADER_IDENT_REPLY, BOOTLOADER_PROBE_BYTE,
    DISPLAY_ISP_STX, FLASH_CHUNK_SIZE, FLASH_SECTOR_SIZE, IAP_ACK_BYTE, IAP_BAUD_115200,
    IAP_EOT_BYTE, IAP_WAKEUP_BAUD,
};
use crate::protocol::{TelemetryFrame, FRAME_SOF, UART_STREAM_XOR_KEY};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

/// Transport abstraction for bidirectional serial / USB-CDC communication.
pub trait SerialTransport {
    /// Configures the serial line speed in bits per second (e.g. `9600` or `115200`) in 8N1 raw mode.
    fn set_baud_rate(&mut self, baud: u32) -> io::Result<()>;

    /// Writes the entire buffer to the serial line and flushes output.
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()>;

    /// Reads up to `buf.len()` bytes waiting up to `timeout`. Returns `Ok(0)` on timeout.
    fn read_timeout(&mut self, buf: &mut [u8], timeout: Duration) -> io::Result<usize>;

    /// Reads a single byte waiting up to `timeout`. Returns `Ok(None)` on timeout.
    fn read_byte_timeout(&mut self, timeout: Duration) -> io::Result<Option<u8>> {
        let mut b = [0u8; 1];
        let n = self.read_timeout(&mut b, timeout)?;
        if n == 1 {
            Ok(Some(b[0]))
        } else {
            Ok(None)
        }
    }

    /// Reads exactly `buf.len()` bytes before `timeout` expires, or returns `TimedOut`.
    fn read_exact_timeout(&mut self, buf: &mut [u8], timeout: Duration) -> io::Result<()> {
        let deadline = Instant::now() + timeout;
        let mut offset = 0;
        while offset < buf.len() {
            let now = Instant::now();
            if now >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "serial read_exact_timeout expired",
                ));
            }
            let rem = deadline - now;
            let n = self.read_timeout(&mut buf[offset..], rem)?;
            if n == 0 {
                thread::sleep(Duration::from_millis(1));
            } else {
                offset += n;
            }
        }
        Ok(())
    }
}

/// Zero-dependency POSIX serial port driver for macOS and Linux.
pub struct PosixSerialPort {
    path: String,
    file: File,
    baud_rate: u32,
}

impl PosixSerialPort {
    /// Opens a serial device node (e.g. `/dev/cu.usbserial-110` or `/dev/ttyUSB0`) in non-blocking
    /// raw 8N1 mode and configures the initial baud rate.
    /// On macOS, automatically converts `/dev/tty.*` to `/dev/cu.*` to avoid DCD carrier hangs.
    pub fn open(path: &str, initial_baud: u32) -> io::Result<Self> {
        let canonical_path = if cfg!(target_os = "macos") && path.starts_with("/dev/tty.") {
            path.replace("/dev/tty.", "/dev/cu.")
        } else {
            path.to_string()
        };

        #[cfg(unix)]
        let file = {
            use std::os::unix::fs::OpenOptionsExt;
            // O_NONBLOCK (0x0004 on macOS, 0x0800 on Linux) + O_NOCTTY (0x20000 on macOS, 0x0100 on Linux)
            #[cfg(target_os = "macos")]
            const CUSTOM_FLAGS: i32 = 0x0004 | 0x20000;
            #[cfg(target_os = "linux")]
            const CUSTOM_FLAGS: i32 = 0x0800 | 0x0100;
            #[cfg(not(any(target_os = "macos", target_os = "linux")))]
            const CUSTOM_FLAGS: i32 = 0;

            OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(CUSTOM_FLAGS)
                .open(&canonical_path)?
        };

        #[cfg(not(unix))]
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&canonical_path)?;

        let mut port = Self {
            path: canonical_path,
            file,
            baud_rate: initial_baud,
        };
        port.set_baud_rate(initial_baud)?;
        Ok(port)
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn baud_rate(&self) -> u32 {
        self.baud_rate
    }
}

/// Scans the system for connected USB serial ports (macOS `/dev/cu.usb*`, Linux `/dev/ttyUSB*`, etc.).
pub fn list_serial_ports() -> Vec<String> {
    let mut ports = Vec::new();
    #[cfg(unix)]
    {
        if let Ok(entries) = std::fs::read_dir("/dev") {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    let is_match = if cfg!(target_os = "macos") {
                        name.starts_with("cu.usb")
                            || name.starts_with("cu.wch")
                            || name.starts_with("cu.SLAB")
                            || name.starts_with("cu.usbmodem")
                    } else if cfg!(target_os = "linux") {
                        name.starts_with("ttyUSB") || name.starts_with("ttyACM")
                    } else {
                        false
                    };
                    if is_match {
                        ports.push(format!("/dev/{}", name));
                    }
                }
            }
        }
    }
    ports.sort();
    ports
}

impl SerialTransport for PosixSerialPort {
    fn set_baud_rate(&mut self, baud: u32) -> io::Result<()> {
        self.baud_rate = baud;
        #[cfg(unix)]
        {
            let flag = if cfg!(target_os = "macos") {
                "-f"
            } else {
                "-F"
            };
            let baud_str = baud.to_string();
            let status = Command::new("stty")
                .args([
                    flag, &self.path, &baud_str, "cs8", "-parenb", "-cstopb", "-icanon", "-echo",
                    "-echoe", "-isig", "-ixon", "-ixoff", "-icrnl", "-opost", "min", "0", "time",
                    "1",
                ])
                .status()?;
            if !status.success() {
                return Err(io::Error::other(format!(
                    "stty failed to configure {} at {} baud",
                    self.path, baud
                )));
            }
        }
        Ok(())
    }

    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        let mut offset = 0;
        while offset < bytes.len() {
            match self.file.write(&bytes[offset..]) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "serial write returned 0",
                    ))
                }
                Ok(n) => offset += n,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(1));
                }
                Err(e) => return Err(e),
            }
        }
        self.file.flush()
    }

    fn read_timeout(&mut self, buf: &mut [u8], timeout: Duration) -> io::Result<usize> {
        let deadline = Instant::now() + timeout;
        loop {
            match self.file.read(buf) {
                Ok(n) if n > 0 => return Ok(n),
                Ok(_) => {}
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e),
            }
            if Instant::now() >= deadline {
                return Ok(0);
            }
            thread::sleep(Duration::from_millis(2));
        }
    }
}

/// Progress callback events emitted during live motor controller IAP flashing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotorFlashProgress {
    ProbingBootloader,
    SendingWakeup9600,
    BootloaderIdentified,
    HeaderAccepted,
    SectorAcked {
        sector_index: usize,
        total_sectors: usize,
        bytes_sent: usize,
        total_bytes: usize,
    },
    VerifiedComplete,
}

/// Executes the complete live ToSeven motor controller IAP flashing sequence over any [`SerialTransport`].
///
/// Sequence:
/// 1. Configures `115200` Baud and sends probe byte `0x02`.
/// 2. If no `0x02, 0xFD` reply arrives within 250 ms, switches to `9600` Baud, transmits the
///    21-byte `0x69`-masked wakeup frame (`0xC9, 0x77, ...`), switches back to `115200` Baud,
///    and probes with `0x02` until the bootloader replies `0x02, 0xFD`.
/// 3. Sends `0x06` (`ACK`), 12-byte header frame, and 8-bit modulo-256 header checksum, then
///    waits for header `0x06` (`ACK`).
/// 4. Streams the decrypted firmware payload in 16-byte chunks, pausing after every 1024-byte
///    sector (64 chunks) to verify the bootloader's `0x06` sector `ACK`.
/// 5. Waits for the final `0x04` (`EOT`) global checksum verification byte.
pub fn execute_motor_iap_flash<T, F>(
    transport: &mut T,
    payload: &[u8],
    payload_xor_key: u8,
    customer_num: u16,
    mut on_progress: F,
) -> Result<(), FlasherError>
where
    T: SerialTransport,
    F: FnMut(MotorFlashProgress),
{
    if payload.is_empty() {
        return Err(FlasherError::InvalidPacket);
    }

    let program_size = payload.len() as u32;
    let mut payload_sum: u32 = 0;
    for &b in payload {
        payload_sum = payload_sum.wrapping_add(b as u32);
    }

    // Step 1: Probe bootloader at 115200 Baud
    on_progress(MotorFlashProgress::ProbingBootloader);
    transport
        .set_baud_rate(IAP_BAUD_115200)
        .map_err(|_| FlasherError::Timeout)?;
    transport
        .write_all(&[BOOTLOADER_PROBE_BYTE])
        .map_err(|_| FlasherError::Timeout)?;

    let mut connected = wait_for_ident(transport, Duration::from_millis(250));

    // Step 2: If application firmware is running, send 21-byte wakeup packet at 9600 Baud
    if !connected {
        on_progress(MotorFlashProgress::SendingWakeup9600);
        transport
            .set_baud_rate(IAP_WAKEUP_BAUD)
            .map_err(|_| FlasherError::Timeout)?;
        let wakeup = build_wakeup_packet(program_size, payload_sum, payload_xor_key, customer_num);
        transport
            .write_all(&wakeup)
            .map_err(|_| FlasherError::Timeout)?;

        // Switch back to 115200 Baud and poll with 0x02 probe
        transport
            .set_baud_rate(IAP_BAUD_115200)
            .map_err(|_| FlasherError::Timeout)?;
        for _ in 0..10 {
            transport
                .write_all(&[BOOTLOADER_PROBE_BYTE])
                .map_err(|_| FlasherError::Timeout)?;
            if wait_for_ident(transport, Duration::from_millis(150)) {
                connected = true;
                break;
            }
        }
    }

    if !connected {
        return Err(FlasherError::Timeout);
    }
    on_progress(MotorFlashProgress::BootloaderIdentified);

    // Step 3: Send ACK (0x06) + 12-byte Header + 1-byte Checksum8
    let baud_idx = 2u8; // 115200 Baud
    let hdr = build_header_packet(program_size, payload_sum, payload_xor_key, baud_idx);
    let hdr_chk = calculate_checksum8(&hdr);

    let mut hdr_frame = [0u8; 14];
    hdr_frame[0] = IAP_ACK_BYTE;
    hdr_frame[1..13].copy_from_slice(&hdr);
    hdr_frame[13] = hdr_chk;
    transport
        .write_all(&hdr_frame)
        .map_err(|_| FlasherError::Timeout)?;

    let ack = transport
        .read_byte_timeout(Duration::from_secs(2))
        .map_err(|_| FlasherError::Timeout)?;
    if ack != Some(IAP_ACK_BYTE) {
        return Err(FlasherError::UnexpectedResponse(ack.unwrap_or(0)));
    }
    on_progress(MotorFlashProgress::HeaderAccepted);

    // Step 4: Stream payload in 16-byte chunks, awaiting 0x06 ACK every 1024-byte sector
    let total_sectors = payload.len().div_ceil(FLASH_SECTOR_SIZE);
    for (sector_idx, sector) in payload.chunks(FLASH_SECTOR_SIZE).enumerate() {
        for chunk in sector.chunks(FLASH_CHUNK_SIZE) {
            transport
                .write_all(chunk)
                .map_err(|_| FlasherError::Timeout)?;
            // 1 ms inter-chunk pacing prevents UART RX FIFO overrun on MCU
            thread::sleep(Duration::from_millis(1));
        }

        let sector_ack = transport
            .read_byte_timeout(Duration::from_secs(3))
            .map_err(|_| FlasherError::Timeout)?;
        if sector_ack != Some(IAP_ACK_BYTE) {
            return Err(FlasherError::UnexpectedResponse(sector_ack.unwrap_or(0)));
        }

        let bytes_sent = ((sector_idx + 1) * FLASH_SECTOR_SIZE).min(payload.len());
        on_progress(MotorFlashProgress::SectorAcked {
            sector_index: sector_idx,
            total_sectors,
            bytes_sent,
            total_bytes: payload.len(),
        });
    }

    // Step 5: Wait for final verification EOT (0x04)
    let eot = transport
        .read_byte_timeout(Duration::from_secs(3))
        .map_err(|_| FlasherError::Timeout)?;
    if eot != Some(IAP_EOT_BYTE) {
        return Err(FlasherError::VerificationFailed);
    }

    on_progress(MotorFlashProgress::VerifiedComplete);
    Ok(())
}

fn wait_for_ident<T: SerialTransport>(transport: &mut T, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let mut saw_first = false;
    while Instant::now() < deadline {
        let rem = deadline.saturating_duration_since(Instant::now());
        if let Ok(Some(b)) = transport.read_byte_timeout(rem) {
            if b == BOOTLOADER_IDENT_REPLY[0] {
                saw_first = true;
            } else if saw_first && b == BOOTLOADER_IDENT_REPLY[1] {
                return true;
            } else {
                saw_first = false;
            }
        } else {
            break;
        }
    }
    false
}

/// Progress callback events emitted during live MegaHunt display ISP flashing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayFlashProgress {
    HandshakeComplete,
    ChipIdentified,
    FlashErased,
    PacketProgrammed {
        packet_index: usize,
        total_packets: usize,
        bytes_sent: usize,
        total_bytes: usize,
    },
    UpdateFinalized,
}

/// Executes the complete live MegaHunt (`MH2020C`/`MH1903`) Display ISP flashing sequence.
pub fn execute_display_isp_flash<T, F>(
    transport: &mut T,
    base_addr: u32,
    bin: &[u8],
    mut on_progress: F,
) -> Result<(), FlasherError>
where
    T: SerialTransport,
    F: FnMut(DisplayFlashProgress),
{
    if bin.is_empty() {
        return Err(FlasherError::InvalidPacket);
    }

    transport
        .set_baud_rate(115_200)
        .map_err(|_| FlasherError::Timeout)?;

    let mut session = DisplayIspSession::new(base_addr, bin.len());
    let mut tx_buf = [0u8; 512];

    // 1. Handshake 1 (0x20)
    let n = session.build_handshake1(&mut tx_buf)?;
    send_and_expect_isp_ack(transport, &tx_buf[..n], DisplayIspCmd::Handshake1 as u8)?;

    // 2. Handshake 2 (0x21)
    let n = session.build_handshake2(&mut tx_buf)?;
    send_and_expect_isp_ack(transport, &tx_buf[..n], DisplayIspCmd::Handshake2 as u8)?;
    on_progress(DisplayFlashProgress::HandshakeComplete);

    // 3. Chip Info (0x22)
    let n = session.build_chip_info(&mut tx_buf)?;
    send_and_expect_isp_ack(transport, &tx_buf[..n], DisplayIspCmd::ChipInfo as u8)?;

    // 4. Read Flash ID (0x3A)
    let n = session.build_read_flash_id(&mut tx_buf)?;
    send_and_expect_isp_ack(transport, &tx_buf[..n], DisplayIspCmd::ReadFlashId as u8)?;
    on_progress(DisplayFlashProgress::ChipIdentified);

    // 5. Erase Flash (0x3D)
    let end_addr = base_addr.saturating_add(bin.len() as u32);
    let n = session.build_erase_flash(base_addr, end_addr, &mut tx_buf)?;
    send_and_expect_isp_ack(transport, &tx_buf[..n], DisplayIspCmd::EraseFlash as u8)?;
    on_progress(DisplayFlashProgress::FlashErased);

    // 6. Write Flash Option (0x3E)
    let n = session.build_write_option(0, &mut tx_buf)?;
    send_and_expect_isp_ack(
        transport,
        &tx_buf[..n],
        DisplayIspCmd::WriteFlashOption as u8,
    )?;

    // 7. Program Firmware in 256-byte chunks (0x50)
    let chunk_size = 256;
    let total_packets = bin.len().div_ceil(chunk_size);
    for (idx, chunk) in bin.chunks(chunk_size).enumerate() {
        let addr = base_addr + (idx * chunk_size) as u32;
        let n = session.build_program_chunk(addr, chunk, &mut tx_buf)?;
        send_and_expect_isp_ack(
            transport,
            &tx_buf[..n],
            DisplayIspCmd::ProgramFirmware as u8,
        )?;
        let bytes_sent = ((idx + 1) * chunk_size).min(bin.len());
        on_progress(DisplayFlashProgress::PacketProgrammed {
            packet_index: idx,
            total_packets,
            bytes_sent,
            total_bytes: bin.len(),
        });
    }

    // 8. Finalize / Chip Update (0x58)
    let n = session.build_chip_update(&mut tx_buf)?;
    send_and_expect_isp_ack(transport, &tx_buf[..n], DisplayIspCmd::ChipUpdate as u8)?;
    on_progress(DisplayFlashProgress::UpdateFinalized);

    Ok(())
}

fn send_and_expect_isp_ack<T: SerialTransport>(
    transport: &mut T,
    tx_frame: &[u8],
    expected_cmd: u8,
) -> Result<(), FlasherError> {
    transport
        .write_all(tx_frame)
        .map_err(|_| FlasherError::Timeout)?;

    // Read STX (0x02), CMD, LEN_LO, LEN_HI
    let mut hdr = [0u8; 4];
    transport
        .read_exact_timeout(&mut hdr, Duration::from_secs(3))
        .map_err(|_| FlasherError::Timeout)?;
    if hdr[0] != DISPLAY_ISP_STX {
        return Err(FlasherError::InvalidPacket);
    }
    let payload_len = u16::from_le_bytes([hdr[2], hdr[3]]) as usize;
    if payload_len > 260 {
        return Err(FlasherError::InvalidPacket);
    }
    let total_len = 4 + payload_len + 2;
    let mut full = vec![0u8; total_len];
    full[0..4].copy_from_slice(&hdr);
    transport
        .read_exact_timeout(&mut full[4..], Duration::from_secs(3))
        .map_err(|_| FlasherError::Timeout)?;

    let (resp_cmd, _payload) = parse_display_isp_packet(&full)?;
    if resp_cmd != expected_cmd {
        return Err(FlasherError::UnexpectedResponse(resp_cmd));
    }
    Ok(())
}

/// Decoded UART monitor event from the motor controller / display bus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorEvent {
    /// Verified 18-byte or 20-byte [`TelemetryFrame`].
    Telemetry(TelemetryFrame),
    /// Raw `0x69`-decrypted ToSeven frame starting with `0x5A` (`FRAME_SOF`) that is not a type `0x01`/`0x11` telemetry frame.
    RawDecryptedFrame(Vec<u8>),
}

/// Sliding-window `0x69` stream-cipher decoder for live UART monitoring.
pub struct TelemetryStreamDecoder {
    plain_buf: VecDeque<u8>,
}

impl Default for TelemetryStreamDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryStreamDecoder {
    pub fn new() -> Self {
        Self {
            plain_buf: VecDeque::with_capacity(128),
        }
    }

    /// Feeds raw encrypted wire bytes (`0x69`-XORed on the bus) and returns any decoded frames.
    pub fn push_wire_bytes(&mut self, wire_bytes: &[u8]) -> Vec<MonitorEvent> {
        for &b in wire_bytes {
            self.plain_buf.push_back(b ^ UART_STREAM_XOR_KEY);
        }
        self.extract_events()
    }

    /// Feeds already-decrypted plaintext bytes and returns any decoded frames.
    pub fn push_plain_bytes(&mut self, plain_bytes: &[u8]) -> Vec<MonitorEvent> {
        for &b in plain_bytes {
            self.plain_buf.push_back(b);
        }
        self.extract_events()
    }

    fn extract_events(&mut self) -> Vec<MonitorEvent> {
        let mut events = Vec::new();
        loop {
            // Advance until FRAME_SOF (0x5A)
            while let Some(&front) = self.plain_buf.front() {
                if front == FRAME_SOF {
                    break;
                }
                self.plain_buf.pop_front();
            }

            if self.plain_buf.len() < TelemetryFrame::ENCODED_LEN {
                break;
            }

            let mut candidate18 = [0u8; 18];
            for (i, b) in self.plain_buf.iter().take(18).enumerate() {
                candidate18[i] = *b;
            }

            if let Some(frame) = TelemetryFrame::decode(&candidate18) {
                for _ in 0..18 {
                    self.plain_buf.pop_front();
                }
                events.push(MonitorEvent::Telemetry(frame));
                continue;
            }

            if self.plain_buf.len() >= TelemetryFrame::ENCODED_WITH_FAULTS_LEN {
                let mut candidate20 = [0u8; 20];
                for (i, b) in self.plain_buf.iter().take(20).enumerate() {
                    candidate20[i] = *b;
                }
                if let Some(frame) = TelemetryFrame::decode_ext(&candidate20) {
                    for _ in 0..20 {
                        self.plain_buf.pop_front();
                    }
                    events.push(MonitorEvent::Telemetry(frame));
                    continue;
                }
            }

            // If we have at least 32 bytes and it starts with 0x5A, emit a raw decrypted frame
            // snapshot if the next byte is not an valid telemetry frame header
            if self.plain_buf.len() >= 32 {
                let raw: Vec<u8> = self.plain_buf.iter().take(32).copied().collect();
                // Pop the 0x5A byte so we can continue scanning
                self.plain_buf.pop_front();
                events.push(MonitorEvent::RawDecryptedFrame(raw));
            } else {
                // Could be a false 0x5A byte in noise if buffer grows large; otherwise wait for more bytes
                if self.plain_buf.len() > 20 {
                    self.plain_buf.pop_front();
                } else {
                    break;
                }
            }
        }
        events
    }
}

/// Formats a [`TelemetryFrame`] as a human-readable single-line or multi-line terminal dashboard.
pub fn format_telemetry_row(step: usize, frame: &TelemetryFrame) -> String {
    let volts = frame.battery_voltage_deci_v as f32 / 10.0;
    let amps = frame.phase_current_deci_a as f32 / 10.0;
    let watts = volts * amps;
    let kph = frame.wheel_speed_deci_kph as f32 / 10.0;
    let fault_str = format_fault_flags(frame.faults);
    format!(
        "[{:04}] {:5.1}V ({:3}%) | {:5.1}A ({:6.1}W) | {:4.1} km/h | {:3} RPM | Trq:{:4}mV Thr:{:4}mV | {:2}°C | Faults: {}",
        step,
        volts,
        frame.battery_soc_percent,
        amps,
        watts,
        kph,
        frame.cadence_rpm,
        frame.torque_sensor_mv,
        frame.throttle_mv,
        frame.controller_temp_c,
        fault_str
    )
}

/// Returns a human-readable summary of active [`FaultFlags`].
pub fn format_fault_flags(faults: FaultFlags) -> String {
    if faults.is_empty() {
        return "NONE (OK)".to_string();
    }
    let mut names = Vec::new();
    let table: &[(u16, &str)] = &[
        (FaultFlags::PROTOCOL_DISMATCH, "PROTOCOL_DISMATCH"),
        (FaultFlags::COMM_RECEPTION_FAIL, "COMM_RECEPTION_FAIL"),
        (FaultFlags::CONTROLLER_MEMORY_FAIL, "CONTROLLER_MEMORY_FAIL"),
        (FaultFlags::GALLOP_THROTTLE_FAIL, "GALLOP_THROTTLE_FAIL"),
        (FaultFlags::CONTROLLER_MOSFET_FAIL, "CONTROLLER_MOSFET_FAIL"),
        (FaultFlags::MOTOR_BLOCKED, "MOTOR_BLOCKED"),
        (FaultFlags::SPEED_SENSOR_FAIL, "SPEED_SENSOR_FAIL"),
        (FaultFlags::MOTOR_HALL_FAIL, "MOTOR_HALL_FAIL"),
        (FaultFlags::CONTROLLER_OVER_TEMP, "CONTROLLER_OVER_TEMP"),
        (FaultFlags::MOTOR_OVER_TEMP, "MOTOR_OVER_TEMP"),
        (FaultFlags::TORQUE_SENSOR_FAULT, "TORQUE_SENSOR_FAULT"),
        (FaultFlags::LOW_VOLTAGE, "LOW_VOLTAGE"),
    ];
    for &(bit, name) in table {
        if faults.contains(bit) {
            names.push(name);
        }
    }
    names.join("|")
}

type WriteResponderCallback = Box<dyn FnMut(&[u8], u32, &mut VecDeque<u8>)>;

/// Deterministic in-memory mock serial port for testing IAP and ISP flashing state machines.
pub struct MockSerialPort {
    pub current_baud: u32,
    pub baud_history: Vec<u32>,
    pub tx_log: Vec<u8>,
    pub rx_queue: VecDeque<u8>,
    on_write: Option<WriteResponderCallback>,
}

impl Default for MockSerialPort {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSerialPort {
    pub fn new() -> Self {
        Self {
            current_baud: 115_200,
            baud_history: Vec::new(),
            tx_log: Vec::new(),
            rx_queue: VecDeque::new(),
            on_write: None,
        }
    }

    /// Creates a mock serial port that emulates a ToSeven DM01/DM02 motor controller:
    /// requires the 9600-baud 21-byte wakeup packet before answering the 115200-baud `0x02` probe,
    /// then validates the 12-byte header and ACKs every 1024-byte sector and final EOT.
    pub fn new_motor_bootloader_responder(expected_payload_len: usize) -> Self {
        let mut awakened = false;
        let mut awaiting_header = false;
        let mut header_buf = Vec::new();
        let mut stream_bytes_received = 0usize;
        let mut sector_accum = 0usize;

        let responder = move |written: &[u8], baud: u32, rx: &mut VecDeque<u8>| {
            if !awakened {
                if baud == IAP_WAKEUP_BAUD && written.len() == 21 {
                    awakened = true;
                }
                return;
            }
            if awaiting_header {
                header_buf.extend_from_slice(written);
                if header_buf.len() >= 14 {
                    // [0] = 0x06, [1..13] = 12B header, [13] = chk8
                    let chk = calculate_checksum8(&header_buf[1..13]);
                    if header_buf[0] == IAP_ACK_BYTE && chk == header_buf[13] {
                        rx.push_back(IAP_ACK_BYTE);
                        awaiting_header = false;
                    }
                }
                return;
            }
            if stream_bytes_received == 0 && written == [BOOTLOADER_PROBE_BYTE] {
                rx.push_back(BOOTLOADER_IDENT_REPLY[0]);
                rx.push_back(BOOTLOADER_IDENT_REPLY[1]);
                awaiting_header = true;
                return;
            }
            // Receiving 16-byte payload chunks
            stream_bytes_received += written.len();
            sector_accum += written.len();
            if sector_accum >= FLASH_SECTOR_SIZE || stream_bytes_received >= expected_payload_len {
                sector_accum = 0;
                rx.push_back(IAP_ACK_BYTE);
                if stream_bytes_received >= expected_payload_len {
                    rx.push_back(IAP_EOT_BYTE);
                }
            }
        };

        Self {
            current_baud: 115_200,
            baud_history: Vec::new(),
            tx_log: Vec::new(),
            rx_queue: VecDeque::new(),
            on_write: Some(Box::new(responder)),
        }
    }

    /// Creates a mock serial port that emulates a MegaHunt MH2020C display ROM ISP responder.
    pub fn new_display_isp_responder() -> Self {
        let responder = move |written: &[u8], _baud: u32, rx: &mut VecDeque<u8>| {
            if let Ok((cmd, _payload)) = parse_display_isp_packet(written) {
                let mut reply = [0u8; 16];
                if let Ok(n) = crate::flasher::build_display_isp_packet(cmd, &[0x00], &mut reply) {
                    for &b in &reply[..n] {
                        rx.push_back(b);
                    }
                }
            }
        };

        Self {
            current_baud: 115_200,
            baud_history: Vec::new(),
            tx_log: Vec::new(),
            rx_queue: VecDeque::new(),
            on_write: Some(Box::new(responder)),
        }
    }
}

impl SerialTransport for MockSerialPort {
    fn set_baud_rate(&mut self, baud: u32) -> io::Result<()> {
        self.current_baud = baud;
        self.baud_history.push(baud);
        Ok(())
    }

    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.tx_log.extend_from_slice(bytes);
        if let Some(ref mut cb) = self.on_write {
            cb(bytes, self.current_baud, &mut self.rx_queue);
        }
        Ok(())
    }

    fn read_timeout(&mut self, buf: &mut [u8], _timeout: Duration) -> io::Result<usize> {
        let mut count = 0;
        while count < buf.len() {
            if let Some(b) = self.rx_queue.pop_front() {
                buf[count] = b;
                count += 1;
            } else {
                break;
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_motor_iap_flash_over_mock_serial() {
        // 2500-byte payload spanning 3 sectors (1024 + 1024 + 452)
        let payload = vec![0x55u8; 2500];
        let mut mock = MockSerialPort::new_motor_bootloader_responder(payload.len());
        let mut events = Vec::new();

        execute_motor_iap_flash(&mut mock, &payload, 0x67, 0x196E, |ev| {
            events.push(ev);
        })
        .expect("IAP flash should succeed over simulated serial");

        // Verify baud-rate switching sequence: 115200 (probe) -> 9600 (wakeup) -> 115200 (flash)
        assert_eq!(mock.baud_history, vec![115_200, 9_600, 115_200]);
        assert!(events.contains(&MotorFlashProgress::SendingWakeup9600));
        assert!(events.contains(&MotorFlashProgress::BootloaderIdentified));
        assert!(events.contains(&MotorFlashProgress::HeaderAccepted));
        assert!(events.contains(&MotorFlashProgress::VerifiedComplete));

        let sector_acks: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, MotorFlashProgress::SectorAcked { .. }))
            .collect();
        assert_eq!(sector_acks.len(), 3);
    }

    #[test]
    fn test_live_display_isp_flash_over_mock_serial() {
        let bin = vec![0xAAu8; 600]; // 3 packets (256 + 256 + 88)
        let mut mock = MockSerialPort::new_display_isp_responder();
        let mut events = Vec::new();

        execute_display_isp_flash(&mut mock, 0x0100_1000, &bin, |ev| {
            events.push(ev);
        })
        .expect("Display ISP flash should succeed over simulated serial");

        assert!(events.contains(&DisplayFlashProgress::HandshakeComplete));
        assert!(events.contains(&DisplayFlashProgress::ChipIdentified));
        assert!(events.contains(&DisplayFlashProgress::FlashErased));
        assert!(events.contains(&DisplayFlashProgress::UpdateFinalized));

        let pkt_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, DisplayFlashProgress::PacketProgrammed { .. }))
            .collect();
        assert_eq!(pkt_events.len(), 3);
    }

    #[test]
    fn test_telemetry_stream_decoder_with_noise_and_multiple_frames() {
        let mut decoder = TelemetryStreamDecoder::new();

        let f1 = TelemetryFrame {
            battery_voltage_deci_v: 520,
            battery_soc_percent: 90,
            phase_current_deci_a: 120,
            wheel_speed_deci_kph: 250,
            cadence_rpm: 75,
            torque_sensor_mv: 1400,
            throttle_mv: 0,
            controller_temp_c: 42,
            faults: FaultFlags::NONE,
        };
        let f2 = TelemetryFrame {
            battery_voltage_deci_v: 485,
            battery_soc_percent: 60,
            phase_current_deci_a: 280,
            wheel_speed_deci_kph: 340,
            cadence_rpm: 92,
            torque_sensor_mv: 2100,
            throttle_mv: 0,
            controller_temp_c: 68,
            faults: FaultFlags(FaultFlags::CONTROLLER_OVER_TEMP),
        };

        let mut w1 = [0u8; 18];
        let mut w2 = [0u8; 18];
        f1.encode_encrypted_wire(&mut w1);
        f2.encode_encrypted_wire(&mut w2);

        // Prepend random wire noise before w1, and split w2 across two pushes
        let mut chunk1 = vec![0xFF, 0x00, 0x5A ^ UART_STREAM_XOR_KEY, 0x99];
        chunk1.extend_from_slice(&w1);
        chunk1.extend_from_slice(&w2[..7]);

        let ev1 = decoder.push_wire_bytes(&chunk1);
        assert_eq!(ev1, vec![MonitorEvent::Telemetry(f1)]);

        let ev2 = decoder.push_wire_bytes(&w2[7..]);
        assert_eq!(ev2, vec![MonitorEvent::Telemetry(f2)]);
    }
}
