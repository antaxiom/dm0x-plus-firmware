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
    build_header_packet, build_wakeup_packet, parse_display_isp_packet, DisplayIspCmd,
    DisplayIspSession, FlasherError, BOOTLOADER_IDENT_REPLY, BOOTLOADER_PROBE_BYTE,
    DISPLAY_ISP_STX, FLASH_CHUNK_SIZE, FLASH_SECTOR_SIZE, IAP_ACK_BYTE, IAP_BAUD_115200,
    IAP_BAUD_38400, IAP_EOT_BYTE, IAP_WAKEUP_BAUD,
};
use crate::protocol::{TelemetryFrame, FRAME_SOF, UART_STREAM_XOR_KEY};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, FromRawHandle, RawHandle};

#[cfg(windows)]
#[repr(C)]
struct DCB {
    pub DCBlength: u32,
    pub BaudRate: u32,
    pub Flags: u32,
    pub wReserved: u16,
    pub XonLim: u16,
    pub XoffLim: u16,
    pub ByteSize: u8,
    pub Parity: u8,
    pub StopBits: u8,
    pub XonChar: u8,
    pub XoffChar: u8,
    pub ErrorChar: u8,
    pub EofChar: u8,
    pub EvtChar: u8,
    pub wReserved1: u16,
}

#[cfg(windows)]
#[repr(C)]
struct COMMTIMEOUTS {
    pub ReadIntervalTimeout: u32,
    pub ReadTotalTimeoutMultiplier: u32,
    pub ReadTotalTimeoutConstant: u32,
    pub WriteTotalTimeoutMultiplier: u32,
    pub WriteTotalTimeoutConstant: u32,
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn QueryDosDeviceW(lpDeviceName: *const u16, lpTargetPath: *mut u16, ucchMax: u32) -> u32;
    fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *mut std::ffi::c_void,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    fn GetCommState(hFile: *mut std::ffi::c_void, lpDCB: *mut DCB) -> i32;
    fn SetCommState(hFile: *mut std::ffi::c_void, lpDCB: *const DCB) -> i32;
    fn SetCommTimeouts(hFile: *mut std::ffi::c_void, lpCommTimeouts: *const COMMTIMEOUTS) -> i32;
    fn EscapeCommFunction(hFile: *mut std::ffi::c_void, dwFunc: u32) -> i32;
    fn PurgeComm(hFile: *mut std::ffi::c_void, dwFlags: u32) -> i32;
}

#[cfg(windows)]
#[link(name = "advapi32")]
extern "system" {
    fn RegOpenKeyExW(
        hKey: *mut std::ffi::c_void,
        lpSubKey: *const u16,
        ulOptions: u32,
        samDesired: u32,
        phkResult: *mut *mut std::ffi::c_void,
    ) -> i32;
    fn RegEnumValueW(
        hKey: *mut std::ffi::c_void,
        dwIndex: u32,
        lpValueName: *mut u16,
        lpcchValueName: *mut u32,
        lpReserved: *mut u32,
        lpType: *mut u32,
        lpData: *mut u8,
        lpcbData: *mut u32,
    ) -> i32;
    fn RegCloseKey(hKey: *mut std::ffi::c_void) -> i32;
}

/// Transport abstraction for bidirectional serial / USB-CDC communication.
pub trait SerialTransport {
    /// Configures the serial line speed in bits per second (e.g. `9600` or `115200`) in 8N1 raw mode.
    fn set_baud_rate(&mut self, baud: u32) -> io::Result<()>;

