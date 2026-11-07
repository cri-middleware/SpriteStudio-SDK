use crate::ssloader::SsLoader;
use crate::sspj_error::SspjError;
use crate::sstypes::SsSequenceType;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsSequenceItem {
    pub ref_anime_pack: String,
    pub ref_anime: String,
    pub repeat_count: i32,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsSequence {
    pub name: String,
    pub index: i32,
    #[serde(rename = "type")]
    pub sequence_type: SsSequenceType,
    #[serde(deserialize_with = "parse_ss_sequence_item_list")]
    pub list: Vec<SsSequenceItem>,
}
fn parse_ss_sequence_item_list<'de, D>(deserializer: D) -> Result<Vec<SsSequenceItem>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct SequenceItemList {
        value: Vec<SsSequenceItem>,
    }
    Ok(SequenceItemList::deserialize(deserializer)?.value)
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename = "SpriteStudioSequencePack")]
#[serde(rename_all = "camelCase")]
pub struct SsSequencePack {
    #[serde(rename = "@version")]
    pub version: String,
    pub name: String,
    #[serde(deserialize_with = "parse_ss_sequence_list")]
    pub sequence_list: Vec<SsSequence>,
}
impl SsSequencePack {
    pub fn find_sequence(&self, name: &str) -> Option<&SsSequence> {
        self.sequence_list.iter().find(|s| s.name == name)
    }
}
impl SsLoader for SsSequencePack {
    type Output = SsSequencePack;

    fn load(s: &str) -> Result<Self::Output, SspjError> {
        quick_xml::de::from_str::<Self::Output>(s).map_err(SspjError::XmlParseError)
    }
}

fn parse_ss_sequence_list<'de, D>(deserializer: D) -> Result<Vec<SsSequence>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct SequenceList {
        sequence: Vec<SsSequence>,
    }
    Ok(SequenceList::deserialize(deserializer)?.sequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use std::path::{Path, PathBuf};

    fn load_ssqe(path: &Path) {
        SsSequencePack::load_from_file(path).unwrap();
    }

    fn get_test_base_path() -> PathBuf {
        std::env::current_dir().expect("Failed to get the current directory").ancestors().nth(2).expect("Failed to get the base directory").to_path_buf()
    }

    #[test]
    fn parse_ssqe() {
        ["tests/overall/sequence.ssqe"].par_iter().for_each(|&f| {
            let p = get_test_base_path().join(f);
            load_ssqe(p.as_path());
        });
    }
}
