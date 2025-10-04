use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use sspj::ssloader::SsLoader;
use sspj::sspj::SsProject;
use sspj::ssloader::SspjError;
use ssfb::ssab::ss::data::{CellMapT, CellT, RectT, SsAnimeBinaryT, SsTexFilterMode, SsTexWrapMode};
use sspj::sstypes;
use sspj::sstypes::{SsPoint2, SsPoint3};
use sspj::ssce::{SsCell, SsCellMap};

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

fn convert_vec2t(point2: &SsPoint2) -> ssfb::ssab::ss::data::Vec2T {
    let mut vec = ssfb::ssab::ss::data::Vec2T::default();
    vec.x = point2.x;
    vec.y = point2.y;
    vec
}
fn convert_vec2t_list(point2_list: &Vec<SsPoint2>) -> Vec<ssfb::ssab::ss::data::Vec2T> {
    point2_list.iter().map(|point2| {
        convert_vec2t(point2)
    }).collect::<Vec<ssfb::ssab::ss::data::Vec2T>>()
}
fn convert_vec3t(point3: &SsPoint3) -> ssfb::ssab::ss::data::Vec3T {
    let mut vec = ssfb::ssab::ss::data::Vec3T::default();
    vec.x = point3.x;
    vec.y = point3.y;
    vec.z = point3.z;
    vec
}
fn convert_vec3t_list(point3_list: &Vec<SsPoint3>) -> Vec<ssfb::ssab::ss::data::Vec3T> {
    point3_list.iter().map(|point3| {
        convert_vec3t(point3)
    }).collect::<Vec<ssfb::ssab::ss::data::Vec3T>>()
}

fn convert_rectt(x1: f32, y1: f32, x2: f32, y2: f32) -> ssfb::ssab::ss::data::RectT {
    let mut rect = RectT::default();
    rect.x1 = x1;
    rect.y1 = y1;
    rect.x2 = x2;
    rect.y2 = y2;

    rect
}

fn convert_cellt(cell: &SsCell) -> CellT {
    let mut cell_t = CellT::default();
    cell_t.name = cell.name.clone();
    cell_t.rectangle = Some(Box::new(convert_rectt(cell.pos.x, cell.pos.y, cell.size.x, cell.size.y)));
    cell_t.pivot = Some(Box::new(convert_vec2t(&cell.pivot)));
    cell_t.rotated = cell.rotated;
    // cell_t.table_coordinate = convert_vec2t_list(cell.), // TODO: FIXME
    //cell_t.table_index_vertex: None,

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
        sstypes::SsTexWrapMode::Clamp => SsTexWrapMode::Clamp,
        sstypes::SsTexWrapMode::Repeat => SsTexWrapMode::Repeat,
        sstypes::SsTexWrapMode::Mirror => SsTexWrapMode::Mirror,
    };
    cell_map_t.filter_mode = match cellmap.filter_mode {
        sstypes::SsTexFilterMode::Nearest => SsTexFilterMode::Nearest,
        sstypes::SsTexFilterMode::Linear => SsTexFilterMode::Linear,
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

pub fn convert_ss_project(path: &Path, config: ConverterConfig) -> Result<HashMap<String, Vec<u8>>, SspjError> {
    let sspj_loader = crate::SspjLoader::new(path);
    let sspj = match sspj_loader.load_ss_project() {
        Ok(sspj) => sspj,
        Err(error) => {
            println!("{:?} error: {:?}", path.as_os_str().to_string_lossy(), error);
            return Err(error);
        }
    };

    // pack sspj animation data to ssab flutbuffers structures
    let _a_list = sspj.anime_list.iter().map(|anime_pack| {
        let mut anime_binary = ssfb::ssab::ss::data::SsAnimeBinaryT::default();
        anime_binary.animation = None;
        anime_binary.cell_map = Some(convert_cellmapt_list(&sspj, &anime_pack.cellmap_names));
        anime_binary.effect = None;
        anime_binary.sound_list = None;
        anime_binary.font_bitmap = None;
        anime_binary.instance = None;

        anime_binary
    }).collect::<Vec<SsAnimeBinaryT>>();

    // pack sspj sequence data to ssqb flutbuffers structures
    let _q_list = sspj.sequence_list.iter().map(|_sequence| {

    });

    // convert ssab flutbuffers structures to binary data

    // convert ssqb flutbuffers structures to binary data

    Ok(HashMap::new())
}

pub fn convert_ss_project_to_ssab(path: &Path, config: ConverterConfig) -> Result<(), SspjError> {
    let ssab = convert_ss_project(path, config)?;

    let Some(parent) = path.parent() else {
        return Err(SspjError::FileError(std::io::Error::new(ErrorKind::NotFound, "")));
    };
    ssab.iter().for_each(|(key, value)| {
        let output_path = parent.join(key).with_extension("ssab");
        std::fs::write(output_path.as_path(), value).expect("TODO: panic message"); // TODO: FIXME
    });

    Ok(())
}
