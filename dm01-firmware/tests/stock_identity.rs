//! Verifies that official stock .Pack containers reproduce bit-for-bit when repacked,
//! and that source-level tuning profiles produce correct motor control responses.

use dm01_firmware::pack::{
    decrypt_payload, parse_pack_metadata, repack_firmware, MotorTarget, PACK_FILE_SIZE,
};
use dm01_firmware::power_curve::TorqueSensorState;
use dm01_firmware::sha256::sha256;
use dm01_firmware::stock;
use dm01_firmware::tuning::TuningProfile;
use std::fs;
use std::path::{Path, PathBuf};

const PACKS: &[(&[&str], MotorTarget)] = &[
    (
        &[
            "../blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack",
            "blobs/firmware/dm01/TO7-DM01-K260521A-V6510C.Pack",
        ],
        MotorTarget::Dm01,
    ),
    (
        &[
            "../blobs/firmware/dm02/TO7-DM02-K260526-V6510C.Pack",
            "blobs/firmware/dm02/TO7-DM02-K260526-V6510C.Pack",
        ],
        MotorTarget::Dm02,
    ),
];

fn find_pack(candidates: &[&str]) -> Option<PathBuf> {
    for c in candidates {
        let p = Path::new(c);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }
    None
}

#[test]
fn stock_is_registered_and_round_trips_bit_identical() {
    for (candidates, expected_target) in PACKS {
        let Some(path) = find_pack(candidates) else {
            println!("Skipping: stock .Pack not found in candidates {candidates:?}");
            continue;
        };
        let raw = fs::read(&path).expect("read pack");
        assert_eq!(raw.len(), PACK_FILE_SIZE);

        let meta = parse_pack_metadata(&raw).expect("metadata");
        assert_eq!(meta.target, *expected_target);

        let release = stock::identify(&sha256(&raw)).expect("known official release");
        assert_eq!(release.target, *expected_target);

        let mut payload = vec![0u8; meta.payload_len];
        decrypt_payload(&raw, &meta, &mut payload).expect("decrypt");

        // Repack with template => bit-for-bit identical to the official vendor container
        let mut out = [0u8; PACK_FILE_SIZE];
        repack_firmware(&meta, &payload, Some(&raw), &mut out).expect("repack");
        assert_eq!(out[..], raw[..], "Pack roundtrip must match byte for byte");
        assert_eq!(sha256(&out), sha256(&raw));

        // Repack without template => header and payload regions match exactly
        let mut bare = [0u8; PACK_FILE_SIZE];
        repack_firmware(&meta, &payload, None, &mut bare).expect("bare repack");
        let used = 34 + meta.payload_len;
        assert_eq!(bare[..used], raw[..used]);
    }
}

#[test]
fn test_source_level_tuning_customizations() {
    let mut torque_stock = TorqueSensorState::new(750, 3000, 50);
    let mut torque_uncapped = TorqueSensorState::new(750, 3000, 50);

    // Simulate riding at 25 RPM (climbing a steep grade with 1.8V strain gauge force)
    let pedal_mv = 1800;
    let cadence_rpm = 25;
    let strength_value = 10;
    let eassist_pct = 100;
    let mode_max_da = 300; // 30.0 A
    let global_ceiling_da = 300;
    let level_ceiling_da = 200; // Assist level 3 is 20A

    let stock_profile = TuningProfile::stock_v6510c();
    let uncapped_profile = TuningProfile::uncapped_cadence();
    let commuter_profile = TuningProfile::commuter_comfort();

    let stock_out = torque_stock.compute_current_da_with_profile(
        pedal_mv,
        cadence_rpm,
        1,
        strength_value,
        eassist_pct,
        5,
        mode_max_da,
        global_ceiling_da,
        level_ceiling_da,
        &stock_profile,
    );

    let uncapped_out = torque_uncapped.compute_current_da_with_profile(
        pedal_mv,
        cadence_rpm,
        1,
        strength_value,
        eassist_pct,
        5,
        mode_max_da,
        global_ceiling_da,
        level_ceiling_da,
        &uncapped_profile,
    );

    let mut torque_commuter = TorqueSensorState::new(750, 3000, 50);
    let commuter_out = torque_commuter.compute_current_da_with_profile(
        pedal_mv,
        cadence_rpm,
        1,
        strength_value,
        eassist_pct,
        5,
        mode_max_da,
        global_ceiling_da,
        level_ceiling_da,
        &commuter_profile,
    );

    // In stock, 25 RPM scales current down: 25 / 60 = ~41% of full power
    // With uncapped cadence, 25 RPM receives full 100% torque assist!
    assert!(
        uncapped_out > stock_out,
        "Uncapped profile must deliver more power at low cadence than stock"
    );

    // In commuter profile with UnifiedLevelCeiling, current must not exceed level_ceiling_da (200 dA = 20A)
    assert!(
        commuter_out <= level_ceiling_da,
        "Commuter profile with unified ceiling must respect assist level ceiling"
    );
}
