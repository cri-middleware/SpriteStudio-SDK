use std::collections::HashMap;

pub fn version_string() -> String {
    let sha = env!("VERGEN_GIT_SHA");
    let short_sha = if sha.len() >= 7 { &sha[..7] } else { sha };
    format!("{}+{}", env!("CARGO_PKG_VERSION"), short_sha)
}

/// `.ssab` format version emitted (distinct from build [`version_string`]).
pub fn ssab_schema_version() -> u32 {
    ssab::ssab_schema_version()
}

/// `.ssqb` format version emitted (distinct from build [`version_string`]).
pub fn ssqb_schema_version() -> u32 {
    ssab::ssqb_schema_version()
}

pub fn fnv1a_hash_str(text: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    let prime: u32 = 16777619;

    for b in text.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(prime);
    }
    hash
}

/// Hash a `TCHG` `TextureName`, mapping the authored "no override" to `0`.
///
/// An empty `TextureName` means "keep the cell's own texture". Hashing it verbatim would
/// yield the FNV offset basis (`2166136261`) — a value indistinguishable from a real texture
/// that no lookup table can resolve, so the Player would fail to bind a texture at all.
/// `0` is the schema default for `InitialPartAttributes.texture` and is reserved for this.
pub fn fnv1a_hash_texture_name(texture_name: &str) -> u32 {
    if texture_name.is_empty() {
        return 0;
    }
    fnv1a_hash_str(texture_name)
}

pub const ASSET_PATH_SEPARATOR: char = '/';

pub fn join_asset_path(base: &str, name: &str) -> String {
    let base = base.trim_end_matches(ASSET_PATH_SEPARATOR);
    if base.is_empty() {
        return name.to_string();
    }
    format!("{}{}{}", base, ASSET_PATH_SEPARATOR, name)
}

pub fn asset_path_parent(path: &str) -> Option<&str> {
    path.rsplit_once(ASSET_PATH_SEPARATOR).map(|(parent, _)| parent)
}

/// File extension used by sspj cellmap entries.
pub const CELLMAP_FILE_EXT: &str = ".ssce";

/// Strip the `.ssce` suffix from a cellmap file name. Inputs without the
/// suffix are returned unchanged so callers can pass either form.
pub fn cellmap_basename(file_name: &str) -> &str {
    file_name.strip_suffix(CELLMAP_FILE_EXT).unwrap_or(file_name)
}

#[cfg(feature = "native")]
fn resolve_within(base: &std::path::Path, path: &std::path::Path) -> Result<std::path::PathBuf, std::io::Error> {
    let base = if base.as_os_str().is_empty() { std::path::Path::new(".") } else { base };
    let base = base.canonicalize()?;
    let resolved = path.canonicalize()?;
    if resolved.starts_with(&base) { Ok(resolved) } else { Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "outside the base directory")) }
}

#[cfg(feature = "native")]
pub fn validate_file_path(file: &str, dir: &std::path::Path) -> Result<(), crate::ConverterError> {
    let file_path_buf = std::path::PathBuf::from(file);
    if file_path_buf.is_absolute() {
        return Err(crate::ConverterError::ConversionFailed(format!("The file path is absolute path. The converter doesn't support absolute path: {}", file)));
    }

    let path_buf = std::path::PathBuf::from(dir).join(file);

    if !path_buf.as_path().exists() {
        return Err(crate::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("The file is not found: {}", file))));
    }

    if !path_buf.is_file() {
        return Err(crate::ConverterError::ConversionFailed(format!("The file is not file: {}", file)));
    }

    if resolve_within(dir, &path_buf).is_err() {
        return Err(crate::ConverterError::ConversionFailed(format!("The file resolves outside the project directory: {}", file)));
    }

    Ok(())
}

