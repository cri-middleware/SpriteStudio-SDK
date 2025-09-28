use std::collections::HashMap;
use serde::Deserialize;
use rayon::prelude::*;
use crate::sspj::ssae::SsAnimePack;
use crate::sspj::ssce::SsCellMap;
use crate::sspj::sscharmap::SsCharMap;
use crate::sspj::ssee::SsEffectFile;
use crate::sspj::ssloader::{SsLoader, SspjError};
use crate::sspj::ssqe::SsSequencePack;
use crate::sspj::ssse::SsAudioPack;
use crate::sspj::sstypes::{SsTexFilterMode, SsTexWrapMode};

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsProjectSetting {
    #[serde(default)]
    anime_base_directory: String,
    #[serde(default)]
    cell_map_base_directory: String,
    #[serde(default)]
    image_base_directory: String,
    #[serde(default)]
    effect_base_directory: String,

    export_base_directory: String,
    query_export_base_directory: bool,
    wrap_mode: SsTexWrapMode,
    filter_mode: SsTexFilterMode,
    vertex_anime_float: i32,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct TextureInfo {
    #[serde(rename = "PathName")]
    path_name: String,
    index: i32,
    flags: i32,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename = "SpriteStudioProject")]
#[serde(rename_all = "camelCase")]
pub struct SsProject {
    #[serde(rename = "@version")]
    version: String,
    name: String,
    settings: SsProjectSetting,

    #[serde(deserialize_with = "parse_name_list", default)]
    cellmap_names: Vec<String>,
    #[serde(deserialize_with = "parse_name_list", default)]
    animepack_names: Vec<String>,
    #[serde(deserialize_with = "parse_name_list", default)]
    effect_file_names: Vec<String>,
    //#[serde(deserialize_with = "parse_name_list", default)]
    //texture_list: Vec<String>,
    #[serde(deserialize_with = "parse_name_list", default)]
    sequencepack_names: Vec<String>,

    #[serde(deserialize_with = "parse_name_list", default)]
    audiopack_names: Vec<String>,
    #[serde(deserialize_with = "parse_name_list", default)]
    charmap_names: Vec<String>,
    #[serde(rename = "ExternalTextures", deserialize_with = "parse_external_textures", default)]
    external_textures: Vec<TextureInfo>,

    #[serde(default)]
    anime_list: Vec<SsAnimePack>,
    #[serde(default)]
    cellmap_list: Vec<SsCellMap>,
    #[serde(default)]
    effect_file_list: Vec<SsEffectFile>,
    #[serde(default)]
    sequence_list: Vec<SsSequencePack>,
    #[serde(default)]
    sound_list: Vec<SsAudioPack>,

    #[serde(default)]
    charmap_dic: HashMap<String, SsCharMap>,

}
fn parse_name_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct List {
        value: Vec<String>,
    }
    List::deserialize(deserializer).map(|l| l.value)
}

fn parse_external_textures<'de, D>(deserializer: D) -> Result<Vec<TextureInfo>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize, Default)]
    struct Files {
        #[serde(default)]
        files: File,
    }
    #[derive(Deserialize, Default)]
    struct File {
        #[serde(default)]
        value: Vec<TextureInfo>,
    }
    Files::deserialize(deserializer).map(|l| l.files.value)
}

impl SsLoader for SsProject {
    type Output = SsProject;
    fn load_from_file(path: &std::path::Path) -> Result<Self::Output, SspjError> {
        let xml_data = std::fs::read_to_string(path).map_err(SspjError::FileError)?;
        let parent = path.parent().expect("Failed to get a parent directory");
        let mut sspj = quick_xml::de::from_str::<SsProject>(&xml_data).map_err(SspjError::XmlParseError)?;

        let mut parent_path: std::path::PathBuf;

        parent_path = parent.to_path_buf();
        let audiopack_names = sspj.audiopack_names.clone();
        let sound_handle = std::thread::spawn(move || {
            audiopack_names.par_iter().filter_map(|audiopack| {
                SsAudioPack::load_from_file(&parent_path.join(audiopack)).ok()
            }).collect::<Vec<SsAudioPack>>()
        });

        parent_path = parent.to_path_buf();
        let sequencepack_names = sspj.sequencepack_names.clone();
        let sequence_handle = std::thread::spawn(move || {
            sequencepack_names.par_iter().filter_map(|sequencepack| {
                SsSequencePack::load_from_file(&parent_path.join(sequencepack)).ok()
            }).collect::<Vec<SsSequencePack>>()
        });

        parent_path = parent.to_path_buf();
        let charmap_names = sspj.charmap_names.clone();
        let charmap_handle = std::thread::spawn(move || {
            charmap_names.par_iter().filter_map(|charmap_name| {
                let Ok(charmap_data) = SsCharMap::load_from_file(&parent_path.join(charmap_name)) else {
                    return None;
                };
                Some((charmap_name.to_string(), charmap_data))
            }).collect::<HashMap<String, SsCharMap>>()
        });

        parent_path = parent.to_path_buf();
        let cellmap_names = sspj.cellmap_names.clone();
        let cellmap_handle = std::thread::spawn(move || {
            cellmap_names.par_iter().filter_map(|cellmap| {
                SsCellMap::load_from_file(&parent_path.join(cellmap)).ok()
            }).collect::<Vec<SsCellMap>>()
        });

        parent_path = parent.to_path_buf();
        let effect_file_names = sspj.effect_file_names.clone();
        let effect_handle = std::thread::spawn(move || {
            effect_file_names.par_iter().filter_map(|effect_file| {
                SsEffectFile::load_from_file(&parent_path.join(effect_file)).ok()
            }).collect::<Vec<SsEffectFile>>()
        });

        parent_path = parent.to_path_buf();
        let animepack_names = sspj.animepack_names.clone();
        let animepack_handle = std::thread::spawn(move || {
            animepack_names.par_iter().filter_map(|animepack| {
                SsAnimePack::load_from_file(&parent_path.join(animepack)).ok()
            }).collect::<Vec<SsAnimePack>>()
        });

        sspj.sound_list = sound_handle.join().unwrap();
        sspj.sequence_list = sequence_handle.join().unwrap();
        sspj.charmap_dic = charmap_handle.join().unwrap();
        sspj.cellmap_list = cellmap_handle.join().unwrap();
        sspj.effect_file_list = effect_handle.join().unwrap();
        sspj.anime_list = animepack_handle.join().unwrap();

        Ok(sspj)
    }

    fn load(_: &str) -> Result<<Self as SsLoader>::Output, SspjError> {
        panic!("Not support Sspj::load()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn load_sspj(path: &Path) {
        let r = SsProject::load_from_file(path);
        match r {
            Ok(sspj) => {
                println!("{:?}", sspj);
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
    fn parse_sspj() {
        [
            "tests/ParticleEffect/ParticleEffect.sspj",
            "tests/allAttributeV7/allAttributeV7.sspj",
            "tests/allPartsV7/allPartsV7.sspj",
            "tests/overall/overall.sspj",
        ].par_iter().for_each(|&f| {
            let p = get_test_base_path().join(f);
            load_sspj(&p.as_path());
        });
    }
}