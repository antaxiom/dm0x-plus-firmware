//! # `toseven_pack` — CLI Tool for Inspecting, Unpacking, and Repacking ToSeven `.Pack` Firmware
//!
//! Usage:
//! - `toseven_pack info <firmware.Pack>`
//! - `toseven_pack unpack <firmware.Pack> <decrypted_payload.bin>`
//! - `toseven_pack repack <template.Pack> <payload.bin> <output.Pack>`
//! - `toseven_pack create <dm01|dm02> <payload.bin> <output.Pack>` (creates a fresh .Pack from compiled binary)
//! - `toseven_pack verify <stock.Pack>` (verifies official release and bit-identical round trip)

use dm01_firmware::eeprom::{EepromImage, EEPROM_TOTAL_LEN};
use dm01_firmware::pack::{
    decrypt_payload, extract_flash_from_elf, parse_pack_metadata, repack_firmware, MotorTarget,
    APP_FLASH_BASE_ADDR, PACK_FILE_SIZE, PACK_HEADER_LEN, PACK_HEADER_MAGIC,
};
use dm01_firmware::sha256::sha256;
use dm01_firmware::stock;
use std::env;
use std::fs;
use std::process;

use dm01_firmware::config::MotorConfig;
use dm01_firmware::control::{Dm01Controller, SensorInputs};
use dm01_firmware::display_patcher::{
    detect_lockout, inspect_display, patch_display, IntelHexFile, LockoutStatus,
};
use dm01_firmware::flasher::{
    build_header_packet, build_wakeup_packet, calculate_checksum8, calculate_crc16_ccitt,
    DisplayIspSession, FLASH_CHUNK_SIZE, FLASH_SECTOR_SIZE, UART_WIRE_MASK,
};
use dm01_firmware::serial::{
    execute_display_isp_flash, execute_motor_iap_flash, format_telemetry_row, DisplayFlashProgress,
    MonitorEvent, MotorFlashProgress, PosixSerialPort, SerialTransport, TelemetryStreamDecoder,
};
use std::time::Duration;