    /// Flushes any pending input / receive queue, discarding stale bytes.
    fn flush_input(&mut self) -> io::Result<()>;

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

#[cfg(unix)]
fn configure_raw_termios(fd: std::os::unix::io::RawFd, baud: u32) -> io::Result<()> {
    unsafe {
        let mut termios = std::mem::zeroed::<libc::termios>();
        if libc::tcgetattr(fd, &mut termios) != 0 {
            return Err(io::Error::last_os_error());
        }

        // Configure raw 8N1 mode (cfmakeraw)
        libc::cfmakeraw(&mut termios);

        // 8 data bits, 1 stop bit, no parity, receiver enabled, ignore modem lines
        // Clear HUPCL to prevent Darwin kernel from dropping DTR on descriptor close
        termios.c_cflag &= !(libc::CSIZE | libc::PARENB | libc::CSTOPB | libc::HUPCL);
        termios.c_cflag |= libc::CS8 | libc::CREAD | libc::CLOCAL;

        // Disable hardware flow control (CRTSCTS = 0x00030000 on macOS/Linux)
        const CRTSCTS: libc::tcflag_t = 0x00030000;
        termios.c_cflag &= !CRTSCTS;

        // Disable software flow control
        termios.c_iflag &= !(libc::IXON | libc::IXOFF | libc::IXANY);

        // Non-blocking polled read (VMIN = 0, VTIME = 0)
        termios.c_cc[libc::VMIN] = 0;
        termios.c_cc[libc::VTIME] = 0;

        let speed = match baud {
            9600 => libc::B9600,
            19200 => libc::B19200,
            38400 => libc::B38400,
            57600 => libc::B57600,
            115200 => libc::B115200,
            #[cfg(target_os = "macos")]
            230400 => libc::B230400,
            _ => libc::B115200,
        };

        if libc::cfsetispeed(&mut termios, speed) != 0 {
            return Err(io::Error::last_os_error());
        }
        if libc::cfsetospeed(&mut termios, speed) != 0 {
            return Err(io::Error::last_os_error());
        }

        if libc::tcsetattr(fd, libc::TCSANOW, &termios) != 0 {
            return Err(io::Error::last_os_error());
        }

        // On macOS, configure custom/precise USB serial baud rate via IOKit IOSSIOSPEED ioctl
        #[cfg(target_os = "macos")]
        {
            const IOSSIOSPEED: libc::c_ulong = 0x80085402;
            let mut speed_val = baud as libc::speed_t;
            let _ = libc::ioctl(fd, IOSSIOSPEED, &mut speed_val);
        }

        // Assert DTR and RTS modem control lines (vital for MegaHunt USB CDC-ACM display bootloaders)
        let mut status: libc::c_int = 0;
        if libc::ioctl(fd, libc::TIOCMGET, &mut status) == 0 {
            status |= libc::TIOCM_DTR | libc::TIOCM_RTS;
            let _ = libc::ioctl(fd, libc::TIOCMSET, &status);
        }

        // Flush driver FIFO queues
        libc::tcflush(fd, libc::TCIOFLUSH);
    }
    Ok(())
}

#[cfg(windows)]
fn configure_windows_comm(handle: RawHandle, baud: u32) -> io::Result<()> {
    unsafe {
        let mut dcb = std::mem::zeroed::<DCB>();
        dcb.DCBlength = std::mem::size_of::<DCB>() as u32;
        if GetCommState(handle as _, &mut dcb) == 0 {
            return Err(io::Error::last_os_error());
        }

        dcb.BaudRate = baud;
        dcb.ByteSize = 8;
        dcb.Parity = 0; // NOPARITY
        dcb.StopBits = 0; // ONESTOPBIT
                          // Flags: fBinary = 1 (bit 0), fDtrControl = 1 (DTR_CONTROL_ENABLE, bits 4..5), fRtsControl = 1 (RTS_CONTROL_ENABLE, bits 12..13)
                          // 1 | 16 | 4096 = 0x1011
        dcb.Flags = 0x1011;

        if SetCommState(handle as _, &dcb) == 0 {
            return Err(io::Error::last_os_error());
        }

        // Configure non-blocking read timeouts: ReadIntervalTimeout = MAXDWORD
        // causes ReadFile to return immediately with whatever bytes are in the input buffer.
        let mut timeouts = std::mem::zeroed::<COMMTIMEOUTS>();
        timeouts.ReadIntervalTimeout = 0xFFFFFFFF;
        timeouts.ReadTotalTimeoutMultiplier = 0;
        timeouts.ReadTotalTimeoutConstant = 0;
        timeouts.WriteTotalTimeoutMultiplier = 0;
        timeouts.WriteTotalTimeoutConstant = 5000;

        if SetCommTimeouts(handle as _, &timeouts) == 0 {
            return Err(io::Error::last_os_error());
        }

        // Assert DTR and RTS modem control lines
        const SETDTR: u32 = 5;
        const SETRTS: u32 = 3;
        EscapeCommFunction(handle as _, SETDTR);
        EscapeCommFunction(handle as _, SETRTS);

        // Purge driver FIFO queues
        const PURGE_TXABORT: u32 = 0x0001;
        const PURGE_RXABORT: u32 = 0x0002;
        const PURGE_TXCLEAR: u32 = 0x0004;
        const PURGE_RXCLEAR: u32 = 0x0008;
        PurgeComm(
            handle as _,
            PURGE_TXABORT | PURGE_RXABORT | PURGE_TXCLEAR | PURGE_RXCLEAR,
        );
    }
    Ok(())
}

/// Zero-dependency native serial port driver for macOS, Linux, and Windows.
pub struct PosixSerialPort {
    path: String,
    file: File,
    baud_rate: u32,
}

pub type NativeSerialPort = PosixSerialPort;

impl PosixSerialPort {
    /// Opens a serial device node (e.g. `/dev/cu.usbserial-110`, `/dev/ttyUSB0`, or `COM3`) in non-blocking
    /// raw 8N1 mode and configures the initial baud rate.
    /// On macOS, automatically converts `/dev/tty.*` to `/dev/cu.*` to avoid DCD carrier hangs.
    /// On Windows, automatically ensures `\\.\` prefix for ports COM1..COM255.
    pub fn open(path: &str, initial_baud: u32) -> io::Result<Self> {
        let trimmed = path.trim();
        let canonical_path = if cfg!(target_os = "macos") && trimmed.starts_with("/dev/tty.") {
            trimmed.replace("/dev/tty.", "/dev/cu.")
        } else if cfg!(windows) {
            let upper = trimmed.to_ascii_uppercase();
            if !trimmed.starts_with(r"\\.\") && upper.starts_with("COM") {
                format!(r"\\.\{}", trimmed)
            } else {
                trimmed.to_string()
            }
        } else {
            trimmed.to_string()
        };

        #[cfg(unix)]
        let file = {
            use std::os::unix::fs::OpenOptionsExt;
            OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(libc::O_NONBLOCK | libc::O_NOCTTY)
                .open(&canonical_path)?
        };

        #[cfg(windows)]
        let file = {
            let wide_path: Vec<u16> = canonical_path
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            const GENERIC_READ: u32 = 0x80000000;
            const GENERIC_WRITE: u32 = 0x40000000;
            const OPEN_EXISTING: u32 = 3;
            const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
            const INVALID_HANDLE_VALUE: *mut std::ffi::c_void = -1isize as *mut std::ffi::c_void;

            let handle = unsafe {
                CreateFileW(
                    wide_path.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    0, // Exclusive access (standard for serial ports on Windows)
                    std::ptr::null_mut(),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    std::ptr::null_mut(),
                )
            };

            if handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }

            unsafe { std::fs::File::from_raw_handle(handle as RawHandle) }
        };

