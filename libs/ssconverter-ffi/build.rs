use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../ssconverter/src");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let core_dir = PathBuf::from(&crate_dir).join("..").join("ssconverter");

    let c_output_file = PathBuf::from(&crate_dir).join("target").join("ssconverter.h");
    match cbindgen::Builder::new().with_crate(&core_dir).with_language(cbindgen::Language::C).with_cpp_compat(true).generate() {
        Ok(bindings) => {
            bindings.write_to_file(&c_output_file);
        }
        Err(e) => {
            eprintln!("Error generating C bindings: {:?}", e);
            panic!("Unable to generate C bindings: {}", e);
        }
    };

    let csharp_output_path = "target/SSConverterMethods.g.cs";
    match csbindgen::Builder::default().input_extern_file("../ssconverter/src/ffi/mod.rs").csharp_dll_name("ssconverter").csharp_class_accessibility("public").csharp_namespace("SpriteStudio.SsConverter").generate_csharp_file(csharp_output_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error generating C# bindings: {:?}", e);
            panic!("Unable to generate C# bindings: {}", e);
        }
    };
}
