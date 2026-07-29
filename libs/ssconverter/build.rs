// build.rs

use std::env;
use std::error::Error;
use std::path::PathBuf;
use vergen_gitcl::{Emitter, Gitcl};

fn main() -> Result<(), Box<dyn Error>> {
    // Tell cargo to rerun this script if src directory changes
    println!("cargo:rerun-if-changed=src");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Emit a plain C header (.h) for ffigen / C consumers. `cpp_compat` wraps it
    // in `extern "C"` so the same header links correctly from C++ as well;
    // ffigen parses in C mode, where the guard disappears. See
    // libs/ssruntime/build.rs for the full reasoning.
    let c_output_file = PathBuf::from(&crate_dir).join("target").join("ssconverter.h");
    match cbindgen::Builder::new().with_crate(&crate_dir).with_language(cbindgen::Language::C).with_cpp_compat(true).generate() {
        Ok(bindings) => {
            bindings.write_to_file(&c_output_file);
        }
        Err(e) => {
            eprintln!("Error generating C bindings: {:?}", e);
            panic!("Unable to generate C bindings: {}", e);
        }
    };

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
    let git = Gitcl::builder().sha(true).build();

    Emitter::default().add_instructions(&git)?.emit()?;

    Ok(())
}
