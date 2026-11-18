use crate::asset_path::{join_asset_path, normalize_asset_path};
use crate::parallel::*;
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
    #[serde(deserialize_with = "parse_name_list", default)]
    pub sequencepack_names: Vec<String>,

    #[serde(deserialize_with = "parse_name_list", default)]
    pub audiopack_names: Vec<String>,
    #[serde(deserialize_with = "parse_name_list", default)]
    pub charmap_names: Vec<String>,
    #[serde(rename = "ExternalTextures", deserialize_with = "parse_external_textures", default)]
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
        #[serde(default)]
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
    pub fn find_animation(&mut self, ref_anime_pack: &str, ref_anime: &str) -> Option<&crate::ssae::SsAnimation> {
        self.anime_list.iter().find(|ss_animepack| ss_animepack.name == ref_anime_pack).and_then(|anime_list| anime_list.anime_list.iter().find(|ss_anime| ss_anime.name == ref_anime))
    }
}

fn load_listed_from_file<L: crate::ssloader::SsLoader>(path: &std::path::Path, name: String) -> Result<L::Output, crate::sspj_error::SspjError> {
    L::load_from_file(path).map_err(|e| e.in_file(name))
}

fn load_listed_from_maps<L: crate::ssloader::SsLoader>(key: String, maps: &HashMap<String, String>) -> Result<L::Output, crate::sspj_error::SspjError> {
    L::load_from_maps(&key, maps).map_err(|e| e.in_file(key))
}

impl crate::ssloader::SsLoader for SsProject {
    type Output = SsProject;
    fn load_from_file(path: &std::path::Path) -> Result<Self::Output, crate::sspj_error::SspjError> {
        let Ok(xml_data) = std::fs::read_to_string(path) else {
            return Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to read a file: {:?}", path.to_string_lossy()))));
        };
        let Some(parent) = path.parent() else {
            return Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to get a parent directory: {:?}", path.parent()))));
        };
        let parent_path_buf = parent.to_path_buf();
        let mut sspj = quick_xml::de::from_str::<SsProject>(&xml_data).map_err(crate::sspj_error::SspjError::XmlParseError)?;

        let ((sound_list, (sequence_list, charmap_dic)), (cellmap_list, (effect_file_list, anime_list))) = join(
            || {
                join(
                    || {
                        par_iter!(sspj.audiopack_names)
                            .map(|audiopack| load_listed_from_file::<crate::ssse::SsAudioPack>(&parent_path_buf.join(audiopack), normalize_asset_path(audiopack)))
                            .collect::<Result<Vec<crate::ssse::SsAudioPack>, crate::sspj_error::SspjError>>()
                    },
                    || {
                        join(
                            || {
                                par_iter!(sspj.sequencepack_names)
                                    .map(|sequencepack| load_listed_from_file::<crate::ssqe::SsSequencePack>(&parent_path_buf.join(sequencepack), normalize_asset_path(sequencepack)))
                                    .collect::<Result<Vec<crate::ssqe::SsSequencePack>, crate::sspj_error::SspjError>>()
                            },
                            || {
                                par_iter!(sspj.charmap_names)
                                    .map(|charmap_name| {
                                        let path = parent_path_buf.join(charmap_name);
                                        let charmap_data = load_listed_from_file::<crate::sscharmap::SsCharMap>(&path, normalize_asset_path(charmap_name))?;
                                        Ok((charmap_name.to_string(), charmap_data))
                                    })
                                    .collect::<Result<HashMap<String, crate::sscharmap::SsCharMap>, crate::sspj_error::SspjError>>()
                            },
                        )
                    },
                )
            },
            || {
                join(
                    || {
                        let base = &sspj.settings.cell_map_base_directory;
                        let p = parent_path_buf.join(base);
                        par_iter!(sspj.cellmap_names).map(|cellmap| load_listed_from_file::<crate::ssce::SsCellMap>(&p.join(cellmap), join_asset_path(base, cellmap))).collect::<Result<Vec<crate::ssce::SsCellMap>, crate::sspj_error::SspjError>>()
                    },
                    || {
                        join(
                            || {
                                let base = &sspj.settings.effect_base_directory;
                                let p = parent_path_buf.join(base);
                                par_iter!(sspj.effect_file_names)
                                    .map(|effect_file| load_listed_from_file::<crate::ssee::SsEffectFile>(&p.join(effect_file), join_asset_path(base, effect_file)))
                                    .collect::<Result<Vec<crate::ssee::SsEffectFile>, crate::sspj_error::SspjError>>()
                            },
                            || {
                                let base = &sspj.settings.anime_base_directory;
                                let p = parent_path_buf.join(base);
                                par_iter!(sspj.animepack_names)
                                    .map(|animepack| load_listed_from_file::<crate::ssae::SsAnimePack>(&p.join(animepack), join_asset_path(base, animepack)))
                                    .collect::<Result<Vec<crate::ssae::SsAnimePack>, crate::sspj_error::SspjError>>()
                            },
                        )
                    },
                )
            },
        );

        sspj.sound_list = sound_list?;
        sspj.sequence_list = sequence_list?;
        sspj.charmap_dic = charmap_dic?;
        sspj.cellmap_list = cellmap_list?;
        sspj.effect_file_list = effect_file_list?;
        sspj.anime_list = anime_list?;

        Ok(sspj)
    }

