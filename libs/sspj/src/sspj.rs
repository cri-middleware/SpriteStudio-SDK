use std::collections::HashMap;
use serde::Deserialize;
use rayon::prelude::*;
use crate::ssae::SsAnimePack;
use crate::ssce::SsCellMap;
use crate::sscharmap::SsCharMap;
use crate::ssee::SsEffectFile;
use crate::ssloader::{SsLoader, SspjError};
use crate::ssqe::SsSequencePack;
use crate::ssse::SsAudioPack;
use crate::sstypes::{SsTexFilterMode, SsTexWrapMode};

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

        let sound_handle = {
            let parent_path = parent.to_path_buf();
            let audiopack_names = sspj.audiopack_names.clone();
            std::thread::spawn(move || {
                audiopack_names.par_iter().filter_map(|audiopack| {
                    SsAudioPack::load_from_file(&parent_path.join(audiopack)).ok()
                }).collect::<Vec<SsAudioPack>>()
            })
        };

        let sequence_handle = {
            let parent_path = parent.to_path_buf();
            let sequencepack_names = sspj.sequencepack_names.clone();
            std::thread::spawn(move || {
                sequencepack_names.par_iter().filter_map(|sequencepack| {
                    SsSequencePack::load_from_file(&parent_path.join(sequencepack)).ok()
                }).collect::<Vec<SsSequencePack>>()
            })
        };

        let charmap_handle = {
            let parent_path = parent.to_path_buf();
            let charmap_names = sspj.charmap_names.clone();
            std::thread::spawn(move || {
                charmap_names.par_iter().filter_map(|charmap_name| {
                    let Ok(charmap_data) = SsCharMap::load_from_file(&parent_path.join(charmap_name)) else {
                        return None;
                    };
                    Some((charmap_name.to_string(), charmap_data))
                }).collect::<HashMap<String, SsCharMap>>()
            })
        };

        let cellmap_handle = {
            let parent_path = parent.to_path_buf();
            let cellmap_names = sspj.cellmap_names.clone();
            std::thread::spawn(move || {
                cellmap_names.par_iter().filter_map(|cellmap| {
                    let ssce_path = &parent_path.join(cellmap);
                    let Ok(mut ssce) = SsCellMap::load_from_file(&ssce_path) else {
                        return None;
                    };
                    ssce.load_file_path = ssce_path.to_string_lossy().to_string();
                    Some(ssce)
                }).collect::<Vec<SsCellMap>>()
            })
        };

        let effect_handle = {
            let parent_path = parent.to_path_buf();
            let effect_file_names = sspj.effect_file_names.clone();
            std::thread::spawn(move || {
                effect_file_names.par_iter().filter_map(|effect_file| {
                    SsEffectFile::load_from_file(&parent_path.join(effect_file)).ok()
                }).collect::<Vec<SsEffectFile>>()
            })
        };

        let animepack_handle = {
            let parent_path = parent.to_path_buf();
            let animepack_names = sspj.animepack_names.clone();
            std::thread::spawn(move || {
                animepack_names.par_iter().filter_map(|animepack| {
                    SsAnimePack::load_from_file(&parent_path.join(animepack)).ok()
                }).collect::<Vec<SsAnimePack>>()
            })
        };

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
            .expect("Failed to get the current directory")
            .ancestors()
            .nth(2)
            .expect("Failed to get the base directory")
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