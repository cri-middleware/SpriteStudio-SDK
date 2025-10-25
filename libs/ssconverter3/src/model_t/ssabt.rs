use crate::model_t::ssabt;
use ssab::ssab::{
    AnimationDataT, CellMapT, CellT, EffectT, FontBitmapT, LabelT, MeshDataIndicesT, MeshDataUVT,
    PartAttributesT, PartStateT, RectT,
};
use sspj::ssae::{SsAnimation, SsAnimePack, SsPart};
use sspj::ssce::{SsCell, SsCellMap};
use sspj::sscharmap::SsCharMap;
use sspj::ssee::SsEffectFile;
use sspj::sspj::SsProject;
use sspj::ssse::{SsAudioPack, SsSoundFileList};
use sspj::sstypes;
use sspj::sstypes::{SsPartType, SsPoint2, SsPoint3};
use std::collections::{HashMap, HashSet};

pub fn convert_vec2t(point2: &SsPoint2) -> ssab::ssab::Vec2T {
    let mut vec = ssab::ssab::Vec2T::default();
    vec.v1 = point2.x;
    vec.v2 = point2.y;
    vec
}
pub fn convert_vec2t_list(point2_list: &[SsPoint2]) -> Option<Vec<ssab::ssab::Vec2T>> {
    Some(
        point2_list
            .iter()
            .map(|point2| convert_vec2t(point2))
            .collect::<Vec<ssab::ssab::Vec2T>>(),
    )
}

pub fn convert_flat_list_from_vec3t_list(point3_list: &[SsPoint3]) -> Option<Vec<u16>> {
    Some(
        point3_list
            .iter()
            .flat_map(|point3| [point3.x as u16, point3.y as u16, point3.z as u16])
            .collect::<Vec<u16>>(),
    )
}

pub fn convert_rectt(x1: f32, y1: f32, x2: f32, y2: f32) -> ssab::ssab::RectT {
    let mut rect = ssab::ssab::RectT::default();
    rect.x1 = x1;
    rect.y1 = y1;
    rect.x2 = x2;
    rect.y2 = y2;

    rect
}

fn convert_animedatat(anime_data: &SsAnimation, setup_animation: &SsAnimation) -> AnimationDataT {
    let mut anime_data_t = AnimationDataT::default();

    anime_data_t.name = anime_data.name.clone();
    anime_data_t.fps = anime_data.settings.fps as i16;
    let mut canvas = ssab::ssab::RectT::default();
    canvas.x1 = anime_data.settings.pivot.x;
    canvas.y1 = anime_data.settings.pivot.y;
    canvas.x2 = anime_data.settings.canvas_size.x;
    canvas.y2 = anime_data.settings.canvas_size.y;
    anime_data_t.canvas = Some(Box::new(canvas));
    anime_data_t.start_frames = anime_data.settings.start_frame as i16;
    anime_data_t.end_frames = anime_data.settings.end_frame as i16;
    anime_data_t.total_frames = anime_data.settings.frame_count as i16;

    // anime_data_t.setup_animation = setup_animation;

    /*
    anime_data_t.frame_data;
    anime_data_t.events;
    anime_data_t.label_data;
    anime_data_t.meshs_data_uv;
    anime_data_t.meshs_data_indices;
     */

    anime_data_t
}
fn convert_animation_data_list(
    sspj: &&SsProject,
    anime_pack: &SsAnimePack,
) -> Option<Vec<AnimationDataT>> {
    let Some(setup_animation) = anime_pack
        .anime_list
        .iter()
        .find(|anime_data| anime_data.is_setup)
    else {
        // not found setup animation
        return None;
    };

    Some(
        anime_pack
            .anime_list
            .iter()
            .filter_map(|anime_data| {
                if anime_data == setup_animation {
                    return None;
                }
                Some(convert_animedatat(anime_data, setup_animation))
            })
            .collect::<Vec<AnimationDataT>>(),
    )
}

