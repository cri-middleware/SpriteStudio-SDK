use std::fmt::Formatter;
use serde::{Deserialize};
use serde::de::{MapAccess, Visitor};
use serde::de::value::MapAccessDeserializer;
use crate::sspj::ssloader::{SsLoader, SspjError};
use crate::sspj::sstypes::*;

#[derive(Deserialize, PartialEq, Debug)]
pub struct VarianceValue<T> {
    #[serde(rename = "@value")]
    pub value: T,
    #[serde(rename = "@subvalue")]
    pub subvalue: T,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParticleElementBasic {
    #[serde(rename = "name")]
    pub name: String,
    pub priority: i32,
    pub maximum_particle: i32,
    pub attime_create: i32,
    pub interval: i32,
    pub speed: VarianceValue<f32>,
    pub lifespan: VarianceValue<i32>,
    pub angle: f32,
    pub angle_variance: f32,

    pub lifetime: i32,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementRndSeedChange {
    #[serde(rename = "name")]
    pub name: String,
    pub seed: i32,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementDelay {
    #[serde(rename = "name")]
    pub name: String,
    pub delay_time: i32,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementGravity {
    #[serde(rename = "name")]
    pub name: String,
    pub gravity: SsPoint2,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementPosition {
    #[serde(rename = "name")]
    pub name: String,
    pub offset_x: VarianceValue<f32>,
    pub offset_y: VarianceValue<f32>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementRotation {
    #[serde(rename = "name")]
    pub name: String,
    pub rotation: VarianceValue<f32>,
    pub rotation_add: VarianceValue<f32>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementRotationTrans {
    #[serde(rename = "name")]
    pub name: String,
    pub rotation_factor: f32,
    pub end_life_time_per: f32,
}
impl Default for ParticleElementRotationTrans {
    fn default() -> Self {
        Self {
            name: String::from(""),
            rotation_factor: 0.0,
            end_life_time_per: 75.0,
        }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementTransSpeed {
    #[serde(rename = "name")]
    pub name: String,
    pub speed: VarianceValue<f32>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementTangentialAcceleration {
    #[serde(rename = "name")]
    pub name: String,
    pub acceleration: VarianceValue<f32>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementInitColor {
    #[serde(rename = "name")]
    pub name: String,
    pub color: VarianceValue<SsU8Color>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementTransColor {
    #[serde(rename = "name")]
    pub name: String,
    pub color: VarianceValue<SsU8Color>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct ParticleElementAlphaFade {
    #[serde(rename = "name")]
    pub name: String,
    pub disprange: VarianceValue<f32>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementSize {
    #[serde(rename = "name")]
    pub name: String,
    pub size_x: VarianceValue<f32>,
    pub size_y: VarianceValue<f32>,
    pub scale_factor: VarianceValue<f32>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleElementTransSize {
    #[serde(rename = "name")]
    pub name: String,
    pub size_x: VarianceValue<f32>,
    pub size_y: VarianceValue<f32>,
    pub scale_factor: VarianceValue<f32>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticlePointGravity {
    #[serde(rename = "name")]
    pub name: String,
    pub position: SsPoint2,
    pub power: f32,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ParticleTurnToDirectionEnabled {
    #[serde(rename = "name")]
    pub name: String,
    pub rotation: f32,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct ParticleInfiniteEmitEnabled {
    #[serde(rename = "name")]
    pub name: String,
}

#[derive(PartialEq, Debug)]
pub enum SsEffectFunctionType {
    Base,
    Basic(ParticleElementBasic),
    RndSeedChange(ParticleElementRndSeedChange),
    Delay(ParticleElementDelay),
    Gravity(ParticleElementGravity),
    Position(ParticleElementPosition),
    //TransPosition,
    Rotation(ParticleElementRotation),
    TransRotation(ParticleElementRotationTrans),
    TransSpeed(ParticleElementTransSpeed),
    TangentialAcceleration(ParticleElementTangentialAcceleration),
    InitColor(ParticleElementInitColor),
    TransColor(ParticleElementTransColor),
    AlphaFade(ParticleElementAlphaFade),
    Size(ParticleElementSize),
    TransSize(ParticleElementTransSize),
    PointGravity(ParticlePointGravity),
    TurnToDirectionEnabled(ParticleTurnToDirectionEnabled),
    InfiniteEmitEnabled(ParticleInfiniteEmitEnabled),
}
impl<'de> Deserialize<'de> for SsEffectFunctionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        struct SsEffectFunctionTypeVisitor;

        impl<'de> Visitor<'de> for SsEffectFunctionTypeVisitor {
            type Value = SsEffectFunctionType;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a object with a `type` field")
            }

            fn visit_map<A>(self, mut map: A) -> Result<SsEffectFunctionType, A::Error>
            where
                A: MapAccess<'de>
            {
                if let Some((key, value)) = map.next_entry::<String, String>()? {
                    return match key.as_str() {
                        "@name" => match value.as_str() {
                            // "Base",
                            "Basic" => {
                                let f = ParticleElementBasic::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::Basic(f))
                            }
                            "OverWriteSeed" => {
                                let f = ParticleElementRndSeedChange::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::RndSeedChange(f))
                            }
                            "Delay" => {
                                let f = ParticleElementDelay::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::Delay(f))
                            }
                            "Gravity" => {
                                let f = ParticleElementGravity::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::Gravity(f))
                            }
                            "init_position" => {
                                let f = ParticleElementPosition::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::Position(f))
                            }
                            "init_rotation" => {
                                let f = ParticleElementRotation::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::Rotation(f))
                            }
                            "trans_rotation" => {
                                let f = ParticleElementRotationTrans::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::TransRotation(f))
                            }
                            "trans_speed" => {
                                let f = ParticleElementTransSpeed::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::TransSpeed(f))
                            }
                            "add_tangentiala" => {
                                let f = ParticleElementTangentialAcceleration::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::TangentialAcceleration(f))
                            }
                            "init_vertexcolor" => {
                                let f = ParticleElementInitColor::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::InitColor(f))
                            }
                            "trans_vertexcolor" => {
                                let f = ParticleElementTransColor::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::TransColor(f))
                            }
                            "trans_colorfade" => {
                                let f = ParticleElementAlphaFade::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::AlphaFade(f))
                            }
                            "init_size" => {
                                let f = ParticleElementSize::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::Size(f))
                            }
                            "trans_size" => {
                                let f = ParticleElementTransSize::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::TransSize(f))
                            }
                            "add_pointgravity" => {
                                let f = ParticlePointGravity::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::PointGravity(f))
                            }
                            "TurnToDirection" => {
                                let f = ParticleTurnToDirectionEnabled::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::TurnToDirectionEnabled(f))
                            }
                            "InfiniteEmit" => {
                                let f = ParticleInfiniteEmitEnabled::deserialize(MapAccessDeserializer::new(map))?;
                                Ok(SsEffectFunctionType::InfiniteEmitEnabled(f))
                            }
                            _ => {
                                return Err(serde::de::Error::custom(format!("unknown type: {}", value)))
                            }
                        },
                        _ => {
                            return Err(serde::de::Error::custom(format!("expected @name attribute, found: {}", key)))
                        }
                    }
                }
                Err(serde::de::Error::custom(format!("expected `type` attribute")))
            }
        }
        deserializer.deserialize_map(SsEffectFunctionTypeVisitor)
    }
}

