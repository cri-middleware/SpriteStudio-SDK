use crate::model_t;
use std::collections::{HashMap, HashSet};

/// The attribute converters size their per-part vectors to the length of the
/// name-keyed part map and then address them by `arrayIndex`, so the two have to
/// describe the same set. A duplicate name shortens the map while `arrayIndex`
/// keeps counting, and an out-of-range or repeated `arrayIndex` misses on its
/// own. Both are malformed input, and both index out of bounds several calls
/// deeper, where `panic = "abort"` would take the host process with them.
fn validate_part_list(anime_pack: &sspj::ssae::SsAnimePack) -> Result<(), crate::ConverterError> {
    let part_list = &anime_pack.model.part_list;
    let mut seen_names = HashSet::with_capacity(part_list.len());
    let mut seen_indices = HashSet::with_capacity(part_list.len());
    for part in part_list {
        if !seen_names.insert(part.name.as_str()) {
            return Err(crate::ConverterError::ConversionFailed(format!("{}: more than one part is named \"{}\"", anime_pack.name, part.name)));
        }
        if part.array_index < 0 || part.array_index as usize >= part_list.len() {
            return Err(crate::ConverterError::ConversionFailed(format!("{}: part \"{}\" has arrayIndex {}, outside 0..{}", anime_pack.name, part.name, part.array_index, part_list.len())));
        }
        if !seen_indices.insert(part.array_index) {
            return Err(crate::ConverterError::ConversionFailed(format!("{}: part \"{}\" reuses arrayIndex {}", anime_pack.name, part.name, part.array_index)));
        }
    }
    Ok(())
}

