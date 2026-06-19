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

/// Converter version string (`CARGO_PKG_VERSION` + short git sha).
#[wasm_bindgen]
pub fn version() -> String {
    crate::version_string()
}

/// `.ssab` schema version emitted (distinct from build [`version`]).
#[wasm_bindgen]
pub fn ssab_schema_version() -> u32 {
    crate::ssab_schema_version()
}

/// `.ssqb` schema version emitted (distinct from build [`version`]).
#[wasm_bindgen]
pub fn ssqb_schema_version() -> u32 {
    crate::ssqb_schema_version()
}

/// Result of [`convert`].
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
/// whole project directory (it must include the `.sspj`). `embedded`, `debug`,
/// `split` and `optimize` mirror [`crate::ConvertConfig`].
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
        let key = pair.get(0).as_string().ok_or_else(|| JsValue::from_str("file map keys must be strings"))?;
        let value: js_sys::Uint8Array = pair.get(1).dyn_into().map_err(|_| JsValue::from_str(&format!("file map value for '{}' must be a Uint8Array", key)))?;
        collected.push((key, value.to_vec()));
    }

    let provider = MemoryAssetProvider::from_files(collected).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

    let config = crate::ConvertConfig { embedded, debug, split, optimize };
    let (sspj, output_map) = crate::convert_sspj_to_ssab_data(&provider, &config, &|_: &str| {}).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

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
