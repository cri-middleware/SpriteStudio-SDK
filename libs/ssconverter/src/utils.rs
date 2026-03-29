use std::collections::HashMap;

pub fn version_string() -> String {
    let sha = env!("VERGEN_GIT_SHA");
    let short_sha = if sha.len() >= 7 { &sha[..7] } else { sha };
    format!("{}+{}", env!("CARGO_PKG_VERSION"), short_sha)
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

pub fn get_sspj_image_dir_path(sspj_parent: &std::path::Path, sspj: &sspj::sspj::SsProject) -> std::path::PathBuf {
    std::path::PathBuf::from(sspj_parent).join(sspj.settings.image_base_directory.clone())
}

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
    if path_buf.is_symlink() {
        return Err(crate::ConverterError::ConversionFailed(format!("The file is symlink: {}", file)));
    }

    Ok(())
}

pub fn get_ssab_outout_directory(sspj_path: &std::path::Path, embedded: bool) -> Result<std::path::PathBuf, crate::ConverterError> {
    let suffix = if embedded { "_emb_ssab" } else { "_ssab" };
    let Some(stem) = sspj_path.file_stem() else {
        return Err(crate::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("not found: path stem: {}", sspj_path.as_os_str().to_string_lossy()))))
    };
    let Some(sspj_parent) = sspj_path.parent() else {
        return Err(crate::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("not found: path parent: {}", sspj_path.as_os_str().to_string_lossy()))))
    };
    Ok(sspj_parent.join(format!("{}{}", stem.to_string_lossy(), suffix)))
}

pub fn get_assets_of_ssab_object_class(ssab_obj: &ssab::ssab::ss::format::SsAnimeBinaryT, sspj: &sspj::sspj::SsProject) -> HashMap<String, String> {

    let images = ssab_obj.cellmaps.iter().flat_map(|cellmaps| cellmaps.iter()).flat_map(|cellmap|
        cellmap.image_path.as_ref().map(|s| (s.to_string(), std::path::PathBuf::from(&sspj.settings.image_base_directory).join(s).to_string_lossy().to_string()))
    ).collect::<HashMap<_, _>>();

    let external_textures = ssab_obj.external_textures.iter().flat_map(|external_textures| external_textures.iter()).flat_map(|external_texture|
        external_texture.name.as_ref().map(|s| (s.to_string(), std::path::PathBuf::from(&sspj.settings.image_base_directory).join(s).to_string_lossy().to_string()))
    ).collect::<HashMap<String, String>>();

    let sounds = ssab_obj.sound_lists.iter().flat_map(|soundlist| soundlist.iter()).flat_map(|soundlist| {
        soundlist.table_data.iter().flatten().filter_map(|file| {
            let file_path = file.file_path.as_ref()?;
            Some((file_path.to_string(), file_path.to_string()))
        })
    }).collect::<HashMap<String, String>>();

    // TODO: improve font bitmap path
    /*
    let fonts =  anime.font_bitmaps().into_iter().flatten().flat_map(|font_bitmap| {
        font_bitmap.name_texture().into_iter().flatten().map(|name_texture| name_texture.to_string())
    });
     */

    let mut output = HashMap::<String, String>::new();
    output.extend(images);
    output.extend(external_textures);
    output.extend(sounds);
    output
}

pub fn get_assets_of_ssab_file(ssab_pair: (&String, &[u8]), sspj: &sspj::sspj::SsProject) -> HashMap<String, String> {
    let ssab_data = ssab_pair.1;
    let anime = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_data) };

    let images = anime.cellmaps().iter().flatten().flat_map(|cellmap| cellmap.image_path().map(|s| {
        (s.to_string(), std::path::PathBuf::from(&sspj.settings.image_base_directory).join(s).to_string_lossy().to_string())
    })).collect::<HashMap<_, _>>();

    let external_textures = anime.external_textures().iter().flatten().filter_map(|external_texture| {
        let s = external_texture.name()?;
        Some((s.to_string(), std::path::PathBuf::from(&sspj.settings.image_base_directory).join(s).to_string_lossy().to_string()))
    }).collect::<HashMap<_, _>>();

    let sounds = anime.sound_lists().iter().flatten().flat_map(|soundlist|
        soundlist.table_data().into_iter().flatten().filter_map(|item| {
            let file_path = item.file_path()?;
            Some((file_path.to_string(), file_path.to_string()))
        })).collect::<HashMap<_, _>>();

    // TODO: improve font bitmap path
    /*
    let fonts = anime.font_bitmaps().into_iter().flatten().flat_map(|font_bitmap| {
        font_bitmap.name_texture().into_iter().flatten().map(|name_texture| name_texture.to_string())
    });
     */

    let mut output = HashMap::<String, String>::new();
    output.extend(images);
    output.extend(external_textures);
    output.extend(sounds);
    output
}

pub fn get_assets_of_ssab_files(ssab_map: &HashMap<String, Vec<u8>>, sspj: &sspj::sspj::SsProject) -> HashMap<String, String> {
    ssab_map.iter().flat_map(|(key, value)| {
        self::get_assets_of_ssab_file((key, value), sspj).into_iter().map(|(k, v)| (k.clone(), v.clone()))
    }).collect()
}


#[cfg(test)]
mod test {
    
}