pub fn convert_cellt(sscell: &SsCell) -> CellT {
    let mut cell_t = CellT::default();
    cell_t.name = sscell.name.clone();
    cell_t.rectangle = Some(Box::new(convert_rectt(
        sscell.pos.x,
        sscell.pos.y,
        sscell.size.x,
        sscell.size.y,
    )));
    cell_t.pivot = Some(Box::new(convert_vec2t(&sscell.pivot)));
    cell_t.rotated = sscell.rotated;
    cell_t.table_coordinate = convert_vec2t_list(&sscell.mesh_point_list);
    cell_t.table_index_vertex = convert_flat_list_from_vec3t_list(&sscell.mesh_tri_list);

    cell_t
}
pub fn convert_cellt_list(cells: &[SsCell]) -> Option<Vec<CellT>> {
    Some(
        cells
            .iter()
            .map(|cell| convert_cellt(cell))
            .collect::<Vec<CellT>>(),
    )
}

pub fn convert_cellmapt(cellmap: &SsCellMap) -> CellMapT {
    let mut cell_map_t = CellMapT::default();
    cell_map_t.name = cellmap.name.clone();
    cell_map_t.image_path = Some(cellmap.image_path.clone());
    cell_map_t.size_original = Some(Box::new(convert_vec2t(&cellmap.pixel_size)));
    cell_map_t.wrap_mode = match cellmap.wrap_mode {
        sstypes::SsTexWrapMode::Clamp => ssab::ssab::TexWrapMode::Clamp,
        sstypes::SsTexWrapMode::Repeat => ssab::ssab::TexWrapMode::Repeat,
        sstypes::SsTexWrapMode::Mirror => ssab::ssab::TexWrapMode::Mirror,
    };
    cell_map_t.filter_mode = match cellmap.filter_mode {
        sstypes::SsTexFilterMode::Nearest => ssab::ssab::TexFilterMode::Nearest,
        sstypes::SsTexFilterMode::Linear => ssab::ssab::TexFilterMode::Linear,
    };
    cell_map_t.cells = convert_cellt_list(&cellmap.cells);

    cell_map_t
}
pub fn convert_cellmapt_list(
    sspj: &SsProject,
    cellmap_names: &[SsCellMap],
) -> Option<Vec<CellMapT>> {
    if cellmap_names.is_empty() {
        return None;
    }
    Some(
        cellmap_names
            .iter()
            .map(|cellmap| convert_cellmapt(cellmap))
            .collect::<Vec<CellMapT>>(),
    )
}

fn convert_effect_list(
    _sspj: &SsProject,
    effect_file_vec: &[SsEffectFile],
) -> Option<Vec<EffectT>> {
    if effect_file_vec.is_empty() {
        return None;
    }
    Some(
        effect_file_vec
            .iter()
            .map(|effect_file| {
                let mut effect_t = EffectT::default();
                effect_t.name = effect_file.name.clone();
                // effect_t.fps = effect_file.effect_data.fps as i16;
                // effect_t.is_lock_rand_seed;
                // effect_t.lock_rand_seed;
                // effect_t.layout_scale;
                // effect_t.effect_node;

                effect_t
            })
            .collect::<Vec<EffectT>>(),
    )
}

pub fn convert_sound_file_list(
    _sspj: &SsProject,
    sssound_list: &SsSoundFileList,
) -> Option<Vec<ssab::ssab::SoundFileT>> {
    Some(
        sssound_list
            .sounds
            .iter()
            .map(|sssound| {
                let mut sound_file_t = ssab::ssab::SoundFileT::default();

                sound_file_t.name = sssound.alias.clone();
                sound_file_t.time_total = sssound.msec;

                sound_file_t
            })
            .collect::<Vec<ssab::ssab::SoundFileT>>(),
    )
}

pub fn convert_sound_list(
    sspj: &SsProject,
    sound_list: &[SsAudioPack],
) -> Option<Vec<ssab::ssab::SoundListT>> {
    if sound_list.is_empty() {
        return None;
    }
    Some(
        sound_list
            .iter()
            .map(|sssound_pack| {
                let mut sound_list_t = ssab::ssab::SoundListT::default();

                sound_list_t.name = sssound_pack.name.clone();
                sound_list_t.table_data = convert_sound_file_list(sspj, &sssound_pack.table);

                sound_list_t
            })
            .collect::<Vec<ssab::ssab::SoundListT>>(),
    )
}

