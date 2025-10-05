use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use sspj::ssloader::SsLoader;
use sspj::sspj::SsProject;
use sspj::ssloader::SspjError;
use ssfb::ssab::{CellMapT, CellT, FontBitmapT, FontGlyphBitmapT, RectT, SsAnimeBinaryT, TexFilterMode, TexWrapMode, Vec2T};
use ssfb::ssqb;
use sspj::sstypes;
use sspj::sstypes::{SsPoint2, SsPoint3};
use sspj::ssce::{SsCell, SsCellMap};
use sspj::sscharmap::SsCharMap;
use sspj::ssqe::{SsSequence, SsSequenceItem};
use sspj::ssse::{SsAudioPack, SsSoundFileList};

pub struct SspjLoader {
    path_buf: PathBuf,
}
impl SspjLoader {
    pub fn new(path: &Path) -> Self {
        SspjLoader {
            path_buf: path.to_path_buf()
        }
    }
    pub fn load_ss_project(&self) -> Result<SsProject, SspjError> {
        SsProject::load_from_file(self.path_buf.as_path())
    }
}

pub struct ConverterConfig {
    pub debug: bool,
    pub split: bool,
}
impl Default for ConverterConfig {
    fn default() -> Self {
        Self {
            debug: false,
            split: false,
        }
    }
}

fn convert_vec2t(point2: &SsPoint2) -> ssfb::ssab::Vec2T {
    let mut vec = ssfb::ssab::Vec2T::default();
    vec.x = point2.x;
    vec.y = point2.y;
    vec
}
fn convert_vec2t_list(point2_list: &Vec<SsPoint2>) -> Vec<ssfb::ssab::Vec2T> {
    point2_list.iter().map(|point2| {
        convert_vec2t(point2)
    }).collect::<Vec<ssfb::ssab::Vec2T>>()
}
fn convert_vec3t(point3: &SsPoint3) -> ssfb::ssab::Vec3T {
    let mut vec = ssfb::ssab::Vec3T::default();
    vec.x = point3.x;
    vec.y = point3.y;
    vec.z = point3.z;
    vec
}
fn convert_vec3t_list(point3_list: &Vec<SsPoint3>) -> Vec<ssfb::ssab::Vec3T> {
    point3_list.iter().map(|point3| {
        convert_vec3t(point3)
    }).collect::<Vec<ssfb::ssab::Vec3T>>()
}
fn convert_flat_list_from_vec3t_list(point3_list: &Vec<SsPoint3>) -> Vec<u16> {
    point3_list.iter().flat_map(|point3| {
        [point3.x as u16, point3.y as u16, point3.z as u16]
    }).collect::<Vec<u16>>()
}

fn convert_rectt(x1: f32, y1: f32, x2: f32, y2: f32) -> ssfb::ssab::RectT {
    let mut rect = RectT::default();
    rect.x1 = x1;
    rect.y1 = y1;
    rect.x2 = x2;
    rect.y2 = y2;

    rect
}

fn convert_cellt(sscell: &SsCell) -> CellT {
    let mut cell_t = CellT::default();
    cell_t.name = sscell.name.clone();
    cell_t.rectangle = Some(Box::new(convert_rectt(sscell.pos.x, sscell.pos.y, sscell.size.x, sscell.size.y)));
    cell_t.pivot = Some(Box::new(convert_vec2t(&sscell.pivot)));
    cell_t.rotated = sscell.rotated;
    cell_t.table_coordinate = Some(convert_vec2t_list(&sscell.mesh_point_list));
    cell_t.table_index_vertex = Some(convert_flat_list_from_vec3t_list(&sscell.mesh_tri_list));

    cell_t
}
fn convert_cellt_list(cells: &Vec<SsCell>) -> Vec<CellT> {
    cells.iter().map(|cell| {
        convert_cellt(cell)
    }).collect::<Vec<CellT>>()
}

