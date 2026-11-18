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
    println!("cargo:rerun-if-changed=../ssruntime/src");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let core_dir = PathBuf::from(&crate_dir).join("..").join("ssruntime");

    // A plain C header: ffigen runs over it, and it is the module-map umbrella for
    // the macOS/iOS framework. `cpp_compat` wraps it in `extern "C"`, so this one
    // header serves C++ as well; ffigen parses in C mode, where the guard disappears.
    //
    // The `ffi` module doc is written for C readers and states the contract
    // every entry point relies on, so it opens the header.
    let c_output_file = PathBuf::from(&crate_dir).join("target").join("ssruntime.h");
    let preamble = module_doc(&core_dir.join("src").join("ffi").join("mod.rs"));
    match cbindgen::Builder::new()
        .with_crate(&core_dir)
        .with_language(cbindgen::Language::C)
        .with_cpp_compat(true)
        .with_header(preamble)
        .with_include_guard("SSRUNTIME_H")
        // Public in Rust, but internal to the evaluation: no Player needs them.
        .exclude_item("SKEW_PROPAGATES")
        .exclude_item("SEED_MAGIC")
        .exclude_item("LIFE_EXTEND_SCALE")
        .exclude_item("LIFE_EXTEND_MIN")
        .exclude_item("NINES_MAX_TILE_DIV")
        .exclude_item("NINES_MAX_TILE_QUADS")
        // Not part of the C-API: the no_std panic stub, and the allocator
        // symbols a host defines or the runtime imports from its libc.
        .exclude_item("rust_eh_personality")
        .exclude_item("em_malloc")
        .exclude_item("em_free")
        .exclude_item("em_posix_memalign")
        .exclude_item("ss_custom_alloc")
        .exclude_item("ss_custom_free")
        .generate()
    {
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
