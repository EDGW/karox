
use std::{collections::HashMap, env, fs, path::PathBuf};

fn main(){

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let flags_str = fs::read_to_string(PathBuf::from(manifest_dir.clone()).join("../../flags.json")).unwrap();
    let flagmap: HashMap<String,HashMap<String,String>> = serde_json::from_str(&flags_str).unwrap();
    let flags = match flagmap.get(target_arch.as_str()){
        Some(value) => value,
        None => panic!("Unknown Architecture.")
    };
    make_flags(flags);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../flags.json");
}


fn make_flags(flagmap: &HashMap<String,String>){
    let mut s: String = String::from("");
    s+="#![allow(dead_code)]\n#![allow(missing_docs)]\n";
    for key in flagmap.keys() {
        s += format!("pub const {}:usize       = {};\n",key,flagmap.get(key).unwrap()).as_str();
    }
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir_path = PathBuf::from(manifest_dir.clone()).join("src");
    if !fs::exists(&dir_path).unwrap()
    {
        fs::create_dir(&dir_path).unwrap();
    }
    let path = PathBuf::from(manifest_dir.clone()).join("src/build_flags.rs");
    fs::write(path,s).unwrap();
}