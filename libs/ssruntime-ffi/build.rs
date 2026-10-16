// build.rs

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to rerun this script if src directory changes
    println!("cargo:rerun-if-changed=../ssruntime/src");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let core_dir = PathBuf::from(&crate_dir).join("..").join("ssruntime");

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
    match cbindgen::Builder::new().with_crate(&core_dir).with_language(cbindgen::Language::C).with_cpp_compat(true).generate() {
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
        .input_extern_file("../ssruntime/src/ffi/mod.rs")
        .input_extern_file("../ssruntime/src/types.rs")
        .csharp_dll_name("ssruntime")
        .csharp_class_accessibility("public")
        .csharp_namespace("SpriteStudio.SsRuntime")
        .generate_csharp_file(csharp_output_path)
    {
        Ok(_) => {
            // Post-process the generated C# file to add Unity-specific platform macros for static linking (__Internal)
            let content = std::fs::read_to_string(csharp_output_path).expect("read the generated C# bindings");
            let dll_name = "const string __DllName = \"ssruntime\";";
            assert!(content.contains(dll_name), "the generated C# bindings no longer contain `{dll_name}`, so iOS, tvOS and WebGL would not get `__Internal`");
            let modified = content.replace(dll_name, "#if (UNITY_IOS || UNITY_TVOS || UNITY_WEBGL) && !UNITY_EDITOR\n        const string __DllName = \"__Internal\";\n#else\n        const string __DllName = \"ssruntime\";\n#endif");
            std::fs::write(csharp_output_path, modified).unwrap();
        }
        Err(e) => {
            eprintln!("Error generating C# bindings: {:?}", e);
            panic!("Unable to generate C# bindings: {}", e);
        }
    };
}