#[derive(Deserialize, PartialEq, Debug)]
pub enum SsEffectNodeType {
    Root,
    #[serde(rename = "Emmiter")]
    Emitter,
    Particle,
}

#[derive(Deserialize, PartialEq, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SsEffectBehavior {
    pub cell_name: String,
    pub cell_map_name: String,
    pub blend_type: SsRenderBlendType,
    #[serde(rename = "list", deserialize_with = "parse_behavior_list")]
    pub list: Vec<SsEffectFunctionType>,
}

fn parse_behavior_list<'de, D>(deserializer: D) -> Result<Vec<SsEffectFunctionType>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct BehaviorList {
        #[serde(default)]
        value: Vec<SsEffectFunctionType>,
    }
    Ok(BehaviorList::deserialize(deserializer)?.value)
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsEffectNode {
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: SsEffectNodeType,
    pub array_index: i32,
    pub parent_index: i32,
    pub visible: bool,
    #[serde(default)]
    pub behavior: SsEffectBehavior,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsEffectModel {
    #[serde(deserialize_with = "parse_node_list")]
    pub node_list: Vec<SsEffectNode>,
    pub lock_rand_seed: i32,
    pub is_lock_rand_seed: bool,
    pub fps: i32,
    pub bgcolor: String,
    #[serde(default)]
    pub layout_scale_x: i32,
    #[serde(default)]
    pub layout_scale_y: i32,

    #[serde(default)]
    pub effect_name: String, // TODO: impl
}
fn parse_node_list<'de, D>(deserializer: D) -> Result<Vec<SsEffectNode>, D::Error>
where
    D: serde::Deserializer<'de>
{
    #[derive(Deserialize, PartialEq)]
    struct NodeList {
        node: Vec<SsEffectNode>,
    }
    Ok(NodeList::deserialize(deserializer)?.node)
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsEffectFile {
    #[serde(rename = "@version")]
    pub version: String,
    pub effect_data: SsEffectModel,
    pub name: String,
}
impl SsLoader for SsEffectFile {
    type Output = SsEffectFile;

    fn load(s: &str) -> Result<Self::Output, SspjError> {
        let r = quick_xml::de::from_str::<Self::Output>(s);
        match r {
            Ok(r) => Ok(r),
            Err(e) => Err(SspjError::XmlParseError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use rayon::prelude::*;
    use super::*;


    fn load_ssee(path : &Path) {
        let r = SsEffectFile::load_from_file(path);
        match r {
            Ok(ss_effect_file) => {
                println!("{:?}", ss_effect_file);
                assert!(true);
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
    fn parse_ssee() {
        [
            "tests/ParticleEffect/001a.ssee",
            "tests/ParticleEffect/001b.ssee",
            "tests/ParticleEffect/001c.ssee",
            "tests/ParticleEffect/001d.ssee",
            "tests/ParticleEffect/002a.ssee",
            "tests/ParticleEffect/002b.ssee",
            "tests/ParticleEffect/003a.ssee",
            "tests/ParticleEffect/004a.ssee",
            "tests/ParticleEffect/004b.ssee",
            "tests/ParticleEffect/005a.ssee",
            "tests/ParticleEffect/005b.ssee",
            "tests/ParticleEffect/005c.ssee",
            "tests/ParticleEffect/005d.ssee",
            "tests/ParticleEffect/005e.ssee",
            "tests/ParticleEffect/005f.ssee",
            "tests/ParticleEffect/006a.ssee",
            "tests/ParticleEffect/006b.ssee",
            "tests/ParticleEffect/007a.ssee",
            "tests/ParticleEffect/007b.ssee",
            "tests/ParticleEffect/008a.ssee",
            "tests/ParticleEffect/009a.ssee",
            "tests/ParticleEffect/009b.ssee",
            "tests/ParticleEffect/010a.ssee",
            "tests/ParticleEffect/010b.ssee",
            "tests/ParticleEffect/011a.ssee",
            "tests/ParticleEffect/011b.ssee",
            "tests/ParticleEffect/012a.ssee",
            "tests/ParticleEffect/012b.ssee",
            "tests/ParticleEffect/013a.ssee",
            "tests/ParticleEffect/014a.ssee",
            "tests/ParticleEffect/015a.ssee",
            "tests/ParticleEffect/016a.ssee",
            "tests/ParticleEffect/016b.ssee",
            "tests/ParticleEffect/017a.ssee",
            "tests/ParticleEffect/017b.ssee",
            "tests/ParticleEffect/018a.ssee",
            "tests/ParticleEffect/019a.ssee",
            "tests/ParticleEffect/020a.ssee",
            "tests/ParticleEffect/020b.ssee",
            "tests/ParticleEffect/021a.ssee",
            "tests/ParticleEffect/021b.ssee",
            "tests/ParticleEffect/022a.ssee",
            "tests/ParticleEffect/023a.ssee",
            "tests/ParticleEffect/024a.ssee",
            "tests/ParticleEffect/024b.ssee",
            "tests/ParticleEffect/024c.ssee",
            "tests/ParticleEffect/025a.ssee",
            "tests/ParticleEffect/026a.ssee",
            "tests/ParticleEffect/027a.ssee",
            "tests/ParticleEffect/028a.ssee",
            "tests/ParticleEffect/028b.ssee",
            "tests/ParticleEffect/028c.ssee",
            "tests/ParticleEffect/029a.ssee",
            "tests/ParticleEffect/029b.ssee",
            "tests/ParticleEffect/029c.ssee",
            "tests/ParticleEffect/030a.ssee",
            "tests/ParticleEffect/030b.ssee",
            "tests/ParticleEffect/030c.ssee",
            "tests/ParticleEffect/031a.ssee",
            "tests/ParticleEffect/032a.ssee",
            "tests/ParticleEffect/033a.ssee",
            "tests/ParticleEffect/034a.ssee",
            "tests/ParticleEffect/035a.ssee",
            "tests/ParticleEffect/036a.ssee",
            "tests/ParticleEffect/037a.ssee",
            "tests/ParticleEffect/038a.ssee",
            "tests/ParticleEffect/039a.ssee",
            "tests/ParticleEffect/040a.ssee",
            "tests/allAttributeV7/NewEffect.ssee",
            "tests/allPartsV7/NewEffect.ssee",
            "tests/overall/effect.ssee",
            "tests/overall/effect_checker.ssee",
        ].par_iter().for_each( |&f| {
            let p = get_test_base_path().join(f);
            load_ssee(&p.as_path());
        });
    }
}
