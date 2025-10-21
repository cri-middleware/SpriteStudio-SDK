use std::collections::HashMap;
use serde::Deserialize;
use crate::ssloader::{SsLoader, SspjError};
use crate::sstypes::{SsIRect, SsPoint2};

pub trait CharMapLoader {
    type Output;
    fn load(parts: &HashMap<String, String>) -> Result<Self::Output, SspjError>;
}

#[derive(Deserialize, PartialEq, Default, Debug)]
pub struct SsCharMapInfo {
    pub face: String,
    pub unicode: bool,
    pub smooth: bool,
}
impl CharMapLoader for SsCharMapInfo {
    type Output = SsCharMapInfo;

    fn load(parts: &HashMap<String, String>) -> Result<Self::Output, SspjError> {
        Ok(SsCharMapInfo {
            face: parts["face"].trim_matches('"').to_string(),
            unicode: parts["unicode"].as_str() == "1",
            smooth: parts["smooth"].as_str() == "1",
        })
    }
}

#[derive(Deserialize, PartialEq, Default, Debug)]
pub struct SsCharMapCommon {
    pub line_height: i32,
    pub base: i32,
    pub scale_w: f32,
    pub scale_h: f32,
    pub pages: i32,
}
impl CharMapLoader for SsCharMapCommon {
    type Output = SsCharMapCommon;

    fn load(parts: &HashMap<String, String>) -> Result<Self::Output, SspjError> {
        Ok(SsCharMapCommon {
            line_height: parts["lineHeight"].parse::<i32>().unwrap(),
            base: parts["base"].parse::<i32>().unwrap(),
            scale_w: parts["scaleW"].parse::<f32>().unwrap(),
            scale_h: parts["scaleH"].parse::<f32>().unwrap(),
            pages: parts["pages"].parse::<i32>().unwrap(),
        })
    }
}

#[derive(Deserialize, PartialEq, Default, Debug)]
pub struct SsCharMapPage {
    id: i32,
    file: String,
}
impl CharMapLoader for SsCharMapPage {
    type Output = SsCharMapPage;

    fn load(parts: &HashMap<String, String>) -> Result<Self::Output, SspjError> {
        let id = parts["id"].parse::<i32>().unwrap();
        let file = parts["file"].trim_matches('"');
        Ok(SsCharMapPage {
            id: id,
            file: file.to_string(),
        })
    }
}

#[derive(Deserialize, PartialEq, Default, Debug)]
pub struct SsCharMapChar {
    pub id: i32,
    pub src_rect: SsIRect,
    pub offset: SsPoint2,
    pub xadvance: i32,
    pub page: i32,
}
impl CharMapLoader for SsCharMapChar {
    type Output = SsCharMapChar;

    fn load(parts: &HashMap<String, String>) -> Result<Self::Output, SspjError> {
        Ok(SsCharMapChar {
            id: parts["id"].parse::<i32>().unwrap(),
            src_rect: SsIRect {
                x1: parts["x"].parse::<i32>().unwrap(),
                y1: parts["y"].parse::<i32>().unwrap(),
                x2: parts["width"].parse::<i32>().unwrap(),
                y2: parts["height"].parse::<i32>().unwrap(),
            },
            offset: SsPoint2 {
                x: parts["xoffset"].parse::<f32>().unwrap(),
                y: parts["yoffset"].parse::<f32>().unwrap()
            },
            xadvance: parts["xadvance"].parse::<i32>().unwrap(),
            page: parts["page"].parse::<i32>().unwrap(),
        })
    }
}

#[derive(Deserialize, PartialEq, Default, Debug)]
pub struct SsCharMapKerning {
    pub id: i32,
}
impl CharMapLoader for SsCharMapKerning {
    type Output = SsCharMapKerning;

    fn load(parts: &HashMap<String, String>) -> Result<Self::Output, SspjError> {
        Ok(SsCharMapKerning {
            id: parts["id"].parse::<i32>().unwrap(),
        })
    }
}