fn convert_cellmapt(cellmap: &SsCellMap) -> CellMapT {
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
fn convert_cellmapt_list(sspj: &SsProject, cellmap_names: &Vec<String>) -> Vec<CellMapT> {
    cellmap_names.iter().filter_map(|cellmap_name| {
        let Some(cellmap) = sspj.cellmap_list.iter().find(|cellmap| cellmap.name == *cellmap_name) else {
            // TODO: error handling
            return None;
        };
        Some(convert_cellmapt(cellmap))
    }).collect::<Vec<CellMapT>>()
}

fn convert_sound_file_list(sspj: &SsProject, sssound_list: &SsSoundFileList) -> Vec<ssfb::ssab::SoundFileT> {
    sssound_list.sounds.iter().map(|sssound| {
        let mut sound_file_t = ssfb::ssab::SoundFileT::default();

        sound_file_t.name = sssound.alias.clone();
        sound_file_t.path = Some(sssound.filename.clone());
        sound_file_t.time_total = sssound.msec;

        sound_file_t
    }).collect::<Vec<ssfb::ssab::SoundFileT>>()
}

fn convert_sound_list(sspj: &SsProject, sound_list: &Vec<SsAudioPack>) -> Vec<ssfb::ssab::SoundListT> {
    sound_list.iter().map(|sssound_pack| {
        let mut sound_list_t = ssfb::ssab::SoundListT::default();

        sound_list_t.name = sssound_pack.name.clone();
        sound_list_t.table_data = Some(convert_sound_file_list(sspj, &sssound_pack.table));

        sound_list_t
    }).collect::<Vec<ssfb::ssab::SoundListT>>()
}

fn convert_font_bitmap(sspj: &SsProject, charmap_dic: &HashMap<String, SsCharMap>) -> Vec<FontBitmapT> {
    charmap_dic.into_iter().map(|(_key, value)| {
        let mut font_bitmap_t = FontBitmapT::default();
        // font_bitmap_t.name = value.info.face;

        font_bitmap_t
    }).collect::<Vec<FontBitmapT>>()
}

fn convert_sequence_t_list(sequence_list: &Vec<SsSequence>) -> Vec<ssfb::ssqb::SequenceT> {
    sequence_list.iter().map(|sequence| {
        let mut sequence_t = ssfb::ssqb::SequenceT::default();
        sequence_t.name = sequence.name.clone();
        sequence_t.index = sequence.index;
        sequence_t.type_ = match sequence.sequence_type {
            sspj::sstypes::SsSequenceType::Last => ssfb::ssqb::SequenceType::Last,
            sspj::sstypes::SsSequenceType::Keep => ssfb::ssqb::SequenceType::Keep,
            sspj::sstypes::SsSequenceType::Top => ssfb::ssqb::SequenceType::Top,
        };
        sequence_t.list = Some(convert_sequence_item_t_list(&sequence.list));

        sequence_t
    }).collect::<Vec<ssfb::ssqb::SequenceT>>()
}

fn convert_sequence_item_t_list(sequence_item_list: &Vec<SsSequenceItem>) -> Vec<ssfb::ssqb::SequenceItemT> {
    sequence_item_list.iter().map(|sequence_item| {
        let mut sequence_item_t = ssfb::ssqb::SequenceItemT::default();
        sequence_item_t.ref_anime = Some(sequence_item.ref_anime.clone());
        sequence_item_t.ref_anime_pack = Some(sequence_item.ref_anime_pack.clone());
        sequence_item_t.repeat_count = sequence_item.repeat_count;

        sequence_item_t
    }).collect::<Vec<ssfb::ssqb::SequenceItemT>>()
}

pub fn convert_ssab_from_sspj(path: &Path, config: ConverterConfig) -> Result<HashMap<String, Vec<u8>>, SspjError> {
    let sspj_loader = crate::SspjLoader::new(path);
    let sspj = match sspj_loader.load_ss_project() {
        Ok(sspj) => sspj,
        Err(error) => {
            println!("{:?} error: {:?}", path.as_os_str().to_string_lossy(), error);
            return Err(error);
        }
    };

    // pack sspj animation data and sequence data to ssab ssab and ssqb structures
    let mut ssab_model_map = sspj.animepack_names.iter().zip(sspj.anime_list.iter()).map(|(f, anime_pack)| {
        let mut anime_binary = ssfb::ssab::SsAnimeBinaryT::default();
        anime_binary.animation_pack = None;
        anime_binary.cell_map = Some(convert_cellmapt_list(&sspj, &anime_pack.cellmap_names));
        anime_binary.effect = None;
        anime_binary.sound_list = Some(convert_sound_list(&sspj, &sspj.sound_list));
        anime_binary.font_bitmap = Some(convert_font_bitmap(&sspj, &sspj.charmap_dic));
        anime_binary.dependencies = Some(Vec::new());

        (f.to_string(), anime_binary)
    }).collect::<HashMap<String, SsAnimeBinaryT>>();

    let ssqb_model_map = sspj.sequencepack_names.iter().zip(sspj.sequence_list.iter()).map(|(f, sequence)| {
        let mut sequence_binary_t = ssfb::ssqb::SsSequenceBinaryT::default();
        sequence_binary_t.sequence_list = Some(convert_sequence_t_list(&sequence.sequence_list));

        (f.to_string(), sequence_binary_t)
    }).collect::<HashMap<String, ssfb::ssqb::SsSequenceBinaryT>>();

    // optimize ssab ssab structures (e.g. set dependencies)
    // TODO: implement

    // convert ssab structures to binary data
    let mut ssab_bin_map = ssab_model_map.iter().map(|(f, anime_binary)| {
        // TODO: implement
        (f.to_string(), Vec::new())
    }).collect::<HashMap<String, Vec<u8>>>();

    // convert ssqb structures to binary data
    let ssqb_bin_map = ssqb_model_map.iter().map(|(f, sequence_binary_t)| {
        let mut fb = flatbuffers::FlatBufferBuilder::with_capacity(1);
        let offset = sequence_binary_t.pack(&mut fb);
        fb.finish(offset, Some("ssqb"));
        let buf = fb.finished_data();

        (f.to_string(), buf.to_vec())
    }).collect::<HashMap<String, Vec<u8>>>();

    ssab_bin_map.extend(ssqb_bin_map);
    Ok(ssab_bin_map)
}

pub fn convert_ss_project_to_ssab(path: &Path, config: ConverterConfig) -> Result<(), SspjError> {
    let ssab = convert_ssab_from_sspj(path, config)?;

    let Some(parent) = path.parent() else {
        return Err(SspjError::FileError(std::io::Error::new(ErrorKind::NotFound, "")));
    };
    ssab.iter().for_each(|(key, value)| {
        let output_path = parent.join(key).with_extension("ssab");
        std::fs::write(output_path.as_path(), value).expect("TODO: panic message"); // TODO: FIXME
    });

    Ok(())
}
