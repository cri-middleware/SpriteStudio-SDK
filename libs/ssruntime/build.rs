// build.rs

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to rerun this script if src directory changes
    println!("cargo:rerun-if-changed=src");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = PathBuf::from(&crate_dir).join("target").join("ssruntime.h");

    let config = match cbindgen::Builder::new().with_crate(&crate_dir).with_language(cbindgen::Language::Cxx).generate() {
        Ok(bindings) => bindings,
        Err(e) => {
            eprintln!("Error generating C++ bindings: {:?}", e);
            panic!("Unable to generate bindings: {}", e);
        }
    };

    config.write_to_file(output_file);

    match csbindgen::Builder::default().input_extern_file("src/ffi/mod.rs").input_extern_file("src/ffi/types.rs").csharp_dll_name("ssruntime").generate_csharp_file("target/SSRuntimeMethods.g.cs") {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error generating C# bindings: {:?}", e);
            panic!("Unable to generate C# bindings: {}", e);
        }
    };
}