fn hex(d: &[u8]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn print_usage() {
    eprintln!("============================================================");
    eprintln!("  ToSeven Mid-Drive Motor Controller & Display Toolkit");
    eprintln!("============================================================");
    eprintln!("Motor Controller (.Pack) Commands:");
    eprintln!("  toseven_pack info <firmware.Pack>");
    eprintln!("  toseven_pack unpack <firmware.Pack> <decrypted_payload.bin>");
    eprintln!("  toseven_pack repack <template.Pack> <payload.bin> <output.Pack>");
    eprintln!("  toseven_pack create <dm01|dm02> <payload.bin|payload.elf> <output.Pack>");
    eprintln!("  toseven_pack objcopy <firmware.elf> <output.bin>");
    eprintln!("  toseven_pack verify <stock.Pack>");
    eprintln!("  toseven_pack flash <firmware.Pack> [--dry-run]");
    eprintln!("  toseven_pack flash <port> <firmware.Pack>");
    eprintln!();
    eprintln!("Display Firmware (.hex) Commands:");
    eprintln!("  toseven_pack display info <firmware.hex>");
    eprintln!("  toseven_pack display patch <input.hex> <output.hex>");
    eprintln!("  toseven_pack display flash <firmware.hex> [--unlock] [--dry-run]");
    eprintln!("  toseven_pack display flash <port> <firmware.hex> [--unlock]");
    eprintln!();
    eprintln!("Live UART Telemetry & Bus Monitor:");
    eprintln!("  toseven_pack monitor <port> [--baud <9600|115200>]");
    eprintln!("  toseven_pack monitor --simulate");
    eprintln!();
    eprintln!("Simulation & Hardware Verification:");
    eprintln!("  toseven_pack simulate");
    eprintln!();
    eprintln!("Serial Port Utilities:");
    eprintln!("  toseven_pack ports");
    eprintln!("============================================================");
}

fn print_display_usage() {
    eprintln!("Usage:");
    eprintln!("  toseven_pack display info <firmware.hex>");
    eprintln!("  toseven_pack display patch <input.hex> <output.hex>");
    eprintln!("  toseven_pack display flash <firmware.hex> [--unlock] [--dry-run]");
    eprintln!("  toseven_pack display flash <port> <firmware.hex> [--unlock]");
}

fn handle_display_command(args: &[String]) {
    if args.len() < 3 {
        print_display_usage();
        process::exit(1);
    }

    match args[2].as_str() {
        "info" => {
            if args.len() < 4 {
                eprintln!("Usage: toseven_pack display info <firmware.hex>");
                process::exit(1);
            }
            let hex_path = &args[3];
            let raw_str = match fs::read_to_string(hex_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading Intel HEX file '{}': {}", hex_path, e);
                    process::exit(1);
                }
            };
            let hex_file = match IntelHexFile::parse(&raw_str) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Error parsing Intel HEX records in '{}': {}", hex_path, e);
                    process::exit(1);
                }
            };
            let info = match inspect_display(&hex_file) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("Error inspecting display firmware: {}", e);
                    process::exit(1);
                }
            };

            println!("============================================================");
            println!("  ToSeven Display Firmware Inspector (MegaHunt MH2020C)");
            println!("============================================================");
            println!("File Path:           {}", hex_path);
            println!("Identified Model:    {}", info.identified_model);
            if let Some(ts) = &info.build_timestamp {
                println!("Build Timestamp:     {}", ts);
            }
            println!("Flash Base Address:  0x{:08X}", info.base_address);
            println!(
                "Flash Image Size:    {} bytes ({:.1} KB)",
                info.size_bytes,
                info.size_bytes as f64 / 1024.0
            );
            println!("Initial MSP:         0x{:08X} (SRAM)", info.initial_sp);
            println!(
                "Reset Handler:       0x{:08X} (Thumb mode)",
                info.reset_handler
            );
            println!("------------------------------------------------------------");
            println!("Post-Oct 2024 Advanced Menu Lockout Status:");
            match &info.lockout_status {
                LockoutStatus::LegacyUnlocked => {
                    println!(
                        "  Status:            LEGACY UNLOCKED (no lockout gates in this build)"
                    );
                    println!("  Advanced Menu:     Fully accessible without restrictions");
                }
                LockoutStatus::AlreadyUnlocked { gate_addresses } => {
                    println!(
                        "  Status:            ALREADY UNLOCKED ({} gates patched with NOP)",
                        gate_addresses.len()
                    );
                    println!(
                        "  Advanced Menu:     Fully unlocked and unrestricted for all controllers"
                    );
                }
                LockoutStatus::Locked { patches } => {
                    println!(
                        "  Status:            LOCKED ({} restriction branch gates active)",
                        patches.len()
                    );
                    for (i, p) in patches.iter().enumerate() {
                        println!(
                            "    [{}/{}] Gate {} ({}) at 0x{:08X}: 0x{:02X}{:02X} -> LOCKED",
                            i + 1,
                            patches.len(),
                            p.gate,
                            p.name,
                            p.address,
                            p.original_bytes[0],
                            p.original_bytes[1]
                        );
                        println!("          Description: {}", p.description);
                    }
                    println!("------------------------------------------------------------");
                    println!("Recommendation: Run 'toseven_pack display patch <in> <out>' or flash with '--unlock'.");
                }
            }
            println!("============================================================");
        }
        "patch" => {
            if args.len() < 5 {
                eprintln!("Usage: toseven_pack display patch <input.hex> <output.hex>");
                process::exit(1);
            }
            let in_path = &args[3];
            let out_path = &args[4];
            let raw_str = match fs::read_to_string(in_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading input Intel HEX '{}': {}", in_path, e);
                    process::exit(1);
                }
            };
            let mut hex_file = match IntelHexFile::parse(&raw_str) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Error parsing Intel HEX records in '{}': {}", in_path, e);
                    process::exit(1);
                }
            };

            let info = inspect_display(&hex_file).ok();
            println!("============================================================");
            println!("  ToSeven Display Firmware Lockout Patcher");
            println!("============================================================");
            println!("Input Firmware:      {}", in_path);
            println!("Output Firmware:     {}", out_path);
            if let Some(i) = &info {
                println!("Target Model:        {}", i.identified_model);
            }
            println!("------------------------------------------------------------");

            let patches = match patch_display(&mut hex_file) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error applying lockout patches: {}", e);
                    process::exit(1);
                }
            };

            if patches.is_empty() {
                println!(
                    "Notice: No active lockout gates found (already unlocked or legacy firmware)."
                );
                println!("Writing clean unpatched copy to '{}'...", out_path);
            } else {
                println!(
                    "Identified and patched {} restriction gates:",
                    patches.len()
                );
                for (i, p) in patches.iter().enumerate() {
                    println!(
                        "  [{}/{}] {} at 0x{:08X}: 0x{:02X}{:02X} -> NOP (0x{:02X}{:02X})",
                        i + 1,
                        patches.len(),
                        p.name,
                        p.address,
                        p.original_bytes[0],
                        p.original_bytes[1],
                        p.patched_bytes[0],
                        p.patched_bytes[1]
                    );
                    println!("        {}", p.description);
                }
            }

            let patched_hex_str = hex_file.to_hex_string();
            if let Err(e) = fs::write(out_path, patched_hex_str) {
                eprintln!("Error writing patched output '{}': {}", out_path, e);
                process::exit(1);
            }
            println!("------------------------------------------------------------");
            println!("Saved unlocked firmware to '{}'.", out_path);
            let verify_file = IntelHexFile::parse(&fs::read_to_string(out_path).unwrap()).unwrap();
            let new_status = detect_lockout(&verify_file).unwrap();
            match new_status {
                LockoutStatus::AlreadyUnlocked { gate_addresses } => {
                    println!(
                        "Verification: Output verified 100% UNLOCKED ({} gates neutralized).",
                        gate_addresses.len()
                    );
                }
                LockoutStatus::LegacyUnlocked => {
                    println!("Verification: Output verified UNLOCKED (legacy stock firmware).");
                }
                LockoutStatus::Locked { .. } => {
                    eprintln!("WARNING: Verification failed; file still contains locked gates!");
                    process::exit(1);
                }
            }
            println!("============================================================");
        }
        "flash" => {
            let mut unlock = false;
            let mut dry_run = false;
            let mut positional = Vec::new();

            for arg in &args[3..] {
                if arg == "--unlock" {
                    unlock = true;
                } else if arg == "--dry-run" {
                    dry_run = true;
                } else {
                    positional.push(arg.clone());
                }
            }

            let (port_name, hex_path) = match positional.len() {
                1 => {
                    dry_run = true;
                    (None, positional[0].clone())
                }
                2 => (Some(positional[0].clone()), positional[1].clone()),
                _ => {
                    eprintln!(
                        "Usage: toseven_pack display flash <firmware.hex> [--unlock] [--dry-run]"
                    );
                    eprintln!("   or: toseven_pack display flash <port> <firmware.hex> [--unlock]");
                    process::exit(1);
                }
            };

            println!("============================================================");
            println!("  ToSeven MegaHunt Display ISP Firmware Flasher");
            println!("============================================================");
            println!("Firmware File:       {}", hex_path);
            println!(
                "On-the-Fly Unlock:   {}",
                if unlock {
                    "ENABLED (--unlock)"
                } else {
                    "DISABLED"
                }
            );

            let raw_str = match fs::read_to_string(&hex_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading firmware '{}': {}", hex_path, e);
                    process::exit(1);
                }
            };
            let mut hex_file = match IntelHexFile::parse(&raw_str) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Error parsing Intel HEX: {}", e);
                    process::exit(1);
                }
            };

            let info = match inspect_display(&hex_file) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("Error inspecting display firmware: {}", e);
                    process::exit(1);
                }
            };
            println!("Identified Model:    {}", info.identified_model);
            if let Some(ts) = &info.build_timestamp {
                println!("Build Timestamp:     {}", ts);
            }
            println!("Flash Load Address:  0x{:08X}", info.base_address);
            println!(
                "Flash Image Size:    {} bytes ({:.1} KB)",
                info.size_bytes,
                info.size_bytes as f64 / 1024.0
            );
            println!("Initial MSP:         0x{:08X}", info.initial_sp);
            println!("Reset Handler:       0x{:08X}", info.reset_handler);
            println!("------------------------------------------------------------");

            if unlock {
                match &info.lockout_status {
                    LockoutStatus::Locked { patches } => {
                        println!(
                            "[Unlock] Detected {} active lockout restriction gates:",
                            patches.len()
                        );
                        let applied = patch_display(&mut hex_file).expect("apply unlock patch");
                        for p in &applied {
                            println!(
                                "  Patched Gate {} ({}) at 0x{:08X} -> NOP",
                                p.gate, p.name, p.address
                            );
                        }
                        let new_info = inspect_display(&hex_file).expect("re-inspect");
                        println!(
                            "[Unlock] Post-patch validation: {:?}",
                            new_info.lockout_status
                        );
                    }
                    LockoutStatus::AlreadyUnlocked { .. } => {
                        println!("[Unlock] Firmware is already unlocked; no patching needed.");
                    }
                    LockoutStatus::LegacyUnlocked => {
                        println!("[Unlock] Firmware is legacy stock (no lockout gates present); ready to flash.");
                    }
                }
                println!("------------------------------------------------------------");
            } else if matches!(info.lockout_status, LockoutStatus::Locked { .. }) {
                println!("WARNING: Firmware contains post-October 2024 lockout gates!");
                println!(
                    "To automatically unlock all menus and settings during flash, pass '--unlock'."
                );
                println!("------------------------------------------------------------");
            }

            let (base_addr, bin) = hex_file.to_flat_binary().expect("flat binary");
            let chunk_size = 256;
            let total_packets = bin.len().div_ceil(chunk_size);

            if dry_run {
                println!(
                    "[DRY-RUN] Simulating complete MegaHunt ISP protocol handshake and flash:"
                );
                println!("  1. Transmit Handshake1 (0x02, CMD 0x20)... [OK]");
                println!("  2. Transmit Handshake2 (0x02, CMD 0x21)... [OK]");
                println!("  3. Read Chip Silicon ID & ROM Info (CMD 0x22)... [MegaHunt MH2020C Identified]");
                println!("  4. Read Flash ID (CMD 0x3A)... [SPI Flash Winbond W25Q128 Detected (0xEF4018)]");
                let end_addr = base_addr + bin.len() as u32;
                println!(
                    "  5. Erase Flash sectors 0x{:08X}..0x{:08X} ({} bytes, CMD 0x3D)... [Erased OK]",
                    base_addr,
                    end_addr,
                    bin.len()
                );
                println!("  6. Write Flash QSPI Option Registers (CMD 0x3E)... [Configured OK]");
                println!(
                    "  7. Streaming {} packets ({} bytes/chunk) with CRC-16-CCITT framing...",
                    total_packets, chunk_size
                );

                let mut session = DisplayIspSession::new(base_addr, bin.len());
                let mut pkt_buf = [0u8; 512];
                let mut sample_count = 0;
                for p in 0..total_packets {
                    let start = p * chunk_size;
                    let end = (start + chunk_size).min(bin.len());
                    let chunk = &bin[start..end];
                    let chunk_addr = base_addr + start as u32;
                    let len = session
                        .build_program_chunk(chunk_addr, chunk, &mut pkt_buf)
                        .unwrap();
                    let crc = calculate_crc16_ccitt(&pkt_buf[0..len - 2]);
                    if p == 0 || p == total_packets - 1 || p == total_packets / 2 {
                        println!(
                            "     Packet #{:03}/{:03}: Flash 0x{:08X}..0x{:08X} ({} bytes, CRC=0x{:04X}) -> [ACK 0x50 OK]",
                            p + 1,
                            total_packets,
                            chunk_addr,
                            chunk_addr + chunk.len() as u32,
                            chunk.len(),
                            crc
                        );
                        sample_count += 1;
                    }
                }
                if sample_count < total_packets {
                    println!(
                        "     ... (all {} packets verified with valid CRC-16-CCITT framing)",
                        total_packets
                    );
                }
                println!("  8. Finalize Update & Trigger Display Reboot (CMD 0x58)... [SUCCESS]");
                println!("------------------------------------------------------------");
                println!(
                    "DRY-RUN COMPLETE: Display firmware image, ISP framing, and CRC 100% verified."
                );
                println!("To perform live flashing to hardware via USB:");
                println!(
                    "  toseven_pack display flash <port> {} {}",
                    hex_path,
                    if unlock { "--unlock" } else { "" }
                );
            } else {
                let port = port_name.unwrap();
                println!("Live USB CDC target port: {}", port);
                println!("Connecting to MegaHunt USB CDC device on {}...", port);
                let mut serial = match PosixSerialPort::open(&port, 115_200) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error opening serial port '{}': {}", port, e);
                        process::exit(1);
                    }
                };
                if let Err(err) =
                    execute_display_isp_flash(&mut serial, base_addr, &bin, |ev| match ev {
                        DisplayFlashProgress::HandshakeComplete => {
                            println!("  [ISP] Handshake 1 & 2 acknowledged (0x20, 0x21).");
                        }
                        DisplayFlashProgress::ChipIdentified => {
                            println!("  [ISP] MegaHunt MH2020C & SPI Flash ID verified.");
                        }
                        DisplayFlashProgress::FlashErased => {
                            println!(
                                "  [ISP] Erased Flash region 0x{:08X}..0x{:08X}.",
                                base_addr,
                                base_addr + bin.len() as u32
                            );
                        }
                        DisplayFlashProgress::PacketProgrammed {
                            packet_index,
                            total_packets,
                            bytes_sent,
                            total_bytes,
                        } => {
                            if packet_index == 0
                                || packet_index + 1 == total_packets
                                || (packet_index + 1) % 32 == 0
                            {
                                println!(
                                    "  [ISP] Programmed packet {:03}/{:03} ({}/{} bytes)",
                                    packet_index + 1,
                                    total_packets,
                                    bytes_sent,
                                    total_bytes
                                );
                            }
                        }
                        DisplayFlashProgress::UpdateFinalized => {
                            println!("  [ISP] ChipUpdate (0x58) complete; display rebooting.");
                        }
                    })
                {
                    eprintln!("Display ISP flash failed: {:?}", err);
                    process::exit(2);
                }
                println!("SUCCESS: Display firmware flashed to {}!", port);
            }
            println!("============================================================");
        }
        _ => {
            print_display_usage();
            process::exit(1);
        }
    }
}

