//! Registry of known **official** ToSeven `.Pack` releases (SHA-256 of the whole 128 KB container).
//!
//! When a new official release appears, add it here (and drop the file in the matching
//! `* Sources/` folder). `toseven_pack verify` reports whether a file is a known stock release.

use crate::pack::MotorTarget;

/// One known official release.
#[derive(Debug, Clone, Copy)]
pub struct StockRelease {
    pub name: &'static str,
    pub target: MotorTarget,
    pub pack_sha256: [u8; 32],
}

const fn hex(s: &str) -> [u8; 32] {
    let b = s.as_bytes();
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = (nib(b[2 * i]) << 4) | nib(b[2 * i + 1]);
        i += 1;
    }
    out
}

const fn nib(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        _ => panic!("bad hex"),
    }
}

/// All official releases analysed so far.
pub const KNOWN_RELEASES: &[StockRelease] = &[
    StockRelease {
        name: "TO7-DM01-K260521A-V6510C",
        target: MotorTarget::Dm01,
        pack_sha256: hex("eb964a1787ba90f35965921353911c8a6e3083e1914cf422caeaa515f36ee579"),
    },
    StockRelease {
        name: "TO7-DM02-K260526-V6510C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("3ab150d10d3106080233d4f41e7df6a13f67154994138d831df67b1094ea1ca2"),
    },
];

/// Looks up a release by container hash.
pub fn identify(pack_sha256: &[u8; 32]) -> Option<&'static StockRelease> {
    KNOWN_RELEASES
        .iter()
        .find(|r| &r.pack_sha256 == pack_sha256)
}
