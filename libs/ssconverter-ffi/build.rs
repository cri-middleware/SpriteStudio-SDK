use std::env;
use std::path::{Path, PathBuf};

/// The `//!` lines that open `path`, as a C block comment.
fn module_doc(path: &Path) -> String {
    let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut comment = String::from("/*\n");
    for line in source.lines().map_while(|line| line.strip_prefix("//!")) {
        comment.push_str(" *");
        comment.push_str(line);
        comment.push('\n');
    }
    comment.push_str(" */");
    comment
}

fn main() {
    println!("cargo:rerun-if-changed=../ssconverter/src");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let core_dir = PathBuf::from(&crate_dir).join("..").join("ssconverter");

    // The `ffi` module doc is written for C readers, so it opens the header.
    let c_output_file = PathBuf::from(&crate_dir).join("target").join("ssconverter.h");
    let preamble = module_doc(&core_dir.join("src").join("ffi").join("mod.rs"));
    match cbindgen::Builder::new().with_crate(&core_dir).with_language(cbindgen::Language::C).with_cpp_compat(true).with_header(preamble).with_include_guard("SSCONVERTER_H").generate() {
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
