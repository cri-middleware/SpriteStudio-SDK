use crate::sstypes::{
    SsBlendType, SsColorBlendTarget, SsCurve, SsIRect, SsInterpolationType, SsPoint2, SsU8Color,
};
use serde::Deserialize;
use serde::de::{MapAccess, Visitor};
use std::fmt::Formatter;

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributeCellKeyValue {
    pub map_id: i32,
    pub name: String,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeCellKey {
    #[serde(rename = "@time")]
    pub time: i32,
    pub value: SsAttributeCellKeyValue,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeCell {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeCellKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeBoolKey {
    #[serde(rename = "@time")]
    pub time: i32,
    #[serde(rename = "value")]
    pub value: bool,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeIntKey {
    #[serde(rename = "@time")]
    pub time: i32,
    #[serde(rename = "@ipType", default)]
    pub ip_type: SsInterpolationType,
    #[serde(rename = "value")]
    pub value: i32,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeFloatKey {
    #[serde(rename = "@time")]
    pub time: i32,
    #[serde(rename = "@ipType", default)]
    pub ip_type: SsInterpolationType,
    #[serde(rename = "@easingRate", default)]
    pub easing_rate: f32,
    #[serde(default)]
    pub curve: SsCurve,
    #[serde(rename = "value")]
    pub value: f32,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributePosX {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributePosY {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributePosZ {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeRotX {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeRotY {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeRotZ {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeSclX {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeSclY {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeAlpha {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeLocalAlpha {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributePrio {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeIntKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeFlipH {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeBoolKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeFlipV {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeBoolKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeHide {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeBoolKey>,
}

#[derive(Deserialize, PartialEq, Default, Debug)]
pub struct SsAttributePartsColorKeyValueColor {
    pub rgba: SsU8Color,
    pub rate: f32,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributePartsColorKeyValue {
    pub target: SsColorBlendTarget,
    pub blend_type: SsBlendType,

    // ---
    #[serde(default)]
    pub color: SsAttributePartsColorKeyValueColor,

    // ---
    #[serde(rename = "LT", default)]
    pub lt: SsAttributePartsColorKeyValueColor,
    #[serde(rename = "RT", default)]
    pub rt: SsAttributePartsColorKeyValueColor,
    #[serde(rename = "LB", default)]
    pub lb: SsAttributePartsColorKeyValueColor,
    #[serde(rename = "RB", default)]
    pub rb: SsAttributePartsColorKeyValueColor,
    // ---
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributePartsColorKey {
    #[serde(rename = "@time")]
    pub time: i32,
    pub value: SsAttributePartsColorKeyValue,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributePartsColor {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributePartsColorKey>,
}

#[derive(Deserialize, PartialEq, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributeShaderKeyValueMap {
    pub map_id: i32,
    pub name: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributeShaderKeyValue {
    pub id: String,
    pub param0: f32,
    pub param1: f32,
    pub param2: f32,
    pub param3: f32,
    #[serde(default)]
    pub param4: f32,
    #[serde(default)]
    pub param5: f32,
    #[serde(default)]
    pub param6: f32,
    #[serde(default)]
    pub param7: f32,
    #[serde(default)]
    pub map0: SsAttributeShaderKeyValueMap,
    #[serde(default)]
    pub map1: SsAttributeShaderKeyValueMap,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributeShaderKey {
    #[serde(rename = "@time")]
    pub time: i32,
    #[serde(rename = "@ipType", default)]
    pub ip_type: SsInterpolationType,
    pub value: SsAttributeShaderKeyValue,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeShader {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeShaderKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributeVertexKeyValue {
    #[serde(rename = "LT")]
    pub lt: SsPoint2,
    #[serde(rename = "RT")]
    pub rt: SsPoint2,
    #[serde(rename = "LB")]
    pub lb: SsPoint2,
    #[serde(rename = "RB")]
    pub rb: SsPoint2,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeVertexKey {
    #[serde(rename = "@time")]
    pub time: i32,
    #[serde(rename = "@ipType", default)]
    pub ip_type: SsInterpolationType,
    pub value: SsAttributeVertexKeyValue,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeVertex {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeVertexKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributePivotX {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributePivotY {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeAnchorX {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeAnchorY {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeSizeX {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeSizeY {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeImgFlipH {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeBoolKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeImgFlipV {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeBoolKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeUvtX {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeUvtY {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeUvrZ {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeUvsX {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeUvsY {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeBoundr {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeMask {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeFloatKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeUserKeyValue {
    pub integer: Option<i32>,
    pub point: Option<SsPoint2>,
    pub rect: Option<SsIRect>,
    #[serde(rename = "string")]
    pub user_string: Option<String>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeUserKey {
    #[serde(rename = "@time")]
    pub time: i32,
    pub value: SsAttributeUserKeyValue,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeUser {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeUserKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SsAttributeSignalCommandParamType {
    Bool,
    Index,
    Integer,
    Floating,
    Text,
    None,
}

#[derive(Debug, PartialEq)]
pub enum SsAttributeSignalCommandParamValue {
    Bool(bool),
    Index(i32),
    Integer(i32),
    Floating(f32),
    Text(String),
    None,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(try_from = "RawSsAttributeSignalCommandParam")]
pub struct SsAttributeSignalCommandParam {
    pub param_id: String,
    pub param_type: SsAttributeSignalCommandParamType,
    pub value: SsAttributeSignalCommandParamValue,
}

#[derive(Deserialize)]
struct RawSsAttributeSignalCommandParam {
    #[serde(rename = "paramId")]
    param_id: String,
    #[serde(rename = "type")]
    param_type: SsAttributeSignalCommandParamType,
    value: Option<String>,
}
impl TryFrom<RawSsAttributeSignalCommandParam> for SsAttributeSignalCommandParam {
    type Error = String;

    fn try_from(raw: RawSsAttributeSignalCommandParam) -> Result<Self, Self::Error> {
        let val_str = raw.value.as_deref().unwrap_or("");

        let typed_value = match raw.param_type {
            SsAttributeSignalCommandParamType::Bool => {
                // "1", "true" などの揺らぎがある場合はここで吸収できます
                let b = val_str.parse::<bool>()
                    .or_else(|_| match val_str {
                        "1" => Ok(true),
                        "0" => Ok(false),
                        _ => Err(format!("Invalid bool value: {}", val_str))
                    })?;
                SsAttributeSignalCommandParamValue::Bool(b)
            },
            SsAttributeSignalCommandParamType::Index => {
                SsAttributeSignalCommandParamValue::Index(val_str.parse().map_err(|_| "Invalid index value")?)
            },
            SsAttributeSignalCommandParamType::Integer => {
                SsAttributeSignalCommandParamValue::Integer(val_str.parse().map_err(|_| "Invalid integer value")?)
            },
            SsAttributeSignalCommandParamType::Floating => {
                SsAttributeSignalCommandParamValue::Floating(val_str.parse().map_err(|_| "Invalid float value")?)
            },
            SsAttributeSignalCommandParamType::Text => {
                SsAttributeSignalCommandParamValue::Text(val_str.to_string())
            },
            SsAttributeSignalCommandParamType::None => {
                SsAttributeSignalCommandParamValue::None
            },
        };

        Ok(SsAttributeSignalCommandParam {
            param_id: raw.param_id,
            param_type: raw.param_type,
            value: typed_value,
        })
    }
}



#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributeSignalCommand {
    pub active: bool,
    pub command_id: String,
    #[serde(deserialize_with = "parse_signal_command_param_list")]
    pub params: Vec<SsAttributeSignalCommandParam>,
}
fn parse_signal_command_param_list<'de, D>(
    deserializer: D,
) -> Result<Vec<SsAttributeSignalCommandParam>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct SignalCommandParamList {
        value: Vec<SsAttributeSignalCommandParam>,
    }
    Ok(SignalCommandParamList::deserialize(deserializer)?.value)
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeSignalKeyValue {
    #[serde(deserialize_with = "parse_signal_command_list")]
    pub commands: Vec<SsAttributeSignalCommand>,
}
fn parse_signal_command_list<'de, D>(
    deserializer: D,
) -> Result<Vec<SsAttributeSignalCommand>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct SignalCommandList {
        value: Vec<SsAttributeSignalCommand>,
    }
    Ok(SignalCommandList::deserialize(deserializer)?.value)
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeSignalKey {
    #[serde(rename = "@time")]
    pub time: i32,
    pub value: SsAttributeSignalKeyValue,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeSignal {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeSignalKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributeInstanceKeyValue {
    #[serde(default)]
    pub infinity: bool,
    #[serde(default)]
    pub pingpong: bool,
    #[serde(default)]
    pub independent: bool,
    pub loop_num: i32,
    pub start_label: String,
    pub start_offset: i32,
    pub end_label: String,
    pub end_offset: i32,
    pub speed: f32,
}
#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributeInstanceKey {
    #[serde(rename = "@time")]
    pub time: i32,
    pub value: SsAttributeInstanceKeyValue,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeInstance {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeInstanceKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributeEffectKeyValue {
    pub start_time: i32,
    pub speed: f32,
    pub independent: bool,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeEffectKey {
    #[serde(rename = "@time")]
    pub time: i32,
    pub value: SsAttributeEffectKeyValue,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeEffect {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeEffectKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsDeformValue {
    pub x_index: f32,
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeDeformKeyValue {
    pub vsize: i32,
    #[serde(deserialize_with = "parse_ss_point3_list")]
    pub vchg: Vec<SsDeformValue>, // TODO: fix changing from String to SsMeshBindInfo
}
fn parse_ss_point3_list<'de, D>(deserializer: D) -> Result<Vec<SsDeformValue>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let items: Vec<f32> = s
        .split_whitespace()
        .filter_map(|sub| sub.parse::<f32>().ok())
        .collect();
    if items.is_empty() {
        return Err(serde::de::Error::custom("empty"));
    }
    let num = items[0] as i32;
    if num * 3 + 1 != items.len() as i32 {
        return Err(serde::de::Error::custom("invalid the number of items"));
    }
    Ok((0..num)
        .map(|idx| {
            let offset = 1 + (idx * 3) as usize;
            SsDeformValue {
                x_index: items[offset + 0],
                x: items[offset + 1],
                y: items[offset + 2],
            }
        })
        .collect())
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsAttributeDeformKey {
    #[serde(rename = "@time")]
    pub time: i32,
    #[serde(rename = "@ipType", default)]
    pub ip_type: SsInterpolationType,
    pub value: SsAttributeDeformKeyValue,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeDeform {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeDeformKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SsAttributeAudioKeyValue {
    #[serde(rename = "SoundListID")]
    pub sound_list_id: i32,
    pub sound_name: String,
    pub loop_num: i32,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeAudioKey {
    #[serde(rename = "@time")]
    pub time: i32,
    pub value: SsAttributeAudioKeyValue,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeAudio {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeAudioKey>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SsAttributeTexchangeKeyValue {
    texture_name: String,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeTexchangeKey {
    #[serde(rename = "@time")]
    pub time: i32,
    pub value: SsAttributeTexchangeKeyValue,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct SsAttributeTexchange {
    #[serde(rename = "key")]
    pub keys: Vec<SsAttributeTexchangeKey>,
}

#[derive(PartialEq, Debug)]
pub enum SsAttributeKind {
    Cell(SsAttributeCell),
    PosX(SsAttributePosX),
    PosY(SsAttributePosY),
    PosZ(SsAttributePosZ),
    RotX(SsAttributeRotX),
    RotY(SsAttributeRotY),
    RotZ(SsAttributeRotZ),
    SclX(SsAttributeSclX),
    SclY(SsAttributeSclY),
    LosclX(SsAttributeSclX),
    LosclY(SsAttributeSclY),
    Alpha(SsAttributeAlpha),
    LoAlpha(SsAttributeLocalAlpha),
    Prio(SsAttributePrio),
    FlipH(SsAttributeFlipH),
    FlipV(SsAttributeFlipV),
    Hide(SsAttributeHide),
    PartsColor(SsAttributePartsColor),
    Color,
    Shader(SsAttributeShader),
    Vertex(SsAttributeVertex),
    PivotX(SsAttributePivotX),
    PivotY(SsAttributePivotY),
    AnchorX(SsAttributeAnchorX),
    AnchorY(SsAttributeAnchorY),
    SizeX(SsAttributeSizeX),
    SizeY(SsAttributeSizeY),
    ImgFlipH(SsAttributeImgFlipH),
    ImgFlipV(SsAttributeImgFlipV),
    UvtX(SsAttributeUvtX),
    UvtY(SsAttributeUvtY),
    UvrZ(SsAttributeUvrZ),
    UvsX(SsAttributeUvsX),
    UvsY(SsAttributeUvsY),
    Boundr(SsAttributeBoundr),
    Mask(SsAttributeMask),
    User(SsAttributeUser),
    Signal(SsAttributeSignal),
    Instance(SsAttributeInstance),
    Effect(SsAttributeEffect),
    Deform(SsAttributeDeform),
    Audio(SsAttributeAudio),
    Texchange(SsAttributeTexchange),
}
impl<'de> Deserialize<'de> for SsAttributeKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SsAttributeKindVisitor;
        impl<'de> Visitor<'de> for SsAttributeKindVisitor {
            type Value = SsAttributeKind;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a object with a `type` field")
            }
            fn visit_map<A>(self, mut map: A) -> Result<SsAttributeKind, A::Error>
            where
                A: MapAccess<'de>,
            {
                if let Some((key, value)) = map.next_entry::<String, String>()? {
                    return match key.as_str() {
                        "@tag" => match value.as_str() {
                            "CELL" => {
                                let cell = SsAttributeCell::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Cell(cell))
                            }
                            "POSX" => {
                                let posx = SsAttributePosX::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::PosX(posx))
                            }
                            "POSY" => {
                                let posy = SsAttributePosY::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::PosY(posy))
                            }
                            "POSZ" => {
                                let posz = SsAttributePosZ::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::PosZ(posz))
                            }
                            "ROTX" => {
                                let rotx = SsAttributeRotX::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::RotX(rotx))
                            }
                            "ROTY" => {
                                let roty = SsAttributeRotY::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::RotY(roty))
                            }
                            "ROTZ" => {
                                let rotz = SsAttributeRotZ::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::RotZ(rotz))
                            }
                            "SCLX" => {
                                let sclx = SsAttributeSclX::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::SclX(sclx))
                            }
                            "SCLY" => {
                                let scly = SsAttributeSclY::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::SclY(scly))
                            }
                            "LSCX" => {
                                let lscx = SsAttributeSclX::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::LosclX(lscx))
                            }
                            "LSCY" => {
                                let lscy = SsAttributeSclY::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::LosclY(lscy))
                            }
                            "ALPH" => {
                                let alpha = SsAttributeAlpha::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Alpha(alpha))
                            }
                            "LALP" => {
                                let loalpha = SsAttributeLocalAlpha::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::LoAlpha(loalpha))
                            }
                            "PRIO" => {
                                let prio = SsAttributePrio::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Prio(prio))
                            }
                            "FLPH" => {
                                let flip_h = SsAttributeFlipH::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::FlipH(flip_h))
                            }
                            "FLPV" => {
                                let flip_v = SsAttributeFlipV::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::FlipV(flip_v))
                            }
                            "HIDE" => {
                                let hide = SsAttributeHide::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Hide(hide))
                            }
                            "PCOL" => {
                                let parts_color = SsAttributePartsColor::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::PartsColor(parts_color))
                            }
                            "VCOL" => {
                                // warn!("VCOL is deprecated");
                                Ok(SsAttributeKind::Color)
                            }
                            "SHDR" => {
                                let shader = SsAttributeShader::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Shader(shader))
                            }
                            "VERT" => {
                                let vertex = SsAttributeVertex::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Vertex(vertex))
                            }
                            "PVTX" => {
                                let pivot_x = SsAttributePivotX::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::PivotX(pivot_x))
                            }
                            "PVTY" => {
                                let pivot_y = SsAttributePivotY::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::PivotY(pivot_y))
                            }
                            "ANCX" => {
                                let anchor_x = SsAttributeAnchorX::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::AnchorX(anchor_x))
                            }
                            "ANCY" => {
                                let anchor_y = SsAttributeAnchorY::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::AnchorY(anchor_y))
                            }
                            "SIZX" => {
                                let size_x = SsAttributeSizeX::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::SizeX(size_x))
                            }
                            "SIZY" => {
                                let size_y = SsAttributeSizeY::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::SizeY(size_y))
                            }
                            "IFLH" => {
                                let img_flip_h = SsAttributeImgFlipH::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::ImgFlipH(img_flip_h))
                            }
                            "IFLV" => {
                                let img_flip_v = SsAttributeImgFlipV::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::ImgFlipV(img_flip_v))
                            }
                            "UVTX" => {
                                let uvtx = SsAttributeUvtX::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::UvtX(uvtx))
                            }
                            "UVTY" => {
                                let uvty = SsAttributeUvtY::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::UvtY(uvty))
                            }
                            "UVRZ" => {
                                let uvrz = SsAttributeUvrZ::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::UvrZ(uvrz))
                            }
                            "UVSX" => {
                                let uvsx = SsAttributeUvsX::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::UvsX(uvsx))
                            }
                            "UVSY" => {
                                let uvsy = SsAttributeUvsY::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::UvsY(uvsy))
                            }
                            "BNDR" => {
                                let boundr = SsAttributeBoundr::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Boundr(boundr))
                            }
                            "MASK" => {
                                let mask = SsAttributeMask::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Mask(mask))
                            }
                            "USER" => {
                                let user = SsAttributeUser::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::User(user))
                            }
                            "SIGN" => {
                                let signal = SsAttributeSignal::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Signal(signal))
                            }
                            "IPRM" => {
                                let instance = SsAttributeInstance::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Instance(instance))
                            }
                            "EFCT" => {
                                let effect = SsAttributeEffect::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Effect(effect))
                            }
                            "DEFM" => {
                                let deform = SsAttributeDeform::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Deform(deform))
                            }
                            "ADIO" => {
                                let audio = SsAttributeAudio::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Audio(audio))
                            }
                            "TCHG" => {
                                let texchange = SsAttributeTexchange::deserialize(
                                    serde::de::value::MapAccessDeserializer::new(map),
                                )?;
                                Ok(SsAttributeKind::Texchange(texchange))
                            }
                            _ => {
                                return Err(serde::de::Error::custom(format!(
                                    "unknown attribute type: {}",
                                    value
                                )));
                            }
                        },
                        _ => {
                            return Err(serde::de::Error::custom(format!(
                                "expected @name attribute, found: {}",
                                key
                            )));
                        }
                    };
                }
                Err(serde::de::Error::custom(format!(
                    "expected `type` attribute"
                )))
            }
        }
        deserializer
            .deserialize_map(SsAttributeKindVisitor)
            .map_err(serde::de::Error::custom)
    }
}
