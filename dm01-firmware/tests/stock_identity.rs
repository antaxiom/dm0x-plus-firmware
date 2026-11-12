//! Verifies that official stock .Pack containers reproduce bit-for-bit when repacked,
//! and that the compiled power-curve constants produce the intended gear-safe behavior.

use dm01_firmware::pack::{
    decrypt_payload, parse_pack_metadata, repack_firmware, MotorTarget, PACK_FILE_SIZE,
};
use dm01_firmware::power_curve::{
    cadence_multiplier_q8, TorqueSensorState, TORQUE_USES_LEVEL_CEILING,
};
use dm01_firmware::sha256::sha256;
use dm01_firmware::stock;
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
            "../blobs/firmware/dm01/TO7-DM01-K261023-V6510C.Pack",
            "blobs/firmware/dm01/TO7-DM01-K261023-V6510C.Pack",
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
    (
        &[
            "../blobs/firmware/dm02/TO7-DM02-K261023-V6510C.Pack",
            "blobs/firmware/dm02/TO7-DM02-K261023-V6510C.Pack",
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
fn test_compiled_constants_are_gear_safe_and_dual_ceiling() {
    let mut torque = TorqueSensorState::new(750, 3000, 50);

    let full_at_35 = torque.compute_current_da(3000, 35, 0, 10, 100, 5, 300, 300, 200);
    let protected_at_10 = torque.compute_current_da(3000, 10, 0, 10, 100, 5, 300, 300, 200);

    assert!(
        protected_at_10 < full_at_35,
        "Gear-safe knee must reduce current at 10 RPM"
    );
    assert_eq!(
        cadence_multiplier_q8(35),
        256,
        "35 RPM must reach full cadence assist"
    );

    if TORQUE_USES_LEVEL_CEILING {
        assert_eq!(
            full_at_35, 200,
            "Unified ceiling must clamp torque to the level ceiling"
        );
    } else {
        assert_eq!(
            full_at_35, 300,
            "Stock dual ceiling must let torque use the global mode ceiling"
        );
    }
}

#[test]
fn test_gear_safe_knee_limits_low_cadence_torque() {
    let mut torque = TorqueSensorState::new(750, 3000, 50);

    let at_10 = torque.compute_current_da(3000, 10, 0, 10, 100, 5, 300, 300, 300);
    let at_18 = torque.compute_current_da(3000, 18, 0, 10, 100, 5, 300, 300, 300);
    let at_35 = torque.compute_current_da(3000, 35, 0, 10, 100, 5, 300, 300, 300);

    assert!(at_10 > 0, "Gear-safe knee still provides gentle assist");
    assert!(
        at_10 < at_18 && at_18 < at_35,
        "Current must ramp monotonically from the knee to full assist"
    );
    assert_eq!(cadence_multiplier_q8(10), 54);
    assert_eq!(cadence_multiplier_q8(18), 102);
    assert_eq!(cadence_multiplier_q8(35), 256);
}
