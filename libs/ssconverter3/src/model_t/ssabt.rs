use std::collections::{HashMap, HashSet};
use crate::model_t;

pub fn convert_ss_anime_binary_t(anime_pack: &sspj::ssae::SsAnimePack, sspj: &sspj::sspj::SsProject, sspj_parent_path: &std::path::Path, embedded: bool) -> Result<ssab::ssab::SsAnimeBinaryT, crate::ConverterError> {
    // effect
    let using_effect_file_names = anime_pack.model.part_list.iter().filter_map(|part| {
        if part.part_type == sspj::sstypes::SsPartType::Effect {
            Some(part.ref_effect_name.to_string())
        } else {
            None
        }
    }).collect::<HashSet<String>>();
    let using_effect_file_list = using_effect_file_names.iter().filter_map(|using_effect_file_name| {
        sspj.effect_file_list.iter().find(|effect_file| effect_file.name == *using_effect_file_name)
    }).collect::<Vec<_>>();

    // cellmap
    let effects_use_cellmap_names = using_effect_file_list.iter().flat_map(|f| &f.effect_data.node_list).map(|n| &n.behavior.cell_map_name).filter(|name| !name.is_empty()).cloned().collect::<Vec<String>>();
    let cellmap_names = anime_pack.cellmap_names.iter().map(|cellmap_file| cellmap_file.to_string()).chain(effects_use_cellmap_names).collect::<HashSet<String>>();
    let cellmaps = cellmap_names.iter().filter_map(|cellmap_name| {
        sspj.cellmap_list.iter().find(|cellmap| cellmap.load_file_path == *cellmap_name)
    }).collect::<Vec<_>>();

    // font_bitmap
    let using_font_bitmap_names = anime_pack.model.part_list.iter().filter(|part| part.part_type == sspj::sstypes::SsPartType::Text).filter_map(|part| {
        if part.text_bitmap { Some(part.text_char_map.to_string()) } else { None }
    }).collect::<HashSet<String>>();
    let using_fnt_bitmaps = using_font_bitmap_names.iter().filter_map(|name| {
        let Some(charmap) = sspj.charmap_dic.get(name) else {
            return None;
        };
        Some((name.to_string(), charmap))
    }).collect::<HashMap<String, &sspj::sscharmap::SsCharMap>>();
    
    // sound list
    let using_sound_list = if anime_pack.model.part_list.iter().filter(|part| part.part_type == sspj::sstypes::SsPartType::Audio).collect::<Vec<&sspj::ssae::SsPart>>().is_empty() { vec![] } else {
        sspj.sound_list.iter().map(|sound| sound).collect::<Vec<&sspj::ssse::SsAudioPack>>()
    };
    
    
    let external_textures = sspj.external_textures.iter().map(|texture_info| texture_info.path_name.clone()).collect::<HashSet<String>>();

    let external_instances = anime_pack.model.part_list.iter().filter(|part| part.part_type == sspj::sstypes::SsPartType::Instance)
        .filter(|part| part.ref_anime_pack != anime_pack.name)
        .map(|part| [part.ref_anime_pack.as_str(), part.ref_anime.as_str()].join("/")).collect::<HashSet<String>>();


    let mut anime_binary = ssab::ssab::SsAnimeBinaryT::default();
    anime_binary.name = if anime_pack.name.is_empty() { None } else { Some(anime_pack.name.to_string()) };
    anime_binary.parts = if anime_pack.model.part_list.is_empty() { None } else { Some(model_t::animedata::convert_part_list(&anime_pack)) };
    let parts_map: HashMap<String, &sspj::ssae::SsPart> = anime_pack.model.part_list.iter().map(|part| (part.name.to_string(), part)).collect();
    anime_binary.animations = if anime_pack.anime_list.is_empty() { None } else { 
        let animations = model_t::animedata::convert_animation_data_list(&anime_pack, &parts_map);
        if animations.is_empty() { None } else { Some(animations) }
    };
    anime_binary.cellmaps = if cellmaps.is_empty() { None } else { Some(model_t::cellmap::convert_cellmapt_list(&sspj, &cellmaps, sspj_parent_path)?) };
    anime_binary.effects = if using_effect_file_list.is_empty() { None } else { Some(model_t::effect::convert_effect_list(&sspj, &using_effect_file_list)) };
    anime_binary.sound_lists = if using_sound_list.is_empty() { None } else { Some(model_t::sound::convert_sound_list(&sspj, &using_sound_list)) };
    anime_binary.font_bitmaps = if using_fnt_bitmaps.is_empty() { None } else { Some(model_t::font::convert_font_bitmaps(&sspj, &using_fnt_bitmaps)) };
    anime_binary.external_textures = if external_textures.is_empty() { None } else { Some(external_textures.iter().map(|texture_path| {
        let mut external_texture = ssab::ssab::ExternalTextureT::default();
        external_texture.name_hash = crate::utils::fnv1a_hash_str(texture_path);
        external_texture.name = Some(texture_path.to_string());
        let r = crate::utils::validate_file_path(texture_path, crate::utils::get_sspj_image_dir_path(sspj_parent_path, sspj).as_path());
        match r {
            Ok(_) => Ok(external_texture),
            Err(e) => Err(e),
        }
    }).collect::<Result<Vec<ssab::ssab::ExternalTextureT>, _>>()?) };
    anime_binary.external_instances = if external_instances.is_empty() { None } else { Some( external_instances.iter().map(|external_instance| external_instance.clone()).collect::<Vec<_>>()) };
    anime_binary.embedded_assets = if embedded {
        let asset_files = crate::utils::get_assets_of_ssab_object_class(&anime_binary, &sspj);
        let v = asset_files.iter().map(|(name, path)| {
            let r = std::fs::read(path);
            let data = match r {
                Ok(data) => data,
                Err(e) => {
                    return Err(crate::ConverterError::IoError(e));
                }
            };
            let mut embedded_asset = ssab::ssab::EmbeddedAssetT::default();
            embedded_asset.name = Some(name.to_string());
            embedded_asset.name_hash = crate::utils::fnv1a_hash_str(name);
            embedded_asset.data = Some(data);

            Ok(embedded_asset)
        }).collect::<Result<Vec<ssab::ssab::EmbeddedAssetT>, _>>()?;
        Some(v)
    } else { None };

    Ok(anime_binary)
}
