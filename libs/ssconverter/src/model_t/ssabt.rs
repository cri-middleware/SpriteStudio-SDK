use crate::model_t;
use std::collections::{HashMap, HashSet};

pub fn convert_ss_anime_binary_t(
    anime_pack: &sspj::ssae::SsAnimePack,
    sspj: &sspj::sspj::SsProject,
    asset_provider: &dyn crate::asset_provider::AssetProvider,
    embedded: bool,
) -> Result<ssab::ssab::ss::format::SsAnimeBinaryT, crate::ConverterError> {
    // effect
    let using_effect_file_names = anime_pack
        .model
        .part_list
        .iter()
        .filter_map(|part| if part.part_type == sspj::sstypes::SsPartType::Effect { Some(part.ref_effect_name.to_string()) } else { None })
        .collect::<HashSet<String>>();
    let using_effect_file_list =
        using_effect_file_names.iter().filter_map(|using_effect_file_name| sspj.effect_file_list.iter().find(|effect_file| effect_file.name == *using_effect_file_name)).collect::<Vec<_>>();

    // cellmap
    let effects_use_cellmap_names =
        using_effect_file_list.iter().flat_map(|f| &f.effect_data.node_list).map(|n| &n.behavior.cell_map_name).filter(|name| !name.is_empty()).cloned().collect::<Vec<String>>();
    let cellmap_names = anime_pack.cellmap_names.iter().map(|cellmap_file| cellmap_file.to_string()).chain(effects_use_cellmap_names).collect::<HashSet<String>>();
    let cellmaps = cellmap_names
        .iter()
        .filter_map(|cellmap_name| {
            sspj.cellmap_list.iter().find(|cellmap| {
                let cell_file_name = cellmap.name.to_string() + ".ssce";
                cell_file_name == *cellmap_name
            })
        })
        .collect::<Vec<_>>();

    // font_bitmap
    let using_font_bitmap_names = anime_pack
        .model
        .part_list
        .iter()
        .filter(|part| part.part_type == sspj::sstypes::SsPartType::Text)
        .filter_map(|part| if part.text_bitmap { Some(part.text_char_map.to_string()) } else { None })
        .collect::<HashSet<String>>();
    let using_fnt_bitmaps = using_font_bitmap_names
        .iter()
        .filter_map(|name| {
            let charmap = sspj.charmap_dic.get(name)?;
            Some((name.to_string(), charmap))
        })
        .collect::<HashMap<String, &sspj::sscharmap::SsCharMap>>();

    // sound list
    let using_sound_list = if anime_pack.model.part_list.iter().filter(|part| part.part_type == sspj::sstypes::SsPartType::Audio).collect::<Vec<&sspj::ssae::SsPart>>().is_empty() {
        vec![]
    } else {
        sspj.sound_list.iter().collect::<Vec<&sspj::ssse::SsAudioPack>>()
    };

    let external_textures = sspj.external_textures.iter().map(|texture_info| texture_info.path_name.clone()).collect::<HashSet<String>>();

    let external_instances = anime_pack
        .model
        .part_list
        .iter()
        .filter(|part| part.part_type == sspj::sstypes::SsPartType::Instance)
        .filter(|part| part.ref_anime_pack != anime_pack.name)
        .map(|part| [part.ref_anime_pack.as_str(), part.ref_anime.as_str()].join("/"))
        .collect::<HashSet<String>>();

    let mut anime_binary = ssab::ssab::ss::format::SsAnimeBinaryT::default();
    anime_binary.name = if anime_pack.name.is_empty() { None } else { Some(anime_pack.name.to_string()) };
    anime_binary.parts = if anime_pack.model.part_list.is_empty() { None } else { Some(model_t::animedata::convert_part_list(anime_pack)) };
    let parts_map: HashMap<String, &sspj::ssae::SsPart> = anime_pack.model.part_list.iter().map(|part| (part.name.to_string(), part)).collect();
    anime_binary.animations = if anime_pack.anime_list.is_empty() {
        None
    } else {
        let animations = model_t::animedata::convert_animation_data_list(anime_pack, &parts_map);
        if animations.is_empty() { None } else { Some(animations) }
    };
    anime_binary.cellmaps = if cellmaps.is_empty() { None } else { Some(model_t::cellmap::convert_cellmapt_list(sspj, &cellmaps, asset_provider)?) };
    anime_binary.effects = if using_effect_file_list.is_empty() { None } else { Some(model_t::effect::convert_effect_list(sspj, &using_effect_file_list)) };
    anime_binary.sound_lists = if using_sound_list.is_empty() { None } else { Some(model_t::sound::convert_sound_list(sspj, &using_sound_list, asset_provider)?) };
    anime_binary.font_bitmaps = if using_fnt_bitmaps.is_empty() { None } else { Some(model_t::font::convert_font_bitmaps(sspj, &using_fnt_bitmaps)) };
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
    anime_binary.external_instances = if external_instances.is_empty() { None } else { Some(external_instances.iter().cloned().collect::<Vec<_>>()) };
    anime_binary.embedded_assets = if embedded {
        let asset_files = crate::utils::get_assets_of_ssab_object_class(&anime_binary, sspj);
        let mut v = asset_files
            .iter()
            .map(|(name, path)| {
                //let src = sspj_parent_path.join(path);
                //let r = std::fs::read(&src);
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
