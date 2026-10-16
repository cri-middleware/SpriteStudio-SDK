//! The C header and the C# bindings that build.rs generates into `target/`,
//! held against the copies committed in `bindings/`, so that a change to the C
//! ABI shows in the pull request that makes it.

use std::path::Path;

const UPDATE: &str = "UPDATE_BINDINGS=1 cargo test -p ssconverter-ffi --test bindings";

fn check(file: &str) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let generated_path = dir.join("target").join(file);
    let committed_path = dir.join("bindings").join(file);
    let generated = std::fs::read_to_string(&generated_path).unwrap_or_else(|e| panic!("read {}: {e}", generated_path.display())).replace("\r\n", "\n");
    if std::env::var_os("UPDATE_BINDINGS").is_some() {
        std::fs::write(&committed_path, &generated).unwrap_or_else(|e| panic!("write {}: {e}", committed_path.display()));
        return;
    }
    let committed = std::fs::read_to_string(&committed_path).unwrap_or_default().replace("\r\n", "\n");
    if generated != committed {
        let line = generated.lines().zip(committed.lines()).position(|(a, b)| a != b).unwrap_or_else(|| generated.lines().count().min(committed.lines().count()));
        panic!("bindings/{file} differs from what build.rs generated, first at line {}. If the C ABI change is intended, run `{UPDATE}` and commit bindings/{file}.", line + 1);
    }
}

#[test]
fn the_committed_c_header_is_the_generated_one() {
    check("ssconverter.h");
}

#[test]
fn the_committed_csharp_bindings_are_the_generated_ones() {
    check("SSConverterMethods.g.cs");
}
