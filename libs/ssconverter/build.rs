// build.rs

use std::env;
use std::error::Error;
use std::path::PathBuf;
use vergen_gitcl::{Emitter, GitclBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    // Tell cargo to rerun this script if src directory changes
    println!("cargo:rerun-if-changed=src");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = PathBuf::from(&crate_dir).join("target").join("ssconverter.h");

    let config = match cbindgen::Builder::new().with_crate(&crate_dir).with_language(cbindgen::Language::Cxx).generate() {
        Ok(bindings) => bindings,
        Err(e) => {
            eprintln!("Error generating C++ bindings: {:?}", e);
            panic!("Unable to generate bindings: {}", e);
        }
    };

    config.write_to_file(output_file);

    let csharp_output_path = "target/SSConverterMethods.g.cs";
    match csbindgen::Builder::default().input_extern_file("src/ffi/mod.rs").csharp_dll_name("ssconverter").csharp_class_accessibility("public").csharp_namespace("SpriteStudio.SsConverter").generate_csharp_file(csharp_output_path) {
        Ok(_) => {
            if let Ok(content) = std::fs::read_to_string(csharp_output_path) {
                let modified = content.replace("namespace CsBindgen", "namespace SpriteStudio.SsConverter");
                std::fs::write(csharp_output_path, modified).unwrap();
            }
        }
        Err(e) => {
            eprintln!("Error generating C# bindings: {:?}", e);
            panic!("Unable to generate C# bindings: {}", e);
        }
    };

    // get commit hash
    let git = GitclBuilder::default().sha(true).build()?;

    Emitter::default().add_instructions(&git)?.emit()?;

    Ok(())
}
