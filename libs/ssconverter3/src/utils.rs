
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
        return Err(crate::ConverterError::ConversionFailed(format!("The file is not found: {}", file)));
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
    Ok(sspj_parent.join(format!("{}{}", stem.to_str().unwrap(), suffix)))
}

#[cfg(test)]
mod test {
    
}