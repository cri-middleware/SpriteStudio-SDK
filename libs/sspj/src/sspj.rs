use crate::ssae::{SsAnimation, SsAnimePack};
use crate::ssce::SsCellMap;
use crate::sscharmap::SsCharMap;
use crate::ssee::SsEffectFile;
use crate::ssloader::SsLoader;
use crate::ssqe::SsSequencePack;
use crate::ssse::SsAudioPack;
use crate::sstypes::{SsTexFilterMode, SsTexWrapMode};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsProjectSetting {
    #[serde(default)]
    pub anime_base_directory: String,
    #[serde(default)]
    pub cell_map_base_directory: String,
    #[serde(default)]
    pub image_base_directory: String,
    #[serde(default)]
    pub effect_base_directory: String,

    pub export_base_directory: String,
    pub query_export_base_directory: bool,
    pub wrap_mode: SsTexWrapMode,
    pub filter_mode: SsTexFilterMode,

    #[serde(default)]
    pub vertex_anime_float: bool,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct TextureInfo {
    #[serde(rename = "PathName")]
    pub path_name: String,
    pub index: i32,
    pub flags: i32,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename = "SpriteStudioProject")]
#[serde(rename_all = "camelCase")]
pub struct SsProject {
    #[serde(rename = "@version")]
    pub version: String,
    pub name: String,
    pub settings: SsProjectSetting,

    #[serde(deserialize_with = "parse_name_list", default)]
    pub cellmap_names: Vec<String>,
    #[serde(deserialize_with = "parse_name_list", default)]
    pub animepack_names: Vec<String>,
    #[serde(deserialize_with = "parse_name_list", default)]
    pub effect_file_names: Vec<String>,
    //#[serde(deserialize_with = "parse_name_list", default)]
    //texture_list: Vec<String>,
    #[serde(deserialize_with = "parse_name_list", default)]
    pub sequencepack_names: Vec<String>,

    #[serde(deserialize_with = "parse_name_list", default)]
    pub audiopack_names: Vec<String>,
    #[serde(deserialize_with = "parse_name_list", default)]
    pub charmap_names: Vec<String>,
    #[serde(
        rename = "ExternalTextures",
        deserialize_with = "parse_external_textures",
        default
    )]
    pub external_textures: Vec<TextureInfo>,

    #[serde(default)]
    pub anime_list: Vec<SsAnimePack>,
    #[serde(default)]
    pub cellmap_list: Vec<SsCellMap>,
    #[serde(default)]
    pub effect_file_list: Vec<SsEffectFile>,
    #[serde(default)]
    pub sequence_list: Vec<SsSequencePack>,
    #[serde(default)]
    pub sound_list: Vec<SsAudioPack>,

    #[serde(default)]
    pub charmap_dic: HashMap<String, SsCharMap>,
}
fn parse_name_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct List {
        value: Vec<String>,
    }
    List::deserialize(deserializer).map(|l| l.value)
}

fn parse_external_textures<'de, D>(deserializer: D) -> Result<Vec<TextureInfo>, D::Error>
where
    D: serde::Deserializer<'de>,
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
impl SsProject {
    pub fn find_animation(
        &mut self,
        ref_anime_pack: &str,
        ref_anime: &str,
    ) -> Option<&SsAnimation> {
        self.anime_list
            .iter()
            .find(|ss_animepack| ss_animepack.name == ref_anime_pack)
            .and_then(|anime_list| {
                anime_list
                    .anime_list
                    .iter()
                    .find(|ss_anime| ss_anime.name == ref_anime)
            })
    }
}

