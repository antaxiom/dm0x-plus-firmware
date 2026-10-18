use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("thumbv7em") {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        fs::copy(manifest_dir.join("memory.ld"), out_dir.join("memory.ld")).unwrap();
        println!("cargo:rustc-link-search={}", out_dir.display());
        println!("cargo:rustc-link-arg=-Tmemory.ld");
    }
    println!("cargo:rerun-if-changed=memory.ld");
    println!("cargo:rerun-if-changed=build.rs");
}
