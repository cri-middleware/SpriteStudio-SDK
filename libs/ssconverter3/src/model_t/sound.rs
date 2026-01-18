fn convert_sound_file_list(_sspj: &sspj::sspj::SsProject, sssound_list: &sspj::ssse::SsSoundFileList, sspj_parent_path: &std::path::Path) -> Result<Vec<ssab::ssab::SoundFileT>, crate::ConverterError> {
    let mut v = sssound_list.sounds.iter().map(|sssound| {
        let mut sound_file_t = ssab::ssab::SoundFileT::default();

        sound_file_t.name_hash = crate::utils::fnv1a_hash_str(&sssound.filename);
        sound_file_t.name = if sssound.filename.is_empty() { None } else { Some(sssound.filename.to_string()) };
        sound_file_t.file_path = if sssound.filename.is_empty() { None } else {
            let r = crate::utils::validate_file_path(&sssound.filename, sspj_parent_path);
            let filename = match r {
                Ok(_) => sssound.filename.to_string(),
                Err(e) => return Err(e),
            };
            Some(filename)
        };
        sound_file_t.time_total = sssound.msec;
        Ok(sound_file_t)
    }).collect::<Result<Vec<ssab::ssab::SoundFileT>, _>>()?;
    v.sort_by_key(|e| e.name_hash);
    Ok(v)
}

pub fn convert_sound_list(sspj: &sspj::sspj::SsProject, sound_list: &[&sspj::ssse::SsAudioPack], sspj_parent_path: &std::path::Path) -> Result<Vec<ssab::ssab::SoundListT>, crate::ConverterError> {
    let mut v = sound_list.iter().map(|sssound_pack| {
        let mut sound_list_t = ssab::ssab::SoundListT::default();

        sound_list_t.name_hash = crate::utils::fnv1a_hash_str(&sssound_pack.name);
        sound_list_t.name = if sssound_pack.name.is_empty() { None } else { Some(sssound_pack.name.to_string()) };
        let r = convert_sound_file_list(sspj, &sssound_pack.table, sspj_parent_path);
        let table_data = match r {
            Ok(table_data) => table_data,
            Err(e) => return Err(e),
        };
        sound_list_t.table_data = Some(table_data);

        Ok(sound_list_t)
    }).collect::<Result<Vec<ssab::ssab::SoundListT>, _>>()?;
    v.sort_by_key(|e| e.name_hash);
    Ok(v)
}
