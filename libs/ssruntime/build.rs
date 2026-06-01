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

    let csharp_output_path = "target/SSRuntimeMethods.g.cs";
    match csbindgen::Builder::default()
        .input_extern_file("src/ffi/mod.rs")
        .input_extern_file("src/types.rs")
        .csharp_dll_name("ssruntime")
        .csharp_class_accessibility("public")
        .csharp_namespace("SpriteStudio.SsRuntime")
        .generate_csharp_file(csharp_output_path)
    {
        Ok(_) => {
            // Post-process the generated C# file to add Unity-specific platform macros for static linking (__Internal)
            if let Ok(content) = std::fs::read_to_string(csharp_output_path) {
                let modified = content
                    .replace("const string __DllName = \"ssruntime\";", "#if UNITY_IOS || UNITY_TVOS || UNITY_WEBGL\n        const string __DllName = \"__Internal\";\n#else\n        const string __DllName = \"ssruntime\";\n#endif")
                    .replace("namespace CsBindgen", "namespace SpriteStudio.SsRuntime");
                std::fs::write(csharp_output_path, modified).unwrap();
            }
        }
        Err(e) => {
            eprintln!("Error generating C# bindings: {:?}", e);
            panic!("Unable to generate C# bindings: {}", e);
        }
    };
}
