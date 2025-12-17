//! Generate stdlib.

use std::{env, fs, path::Path};

use glob::glob;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("stdlib.dr");

    let files = glob("./stdlib/**/*.dr").expect("Could not read stdlib");

    let output = files
        .into_iter()
        .map(|file| fs::read_to_string(file.unwrap()).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&dest_path, output).unwrap();

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=stdlib/");
}
