use serde::Deserialize;
use crate::sspj::ssloader::{SsLoader, SspjError};
use crate::sspj::sstypes::*;

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SsMeshDivType {
    Unknown,
    #[serde(rename = "polyline")]
    PolylineBase,
    Boxdiv,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsCell {
    pub name: String,
    pub pos: SsPoint2,
    pub size: SsPoint2,
    pub pivot: SsPoint2,
    pub rotated: bool,
    #[serde(default)]
    pub parent_size: SsPoint2,
    #[serde(rename = "ismesh")]
    pub is_mesh: bool,

    #[serde(deserialize_with = "parse_ss_point2_list")]
    pub inner_point: Vec<SsPoint2>,
    #[serde(deserialize_with = "parse_ss_point2_list")]
    pub outer_point: Vec<SsPoint2>,

    #[serde(deserialize_with = "parse_ss_point2_list")]
    pub mesh_point_list: Vec<SsPoint2>,

    #[serde(deserialize_with = "parse_ss_point3_list")]
    pub mesh_tri_list: Vec<SsPoint3>,

    #[serde(rename = "divtype")]
    pub div_type: SsMeshDivType,

    #[serde(rename = "divw", default)]
    pub div_w: i32,
    #[serde(rename = "divh", default)]
    pub div_h: i32,
}
fn parse_ss_point2_list<'de, D>(deserializer: D) -> Result<Vec<SsPoint2>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct Point2List {
        #[serde(default)]
        value: Vec<SsPoint2>,
    }
     Ok(Point2List::deserialize(deserializer)?.value)
}

fn parse_ss_point3_list<'de, D>(deserializer: D) -> Result<Vec<SsPoint3>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct Point3List {
        #[serde(default)]
        value: Vec<SsPoint3>,
    }
    Ok(Point3List::deserialize(deserializer)?.value)
}


#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename = "SpriteStudioCellMap")]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SsCellMap {
    #[serde(rename = "@version")]
    pub version: String,
    pub name: String,
    pub image_path: String,
    pub pixel_size: SsPoint2,
    pub override_tex_settings: bool,
    pub wrap_mode: SsTexWrapMode,
    pub filter_mode: SsTexFilterMode,

    #[serde(deserialize_with = "parse_ss_cells")]
    pub cells: Vec<SsCell>,

    #[serde(default)]
    pub f_name: String,
    #[serde(default)]
    pub load_file_path: String,
}
impl SsLoader for SsCellMap {
    type Output = SsCellMap;

    fn load(s: &str) -> Result<Self::Output, SspjError> {
        let r = quick_xml::de::from_str(s);
        match r {
            Ok(ss_cell_map) => Ok(ss_cell_map),
            Err(e) => Err(SspjError::ParseError(e)),
        }
    }

}
fn parse_ss_cells<'de, D>(deserializer: D) -> Result<Vec<SsCell>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct Cells {
        cell: Vec<SsCell>,
    }
    Ok(Cells::deserialize(deserializer)?.cell)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use rayon::prelude::*;
    use super::*;

    fn load_ssce(path: &Path) {
        let r = SsCellMap::load_from_file(path);
        match r {
            Ok(ss_cell_map) => {
                println!("{:?}", ss_cell_map);
                assert!(true)
            }
            Err(e) => {
                assert!(false, "{:?}", e);
            }
        }
    }

    fn get_test_base_path() -> PathBuf {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .ancestors()
            .nth(2)
            .expect("Failed to get base directory")
            .to_path_buf()
    }

    #[test]
    fn parse_ssce() {
        [
            "tests/overall/9slice_64tile.ssce",
            "tests/overall/box_00_00.ssce",
            "tests/overall/common.ssce",
        ].par_iter().for_each(|&f| {
            let p = get_test_base_path().join(f);
            load_ssce(&p.as_path());
        });
    }
}
