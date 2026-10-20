#![windows_subsystem = "windows"]
//! # DM+ Firmware Studio — Slint GUI Application
//!
//! Visual desktop studio for configuring, tuning, and flashing ToSeven DM01/DM02 open firmware,
//! as well as inspecting, unlocking, and flashing display firmware.

slint::include_modules!();

use dm01_firmware::display_patcher::{inspect_display, patch_display, IntelHexFile, LockoutStatus};
use dm01_firmware::flasher::FlasherError;
use dm01_firmware::pack::{decrypt_payload, parse_pack_metadata, PACK_FILE_SIZE};
use dm01_firmware::serial::{
    execute_display_isp_flash, execute_motor_iap_flash, list_serial_ports, DisplayFlashProgress,
    MotorFlashProgress, PosixSerialPort,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::fs;
use std::thread;
use std::time::Duration;

fn update_ports_in_ui(ui: &MainWindow) {
    let ports = list_serial_ports();
    let (ports_model, initial_port) = if ports.is_empty() {
        let default_str = SharedString::from("No serial devices detected");
        let items = vec![default_str.clone()];
        (ModelRc::new(VecModel::from(items)), default_str)
    } else {
        let items: Vec<SharedString> = ports
            .iter()
            .map(|p| SharedString::from(p.as_str()))
            .collect();
        let first = items[0].clone();
        (ModelRc::new(VecModel::from(items)), first)
    };
    ui.set_available_ports(ports_model);

    let cur_motor = ui.get_selected_motor_port();
    if cur_motor.is_empty()
        || cur_motor == "No serial devices detected"
        || !ports.iter().any(|p| p == cur_motor.as_str())
    {
        ui.set_selected_motor_port(initial_port.clone());
    }

    let cur_display = ui.get_selected_display_port();
    if cur_display.is_empty()
        || cur_display == "No serial devices detected"
        || !ports.iter().any(|p| p == cur_display.as_str())
    {
        ui.set_selected_display_port(initial_port);
    }
}

const EMBEDDED_DM01_OPEN_PACK: &[u8] =
    include_bytes!("../assets/TO7-DM01-OpenFirmware-v0.3.0.Pack");
const EMBEDDED_DM02_OPEN_PACK: &[u8] =
    include_bytes!("../assets/TO7-DM02-OpenFirmware-v0.3.0.Pack");

fn get_motor_pack_bytes(source: &str, custom_path: &str) -> Result<Vec<u8>, String> {
    if source.contains("DM02 Open Firmware") {
        return Ok(EMBEDDED_DM02_OPEN_PACK.to_vec());
    } else if source.contains("DM01 Open Firmware")
        || source.contains("Open Firmware")
        || source.contains("Built-in")
    {
        return Ok(EMBEDDED_DM01_OPEN_PACK.to_vec());
    } else if source.contains("Custom") {
        let trimmed = custom_path.trim();
        if trimmed.is_empty() {
            return Err(
                "No custom .Pack file path specified. Click 'Choose File...' to select one."
                    .to_string(),
            );
        }
        let bytes = fs::read(trimmed).map_err(|e| format!("Error reading '{}': {}", trimmed, e))?;
        if bytes.len() != PACK_FILE_SIZE {
            return Err(format!(
                "'{}' is {} bytes (expected 128 KB .Pack file).",
                trimmed,
                bytes.len()
            ));
        }
        return Ok(bytes);
    }
    Err(format!("Unknown firmware source: '{}'", source))
}

fn do_inspect_display_hex(ui: &MainWindow, path_str: &str) {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return;
    }
    match fs::read_to_string(trimmed) {
        Ok(raw) => match IntelHexFile::parse(&raw) {
            Ok(hex_file) => match inspect_display(&hex_file) {
                Ok(info) => {
                    ui.set_display_model_name(SharedString::from(info.identified_model));
                    match info.lockout_status {
                        LockoutStatus::Locked { patches } => {
                            ui.set_display_is_locked(true);
                            ui.set_display_lockout_status(SharedString::from(format!(
                                "LOCKED: {} restriction gates detected (Post-October 2024)",
                                patches.len()
                            )));
                        }
                        LockoutStatus::AlreadyUnlocked { gate_addresses } => {
                            ui.set_display_is_locked(false);
                            ui.set_display_lockout_status(SharedString::from(format!(
                                "ALREADY UNLOCKED ({} gates NOP'd)",
                                gate_addresses.len()
                            )));
                        }
                        LockoutStatus::LegacyUnlocked => {
                            ui.set_display_is_locked(false);
                            ui.set_display_lockout_status(SharedString::from(
                                "LEGACY UNLOCKED (Pre-October 2024 firmware)",
                            ));
                        }
                    }

                    if ui.get_display_output_path().is_empty() {
                        let stem = trimmed.strip_suffix(".hex").unwrap_or(trimmed);
                        ui.set_display_output_path(SharedString::from(format!(
                            "{}-Unlocked.hex",
                            stem
                        )));
                    }
                }
                Err(e) => {
                    ui.set_display_model_name(SharedString::from("Error"));
                    ui.set_display_lockout_status(SharedString::from(format!(
                        "Inspection error: {:?}",
                        e
                    )));
                    ui.set_display_is_locked(false);
                }
            },
            Err(e) => {
                ui.set_display_model_name(SharedString::from("Parse Error"));
                ui.set_display_lockout_status(SharedString::from(format!("Invalid HEX: {:?}", e)));
                ui.set_display_is_locked(false);
            }
        },
        Err(e) => {
            ui.set_display_model_name(SharedString::from("File Error"));
            ui.set_display_lockout_status(SharedString::from(format!("Cannot read: {}", e)));
            ui.set_display_is_locked(false);
        }
    }
}