#[cfg(feature = "native")]
pub fn get_ssab_output_directory(sspj_path: &std::path::Path, embedded: bool) -> Result<std::path::PathBuf, crate::ConverterError> {
    let suffix = if embedded { "_emb" } else { "" };
    let Some(stem) = sspj_path.file_stem() else {
        return Err(crate::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("not found: path stem: {}", sspj_path.as_os_str().to_string_lossy()))));
    };
    let Some(sspj_parent) = sspj_path.parent() else {
        return Err(crate::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("not found: path parent: {}", sspj_path.as_os_str().to_string_lossy()))));
    };
    Ok(sspj_parent.join(format!("{}{}", stem.to_string_lossy(), suffix)))
}

pub fn get_assets_of_ssab_object_class(ssab_obj: &ssab::ssab::ss::format::SsAnimeBinaryT, sspj: &sspj::sspj::SsProject) -> HashMap<String, String> {
    let images = ssab_obj
        .cellmaps
        .iter()
        .flat_map(|cellmaps| cellmaps.iter())
        .map(|cellmap| {
            let s = &cellmap.image_path;
            (s.to_string(), std::path::PathBuf::from(&sspj.settings.image_base_directory).join(s).to_string_lossy().to_string())
        })
        .collect::<HashMap<_, _>>();

    let external_textures = ssab_obj
        .external_textures
        .iter()
        .flat_map(|external_textures| external_textures.iter())
        .flat_map(|external_texture| external_texture.name.as_ref().map(|s| (s.to_string(), std::path::PathBuf::from(&sspj.settings.image_base_directory).join(s).to_string_lossy().to_string())))
        .collect::<HashMap<String, String>>();

    let sounds = ssab_obj
        .sound_lists
        .iter()
        .flat_map(|soundlist| soundlist.iter())
        .flat_map(|soundlist| {
            soundlist.table_data.iter().flatten().filter_map(|file| {
                let file_path = file.file_path.as_ref()?;
                Some((file_path.to_string(), file_path.to_string()))
            })
        })
        .collect::<HashMap<String, String>>();

    let fonts = ssab_obj
        .font_bitmaps
        .iter()
        .flat_map(|font_bitmaps| font_bitmaps.iter())
        .flat_map(|font_bitmap| font_bitmap.name_texture.iter().flatten().map(|name_texture| (name_texture.to_string(), name_texture.to_string())))
        .collect::<HashMap<String, String>>();

    let mut output = HashMap::<String, String>::new();
    output.extend(images);
    output.extend(external_textures);
    output.extend(sounds);
    output.extend(fonts);
    output
}

pub fn get_assets_of_ssab_file(ssab_pair: (&String, &[u8]), sspj: &sspj::sspj::SsProject) -> HashMap<String, String> {
    let ssab_data = ssab_pair.1;
    let anime = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_data) };

    let images = anime
        .cellmaps()
        .iter()
        .flatten()
        .map(|cellmap| {
            let s = cellmap.image_path();
            (s.to_string(), std::path::PathBuf::from(&sspj.settings.image_base_directory).join(s).to_string_lossy().to_string())
        })
        .collect::<HashMap<_, _>>();

    let external_textures = anime
        .external_textures()
        .iter()
        .flatten()
        .filter_map(|external_texture| {
            let s = external_texture.name()?;
            Some((s.to_string(), std::path::PathBuf::from(&sspj.settings.image_base_directory).join(s).to_string_lossy().to_string()))
        })
        .collect::<HashMap<_, _>>();

    let sounds = anime
        .sound_lists()
        .iter()
        .flatten()
        .flat_map(|soundlist| {
            soundlist.table_data().into_iter().flatten().filter_map(|item| {
                let file_path = item.file_path()?;
                Some((file_path.to_string(), file_path.to_string()))
            })
        })
        .collect::<HashMap<_, _>>();

    let fonts = anime.font_bitmaps().iter().flatten().flat_map(|font_bitmap| font_bitmap.name_texture().into_iter().flatten().map(|name_texture| (name_texture.to_string(), name_texture.to_string()))).collect::<HashMap<_, _>>();

    let mut output = HashMap::<String, String>::new();
    output.extend(images);
    output.extend(external_textures);
    output.extend(sounds);
    output.extend(fonts);
    output
}