    /// `maps` is keyed by path relative to the `.sspj`'s directory, in the form
    /// [`normalize_asset_path`] gives.
    fn load_from_maps(name: &str, maps: &std::collections::HashMap<String, String>) -> Result<Self::Output, crate::sspj_error::SspjError> {
        let Some(xml_data) = maps.get(name) else {
            return Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to read a file: {:?}", name))));
        };
        let mut sspj = quick_xml::de::from_str::<SsProject>(xml_data).map_err(crate::sspj_error::SspjError::XmlParseError)?;

        sspj.sound_list = par_iter!(sspj.audiopack_names).map(|audiopack| load_listed_from_maps::<crate::ssse::SsAudioPack>(normalize_asset_path(audiopack), maps)).collect::<Result<Vec<crate::ssse::SsAudioPack>, crate::sspj_error::SspjError>>()?;
        sspj.sequence_list =
            par_iter!(sspj.sequencepack_names).map(|sequence_name| load_listed_from_maps::<crate::ssqe::SsSequencePack>(normalize_asset_path(sequence_name), maps)).collect::<Result<Vec<crate::ssqe::SsSequencePack>, crate::sspj_error::SspjError>>()?;
        sspj.charmap_dic = par_iter!(sspj.charmap_names)
            .map(|charmap_name| {
                let charmap_data = load_listed_from_maps::<crate::sscharmap::SsCharMap>(normalize_asset_path(charmap_name), maps)?;
                Ok((charmap_name.to_string(), charmap_data))
            })
            .collect::<Result<HashMap<String, crate::sscharmap::SsCharMap>, crate::sspj_error::SspjError>>()?;
        sspj.cellmap_list = par_iter!(sspj.cellmap_names)
            .map(|cellmap_name| load_listed_from_maps::<crate::ssce::SsCellMap>(join_asset_path(&sspj.settings.cell_map_base_directory, cellmap_name), maps))
            .collect::<Result<Vec<crate::ssce::SsCellMap>, crate::sspj_error::SspjError>>()?;
        sspj.effect_file_list = par_iter!(sspj.effect_file_names)
            .map(|effect_file_name| load_listed_from_maps::<crate::ssee::SsEffectFile>(join_asset_path(&sspj.settings.effect_base_directory, effect_file_name), maps))
            .collect::<Result<Vec<crate::ssee::SsEffectFile>, crate::sspj_error::SspjError>>()?;
        sspj.anime_list = par_iter!(sspj.animepack_names)
            .map(|animepack_name| load_listed_from_maps::<crate::ssae::SsAnimePack>(join_asset_path(&sspj.settings.anime_base_directory, animepack_name), maps))
            .collect::<Result<Vec<crate::ssae::SsAnimePack>, crate::sspj_error::SspjError>>()?;

        Ok(sspj)
    }

    fn load(_: &str) -> Result<<Self as crate::ssloader::SsLoader>::Output, crate::sspj_error::SspjError> {
        Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, "Not support Sspj::load()")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssloader::SsLoader;
    use crate::sspj_error::SspjError;
    use std::path::{Path, PathBuf};

    fn load_sspj(path: &Path) {
        SsProject::load_from_file(path).unwrap();
    }

    fn get_test_base_path() -> PathBuf {
        std::env::current_dir().expect("Failed to get the current directory").ancestors().nth(2).expect("Failed to get the base directory").to_path_buf()
    }

