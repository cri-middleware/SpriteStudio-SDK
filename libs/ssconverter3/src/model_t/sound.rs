fn convert_sound_file_list(_sspj: &sspj::sspj::SsProject, sssound_list: &sspj::ssse::SsSoundFileList) -> Vec<ssab::ssab::SoundFileT> {
    let mut v = sssound_list.sounds.iter().map(|sssound| {
        let mut sound_file_t = ssab::ssab::SoundFileT::default();

        sound_file_t.name_hash = crate::utils::fnv1a_hash_str(&sssound.filename);
        sound_file_t.name = if sssound.filename.is_empty() { None } else { Some(sssound.filename.to_string()) };
        sound_file_t.file_path = if sssound.filename.is_empty() { None } else { Some(sssound.filename.to_string()) };
        sound_file_t.time_total = sssound.msec;

        sound_file_t
    }).collect::<Vec<ssab::ssab::SoundFileT>>();
    v.sort_by_key(|e| e.name_hash);
    v
}

pub fn convert_sound_list(sspj: &sspj::sspj::SsProject, sound_list: &[&sspj::ssse::SsAudioPack], ) -> Vec<ssab::ssab::SoundListT> {
    let mut v = sound_list.iter().map(|sssound_pack| {
        let mut sound_list_t = ssab::ssab::SoundListT::default();

        sound_list_t.name_hash = crate::utils::fnv1a_hash_str(&sssound_pack.name);
        sound_list_t.name = if sssound_pack.name.is_empty() { None } else { Some(sssound_pack.name.to_string()) };
        sound_list_t.table_data = Some(convert_sound_file_list(sspj, &sssound_pack.table));

        sound_list_t
    }).collect::<Vec<ssab::ssab::SoundListT>>();
    v.sort_by_key(|e| e.name_hash);
    v
}