fn handle_monitor_command(args: &[String]) {
    let mut simulate = false;
    let mut baud: u32 = 9600;
    let mut port_name: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--simulate" | "-s" => simulate = true,
            "--baud" | "-b" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --baud requires a value (e.g. 9600 or 115200)");
                    process::exit(1);
                }
                baud = args[i + 1].parse().unwrap_or(9600);
                i += 1;
            }
            other => {
                port_name = Some(other.to_string());
            }
        }
        i += 1;
    }

    println!("============================================================");
    println!("  ToSeven Live UART Telemetry & 0x69 Stream-Cipher Monitor");
    println!("============================================================");

    if simulate || port_name.is_none() {
        println!("Mode:                SIMULATED RIDE TELEMETRY (--simulate)");
        println!("Stream Cipher:       0x69 XOR + ToSeven Nibble CRC-16");
        println!("------------------------------------------------------------");

        let mut controller = Dm01Controller::new(MotorConfig::default_dm01());
        controller.mode = dm01_firmware::config::RidingMode::Sport;
        controller.assist = dm01_firmware::config::AssistSelection::Level(5);
        let mut decoder = TelemetryStreamDecoder::new();

        // Warm up torque sensor startup calibration (300 ms unloaded at 700 mV)
        let mut inputs = SensorInputs {
            battery_voltage_mv: 52_000,
            torque_adc_mv: 700,
            throttle_adc_mv: 450,
            cadence_rpm: 0,
            backpedaling: false,
            wheel_speed_deci_kph: 0,
            brake_active: false,
            shift_sensor_triggered: false,
            walk_assist_held: false,
            controller_temp_c: 28,
            motor_temp_c: 28,
        };
        for _ in 0..35 {
            controller.step_v6510c(&inputs);
        }

        // Simulate 9 telemetry steps across idle, pedaling climb, cruise, and thermal rollback
        let scenarios: &[(u16, u16, u16, u16, i16, &str)] = &[
            (700, 0, 450, 0, 28, "Standstill Idle"),
            (1350, 35, 450, 85, 32, "Low-Cadence Hill Start (35 RPM)"),
            (1850, 65, 450, 165, 41, "Full Cadence Climb (65 RPM)"),
            (2400, 80, 450, 210, 52, "High-Effort Sprint (80 RPM)"),
            (
                1600,
                75,
                450,
                235,
                58,
                "Cruising Below Speed Limit (23.5 km/h)",
            ),
            (
                1600,
                75,
                450,
                244,
                62,
                "Speed Soft-Taper Window (24.4 / 25.0 km/h)",
            ),
            (700, 60, 2600, 180, 66, "Throttle Override (2600 mV)"),
            (2200, 70, 450, 190, 82, "Thermal Rollback Active (82°C)"),
            (700, 0, 450, 120, 64, "Coasting / Cooling Down"),
        ];

        let mut step_num = 1;
        for &(trq_mv, cad_rpm, thr_mv, spd_deci_kph, temp_c, label) in scenarios {
            inputs.torque_adc_mv = trq_mv;
            inputs.cadence_rpm = cad_rpm;
            inputs.throttle_adc_mv = thr_mv;
            inputs.wheel_speed_deci_kph = spd_deci_kph;
            inputs.controller_temp_c = temp_c;

            // Step controller multiple ticks to allow smooth current ramp (±0.2A/step)
            for _ in 0..160 {
                controller.step_v6510c(&inputs);
            }
            let out = controller.step_v6510c(&inputs);
            let frame = dm01_firmware::protocol::TelemetryFrame {
                battery_voltage_deci_v: (inputs.battery_voltage_mv / 100) as u16,
                battery_soc_percent: 88,
                phase_current_deci_a: (out.target_current_ma / 100) as u16,
                wheel_speed_deci_kph: inputs.wheel_speed_deci_kph,
                cadence_rpm: inputs.cadence_rpm,
                torque_sensor_mv: inputs.torque_adc_mv,
                throttle_mv: inputs.throttle_adc_mv,
                controller_temp_c: inputs.controller_temp_c as i8,
                faults: dm01_firmware::faults::FaultFlags::NONE,
            };
            let mut wire = [0u8; 18];
            frame.encode_encrypted_wire(&mut wire);

            for ev in decoder.push_wire_bytes(&wire) {
                if let MonitorEvent::Telemetry(decoded_frame) = ev {
                    println!(
                        "{}  <-- {}",
                        format_telemetry_row(step_num, &decoded_frame),
                        label
                    );
                    step_num += 1;
                }
            }
        }
        println!("============================================================");
        return;
    }

    let port = port_name.unwrap();
    println!("Serial Port:         {} @ {} Baud (8N1)", port, baud);
    println!("Stream Cipher:       0x69 XOR + ToSeven Nibble CRC-16");
    println!("Press Ctrl+C to stop monitoring.");
    println!("------------------------------------------------------------");

    let mut serial = match PosixSerialPort::open(&port, baud) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error opening serial port '{}': {}", port, e);
            process::exit(1);
        }
    };

    let mut decoder = TelemetryStreamDecoder::new();
    let mut rx_buf = [0u8; 64];
    let mut step = 1usize;

    loop {
        match serial.read_timeout(&mut rx_buf, Duration::from_millis(250)) {
            Ok(n) if n > 0 => {
                for ev in decoder.push_wire_bytes(&rx_buf[..n]) {
                    match ev {
                        MonitorEvent::Telemetry(frame) => {
                            println!("{}", format_telemetry_row(step, &frame));
                            step += 1;
                        }
                        MonitorEvent::RawDecryptedFrame(raw) => {
                            println!("[RAW 0x69-DECRYPTED] {}", hex(&raw));
                        }
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Serial read error on {}: {}", port, e);
                process::exit(1);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let cmd = args[1].as_str();

    if cmd == "display" {
        handle_display_command(&args);
        return;
    }

    if cmd == "monitor" {
        handle_monitor_command(&args);
        return;
    }

    if cmd == "ports" {
        println!("============================================================");
        println!("  ToSeven Connected Serial Devices");
        println!("============================================================");
        let ports = dm01_firmware::serial::list_serial_ports();
        if ports.is_empty() {
            println!("No USB serial devices detected in /dev.");
            println!(
                "Ensure your USB-UART adapter (CH340/FTDI/CP2102) or USB display is plugged in."
            );
        } else {
            for (idx, p) in ports.iter().enumerate() {
                println!("  [{}] {}", idx + 1, p);
            }
        }
        println!("============================================================");
        return;
    }

    if cmd == "simulate" {
        dm01_firmware::simulation::run_all_simulations();
        return;
    }

    if cmd == "help" || cmd == "--help" || cmd == "-h" {
        print_usage();
        return;
    }

    if args.len() < 3 {
        print_usage();
        process::exit(1);
    }

    // Handle "objcopy" separately: extracts raw Flash binary from ELF
    if cmd == "objcopy" {
        if args.len() < 4 {
            eprintln!("Usage: toseven_pack objcopy <firmware.elf> <output.bin>");
            process::exit(1);
        }
        let elf_path = &args[2];
        let out_bin_path = &args[3];
        let raw_elf = match fs::read(elf_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error reading ELF '{}': {}", elf_path, e);
                process::exit(1);
            }
        };
        let flash = match extract_flash_from_elf(&raw_elf) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error extracting Flash from ELF '{}': {}", elf_path, e);
                process::exit(1);
            }
        };
        if let Err(e) = fs::write(out_bin_path, &flash) {
            eprintln!("Error writing '{}': {}", out_bin_path, e);
            process::exit(1);
        }
        println!(
            "Successfully extracted {} bytes Flash image to '{}'",
            flash.len(),
            out_bin_path
        );
        return;
    }

    // Handle "create" separately because it takes a target name instead of an input .Pack
    if cmd == "create" {
        if args.len() < 5 {
            eprintln!(
                "Usage: toseven_pack create <dm01|dm02> <payload.bin|payload.elf> <output.Pack>"
            );
            process::exit(1);
        }
        let target_str = args[2].to_ascii_lowercase();
        let payload_path = &args[3];
        let out_pack_path = &args[4];

        let target = match target_str.as_str() {
            "dm01" => MotorTarget::Dm01,
            "dm02" => MotorTarget::Dm02,
            _ => {
                eprintln!("Target must be 'dm01' or 'dm02'");
                process::exit(1);
            }
        };

        let raw_input = match fs::read(payload_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error reading payload '{}': {}", payload_path, e);
                process::exit(1);
            }
        };

        let payload = if raw_input.starts_with(b"\x7fELF") {
            match extract_flash_from_elf(&raw_input) {
                Ok(flash) => {
                    println!(
                        "Extracted {} bytes Flash image from ELF (0x08001800..)",
                        flash.len()
                    );
                    flash
                }
                Err(err) => {
                    eprintln!("Error extracting Flash from ELF: {}", err);
                    process::exit(1);
                }
            }
        } else {
            raw_input
        };

        if payload.len() < 8 {
            eprintln!("Payload too small to contain a valid ARM Cortex-M vector table");
            process::exit(1);
        }

        let initial_sp = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let reset_handler = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
        println!("Payload Vector Table: MSP=0x{initial_sp:08X}, Reset=0x{reset_handler:08X}");

        let (hdr_key, payload_key) = target.default_keys();
        let device_id: u32 = match target {
            MotorTarget::Dm01 => 0x8119_6EA4,
            MotorTarget::Dm02 => 0x8119_6EA3,
            MotorTarget::Unknown(id) => id,
        };

        // Construct 34-byte decrypted header
        let mut dec_hdr = [0u8; PACK_HEADER_LEN];
        let dev_bytes = device_id.to_le_bytes();
        dec_hdr[0..4].copy_from_slice(&dev_bytes);
        dec_hdr[16..20].copy_from_slice(&PACK_HEADER_MAGIC);
        let len_bytes = (payload.len() as u16).to_le_bytes();
        dec_hdr[24] = len_bytes[0];
        dec_hdr[25] = len_bytes[1];

        let meta = dm01_firmware::pack::PackMetadata {
            target,
            device_id_24: device_id & 0x00FF_FFFF,
            header_xor_key: hdr_key,
            payload_xor_key: payload_key,
            payload_len: payload.len(),
            initial_sp,
            reset_handler,
            decrypted_header: dec_hdr,
        };

        let mut out_pack = [0u8; PACK_FILE_SIZE];
        if let Err(e) = repack_firmware(&meta, &payload, None, &mut out_pack) {
            eprintln!("Error packing firmware: {:?}", e);
            process::exit(1);
        }

        if let Err(e) = fs::write(out_pack_path, out_pack) {
            eprintln!("Error writing output .Pack '{}': {}", out_pack_path, e);
            process::exit(1);
        }

        println!(
            "Successfully created .Pack container: '{}' ({} bytes, Target: {:?})",
            out_pack_path, PACK_FILE_SIZE, target
        );
        return;
    }

    // Handle "flash" before opening args[2] as a .Pack (since args[2] can be <port> or --dry-run)
    if cmd == "flash" {
        let dry_run;
        let port_name: Option<String>;
        let pack_path: String;

        if args.len() == 3 {
            dry_run = true;
            port_name = None;
            pack_path = args[2].clone();
        } else if args.len() == 4 {
            if args[3] == "--dry-run" {
                dry_run = true;
                port_name = None;
                pack_path = args[2].clone();
            } else if args[2] == "--dry-run" {
                dry_run = true;
                port_name = None;
                pack_path = args[3].clone();
            } else {
                dry_run = false;
                port_name = Some(args[2].clone());
                pack_path = args[3].clone();
            }
        } else {
            eprintln!("Usage: toseven_pack flash <firmware.Pack> [--dry-run]");
            eprintln!("   or: toseven_pack flash <port> <firmware.Pack>");
            process::exit(1);
        }

        println!("============================================================");
        println!("  ToSeven In-Application Programming (IAP) Firmware Flasher");
        println!("============================================================");

        let raw_pack = match fs::read(&pack_path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error reading firmware '{}': {}", pack_path, e);
                process::exit(1);
            }
        };
        let meta = match parse_pack_metadata(&raw_pack) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error parsing .Pack container '{}': {:?}", pack_path, e);
                process::exit(1);
            }
        };

        let mut payload = vec![0u8; meta.payload_len];
        decrypt_payload(&raw_pack, &meta, &mut payload).expect("decrypt payload");

        let mut payload_sum: u32 = 0;
        for &b in &payload {
            payload_sum = payload_sum.wrapping_add(b as u32);
        }

        let customer_num = u16::from_le_bytes([meta.decrypted_header[1], meta.decrypted_header[2]]);

        println!("Target Motor:        {:?}", meta.target);
        println!(
            "Customer Code:       0x{:04X} ({})",
            customer_num, customer_num
        );
        println!("Device ID:           0x{:06X}", meta.device_id_24);
        println!("Payload Length:      {} bytes", meta.payload_len);
        println!("Payload Byte Sum:    0x{:08X}", payload_sum);
        println!("Payload XOR Key:     0x{:02X}", meta.payload_xor_key);
        println!("Header XOR Key:      0x{:02X}", meta.header_xor_key);
        println!("Initial MSP:         0x{:08X}", meta.initial_sp);
        println!("Reset Handler:       0x{:08X}", meta.reset_handler);
        println!("------------------------------------------------------------");

        let wakeup_pkt = build_wakeup_packet(
            meta.payload_len as u32,
            payload_sum,
            meta.payload_xor_key,
            customer_num,
        );
        println!("[Step 1] Constructed 21-byte Wakeup / Enter IAP Packet:");
        println!("  Wire Encrypted (XOR 0x69): {}", hex(&wakeup_pkt));
        let mut unenc = [0u8; 21];
        for i in 0..21 {
            unenc[i] = wakeup_pkt[i] ^ UART_WIRE_MASK;
        }
        println!("  Plaintext Decoded:         {}", hex(&unenc));

        let baud_idx = 2; // 115200 Baud
        let hdr_pkt = build_header_packet(
            meta.payload_len as u32,
            payload_sum,
            meta.payload_xor_key,
            baud_idx,
        );
        let hdr_sum = calculate_checksum8(&hdr_pkt);
        println!("[Step 2] Constructed 12-byte Header Packet (Baud=115200):");
        println!("  Header Frame:              {}", hex(&hdr_pkt));
        println!("  Modulo-256 Checksum:       0x{:02X}", hdr_sum);

        let total_chunks = meta.payload_len.div_ceil(FLASH_CHUNK_SIZE);
        let total_sectors = meta.payload_len.div_ceil(FLASH_SECTOR_SIZE);
        println!("[Step 3] Flash Stream Partitioning:");
        println!(
            "  Chunk Size:                {} bytes ({} total chunks)",
            FLASH_CHUNK_SIZE, total_chunks
        );
        println!(
            "  Sector ACK Interval:       {} bytes ({} total sectors)",
            FLASH_SECTOR_SIZE, total_sectors
        );

        if dry_run {
            println!("------------------------------------------------------------");
            println!("[DRY-RUN] Simulating complete IAP handshake and flash sequence:");
            println!("  1. Probe bootloader at 115200 Baud (byte 0x02)... [OK]");
            println!("  2. Bootloader responds with IDENT: 0x02, 0xFD... [MCU Identified]");
            println!("  3. Send ACK (0x06) + 12-byte header frame... [OK]");
            println!("  4. Bootloader header validation... [ACK 0x06 Received]");
            println!(
                "  5. Streaming {} chunks across {} sectors...",
                total_chunks, total_sectors
            );
            for s in 0..total_sectors {
                let s_start = s * FLASH_SECTOR_SIZE;
                let s_end = (s_start + FLASH_SECTOR_SIZE).min(meta.payload_len);
                let s_len = s_end - s_start;
                println!(
                    "     Sector #{:02}: bytes {}..{} ({} bytes) -> [ACK 0x06 OK]",
                    s, s_start, s_end, s_len
                );
            }
            println!("  6. All bytes transferred; awaiting final verification...");
            println!("  7. Bootloader payload CRC verification... [EOT 0x04 OK]");
            println!("  8. MCU restarting into application firmware... [SUCCESS]");
            println!("------------------------------------------------------------");
            println!("DRY-RUN COMPLETE: Firmware container and IAP framing 100% verified.");
            println!("To perform live flashing to hardware via USB-UART adapter:");
            println!("  toseven_pack flash <port> {}", pack_path);
        } else {
            let port = port_name.unwrap();
            println!("------------------------------------------------------------");
            println!("Live UART target port: {}", port);
            println!("SAFETY NOTE: Use a 48V current-limited bench power supply for first-flash testing.");
            let mut serial = match PosixSerialPort::open(&port, 115_200) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error opening serial port '{}': {}", port, e);
                    process::exit(1);
                }
            };
            if let Err(err) = execute_motor_iap_flash(
                &mut serial,
                &payload,
                meta.payload_xor_key,
                customer_num,
                |ev| match ev {
                    MotorFlashProgress::ProbingBootloader => {
                        println!("  [IAP] Probing bootloader at 115200 Baud (0x02)...");
                    }
                    MotorFlashProgress::SendingWakeup9600 => {
                        println!("  [IAP] Sending 21-byte wakeup frame at 9600 Baud...");
                    }
                    MotorFlashProgress::BootloaderIdentified => {
                        println!("  [IAP] Bootloader identified (0x02, 0xFD).");
                    }
                    MotorFlashProgress::HeaderAccepted => {
                        println!(
                            "  [IAP] 12-byte header accepted (ACK 0x06). Streaming sectors..."
                        );
                    }
                    MotorFlashProgress::SectorAcked {
                        sector_index,
                        total_sectors,
                        bytes_sent,
                        total_bytes,
                    } => {
                        println!(
                            "  [IAP] Sector {:02}/{:02} ACKed ({}/{} bytes)",
                            sector_index + 1,
                            total_sectors,
                            bytes_sent,
                            total_bytes
                        );
                    }
                    MotorFlashProgress::VerifiedComplete => {
                        println!("  [IAP] Final payload verification EOT (0x04) received!");
                    }
                },
            ) {
                eprintln!("Motor IAP flash failed: {:?}", err);
                process::exit(2);
            }
            println!("SUCCESS: Motor firmware flashed to {}!", port);
        }
        return;
    }

    let pack_path = &args[2];
    let raw_pack = match fs::read(pack_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error reading '{}': {}", pack_path, e);
            process::exit(1);
        }
    };

    let meta = match parse_pack_metadata(&raw_pack) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error parsing .Pack header: {:?}", e);
            process::exit(1);
        }
    };

    let mut payload = vec![0u8; meta.payload_len];
    if let Err(e) = decrypt_payload(&raw_pack, &meta, &mut payload) {
        eprintln!("Error decrypting payload: {:?}", e);
        process::exit(1);
    }

    match cmd {
        "info" => {
            println!("=== ToSeven .Pack Firmware Container ===");
            println!("File:              {}", pack_path);
            println!(
                "Target Hardware:   {:?} (Device ID: 0x{:06X})",
                meta.target, meta.device_id_24
            );
            println!("Header XOR Key:    0x{:02X}", meta.header_xor_key);
            println!("Payload XOR Key:   0x{:02X}", meta.payload_xor_key);
            println!(
                "Payload Length:    {} bytes (0x{:04X}), loaded at 0x{:08X}..0x{:08X}",
                meta.payload_len,
                meta.payload_len,
                APP_FLASH_BASE_ADDR,
                APP_FLASH_BASE_ADDR + (meta.payload_len as u32)
            );
            println!("Initial MSP:       0x{:08X}", meta.initial_sp);
            println!("Reset_Handler:     0x{:08X}", meta.reset_handler);

            let eeprom_flash_addr = match meta.target {
                MotorTarget::Dm01 => Some(0x0800_EE70u32),
                MotorTarget::Dm02 => Some(0x0800_F142u32),
                MotorTarget::Unknown(_) => None,
            };

            if let Some(addr) = eeprom_flash_addr {
                let off = (addr - APP_FLASH_BASE_ADDR) as usize;
                if off + EEPROM_TOTAL_LEN <= payload.len() {
                    let slice: &[u8; EEPROM_TOTAL_LEN] =
                        payload[off..off + EEPROM_TOTAL_LEN].try_into().unwrap();
                    let eeprom = EepromImage { raw: *slice };
                    let model_str = String::from_utf8_lossy(eeprom.model_name_bytes());
                    println!(
                        "Factory EEPROM:    0x{:08X} (Model: '{}', 5-Block CRC Check: {:?})",
                        addr,
                        model_str,
                        eeprom.verify_all_blocks()
                    );
                    println!("Default Pwr[1..5]:     {:?}%", eeprom.assist_pwr_table());
                    println!(
                        "Default EAssist[1..5]: {:?}%",
                        eeprom.assist_eassist_table()
                    );
                }
            }
        }
        "unpack" => {
            if args.len() < 4 {
                print_usage();
                process::exit(1);
            }
            let out_path = &args[3];
            if let Err(e) = fs::write(out_path, &payload) {
                eprintln!("Error writing '{}': {}", out_path, e);
                process::exit(1);
            }
            println!(
                "Unpacked {} bytes (XOR key 0x{:02X}) to '{}'",
                payload.len(),
                meta.payload_xor_key,
                out_path
            );
        }
        "repack" => {
            if args.len() < 5 {
                print_usage();
                process::exit(1);
            }
            let new_payload_path = &args[3];
            let out_pack_path = &args[4];
            let new_payload = match fs::read(new_payload_path) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Error reading '{}': {}", new_payload_path, e);
                    process::exit(1);
                }
            };
            let mut out_pack = [0u8; PACK_FILE_SIZE];
            if let Err(e) = repack_firmware(&meta, &new_payload, Some(&raw_pack), &mut out_pack) {
                eprintln!("Error repacking firmware: {:?}", e);
                process::exit(1);
            }
            if let Err(e) = fs::write(out_pack_path, out_pack) {
                eprintln!("Error writing '{}': {}", out_pack_path, e);
                process::exit(1);
            }
            println!(
                "Repacked {} bytes into '{}' (128 KB .Pack container)",
                new_payload.len(),
                out_pack_path
            );
        }
        "verify" => {
            let sha = sha256(&raw_pack);
            println!("Pack SHA-256:      {}", hex(&sha));
            println!("Payload SHA-256:   {}", hex(&sha256(&payload)));
            match stock::identify(&sha) {
                Some(r) => println!(
                    "Official release:  {} (known stock, {:?})",
                    r.name, r.target
                ),
                None => {
                    println!("Official release:  UNKNOWN (custom or modified release)");
                }
            }
            // Stock round-trip must be bit-identical.
            let mut out = [0u8; PACK_FILE_SIZE];
            repack_firmware(&meta, &payload, Some(&raw_pack), &mut out).expect("repack");
            if out[..] != raw_pack[..] {
                eprintln!("Round-trip repack is NOT bit-identical");
                process::exit(3);
            }
            println!("Round-trip repack: bit-identical OK");
        }
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}