pub fn convert_font_bitmap(
    _sspj: &SsProject,
    charmap_dic: &HashMap<String, SsCharMap>,
) -> Option<Vec<FontBitmapT>> {
    if charmap_dic.is_empty() {
        return None;
    }
    Some(
        charmap_dic
            .into_iter()
            .map(|(_key, _value)| {
                // let mut font_bitmap_t = FontBitmapT::default();
                // font_bitmap_t.name = value.info.face;

                FontBitmapT::default()
            })
            .collect::<Vec<FontBitmapT>>(),
    )
}

fn get_cellmap_names_of_instance(anime_pack: &SsAnimePack, sspj: &SsProject) -> Vec<String> {
    anime_pack
        .model
        .part_list
        .iter()
        .filter(|part| part.part_type == SsPartType::Instance)
        .filter_map(|part| {
            let Some(a) = sspj
                .anime_list
                .iter()
                .find(|anime| anime.name == part.ref_anime_pack)
            else {
                return None;
            };
            Some(
                a.cellmap_names
                    .iter()
                    .map(|cellmap_name| cellmap_name.clone())
                    .collect::<Vec<String>>(),
            )
        })
        .flatten()
        .collect::<Vec<String>>()
}

pub fn convert_ss_anime_binary_t(
    anime_pack: &SsAnimePack,
    sspj: &SsProject,
) -> ssab::ssab::SsAnimeBinaryT {
    let mut anime_binary = ssab::ssab::SsAnimeBinaryT::default();
    anime_binary.animations = ssabt::convert_animation_data_list(&sspj, &anime_pack);
    let using_effect_file_names = anime_pack
        .model
        .part_list
        .iter()
        .filter_map(|part| {
            if part.part_type == SsPartType::Effect {
                Some(part.ref_effect_name.clone())
            } else {
                None
            }
        })
        .collect::<HashSet<String>>();
    let effect_file_list = using_effect_file_names
        .iter()
        .filter_map(|using_effect_file_name| {
            sspj.effect_file_list
                .iter()
                .find(|effect_file| effect_file.name == *using_effect_file_name)
                .cloned()
        })
        .collect::<Vec<SsEffectFile>>();
    anime_binary.effect = ssabt::convert_effect_list(&sspj, &effect_file_list);

    anime_binary.sound_list = if anime_pack
        .model
        .part_list
        .iter()
        .filter(|part| part.part_type == SsPartType::Audio)
        .collect::<Vec<&SsPart>>()
        .is_empty()
    {
        // don't use sound
        None
    } else {
        convert_sound_list(&sspj, &sspj.sound_list)
    };
    anime_binary.font_bitmap = ssabt::convert_font_bitmap(&sspj, &sspj.charmap_dic);
    anime_binary.instances = None; // TODO: impl

    let mut cellmap_names = anime_pack
        .cellmap_names
        .iter()
        .map(|cellmap_file| cellmap_file.clone())
        .collect::<HashSet<String>>();
    get_cellmap_names_of_instance(&anime_pack, &sspj)
        .iter()
        .for_each(|cellmap_name| {
            cellmap_names.insert(cellmap_name.clone());
        });
    if anime_pack
        .model
        .part_list
        .iter()
        .filter(|part| part.part_type == SsPartType::Effect)
        .collect::<Vec<&SsPart>>()
        .is_empty()
        != true
    {
        sspj.effect_file_list.iter().for_each(|effect_file| {
            effect_file.effect_data.node_list.iter().for_each(|node| {
                if node.behavior.cell_map_name.is_empty() {
                    return;
                }
                cellmap_names.insert(node.behavior.cell_map_name.clone());
            })
        });
    };
    anime_binary.instances.as_ref().map(|instances| {
        instances.iter().for_each(|instance| {
            let splt = instance.name.split("/").collect::<Vec<&str>>();
            let animepack = splt[0];
            get_cellmap_names_of_instance(anime_pack, sspj)
                .iter()
                .for_each(|cellmap_name| {
                    cellmap_names.insert(cellmap_name.clone());
                })
        })
    });
    let cellmaps = cellmap_names
        .iter()
        .filter_map(|cellmap_name| {
            sspj.cellmap_list
                .iter()
                .find(|cellmap| cellmap.load_file_path == *cellmap_name)
                .cloned()
        })
        .collect::<Vec<SsCellMap>>();
    anime_binary.cell_map = ssabt::convert_cellmapt_list(&sspj, &cellmaps);

    anime_binary
}
