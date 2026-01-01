use std::collections::HashSet;
use crate::model_t;


pub fn convert_ss_anime_binary_t(anime_pack: &sspj::ssae::SsAnimePack, sspj: &sspj::sspj::SsProject, ) -> ssab::ssab::SsAnimeBinaryT {
    let mut anime_binary = ssab::ssab::SsAnimeBinaryT::default();
    anime_binary.name = if anime_pack.name.is_empty() { None } else { Some(anime_pack.name.to_string()) };
    anime_binary.animations = if anime_pack.anime_list.is_empty() { None } else { Some(model_t::animedata::convert_animation_data_list(&sspj, &anime_pack)) };

    let using_effect_file_names = anime_pack.model.part_list.iter().filter_map(|part| {
        if part.part_type == sspj::sstypes::SsPartType::Effect {
            Some(part.ref_effect_name.to_string())
        } else {
            None
        }
    }).collect::<HashSet<String>>();
    let effect_file_list = using_effect_file_names.iter().filter_map(|using_effect_file_name| {
        sspj.effect_file_list.iter().find(|effect_file| effect_file.name == *using_effect_file_name).cloned()
    }).collect::<Vec<sspj::ssee::SsEffectFile>>();
    anime_binary.effects = if effect_file_list.is_empty() { None } else { Some(model_t::effect::convert_effect_list(&sspj, &effect_file_list)) };

    let effect_cellmap_names = effect_file_list.iter().flat_map(|f| &f.effect_data.node_list)
        .map(|n| &n.behavior.cell_map_name)
        .filter(|name| !name.is_empty()).cloned().collect::<Vec<String>>();

    anime_binary.sound_lists = if anime_pack.model.part_list.iter().filter(|part| part.part_type == sspj::sstypes::SsPartType::Audio).collect::<Vec<&sspj::ssae::SsPart>>().is_empty() { // don't use sound
        None
    } else {
        model_t::sound::convert_sound_list(&sspj, &sspj.sound_list)
    };

    /*
    anime_pack.model.part_list.iter().filter(|part| part.part_type == SsPartType::Text)
        .filter_map(|part| {
            // TODO: ?
            Some(part.array_index)
        });
     */
    anime_binary.font_bitmaps = if sspj.charmap_dic.is_empty() { None } else { Some(model_t::font::convert_font_bitmaps(&sspj, &sspj.charmap_dic)) };

    let cellmap_names = anime_pack.cellmap_names.iter().map(|cellmap_file| cellmap_file.to_string()).chain(effect_cellmap_names).collect::<HashSet<String>>();
    let cellmaps = cellmap_names.iter().filter_map(|cellmap_name| {
        sspj.cellmap_list.iter().find(|cellmap| cellmap.load_file_path == *cellmap_name).cloned()
    }).collect::<Vec<sspj::ssce::SsCellMap>>();
    anime_binary.cellmaps = if cellmaps.is_empty() { None } else { Some(model_t::cellmap::convert_cellmapt_list(&sspj, &cellmaps)) };

    let external_instances = anime_pack.model.part_list.iter().filter(|part| part.part_type == sspj::sstypes::SsPartType::Instance)
        .filter(|part| part.ref_anime_pack != anime_pack.name)
        .map(|part| format!("{}/{}", anime_pack.name, part.ref_anime_pack)).collect::<Vec<String>>();
    anime_binary.external_instances = if external_instances.is_empty() { None } else { Some(external_instances) };

    anime_binary
}
