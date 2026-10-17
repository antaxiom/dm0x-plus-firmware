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
    decrypt_payload, parse_pack_metadata, repack_firmware, MotorTarget, APP_FLASH_BASE_ADDR,
    PACK_FILE_SIZE, PACK_HEADER_LEN, PACK_HEADER_MAGIC,
};
use dm01_firmware::sha256::sha256;
use dm01_firmware::stock;
use std::env;
use std::fs;
use std::process;

fn hex(d: &[u8]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  toseven_pack info <firmware.Pack>");
    eprintln!("  toseven_pack unpack <firmware.Pack> <decrypted_payload.bin>");
    eprintln!("  toseven_pack repack <template.Pack> <payload.bin> <output.Pack>");
    eprintln!("  toseven_pack create <dm01|dm02> <payload.bin> <output.Pack>");
    eprintln!("  toseven_pack verify <stock.Pack>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        process::exit(1);
    }

    let cmd = args[1].as_str();

    // Handle "create" separately because it takes a target name instead of an input .Pack
    if cmd == "create" {
        if args.len() < 5 {
            eprintln!("Usage: toseven_pack create <dm01|dm02> <payload.bin> <output.Pack>");
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

        let payload = match fs::read(payload_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error reading payload '{}': {}", payload_path, e);
                process::exit(1);
            }
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

    // Handle "create" separately because it takes a target name instead of an input .Pack
    if cmd == "create" {
        if args.len() < 5 {
            eprintln!("Usage: toseven_pack create <dm01|dm02> <payload.bin> <output.Pack>");
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

        let payload = match fs::read(payload_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error reading payload '{}': {}", payload_path, e);
                process::exit(1);
            }
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
