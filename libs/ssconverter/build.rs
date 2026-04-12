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

    // get commit hash
    let git = GitclBuilder::default().sha(true).build()?;

    Emitter::default().add_instructions(&git)?.emit()?;

    Ok(())
}