#[cfg(any(feature = "native", feature = "web"))]
pub fn get_assets_of_ssab_files(ssab_map: &HashMap<String, Vec<u8>>, sspj: &sspj::sspj::SsProject) -> HashMap<String, String> {
    ssab_map.iter().flat_map(|(key, value)| self::get_assets_of_ssab_file((key, value), sspj).into_iter().map(|(k, v)| (k.clone(), v.clone()))).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn join_asset_path_always_uses_forward_slash() {
        assert_eq!(join_asset_path("images", "chara.png"), "images/chara.png");
        assert_eq!(join_asset_path("font/nested", "mplus_0.png"), "font/nested/mplus_0.png");
        // An empty base (a file sitting at the project root) must not introduce a leading separator.
        assert_eq!(join_asset_path("", "chara.png"), "chara.png");
        // A base already carrying a trailing separator must not double it.
        assert_eq!(join_asset_path("images/", "chara.png"), "images/chara.png");
    }

    #[test]
    fn asset_path_parent_splits_on_forward_slash_only() {
        assert_eq!(asset_path_parent("font/RoundedMPlus.fnt"), Some("font"));
        assert_eq!(asset_path_parent("font/nested/RoundedMPlus.fnt"), Some("font/nested"));
        assert_eq!(asset_path_parent("RoundedMPlus.fnt"), None);
    }

    #[cfg(feature = "native")]
    fn scratch_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ssconverter-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("Failed to create the scratch directory");
        dir
    }

    #[test]
    #[cfg(feature = "native")]
    fn a_path_that_leaves_the_project_directory_is_rejected() {
        let dir = scratch_dir("leaves");
        let project = dir.join("project");
        std::fs::create_dir_all(project.join("nested")).expect("Failed to create the project directory");
        std::fs::write(project.join("kept.png"), b"x").expect("Failed to write the asset");
        let outside = dir.join("outside");
        std::fs::create_dir_all(&outside).expect("Failed to create the outside directory");
        std::fs::write(outside.join("escaped.png"), b"x").expect("Failed to write the asset");

        assert!(validate_file_path("../outside/escaped.png", &project).is_err());
        assert!(validate_file_path("nested/../../outside/escaped.png", &project).is_err());

        assert!(validate_file_path("kept.png", &project).is_ok());
        assert!(validate_file_path("./kept.png", &project).is_ok());
        assert!(validate_file_path("nested/../kept.png", &project).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[cfg(all(unix, feature = "native"))]
    fn a_symlink_that_stays_inside_the_project_is_accepted() {
        let dir = scratch_dir("symlink-inside");
        let project = dir.join("project");
        std::fs::create_dir_all(project.join("nested")).expect("Failed to create the project directory");
        std::fs::write(project.join("real.png"), b"x").expect("Failed to write the asset");
        std::os::unix::fs::symlink("real.png", project.join("link.png")).expect("Failed to create the symlink");
        std::os::unix::fs::symlink("../real.png", project.join("nested").join("deep.png")).expect("Failed to create the symlink");

        assert!(validate_file_path("link.png", &project).is_ok());
        assert!(validate_file_path("nested/deep.png", &project).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[cfg(all(unix, feature = "native"))]
    fn a_symlink_that_points_outside_the_project_is_rejected() {
        let dir = scratch_dir("symlink-outside");
        let project = dir.join("project");
        std::fs::create_dir_all(&project).expect("Failed to create the project directory");
        let outside = dir.join("outside");
        std::fs::create_dir_all(&outside).expect("Failed to create the outside directory");
        std::fs::write(outside.join("escaped.png"), b"x").expect("Failed to write the asset");
        std::os::unix::fs::symlink("../outside/escaped.png", project.join("link.png")).expect("Failed to create the symlink");

        assert!(validate_file_path("link.png", &project).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[cfg(feature = "native")]
    fn an_absolute_path_is_rejected() {
        let dir = scratch_dir("absolute");
        let asset = dir.join("real.png");
        std::fs::write(&asset, b"x").expect("Failed to write the asset");

        assert!(validate_file_path(asset.to_str().expect("Failed to render the path"), &dir).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
