use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec2Key {
    pub v1: ordered_float::OrderedFloat<f32>,
    pub v2: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::Vec2T> for Vec2Key {
    fn from(v: &ssab::ssab::ss::format::Vec2T) -> Self {
        Self { v1: ordered_float::OrderedFloat(v.v1), v2: ordered_float::OrderedFloat(v.v2) }
    }
}
impl Vec2Key {}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct BoundsKey {
    pub left: ordered_float::OrderedFloat<f32>,
    pub top: ordered_float::OrderedFloat<f32>,
    pub right: ordered_float::OrderedFloat<f32>,
    pub bottom: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::BoundsT> for BoundsKey {
    fn from(value: &ssab::ssab::ss::format::BoundsT) -> Self {
        Self { left: ordered_float::OrderedFloat(value.left), top: ordered_float::OrderedFloat(value.top), right: ordered_float::OrderedFloat(value.right), bottom: ordered_float::OrderedFloat(value.bottom) }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RegionKey {
    pub x: ordered_float::OrderedFloat<f32>,
    pub y: ordered_float::OrderedFloat<f32>,
    pub w: ordered_float::OrderedFloat<f32>,
    pub h: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::RegionT> for RegionKey {
    fn from(value: &ssab::ssab::ss::format::RegionT) -> Self {
        Self { x: ordered_float::OrderedFloat(value.x), y: ordered_float::OrderedFloat(value.y), w: ordered_float::OrderedFloat(value.w), h: ordered_float::OrderedFloat(value.h) }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct MinMax2Key {
    pub x_min: ordered_float::OrderedFloat<f32>,
    pub y_min: ordered_float::OrderedFloat<f32>,
    pub x_max: ordered_float::OrderedFloat<f32>,
    pub y_max: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::MinMax2T> for MinMax2Key {
    fn from(value: &ssab::ssab::ss::format::MinMax2T) -> Self {
        Self { x_min: ordered_float::OrderedFloat(value.x_min), y_min: ordered_float::OrderedFloat(value.y_min), x_max: ordered_float::OrderedFloat(value.x_max), y_max: ordered_float::OrderedFloat(value.y_max) }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct U8ColorKey {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl From<&ssab::ssab::ss::format::U8ColorT> for U8ColorKey {
    fn from(value: &ssab::ssab::ss::format::U8ColorT) -> Self {
        Self { a: value.a, r: value.r, g: value.g, b: value.b }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct NoneValueEntryKey {}
impl From<&ssab::ssab::ss::format::NoneValueEntryT> for NoneValueEntryKey {
    fn from(_none_value_table_t: &ssab::ssab::ss::format::NoneValueEntryT) -> Self {
        Self {}
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct BoolValueEntryKey {
    pub value: bool,
}
impl From<&ssab::ssab::ss::format::BoolValueEntryT> for BoolValueEntryKey {
    fn from(bool_value_table_t: &ssab::ssab::ss::format::BoolValueEntryT) -> Self {
        Self { value: bool_value_table_t.value }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct IntValueEntryKey {
    pub value: i32,
}
impl From<&ssab::ssab::ss::format::IntValueEntryT> for IntValueEntryKey {
    fn from(int_value_table_t: &ssab::ssab::ss::format::IntValueEntryT) -> Self {
        Self { value: int_value_table_t.value }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct FloatValueEntryKey {
    pub value: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::FloatValueEntryT> for FloatValueEntryKey {
    fn from(float_value_table_t: &ssab::ssab::ss::format::FloatValueEntryT) -> Self {
        Self { value: ordered_float::OrderedFloat(float_value_table_t.value) }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct StringValueEntryKey {
    pub value: Option<String>,
}
impl From<&ssab::ssab::ss::format::StringValueEntryT> for StringValueEntryKey {
    fn from(string_value_table_t: &ssab::ssab::ss::format::StringValueEntryT) -> Self {
        Self { value: Some(string_value_table_t.value.clone()) }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec2ValueEntryKey {
    pub value: Option<Vec2Key>,
}
impl From<&ssab::ssab::ss::format::Vec2ValueEntryT> for Vec2ValueEntryKey {
    fn from(vec2_value_table_t: &ssab::ssab::ss::format::Vec2ValueEntryT) -> Self {
        Self { value: Some(Vec2Key::from(vec2_value_table_t.value.as_ref())) }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct MinMax2ValueEntryKey {
    pub value: Option<MinMax2Key>,
}
impl From<&ssab::ssab::ss::format::MinMax2ValueEntryT> for MinMax2ValueEntryKey {
    fn from(min_max2_value_table_t: &ssab::ssab::ss::format::MinMax2ValueEntryT) -> Self {
        Self { value: Some(MinMax2Key::from(min_max2_value_table_t.value.as_ref())) }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct InitialEventsKey {
    pub user: Option<PartAttributeUserDataKey>,
    pub signal: Option<PartAttributeSignalKey>,
    pub audio: Option<PartAttributeAudioKey>,
    pub instance: Option<PartAttributeInstanceKey>,
    pub effect: Option<PartAttributeEffectKey>,
}
impl From<&ssab::ssab::ss::format::InitialEventsT> for InitialEventsKey {
    fn from(value: &ssab::ssab::ss::format::InitialEventsT) -> Self {
        Self {
            user: value.user.as_ref().map(|u| PartAttributeUserDataKey::from(u.as_ref())),
            signal: value.signal.as_ref().map(|s| PartAttributeSignalKey::from(s.as_ref())),
            audio: value.audio.as_ref().map(|a| PartAttributeAudioKey::from(a.as_ref())),
            instance: value.instance.as_ref().map(|i| PartAttributeInstanceKey::from(i.as_ref())),
            effect: value.effect.as_ref().map(|e| PartAttributeEffectKey::from(e.as_ref())),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct InitialEventsVecKey {
    pub v: Vec<InitialEventsKey>,
}
impl From<&[ssab::ssab::ss::format::InitialEventsT]> for InitialEventsVecKey {
    fn from(value: &[ssab::ssab::ss::format::InitialEventsT]) -> Self {
        Self { v: value.iter().map(InitialEventsKey::from).collect() }
    }
}

/// A part's run per attribute, in schema order.
#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataKey {
    pub runs: [u32; 38],
}
impl From<&ssab::ssab::ss::format::PartAnimationDataT> for PartAnimationDataKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataT) -> Self {
        Self {
            runs: [
                value.cell_keys,
                value.position_x_keys,
                value.position_y_keys,
                value.position_z_keys,
                value.rotation_x_keys,
                value.rotation_y_keys,
                value.rotation_z_keys,
                value.scale_x_keys,
                value.scale_y_keys,
                value.local_scale_x_keys,
                value.local_scale_y_keys,
                value.alpha_keys,
                value.local_alpha_keys,
                value.priority_keys,
                value.flip_h_keys,
                value.flip_v_keys,
                value.hide_keys,
                value.part_color_keys,
                value.shader_keys,
                value.vertex_keys,
                value.pivot_x_keys,
                value.pivot_y_keys,
                value.anchor_x_keys,
                value.anchor_y_keys,
                value.size_x_keys,
                value.size_y_keys,
                value.img_flip_h_keys,
                value.img_flip_v_keys,
                value.uv_translation_x_keys,
                value.uv_translation_y_keys,
                value.uv_rotation_z_keys,
                value.uv_scale_x_keys,
                value.uv_scale_y_keys,
                value.bounding_radius_keys,
                value.mask_keys,
                value.skew_keys,
                value.deform_keys,
                value.texture_keys,
            ],
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CellKey {
    pub name_hash: u32,
    pub name: Option<String>,
    pub region: Option<RegionKey>,
    pub pivot: Option<Vec2Key>,
    pub rotated: bool,
    pub table_coord_x: Option<Vec<ordered_float::OrderedFloat<f32>>>,
    pub table_coord_y: Option<Vec<ordered_float::OrderedFloat<f32>>>,
    pub table_index_vertex: Option<Vec<u16>>,
}
impl From<&ssab::ssab::ss::format::CellT> for CellKey {
    fn from(value: &ssab::ssab::ss::format::CellT) -> Self {
        Self {
            name_hash: value.name_hash,
            name: Some(value.name.to_string()),
            region: Some(RegionKey::from(value.region.as_ref())),
            pivot: Some(Vec2Key::from(value.pivot.as_ref())),
            rotated: value.rotated,
            table_coord_x: value.table_coord_x.as_ref().map(|t| t.iter().map(|f| ordered_float::OrderedFloat(*f)).collect()),
            table_coord_y: value.table_coord_y.as_ref().map(|t| t.iter().map(|f| ordered_float::OrderedFloat(*f)).collect()),
            table_index_vertex: value.table_index_vertex.as_ref().map(|t| t.to_vec()),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct LabelKey {
    pub name_hash: u32,
    pub name: Option<String>,
    pub time: i32,
}
impl From<&ssab::ssab::ss::format::LabelT> for LabelKey {
    fn from(value: &ssab::ssab::ss::format::LabelT) -> Self {
        Self { name_hash: value.name_hash, name: Some(value.name.to_string()), time: value.time }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeCellKey {
    pub cellmap_name_hash: u32,
    pub name_hash: u32,
}
impl From<&ssab::ssab::ss::format::PartAttributeCellT> for PartAttributeCellKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributeCellT) -> Self {
        Self { cellmap_name_hash: value.cellmap_name_hash, name_hash: value.name_hash }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct SsAttributePartColorKeyValueColorKey {
    pub rgba: Option<U8ColorKey>,
    pub rate: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT> for SsAttributePartColorKeyValueColorKey {
    fn from(value: &ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT) -> Self {
        Self { rgba: value.rgba.as_ref().map(|r| U8ColorKey::from(r.as_ref())), rate: ordered_float::OrderedFloat(value.rate) }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributePartColorKey {
    pub blend_type: ssab::ssab::ss::format::BlendType,
    pub lt: Option<SsAttributePartColorKeyValueColorKey>,
    pub rt: Option<SsAttributePartColorKeyValueColorKey>,
    pub lb: Option<SsAttributePartColorKeyValueColorKey>,
    pub rb: Option<SsAttributePartColorKeyValueColorKey>,
}
impl From<&ssab::ssab::ss::format::PartAttributePartColorT> for PartAttributePartColorKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributePartColorT) -> Self {
        Self {
            blend_type: value.blend_type,
            lt: value.lt.as_ref().map(|l| SsAttributePartColorKeyValueColorKey::from(l.as_ref())),
            rt: value.rt.as_ref().map(|r| SsAttributePartColorKeyValueColorKey::from(r.as_ref())),
            lb: value.lb.as_ref().map(|l| SsAttributePartColorKeyValueColorKey::from(l.as_ref())),
            rb: value.rb.as_ref().map(|r| SsAttributePartColorKeyValueColorKey::from(r.as_ref())),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeShaderKey {
    pub id: Option<String>,
    pub id_hash: u32,
    pub param0: ordered_float::OrderedFloat<f32>,
    pub param1: ordered_float::OrderedFloat<f32>,
    pub param2: ordered_float::OrderedFloat<f32>,
    pub param3: ordered_float::OrderedFloat<f32>,
    pub param4: ordered_float::OrderedFloat<f32>,
    pub param5: ordered_float::OrderedFloat<f32>,
    pub param6: ordered_float::OrderedFloat<f32>,
    pub param7: ordered_float::OrderedFloat<f32>,
    pub map0_cellmap_name_hash: u32,
    pub map0_name_hash: u32,
    pub map1_cellmap_name_hash: u32,
    pub map1_name_hash: u32,
}
impl From<&ssab::ssab::ss::format::PartAttributeShaderT> for PartAttributeShaderKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributeShaderT) -> Self {
        Self {
            id: Some(value.id.clone()),
            id_hash: value.id_hash,
            param0: ordered_float::OrderedFloat(value.param0),
            param1: ordered_float::OrderedFloat(value.param1),
            param2: ordered_float::OrderedFloat(value.param2),
            param3: ordered_float::OrderedFloat(value.param3),
            param4: ordered_float::OrderedFloat(value.param4),
            param5: ordered_float::OrderedFloat(value.param5),
            param6: ordered_float::OrderedFloat(value.param6),
            param7: ordered_float::OrderedFloat(value.param7),
            map0_cellmap_name_hash: value.map0_cellmap_name_hash,
            map0_name_hash: value.map0_name_hash,
            map1_cellmap_name_hash: value.map1_cellmap_name_hash,
            map1_name_hash: value.map1_name_hash,
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeVertexKey {
    pub lt: Option<Vec2Key>,
    pub rt: Option<Vec2Key>,
    pub lb: Option<Vec2Key>,
    pub rb: Option<Vec2Key>,
}
impl From<&ssab::ssab::ss::format::PartAttributeVertexT> for PartAttributeVertexKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributeVertexT) -> Self {
        Self { lt: value.lt.as_ref().map(|v| Vec2Key::from(v.as_ref())), rt: value.rt.as_ref().map(|v| Vec2Key::from(v.as_ref())), lb: value.lb.as_ref().map(|v| Vec2Key::from(v.as_ref())), rb: value.rb.as_ref().map(|v| Vec2Key::from(v.as_ref())) }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeDeformKey {
    pub coord_x: Option<Vec<ordered_float::OrderedFloat<f32>>>,
    pub coord_y: Option<Vec<ordered_float::OrderedFloat<f32>>>,
    pub index: Option<Vec<i16>>,
}
impl From<&ssab::ssab::ss::format::PartAttributeDeformT> for PartAttributeDeformKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributeDeformT) -> Self {
        Self {
            coord_x: value.coord_x.as_ref().map(|c| c.iter().map(|f| ordered_float::OrderedFloat(*f)).collect()),
            coord_y: value.coord_y.as_ref().map(|c| c.iter().map(|f| ordered_float::OrderedFloat(*f)).collect()),
            index: value.index.as_ref().map(|i| i.to_vec()),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributesKey {
    pub update_flag: u64,
    pub cell: Option<PartAttributeCellKey>,
    pub position_x: ordered_float::OrderedFloat<f32>,
    pub position_y: ordered_float::OrderedFloat<f32>,
    pub position_z: ordered_float::OrderedFloat<f32>,
    pub rotation_x: ordered_float::OrderedFloat<f32>,
    pub rotation_y: ordered_float::OrderedFloat<f32>,
    pub rotation_z: ordered_float::OrderedFloat<f32>,
    pub scale_x: ordered_float::OrderedFloat<f32>,
    pub scale_y: ordered_float::OrderedFloat<f32>,
    pub local_scale_x: ordered_float::OrderedFloat<f32>,
    pub local_scale_y: ordered_float::OrderedFloat<f32>,
    pub alpha: ordered_float::OrderedFloat<f32>,
    pub local_alpha: ordered_float::OrderedFloat<f32>,
    pub priority: i32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub hide: bool,
    pub part_color: Option<PartAttributePartColorKey>,
    pub shader: Option<PartAttributeShaderKey>,
    pub vertex: Option<PartAttributeVertexKey>,
    pub pivot_x: ordered_float::OrderedFloat<f32>,
    pub pivot_y: ordered_float::OrderedFloat<f32>,
    pub anchor_x: ordered_float::OrderedFloat<f32>,
    pub anchor_y: ordered_float::OrderedFloat<f32>,
    pub size_x: ordered_float::OrderedFloat<f32>,
    pub size_y: ordered_float::OrderedFloat<f32>,
    pub img_flip_h: bool,
    pub img_flip_v: bool,
    pub uv_translation_x: ordered_float::OrderedFloat<f32>,
    pub uv_translation_y: ordered_float::OrderedFloat<f32>,
    pub uv_rotation_z: ordered_float::OrderedFloat<f32>,
    pub uv_scale_x: ordered_float::OrderedFloat<f32>,
    pub uv_scale_y: ordered_float::OrderedFloat<f32>,
    pub bounding_radius: ordered_float::OrderedFloat<f32>,
    pub mask: ordered_float::OrderedFloat<f32>,
    pub skew_x: ordered_float::OrderedFloat<f32>,
    pub skew_y: ordered_float::OrderedFloat<f32>,
    pub deform: Option<PartAttributeDeformKey>,
    pub texture: u32,
}
impl From<&ssab::ssab::ss::format::InitialPartAttributesT> for PartAttributesKey {
    fn from(value: &ssab::ssab::ss::format::InitialPartAttributesT) -> Self {
        Self {
            update_flag: value.update_flag.bits(),
            cell: value.cell.as_ref().map(|c| PartAttributeCellKey::from(c.as_ref())),
            position_x: ordered_float::OrderedFloat(value.position_x),
            position_y: ordered_float::OrderedFloat(value.position_y),
            position_z: ordered_float::OrderedFloat(value.position_z),
            rotation_x: ordered_float::OrderedFloat(value.rotation_x),
            rotation_y: ordered_float::OrderedFloat(value.rotation_y),
            rotation_z: ordered_float::OrderedFloat(value.rotation_z),
            scale_x: ordered_float::OrderedFloat(value.scale_x),
            scale_y: ordered_float::OrderedFloat(value.scale_y),
            local_scale_x: ordered_float::OrderedFloat(value.local_scale_x),
            local_scale_y: ordered_float::OrderedFloat(value.local_scale_y),
            alpha: ordered_float::OrderedFloat(value.alpha),
            local_alpha: ordered_float::OrderedFloat(value.local_alpha),
            priority: value.priority,
            flip_h: value.flip_h,
            flip_v: value.flip_v,
            hide: value.hide,
            part_color: value.part_color.as_ref().map(|c| PartAttributePartColorKey::from(c.as_ref())),
            shader: value.shader.as_ref().map(|s| PartAttributeShaderKey::from(s.as_ref())),
            vertex: value.vertex.as_ref().map(|v| PartAttributeVertexKey::from(v.as_ref())),
            pivot_x: ordered_float::OrderedFloat(value.pivot_x),
            pivot_y: ordered_float::OrderedFloat(value.pivot_y),
            anchor_x: ordered_float::OrderedFloat(value.anchor_x),
            anchor_y: ordered_float::OrderedFloat(value.anchor_y),
            size_x: ordered_float::OrderedFloat(value.size_x),
            size_y: ordered_float::OrderedFloat(value.size_y),
            img_flip_h: value.img_flip_h,
            img_flip_v: value.img_flip_v,
            uv_translation_x: ordered_float::OrderedFloat(value.uv_translation_x),
            uv_translation_y: ordered_float::OrderedFloat(value.uv_translation_y),
            uv_rotation_z: ordered_float::OrderedFloat(value.uv_rotation_z),
            uv_scale_x: ordered_float::OrderedFloat(value.uv_scale_x),
            uv_scale_y: ordered_float::OrderedFloat(value.uv_scale_y),
            bounding_radius: ordered_float::OrderedFloat(value.bounding_radius),
            mask: ordered_float::OrderedFloat(value.mask),
            skew_x: ordered_float::OrderedFloat(value.skew_x),
            skew_y: ordered_float::OrderedFloat(value.skew_y),
            deform: value.deform.as_ref().map(|d| PartAttributeDeformKey::from(d.as_ref())),
            texture: value.texture,
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributesVecKey {
    v: Vec<PartAttributesKey>,
}
impl From<&Vec<ssab::ssab::ss::format::InitialPartAttributesT>> for PartAttributesVecKey {
    fn from(value: &Vec<ssab::ssab::ss::format::InitialPartAttributesT>) -> Self {
        Self { v: value.iter().map(PartAttributesKey::from).collect() }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeInstanceKey {
    pub pingpong: bool,
    pub independent: bool,
    pub reverse: bool,
    pub loop_num: i32,
    pub start_label_hash: u32,
    pub start_offset: i32,
    pub end_label_hash: u32,
    pub end_offset: i32,
    pub speed: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::PartAttributeInstanceT> for PartAttributeInstanceKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributeInstanceT) -> Self {
        Self {
            pingpong: value.pingpong,
            independent: value.independent,
            reverse: value.reverse,
            loop_num: value.loop_num,
            start_label_hash: value.start_label_hash,
            start_offset: value.start_offset,
            end_label_hash: value.end_label_hash,
            end_offset: value.end_offset,
            speed: ordered_float::OrderedFloat(value.speed),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeUserDataKey {
    pub integer: Option<IntValueEntryKey>,
    pub rect: Option<BoundsKey>,
    pub point: Option<Vec2Key>,
    pub str: Option<String>,
}
impl From<&ssab::ssab::ss::format::PartAttributeUserDataT> for PartAttributeUserDataKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributeUserDataT) -> Self {
        Self { integer: value.integer.as_ref().map(|i| IntValueEntryKey::from(i.as_ref())), rect: value.rect.as_ref().map(|r| BoundsKey::from(r.as_ref())), point: value.point.as_ref().map(|p| Vec2Key::from(p.as_ref())), str: value.str.clone() }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum SignalCommandParamValueKey {
    Bool(BoolValueEntryKey),
    Index(IntValueEntryKey),
    Integer(IntValueEntryKey),
    Floating(FloatValueEntryKey),
    Text(StringValueEntryKey),
    None,
}
impl From<&ssab::ssab::ss::format::SignalCommandParamValueT> for SignalCommandParamValueKey {
    fn from(value: &ssab::ssab::ss::format::SignalCommandParamValueT) -> Self {
        match value {
            ssab::ssab::ss::format::SignalCommandParamValueT::Bool(v) => Self::Bool(BoolValueEntryKey::from(v.as_ref())),
            ssab::ssab::ss::format::SignalCommandParamValueT::Index(v) => Self::Index(IntValueEntryKey::from(v.as_ref())),
            ssab::ssab::ss::format::SignalCommandParamValueT::Integer(v) => Self::Integer(IntValueEntryKey::from(v.as_ref())),
            ssab::ssab::ss::format::SignalCommandParamValueT::Floating(v) => Self::Floating(FloatValueEntryKey::from(v.as_ref())),
            ssab::ssab::ss::format::SignalCommandParamValueT::Text(v) => Self::Text(StringValueEntryKey::from(v.as_ref())),
            _ => Self::None,
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct SignalCommandParamKey {
    pub param_id: Option<String>,
    pub type_: ssab::ssab::ss::format::SignalCommandParamType,
    pub value: SignalCommandParamValueKey,
}
impl From<&ssab::ssab::ss::format::SignalCommandParamT> for SignalCommandParamKey {
    fn from(value: &ssab::ssab::ss::format::SignalCommandParamT) -> Self {
        Self { param_id: Some(value.param_id.clone()), type_: value.type_, value: SignalCommandParamValueKey::from(&value.value) }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeSignalKey {
    pub active: bool,
    pub command_id: Option<String>,
    pub params: Option<Vec<SignalCommandParamKey>>,
}
impl From<&ssab::ssab::ss::format::PartAttributeSignalT> for PartAttributeSignalKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributeSignalT) -> Self {
        Self { active: value.active, command_id: Some(value.command_id.clone()), params: Some(value.params.iter().map(SignalCommandParamKey::from).collect()) }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeAudioKey {
    pub sound_list_name_hash: u32,
    pub sound_name_hash: u32,
    pub sound_name: Option<String>,
    pub loop_num: i32,
}
impl From<&ssab::ssab::ss::format::PartAttributeAudioT> for PartAttributeAudioKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributeAudioT) -> Self {
        Self { sound_list_name_hash: value.sound_list_name_hash, sound_name_hash: value.sound_name_hash, sound_name: Some(value.sound_name.clone()), loop_num: value.loop_num }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeEffectKey {
    pub start_time: i32,
    pub speed: ordered_float::OrderedFloat<f32>,
    pub independent: bool,
}
impl From<&ssab::ssab::ss::format::PartAttributeEffectT> for PartAttributeEffectKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributeEffectT) -> Self {
        Self { start_time: value.start_time, speed: ordered_float::OrderedFloat(value.speed), independent: value.independent }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartMeshBindingKey {
    pub vertex_influence_offset: Option<Vec<u32>>,
    pub influence_bone: Option<Vec<u16>>,
    pub influence_weight: Option<Vec<ordered_float::OrderedFloat<f32>>>,
    pub influence_coord_x: Option<Vec<ordered_float::OrderedFloat<f32>>>,
    pub influence_coord_y: Option<Vec<ordered_float::OrderedFloat<f32>>>,
}
impl From<&ssab::ssab::ss::format::PartMeshBindingT> for PartMeshBindingKey {
    fn from(value: &ssab::ssab::ss::format::PartMeshBindingT) -> Self {
        Self {
            vertex_influence_offset: value.vertex_influence_offset.clone(),
            influence_bone: value.influence_bone.clone(),
            influence_weight: value.influence_weight.as_ref().map(|v| v.iter().copied().map(ordered_float::OrderedFloat).collect()),
            influence_coord_x: value.influence_coord_x.as_ref().map(|v| v.iter().copied().map(ordered_float::OrderedFloat).collect()),
            influence_coord_y: value.influence_coord_y.as_ref().map(|v| v.iter().copied().map(ordered_float::OrderedFloat).collect()),
        }
    }
}

pub struct SsabCache<'a> {
    pub vec2_cache: HashMap<Vec2Key, flatbuffers::WIPOffset<ssab::ssab::ss::format::Vec2<'a>>>,
    pub bounds_cache: HashMap<BoundsKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::Bounds<'a>>>,
    pub region_cache: HashMap<RegionKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::Region<'a>>>,
    pub min_max2_cache: HashMap<MinMax2Key, flatbuffers::WIPOffset<ssab::ssab::ss::format::MinMax2<'a>>>,
    pub u8color_cache: HashMap<U8ColorKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::U8Color<'a>>>,
    pub none_value_table_cache: HashMap<NoneValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::NoneValueEntry<'a>>>,
    pub bool_value_table_cache: HashMap<BoolValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::BoolValueEntry<'a>>>,
    pub int_value_table_cache: HashMap<IntValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::IntValueEntry<'a>>>,
    pub float_value_table_cache: HashMap<FloatValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::FloatValueEntry<'a>>>,
    pub string_value_table_cache: HashMap<StringValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::StringValueEntry<'a>>>,
    pub vec2_value_table_cache: HashMap<Vec2ValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::Vec2ValueEntry<'a>>>,
    pub min_max2_value_table_cache: HashMap<MinMax2ValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::MinMax2ValueEntry<'a>>>,

    pub cell_cache: HashMap<CellKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::Cell<'a>>>,
    pub label_cache: HashMap<LabelKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::Label<'a>>>,
    pub part_attribute_cell_cache: HashMap<PartAttributeCellKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeCell<'a>>>,
    pub attribute_part_color_key_value_color_cache: HashMap<SsAttributePartColorKeyValueColorKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::SsAttributePartColorKeyValueColor<'a>>>,
    pub part_attribute_part_color_cache: HashMap<PartAttributePartColorKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributePartColor<'a>>>,
    pub part_attribute_shader_cache: HashMap<PartAttributeShaderKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeShader<'a>>>,
    pub part_attribute_vertex_cache: HashMap<PartAttributeVertexKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeVertex<'a>>>,
    pub part_attribute_deform_cache: HashMap<PartAttributeDeformKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeDeform<'a>>>,
    pub part_attributes_cache: HashMap<PartAttributesKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::InitialPartAttributes<'a>>>,
    pub part_attributes_vec_cache: HashMap<PartAttributesVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::InitialPartAttributes<'a>>>>>,
    pub part_attribute_instance_cache: HashMap<PartAttributeInstanceKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeInstance<'a>>>,
    pub part_attribute_user_data_cache: HashMap<PartAttributeUserDataKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeUserData<'a>>>,
    pub part_attribute_signal_cache: HashMap<PartAttributeSignalKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeSignal<'a>>>,
    pub part_attribute_audio_cache: HashMap<PartAttributeAudioKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeAudio<'a>>>,
    pub part_attribute_effect_cache: HashMap<PartAttributeEffectKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeEffect<'a>>>,
    pub initial_events_cache: HashMap<InitialEventsKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::InitialEvents<'a>>>,
    pub initial_events_vec_cache: HashMap<InitialEventsVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::InitialEvents<'a>>>>>,

    pub part_animation_data_cache: HashMap<PartAnimationDataKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationData<'a>>>,

    pub part_mesh_binding_cache: HashMap<PartMeshBindingKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartMeshBinding<'a>>>,
}
impl<'a> SsabCache<'a> {
    pub fn new() -> Self {
        Self {
            vec2_cache: HashMap::new(),
            bounds_cache: HashMap::new(),
            region_cache: HashMap::new(),
            min_max2_cache: HashMap::new(),
            none_value_table_cache: HashMap::new(),
            bool_value_table_cache: HashMap::new(),
            int_value_table_cache: HashMap::new(),
            float_value_table_cache: HashMap::new(),
            string_value_table_cache: HashMap::new(),
            vec2_value_table_cache: HashMap::new(),
            min_max2_value_table_cache: HashMap::new(),
            u8color_cache: HashMap::new(),

            cell_cache: HashMap::new(),
            label_cache: HashMap::new(),
            part_attribute_cell_cache: HashMap::new(),
            attribute_part_color_key_value_color_cache: HashMap::new(),
            part_attribute_part_color_cache: HashMap::new(),
            part_attribute_shader_cache: HashMap::new(),
            part_attribute_vertex_cache: HashMap::new(),
            part_attribute_deform_cache: HashMap::new(),
            part_attributes_cache: HashMap::new(),
            part_attributes_vec_cache: HashMap::new(),
            part_attribute_instance_cache: HashMap::new(),
            part_attribute_user_data_cache: HashMap::new(),
            part_attribute_signal_cache: HashMap::new(),
            part_attribute_audio_cache: HashMap::new(),
            part_attribute_effect_cache: HashMap::new(),
            initial_events_cache: HashMap::new(),
            initial_events_vec_cache: HashMap::new(),

            part_animation_data_cache: HashMap::new(),

            part_mesh_binding_cache: HashMap::new(),
        }
    }
    fn create_shared_vec2(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, vec2t: &ssab::ssab::ss::format::Vec2T) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::Vec2<'a>> {
        let key = Vec2Key::from(vec2t);
        if let Some(&offset) = self.vec2_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::Vec2Args { v1: vec2t.v1, v2: vec2t.v2 };
        let offset = ssab::ssab::ss::format::Vec2::create(fb, &args);
        self.vec2_cache.insert(key, offset);
        offset
    }

    fn create_shared_bounds(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, boundst: &ssab::ssab::ss::format::BoundsT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::Bounds<'a>> {
        let key = BoundsKey::from(boundst);
        if let Some(&offset) = self.bounds_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::BoundsArgs { left: boundst.left, top: boundst.top, right: boundst.right, bottom: boundst.bottom };
        let offset = ssab::ssab::ss::format::Bounds::create(fb, &args);
        self.bounds_cache.insert(key, offset);
        offset
    }

    fn create_shared_region(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, regiont: &ssab::ssab::ss::format::RegionT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::Region<'a>> {
        let key = RegionKey::from(regiont);
        if let Some(&offset) = self.region_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::RegionArgs { x: regiont.x, y: regiont.y, w: regiont.w, h: regiont.h };
        let offset = ssab::ssab::ss::format::Region::create(fb, &args);
        self.region_cache.insert(key, offset);
        offset
    }

    fn create_shared_min_max2(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, min_max2t: &ssab::ssab::ss::format::MinMax2T) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::MinMax2<'a>> {
        let key = MinMax2Key::from(min_max2t);
        if let Some(&offset) = self.min_max2_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::MinMax2Args { x_min: min_max2t.x_min, y_min: min_max2t.y_min, x_max: min_max2t.x_max, y_max: min_max2t.y_max };
        let offset = ssab::ssab::ss::format::MinMax2::create(fb, &args);
        self.min_max2_cache.insert(key, offset);
        offset
    }

    fn create_shared_u8color(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, u8colort: &ssab::ssab::ss::format::U8ColorT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::U8Color<'a>> {
        let key = U8ColorKey::from(u8colort);
        if let Some(&offset) = self.u8color_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::U8ColorArgs { a: u8colort.a, r: u8colort.r, g: u8colort.g, b: u8colort.b };
        let offset = ssab::ssab::ss::format::U8Color::create(fb, &args);
        self.u8color_cache.insert(key, offset);
        offset
    }

    fn create_shared_none_value_entry(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, none_value_table: &ssab::ssab::ss::format::NoneValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::NoneValueEntry<'a>> {
        let key = NoneValueEntryKey::from(none_value_table);
        if let Some(&offset) = self.none_value_table_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::NoneValueEntryArgs::default();
        let offset = ssab::ssab::ss::format::NoneValueEntry::create(fb, &args);
        self.none_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_bool_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, bool_value_tablet: &ssab::ssab::ss::format::BoolValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::BoolValueEntry<'a>> {
        let key = BoolValueEntryKey::from(bool_value_tablet);
        if let Some(&offset) = self.bool_value_table_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::BoolValueEntryArgs { value: bool_value_tablet.value };
        let offset = ssab::ssab::ss::format::BoolValueEntry::create(fb, &args);
        self.bool_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_int_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, int_value_table_t: &ssab::ssab::ss::format::IntValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::IntValueEntry<'a>> {
        let key = IntValueEntryKey::from(int_value_table_t);
        if let Some(&offset) = self.int_value_table_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::IntValueEntryArgs { value: int_value_table_t.value };
        let offset = ssab::ssab::ss::format::IntValueEntry::create(fb, &args);
        self.int_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_float_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, float_value_table_t: &ssab::ssab::ss::format::FloatValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::FloatValueEntry<'a>> {
        let key = FloatValueEntryKey::from(float_value_table_t);
        if let Some(&offset) = self.float_value_table_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::FloatValueEntryArgs { value: float_value_table_t.value };
        let offset = ssab::ssab::ss::format::FloatValueEntry::create(fb, &args);
        self.float_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_string_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, string_value_table_t: &ssab::ssab::ss::format::StringValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::StringValueEntry<'a>> {
        let key = StringValueEntryKey::from(string_value_table_t);
        if let Some(&offset) = self.string_value_table_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::StringValueEntryArgs { value: Some(fb.create_shared_string(&string_value_table_t.value)) };
        let offset = ssab::ssab::ss::format::StringValueEntry::create(fb, &args);
        self.string_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_vec2_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, vec2_value_table_t: &ssab::ssab::ss::format::Vec2ValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::Vec2ValueEntry<'a>> {
        let key = Vec2ValueEntryKey::from(vec2_value_table_t);
        if let Some(&offset) = self.vec2_value_table_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::Vec2ValueEntryArgs { value: Some(self.create_shared_vec2(fb, &vec2_value_table_t.value)) };
        let offset = ssab::ssab::ss::format::Vec2ValueEntry::create(fb, &args);
        self.vec2_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_min_max2_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, min_max2_value_table_t: &ssab::ssab::ss::format::MinMax2ValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::MinMax2ValueEntry<'a>> {
        let key = MinMax2ValueEntryKey::from(min_max2_value_table_t);
        if let Some(&offset) = self.min_max2_value_table_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::MinMax2ValueEntryArgs { value: Some(self.create_shared_min_max2(fb, &min_max2_value_table_t.value)) };
        let offset = ssab::ssab::ss::format::MinMax2ValueEntry::create(fb, &args);
        self.min_max2_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data: &ssab::ssab::ss::format::PartAnimationDataT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationData<'a>> {
        let key = PartAnimationDataKey::from(part_animation_data);
        if let Some(&offset) = self.part_animation_data_cache.get(&key) {
            return offset;
        }
        let offset = part_animation_data.pack(fb);
        self.part_animation_data_cache.insert(key, offset);
        offset
    }

    fn create_keyframes(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, keyframes: &ssab::ssab::ss::format::KeyframesT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::Keyframes<'a>> {
        let mut args = ssab::ssab::ss::format::KeyframesArgs::default();
        args.runs = Some(fb.create_vector(&keyframes.runs));
        args.frame = Some(fb.create_vector(&keyframes.frame));
        args.interpolation = Some(fb.create_vector(&keyframes.interpolation));
        args.curve = Some(fb.create_vector(&keyframes.curve));
        args.value = Some(fb.create_vector(&keyframes.value));
        let curves = keyframes.curves.iter().map(|curve| curve.pack()).collect::<Vec<_>>();
        args.curves = Some(fb.create_vector(&curves));
        args.easing_rates = Some(fb.create_vector(&keyframes.easing_rates));
        let cells = keyframes.cells.iter().map(|cell| self.create_shared_part_attribute_cell(fb, cell)).collect::<Vec<_>>();
        args.cells = Some(fb.create_vector(&cells));
        let part_colors = keyframes.part_colors.iter().map(|part_color| self.create_shared_part_attribute_part_color(fb, part_color)).collect::<Vec<_>>();
        args.part_colors = Some(fb.create_vector(&part_colors));
        let shaders = keyframes.shaders.iter().map(|shader| self.create_shared_part_attribute_shader(fb, shader)).collect::<Vec<_>>();
        args.shaders = Some(fb.create_vector(&shaders));
        let vertices = keyframes.vertices.iter().map(|vertex| self.create_shared_part_attribute_vertex(fb, vertex)).collect::<Vec<_>>();
        args.vertices = Some(fb.create_vector(&vertices));
        let skews = keyframes.skews.iter().map(|skew| self.create_shared_vec2(fb, skew)).collect::<Vec<_>>();
        args.skews = Some(fb.create_vector(&skews));
        let deforms = keyframes.deforms.iter().map(|deform| self.create_shared_part_attribute_deform(fb, deform)).collect::<Vec<_>>();
        args.deforms = Some(fb.create_vector(&deforms));
        ssab::ssab::ss::format::Keyframes::create(fb, &args)
    }
}

impl<'a> SsabCache<'a> {
    fn create_shared_cell(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, cell: &ssab::ssab::ss::format::CellT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::Cell<'a>> {
        let key = CellKey::from(cell);
        if let Some(&offset) = self.cell_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::CellArgs::default();
        args.name_hash = cell.name_hash;
        args.name = Some(fb.create_shared_string(&cell.name));
        args.region = Some(self.create_shared_region(fb, &cell.region));
        args.pivot = Some(self.create_shared_vec2(fb, &cell.pivot));
        args.rotated = cell.rotated;
        args.table_coord_x = cell.table_coord_x.as_ref().map(|p| fb.create_vector(p));
        args.table_coord_y = cell.table_coord_y.as_ref().map(|p| fb.create_vector(p));
        args.table_index_vertex = cell.table_index_vertex.as_ref().map(|p| fb.create_vector(p));

        let offset = ssab::ssab::ss::format::Cell::create(fb, &args);
        self.cell_cache.insert(key, offset);
        offset
    }

    fn create_shared_label(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, label_t: &ssab::ssab::ss::format::LabelT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::Label<'a>> {
        let key = LabelKey::from(label_t);
        if let Some(&offset) = self.label_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::LabelArgs::default();
        args.name_hash = label_t.name_hash;
        args.name = Some(fb.create_shared_string(&label_t.name));
        args.time = label_t.time;

        let offset = ssab::ssab::ss::format::Label::create(fb, &args);
        self.label_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attribute_cell(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_cell: &ssab::ssab::ss::format::PartAttributeCellT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeCell<'a>> {
        let key = PartAttributeCellKey::from(part_attribute_cell);
        if let Some(&offset) = self.part_attribute_cell_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::PartAttributeCellArgs::default();
        args.cellmap_name_hash = part_attribute_cell.cellmap_name_hash;
        args.name_hash = part_attribute_cell.name_hash;

        let offset = ssab::ssab::ss::format::PartAttributeCell::create(fb, &args);
        self.part_attribute_cell_cache.insert(key, offset);
        offset
    }

    fn create_shared_attribute_part_color_key_value_color(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_attribute_cell: &ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::SsAttributePartColorKeyValueColor<'a>> {
        let key = SsAttributePartColorKeyValueColorKey::from(part_attribute_cell);
        if let Some(&offset) = self.attribute_part_color_key_value_color_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorArgs::default();
        args.rgba = part_attribute_cell.rgba.as_ref().map(|rgba| self.create_shared_u8color(fb, rgba));
        args.rate = part_attribute_cell.rate;

        let offset = ssab::ssab::ss::format::SsAttributePartColorKeyValueColor::create(fb, &args);
        self.attribute_part_color_key_value_color_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attribute_part_color(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_attribute_part_color: &ssab::ssab::ss::format::PartAttributePartColorT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributePartColor<'a>> {
        let key = PartAttributePartColorKey::from(part_attribute_part_color);
        if let Some(&offset) = self.part_attribute_part_color_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::PartAttributePartColorArgs::default();
        args.blend_type = part_attribute_part_color.blend_type;
        args.lt = part_attribute_part_color.lt.as_ref().map(|lt| self.create_shared_attribute_part_color_key_value_color(fb, lt));
        args.rt = part_attribute_part_color.rt.as_ref().map(|rt| self.create_shared_attribute_part_color_key_value_color(fb, rt));
        args.lb = part_attribute_part_color.lb.as_ref().map(|lb| self.create_shared_attribute_part_color_key_value_color(fb, lb));
        args.rb = part_attribute_part_color.rb.as_ref().map(|rb| self.create_shared_attribute_part_color_key_value_color(fb, rb));

        let offset = ssab::ssab::ss::format::PartAttributePartColor::create(fb, &args);
        self.part_attribute_part_color_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attribute_shader(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_shader: &ssab::ssab::ss::format::PartAttributeShaderT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeShader<'a>> {
        let key = PartAttributeShaderKey::from(part_attribute_shader);
        if let Some(&offset) = self.part_attribute_shader_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::PartAttributeShaderArgs::default();
        args.id = Some(fb.create_shared_string(&part_attribute_shader.id));
        args.id_hash = part_attribute_shader.id_hash;
        args.param0 = part_attribute_shader.param0;
        args.param1 = part_attribute_shader.param1;
        args.param2 = part_attribute_shader.param2;
        args.param3 = part_attribute_shader.param3;
        args.param4 = part_attribute_shader.param4;
        args.param5 = part_attribute_shader.param5;
        args.param6 = part_attribute_shader.param6;
        args.param7 = part_attribute_shader.param7;
        args.map0_cellmap_name_hash = part_attribute_shader.map0_cellmap_name_hash;
        args.map0_name_hash = part_attribute_shader.map0_name_hash;
        args.map1_cellmap_name_hash = part_attribute_shader.map1_cellmap_name_hash;
        args.map1_name_hash = part_attribute_shader.map1_name_hash;

        let offset = ssab::ssab::ss::format::PartAttributeShader::create(fb, &args);
        self.part_attribute_shader_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attribute_vertex(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_vertex: &ssab::ssab::ss::format::PartAttributeVertexT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeVertex<'a>> {
        let key = PartAttributeVertexKey::from(part_attribute_vertex);
        if let Some(&offset) = self.part_attribute_vertex_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::PartAttributeVertexArgs::default();
        args.lt = part_attribute_vertex.lt.as_ref().map(|lt| self.create_shared_vec2(fb, lt));
        args.rt = part_attribute_vertex.rt.as_ref().map(|rt| self.create_shared_vec2(fb, rt));
        args.lb = part_attribute_vertex.lb.as_ref().map(|lb| self.create_shared_vec2(fb, lb));
        args.rb = part_attribute_vertex.rb.as_ref().map(|rb| self.create_shared_vec2(fb, rb));

        let offset = ssab::ssab::ss::format::PartAttributeVertex::create(fb, &args);
        self.part_attribute_vertex_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attribute_deform(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_deform: &ssab::ssab::ss::format::PartAttributeDeformT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeDeform<'a>> {
        let key = PartAttributeDeformKey::from(part_attribute_deform);
        if let Some(&offset) = self.part_attribute_deform_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::PartAttributeDeformArgs::default();
        args.coord_x = part_attribute_deform.coord_x.as_ref().map(|p| fb.create_vector(p));
        args.coord_y = part_attribute_deform.coord_y.as_ref().map(|p| fb.create_vector(p));
        args.index = part_attribute_deform.index.as_ref().map(|p| fb.create_vector(p));

        let offset = ssab::ssab::ss::format::PartAttributeDeform::create(fb, &args);
        self.part_attribute_deform_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attributes(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attributes: &ssab::ssab::ss::format::InitialPartAttributesT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::InitialPartAttributes<'a>> {
        let key = PartAttributesKey::from(part_attributes);
        if let Some(&offset) = self.part_attributes_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::InitialPartAttributesArgs::default();
        args.update_flag = part_attributes.update_flag;
        args.cell = part_attributes.cell.as_ref().map(|cell| self.create_shared_part_attribute_cell(fb, cell));
        args.position_x = part_attributes.position_x;
        args.position_y = part_attributes.position_y;
        args.position_z = part_attributes.position_z;
        args.rotation_x = part_attributes.rotation_x;
        args.rotation_y = part_attributes.rotation_y;
        args.rotation_z = part_attributes.rotation_z;
        args.scale_x = part_attributes.scale_x;
        args.scale_y = part_attributes.scale_y;
        args.local_scale_x = part_attributes.local_scale_x;
        args.local_scale_y = part_attributes.local_scale_y;
        args.alpha = part_attributes.alpha;
        args.local_alpha = part_attributes.local_alpha;
        args.priority = part_attributes.priority;
        args.flip_h = part_attributes.flip_h;
        args.flip_v = part_attributes.flip_v;
        args.hide = part_attributes.hide;
        args.part_color = part_attributes.part_color.as_ref().map(|parts_color| self.create_shared_part_attribute_part_color(fb, parts_color));
        args.shader = part_attributes.shader.as_ref().map(|shader| self.create_shared_part_attribute_shader(fb, shader));
        args.vertex = part_attributes.vertex.as_ref().map(|vertex| self.create_shared_part_attribute_vertex(fb, vertex));
        args.pivot_x = part_attributes.pivot_x;
        args.pivot_y = part_attributes.pivot_y;
        args.anchor_x = part_attributes.anchor_x;
        args.anchor_y = part_attributes.anchor_y;
        args.size_x = part_attributes.size_x;
        args.size_y = part_attributes.size_y;
        args.img_flip_h = part_attributes.img_flip_h;
        args.img_flip_v = part_attributes.img_flip_v;
        args.uv_translation_x = part_attributes.uv_translation_x;
        args.uv_translation_y = part_attributes.uv_translation_y;
        args.uv_rotation_z = part_attributes.uv_rotation_z;
        args.uv_scale_x = part_attributes.uv_scale_x;
        args.uv_scale_y = part_attributes.uv_scale_y;
        args.bounding_radius = part_attributes.bounding_radius;
        args.mask = part_attributes.mask;
        args.skew_x = part_attributes.skew_x;
        args.skew_y = part_attributes.skew_y;
        args.deform = part_attributes.deform.as_ref().map(|deform| self.create_shared_part_attribute_deform(fb, deform));
        args.texture = part_attributes.texture;
        let offset = ssab::ssab::ss::format::InitialPartAttributes::create(fb, &args);

        self.part_attributes_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attributes_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_attributes_vec: &Vec<ssab::ssab::ss::format::InitialPartAttributesT>,
    ) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::InitialPartAttributes<'a>>>> {
        let key = PartAttributesVecKey::from(part_attributes_vec);
        if let Some(&offset) = self.part_attributes_vec_cache.get(&key) {
            return offset;
        }

        let v = part_attributes_vec.iter().map(|part_attributes| self.create_shared_part_attributes(fb, part_attributes)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_attributes_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_instance(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_instance: &ssab::ssab::ss::format::PartAttributeInstanceT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeInstance<'a>> {
        let key = PartAttributeInstanceKey::from(part_attribute_instance);
        if let Some(&offset) = self.part_attribute_instance_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::PartAttributeInstanceArgs::default();
        args.pingpong = part_attribute_instance.pingpong;
        args.independent = part_attribute_instance.independent;
        args.reverse = part_attribute_instance.reverse;
        args.loop_num = part_attribute_instance.loop_num;
        args.start_label_hash = part_attribute_instance.start_label_hash;
        args.start_offset = part_attribute_instance.start_offset;
        args.end_label_hash = part_attribute_instance.end_label_hash;
        args.end_offset = part_attribute_instance.end_offset;
        args.speed = part_attribute_instance.speed;
        let offset = ssab::ssab::ss::format::PartAttributeInstance::create(fb, &args);

        self.part_attribute_instance_cache.insert(key, offset);
        offset
    }

    fn create_shared_user_data(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_user_data: &ssab::ssab::ss::format::PartAttributeUserDataT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeUserData<'a>> {
        let key = PartAttributeUserDataKey::from(part_attribute_user_data);
        if let Some(&offset) = self.part_attribute_user_data_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::PartAttributeUserDataArgs::default();
        args.integer = part_attribute_user_data.integer.as_ref().map(|integer| self.create_shared_int_value_table(fb, integer));
        args.rect = part_attribute_user_data.rect.as_ref().map(|rect| self.create_shared_bounds(fb, rect));
        args.point = part_attribute_user_data.point.as_ref().map(|point| self.create_shared_vec2(fb, point));
        args.str = part_attribute_user_data.str.as_ref().map(|str| fb.create_shared_string(str));

        let offset = ssab::ssab::ss::format::PartAttributeUserData::create(fb, &args);

        self.part_attribute_user_data_cache.insert(key, offset);
        offset
    }

    fn create_shared_signal(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_signal: &ssab::ssab::ss::format::PartAttributeSignalT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeSignal<'a>> {
        let key = PartAttributeSignalKey::from(part_attribute_signal);
        if let Some(&offset) = self.part_attribute_signal_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::PartAttributeSignalArgs::default();
        args.active = part_attribute_signal.active;
        args.command_id = Some(fb.create_shared_string(&part_attribute_signal.command_id));
        args.params = Some({
            let v = part_attribute_signal
                .params
                .iter()
                .map(|param| {
                    let mut p_args = ssab::ssab::ss::format::SignalCommandParamArgs::default();
                    p_args.param_id = Some(fb.create_shared_string(&param.param_id));
                    p_args.type_ = param.type_;
                    p_args.value = match &param.value {
                        ssab::ssab::ss::format::SignalCommandParamValueT::Bool(v) => {
                            let offset = self.create_shared_bool_value_table(fb, v);
                            p_args.value_type = ssab::ssab::ss::format::SignalCommandParamValue::Bool;
                            Some(offset.as_union_value())
                        }
                        ssab::ssab::ss::format::SignalCommandParamValueT::Index(v) => {
                            let offset = self.create_shared_int_value_table(fb, v);
                            p_args.value_type = ssab::ssab::ss::format::SignalCommandParamValue::Index;
                            Some(offset.as_union_value())
                        }
                        ssab::ssab::ss::format::SignalCommandParamValueT::Integer(v) => {
                            let offset = self.create_shared_int_value_table(fb, v);
                            p_args.value_type = ssab::ssab::ss::format::SignalCommandParamValue::Integer;
                            Some(offset.as_union_value())
                        }
                        ssab::ssab::ss::format::SignalCommandParamValueT::Floating(v) => {
                            let offset = self.create_shared_float_value_table(fb, v);
                            p_args.value_type = ssab::ssab::ss::format::SignalCommandParamValue::Floating;
                            Some(offset.as_union_value())
                        }
                        ssab::ssab::ss::format::SignalCommandParamValueT::Text(v) => {
                            let offset = self.create_shared_string_value_table(fb, v);
                            p_args.value_type = ssab::ssab::ss::format::SignalCommandParamValue::Text;
                            Some(offset.as_union_value())
                        }
                        ssab::ssab::ss::format::SignalCommandParamValueT::None(v) => {
                            let offset = self.create_shared_none_value_entry(fb, v);
                            p_args.value_type = ssab::ssab::ss::format::SignalCommandParamValue::None;
                            Some(offset.as_union_value())
                        }
                        _ => None,
                    };

                    ssab::ssab::ss::format::SignalCommandParam::create(fb, &p_args)
                })
                .collect::<Vec<_>>();
            fb.create_vector(&v)
        });

        let offset = ssab::ssab::ss::format::PartAttributeSignal::create(fb, &args);
        self.part_attribute_signal_cache.insert(key, offset);
        offset
    }

    fn create_shared_audio(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_audio: &ssab::ssab::ss::format::PartAttributeAudioT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeAudio<'a>> {
        let key = PartAttributeAudioKey::from(part_attribute_audio);
        if let Some(&offset) = self.part_attribute_audio_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::PartAttributeAudioArgs::default();
        args.sound_list_name_hash = part_attribute_audio.sound_list_name_hash;
        args.sound_name_hash = part_attribute_audio.sound_name_hash;
        args.sound_name = Some(fb.create_shared_string(&part_attribute_audio.sound_name));
        args.loop_num = part_attribute_audio.loop_num;
        let offset = ssab::ssab::ss::format::PartAttributeAudio::create(fb, &args);

        self.part_attribute_audio_cache.insert(key, offset);
        offset
    }

    fn create_shared_effect(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_effect: &ssab::ssab::ss::format::PartAttributeEffectT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAttributeEffect<'a>> {
        let key = PartAttributeEffectKey::from(part_attribute_effect);
        if let Some(&offset) = self.part_attribute_effect_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::PartAttributeEffectArgs::default();
        args.start_time = part_attribute_effect.start_time;
        args.speed = part_attribute_effect.speed;
        args.independent = part_attribute_effect.independent;
        let offset = ssab::ssab::ss::format::PartAttributeEffect::create(fb, &args);

        self.part_attribute_effect_cache.insert(key, offset);
        offset
    }

    fn create_shared_initial_events(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, initial_events: &ssab::ssab::ss::format::InitialEventsT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::InitialEvents<'a>> {
        let key = InitialEventsKey::from(initial_events);
        if let Some(&offset) = self.initial_events_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::ss::format::InitialEventsArgs::default();
        args.user = initial_events.user.as_ref().map(|user| self.create_shared_user_data(fb, user));
        args.signal = initial_events.signal.as_ref().map(|signal| self.create_shared_signal(fb, signal));
        args.audio = initial_events.audio.as_ref().map(|audio| self.create_shared_audio(fb, audio));
        args.instance = initial_events.instance.as_ref().map(|instance| self.create_shared_part_instance(fb, instance));
        args.effect = initial_events.effect.as_ref().map(|effect| self.create_shared_effect(fb, effect));
        let offset = ssab::ssab::ss::format::InitialEvents::create(fb, &args);
        self.initial_events_cache.insert(key, offset);
        offset
    }

    fn create_shared_initial_events_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        initial_events_array: &[ssab::ssab::ss::format::InitialEventsT],
    ) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::InitialEvents<'a>>>> {
        let key = InitialEventsVecKey::from(initial_events_array);
        if let Some(&offset) = self.initial_events_vec_cache.get(&key) {
            return offset;
        }
        let v = initial_events_array.iter().map(|t| self.create_shared_initial_events(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.initial_events_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_type(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_type: &ssab::ssab::ss::format::PartTypeT) -> flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset> {
        match part_type {
            ssab::ssab::ss::format::PartTypeT::PartTypeShape(v) => {
                /*
                let mut p_args = ssab::ssab::ss::format::PartTypeShapeArgs::default();
                p_args.shape_type = v.shape_type;
                let offset = ssab::ssab::ss::format::PartTypeShape::create(fb, &p_args);
                offset.as_union_value()
                 */
                v.pack(fb).as_union_value()
            }
            ssab::ssab::ss::format::PartTypeT::PartTypeText(v) => {
                let mut p_args = ssab::ssab::ss::format::PartTypeTextArgs::default();
                p_args.text = v.text.as_ref().map(|s| fb.create_shared_string(s));
                p_args.text_bitmap = v.text_bitmap;
                p_args.text_family = v.text_family.as_ref().map(|s| fb.create_shared_string(s));
                p_args.text_char_map = v.text_char_map.as_ref().map(|s| fb.create_shared_string(s));
                p_args.text_size = v.text_size;
                p_args.text_space = v.text_space;
                p_args.text_smooth = v.text_smooth;
                p_args.text_width = v.text_width;
                p_args.text_height = v.text_height;
                p_args.e_anchor = v.e_anchor;
                let offset = ssab::ssab::ss::format::PartTypeText::create(fb, &p_args);
                offset.as_union_value()
            }
            ssab::ssab::ss::format::PartTypeT::PartTypeNines(v) => v.pack(fb).as_union_value(),
            ssab::ssab::ss::format::PartTypeT::PartTypeInstance(v) => {
                let mut p_args = ssab::ssab::ss::format::PartTypeInstanceArgs::default();
                p_args.ref_anime_pack = Some(fb.create_shared_string(&v.ref_anime_pack));
                p_args.ref_anime_pack_hash = v.ref_anime_pack_hash;
                p_args.ref_anime_name = Some(fb.create_shared_string(&v.ref_anime_name));
                p_args.ref_anime_name_hash = v.ref_anime_name_hash;
                let offset = ssab::ssab::ss::format::PartTypeInstance::create(fb, &p_args);
                offset.as_union_value()
            }
            ssab::ssab::ss::format::PartTypeT::PartTypeEffect(v) => {
                let mut p_args = ssab::ssab::ss::format::PartTypeEffectArgs::default();
                p_args.ref_effect_name_hash = v.ref_effect_name_hash;
                let offset = ssab::ssab::ss::format::PartTypeEffect::create(fb, &p_args);
                offset.as_union_value()
            }
            _ => {
                let p_args = ssab::ssab::ss::format::NoneValueEntryT::default();
                let offset = self.create_shared_none_value_entry(fb, &p_args);
                offset.as_union_value()
            }
        }
    }

    fn create_shared_mesh_binding(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, t: &ssab::ssab::ss::format::PartMeshBindingT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartMeshBinding<'a>> {
        let key = PartMeshBindingKey::from(t);
        if let Some(&offset) = self.part_mesh_binding_cache.get(&key) {
            return offset;
        }
        let vertex_influence_offset = t.vertex_influence_offset.as_ref().map(|v| fb.create_vector(v));
        let influence_bone = t.influence_bone.as_ref().map(|v| fb.create_vector(v));
        let influence_weight = t.influence_weight.as_ref().map(|v| fb.create_vector(v));
        let influence_coord_x = t.influence_coord_x.as_ref().map(|v| fb.create_vector(v));
        let influence_coord_y = t.influence_coord_y.as_ref().map(|v| fb.create_vector(v));
        let args = ssab::ssab::ss::format::PartMeshBindingArgs { vertex_influence_offset, influence_bone, influence_weight, influence_coord_x, influence_coord_y };
        let offset = ssab::ssab::ss::format::PartMeshBinding::create(fb, &args);
        self.part_mesh_binding_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, ps: &[ssab::ssab::ss::format::PartDataT]) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartData<'a>>>> {
        let parts = ps
            .iter()
            .map(|part| {
                let mut args = ssab::ssab::ss::format::PartDataArgs::default();
                args.name = Some(fb.create_shared_string(&part.name));
                args.parent_index = part.parent_index;
                args.bounds_type = part.bounds_type;
                args.blend_type = part.blend_type;
                args.mask_influence = part.mask_influence;
                args.mask_write = part.mask_write;
                args.visible_inside_mask = part.visible_inside_mask;
                args.draw_as_mask = part.draw_as_mask;
                args.output_influence = part.output_influence;
                args.mesh_binding = part.mesh_binding.as_ref().map(|b| self.create_shared_mesh_binding(fb, b));
                args.part_type_type = part.part_type.part_type_type();
                args.part_type = Some(self.create_shared_part_type(fb, &part.part_type));

                ssab::ssab::ss::format::PartData::create(fb, &args)
            })
            .collect::<Vec<_>>();
        fb.create_vector(&parts)
    }

    fn create_shared_parts_animation_data(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        pa: &[ssab::ssab::ss::format::PartAnimationDataT],
    ) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationData<'a>>>> {
        let part_animation_data = pa.iter().map(|part_animation_data| self.create_shared_part_animation_data(fb, part_animation_data)).collect::<Vec<_>>();
        fb.create_vector(&part_animation_data)
    }

    fn create_shared_events_per_frame(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        es: &[ssab::ssab::ss::format::EventsPerFrameT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::EventsPerFrame<'a>>>> {
        let v = es
            .iter()
            .map(|event| {
                let mut e_args = ssab::ssab::ss::format::EventsPerFrameArgs::default();
                e_args.frame_index = event.frame_index;
                e_args.users = event.users.as_ref().map(|users| {
                    let v = users
                        .iter()
                        .map(|user| {
                            let mut u_args = ssab::ssab::ss::format::EventUserDataArgs::default();
                            u_args.part_index = user.part_index;
                            u_args.value = Some(self.create_shared_user_data(fb, &user.value));
                            ssab::ssab::ss::format::EventUserData::create(fb, &u_args)
                        })
                        .collect::<Vec<_>>();
                    fb.create_vector(&v)
                });

                e_args.signals = event.signals.as_ref().map(|signals| {
                    let v = signals
                        .iter()
                        .map(|signal| {
                            let mut s_args = ssab::ssab::ss::format::EventSignalArgs::default();
                            s_args.part_index = signal.part_index;
                            s_args.value = Some(self.create_shared_signal(fb, &signal.value));
                            ssab::ssab::ss::format::EventSignal::create(fb, &s_args)
                        })
                        .collect::<Vec<_>>();
                    fb.create_vector(&v)
                });
                e_args.audios = event.audios.as_ref().map(|audios| {
                    let v = audios
                        .iter()
                        .map(|audio| {
                            let mut a_args = ssab::ssab::ss::format::EventAudioArgs::default();
                            a_args.part_index = audio.part_index;
                            a_args.value = Some(self.create_shared_audio(fb, &audio.value));
                            ssab::ssab::ss::format::EventAudio::create(fb, &a_args)
                        })
                        .collect::<Vec<_>>();
                    fb.create_vector(&v)
                });
                e_args.instances = event.instances.as_ref().map(|instances| {
                    let v = instances
                        .iter()
                        .map(|instance| {
                            let mut i_args = ssab::ssab::ss::format::EventInstanceArgs::default();
                            i_args.part_index = instance.part_index;
                            i_args.value = Some(self.create_shared_part_instance(fb, &instance.value));

                            ssab::ssab::ss::format::EventInstance::create(fb, &i_args)
                        })
                        .collect::<Vec<_>>();
                    fb.create_vector(&v)
                });
                e_args.effects = event.effects.as_ref().map(|effects| {
                    let v = effects
                        .iter()
                        .map(|effect| {
                            let mut e_args = ssab::ssab::ss::format::EventEffectArgs::default();
                            e_args.part_index = effect.part_index;
                            e_args.value = Some(self.create_shared_effect(fb, &effect.value));
                            ssab::ssab::ss::format::EventEffect::create(fb, &e_args)
                        })
                        .collect::<Vec<_>>();
                    fb.create_vector(&v)
                });

                ssab::ssab::ss::format::EventsPerFrame::create(fb, &e_args)
            })
            .collect::<Vec<_>>();
        fb.create_vector(&v)
    }

    fn create_animation_data_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        animation_data_t_vec: &[ssab::ssab::ss::format::AnimationDataT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::AnimationData<'a>>>> {
        let animation_data_vec = animation_data_t_vec
            .iter()
            .map(|animation_data_t| {
                let mut args = ssab::ssab::ss::format::AnimationDataArgs::default();
                args.name = Some(fb.create_shared_string(&animation_data_t.name));
                args.name_hash = animation_data_t.name_hash;
                args.parts_initial_attributes = Some(self.create_shared_part_attributes_list(fb, &animation_data_t.parts_initial_attributes));
                args.parts_animation_data = Some(self.create_shared_parts_animation_data(fb, &animation_data_t.parts_animation_data));
                args.events = animation_data_t.events.as_ref().map(|events| self.create_shared_events_per_frame(fb, events));
                args.initial_events = animation_data_t.initial_events.as_ref().map(|initial_events| self.create_shared_initial_events_list(fb, initial_events));
                args.labels = animation_data_t.labels.as_ref().map(|labels| {
                    let v = labels.iter().map(|label| self.create_shared_label(fb, label)).collect::<Vec<_>>();
                    fb.create_vector(&v)
                });
                args.start_frame = animation_data_t.start_frame;
                args.end_frame = animation_data_t.end_frame;
                args.total_frame = animation_data_t.total_frame;
                args.fps = animation_data_t.fps;
                args.canvas_size = Some(self.create_shared_vec2(fb, &animation_data_t.canvas_size));
                args.pivot = Some(self.create_shared_vec2(fb, &animation_data_t.pivot));

                ssab::ssab::ss::format::AnimationData::create(fb, &args)
            })
            .collect::<Vec<_>>();

        fb.create_vector(&animation_data_vec)
    }

    fn create_shared_cell_map(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        cell_map_t_vec: &[ssab::ssab::ss::format::CellMapT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::CellMap<'a>>>> {
        let cell_map = cell_map_t_vec
            .iter()
            .map(|cell_map_t| {
                let mut args = ssab::ssab::ss::format::CellMapArgs::default();
                args.name_hash = cell_map_t.name_hash;
                args.name = Some(fb.create_shared_string(&cell_map_t.name));
                args.image_path = Some(fb.create_shared_string(&cell_map_t.image_path));
                args.image_path_hash = cell_map_t.image_path_hash;
                args.size_original = Some(self.create_shared_vec2(fb, &cell_map_t.size_original));
                args.wrap_mode = cell_map_t.wrap_mode;
                args.filter_mode = cell_map_t.filter_mode;
                args.cells = cell_map_t.cells.as_ref().map(|c| {
                    let a = c.iter().map(|cell| self.create_shared_cell(fb, cell)).collect::<Vec<_>>();
                    fb.create_vector(&a)
                });

                ssab::ssab::ss::format::CellMap::create(fb, &args)
            })
            .collect::<Vec<_>>();

        fb.create_vector(&cell_map)
    }

    fn create_shared_sound_files(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        table_data_t_vec: &[ssab::ssab::ss::format::SoundFileT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::SoundFile<'a>>>> {
        let table_data = table_data_t_vec
            .iter()
            .map(|table_data_t| {
                let mut sound_file_args = ssab::ssab::ss::format::SoundFileArgs::default();
                sound_file_args.name_hash = table_data_t.name_hash;
                sound_file_args.name = table_data_t.name.as_ref().map(|name| fb.create_shared_string(name));
                sound_file_args.file_path = table_data_t.file_path.as_ref().map(|s| fb.create_shared_string(s));
                sound_file_args.file_path_hash = table_data_t.file_path_hash;
                sound_file_args.time_total = table_data_t.time_total;

                ssab::ssab::ss::format::SoundFile::create(fb, &sound_file_args)
            })
            .collect::<Vec<_>>();

        fb.create_vector(&table_data)
    }

    fn create_shared_effect_node_behavior(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, behavior: &ssab::ssab::ss::format::EffectNodeBehaviorT) -> Option<flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset>> {
        match behavior {
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementBasic(v) => Some(v.pack(fb).as_union_value()),
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementRndSeedChange(v)
            | ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementDelay(v)
            | ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleInfiniteEmitEnabled(v) => {
                let offset = self.create_shared_int_value_table(fb, v.as_ref());
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementGravity(v)
            | ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementTransSpeed(v)
            | ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementTangentialAcceleration(v)
            | ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementAlphaFade(v) => {
                let offset = self.create_shared_vec2_value_table(fb, v.as_ref());
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementPosition(v) => {
                let offset = self.create_shared_min_max2_value_table(fb, v.as_ref());
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementRotation(v) => {
                let mut w_args = ssab::ssab::ss::format::EffectParticleElementRotationArgs::default();
                w_args.rotation_value = Some(self.create_shared_vec2(fb, &v.rotation_value));
                w_args.rotation_add_value = Some(self.create_shared_vec2(fb, &v.rotation_add_value));
                let offset = ssab::ssab::ss::format::EffectParticleElementRotation::create(fb, &w_args);
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementRotationTrans(v) => Some(v.pack(fb).as_union_value()),
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementInitColor(v) => Some(v.pack(fb).as_union_value()),
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementTransColor(v) => Some(v.pack(fb).as_union_value()),
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementSize(v) => {
                let mut w_args = ssab::ssab::ss::format::MinMax2AndVec2ValueTableArgs::default();
                w_args.size = v.size.as_ref().map(|size| self.create_shared_min_max2(fb, size));
                w_args.scale_factor_value = Some(self.create_shared_vec2(fb, &v.scale_factor_value));
                let offset = ssab::ssab::ss::format::MinMax2AndVec2ValueTable::create(fb, &w_args);
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementTransSize(v) => {
                let mut w_args = ssab::ssab::ss::format::MinMax2AndVec2ValueTableArgs::default();
                w_args.size = v.size.as_ref().map(|size| self.create_shared_min_max2(fb, size));
                w_args.scale_factor_value = Some(self.create_shared_vec2(fb, &v.scale_factor_value));
                let offset = ssab::ssab::ss::format::MinMax2AndVec2ValueTable::create(fb, &w_args);
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticlePointGravity(v) => {
                let mut w_args = ssab::ssab::ss::format::EffectParticlePointGravityArgs::default();
                w_args.position = Some(self.create_shared_vec2(fb, &v.position));
                w_args.power = v.power;
                let offset = ssab::ssab::ss::format::EffectParticlePointGravity::create(fb, &w_args);
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleTurnToDirectionEnabled(v) => {
                let offset = self.create_shared_float_value_table(fb, v.as_ref());
                Some(offset.as_union_value())
            }
            _ => None,
        }
    }

    fn create_shared_effect_nodes(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        ns: &[ssab::ssab::ss::format::EffectNodeT],
    ) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::EffectNode<'a>>>> {
        let nodes = ns
            .iter()
            .map(|node| {
                let mut n_args = ssab::ssab::ss::format::EffectNodeArgs::default();
                n_args.index = node.index;
                n_args.parent_index = node.parent_index;
                n_args.effect_node_type = node.effect_node_type;
                n_args.cell_name_hash = node.cell_name_hash;
                n_args.cell_map_name_hash = node.cell_map_name_hash;
                n_args.blend_type = node.blend_type;
                n_args.behavior = node.behavior.as_ref().map(|behavior| {
                    let v = behavior
                        .iter()
                        .map(|b| {
                            let mut w_args = ssab::ssab::ss::format::EffectNodeBehaviorWrapperArgs::default();
                            w_args.value_type = b.value.effect_node_behavior_type();
                            w_args.value = self.create_shared_effect_node_behavior(fb, &b.value);
                            ssab::ssab::ss::format::EffectNodeBehaviorWrapper::create(fb, &w_args)
                        })
                        .collect::<Vec<_>>();
                    fb.create_vector(&v)
                });
                ssab::ssab::ss::format::EffectNode::create(fb, &n_args)
            })
            .collect::<Vec<_>>();
        fb.create_vector(&nodes)
    }

    fn create_shared_effect_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        effect_list_t_vec: &[ssab::ssab::ss::format::EffectT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::Effect<'a>>>> {
        let effect_list = effect_list_t_vec
            .iter()
            .map(|effect_t| {
                let mut args = ssab::ssab::ss::format::EffectArgs::default();
                args.name_hash = effect_t.name_hash;
                args.name = Some(fb.create_shared_string(&effect_t.name));
                args.fps = effect_t.fps;
                args.is_lock_rand_seed = effect_t.is_lock_rand_seed;
                args.lock_rand_seed = effect_t.lock_rand_seed;
                args.layout_scale = Some(self.create_shared_vec2(fb, &effect_t.layout_scale));
                args.effect_node = Some(self.create_shared_effect_nodes(fb, &effect_t.effect_node));

                ssab::ssab::ss::format::Effect::create(fb, &args)
            })
            .collect::<Vec<_>>();

        fb.create_vector(&effect_list)
    }

    fn create_shared_sound_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        sound_list_t_vec: &[ssab::ssab::ss::format::SoundListT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::SoundList<'a>>>> {
        let sound_list = sound_list_t_vec
            .iter()
            .map(|sound_list_t| {
                let mut args = ssab::ssab::ss::format::SoundListArgs::default();
                args.name_hash = sound_list_t.name_hash;
                args.name = sound_list_t.name.as_ref().map(|name| fb.create_shared_string(name));
                args.table_data = sound_list_t.table_data.as_ref().map(|table_data| self.create_shared_sound_files(fb, table_data));

                ssab::ssab::ss::format::SoundList::create(fb, &args)
            })
            .collect::<Vec<_>>();

        fb.create_vector(&sound_list)
    }

    fn create_shared_font_bitmap(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        font_bitmap_t_vec: &[ssab::ssab::ss::format::FontBitmapT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::FontBitmap<'a>>>> {
        let font_bitmap = font_bitmap_t_vec
            .iter()
            .map(|font_bitmap_t| {
                let mut args = ssab::ssab::ss::format::FontBitmapArgs::default();
                args.name_hash = font_bitmap_t.name_hash;
                args.name = font_bitmap_t.name.as_ref().map(|name| fb.create_shared_string(name));
                args.name_face = font_bitmap_t.name_face.as_ref().map(|s| fb.create_shared_string(s));
                args.size = font_bitmap_t.size;
                args.padding_l = font_bitmap_t.padding_l;
                args.padding_r = font_bitmap_t.padding_r;
                args.padding_u = font_bitmap_t.padding_u;
                args.padding_d = font_bitmap_t.padding_d;
                args.base = font_bitmap_t.base;
                args.height_line = font_bitmap_t.height_line;
                args.scale = font_bitmap_t.scale.as_ref().map(|s| self.create_shared_vec2(fb, s));
                args.spacing = font_bitmap_t.spacing.as_ref().map(|s| self.create_shared_vec2(fb, s));
                args.size_texture_original = font_bitmap_t.size_texture_original.as_ref().map(|s| self.create_shared_vec2(fb, s));
                args.name_texture = font_bitmap_t.name_texture.as_ref().map(|s| {
                    let a = s.iter().map(|s| fb.create_shared_string(s)).collect::<Vec<_>>();
                    fb.create_vector(&a)
                });
                args.name_texture_hash = font_bitmap_t.name_texture_hash.as_ref().map(|h| fb.create_vector(h));
                args.table_data = font_bitmap_t.table_data.as_ref().map(|table_data| {
                    let v = table_data
                        .iter()
                        .map(|font| {
                            let mut f_args = ssab::ssab::ss::format::FontGlyphBitmapArgs::default();
                            f_args.id = font.id;
                            f_args.atlas = font.atlas.as_ref().map(|atlas| self.create_shared_region(fb, atlas));
                            f_args.offset = font.offset.as_ref().map(|offset| self.create_shared_vec2(fb, offset));
                            f_args.width_advance = font.width_advance;
                            f_args.text = font.text.as_ref().map(|text| fb.create_shared_string(text));
                            f_args.page = font.page;
                            f_args.channel = font.channel;

                            ssab::ssab::ss::format::FontGlyphBitmap::create(fb, &f_args)
                        })
                        .collect::<Vec<_>>();
                    fb.create_vector(&v)
                });

                ssab::ssab::ss::format::FontBitmap::create(fb, &args)
            })
            .collect::<Vec<_>>();

        fb.create_vector(&font_bitmap)
    }

    pub fn create_shared_anime_binary(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, ss_anime_binary_t: &ssab::ssab::ss::format::SsAnimeBinaryT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::SsAnimeBinary<'a>> {
        let mut args = ssab::ssab::ss::format::SsAnimeBinaryArgs::default();

        args.version = ss_anime_binary_t.version;
        args.name_hash = ss_anime_binary_t.name_hash;
        args.name = Some(fb.create_shared_string(&ss_anime_binary_t.name));
        args.parts = Some(self.create_shared_part_list(fb, &ss_anime_binary_t.parts));
        args.animations = Some(self.create_animation_data_list(fb, &ss_anime_binary_t.animations));
        args.keyframes = Some(self.create_keyframes(fb, &ss_anime_binary_t.keyframes));
        args.cellmaps = ss_anime_binary_t.cellmaps.as_ref().map(|cell_map| self.create_shared_cell_map(fb, cell_map));
        args.effects = ss_anime_binary_t.effects.as_ref().map(|effect| self.create_shared_effect_list(fb, effect));
        args.sound_lists = ss_anime_binary_t.sound_lists.as_ref().map(|sound_list| self.create_shared_sound_list(fb, sound_list));
        args.font_bitmaps = ss_anime_binary_t.font_bitmaps.as_ref().map(|font_bitmap| self.create_shared_font_bitmap(fb, font_bitmap));
        args.external_textures = ss_anime_binary_t.external_textures.as_ref().map(|textures| {
            let vec = textures
                .iter()
                .map(|texture| {
                    let mut args = ssab::ssab::ss::format::ExternalTextureArgs::default();
                    args.name_hash = texture.name_hash;
                    args.name = texture.name.as_ref().map(|name| fb.create_shared_string(name));
                    ssab::ssab::ss::format::ExternalTexture::create(fb, &args)
                })
                .collect::<Vec<_>>();
            fb.create_vector(&vec)
        });
        args.external_instances = ss_anime_binary_t.external_instances.as_ref().map(|external_instances| {
            let vec = external_instances
                .iter()
                .map(|entry| {
                    let mut entry_args = ssab::ssab::ss::format::ExternalInstanceArgs::default();
                    entry_args.anime_pack_name_hash = entry.anime_pack_name_hash;
                    entry_args.anime_pack_name = entry.anime_pack_name.as_ref().map(|s| fb.create_shared_string(s));
                    entry_args.anime_name_hash = entry.anime_name_hash;
                    entry_args.anime_name = entry.anime_name.as_ref().map(|s| fb.create_shared_string(s));
                    ssab::ssab::ss::format::ExternalInstance::create(fb, &entry_args)
                })
                .collect::<Vec<_>>();
            fb.create_vector(&vec)
        });
        args.embedded_assets = ss_anime_binary_t.embedded_assets.as_ref().map(|embedded_assets| {
            let vec = embedded_assets
                .iter()
                .map(|s| {
                    let mut args = ssab::ssab::ss::format::EmbeddedAssetArgs::default();
                    args.name_hash = s.name_hash;
                    args.name = s.name.as_ref().map(|name| fb.create_shared_string(name));
                    args.data = s.data.as_ref().map(|data| fb.create_vector(data));
                    ssab::ssab::ss::format::EmbeddedAsset::create(fb, &args)
                })
                .collect::<Vec<_>>();
            fb.create_vector(&vec)
        });

        ssab::ssab::ss::format::SsAnimeBinary::create(fb, &args)
    }
}