impl SsLoader for SsProject {
    type Output = SsProject;
    fn load_from_file(path: &std::path::Path) -> Result<Self::Output, crate::sspj_error::SspjError> {
        let Ok(xml_data) = std::fs::read_to_string(path) else {
            return Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to read a file: {:?}", path.to_string_lossy()))))
        };
        let Some(parent) = path.parent() else {
            return Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to get a parent directory: {:?}", path.parent()))))
        };
        let mut sspj = quick_xml::de::from_str::<SsProject>(&xml_data).map_err(crate::sspj_error::SspjError::XmlParseError)?;

        let sound_handle = {
            let parent_path = parent.to_path_buf();
            let audiopack_names = sspj.audiopack_names.clone();
            std::thread::spawn(move || {
                audiopack_names.par_iter().map(|audiopack| {
                    SsAudioPack::load_from_file(&parent_path.join(audiopack))
                }).collect::<Result<Vec<SsAudioPack>, crate::sspj_error::SspjError>>()
            })
        };

        let sequence_handle = {
            let parent_path = parent.to_path_buf();
            let sequencepack_names = sspj.sequencepack_names.clone();
            std::thread::spawn(move || {
                sequencepack_names.par_iter().map(|sequencepack| {
                    SsSequencePack::load_from_file(&parent_path.join(sequencepack))
                }).collect::<Result<Vec<SsSequencePack>, crate::sspj_error::SspjError>>()
            })
        };

        let charmap_handle = {
            let parent_path = parent.to_path_buf();
            let charmap_names = sspj.charmap_names.clone();
            std::thread::spawn(move || {
                charmap_names.par_iter().map(|charmap_name| {
                    let path = parent_path.join(charmap_name);
                    let charmap_data = SsCharMap::load_from_file(&path).map_err(|e| e)?;
                    Ok((charmap_name.to_string(), charmap_data))
                }).collect::<Result<HashMap<String, SsCharMap>, crate::sspj_error::SspjError>>()
            })
        };
        
        let cellmap_handle = {
            let parent_path = parent.to_path_buf().join(&sspj.settings.cell_map_base_directory);
            let cellmap_names = sspj.cellmap_names.clone();
            std::thread::spawn(move || {
                cellmap_names.par_iter().map(|cellmap| {
                    let ssce_path = parent_path.join(cellmap);
                    let mut ssce = SsCellMap::load_from_file(&ssce_path)?;
                    ssce.load_file_path = ssce_path.file_name().unwrap().to_string_lossy().to_string();
                    Ok(ssce)
                }).collect::<Result<Vec<SsCellMap>, crate::sspj_error::SspjError>>()
            })
        };

        let effect_handle = {
            let parent_path = parent.to_path_buf().join(&sspj.settings.effect_base_directory);
            let effect_file_names = sspj.effect_file_names.clone();
            std::thread::spawn(move || {
                effect_file_names.par_iter().map(|effect_file| {
                    SsEffectFile::load_from_file(&parent_path.join(effect_file))
                }).collect::<Result<Vec<SsEffectFile>, crate::sspj_error::SspjError>>()
            })
        };

        let animepack_handle = {
            let parent_path = parent.to_path_buf().join(&sspj.settings.anime_base_directory);
            let animepack_names = sspj.animepack_names.clone();
            std::thread::spawn(move || {
                animepack_names.par_iter().map(|animepack| {
                    SsAnimePack::load_from_file(&parent_path.join(animepack))
                }).collect::<Result<Vec<SsAnimePack>, crate::sspj_error::SspjError>>()
            })
        };

        sspj.sound_list = sound_handle.join().unwrap()?;
        sspj.sequence_list = sequence_handle.join().unwrap()?;
        sspj.charmap_dic = charmap_handle.join().unwrap()?;
        sspj.cellmap_list = cellmap_handle.join().unwrap()?;
        sspj.effect_file_list = effect_handle.join().unwrap()?;
        sspj.anime_list = animepack_handle.join().unwrap()?;

        Ok(sspj)
    }

    fn load(_: &str) -> Result<<Self as SsLoader>::Output, crate::sspj_error::SspjError> {
        Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, "Not support Sspj::load()")))
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
        ]
        .par_iter()
        .for_each(|&f| {
            let p = get_test_base_path().join(f);
            load_sspj(&p.as_path());
        });
    }
}