pub fn convert_ss_anime_binary_t(anime_pack: &sspj::ssae::SsAnimePack, sspj: &sspj::sspj::SsProject, asset_provider: &dyn crate::asset_provider::AssetProvider, embedded: bool) -> Result<ssab::ssab::ss::format::SsAnimeBinaryT, crate::ConverterError> {
    // effect
    let using_effect_file_names = anime_pack.model.part_list.iter().filter_map(|part| if part.part_type == sspj::sstypes::SsPartType::Effect { Some(part.ref_effect_name.to_string()) } else { None }).collect::<HashSet<String>>();
    let using_effect_file_list = using_effect_file_names.iter().filter_map(|using_effect_file_name| sspj.effect_file_list.iter().find(|effect_file| effect_file.name == *using_effect_file_name)).collect::<Vec<_>>();

    // cellmap
    let effects_use_cellmap_names = using_effect_file_list.iter().flat_map(|f| &f.effect_data.node_list).map(|n| &n.behavior.cell_map_name).filter(|name| !name.is_empty()).cloned().collect::<Vec<String>>();
    let cellmap_names = anime_pack.cellmap_names.iter().map(|cellmap_file| cellmap_file.to_string()).chain(effects_use_cellmap_names).collect::<HashSet<String>>();
    let cellmaps = cellmap_names
        .iter()
        .filter_map(|cellmap_name| {
            let basename = crate::utils::cellmap_basename(cellmap_name);
            sspj.cellmap_list.iter().find(|cellmap| cellmap.name == basename)
        })
        .collect::<Vec<_>>();

    // font_bitmap
    let using_font_bitmap_names = anime_pack.model.part_list.iter().filter(|part| part.part_type == sspj::sstypes::SsPartType::Text).filter_map(|part| if part.text_bitmap { Some(part.text_char_map.to_string()) } else { None }).collect::<HashSet<String>>();
    let using_fnt_bitmaps = using_font_bitmap_names
        .iter()
        .filter_map(|name| {
            let charmap = sspj.charmap_dic.get(name)?;
            Some((name.to_string(), charmap))
        })
        .collect::<HashMap<String, &sspj::sscharmap::SsCharMap>>();

    // sound list
    let using_sound_list =
        if anime_pack.model.part_list.iter().filter(|part| part.part_type == sspj::sstypes::SsPartType::Audio).collect::<Vec<&sspj::ssae::SsPart>>().is_empty() { vec![] } else { sspj.sound_list.iter().collect::<Vec<&sspj::ssse::SsAudioPack>>() };

    // Map .ssse <table><id> to fnv1a(SoundList.name) so PartAttributeAudio
    // events can be linked to a SoundList without storing the integer id.
    let sound_list_id_to_name_hash: HashMap<i32, u32> = sspj.sound_list.iter().map(|p| (p.table.id, crate::utils::fnv1a_hash_str(&p.name))).collect();

    // TCHG keys name a texture directly, so every distinct TextureName must be resolvable
    // hash -> name by the Player. The project's <ExternalTextures> list only covers textures
    // registered by the author, so harvest the attribute values too; otherwise a TCHG naming
    // a cellmap image (or any unregistered texture) leaves the Player with an opaque hash.
    let texchange_texture_names = anime_pack
        .anime_list
        .iter()
        .flat_map(|anime| anime.part_animes.iter())
        .flat_map(|part_anime| part_anime.attributes.iter())
        .filter_map(|attribute| if let sspj::ssattribute::SsAttributeKind::Texchange(v) = attribute { Some(v) } else { None })
        .flat_map(|texchange| texchange.keys.iter())
        // An empty TextureName is the authored "no override" and has no texture to resolve.
        .filter(|key| !key.value.texture_name.is_empty())
        .map(|key| key.value.texture_name.clone());
    let external_textures = sspj.external_textures.iter().map(|texture_info| texture_info.path_name.clone()).chain(texchange_texture_names).collect::<HashSet<String>>();

    let external_instances = anime_pack
        .model
        .part_list
        .iter()
        .filter(|part| part.part_type == sspj::sstypes::SsPartType::Instance)
        .filter(|part| part.ref_anime_pack != anime_pack.name)
        .map(|part| (part.ref_anime_pack.clone(), part.ref_anime.clone()))
        .collect::<HashSet<(String, String)>>();

    let mut anime_binary = ssab::ssab::ss::format::SsAnimeBinaryT::default();
    anime_binary.version = ssab::SSAB_SCHEMA_VERSION;
    anime_binary.name = anime_pack.name.to_string();
    anime_binary.name_hash = crate::utils::fnv1a_hash_str(&anime_pack.name);
    anime_binary.parts = model_t::animedata::convert_part_list(anime_pack);
    validate_part_list(anime_pack)?;
    let parts_map: HashMap<String, &sspj::ssae::SsPart> = anime_pack.model.part_list.iter().map(|part| (part.name.to_string(), part)).collect();
    let cellmap_lookup = model_t::animedata::CellmapResolver::new(&anime_pack.cellmap_names);
    anime_binary.animations = model_t::animedata::convert_animation_data_list(anime_pack, &parts_map, &sound_list_id_to_name_hash, &cellmap_lookup);
    cellmap_lookup.check(&anime_pack.name)?;
    anime_binary.cellmaps = if cellmaps.is_empty() { None } else { Some(model_t::cellmap::convert_cellmapt_list(sspj, &cellmaps, asset_provider)?) };
    anime_binary.effects = if using_effect_file_list.is_empty() { None } else { Some(model_t::effect::convert_effect_list(sspj, &using_effect_file_list)) };
    anime_binary.sound_lists = if using_sound_list.is_empty() { None } else { Some(model_t::sound::convert_sound_list(sspj, &using_sound_list, asset_provider)?) };
    anime_binary.font_bitmaps = if using_fnt_bitmaps.is_empty() { None } else { Some(model_t::font::convert_font_bitmaps(&using_fnt_bitmaps)) };
    anime_binary.external_textures = if external_textures.is_empty() {
        None
    } else {
        let mut v = external_textures
            .iter()
            .map(|texture_path| {
                let mut external_texture = ssab::ssab::ss::format::ExternalTextureT::default();
                external_texture.name_hash = crate::utils::fnv1a_hash_str(texture_path);
                external_texture.name = Some(texture_path.to_string());
                let r = asset_provider.validate_asset(std::path::PathBuf::from(&sspj.settings.image_base_directory).join(texture_path).to_str().unwrap_or(texture_path));
                match r {
                    Ok(_) => Ok(external_texture),
                    Err(e) => Err(e),
                }
            })
            .collect::<Result<Vec<ssab::ssab::ss::format::ExternalTextureT>, _>>()?;
        v.sort_by_key(|external_texture| external_texture.name_hash);
        Some(v)
    };
    anime_binary.external_instances = if external_instances.is_empty() {
        None
    } else {
        let mut v = external_instances
            .iter()
            .map(|(pack, anime)| {
                let mut entry = ssab::ssab::ss::format::ExternalInstanceT::default();
                entry.anime_pack_name_hash = crate::utils::fnv1a_hash_str(pack);
                entry.anime_pack_name = Some(pack.clone());
                entry.anime_name_hash = crate::utils::fnv1a_hash_str(anime);
                entry.anime_name = Some(anime.clone());
                entry
            })
            .collect::<Vec<_>>();
        v.sort_by_key(|e| (e.anime_pack_name_hash, e.anime_name_hash));
        Some(v)
    };
    anime_binary.embedded_assets = if embedded {
        let asset_files = crate::utils::get_assets_of_ssab_object_class(&anime_binary, sspj);
        let mut v = asset_files
            .iter()
            .map(|(name, path)| {
                let r = asset_provider.read_asset(path);
                let data = match r {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(crate::ConverterError::IoError(std::io::Error::new(e.kind(), format!("read: {}: {:?}", path, e))));
                    }
                };
                let mut embedded_asset = ssab::ssab::ss::format::EmbeddedAssetT::default();
                embedded_asset.name = Some(name.to_string());
                embedded_asset.name_hash = crate::utils::fnv1a_hash_str(name);
                embedded_asset.data = Some(data);

                Ok(embedded_asset)
            })
            .collect::<Result<Vec<ssab::ssab::ss::format::EmbeddedAssetT>, _>>()?;
        v.sort_by_key(|embedded_asset| embedded_asset.name_hash);
        Some(v)
    } else {
        None
    };

    Ok(anime_binary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sspj::ssloader::SsLoader;

    fn load_pack() -> sspj::ssae::SsAnimePack {
        let mut path = std::env::current_dir().unwrap();
        if path.ends_with("ssconverter") {
            path = path.parent().unwrap().parent().unwrap().to_path_buf();
        }
        path.push("tests/allPartsV7/normal.ssae");
        sspj::ssae::SsAnimePack::load_from_file(&path).expect("failed to load the fixture")
    }

    #[test]
    fn a_well_formed_part_list_validates() {
        validate_part_list(&load_pack()).expect("the fixture is well formed");
    }

    #[test]
    fn a_duplicate_part_name_is_an_error() {
        let mut pack = load_pack();
        let taken = pack.model.part_list[0].name.clone();
        pack.model.part_list[1].name = taken.clone();
        let e = validate_part_list(&pack).expect_err("a duplicate name shortens the map the converters index");
        assert!(format!("{e}").contains(&taken), "{e}");
    }

    #[test]
    fn an_out_of_range_array_index_is_an_error() {
        let mut pack = load_pack();
        pack.model.part_list[1].array_index = pack.model.part_list.len() as i32;
        validate_part_list(&pack).expect_err("an arrayIndex past the end indexes out of bounds");
    }

    #[test]
    fn a_negative_array_index_is_an_error() {
        let mut pack = load_pack();
        pack.model.part_list[1].array_index = -1;
        validate_part_list(&pack).expect_err("a negative arrayIndex wraps when cast to usize");
    }

    #[test]
    fn a_reused_array_index_is_an_error() {
        let mut pack = load_pack();
        pack.model.part_list[1].array_index = pack.model.part_list[0].array_index;
        validate_part_list(&pack).expect_err("a reused arrayIndex leaves another slot unwritten");
    }
}
