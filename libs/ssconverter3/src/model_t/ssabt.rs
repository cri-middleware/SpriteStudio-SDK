use std::collections::{HashMap, HashSet};
use crate::model_t;


pub fn convert_ss_anime_binary_t(anime_pack: &sspj::ssae::SsAnimePack, sspj: &sspj::sspj::SsProject) -> ssab::ssab::SsAnimeBinaryT {
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
        if part.text_bitmap { Some(part.name.to_string()) } else { None }
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
    
    
    let external_instances = anime_pack.model.part_list.iter().filter(|part| part.part_type == sspj::sstypes::SsPartType::Instance)
        .filter(|part| part.ref_anime_pack != anime_pack.name)
        .map(|part| format!("{}/{}", anime_pack.name, part.ref_anime_pack)).collect::<Vec<String>>();


    let mut anime_binary = ssab::ssab::SsAnimeBinaryT::default();
    anime_binary.name = if anime_pack.name.is_empty() { None } else { Some(anime_pack.name.to_string()) };
    anime_binary.parts = if anime_pack.model.part_list.is_empty() { None } else { Some(model_t::animedata::convert_part_list(&anime_pack)) };
    let parts_map: HashMap<String, &sspj::ssae::SsPart> = anime_pack.model.part_list.iter().map(|part| (part.name.to_string(), part)).collect();
    anime_binary.animations = if anime_pack.anime_list.is_empty() { None } else { Some(model_t::animedata::convert_animation_data_list(&anime_pack, &parts_map)) };
    anime_binary.cellmaps = if cellmaps.is_empty() { None } else { Some(model_t::cellmap::convert_cellmapt_list(&sspj, &cellmaps)) };
    anime_binary.effects = if using_effect_file_list.is_empty() { None } else { Some(model_t::effect::convert_effect_list(&sspj, &using_effect_file_list)) };
    anime_binary.sound_lists = if using_sound_list.is_empty() { None } else { Some(model_t::sound::convert_sound_list(&sspj, &using_sound_list)) };
    anime_binary.font_bitmaps = if using_fnt_bitmaps.is_empty() { None } else { Some(model_t::font::convert_font_bitmaps(&sspj, &using_fnt_bitmaps)) };
    anime_binary.external_instances = if external_instances.is_empty() { None } else { Some(external_instances) };

    anime_binary
}
