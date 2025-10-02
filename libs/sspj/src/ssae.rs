use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Deserializer};
use serde::de::Error;
use crate::ssattribute::SsAttributeKind;
use crate::ssloader::{SsLoader, SspjError};
use crate::sstypes::*;

#[derive(Deserialize, PartialEq, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAnimationSettings {
    pub fps: i32,
    pub frame_count: i32,
    pub canvas_size: SsPoint2,
    pub pivot: SsPoint2,
    pub start_frame: i32,
    pub end_frame: i32,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "UPPERCASE")]
 pub struct InheritRates {
    pub alph: bool,
    pub iflh: bool,
    pub iflv: bool,
    pub flph: bool,
    pub flpv: bool,
    pub hide: bool,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsPart {
    pub name: String,
    pub array_index: i32,
    pub parent_index: i32,
    #[serde(rename = "type")]
    pub part_type: SsPartType,
    pub bounds_type: SsBoundsType,
    pub inherit_type: SsInheritType,
    pub alpha_blend_type: SsBlendType,
    pub show: i32,
    pub locked: i32,
    #[serde(default)]
    pub color_label: String,
    pub mask_influence: bool,
    #[serde(rename = "ineheritRates")]
    pub inherit_rates: InheritRates,
    #[serde(default)]
    pub ref_anime_pack: String,
    #[serde(default)]
    pub ref_anime: String,
    #[serde(default)]
    pub bone_length: i32,
    #[serde(default)]
    pub bone_position: SsPoint2,
    #[serde(default)]
    pub bone_rotation: f32,
    #[serde(default)]
    pub weight_position: SsPoint2,
    #[serde(default)]
    pub weight_impact: f32,

    #[serde(default)]
    pub mesh_weight_type: i32,
    #[serde(default)]
    pub mesh_weight_strong: i32,

    #[serde(rename = "IKDepth", default)]
    pub ik_depth: i32,
    #[serde(rename = "IKRotationArrow", default)]
    pub ik_rotation_arrow: SsIkRotationArrow,

    // m_pPartValueInfo // TODO:
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsPartAnime {
    pub part_name: String,
    #[serde(deserialize_with = "parse_ss_attribute_list")]
    pub attributes: Vec<SsAttributeKind>,
}
fn parse_ss_attribute_list<'de, D>(deserializer: D) -> Result<Vec<SsAttributeKind>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PartAnimeList {
        #[serde(default)]
        attribute: Vec<SsAttributeKind>,
    }
    Ok(PartAnimeList::deserialize(deserializer)?.attribute)
}


#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsLabel {
    pub name: String,
    pub time: i32,
}

#[derive(Deserialize, PartialEq, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAnimation {
    pub name: String,
    pub override_settings: bool,
    pub settings: SsAnimationSettings,
    #[serde(deserialize_with = "parse_ss_part_anime_list")]
    pub part_animes: Vec<SsPartAnime>,
    #[serde(deserialize_with = "parse_ss_label_list")]
    pub labels: Vec<SsLabel>,
    pub is_setup: bool,
}
fn parse_ss_part_anime_list<'de, D>(deserializer: D) -> Result<Vec<SsPartAnime>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PartAnimeList {
        #[serde(default)]
        part_anime: Vec<SsPartAnime>,
    }
    Ok(PartAnimeList::deserialize(deserializer)?.part_anime)
}

fn parse_ss_label_list<'de, D>(deserializer: D) -> Result<Vec<SsLabel>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct LabelList {
        #[serde(default)]
        value: Vec<SsLabel>,
    }
    Ok(LabelList::deserialize(deserializer)?.value)
}

#[derive(PartialEq, Default, Debug)]
pub struct SsMeshBindInfo {
    pub weight: Vec<i32>,
    pub bone_index: Vec<i32>,
    pub offset: Vec<SsPoint2>,
    pub bind_bone_num: i32,
}
impl FromStr for SsMeshBindInfo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let array: Vec<f32> = s.split_whitespace().filter_map(|sub| sub.parse::<f32>().ok()).collect();
        if array.is_empty() {
            return Err("SsMeshBindInfo::from_str: array.len() <= 1".to_string());
        }
        let num: i32 = array[0] as i32;
        if (1 + num * 4) != array.len() as i32 {
            return Err("SsMeshBindInfo::from_str: invalid array length".to_string());
        }
        let mut weight_vec: Vec<i32> = Vec::new();
        let mut bone_index_vec: Vec<i32> = Vec::new();
        let mut offset_vec: Vec<SsPoint2> = Vec::new();
        for idx in 0..num {
            let offset: usize = (1 + idx * 4) as usize;

            let weight: i32 = array[offset] as i32;
            let bone_index: i32 = array[offset + 1] as i32;
            let x: f32 = array[offset + 2];
            let y: f32 = array[offset + 3];
            weight_vec.push(weight);
            bone_index_vec.push(bone_index);
            offset_vec.push(SsPoint2 { x, y });
        }
        Ok(SsMeshBindInfo {
            weight: weight_vec,
            bone_index: bone_index_vec,
            offset: offset_vec,
            bind_bone_num: num,
        })
    }
}
impl<'de> Deserialize<'de> for SsMeshBindInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Err(D::Error::custom("SsMeshBindInfo::deserialize: s.is_empty()"))
        } else {
            Self::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}

