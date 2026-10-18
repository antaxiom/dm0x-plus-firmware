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
    // DM01 Releases (750W / 1000W)
    StockRelease {
        name: "TO7-DM01-K241021-V6212C",
        target: MotorTarget::Dm01,
        pack_sha256: hex("e81a9b49c4ed22a74da7e058138b83451b2ea1b468d577a138b27dca469a26d4"),
    },
    StockRelease {
        name: "TO7-DM01-K250731-V6510C",
        target: MotorTarget::Dm01,
        pack_sha256: hex("0f9815339647faf9bcf46f58cae6d333285ddcfcc8abe8853ace9f631fd815e3"),
    },
    StockRelease {
        name: "TO7-DM01-K250926-V6510C",
        target: MotorTarget::Dm01,
        pack_sha256: hex("3abd0a9ef915b2f40f40353bc82fcec730d4232f4891648d7b1b7ef04d30fbfd"),
    },
    StockRelease {
        name: "TO7-DM01-K260108-V6510C",
        target: MotorTarget::Dm01,
        pack_sha256: hex("c0091ef7958928e6ccd684bf61a6e4b23a5d171db68a3e05892e7cf1d3b6a8ca"),
    },
    StockRelease {
        name: "TO7-DM01-K260521A-V6510C",
        target: MotorTarget::Dm01,
        pack_sha256: hex("eb964a1787ba90f35965921353911c8a6e3083e1914cf422caeaa515f36ee579"),
    },
    // DM02 Releases (250W / 350W / 500W)
    StockRelease {
        name: "TO7-DM02-K240512A-V6206C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("6b3abdddce50963cced5d205699fa6c71b84bec6e54940141803eeb8ae6051db"),
    },
    StockRelease {
        name: "TO7-DM02-K240612B-V6306C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("0b56ee38d790fde50930e7f1fcb76a080b2dc4b17063b262e311c4c10569f8fc"),
    },
    StockRelease {
        name: "TO7-DM02-K241016-V6212C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("983dc41b0f8d83a828bf2707a39b4a0c30c5a7846398a2d039d5790c084b048a"),
    },
    StockRelease {
        name: "TO7-DM02-K241031-V6303C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("4591837f809b2fa3f52a3c13083e8a5cc5d713f1b1d53cd49adaefb568ac112a"),
    },
    StockRelease {
        name: "TO7-DM02-K241129A-V6400C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("5088cac7b137128e29b672b366171501ca5fd8e8d428c99c0c032b0ec9e9d8ec"),
    },
    StockRelease {
        name: "TO7-DM02-K250402-V6400C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("c4fcead3e6239cc4b786f2d25601b22d9633af1fd2ef4863e2d333368507fd66"),
    },
    StockRelease {
        name: "TO7-DM02-K250801-V6510C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("36c61d4581b7042df1c05e3ff974a161d8b6ba8e5d93f612e11ab1c1b3f156ce"),
    },
    StockRelease {
        name: "TO7-DM02-K260203-V6510C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("7a75ab23c82fe918bc8086ec599cf2b1a9f976d66ab62daf5fce095b5f1b9035"),
    },
    StockRelease {
        name: "TO7-DM02-K260526-V6510C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("3ab150d10d3106080233d4f41e7df6a13f67154994138d831df67b1094ea1ca2"),
    },
    StockRelease {
        name: "TO7L-DM02-K240723A-V6306C",
        target: MotorTarget::Dm02,
        pack_sha256: hex("435aae3703971d92ee3a14342f0ae677491fb34fdc5f645bb919d41da40cb6a2"),
    },
];

/// Looks up a release by container hash.
pub fn identify(pack_sha256: &[u8; 32]) -> Option<&'static StockRelease> {
    KNOWN_RELEASES
        .iter()
        .find(|r| &r.pack_sha256 == pack_sha256)
}
