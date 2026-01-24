// build.rs

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to rerun this script if src/lib.rs changes
    println!("cargo:rerun-if-changed=src/lib.rs");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .join("target")
        .join("ssruntime.h");

    let config = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_language(cbindgen::Language::Cxx)
        .generate()
        .expect("Unable to generate bindings");

    config.write_to_file(output_file);

    csbindgen::Builder::default()
        .input_extern_file("src/ffi/mod.rs")
        .csharp_dll_name("ssruntime")
        .generate_csharp_file("target/SSRuntimeMethods.g.cs")
        .unwrap();
}