#[derive(PartialEq, Default, Debug)]
pub struct SsMeshBind {
    pub mesh_vertices_bind_array: Vec<SsMeshBindInfo>,
}
impl FromStr for SsMeshBind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s.split(",").collect::<Vec<&str>>();
        if items.len() == 0 {
            return Ok(Self::default());
        }
        let ss_mesh_bind_vec = items.iter().filter_map(|item| {
            if item.is_empty() { return None; }
            let Ok(r) = SsMeshBindInfo::from_str(item) else {
                return None;
            };
            Some(r)
        }).collect::<Vec<SsMeshBindInfo>>();
        Ok(SsMeshBind {
            mesh_vertices_bind_array: ss_mesh_bind_vec,
        })
    }
}

impl<'de> Deserialize<'de> for SsMeshBind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(Self::default())
        } else {
            Self::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}


#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsModel {
    #[serde(deserialize_with = "parse_part_list")]
    pub part_list: Vec<SsPart>,
    #[serde(default)]
    pub setup_animation: String,

    #[serde(deserialize_with = "parse_mesh_list")]
    pub mesh_list: Vec<SsMeshBind>,
    pub bone_list: HashMap<String, i32>,
}
fn parse_part_list<'de, D>(deserializer: D) -> Result<Vec<SsPart>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct PartList {
        value: Vec<SsPart>,
    }
    Ok(PartList::deserialize(deserializer)?.value)
}

fn parse_mesh_list<'de, D>(deserializer: D) -> Result<Vec<SsMeshBind>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct MeshList {
        #[serde(default)]
        value: Vec<SsMeshBind>,
    }
    Ok(MeshList::deserialize(deserializer)?.value)
}


#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename = "SpriteStudioAnimePack")]
#[serde(rename_all = "camelCase")]
pub struct SsAnimePack {
    #[serde(rename = "@version")]
    pub version: String,
    pub settings: SsAnimationSettings,
    pub name: String,
    #[serde(rename = "Model")]
    pub model: SsModel,
    #[serde(deserialize_with = "parse_cellmap_names")]
    pub cellmap_names: Vec<String>,
    #[serde(deserialize_with = "parse_anime_list")]
    pub anime_list: Vec<SsAnimation>,
}
impl SsLoader for SsAnimePack {
    type Output = SsAnimePack;

    fn load(s: &str) -> Result<Self::Output, SspjError> {
        let mut ssae = quick_xml::de::from_str::<Self::Output>(s).map_err(SspjError::XmlParseError)?;
        ssae.anime_list.iter().for_each(|anime| {
            if anime.is_setup {
                ssae.model.setup_animation = anime.name.clone();
            }
        });
        Ok(ssae)
    }
}

fn parse_cellmap_names<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct CellmapNames {
        #[serde(default)]
        value: Vec<String>,
    }
    Ok(CellmapNames::deserialize(deserializer)?.value)
}

fn parse_anime_list<'de, D>(deserializer: D) -> Result<Vec<SsAnimation>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct AnimeList {
        anime: Vec<SsAnimation>,
    }
    Ok(AnimeList::deserialize(deserializer)?.anime)
}