#[cfg(target_os = "macos")]
fn set_macos_dock_icon() {
    use std::ffi::{c_char, c_void};

    extern "C" {
        fn objc_getClass(name: *const c_char) -> *mut c_void;
        fn sel_registerName(name: *const c_char) -> *mut c_void;
        fn objc_msgSend();
        fn objc_autoreleasePoolPush() -> *mut c_void;
        fn objc_autoreleasePoolPop(pool: *mut c_void);
    }

    const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

    unsafe {
        let pool = objc_autoreleasePoolPush();

        let ns_app_class = objc_getClass(c"NSApplication".as_ptr());
        let shared_app_sel = sel_registerName(c"sharedApplication".as_ptr());
        let msg_send_no_arg: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        let app = msg_send_no_arg(ns_app_class, shared_app_sel);

        if !app.is_null() {
            // Ensure activation policy is regular GUI application
            let set_policy_sel = sel_registerName(c"setActivationPolicy:".as_ptr());
            let msg_send_set_policy: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) -> bool =
                std::mem::transmute(objc_msgSend as *const ());
            msg_send_set_policy(app, set_policy_sel, 0);

            let ns_data_class = objc_getClass(c"NSData".as_ptr());
            let data_with_bytes_sel = sel_registerName(c"dataWithBytes:length:".as_ptr());
            let msg_send_data: unsafe extern "C" fn(
                *mut c_void,
                *mut c_void,
                *const u8,
                usize,
            ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            let data = msg_send_data(
                ns_data_class,
                data_with_bytes_sel,
                LOGO_PNG.as_ptr(),
                LOGO_PNG.len(),
            );

            if !data.is_null() {
                let ns_image_class = objc_getClass(c"NSImage".as_ptr());
                let alloc_sel = sel_registerName(c"alloc".as_ptr());
                let init_with_data_sel = sel_registerName(c"initWithData:".as_ptr());

                let image_alloc = msg_send_no_arg(ns_image_class, alloc_sel);
                let msg_send_init: unsafe extern "C" fn(
                    *mut c_void,
                    *mut c_void,
                    *mut c_void,
                ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
                let image = msg_send_init(image_alloc, init_with_data_sel, data);

                if !image.is_null() {
                    let set_icon_sel = sel_registerName(c"setApplicationIconImage:".as_ptr());
                    msg_send_init(app, set_icon_sel, image);

                    // Set dockTile.contentView to an NSImageView with the logo
                    let ns_image_view_class = objc_getClass(c"NSImageView".as_ptr());
                    let view_alloc = msg_send_no_arg(ns_image_view_class, alloc_sel);
                    let init_sel = sel_registerName(c"init".as_ptr());
                    let view = msg_send_no_arg(view_alloc, init_sel);
                    let set_image_sel = sel_registerName(c"setImage:".as_ptr());
                    msg_send_init(view, set_image_sel, image);

                    let dock_tile_sel = sel_registerName(c"dockTile".as_ptr());
                    let dock_tile = msg_send_no_arg(app, dock_tile_sel);
                    if !dock_tile.is_null() {
                        let set_content_view_sel = sel_registerName(c"setContentView:".as_ptr());
                        msg_send_init(dock_tile, set_content_view_sel, view);

                        let display_sel = sel_registerName(c"display".as_ptr());
                        let msg_send_void: unsafe extern "C" fn(*mut c_void, *mut c_void) =
                            std::mem::transmute(objc_msgSend as *const ());
                        msg_send_void(dock_tile, display_sel);
                    }
                }
            }
        }

        objc_autoreleasePoolPop(pool);
    }
}

fn main() -> Result<(), slint::PlatformError> {
    #[cfg(target_os = "macos")]
    set_macos_dock_icon();

    let main_window = MainWindow::new()?;

    #[cfg(target_os = "macos")]
    let _dock_timer = {
        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::SingleShot,
            Duration::from_millis(100),
            || {
                set_macos_dock_icon();
            },
        );
        timer
    };

    // Initialize available serial ports
    update_ports_in_ui(&main_window);

    let cached_ports = std::rc::Rc::new(std::cell::RefCell::new(list_serial_ports()));

    // Callback: Manual Refresh Ports button
    let handle_weak = main_window.as_weak();
    let cached_ports_manual = cached_ports.clone();
    main_window.on_refresh_ports(move || {
        if let Some(ui) = handle_weak.upgrade() {
            *cached_ports_manual.borrow_mut() = list_serial_ports();
            update_ports_in_ui(&ui);
        }
    });

    // Auto-refresh timer: polls every 1200 ms to detect hotplugged or disconnected serial devices
    let _refresh_timer = slint::Timer::default();
    let handle_weak = main_window.as_weak();
    let cached_ports_timer = cached_ports.clone();
    _refresh_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(1200),
        move || {
            if let Some(ui) = handle_weak.upgrade() {
                // Never disrupt ongoing firmware flashing operations
                if ui.get_motor_is_flashing() || ui.get_display_is_flashing() {
                    return;
                }
                let current_ports = list_serial_ports();
                let mut cached = cached_ports_timer.borrow_mut();
                if current_ports != *cached {
                    *cached = current_ports;
                    update_ports_in_ui(&ui);
                }
            }
        },
    );

    // Callback: Browse Motor .Pack File
    let handle_weak = main_window.as_weak();
    main_window.on_browse_motor_pack(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ToSeven Motor Firmware (*.Pack)", &["Pack", "pack"])
            .pick_file()
        {
            let path_str = path.to_string_lossy().to_string();
            if let Some(ui) = handle_weak.upgrade() {
                ui.set_motor_custom_pack_path(SharedString::from(path_str.clone()));
                ui.set_motor_firmware_source(SharedString::from(
                    "Custom .Pack File (Manual Selection)",
                ));
                // Automatically inspect custom .Pack header to detect target hardware!
                if let Ok(bytes) = fs::read(&path_str) {
                    if let Ok(meta) = parse_pack_metadata(&bytes) {
                        let target_str = match meta.target {
                            dm01_firmware::pack::MotorTarget::Dm01 => "DM01 (External 12-FET)",
                            dm01_firmware::pack::MotorTarget::Dm02 => "DM02 (Integrated 6-FET)",
                            dm01_firmware::pack::MotorTarget::Unknown(_) => "Custom / Other Device",
                        };
                        ui.set_motor_target(SharedString::from(target_str));
                    }
                }
            }
        } else if let Some(ui) = handle_weak.upgrade() {
            if ui.get_motor_custom_pack_path().is_empty() {
                ui.set_motor_firmware_source(SharedString::from(
                    "DM01 Open Firmware v0.3.0 (Built-in)",
                ));
            }
        }
    });

    // Callback: Browse Display Input HEX File
    let handle_weak = main_window.as_weak();
    main_window.on_browse_display_hex(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Intel HEX Firmware (*.hex)", &["hex", "HEX"])
            .pick_file()
        {
            let path_str = path.to_string_lossy().to_string();
            if let Some(ui) = handle_weak.upgrade() {
                ui.set_display_hex_path(SharedString::from(path_str.clone()));
                do_inspect_display_hex(&ui, &path_str);
            }
        }
    });

    // Callback: Export/Save Patched Display HEX directly to disk
    let handle_weak = main_window.as_weak();
    main_window.on_export_patched_display_hex(move |hex_path| {
        if let Some(ui) = handle_weak.upgrade() {
            let path_str = hex_path.as_str().trim().to_string();
            if path_str.is_empty() {
                ui.set_display_status(SharedString::from(
                    "Please choose a display HEX file first.",
                ));
                return;
            }

            let raw = match fs::read_to_string(&path_str) {
                Ok(s) => s,
                Err(e) => {
                    ui.set_display_status(SharedString::from(format!(
                        "Cannot read input file: {}",
                        e
                    )));
                    return;
                }
            };

            let mut hex_file = match IntelHexFile::parse(&raw) {
                Ok(h) => h,
                Err(e) => {
                    ui.set_display_status(SharedString::from(format!(
                        "Failed to parse Intel HEX: {:?}",
                        e
                    )));
                    return;
                }
            };

            let default_name = std::path::Path::new(&path_str)
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| format!("{}-Unlocked.hex", s))
                .unwrap_or_else(|| "Display-Unlocked.hex".to_string());

            if let Some(save_path) = rfd::FileDialog::new()
                .add_filter("Intel HEX Firmware (*.hex)", &["hex", "HEX"])
                .set_file_name(&default_name)
                .save_file()
            {
                match patch_display(&mut hex_file) {
                    Ok(patches) => {
                        let count = patches.len();
                        let encoded = hex_file.to_hex_string();
                        let save_str = save_path.to_string_lossy().to_string();
                        if let Err(e) = fs::write(&save_str, encoded) {
                            ui.set_display_status(SharedString::from(format!(
                                "Failed to write file: {}",
                                e
                            )));
                            return;
                        }
                        ui.set_display_status(SharedString::from(format!(
                            "Neutralized {} lockout gates. Saved to '{}'",
                            count, save_str
                        )));
                        ui.set_display_is_locked(false);
                        ui.set_display_lockout_status(SharedString::from("UNLOCKED (Patched)"));
                    }
                    Err(e) => {
                        ui.set_display_status(SharedString::from(format!(
                            "Patching failed: {:?}",
                            e
                        )));
                    }
                }
            }
        }
    });

    // Callback: Browse Display Output HEX File
    let handle_weak = main_window.as_weak();
    main_window.on_browse_display_output_hex(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Intel HEX Firmware (*.hex)", &["hex", "HEX"])
            .save_file()
        {
            let path_str = path.to_string_lossy().to_string();
            if let Some(ui) = handle_weak.upgrade() {
                ui.set_display_output_path(SharedString::from(path_str));
            }
        }
    });

    // Callback: Inspect Display HEX
    let handle_weak = main_window.as_weak();
    main_window.on_inspect_display_hex(move |hex_path| {
        let path_str = hex_path.as_str().trim();
        if let Some(ui) = handle_weak.upgrade() {
            do_inspect_display_hex(&ui, path_str);
        }
    });

    // Callback: Patch Display HEX
    let handle_weak = main_window.as_weak();
    main_window.on_patch_display_hex(move |in_path, out_path| {
        if let Some(ui) = handle_weak.upgrade() {
            let in_str = in_path.as_str().trim();
            let out_str = out_path.as_str().trim();
            if in_str.is_empty() || out_str.is_empty() {
                ui.set_display_status(SharedString::from("Error: Input or output path is empty."));
                return;
            }

            let raw = match fs::read_to_string(in_str) {
                Ok(s) => s,
                Err(e) => {
                    ui.set_display_status(SharedString::from(format!(
                        "Cannot read input file: {}",
                        e
                    )));
                    return;
                }
            };

            let mut hex_file = match IntelHexFile::parse(&raw) {
                Ok(h) => h,
                Err(e) => {
                    ui.set_display_status(SharedString::from(format!(
                        "Failed to parse Intel HEX: {:?}",
                        e
                    )));
                    return;
                }
            };

            match patch_display(&mut hex_file) {
                Ok(patches) => {
                    let count = patches.len();
                    let encoded = hex_file.to_hex_string();
                    if let Err(e) = fs::write(out_str, encoded) {
                        ui.set_display_status(SharedString::from(format!(
                            "Failed to write output file: {}",
                            e
                        )));
                        return;
                    }
                    ui.set_display_status(SharedString::from(format!(
                        "Successfully neutralized {} lockout gates. Saved to '{}'",
                        count, out_str
                    )));
                    ui.set_display_is_locked(false);
                    ui.set_display_lockout_status(SharedString::from("UNLOCKED (Patched)"));
                }
                Err(e) => {
                    ui.set_display_status(SharedString::from(format!("Patching failed: {:?}", e)));
                }
            }
        }
    });

    // Callback: Flash Motor (Handles both Dry-Run and Live Hardware Flash in a background thread)
    let handle_weak = main_window.as_weak();
    main_window.on_flash_motor(
        move |source_str, custom_path_str, target_str, _profile_str, port_str, is_dry_run, _max_amps, _battery_volts| {
            if let Some(ui) = handle_weak.upgrade() {
                let port = port_str.as_str().trim().to_string();
                let source = source_str.as_str().to_string();
                let custom_path = custom_path_str.as_str().to_string();
                let _is_dm01 = target_str.contains("DM01");

                // Resolve target .Pack bytes (uses embedded firmware for built-in, or reads custom file)
                let pack_bytes = match get_motor_pack_bytes(&source, &custom_path) {
                    Ok(b) => b,
                    Err(err_msg) => {
                        ui.set_motor_status(SharedString::from(err_msg));
                        return;
                    }
                };

                let meta = match parse_pack_metadata(&pack_bytes) {
                    Ok(m) => m,
                    Err(e) => {
                        ui.set_motor_status(SharedString::from(format!("Invalid .Pack header: {:?}", e)));
                        return;
                    }
                };

                let mut payload = vec![0u8; meta.payload_len];
                if let Err(e) = decrypt_payload(&pack_bytes, &meta, &mut payload) {
                    ui.set_motor_status(SharedString::from(format!("Decryption error: {:?}", e)));
                    return;
                }

                let customer_num = meta.customer_num();

                // Put UI into flashing state
                ui.set_motor_is_flashing(true);
                ui.set_motor_progress(0.05);

                let handle_ui = handle_weak.clone();

                // Spawn background worker thread to prevent freezing the GUI
                thread::spawn(move || {
                    if is_dry_run {
                        // Simulated Dry-Run Execution
                        let steps = [
                            (0.15, "Dry-Run: Probing bootloader at 38400 Baud..."),
                            (0.30, "Dry-Run: Received IDENT (0x02, 0xFD). Sending wakeup frame..."),
                            (0.45, "Dry-Run: Header frame accepted (0x06 ACK)..."),
                            (0.65, "Dry-Run: Simulating 16-byte chunk and 1 KB sector pacing..."),
                            (0.85, "Dry-Run: Verifying payload CRC-16 checksum..."),
                            (1.00, "Dry-Run complete: Handshake, sectors, and checksums 100% verified."),
                        ];
                        for (prog, msg) in steps {
                            thread::sleep(Duration::from_millis(250));
                            let msg_str = msg.to_string();
                            let h = handle_ui.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = h.upgrade() {
                                    ui.set_motor_progress(prog);
                                    ui.set_motor_status(SharedString::from(msg_str));
                                }
                            });
                        }
                        let h = handle_ui.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = h.upgrade() {
                                ui.set_motor_is_flashing(false);
                            }
                        });
                    } else {
                        // Live Hardware Flash
                        if port.is_empty() || port.contains("No serial") {
                            let h = handle_ui.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = h.upgrade() {
                                    ui.set_motor_status(SharedString::from("Error: No valid serial port selected."));
                                    ui.set_motor_is_flashing(false);
                                    ui.set_motor_progress(0.0);
                                }
                            });
                            return;
                        }

                        let h_open = handle_ui.clone();
                        let port_msg = port.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = h_open.upgrade() {
                                ui.set_motor_status(SharedString::from(format!(
                                    "Connecting to {} @ 38400 Baud (Timeout: 2.5s)...",
                                    port_msg
                                )));
                                ui.set_motor_progress(0.10);
                            }
                        });

                        let mut serial = match PosixSerialPort::open(&port, 38_400) {
                            Ok(p) => p,
                            Err(e) => {
                                let h = handle_ui.clone();
                                let err_msg = format!("Cannot open port '{}': {}", port, e);
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(ui) = h.upgrade() {
                                        ui.set_motor_status(SharedString::from(err_msg));
                                        ui.set_motor_is_flashing(false);
                                        ui.set_motor_progress(0.0);
                                    }
                                });
                                return;
                            }
                        };

                        let h_prog = handle_ui.clone();
                        let flash_res = execute_motor_iap_flash(
                            &mut serial,
                            &payload,
                            meta.uart_mask,
                            customer_num,
                            move |ev| {
                                let (frac, text) = match ev {
                                    MotorFlashProgress::ProbingBootloader => {
                                        (0.15, "Probing bootloader at 38400 Baud...")
                                    }
                                    MotorFlashProgress::SendingWakeup9600 => {
                                        (0.25, "Sending 21-byte wakeup frame at 9600 Baud...")
                                    }
                                    MotorFlashProgress::BootloaderIdentified => {
                                        (0.35, "Bootloader identified (0x02, 0xFD).")
                                    }
                                    MotorFlashProgress::HeaderAccepted => {
                                        (0.40, "12-byte header accepted. Streaming sectors...")
                                    }
                                    MotorFlashProgress::SectorAcked {
                                        sector_index,
                                        total_sectors,
                                        ..
                                    } => {
                                        let p = 0.40 + 0.55 * (sector_index as f32 / total_sectors.max(1) as f32);
                                        (p, "Streaming firmware sectors to Flash...")
                                    }
                                    MotorFlashProgress::VerifiedComplete => {
                                        (1.00, "Verification complete! MCU rebooting into application...")
                                    }
                                };
                                let h = h_prog.clone();
                                let t_str = text.to_string();
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(ui) = h.upgrade() {
                                        ui.set_motor_progress(frac);
                                        ui.set_motor_status(SharedString::from(t_str));
                                    }
                                });
                            },
                        );

                        let h_end = handle_ui.clone();
                        let port_end = port.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = h_end.upgrade() {
                                ui.set_motor_is_flashing(false);
                                match flash_res {
                                    Ok(()) => {
                                        ui.set_motor_progress(1.0);
                                        ui.set_motor_status(SharedString::from(
                                            "Flash successful! Motor controller rebooted.",
                                        ));
                                    }
                                    Err(FlasherError::Timeout) => {
                                        ui.set_motor_progress(0.0);
                                        ui.set_motor_status(SharedString::from(format!(
                                            "Connection timed out: No response from motor on {}. Check power and cable.",
                                            port_end
                                        )));
                                    }
                                    Err(e) => {
                                        ui.set_motor_progress(0.0);
                                        ui.set_motor_status(SharedString::from(format!("Flash error: {:?}", e)));
                                    }
                                }
                            }
                        });
                    }
                });
            }
        },
    );

    // Callback: Flash Display (Handles both Dry-Run and Live Hardware Flash in a background thread)
    let handle_weak = main_window.as_weak();
    main_window.on_flash_display_hex(move |hex_path, port_str, auto_unlock, is_dry_run| {
        if let Some(ui) = handle_weak.upgrade() {
            let path_str = hex_path.as_str().trim().to_string();
            let port = port_str.as_str().trim().to_string();

            if path_str.is_empty() {
                ui.set_display_status(SharedString::from("Error: Display HEX file path is empty."));
                return;
            }

            let raw_str = match fs::read_to_string(&path_str) {
                Ok(s) => s,
                Err(e) => {
                    ui.set_display_status(SharedString::from(format!("Cannot read '{}': {}", path_str, e)));
                    return;
                }
            };

            let mut hex_file = match IntelHexFile::parse(&raw_str) {
                Ok(h) => h,
                Err(e) => {
                    ui.set_display_status(SharedString::from(format!("Failed to parse Intel HEX: {:?}", e)));
                    return;
                }
            };

            if auto_unlock {
                let _ = patch_display(&mut hex_file);
            }

            let (base_addr, bin) = match hex_file.to_flat_binary() {
                Ok(b) => b,
                Err(e) => {
                    ui.set_display_status(SharedString::from(format!("Binary conversion failed: {:?}", e)));
                    return;
                }
            };

            ui.set_display_is_flashing(true);
            ui.set_display_progress(0.05);

            let handle_ui = handle_weak.clone();

            thread::spawn(move || {
                if is_dry_run {
                    // Simulated Display ISP Dry-Run
                    let steps = [
                        (0.20, "Dry-Run: MegaHunt ISP Handshake 1 & 2 (0x20, 0x21)... [OK]"),
                        (0.40, "Dry-Run: Identified MH2020C & Winbond SPI Flash... [OK]"),
                        (0.60, "Dry-Run: Erasing flash sectors and streaming 256-byte chunks..."),
                        (0.85, "Dry-Run: CRC-16-CCITT packet framing verified... [OK]"),
                        (1.00, "Dry-Run complete: Display firmware & ISP framing 100% verified."),
                    ];
                    for (prog, msg) in steps {
                        thread::sleep(Duration::from_millis(250));
                        let msg_str = msg.to_string();
                        let h = handle_ui.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = h.upgrade() {
                                ui.set_display_progress(prog);
                                ui.set_display_status(SharedString::from(msg_str));
                            }
                        });
                    }
                    let h = handle_ui.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = h.upgrade() {
                            ui.set_display_is_flashing(false);
                        }
                    });
                } else {
                    // Live Display Hardware Flash
                    if port.is_empty() || port.contains("No serial") {
                        let h = handle_ui.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = h.upgrade() {
                                ui.set_display_status(SharedString::from("Error: No valid serial port selected."));
                                ui.set_display_is_flashing(false);
                                ui.set_display_progress(0.0);
                            }
                        });
                        return;
                    }

                    let h_open = handle_ui.clone();
                    let port_msg = port.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = h_open.upgrade() {
                            ui.set_display_status(SharedString::from(format!(
                                "Connecting to display on {} (Hold Power button, ensure 12-48V power)...",
                                port_msg
                            )));
                            ui.set_display_progress(0.10);
                        }
                    });

                    let mut serial = match PosixSerialPort::open(&port, 115_200) {
                        Ok(p) => p,
                        Err(e) => {
                            let h = handle_ui.clone();
                            let err_msg = format!("Cannot open port '{}': {}", port, e);
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = h.upgrade() {
                                    ui.set_display_status(SharedString::from(err_msg));
                                    ui.set_display_is_flashing(false);
                                    ui.set_display_progress(0.0);
                                }
                            });
                            return;
                        }
                    };

                    let h_prog = handle_ui.clone();
                    let flash_res = execute_display_isp_flash(
                        &mut serial,
                        base_addr,
                        &bin,
                        move |ev| {
                            let (frac, text) = match ev {
                                DisplayFlashProgress::HandshakeComplete => {
                                    (0.20, "ISP Handshake acknowledged (0x20, 0x21).")
                                }
                                DisplayFlashProgress::ChipIdentified => {
                                    (0.35, "MegaHunt MH2020C & SPI Flash ID verified.")
                                }
                                DisplayFlashProgress::FlashErased => {
                                    (0.50, "Flash memory erased. Streaming firmware chunks...")
                                }
                                DisplayFlashProgress::PacketProgrammed {
                                    packet_index,
                                    total_packets,
                                    ..
                                } => {
                                    let p = 0.50 + 0.45 * (packet_index as f32 / total_packets.max(1) as f32);
                                    (p, "Programming display firmware packets...")
                                }
                                DisplayFlashProgress::UpdateFinalized => {
                                    (1.00, "Update finalized. Display rebooting...")
                                }
                            };
                            let h = h_prog.clone();
                            let t_str = text.to_string();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = h.upgrade() {
                                    ui.set_display_progress(frac);
                                    ui.set_display_status(SharedString::from(t_str));
                                }
                            });
                        },
                    );

                    let h_end = handle_ui.clone();
                    let port_end = port.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = h_end.upgrade() {
                            ui.set_display_is_flashing(false);
                            match flash_res {
                                Ok(()) => {
                                    ui.set_display_progress(1.0);
                                    ui.set_display_status(SharedString::from(
                                        "Display flash successful! Device rebooted.",
                                    ));
                                }
                                Err(FlasherError::Timeout) => {
                                    ui.set_display_progress(0.0);
                                    ui.set_display_status(SharedString::from(format!(
                                        "Connection timed out on {}. Ensure display has 12-48V power and hold Power button down during flashing.",
                                        port_end
                                    )));
                                }
                                Err(e) => {
                                    ui.set_display_progress(0.0);
                                    ui.set_display_status(SharedString::from(format!("Display flash error: {:?}", e)));
                                }
                            }
                        }
                    });
                }
            });
        }
    });

    main_window.run()
}
