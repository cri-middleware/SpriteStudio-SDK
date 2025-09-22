use quick_xml::DeError;
use serde::Deserialize;
use crate::sspj::ssae::SsAnimePack;
use crate::sspj::ssce::SsCellMap;
use crate::sspj::ssee::SsEffectFile;
use crate::sspj::ssloader::SsLoader;
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
    charmap_naames: Vec<String>,
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
    // CharMapDic			charmapDic;		//!< キャラクターマップ(bitmap font)

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
    fn load_from_file(path: &std::path::Path) -> Result<Self::Output, DeError> {
        let Ok(xml_data) = std::fs::read_to_string(path) else {
            return Err(quick_xml::DeError::Custom(format!("Failed to read file: {}", path.display())))
        };

        let parent = path.parent().expect("Failed to get parent directory");
        let mut sspj = quick_xml::de::from_str::<SsProject>(&xml_data)?;

        sspj.anime_list = sspj.animepack_names.iter().filter_map(|animepack| {
            SsAnimePack::load_from_file(&parent.join(animepack)).ok()
        }).collect::<Vec<SsAnimePack>>();
        sspj.cellmap_list = sspj.cellmap_names.iter().filter_map(|cellmap| {
            SsCellMap::load_from_file(&parent.join(cellmap)).ok()
        }).collect::<Vec<SsCellMap>>();
        sspj.effect_file_list = sspj.effect_file_names.iter().filter_map(|effect_file| {
            SsEffectFile::load_from_file(&parent.join(effect_file)).ok()
        }).collect::<Vec<SsEffectFile>>();
        sspj.sequence_list = sspj.sequencepack_names.iter().filter_map(|sequencepack| {
            SsSequencePack::load_from_file(&parent.join(sequencepack)).ok()
        }).collect::<Vec<SsSequencePack>>();
        sspj.sound_list = sspj.audiopack_names.iter().filter_map(|audiopack| {
            SsAudioPack::load_from_file(&parent.join(audiopack)).ok()
        }).collect::<Vec<SsAudioPack>>();

        Ok(sspj)
    }
    fn load(_: &str) -> Result<<Self as SsLoader>::Output, DeError> { todo!() }
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
            //"tests/allAttributeV7/allAttributeV7.sspj",
            //"tests/allPartsV7/allPartsV7.sspj",
            //"tests/overall/overall.sspj",
        ].iter().for_each(|&f| {
            let p = get_test_base_path().join(f);
            load_sspj(&p.as_path());
        });
    }
}