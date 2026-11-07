//! WASM bindings for the converter (browser, `wasm32-unknown-unknown`).
//!
//! The host (JS) owns all file I/O: it reads the project directory — via the
//! File System Access API, `<input webkitdirectory>`, or drag-and-drop — and
//! passes the files across as a `Map<string, Uint8Array>` of `path -> bytes`.
//! Conversion runs sequentially (rayon cannot spawn threads here) and returns
//! the generated `.ssab` / `.ssqb` files in memory. For non-embedded output
//! the host also receives the list of referenced source assets (images) so it
//! can write them alongside the output.

use crate::asset_provider::MemoryAssetProvider;
use wasm_bindgen::prelude::*;

/// How a failed conversion reaches JS: a JS `Error` with the converter's own
/// wording as its message.
///
/// `Display`, not `Debug`. `Debug` prints the Rust variant name, so a JS caller
/// was handed `ConversionFailed("Asset not found: x.png")` — an identifier from
/// a language they are not writing, which reaches their user through whatever
/// the host shows. `ssruntime`'s wasm face reports the same way.
fn js_error(e: crate::ConverterError) -> JsValue {
    js_message(&e.to_string())
}

/// An `Error` rather than a bare string, so that whoever catches it finds `message`
/// and a stack.
fn js_message(message: &str) -> JsValue {
    JsError::new(message).into()
}

/// Converter version string (`CARGO_PKG_VERSION` + short git sha).
#[wasm_bindgen]
pub fn version() -> String {
    crate::version_string()
}

/// `.ssab` schema version emitted (distinct from the build's `version`).
#[wasm_bindgen]
pub fn ssab_schema_version() -> u32 {
    crate::ssab_schema_version()
}

/// `.ssqb` schema version emitted (distinct from the build's `version`).
#[wasm_bindgen]
pub fn ssqb_schema_version() -> u32 {
    crate::ssqb_schema_version()
}

/// Result of `convert`.
#[wasm_bindgen]
pub struct ConvertResult {
    output: js_sys::Map,
    assets: js_sys::Map,
}

#[wasm_bindgen]
impl ConvertResult {
    /// `Map<string, Uint8Array>`: generated file name -> binary contents.
    #[wasm_bindgen(getter)]
    pub fn output(&self) -> js_sys::Map {
        self.output.clone()
    }

    /// `Map<string, string>`: destination path -> source asset path (relative
    /// to the `.sspj` directory) for non-embedded output. Empty when
    /// `embedded` is true. The host copies each source into the output.
    #[wasm_bindgen(getter)]
    pub fn assets(&self) -> js_sys::Map {
        self.assets.clone()
    }
}

/// Convert a SpriteStudio project to `.ssab`.
///
/// `files` is a `Map<string, Uint8Array>` of `path -> bytes` covering the
/// whole project directory. It must include exactly one `.sspj`: a project
/// folder that also holds another one, a backup beside it or a project in a
/// subfolder, is refused, so leave the others out. `embedded`, `debug`,
/// `split` and `optimize` mirror `ConvertConfig`.
#[wasm_bindgen]
pub fn convert(files: &js_sys::Map, embedded: bool, debug: bool, split: bool, optimize: bool) -> Result<ConvertResult, JsValue> {
    let mut collected: Vec<(String, Vec<u8>)> = Vec::new();
    let entries = files.entries();
    loop {
        let next = entries.next()?;
        if next.done() {
            break;
        }
        let pair: js_sys::Array = next.value().dyn_into()?;
        let key = pair.get(0).as_string().ok_or_else(|| js_message("file map keys must be strings"))?;
        let value: js_sys::Uint8Array = pair.get(1).dyn_into().map_err(|_| js_message(&format!("file map value for '{}' must be a Uint8Array", key)))?;
        collected.push((key, value.to_vec()));
    }

    let provider = MemoryAssetProvider::from_files(collected).map_err(js_error)?;

    let config = crate::ConvertConfig { embedded, debug, split, optimize };
    let (sspj, output_map, _) = crate::convert_sspj_to_ssab_data(&provider, &config, &|_: &str| {}).map_err(js_error)?;

    let output = js_sys::Map::new();
    for (name, data) in &output_map {
        let arr = js_sys::Uint8Array::from(data.as_slice());
        output.set(&JsValue::from_str(name), &JsValue::from(arr));
    }

    let assets = js_sys::Map::new();
    if !embedded {
        for (dst, src) in crate::utils::get_assets_of_ssab_files(&output_map, &sspj) {
            assets.set(&JsValue::from_str(&dst), &JsValue::from_str(&src));
        }
    }

    Ok(ConvertResult { output, assets })
}

#[cfg(test)]
mod tests {
    /// The wasm face reports through `js_error`, never by formatting an error
    /// itself. `convert` needs a JS host to run, so what is pinned is the
    /// source: no `Debug` formatting left in it.
    #[test]
    fn the_wasm_face_does_not_debug_format_an_error() {
        let web = std::fs::read_to_string("src/web/mod.rs").expect("read web/mod.rs");
        assert!(web.contains("fn js_error("), "js_error is gone; this test is pinning nothing");
        assert!(!web.contains(concat!("{", ":?}")), "src/web/mod.rs formats something with Debug. A failed conversion reports through `js_error`.");
    }

    /// A JS exception is an `Error`, so that whoever catches it finds `message` and a
    /// stack. The names in the result's maps are the only plain strings on this face.
    #[test]
    fn the_wasm_face_throws_errors_not_strings() {
        let web = std::fs::read_to_string("src/web/mod.rs").expect("read web/mod.rs");
        assert!(web.contains(concat!("JsError", "::new")), "js_message is gone; this test is pinning nothing");
        for line in web.lines().filter(|line| line.contains(concat!("JsValue::", "from_str"))) {
            assert!(line.contains(".set("), "src/web/mod.rs builds a plain string for something other than a map entry: {line}");
        }
    }
}
