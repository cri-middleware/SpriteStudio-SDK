// build.rs

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to rerun this script if src directory changes
    println!("cargo:rerun-if-changed=src");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Emit a plain C header (.h): the Flutter Body runs `dart run ffigen` over
    // it, and it is the module-map umbrella for the macOS/iOS framework.
    //
    // `cpp_compat` wraps it in `extern "C"`, so this one header serves C **and**
    // C++ — which the docs already assume (libs/ssruntime/README.md's "Example
    // C++ Usage" includes ssruntime.h, as does the Player guide). Without it a
    // C++ translation unit compiles the declarations with C++ linkage and fails
    // to link against the library's C symbols. ffigen is unaffected: it parses
    // in C mode, where `__cplusplus` is undefined and the guard disappears.
    //
    // This is the only header the SDK ships; no Player consumes a C++-only
    // variant.
    let c_output_file = PathBuf::from(&crate_dir).join("target").join("ssruntime.h");
    match cbindgen::Builder::new().with_crate(&crate_dir).with_language(cbindgen::Language::C).with_cpp_compat(true).generate() {
        Ok(bindings) => {
            bindings.write_to_file(&c_output_file);
        }
        Err(e) => {
            eprintln!("Error generating C bindings: {:?}", e);
            panic!("Unable to generate C bindings: {}", e);
        }
    };

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
                    .replace("const string __DllName = \"ssruntime\";", "#if (UNITY_IOS || UNITY_TVOS || UNITY_WEBGL) && !UNITY_EDITOR\n        const string __DllName = \"__Internal\";\n#else\n        const string __DllName = \"ssruntime\";\n#endif")
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
