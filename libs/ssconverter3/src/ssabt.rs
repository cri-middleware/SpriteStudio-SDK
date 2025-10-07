use std::collections::HashMap;
use ssab::ssab::{CellMapT, CellT, FontBitmapT, RectT, TexFilterMode, TexWrapMode};
use sspj::ssae::SsAnimePack;
use sspj::ssce::{SsCell, SsCellMap};
use sspj::sscharmap::SsCharMap;
use sspj::sspj::SsProject;
use sspj::ssse::{SsAudioPack, SsSoundFileList};
use sspj::sstypes;
use sspj::sstypes::{SsPoint2, SsPoint3};
use crate::ssabt;

pub fn convert_vec2t(point2: &SsPoint2) -> ssab::ssab::Vec2T {
    let mut vec = ssab::ssab::Vec2T::default();
    vec.x = point2.x;
    vec.y = point2.y;
    vec
}
pub fn convert_vec2t_list(point2_list: &Vec<SsPoint2>) -> Vec<ssab::ssab::Vec2T> {
    point2_list.iter().map(|point2| {
        convert_vec2t(point2)
    }).collect::<Vec<ssab::ssab::Vec2T>>()
}
#[allow(dead_code)] // TODO: delete when using the function
pub fn convert_vec3t(point3: &SsPoint3) -> ssab::ssab::Vec3T {
    let mut vec = ssab::ssab::Vec3T::default();
    vec.x = point3.x;
    vec.y = point3.y;
    vec.z = point3.z;
    vec
}
#[allow(dead_code)] // TODO: delete when using the function
pub fn convert_vec3t_list(point3_list: &Vec<SsPoint3>) -> Vec<ssab::ssab::Vec3T> {
    point3_list.iter().map(|point3| {
        convert_vec3t(point3)
    }).collect::<Vec<ssab::ssab::Vec3T>>()
}

pub fn convert_flat_list_from_vec3t_list(point3_list: &Vec<SsPoint3>) -> Vec<u16> {
    point3_list.iter().flat_map(|point3| {
        [point3.x as u16, point3.y as u16, point3.z as u16]
    }).collect::<Vec<u16>>()
}

pub fn convert_rectt(x1: f32, y1: f32, x2: f32, y2: f32) -> ssab::ssab::RectT {
    let mut rect = RectT::default();
    rect.x1 = x1;
    rect.y1 = y1;
    rect.x2 = x2;
    rect.y2 = y2;

    rect
}

pub fn convert_cellt(sscell: &SsCell) -> CellT {
    let mut cell_t = CellT::default();
    cell_t.name = sscell.name.clone();
    cell_t.rectangle = Some(Box::new(convert_rectt(sscell.pos.x, sscell.pos.y, sscell.size.x, sscell.size.y)));
    cell_t.pivot = Some(Box::new(convert_vec2t(&sscell.pivot)));
    cell_t.rotated = sscell.rotated;
    cell_t.table_coordinate = Some(convert_vec2t_list(&sscell.mesh_point_list));
    cell_t.table_index_vertex = Some(convert_flat_list_from_vec3t_list(&sscell.mesh_tri_list));

    cell_t
}
pub fn convert_cellt_list(cells: &Vec<SsCell>) -> Vec<CellT> {
    cells.iter().map(|cell| {
        convert_cellt(cell)
    }).collect::<Vec<CellT>>()
}

pub fn convert_cellmapt(cellmap: &SsCellMap) -> CellMapT {
    let mut cell_map_t = CellMapT::default();
    cell_map_t.name = cellmap.name.clone();
    cell_map_t.image_path = Some(cellmap.image_path.clone());
    cell_map_t.size_original = Some(Box::new(convert_vec2t(&cellmap.pixel_size)));
    cell_map_t.wrap_mode = match cellmap.wrap_mode {
        sstypes::SsTexWrapMode::Clamp => TexWrapMode::Clamp,
        sstypes::SsTexWrapMode::Repeat => TexWrapMode::Repeat,
        sstypes::SsTexWrapMode::Mirror => TexWrapMode::Mirror,
    };
    cell_map_t.filter_mode = match cellmap.filter_mode {
        sstypes::SsTexFilterMode::Nearest => TexFilterMode::Nearest,
        sstypes::SsTexFilterMode::Linear => TexFilterMode::Linear,
    };
    cell_map_t.cells = Some(convert_cellt_list(&cellmap.cells));

    cell_map_t
}
pub fn convert_cellmapt_list(sspj: &SsProject, cellmap_names: &Vec<String>) -> Vec<CellMapT> {
    cellmap_names.iter().filter_map(|cellmap_name| {
        let Some(cellmap) = sspj.cellmap_list.iter().find(|cellmap| cellmap.name == *cellmap_name) else {
            // TODO: error handling
            return None;
        };
        Some(convert_cellmapt(cellmap))
    }).collect::<Vec<CellMapT>>()
}

pub fn convert_sound_file_list(_sspj: &SsProject, sssound_list: &SsSoundFileList) -> Vec<ssab::ssab::SoundFileT> {
    sssound_list.sounds.iter().map(|sssound| {
        let mut sound_file_t = ssab::ssab::SoundFileT::default();

        sound_file_t.name = sssound.alias.clone();
        sound_file_t.path = Some(sssound.filename.clone());
        sound_file_t.time_total = sssound.msec;

        sound_file_t
    }).collect::<Vec<ssab::ssab::SoundFileT>>()
}

pub fn convert_sound_list(sspj: &SsProject, sound_list: &Vec<SsAudioPack>) -> Vec<ssab::ssab::SoundListT> {
    sound_list.iter().map(|sssound_pack| {
        let mut sound_list_t = ssab::ssab::SoundListT::default();

        sound_list_t.name = sssound_pack.name.clone();
        sound_list_t.table_data = Some(convert_sound_file_list(sspj, &sssound_pack.table));

        sound_list_t
    }).collect::<Vec<ssab::ssab::SoundListT>>()
}

pub fn convert_font_bitmap(_sspj: &SsProject, charmap_dic: &HashMap<String, SsCharMap>) -> Vec<FontBitmapT> {
    charmap_dic.into_iter().map(|(_key, _value)| {
        // let mut font_bitmap_t = FontBitmapT::default();
        // font_bitmap_t.name = value.info.face;

        FontBitmapT::default()
    }).collect::<Vec<FontBitmapT>>()
}

pub fn convert_ss_anime_binary_t(anime_pack: &SsAnimePack, sspj: &SsProject) -> ssab::ssab::SsAnimeBinaryT {
    let mut anime_binary = ssab::ssab::SsAnimeBinaryT::default();
    anime_binary.animation_pack = None;
    anime_binary.cell_map = Some(ssabt::convert_cellmapt_list(&sspj, &anime_pack.cellmap_names));
    anime_binary.effect = None;
    anime_binary.sound_list = Some(ssabt::convert_sound_list(&sspj, &sspj.sound_list));
    anime_binary.font_bitmap = Some(ssabt::convert_font_bitmap(&sspj, &sspj.charmap_dic));
    anime_binary.dependencies = Some(Vec::new());

    anime_binary
}