    /// A host hands the project's files over by path, so a reference has to reach them
    /// the way a filesystem would: under its base directory, however it is spelled.
    #[test]
    fn load_from_maps_finds_what_load_from_file_finds() {
        let dir = get_test_base_path().join("tests/allPartsV7");
        let on_disk = SsProject::load_from_file(&dir.join("allPartsV7.sspj")).unwrap();

        let read = |name: &str| std::fs::read_to_string(dir.join(name)).unwrap();
        let mut maps = HashMap::new();
        for name in &on_disk.animepack_names {
            maps.insert(format!("anime/{name}"), read(name));
        }
        for name in &on_disk.cellmap_names {
            maps.insert(format!("cells/{name}"), read(name));
        }
        for name in &on_disk.effect_file_names {
            maps.insert(format!("effects/{name}"), read(name));
        }
        for name in on_disk.audiopack_names.iter().chain(&on_disk.charmap_names) {
            maps.insert(name.clone(), read(name));
        }
        let sspj = read("allPartsV7.sspj")
            .replace("<animeBaseDirectory></animeBaseDirectory>", "<animeBaseDirectory>anime</animeBaseDirectory>")
            .replace("<cellMapBaseDirectory></cellMapBaseDirectory>", "<cellMapBaseDirectory>./cells/</cellMapBaseDirectory>")
            .replace("<effectBaseDirectory></effectBaseDirectory>", "<effectBaseDirectory>effects</effectBaseDirectory>")
            .replace("<value>soundlist.ssse</value>", "<value>./soundlist.ssse</value>");
        maps.insert("allPartsV7.sspj".to_string(), sspj);

        let from_maps = SsProject::load_from_maps("allPartsV7.sspj", &maps).unwrap();

        assert_eq!(from_maps.settings.anime_base_directory, "anime");
        assert!(from_maps.anime_list == on_disk.anime_list);
        assert!(from_maps.cellmap_list == on_disk.cellmap_list);
        assert!(from_maps.effect_file_list == on_disk.effect_file_list);
        assert!(from_maps.sound_list == on_disk.sound_list);
        assert!(from_maps.charmap_dic == on_disk.charmap_dic);
    }

    fn sspj_listing(element: &str, base_directory: &str, file: &str) -> String {
        format!(
            r#"<SpriteStudioProject version="2.00.00"><name>p</name><settings><animeBaseDirectory>{base_directory}</animeBaseDirectory><effectBaseDirectory>{base_directory}</effectBaseDirectory><exportBaseDirectory>export</exportBaseDirectory><queryExportBaseDirectory>1</queryExportBaseDirectory><wrapMode>clamp</wrapMode><filterMode>linear</filterMode></settings><{element}><value>{file}</value></{element}></SpriteStudioProject>"#
        )
    }

    #[test]
    fn a_listed_file_that_fails_to_parse_is_named() {
        let maps = HashMap::from([("p.sspj".to_string(), sspj_listing("effectFileNames", "effects", "broken.ssee")), ("effects/broken.ssee".to_string(), "<SpriteStudioEffect".to_string())]);

        let e = SsProject::load_from_maps("p.sspj", &maps).unwrap_err();

        assert!(matches!(&e, SspjError::InFile { path, source } if path == "effects/broken.ssee" && matches!(**source, SspjError::XmlParseError(_))), "{e:?}");
        assert!(e.to_string().starts_with("effects/broken.ssee: XML Parse Error: "), "{e}");
    }

    #[test]
    fn a_listed_file_missing_on_disk_is_named() {
        let dir = std::env::temp_dir().join(format!("sspj-missing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("Failed to create the scratch directory");
        std::fs::write(dir.join("p.sspj"), sspj_listing("animepackNames", "", "missing.ssae")).expect("Failed to write the project");

        let e = SsProject::load_from_file(&dir.join("p.sspj")).unwrap_err();
        let _ = std::fs::remove_dir_all(&dir);

        assert!(matches!(&e, SspjError::InFile { path, source } if path == "missing.ssae" && matches!(**source, SspjError::FileError(_))), "{e:?}");
    }

    #[test]
    fn parse_sspj() {
        ["tests/ParticleEffect/ParticleEffect.sspj", "tests/allAttributeV7/allAttributeV7.sspj", "tests/allPartsV7/allPartsV7.sspj", "tests/overall/overall.sspj"].par_iter().for_each(|&f| {
            let p = get_test_base_path().join(f);
            load_sspj(p.as_path());
        });
    }
}
