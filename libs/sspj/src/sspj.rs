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
    pub wrap_mode: crate::sstypes::SsTexWrapMode,
    pub filter_mode: crate::sstypes::SsTexFilterMode,

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
    pub anime_list: Vec<crate::ssae::SsAnimePack>,
    #[serde(default)]
    pub cellmap_list: Vec<crate::ssce::SsCellMap>,
    #[serde(default)]
    pub effect_file_list: Vec<crate::ssee::SsEffectFile>,
    #[serde(default)]
    pub sequence_list: Vec<crate::ssqe::SsSequencePack>,
    #[serde(default)]
    pub sound_list: Vec<crate::ssse::SsAudioPack>,

    #[serde(default)]
    pub charmap_dic: HashMap<String, crate::sscharmap::SsCharMap>,
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
    ) -> Option<&crate::ssae::SsAnimation> {
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

impl crate::ssloader::SsLoader for SsProject {
    type Output = SsProject;
    fn load_from_file(path: &std::path::Path) -> Result<Self::Output, crate::sspj_error::SspjError> {
        let Ok(xml_data) = std::fs::read_to_string(path) else {
            return Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to read a file: {:?}", path.to_string_lossy()))))
        };
        let Some(parent) = path.parent() else {
            return Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to get a parent directory: {:?}", path.parent()))))
        };
        let parent_path_buf = parent.to_path_buf();
        let mut sspj = quick_xml::de::from_str::<SsProject>(&xml_data).map_err(crate::sspj_error::SspjError::XmlParseError)?;

        let ((sound_list, (sequence_list, charmap_dic)), (cellmap_list, (effect_file_list, anime_list))) = rayon::join(
            || {
                rayon::join(
                    || {
                        sspj.audiopack_names.par_iter().map(|audiopack| {
                            crate::ssse::SsAudioPack::load_from_file(&parent_path_buf.join(audiopack))
                        }).collect::<Result<Vec<crate::ssse::SsAudioPack>, crate::sspj_error::SspjError>>()
                    },
                    || {
                        rayon::join(
                            || {
                                sspj.sequencepack_names.par_iter().map(|sequencepack| {
                                    crate::ssqe::SsSequencePack::load_from_file(&parent_path_buf.join(sequencepack))
                                }).collect::<Result<Vec<crate::ssqe::SsSequencePack>, crate::sspj_error::SspjError>>()
                            },
                            || {
                                sspj.charmap_names.par_iter().map(|charmap_name| {
                                    let path = parent_path_buf.join(charmap_name);
                                    let charmap_data = crate::sscharmap::SsCharMap::load_from_file(&path)?;
                                    Ok((charmap_name.to_string(), charmap_data))
                                }).collect::<Result<HashMap<String, crate::sscharmap::SsCharMap>, crate::sspj_error::SspjError>>()
                            }
                        )
                    }
                )
            },
            || {
                rayon::join(
                    || {
                        let p = parent_path_buf.join(&sspj.settings.cell_map_base_directory);
                        sspj.cellmap_names.par_iter().map(|cellmap| {
                            crate::ssce::SsCellMap::load_from_file(&p.join(cellmap))
                        }).collect::<Result<Vec<crate::ssce::SsCellMap>, crate::sspj_error::SspjError>>()
                    },
                    || {
                        rayon::join(
                            || {
                                let p = parent_path_buf.join(&sspj.settings.effect_base_directory);
                                sspj.effect_file_names.par_iter().map(|effect_file| {
                                    crate::ssee::SsEffectFile::load_from_file(&p.join(effect_file))
                                }).collect::<Result<Vec<crate::ssee::SsEffectFile>, crate::sspj_error::SspjError>>()
                            },
                            || {
                                let p = parent_path_buf.join(&sspj.settings.anime_base_directory);
                                sspj.animepack_names.par_iter().map(|animepack| {
                                    crate::ssae::SsAnimePack::load_from_file(&p.join(animepack))
                                }).collect::<Result<Vec<crate::ssae::SsAnimePack>, crate::sspj_error::SspjError>>()
                            }
                        )
                    }
                )
            }
        );

        sspj.sound_list = sound_list?;
        sspj.sequence_list = sequence_list?;
        sspj.charmap_dic = charmap_dic?;
        sspj.cellmap_list = cellmap_list?;
        sspj.effect_file_list = effect_file_list?;
        sspj.anime_list = anime_list?;

        Ok(sspj)
    }

    fn load_from_maps(name: &str, maps: &std::collections::HashMap<String, String>) -> Result<Self::Output, crate::sspj_error::SspjError> {
        let Some(xml_data) = maps.get(name) else {
            return Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to read a file: {:?}", name))));
        };
        let mut sspj = quick_xml::de::from_str::<SsProject>(xml_data).map_err(crate::sspj_error::SspjError::XmlParseError)?;

        sspj.sound_list = sspj.audiopack_names.par_iter().map(|audiopack| { crate::ssse::SsAudioPack::load_from_maps(audiopack, maps)  }).collect::<Result<Vec<crate::ssse::SsAudioPack>, crate::sspj_error::SspjError>>()?;
        sspj.sequence_list = sspj.sequencepack_names.par_iter().map(|sequence_name| { crate::ssqe::SsSequencePack::load_from_maps(sequence_name, maps)  }).collect::<Result<Vec<crate::ssqe::SsSequencePack>, crate::sspj_error::SspjError>>()?;
        sspj.charmap_dic = sspj.charmap_names.par_iter().map(|charmap_name| {
            let charmap_data = crate::sscharmap::SsCharMap::load_from_maps(charmap_name, maps)?;
            Ok((charmap_name.to_string(), charmap_data))
        }).collect::<Result<HashMap<String, crate::sscharmap::SsCharMap>, crate::sspj_error::SspjError>>()?;
        sspj.cellmap_list = sspj.cellmap_names.par_iter().map(|cellmap_name| { crate::ssce::SsCellMap::load_from_maps(cellmap_name, maps)  }).collect::<Result<Vec<crate::ssce::SsCellMap>, crate::sspj_error::SspjError>>()?;
        sspj.effect_file_list = sspj.effect_file_names.par_iter().map(|effect_file_name| { crate::ssee::SsEffectFile::load_from_maps(effect_file_name, maps)  }).collect::<Result<Vec<crate::ssee::SsEffectFile>, crate::sspj_error::SspjError>>()?;
        sspj.anime_list = sspj.animepack_names.par_iter().map(|animepack_name| { crate::ssae::SsAnimePack::load_from_maps(animepack_name, maps)  }).collect::<Result<Vec<crate::ssae::SsAnimePack>, crate::sspj_error::SspjError>>()?;

        Ok(sspj)
    }

    fn load(_: &str) -> Result<<Self as crate::ssloader::SsLoader>::Output, crate::sspj_error::SspjError> {
        Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, "Not support Sspj::load()")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use crate::ssloader::SsLoader;

    fn load_sspj(path: &Path) {
        let r = SsProject::load_from_file(path);
        match r {
            Ok(sspj) => {
                println!("{:?}", sspj);
            }
            Err(e) => {
                panic!("{:?}", e);
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
            load_sspj(p.as_path());
        });
    }
}