#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use rayon::prelude::*;
    use super::*;

    fn load_ssae(path : &Path) {
        let r = SsAnimePack::load_from_file(path);
        match r {
            Ok(ss_anime_pack) => {
                println!("{:?}", ss_anime_pack);
                assert!(true);
            }
            Err(e) => {
                assert!(false, "{:?}", e)
            }
        }
    }
    fn get_test_base_path() -> PathBuf {
        std::env::current_dir()
            .expect("Failed to get the current directory")
            .ancestors()
            .nth(2)
            .expect("Failed to get the base directory")
            .to_path_buf()
    }

    #[test]
    fn parse_ssae() {
        [
            "tests/overall/01-new.ssae",
            "tests/overall/Basic.ssae",
            "tests/overall/BlendMode.ssae",
            "tests/overall/Bounding.ssae",
            "tests/overall/DeprecatedFeatures.ssae",
            "tests/overall/Effect.ssae",
            "tests/overall/Instance.ssae",
            "tests/overall/Instance_SourceAnime.ssae",
            "tests/overall/Interpolation.ssae",
            "tests/overall/Interpolation_Easing_v1.ssae",
            "tests/overall/Mask.ssae",
            "tests/overall/Mesh.ssae",
            "tests/overall/NineSlice.ssae",
            "tests/overall/PartsColor.ssae",
            "tests/overall/Shader.ssae",
            "tests/overall/Shape.ssae",
            "tests/overall/Signal.ssae",
            "tests/overall/Sound.ssae",
            "tests/overall/Text.ssae",
            "tests/overall/TextureChange.ssae",
            "tests/overall/UserData.ssae",
        ].par_iter().for_each( |&f| {
            let p = get_test_base_path().join(f);
            load_ssae(&p.as_path());
        });
    }

    #[test]
    fn parse_ssae2() {
        [
            "tests/ParticleEffect/e001.ssae",
            "tests/ParticleEffect/e002.ssae",
            "tests/ParticleEffect/e003.ssae",
            "tests/ParticleEffect/e004.ssae",
            "tests/ParticleEffect/e005.ssae",
            "tests/ParticleEffect/e006.ssae",
            "tests/ParticleEffect/e007.ssae",
            "tests/ParticleEffect/e008.ssae",
            "tests/ParticleEffect/e009.ssae",
            "tests/ParticleEffect/e010.ssae",
            "tests/ParticleEffect/e011.ssae",
            "tests/ParticleEffect/e012.ssae",
            "tests/ParticleEffect/e013.ssae",
            "tests/ParticleEffect/e014.ssae",
            "tests/ParticleEffect/e015.ssae",
            "tests/ParticleEffect/e016.ssae",
            "tests/ParticleEffect/e017.ssae",
            "tests/ParticleEffect/e018.ssae",
            "tests/ParticleEffect/e019.ssae",
            "tests/ParticleEffect/e020.ssae",
            "tests/ParticleEffect/e021.ssae",
            "tests/ParticleEffect/e022.ssae",
            "tests/ParticleEffect/e023.ssae",
            "tests/ParticleEffect/e024.ssae",
            "tests/ParticleEffect/e025.ssae",
            "tests/ParticleEffect/e026.ssae",
            "tests/ParticleEffect/e027.ssae",
            "tests/ParticleEffect/e028.ssae",
            "tests/ParticleEffect/e029.ssae",
            "tests/ParticleEffect/e030.ssae",
            "tests/ParticleEffect/e031.ssae",
            "tests/ParticleEffect/e032.ssae",
            "tests/ParticleEffect/e033.ssae",
            "tests/ParticleEffect/e034.ssae",
            "tests/ParticleEffect/e035.ssae",
            "tests/ParticleEffect/e036.ssae",
            "tests/ParticleEffect/e037.ssae",
            "tests/ParticleEffect/e038.ssae",
            "tests/ParticleEffect/e039.ssae",
            "tests/ParticleEffect/e040a.ssae",
        ].par_iter().for_each( |&f| {
            let p = get_test_base_path().join(f);
            load_ssae(&p.as_path());
        });
    }

    #[test]
    fn parse_ssae3() {
        [
            "tests/allAttributeV7/alpha.ssae",
            "tests/allAttributeV7/boundr.ssae",
            "tests/allAttributeV7/cell.ssae",
            "tests/allAttributeV7/deform.ssae",
            "tests/allAttributeV7/deform_bone.ssae",
            "tests/allAttributeV7/easing.ssae",
            "tests/allAttributeV7/effect.ssae",
            "tests/allAttributeV7/hide.ssae",
            "tests/allAttributeV7/imgflip.ssae",
            "tests/allAttributeV7/instance.ssae",
            "tests/allAttributeV7/instance_flag.ssae",
            "tests/allAttributeV7/iptype.ssae",
            "tests/allAttributeV7/loalpha.ssae",
            "tests/allAttributeV7/loscl.ssae",
            "tests/allAttributeV7/mask.ssae",
            "tests/allAttributeV7/partsColor.ssae",
            "tests/allAttributeV7/pivot.ssae",
            "tests/allAttributeV7/pos.ssae",
            "tests/allAttributeV7/prio.ssae",
            "tests/allAttributeV7/rot.ssae",
            "tests/allAttributeV7/scl.ssae",
            "tests/allAttributeV7/setupdata.ssae",
            "tests/allAttributeV7/size.ssae",
            "tests/allAttributeV7/startend.ssae",
            "tests/allAttributeV7/textureChange.ssae",
            "tests/allAttributeV7/userdata.ssae",
            "tests/allAttributeV7/uvs.ssae",
            "tests/allAttributeV7/uvt.ssae",
            "tests/allAttributeV7/vertex.ssae",
            "tests/allPartsV7/effect.ssae",
            "tests/allPartsV7/instanse.ssae",
            "tests/allPartsV7/mask.ssae",
            "tests/allPartsV7/mesh.ssae",
            "tests/allPartsV7/nineslice.ssae",
            "tests/allPartsV7/normal.ssae",
            "tests/allPartsV7/null.ssae",
            "tests/allPartsV7/root.ssae",
            "tests/allPartsV7/shape.ssae",
            "tests/allPartsV7/sound.ssae",
            "tests/allPartsV7/text.ssae",
        ].par_iter().for_each( |&f| {
            let p = get_test_base_path().join(f);
            load_ssae(&p.as_path());
        });
    }
}
