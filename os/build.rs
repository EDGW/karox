use std::{collections::HashMap, env, fs, path::PathBuf, u64};

fn parse_hex(hex_str: &str) -> Result<u64, std::num::ParseIntError> {
    let clean_hex = hex_str.trim().trim_start_matches("0x").replace("_", "");
    u64::from_str_radix(clean_hex.as_str(), 16)
}
fn to_hex(hex_str: u64) -> String {
    format!("{:#x}", hex_str)
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let flags_str =
        fs::read_to_string(PathBuf::from(manifest_dir.clone()).join("link_flags.json")).unwrap();
    let flagmap: HashMap<String, HashMap<String, String>> =
        serde_json::from_str(&flags_str).unwrap();
    let flags = match flagmap.get(target_arch.as_str()) {
        Some(value) => value,
        None => panic!("Unknown Architecture."),
    };
    let entry = parse_hex(flags.get("KERNEL_ENTRY_ADDR").unwrap()).unwrap();
    let space = parse_hex(flags.get("KERNEL_SPACE_OFFSET").unwrap()).unwrap();
    make_linker(entry, space);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=linker.cross.ld");
    println!("cargo:rerun-if-changed=../runtime/qemu-loongarch64.dtb");
    println!("cargo:rerun-if-changed=link_flags.json");
}

fn make_linker(entry: u64, space: u64) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let path = PathBuf::from(out_dir.clone()).join("linker.ld");
    let mut sout =
        fs::read_to_string(PathBuf::from(manifest_dir.clone()).join("linker.cross.ld")).unwrap();
    sout = sout.replace("%ARCH%", target_arch.as_str());
    sout = sout.replace("%RAM_START%", to_hex(entry).as_str());
    sout = sout.replace("%VIRT_START%", to_hex(space + entry).as_str());
    sout = sout.replace("%KERNEL_ENTRY_ADDR%", to_hex(entry).as_str());
    fs::write(&path, sout).unwrap();
    println!("cargo:rustc-link-arg=-T{}", path.display());
}