#[derive(Deserialize, PartialEq, Default, Debug)]
pub struct SsCharMap {
    pub info: SsCharMapInfo,
    pub common: SsCharMapCommon,
    pub pages: HashMap<i32, SsCharMapPage>,
    pub chars: HashMap<i32, SsCharMapChar>,
    pub kernings: HashMap<i32, SsCharMapKerning>,
}

impl SsLoader for SsCharMap {
    type Output = SsCharMap;

    fn load(s: &str) -> Result<Self::Output, SspjError> {
        let mut ss_char_map = SsCharMap::default();
        let mut chars_count: usize = 0;
        let mut kernings_count: usize = 0;
        for line in s.lines() {
            let splits = line.splitn(2, ' ').collect::<Vec<&str>>();
            if splits.len() != 2 {
                // invalid line
                continue;
            }
            let order = splits[0];
            let parts = split_ignoring_quotes(' ', splits[1]).iter().filter_map(|x| {
                let kv = split_ignoring_quotes('=', x.trim());
                if kv.len() != 2 { return None;}
                Some((kv[0].to_string(), kv[1].to_string()))
            }).collect::<HashMap<String, String>>();

            match order {
                "info" => {
                    ss_char_map.info = SsCharMapInfo::load(&parts)?;
                }
                "common" => {
                    ss_char_map.common = SsCharMapCommon::load(&parts)?;
                }
                "page" => {
                    let page = SsCharMapPage::load(&parts)?;
                    ss_char_map.pages.insert(page.id, page);
                }
                "chars" => {
                    chars_count = parts["count"].parse::<usize>().unwrap();
                }
                "char" => {
                    let char = SsCharMapChar::load(&parts)?;
                    ss_char_map.chars.insert(char.id, char);
                }
                "kernings" => {
                    kernings_count = parts["count"].parse::<usize>().unwrap();
                }
                "kerning" => {
                    let kerning = SsCharMapKerning::load(&parts)?;
                    ss_char_map.kernings.insert(kerning.id, kerning);
                }
                _ => {
                    // invalid line
                    continue;
                }
            }
        }
        // validation
        if chars_count != ss_char_map.chars.len() {
            return Err(SspjError::FntParseError(format!("Invalid number of chars count={} chars.len()={}", chars_count, ss_char_map.chars.len())));
        }
        if kernings_count != ss_char_map.kernings.len() {
            return Err(SspjError::FntParseError(format!("Invalid number of kernings count={} kernings.len()={}", kernings_count, ss_char_map.kernings.len())));
        }

        Ok(ss_char_map)
    }
}

fn split_ignoring_quotes(delimiter: char, s: &str) -> Vec<String> {
    let mut parts = vec![];
    let mut current_part = String::new();
    let mut in_quote = false;

    for c in s.chars() {
        match c {
            '"' => {
                in_quote = !in_quote;
                current_part.push(c); // Include the quote in the part
            }
            _ if c == delimiter && !in_quote => {
                // If a space is encountered outside of quotes, finalize the current part
                if !current_part.is_empty() {
                    parts.push(current_part.trim().to_string()); // Trim potential leading/trailing spaces
                    current_part.clear();
                }
            }
            _ => {
                current_part.push(c);
            }
        }
    }

    // Add the last part if it's not empty
    if !current_part.is_empty() {
        parts.push(current_part.trim().to_string());
    }

    parts
}

#[cfg(test)]
mod tests {
    use rayon::prelude::*;
    use std::path::{Path, PathBuf};
    use super::*;

    fn load_ssfnt(path : &Path) {
        let r = SsCharMap::load_from_file(path);
        match r {
            Ok(ss_char_map) => {
                println!("{:?}", ss_char_map);
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
    fn parse_ssfnt() {
        [
            "tests/overall/font/RoundedMPlus.fnt",
            "tests/allPartsV7/font/Sample_PixelMplus12.fnt",
        ].par_iter().for_each( |&f| {
            let p = get_test_base_path().join(f);
            load_ssfnt(&p.as_path());
        });
    }
}