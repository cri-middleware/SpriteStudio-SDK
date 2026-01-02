use serde::Deserialize;
use serde::de::{MapAccess, Visitor};
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(PartialEq, Default, Clone, Debug)]
pub struct SsPoint2 {
    pub x: f32,
    pub y: f32,
}
impl FromStr for SsPoint2 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<f32> = s
            .split_whitespace()
            .filter_map(|sub| sub.parse::<f32>().ok())
            .collect();

        if parts.len() == 2 {
            Ok(SsPoint2 {
                x: parts[0],
                y: parts[1],
            })
        } else {
            Err(format!(
                "Expected a string with two space-separated integers, but got '{}'",
                s
            ))
        }
    }
}
impl<'de> Deserialize<'de> for SsPoint2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(SsPoint2::default())
        } else {
            Self::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}

#[derive(PartialEq, Default, Clone, Debug)]
pub struct SsPoint3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl FromStr for SsPoint3 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<f32> = s
            .split_whitespace()
            .filter_map(|sub| sub.parse::<f32>().ok())
            .collect();

        if parts.len() == 3 {
            Ok(SsPoint3 {
                x: parts[0],
                y: parts[1],
                z: parts[2],
            })
        } else {
            Err(format!(
                "Expected a string with three space-separated integers, but got '{}'",
                s
            ))
        }
    }
}
impl<'de> Deserialize<'de> for SsPoint3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(Self::default())
        } else {
            Self::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}

#[derive(PartialEq, Default, Clone, Debug)]
pub struct SsU8Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl FromStr for SsU8Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(num) = u32::from_str_radix(s, 16) else {
            return Err(format!("Invalid hexadecimal color format: {}", s));
        };
        Ok(SsU8Color {
            a: (num >> 24) as u8,
            r: ((num >> 16) & 0xff) as u8,
            g: ((num >> 8) & 0xff) as u8,
            b: num as u8,
        })
    }
}
impl<'de> Deserialize<'de> for SsU8Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(Self::default())
        } else {
            Self::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}

#[derive(PartialEq, Default, Debug)]
pub struct SsIRect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}
impl FromStr for SsIRect {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<i32> = s
            .split_whitespace()
            .filter_map(|sub| sub.parse::<i32>().ok())
            .collect();

        if parts.len() == 4 {
            Ok(SsIRect {
                x1: parts[0],
                y1: parts[1],
                x2: parts[2],
                y2: parts[3],
            })
        } else {
            Err(format!(
                "Expected a string with four space-separated integers, but got '{}'",
                s
            ))
        }
    }
}
impl<'de> Deserialize<'de> for SsIRect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
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
pub enum SsPartsSortMode {
    Prio,
    Z,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SsPartType {
    Null,
    Normal,
    Shape,
    Text,
    Nines,
    Instance,
    Armature,
    Effect,
    Mesh,
    Movenode,
    Constraint,
    Mask,
    Joint,
    Bonepoint,
    TransformConstraint,
    Camera,
    #[serde(rename = "ssaudio")]
    Audio,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SsBoundsType {
    None,
    Quad,
    Aabb,
    Circle,
    CircleSmin,
    CircleSmax,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SsInheritType {
    Parent,
    #[serde(rename = "self")]
    Self_,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SsBlendType {
    Mix,
    Mul,
    Add,
    Sub,
    Mulalpha,
    Screen,
    Exclusion,
    Invert,
}

#[derive(Deserialize, PartialEq, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SsIkRotationArrow {
    #[default]
    Arrowfree,
    Clockwise,
    Anticlockwise,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum SsSequenceType {
    Last,
    Keep,
    Top,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SsTexWrapMode {
    Clamp,
    Repeat,
    Mirror,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SsTexFilterMode {
    #[serde(rename = "nearlest")]
    Nearest,
    Linear,
}

#[derive(Deserialize, PartialEq, Default, Clone, Debug)]
pub enum SsRenderBlendType {
    #[default]
    Mix,
    Add,
}

#[derive(Deserialize, PartialEq, Default, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SsShapeType {
    #[default]
    Unknown,
    Triangle,
    Rectangle,
    Arrow,
    Star,
}


#[derive(PartialEq, Default, Debug)]
pub struct SsCurve {
    pub start_time: f32,
    pub start_value: f32,
    pub end_time: f32,
    pub end_value: f32,

    pub start_key_time: f32,
    pub end_key_time: f32,

    pub sync_start_end: bool,
}
impl<'de> Deserialize<'de> for SsCurve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SsCurveVisitor;
        impl<'de> Visitor<'de> for SsCurveVisitor {
            type Value = SsCurve;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a object with a `type` field")
            }
            fn visit_map<A>(self, mut map: A) -> Result<SsCurve, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut sync_start_end = false;
                let mut nums: Vec<f32> = vec![];

                while let Some((key, value)) = map.next_entry::<String, String>()? {
                    match key.as_str() {
                        "@syncStartEnd" => {
                            sync_start_end = value == "1" || value.to_lowercase() == "true";
                        }
                        "$text" => {
                            nums = value
                                .split_whitespace()
                                .filter_map(|s| s.parse::<f32>().ok())
                                .collect();
                            if nums.len() != 4 {
                                return Err(serde::de::Error::custom(format!(
                                    "Expected 4 space-separated numbers, but got '{}'",
                                    value
                                )));
                            }
                        }
                        _ => {
                            // ignore
                        }
                    }
                }

                Ok(SsCurve {
                    start_time: nums[0],
                    start_value: nums[1],
                    end_time: nums[2],
                    end_value: nums[3],
                    start_key_time: 0.0,
                    end_key_time: 0.0,
                    sync_start_end,
                })
            }
        }
        deserializer.deserialize_map(SsCurveVisitor)
    }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsRefCell {
    mapid: i32,
    name: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SsColorBlendTarget {
    Whole,
    Vertex,
}

#[derive(Deserialize, PartialEq, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SsInterpolationType {
    #[default]
    None,
    Linear,
    Hermite,
    Bezier,
    Acceleration,
    Deceleration,

    EaseIn,
    EaseOut,
    EaseInOut,

    EaseExponentialIn,
    EaseExponentialOut,
    EaseExponentialInOut,

    EaseSineIn,
    EaseSineOut,
    EaseSineInOut,

    EaseElasticIn,
    EaseElasticOut,
    EaseElasticInOut,

    EaseBounceIn,
    EaseBounceOut,
    EaseBounceInOut,

    EaseBackIn,
    EaseBackOut,
    EaseBackInOut,

    Num,
}
