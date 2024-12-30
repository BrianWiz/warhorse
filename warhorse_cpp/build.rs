use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let cpp_project_dir = Path::new(&crate_dir)
        .parent()
        .unwrap()
        .join("warhorse_cpp_client")
        .join("src")
        .join("include");

    fs::create_dir_all(&cpp_project_dir).unwrap();

    let config = cbindgen::Config::from_file("cbindgen.toml")
        .expect("Unable to find cbindgen.toml configuration file");
    
    cbindgen::Builder::new()
        .with_crate(crate_dir.clone())
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(cpp_project_dir.join("bindings.h"));

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
}
