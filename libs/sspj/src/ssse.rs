use serde::Deserialize;
use crate::ssloader::{SsLoader, SspjError};

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsSoundFile {
    #[serde(rename = "Alias")]
    pub alias: String,
    pub filename: String,
    pub msec: u32,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsSoundFileList {
    pub id: i32,
    #[serde(rename = "Alias")]
    pub alias: String,
    #[serde(deserialize_with = "parse_ss_soundfile")]
    pub sounds: Vec<SsSoundFile>,
}
fn parse_ss_soundfile<'de, D>(deserializer: D) -> Result<Vec<SsSoundFile>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize, PartialEq)]
    struct Sounds {
        value: Vec<SsSoundFile>,
    }
    Ok(Sounds::deserialize(deserializer)?.value)
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename = "SpriteStudioSoundList")]
#[serde(rename_all = "camelCase")]
pub struct SsAudioPack {
    #[serde(rename = "@version")]
    pub version: String,
    pub name: String,
    pub table: SsSoundFileList,
}
impl SsLoader for SsAudioPack {
    type Output = SsAudioPack;

    fn load(s: &str) -> Result<Self::Output, SspjError> {
        quick_xml::de::from_str::<Self::Output>(s).map_err(SspjError::XmlParseError)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use super::*;

    fn load_ssse(path: &Path) {
        let r = SsAudioPack::load_from_file(path);
        match r {
            Ok(ss_cell_map) => {
                println!("{:?}", ss_cell_map);
                assert!(true);
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
    fn parse_ssse() {
        let p = get_test_base_path().join("tests/overall/soundlist.ssse");
        load_ssse(&p.as_path());
    }
}