        #[cfg(not(any(unix, windows)))]
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&canonical_path)?;

        let mut port = Self {
            path: trimmed.to_string(),
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

/// Helper to verify if a string is a valid Windows COM port name (e.g. `COM1`, `COM3`, `\\.\COM10`).
pub fn is_com_port_name(s: &str) -> bool {
    let s = s.strip_prefix(r"\\.\").unwrap_or(s);
    let upper = s.to_ascii_uppercase();
    if upper.starts_with("COM") && upper.len() > 3 {
        upper[3..].chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

/// Prioritizes and sorts serial port paths so that active physical USB serial dongles
/// (`cu.usbserial-*`, `cu.wchusbserial*`, `cu.usbmodem*`, `ttyUSB*`, `ttyACM*`, Windows `COM*`) appear at index 0.
pub fn sort_serial_ports(ports: &mut [String]) {
    ports.sort_by(|a, b| {
        let com_num = |s: &str| -> Option<u32> {
            let s = s.strip_prefix(r"\\.\").unwrap_or(s);
            let upper = s.to_ascii_uppercase();
            if upper.starts_with("COM")
                && upper.len() > 3
                && upper[3..].chars().all(|c| c.is_ascii_digit())
            {
                upper[3..].parse::<u32>().ok()
            } else {
                None
            }
        };

        if let (Some(na), Some(nb)) = (com_num(a), com_num(b)) {
            return na.cmp(&nb);
        }

        let rank = |s: &str| -> u8 {
            let lower = s.to_ascii_lowercase();
            if lower.contains("cu.usbserial") || lower.contains("ttyusb") || is_com_port_name(s) {
                0
            } else if lower.contains("wch") {
                1
            } else if lower.contains("usbmodem") || lower.contains("ttyacm") {
                2
            } else if lower.contains("slab") || lower.contains("cp210") || lower.contains("ftdi") {
                3
            } else {
                10
            }
        };
        rank(a).cmp(&rank(b)).then_with(|| a.cmp(b))
    });
}

#[cfg(windows)]
fn list_windows_serial_ports(ports: &mut Vec<String>) {
    // 1. Check Windows Registry: HKEY_LOCAL_MACHINE\HARDWARE\DEVICEMAP\SERIALCOMM
    // Windows populates this key for every active serial device (FTDI, CH340, CP210x, CDC).
    list_ports_from_registry(ports);

    // 2. Query all MS-DOS devices using QueryDosDeviceW(NULL, ...)
    let mut buffer = vec![0u16; 32768];
    let ret =
        unsafe { QueryDosDeviceW(std::ptr::null(), buffer.as_mut_ptr(), buffer.len() as u32) };
    if ret > 0 {
        let mut start = 0;
        for i in 0..ret as usize {
            if buffer[i] == 0 {
                if i > start {
                    if let Ok(name) = String::from_utf16(&buffer[start..i]) {
                        let clean = name.trim().to_ascii_uppercase();
                        if is_com_port_name(&clean)
                            && !ports.iter().any(|p| p.eq_ignore_ascii_case(&clean))
                        {
                            ports.push(clean);
                        }
                    }
                }
                start = i + 1;
            }
        }
    }

    // 3. Fallback direct probe COM1..COM64 in case QueryDosDeviceW(NULL) was truncated
    for i in 1..=64 {
        let com = format!("COM{}", i);
        if !ports.iter().any(|p| p.eq_ignore_ascii_case(&com)) {
            let wide: Vec<u16> = com.encode_utf16().chain(std::iter::once(0)).collect();
            let mut target = [0u16; 256];
            let ret =
                unsafe { QueryDosDeviceW(wide.as_ptr(), target.as_mut_ptr(), target.len() as u32) };
            if ret > 0 {
                ports.push(com);
            }
        }
    }
}

#[cfg(windows)]
fn list_ports_from_registry(ports: &mut Vec<String>) {
    const HKEY_LOCAL_MACHINE: *mut std::ffi::c_void = 0x80000002 as usize as *mut std::ffi::c_void;
    const KEY_READ: u32 = 0x20019;
    const ERROR_SUCCESS: i32 = 0;
    const REG_SZ: u32 = 1;

    let subkey: Vec<u16> = "HARDWARE\\DEVICEMAP\\SERIALCOMM\0".encode_utf16().collect();
    let mut hkey: *mut std::ffi::c_void = std::ptr::null_mut();

    unsafe {
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_READ, &mut hkey)
            == ERROR_SUCCESS
        {
            let mut index = 0u32;
            let mut value_name = [0u16; 256];
            let mut data = [0u8; 512];

            loop {
                let mut value_name_len = value_name.len() as u32;
                let mut data_len = data.len() as u32;
                let mut val_type = 0u32;

                let status = RegEnumValueW(
                    hkey,
                    index,
                    value_name.as_mut_ptr(),
                    &mut value_name_len,
                    std::ptr::null_mut(),
                    &mut val_type,
                    data.as_mut_ptr(),
                    &mut data_len,
                );

                if status != ERROR_SUCCESS {
                    break;
                }

                if val_type == REG_SZ && data_len >= 2 {
                    let u16_slice = std::slice::from_raw_parts(
                        data.as_ptr() as *const u16,
                        (data_len / 2) as usize,
                    );
                    let end = u16_slice
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(u16_slice.len());
                    if let Ok(port_str) = String::from_utf16(&u16_slice[..end]) {
                        let clean = port_str.trim().to_ascii_uppercase();
                        if is_com_port_name(&clean)
                            && !ports.iter().any(|p| p.eq_ignore_ascii_case(&clean))
                        {
                            ports.push(clean);
                        }
                    }
                }
                index += 1;
            }
            RegCloseKey(hkey);
        }
    }
}

/// Scans the system for connected USB serial ports (macOS `/dev/cu.usb*`, Linux `/dev/ttyUSB*`, Windows `COM*`, etc.).
pub fn list_serial_ports() -> Vec<String> {
    let mut ports = Vec::new();
    #[cfg(unix)]
    {
        if let Ok(entries) = std::fs::read_dir("/dev") {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    let is_match = if cfg!(target_os = "macos") {
                        let lower = name.to_ascii_lowercase();
                        (name.starts_with("cu.usb")
                            || name.starts_with("cu.wch")
                            || name.starts_with("cu.SLAB")
                            || name.starts_with("cu.CP210")
                            || name.starts_with("cu.FTDI")
                            || name.starts_with("cu.PL2303")
                            || lower.contains("usbserial")
                            || lower.contains("usbmodem"))
                            && !lower.contains("bluetooth")
                            && !lower.contains("debug")
                            && !lower.contains("wlan")
                            && !lower.contains("apple")
                            && !lower.contains("airpods")
                            && !lower.contains("maker")
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
    #[cfg(windows)]
    {
        list_windows_serial_ports(&mut ports);
    }
    sort_serial_ports(&mut ports);
    ports
}

impl SerialTransport for PosixSerialPort {
    fn set_baud_rate(&mut self, baud: u32) -> io::Result<()> {
        self.baud_rate = baud;
        #[cfg(unix)]
        {
            configure_raw_termios(self.file.as_raw_fd(), baud)?;
        }
        #[cfg(windows)]
        {
            configure_windows_comm(self.file.as_raw_handle(), baud)?;
        }
        Ok(())
    }

    fn flush_input(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            unsafe {
                libc::tcflush(self.file.as_raw_fd(), libc::TCIFLUSH);
            }
        }
        #[cfg(windows)]
        {
            unsafe {
                const PURGE_RXCLEAR: u32 = 0x0008;
                const PURGE_RXABORT: u32 = 0x0002;
                PurgeComm(
                    self.file.as_raw_handle() as _,
                    PURGE_RXCLEAR | PURGE_RXABORT,
                );
            }
        }
        // Drain any pending user-space bytes in the pipeline
        let mut drain = [0u8; 128];
        while let Ok(n) = self.read_timeout(&mut drain, Duration::from_millis(5)) {
            if n == 0 {
                break;
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
    uart_mask: u8,
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

    // Step 1: Probe bootloader at 38400 Baud (official ToSeven factory bootloader speed)
    on_progress(MotorFlashProgress::ProbingBootloader);
    let mut active_baud_idx = 0u8; // 0 = 38400 Baud (official factory default)
    let mut connected = false;

    // Quick probe at 38400 Baud in case MCU is already in IAP mode
    transport
        .set_baud_rate(IAP_BAUD_38400)
        .map_err(|_| FlasherError::Timeout)?;
    let _ = transport.flush_input();
    thread::sleep(Duration::from_millis(150));

    // Send up to 3 probe bytes at 38400 Baud
    for _ in 0..3 {
        let _ = transport.flush_input();
        transport
            .write_all(&[BOOTLOADER_PROBE_BYTE])
            .map_err(|_| FlasherError::Timeout)?;
        if wait_for_ident(transport, Duration::from_millis(150)) {
            connected = true;
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    // Step 2: If application firmware is running, command reboot into IAP mode at 9600 Baud
    if !connected {
        on_progress(MotorFlashProgress::SendingWakeup9600);
        transport
            .set_baud_rate(IAP_WAKEUP_BAUD)
            .map_err(|_| FlasherError::Timeout)?;
        // Allow UART hardware adapter to settle at 9600 Baud
        thread::sleep(Duration::from_millis(200));
        let _ = transport.flush_input();

        let wakeup = build_wakeup_packet(program_size, payload_sum, uart_mask, customer_num);
        // Transmit 3 bursts of the 21-byte wakeup frame spaced by 40 ms to ensure reliable
        // reception even if running application firmware is streaming periodic telemetry
        for _ in 0..3 {
            transport
                .write_all(&wakeup)
                .map_err(|_| FlasherError::Timeout)?;
            thread::sleep(Duration::from_millis(40));
        }
        // Allow UART hardware buffer to drain
        thread::sleep(Duration::from_millis(300));

        // Switch to official IAP bootloader baud rate: 38400 Baud (exact parity with official TO7 tool)
        transport
            .set_baud_rate(IAP_BAUD_38400)
            .map_err(|_| FlasherError::Timeout)?;
        // Allow 1200 ms for MCU software reset into system bootloader (exact parity with official Windows tool)
        thread::sleep(Duration::from_millis(1200));
        let _ = transport.flush_input();

        // Poll with 0x02 probe across official retry window (25 attempts x 150 ms = ~3.75s)
        for _ in 0..25 {
            transport
                .write_all(&[BOOTLOADER_PROBE_BYTE])
                .map_err(|_| FlasherError::Timeout)?;
            if wait_for_ident(transport, Duration::from_millis(150)) {
                connected = true;
                active_baud_idx = 0u8; // 38400 Baud
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        // Secondary fallback: probe 115200 Baud if 38400 did not respond
        if !connected {
            let _ = transport.set_baud_rate(IAP_BAUD_115200);
            thread::sleep(Duration::from_millis(200));
            let _ = transport.flush_input();
            for _ in 0..5 {
                transport
                    .write_all(&[BOOTLOADER_PROBE_BYTE])
                    .map_err(|_| FlasherError::Timeout)?;
                if wait_for_ident(transport, Duration::from_millis(150)) {
                    connected = true;
                    active_baud_idx = 2u8; // 115200 Baud
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    if !connected {
        return Err(FlasherError::Timeout);
    }
    on_progress(MotorFlashProgress::BootloaderIdentified);

    // Step 3: Send ACK (0x06) + 12-byte Header matching official TO7 IAP tool sequence
    // Send 0x06 preparation byte
    transport
        .write_all(&[IAP_ACK_BYTE])
        .map_err(|_| FlasherError::Timeout)?;
    thread::sleep(Duration::from_millis(10));

    // Send 12-byte Header
    let hdr = build_header_packet(program_size, payload_sum, uart_mask, active_baud_idx);
    transport
        .write_all(&hdr)
        .map_err(|_| FlasherError::Timeout)?;
    thread::sleep(Duration::from_millis(50));

    let ack = transport
        .read_byte_timeout(Duration::from_secs(2))
        .map_err(|_| FlasherError::Timeout)?;
    if ack != Some(IAP_ACK_BYTE) {
        return Err(FlasherError::UnexpectedResponse(ack.unwrap_or(0)));
    }
    on_progress(MotorFlashProgress::HeaderAccepted);

    // Step 4: Stream payload in 16-byte chunks, awaiting 0x06 ACK every 1024-byte sector.
    // The ToSeven bootloader expects incoming payload stream bytes to be XOR-masked with
    // uart_mask (as specified in Header Frame byte 10). The bootloader de-masks them on-the-fly
    // before writing to Flash and computing the cumulative 32-bit checksum.
    let total_sectors = payload.len().div_ceil(FLASH_SECTOR_SIZE);
    let mut chunk_buf = [0u8; FLASH_CHUNK_SIZE];
    for (sector_idx, sector) in payload.chunks(FLASH_SECTOR_SIZE).enumerate() {
        for chunk in sector.chunks(FLASH_CHUNK_SIZE) {
            for (i, &b) in chunk.iter().enumerate() {
                chunk_buf[i] = b ^ uart_mask;
            }
            transport
                .write_all(&chunk_buf[..chunk.len()])
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

    // Step 5: Wait for final verification EOT (0x04) or completion
    // The Cortex-M4 bootloader confirms every sector (including the final sector) with 0x06 (ACK).
    // Some firmware bootloaders (e.g. V6212C / M4FOC) immediately restart into the application
    // after the final sector ACK, while others send 0x04 (EOT) to signal global checksum match.
    // An explicit 0x15 (NAK) indicates a verification failure.
    let deadline = Instant::now() + Duration::from_millis(2500);
    while Instant::now() < deadline {
        let rem = deadline.saturating_duration_since(Instant::now());
        match transport.read_byte_timeout(rem.min(Duration::from_millis(500))) {
            Ok(Some(IAP_EOT_BYTE)) => {
                // Explicit 0x04 EOT received
                on_progress(MotorFlashProgress::VerifiedComplete);
                return Ok(());
            }
            Ok(Some(IAP_ACK_BYTE)) => {
                // Secondary 0x06 ACK
                continue;
            }
            Ok(Some(0x15)) => {
                // Explicit NAK from bootloader indicates checksum mismatch
                return Err(FlasherError::VerificationFailed);
            }
            Ok(Some(_stray)) => {
                // Line noise or application startup byte (e.g. 0x5A/0x33 telemetry SOF)
                continue;
            }
            Ok(None) => {
                // Timeout on this slice
                break;
            }
            Err(_) => break,
        }
    }

    // All sectors were verified and ACKed by the bootloader with 0x06
    on_progress(MotorFlashProgress::VerifiedComplete);
    Ok(())
}

fn wait_for_ident<T: SerialTransport>(transport: &mut T, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let mut saw_stx = false;
    while Instant::now() < deadline {
        let rem = deadline.saturating_duration_since(Instant::now());
        match transport.read_byte_timeout(rem) {
            Ok(Some(BOOTLOADER_PROBE_BYTE)) => {
                saw_stx = true;
            }
            Ok(Some(b)) if saw_stx && b == BOOTLOADER_IDENT_REPLY[1] => {
                return true;
            }
            Ok(Some(IAP_ACK_BYTE)) => {
                return true;
            }
            Ok(Some(_)) => {
                // Ignore stray telemetry or line noise without resetting saw_stx
            }
            Ok(None) | Err(_) => break,
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
    let _ = transport.flush_input();
    thread::sleep(Duration::from_millis(50));

    let mut session = DisplayIspSession::new(base_addr, bin.len());
    let mut tx_buf = [0u8; 512];

    // 1. Handshake 1 (0x20) - with up to 3 retries in case display ROM is synchronizing
    let n = session.build_handshake1(&mut tx_buf)?;
    let mut hs1_ok = false;
    for _ in 0..3 {
        let _ = transport.flush_input();
        if send_and_expect_isp_ack(transport, &tx_buf[..n], DisplayIspCmd::Handshake1 as u8).is_ok()
        {
            hs1_ok = true;
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    if !hs1_ok {
        return Err(FlasherError::Timeout);
    }

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

    let deadline = Instant::now() + Duration::from_secs(3);

    // 1. Scan for STX byte (0x02), discarding any stray/noise bytes
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err(FlasherError::Timeout);
        }
        let rem = deadline - now;
        match transport.read_byte_timeout(rem) {
            Ok(Some(DISPLAY_ISP_STX)) => break,
            Ok(Some(_stray)) => continue,
            Ok(None) => return Err(FlasherError::Timeout),
            Err(_) => return Err(FlasherError::Timeout),
        }
    }

    // 2. Read CMD, LEN_LO, LEN_HI (3 bytes)
    let mut hdr_tail = [0u8; 3];
    let rem = deadline.saturating_duration_since(Instant::now());
    transport
        .read_exact_timeout(&mut hdr_tail, rem)
        .map_err(|_| FlasherError::Timeout)?;

    let payload_len = u16::from_le_bytes([hdr_tail[1], hdr_tail[2]]) as usize;
    if payload_len > 260 {
        return Err(FlasherError::InvalidPacket);
    }

    // 3. Read payload + 2-byte CRC
    let total_len = 4 + payload_len + 2;
    let mut full = vec![0u8; total_len];
    full[0] = DISPLAY_ISP_STX;
    full[1..4].copy_from_slice(&hdr_tail);

    let rem = deadline.saturating_duration_since(Instant::now());
    transport
        .read_exact_timeout(&mut full[4..], rem)
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
    /// requires the 9600-baud 21-byte wakeup packet before answering the 38400-baud `0x02` probe,
    /// then validates the 12-byte header and ACKs every 1024-byte sector and final EOT.
    pub fn new_motor_bootloader_responder(expected_payload_len: usize) -> Self {
        let mut awakened = false;
        let mut awaiting_header = false;
        let mut header_buf = Vec::new();
        let mut stream_bytes_received = 0usize;
        let mut sector_accum = 0usize;

        let responder = move |written: &[u8], baud: u32, rx: &mut VecDeque<u8>| {
            if baud == IAP_WAKEUP_BAUD {
                if written.len() == 21 {
                    awakened = true;
                }
                return;
            }
            if !awakened {
                return;
            }
            if awaiting_header {
                // Ignore standalone 0x06 prep byte if sent
                if written == [IAP_ACK_BYTE] {
                    return;
                }
                header_buf.extend_from_slice(written);
                if header_buf.len() >= 12 {
                    rx.push_back(IAP_ACK_BYTE);
                    awaiting_header = false;
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
            current_baud: 38_400,
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

    fn flush_input(&mut self) -> io::Result<()> {
        self.rx_queue.clear();
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

        execute_motor_iap_flash(&mut mock, &payload, 0x89, 0x196E, |ev| {
            events.push(ev);
        })
        .expect("IAP flash should succeed over simulated serial");

        // Verify baud-rate switching sequence: 38400 (probe) -> 9600 (wakeup) -> 38400 (flash)
        assert_eq!(mock.baud_history, vec![38_400, 9_600, 38_400]);
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

    #[test]
    fn test_display_isp_recovers_from_noise_and_stray_bytes() {
        let mut mock = MockSerialPort::new();
        // Prepend stray noise bytes before the valid 0x02 ACK frame
        mock.rx_queue.push_back(0xFF);
        mock.rx_queue.push_back(0x00);
        mock.rx_queue.push_back(0x55);
        let mut reply = [0u8; 16];
        let n = crate::flasher::build_display_isp_packet(
            DisplayIspCmd::Handshake1 as u8,
            &[0x00],
            &mut reply,
        )
        .unwrap();
        for &b in &reply[..n] {
            mock.rx_queue.push_back(b);
        }

        let mut tx_buf = [0u8; 64];
        let mut session = DisplayIspSession::new(0x0100_1000, 1024);
        let tx_len = session.build_handshake1(&mut tx_buf).unwrap();

        send_and_expect_isp_ack(
            &mut mock,
            &tx_buf[..tx_len],
            DisplayIspCmd::Handshake1 as u8,
        )
        .expect("Should discard noise bytes and successfully sync on STX");
    }

    #[test]
    fn test_sort_serial_ports_prioritizes_physical_adapters() {
        let mut list = vec![
            "/dev/cu.wchusbserial1420".to_string(),
            "/dev/cu.usbmodem1101".to_string(),
            "/dev/cu.usbserial-A5069RR4".to_string(),
            "/dev/ttyUSB0".to_string(),
        ];
        sort_serial_ports(&mut list);
        // usbserial / ttyUSB (direct motor UART) have highest priority (rank 0)
        assert_eq!(list[0], "/dev/cu.usbserial-A5069RR4");
        assert_eq!(list[1], "/dev/ttyUSB0");
        // wch (CH340 motor adapter) has rank 1
        assert_eq!(list[2], "/dev/cu.wchusbserial1420");
        // usbmodem (display CDC) has rank 2
        assert_eq!(list[3], "/dev/cu.usbmodem1101");
    }

    #[test]
    fn test_sort_windows_com_ports_numerically() {
        let mut list = vec![
            "COM10".to_string(),
            "COM1".to_string(),
            "COM21".to_string(),
            "COM3".to_string(),
            r"\\.\COM2".to_string(),
        ];
        sort_serial_ports(&mut list);
        assert_eq!(list[0], "COM1");
        assert_eq!(list[1], r"\\.\COM2");
        assert_eq!(list[2], "COM3");
        assert_eq!(list[3], "COM10");
        assert_eq!(list[4], "COM21");
    }

    #[test]
    fn test_is_com_port_name() {
        assert!(is_com_port_name("COM1"));
        assert!(is_com_port_name("COM3"));
        assert!(is_com_port_name("COM10"));
        assert!(is_com_port_name("com12"));
        assert!(is_com_port_name(r"\\.\COM4"));
        assert!(!is_com_port_name("COM"));
        assert!(!is_com_port_name("COMX"));
        assert!(!is_com_port_name("/dev/ttyUSB0"));
    }
}
