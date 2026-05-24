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
pub struct RectKey {
    pub x1: ordered_float::OrderedFloat<f32>,
    pub y1: ordered_float::OrderedFloat<f32>,
    pub x2: ordered_float::OrderedFloat<f32>,
    pub y2: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::RectT> for RectKey {
    fn from(value: &ssab::ssab::ss::format::RectT) -> Self {
        Self { x1: ordered_float::OrderedFloat(value.x1), y1: ordered_float::OrderedFloat(value.y1), x2: ordered_float::OrderedFloat(value.x2), y2: ordered_float::OrderedFloat(value.y2) }
    }
}
impl RectKey {}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct U8RectKey {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl From<&ssab::ssab::ss::format::U8RectT> for U8RectKey {
    fn from(value: &ssab::ssab::ss::format::U8RectT) -> Self {
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
        Self { value: string_value_table_t.value.clone() }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec2ValueEntryKey {
    pub value: Option<Vec2Key>,
}
impl From<&ssab::ssab::ss::format::Vec2ValueEntryT> for Vec2ValueEntryKey {
    fn from(vec2_value_table_t: &ssab::ssab::ss::format::Vec2ValueEntryT) -> Self {
        Self { value: vec2_value_table_t.value.as_ref().map(|v| Vec2Key::from(v.as_ref())) }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RectValueEntryKey {
    pub value: Option<RectKey>,
}
impl From<&ssab::ssab::ss::format::RectValueEntryT> for RectValueEntryKey {
    fn from(rect_value_table_t: &ssab::ssab::ss::format::RectValueEntryT) -> Self {
        Self { value: rect_value_table_t.value.as_ref().map(|v| RectKey::from(v.as_ref())) }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct SsCurveKey {
    pub start_time: ordered_float::OrderedFloat<f32>,
    pub start_value: ordered_float::OrderedFloat<f32>,
    pub end_time: ordered_float::OrderedFloat<f32>,
    pub end_value: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::SsCurveT> for SsCurveKey {
    fn from(value: &ssab::ssab::ss::format::SsCurveT) -> Self {
        Self { start_time: ordered_float::OrderedFloat(value.start_time), start_value: ordered_float::OrderedFloat(value.start_value), end_time: ordered_float::OrderedFloat(value.end_time), end_value: ordered_float::OrderedFloat(value.end_value) }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataFloatValueKey {
    pub index: u16,
    pub interpolation: ssab::ssab::ss::format::InterpolationType,
    pub value: ordered_float::OrderedFloat<f32>,
    pub curve: Option<SsCurveKey>,
    pub easing_rate: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataFloatValueT> for PartAnimationDataFloatValueKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataFloatValueT) -> Self {
        Self { index: value.index, interpolation: value.interpolation, value: ordered_float::OrderedFloat(value.value), curve: value.curve.as_ref().map(|c| SsCurveKey::from(c.as_ref())), easing_rate: ordered_float::OrderedFloat(value.easing_rate) }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataFloatValueVecKey {
    pub v: Vec<PartAnimationDataFloatValueKey>,
}
impl From<&[ssab::ssab::ss::format::PartAnimationDataFloatValueT]> for PartAnimationDataFloatValueVecKey {
    fn from(value: &[ssab::ssab::ss::format::PartAnimationDataFloatValueT]) -> Self {
        Self { v: value.iter().map(PartAnimationDataFloatValueKey::from).collect() }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataIntValueKey {
    pub index: u16,
    pub interpolation: ssab::ssab::ss::format::InterpolationType,
    pub value: i32,
    pub curve: Option<SsCurveKey>,
    pub easing_rate: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataIntValueT> for PartAnimationDataIntValueKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataIntValueT) -> Self {
        Self { index: value.index, interpolation: value.interpolation, value: value.value, curve: value.curve.as_ref().map(|c| SsCurveKey::from(c.as_ref())), easing_rate: ordered_float::OrderedFloat(value.easing_rate) }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataIntValueVecKey {
    pub v: Vec<PartAnimationDataIntValueKey>,
}
impl From<&[ssab::ssab::ss::format::PartAnimationDataIntValueT]> for PartAnimationDataIntValueVecKey {
    fn from(value: &[ssab::ssab::ss::format::PartAnimationDataIntValueT]) -> Self {
        Self { v: value.iter().map(PartAnimationDataIntValueKey::from).collect() }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataUIntValueKey {
    pub index: u16,
    pub interpolation: ssab::ssab::ss::format::InterpolationType,
    pub value: u32,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataUIntValueT> for PartAnimationDataUIntValueKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataUIntValueT) -> Self {
        Self { index: value.index, interpolation: value.interpolation, value: value.value }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataUIntValueVecKey {
    pub v: Vec<PartAnimationDataUIntValueKey>,
}
impl From<&[ssab::ssab::ss::format::PartAnimationDataUIntValueT]> for PartAnimationDataUIntValueVecKey {
    fn from(value: &[ssab::ssab::ss::format::PartAnimationDataUIntValueT]) -> Self {
        Self { v: value.iter().map(PartAnimationDataUIntValueKey::from).collect() }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataBoolValueKey {
    pub index: u16,
    pub value: bool,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataBoolValueT> for PartAnimationDataBoolValueKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataBoolValueT) -> Self {
        Self { index: value.index, value: value.value }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataBoolValueVecKey {
    pub v: Vec<PartAnimationDataBoolValueKey>,
}
impl From<&[ssab::ssab::ss::format::PartAnimationDataBoolValueT]> for PartAnimationDataBoolValueVecKey {
    fn from(value: &[ssab::ssab::ss::format::PartAnimationDataBoolValueT]) -> Self {
        Self { v: value.iter().map(PartAnimationDataBoolValueKey::from).collect() }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataCellValueKey {
    pub index: u16,
    pub value: Option<PartAttributeCellKey>,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataCellValueT> for PartAnimationDataCellValueKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataCellValueT) -> Self {
        Self { index: value.index, value: value.value.as_ref().map(|v| PartAttributeCellKey::from(v.as_ref())) }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataCellValueVecKey {
    pub v: Vec<PartAnimationDataCellValueKey>,
}
impl From<&[ssab::ssab::ss::format::PartAnimationDataCellValueT]> for PartAnimationDataCellValueVecKey {
    fn from(value: &[ssab::ssab::ss::format::PartAnimationDataCellValueT]) -> Self {
        Self { v: value.iter().map(PartAnimationDataCellValueKey::from).collect() }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataPartColorKey {
    pub index: u16,
    pub interpolation: ssab::ssab::ss::format::InterpolationType,
    pub value: Option<PartAttributePartColorKey>,
    pub curve: Option<SsCurveKey>,
    pub easing_rate: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataPartColorT> for PartAnimationDataPartColorKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataPartColorT) -> Self {
        Self {
            index: value.index,
            interpolation: value.interpolation,
            value: value.value.as_ref().map(|v| PartAttributePartColorKey::from(v.as_ref())),
            curve: value.curve.as_ref().map(|c| SsCurveKey::from(c.as_ref())),
            easing_rate: ordered_float::OrderedFloat(value.easing_rate),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataPartColorVecKey {
    pub v: Vec<PartAnimationDataPartColorKey>,
}
impl From<&[ssab::ssab::ss::format::PartAnimationDataPartColorT]> for PartAnimationDataPartColorVecKey {
    fn from(value: &[ssab::ssab::ss::format::PartAnimationDataPartColorT]) -> Self {
        Self { v: value.iter().map(PartAnimationDataPartColorKey::from).collect() }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataShaderKey {
    pub index: u16,
    pub interpolation: ssab::ssab::ss::format::InterpolationType,
    pub value: Option<PartAttributeShaderKey>,
    pub curve: Option<SsCurveKey>,
    pub easing_rate: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataShaderT> for PartAnimationDataShaderKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataShaderT) -> Self {
        Self {
            index: value.index,
            interpolation: value.interpolation,
            value: value.value.as_ref().map(|v| PartAttributeShaderKey::from(v.as_ref())),
            curve: value.curve.as_ref().map(|c| SsCurveKey::from(c.as_ref())),
            easing_rate: ordered_float::OrderedFloat(value.easing_rate),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataShaderVecKey {
    pub v: Vec<PartAnimationDataShaderKey>,
}
impl From<&[ssab::ssab::ss::format::PartAnimationDataShaderT]> for PartAnimationDataShaderVecKey {
    fn from(value: &[ssab::ssab::ss::format::PartAnimationDataShaderT]) -> Self {
        Self { v: value.iter().map(PartAnimationDataShaderKey::from).collect() }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataVertexKey {
    pub index: u16,
    pub interpolation: ssab::ssab::ss::format::InterpolationType,
    pub value: Option<PartAttributeVertexKey>,
    pub curve: Option<SsCurveKey>,
    pub easing_rate: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataVertexT> for PartAnimationDataVertexKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataVertexT) -> Self {
        Self {
            index: value.index,
            interpolation: value.interpolation,
            value: value.value.as_ref().map(|v| PartAttributeVertexKey::from(v.as_ref())),
            curve: value.curve.as_ref().map(|c| SsCurveKey::from(c.as_ref())),
            easing_rate: ordered_float::OrderedFloat(value.easing_rate),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataVertexVecKey {
    pub v: Vec<PartAnimationDataVertexKey>,
}
impl From<&[ssab::ssab::ss::format::PartAnimationDataVertexT]> for PartAnimationDataVertexVecKey {
    fn from(value: &[ssab::ssab::ss::format::PartAnimationDataVertexT]) -> Self {
        Self { v: value.iter().map(PartAnimationDataVertexKey::from).collect() }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataVec2Key {
    pub index: u16,
    pub interpolation: ssab::ssab::ss::format::InterpolationType,
    pub value: Option<Vec2Key>,
    pub curve: Option<SsCurveKey>,
    pub easing_rate: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataVec2T> for PartAnimationDataVec2Key {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataVec2T) -> Self {
        Self {
            index: value.index,
            interpolation: value.interpolation,
            value: value.value.as_ref().map(|v| Vec2Key::from(v.as_ref())),
            curve: value.curve.as_ref().map(|c| SsCurveKey::from(c.as_ref())),
            easing_rate: ordered_float::OrderedFloat(value.easing_rate),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataVec2VecKey {
    pub v: Vec<PartAnimationDataVec2Key>,
}
impl From<&[ssab::ssab::ss::format::PartAnimationDataVec2T]> for PartAnimationDataVec2VecKey {
    fn from(value: &[ssab::ssab::ss::format::PartAnimationDataVec2T]) -> Self {
        Self { v: value.iter().map(PartAnimationDataVec2Key::from).collect() }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataDeformKey {
    pub index: u16,
    pub interpolation: ssab::ssab::ss::format::InterpolationType,
    pub value: Option<PartAttributeDeformKey>,
    pub curve: Option<SsCurveKey>,
    pub easing_rate: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataDeformT> for PartAnimationDataDeformKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataDeformT) -> Self {
        Self {
            index: value.index,
            interpolation: value.interpolation,
            value: value.value.as_ref().map(|v| PartAttributeDeformKey::from(v.as_ref())),
            curve: value.curve.as_ref().map(|c| SsCurveKey::from(c.as_ref())),
            easing_rate: ordered_float::OrderedFloat(value.easing_rate),
        }
    }
}

// PartAnimationDataDeform
#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataDeformVecKey {
    pub v: Vec<PartAnimationDataDeformKey>,
}
impl From<&[ssab::ssab::ss::format::PartAnimationDataDeformT]> for PartAnimationDataDeformVecKey {
    fn from(value: &[ssab::ssab::ss::format::PartAnimationDataDeformT]) -> Self {
        Self { v: value.iter().map(PartAnimationDataDeformKey::from).collect() }
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

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataKey {
    pub cell_array: Option<PartAnimationDataCellValueVecKey>,
    pub position_x_array: Option<PartAnimationDataFloatValueVecKey>,
    pub position_y_array: Option<PartAnimationDataFloatValueVecKey>,
    pub position_z_array: Option<PartAnimationDataFloatValueVecKey>,
    pub rotation_x_array: Option<PartAnimationDataFloatValueVecKey>,
    pub rotation_y_array: Option<PartAnimationDataFloatValueVecKey>,
    pub rotation_z_array: Option<PartAnimationDataFloatValueVecKey>,
    pub scale_x_array: Option<PartAnimationDataFloatValueVecKey>,
    pub scale_y_array: Option<PartAnimationDataFloatValueVecKey>,
    pub local_scale_x_array: Option<PartAnimationDataFloatValueVecKey>,
    pub local_scale_y_array: Option<PartAnimationDataFloatValueVecKey>,
    pub alpha_array: Option<PartAnimationDataFloatValueVecKey>,
    pub local_alpha_array: Option<PartAnimationDataFloatValueVecKey>,
    pub priority_array: Option<PartAnimationDataIntValueVecKey>,
    pub flip_h_array: Option<PartAnimationDataBoolValueVecKey>,
    pub flip_v_array: Option<PartAnimationDataBoolValueVecKey>,
    pub hide_array: Option<PartAnimationDataBoolValueVecKey>,
    pub part_color_array: Option<PartAnimationDataPartColorVecKey>,
    pub shader_array: Option<PartAnimationDataShaderVecKey>,
    pub vertex_array: Option<PartAnimationDataVertexVecKey>,
    pub pivot_x_array: Option<PartAnimationDataFloatValueVecKey>,
    pub pivot_y_array: Option<PartAnimationDataFloatValueVecKey>,
    pub anchor_x_array: Option<PartAnimationDataFloatValueVecKey>,
    pub anchor_y_array: Option<PartAnimationDataFloatValueVecKey>,
    pub size_x_array: Option<PartAnimationDataFloatValueVecKey>,
    pub size_y_array: Option<PartAnimationDataFloatValueVecKey>,
    pub img_flip_h_array: Option<PartAnimationDataBoolValueVecKey>,
    pub img_flip_v_array: Option<PartAnimationDataBoolValueVecKey>,
    pub uv_translation_x_array: Option<PartAnimationDataFloatValueVecKey>,
    pub uv_translation_y_array: Option<PartAnimationDataFloatValueVecKey>,
    pub uv_rotation_z_array: Option<PartAnimationDataFloatValueVecKey>,
    pub uv_scale_x_array: Option<PartAnimationDataFloatValueVecKey>,
    pub uv_scale_y_array: Option<PartAnimationDataFloatValueVecKey>,
    pub bounding_radius_array: Option<PartAnimationDataFloatValueVecKey>,
    pub mask_array: Option<PartAnimationDataFloatValueVecKey>,
    pub skew_array: Option<PartAnimationDataVec2VecKey>,
    pub deform_array: Option<PartAnimationDataDeformVecKey>,
    pub texture_array: Option<PartAnimationDataUIntValueVecKey>,
}
impl From<&ssab::ssab::ss::format::PartAnimationDataT> for PartAnimationDataKey {
    fn from(value: &ssab::ssab::ss::format::PartAnimationDataT) -> Self {
        Self {
            cell_array: value.cell_array.as_ref().map(|c| PartAnimationDataCellValueVecKey::from(c.as_ref())),
            position_x_array: value.position_x_array.as_ref().map(|p| PartAnimationDataFloatValueVecKey::from(p.as_ref())),
            position_y_array: value.position_y_array.as_ref().map(|p| PartAnimationDataFloatValueVecKey::from(p.as_ref())),
            position_z_array: value.position_z_array.as_ref().map(|p| PartAnimationDataFloatValueVecKey::from(p.as_ref())),
            rotation_x_array: value.rotation_x_array.as_ref().map(|r| PartAnimationDataFloatValueVecKey::from(r.as_ref())),
            rotation_y_array: value.rotation_y_array.as_ref().map(|r| PartAnimationDataFloatValueVecKey::from(r.as_ref())),
            rotation_z_array: value.rotation_z_array.as_ref().map(|r| PartAnimationDataFloatValueVecKey::from(r.as_ref())),
            scale_x_array: value.scale_x_array.as_ref().map(|s| PartAnimationDataFloatValueVecKey::from(s.as_ref())),
            scale_y_array: value.scale_y_array.as_ref().map(|s| PartAnimationDataFloatValueVecKey::from(s.as_ref())),
            local_scale_x_array: value.local_scale_x_array.as_ref().map(|l| PartAnimationDataFloatValueVecKey::from(l.as_ref())),
            local_scale_y_array: value.local_scale_y_array.as_ref().map(|l| PartAnimationDataFloatValueVecKey::from(l.as_ref())),
            alpha_array: value.alpha_array.as_ref().map(|a| PartAnimationDataFloatValueVecKey::from(a.as_ref())),
            local_alpha_array: value.local_alpha_array.as_ref().map(|l| PartAnimationDataFloatValueVecKey::from(l.as_ref())),
            priority_array: value.priority_array.as_ref().map(|p| PartAnimationDataIntValueVecKey::from(p.as_ref())),
            flip_h_array: value.flip_h_array.as_ref().map(|f| PartAnimationDataBoolValueVecKey::from(f.as_ref())),
            flip_v_array: value.flip_v_array.as_ref().map(|f| PartAnimationDataBoolValueVecKey::from(f.as_ref())),
            hide_array: value.hide_array.as_ref().map(|h| PartAnimationDataBoolValueVecKey::from(h.as_ref())),
            part_color_array: value.part_color_array.as_ref().map(|c| PartAnimationDataPartColorVecKey::from(c.as_ref())),
            shader_array: value.shader_array.as_ref().map(|s| PartAnimationDataShaderVecKey::from(s.as_ref())),
            vertex_array: value.vertex_array.as_ref().map(|v| PartAnimationDataVertexVecKey::from(v.as_ref())),
            pivot_x_array: value.pivot_x_array.as_ref().map(|p| PartAnimationDataFloatValueVecKey::from(p.as_ref())),
            pivot_y_array: value.pivot_y_array.as_ref().map(|p| PartAnimationDataFloatValueVecKey::from(p.as_ref())),
            anchor_x_array: value.anchor_x_array.as_ref().map(|a| PartAnimationDataFloatValueVecKey::from(a.as_ref())),
            anchor_y_array: value.anchor_y_array.as_ref().map(|a| PartAnimationDataFloatValueVecKey::from(a.as_ref())),
            size_x_array: value.size_x_array.as_ref().map(|s| PartAnimationDataFloatValueVecKey::from(s.as_ref())),
            size_y_array: value.size_y_array.as_ref().map(|s| PartAnimationDataFloatValueVecKey::from(s.as_ref())),
            img_flip_h_array: value.img_flip_h_array.as_ref().map(|i| PartAnimationDataBoolValueVecKey::from(i.as_ref())),
            img_flip_v_array: value.img_flip_v_array.as_ref().map(|i| PartAnimationDataBoolValueVecKey::from(i.as_ref())),
            uv_translation_x_array: value.uv_translation_x_array.as_ref().map(|u| PartAnimationDataFloatValueVecKey::from(u.as_ref())),
            uv_translation_y_array: value.uv_translation_y_array.as_ref().map(|u| PartAnimationDataFloatValueVecKey::from(u.as_ref())),
            uv_rotation_z_array: value.uv_rotation_z_array.as_ref().map(|u| PartAnimationDataFloatValueVecKey::from(u.as_ref())),
            uv_scale_x_array: value.uv_scale_x_array.as_ref().map(|u| PartAnimationDataFloatValueVecKey::from(u.as_ref())),
            uv_scale_y_array: value.uv_scale_y_array.as_ref().map(|u| PartAnimationDataFloatValueVecKey::from(u.as_ref())),
            bounding_radius_array: value.bounding_radius_array.as_ref().map(|b| PartAnimationDataFloatValueVecKey::from(b.as_ref())),
            mask_array: value.mask_array.as_ref().map(|m| PartAnimationDataFloatValueVecKey::from(m.as_ref())),
            skew_array: value.skew_array.as_ref().map(|s| PartAnimationDataVec2VecKey::from(s.as_ref())),
            deform_array: value.deform_array.as_ref().map(|d| PartAnimationDataDeformVecKey::from(d.as_ref())),
            texture_array: value.texture_array.as_ref().map(|t| PartAnimationDataUIntValueVecKey::from(t.as_ref())),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CellKey {
    pub name_hash: u32,
    pub name: Option<String>,
    pub rectangle: Option<RectKey>,
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
            name: value.name.as_ref().map(|n| n.to_string()),
            rectangle: value.rectangle.as_ref().map(|r| RectKey::from(r.as_ref())),
            pivot: value.pivot.as_ref().map(|p| Vec2Key::from(p.as_ref())),
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
        Self { name_hash: value.name_hash, name: value.name.as_ref().map(|n| n.to_string()), time: value.time }
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
    pub rgba: Option<U8RectKey>,
    pub rate: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT> for SsAttributePartColorKeyValueColorKey {
    fn from(value: &ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT) -> Self {
        Self { rgba: value.rgba.as_ref().map(|r| U8RectKey::from(r.as_ref())), rate: ordered_float::OrderedFloat(value.rate) }
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
            id: value.id.clone(),
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
    pub priority: ordered_float::OrderedFloat<f32>,
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
            priority: ordered_float::OrderedFloat(value.priority),
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
    pub rect: Option<RectKey>,
    pub point: Option<Vec2Key>,
    pub str: Option<String>,
}
impl From<&ssab::ssab::ss::format::PartAttributeUserDataT> for PartAttributeUserDataKey {
    fn from(value: &ssab::ssab::ss::format::PartAttributeUserDataT) -> Self {
        Self { integer: value.integer.as_ref().map(|i| IntValueEntryKey::from(i.as_ref())), rect: value.rect.as_ref().map(|r| RectKey::from(r.as_ref())), point: value.point.as_ref().map(|p| Vec2Key::from(p.as_ref())), str: value.str.clone() }
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
        Self { param_id: value.param_id.clone(), type_: value.type_, value: SignalCommandParamValueKey::from(&value.value) }
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
        Self { active: value.active, command_id: value.command_id.clone(), params: value.params.as_ref().map(|p| p.iter().map(SignalCommandParamKey::from).collect()) }
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
        Self { sound_list_name_hash: value.sound_list_name_hash, sound_name_hash: value.sound_name_hash, sound_name: value.sound_name.clone(), loop_num: value.loop_num }
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
    pub rect_cache: HashMap<RectKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::Rect<'a>>>,
    pub u8rect_cache: HashMap<U8RectKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::U8Rect<'a>>>,
    pub none_value_table_cache: HashMap<NoneValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::NoneValueEntry<'a>>>,
    pub bool_value_table_cache: HashMap<BoolValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::BoolValueEntry<'a>>>,
    pub int_value_table_cache: HashMap<IntValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::IntValueEntry<'a>>>,
    pub float_value_table_cache: HashMap<FloatValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::FloatValueEntry<'a>>>,
    pub string_value_table_cache: HashMap<StringValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::StringValueEntry<'a>>>,
    pub vec2_value_table_cache: HashMap<Vec2ValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::Vec2ValueEntry<'a>>>,
    pub rect_value_table_cache: HashMap<RectValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::RectValueEntry<'a>>>,

    pub ss_curve_cache: HashMap<SsCurveKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::SsCurve<'a>>>,
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

    pub part_animation_data_float_value_cache: HashMap<PartAnimationDataFloatValueKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>,
    pub part_animation_data_float_value_vec_cache: HashMap<PartAnimationDataFloatValueVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>>,
    pub part_animation_data_int_value_cache: HashMap<PartAnimationDataIntValueKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataIntValue<'a>>>,
    pub part_animation_data_int_value_vec_cache: HashMap<PartAnimationDataIntValueVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataIntValue<'a>>>>>,
    pub part_animation_data_uint_value_cache: HashMap<PartAnimationDataUIntValueKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataUIntValue<'a>>>,
    pub part_animation_data_uint_value_vec_cache: HashMap<PartAnimationDataUIntValueVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataUIntValue<'a>>>>>,
    pub part_animation_data_bool_value_cache: HashMap<PartAnimationDataBoolValueKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'a>>>,
    pub part_animation_data_bool_value_vec_cache: HashMap<PartAnimationDataBoolValueVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'a>>>>>,
    pub part_animation_data_cell_value_cache: HashMap<PartAnimationDataCellValueKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataCellValue<'a>>>,
    pub part_animation_data_cell_value_vec_cache: HashMap<PartAnimationDataCellValueVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataCellValue<'a>>>>>,
    pub part_animation_data_part_color_cache: HashMap<PartAnimationDataPartColorKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataPartColor<'a>>>,
    pub part_animation_data_part_color_vec_cache: HashMap<PartAnimationDataPartColorVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataPartColor<'a>>>>>,
    pub part_animation_data_shader_cache: HashMap<PartAnimationDataShaderKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataShader<'a>>>,
    pub part_animation_data_shader_vec_cache: HashMap<PartAnimationDataShaderVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataShader<'a>>>>>,
    pub part_animation_data_vertex_cache: HashMap<PartAnimationDataVertexKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataVertex<'a>>>,
    pub part_animation_data_vertex_vec_cache: HashMap<PartAnimationDataVertexVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVertex<'a>>>>>,
    pub part_animation_data_vec2_cache: HashMap<PartAnimationDataVec2Key, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataVec2<'a>>>,
    pub part_animation_data_vec2_vec_cache: HashMap<PartAnimationDataVec2VecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVec2<'a>>>>>,
    pub part_animation_data_deform_cache: HashMap<PartAnimationDataDeformKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataDeform<'a>>>,
    pub part_animation_data_deform_vec_cache: HashMap<PartAnimationDataDeformVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataDeform<'a>>>>>,
    pub part_animation_data_cache: HashMap<PartAnimationDataKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationData<'a>>>,

    pub part_mesh_binding_cache: HashMap<PartMeshBindingKey, flatbuffers::WIPOffset<ssab::ssab::ss::format::PartMeshBinding<'a>>>,
}
impl<'a> SsabCache<'a> {
    pub fn new() -> Self {
        Self {
            vec2_cache: HashMap::new(),
            rect_cache: HashMap::new(),
            none_value_table_cache: HashMap::new(),
            bool_value_table_cache: HashMap::new(),
            int_value_table_cache: HashMap::new(),
            float_value_table_cache: HashMap::new(),
            string_value_table_cache: HashMap::new(),
            vec2_value_table_cache: HashMap::new(),
            rect_value_table_cache: HashMap::new(),
            u8rect_cache: HashMap::new(),

            ss_curve_cache: HashMap::new(),
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

            part_animation_data_float_value_cache: HashMap::new(),
            part_animation_data_float_value_vec_cache: HashMap::new(),
            part_animation_data_int_value_cache: HashMap::new(),
            part_animation_data_int_value_vec_cache: HashMap::new(),
            part_animation_data_uint_value_cache: HashMap::new(),
            part_animation_data_uint_value_vec_cache: HashMap::new(),
            part_animation_data_bool_value_cache: HashMap::new(),
            part_animation_data_bool_value_vec_cache: HashMap::new(),
            part_animation_data_cell_value_cache: HashMap::new(),
            part_animation_data_cell_value_vec_cache: HashMap::new(),
            part_animation_data_part_color_cache: HashMap::new(),
            part_animation_data_part_color_vec_cache: HashMap::new(),
            part_animation_data_shader_cache: HashMap::new(),
            part_animation_data_shader_vec_cache: HashMap::new(),
            part_animation_data_vertex_cache: HashMap::new(),
            part_animation_data_vertex_vec_cache: HashMap::new(),
            part_animation_data_vec2_cache: HashMap::new(),
            part_animation_data_vec2_vec_cache: HashMap::new(),
            part_animation_data_deform_cache: HashMap::new(),
            part_animation_data_deform_vec_cache: HashMap::new(),
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

    fn create_shared_rect(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, rectt: &ssab::ssab::ss::format::RectT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::Rect<'a>> {
        let key = RectKey::from(rectt);
        if let Some(&offset) = self.rect_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::RectArgs { x1: rectt.x1, y1: rectt.y1, x2: rectt.x2, y2: rectt.y2 };
        let offset = ssab::ssab::ss::format::Rect::create(fb, &args);
        self.rect_cache.insert(key, offset);
        offset
    }

    fn create_shared_u8rect(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, u8rectt: &ssab::ssab::ss::format::U8RectT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::U8Rect<'a>> {
        let key = U8RectKey::from(u8rectt);
        if let Some(&offset) = self.u8rect_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::U8RectArgs { a: u8rectt.a, r: u8rectt.r, g: u8rectt.g, b: u8rectt.b };
        let offset = ssab::ssab::ss::format::U8Rect::create(fb, &args);
        self.u8rect_cache.insert(key, offset);
        offset
    }

    fn create_shared_ss_curve(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, curve: &ssab::ssab::ss::format::SsCurveT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::SsCurve<'a>> {
        let key = SsCurveKey::from(curve);
        if let Some(&offset) = self.ss_curve_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::SsCurveArgs { start_time: curve.start_time, start_value: curve.start_value, end_time: curve.end_time, end_value: curve.end_value };
        let offset = ssab::ssab::ss::format::SsCurve::create(fb, &args);
        self.ss_curve_cache.insert(key, offset);
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
        let args = ssab::ssab::ss::format::StringValueEntryArgs { value: string_value_table_t.value.as_ref().map(|s| fb.create_shared_string(s)) };
        let offset = ssab::ssab::ss::format::StringValueEntry::create(fb, &args);
        self.string_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_vec2_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, vec2_value_table_t: &ssab::ssab::ss::format::Vec2ValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::Vec2ValueEntry<'a>> {
        let key = Vec2ValueEntryKey::from(vec2_value_table_t);
        if let Some(&offset) = self.vec2_value_table_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::Vec2ValueEntryArgs { value: vec2_value_table_t.value.as_ref().map(|v| self.create_shared_vec2(fb, v)) };
        let offset = ssab::ssab::ss::format::Vec2ValueEntry::create(fb, &args);
        self.vec2_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_rect_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, rect_value_table_t: &ssab::ssab::ss::format::RectValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::RectValueEntry<'a>> {
        let key = RectValueEntryKey::from(rect_value_table_t);
        if let Some(&offset) = self.rect_value_table_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::RectValueEntryArgs { value: rect_value_table_t.value.as_ref().map(|v| self.create_shared_rect(fb, v)) };
        let offset = ssab::ssab::ss::format::RectValueEntry::create(fb, &args);
        self.rect_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_float_value(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_float_value_t: &ssab::ssab::ss::format::PartAnimationDataFloatValueT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>> {
        let key = PartAnimationDataFloatValueKey::from(part_animation_data_float_value_t);
        if let Some(&offset) = self.part_animation_data_float_value_cache.get(&key) {
            return offset;
        }
        let curve_off = part_animation_data_float_value_t.curve.as_ref().map(|c| self.create_shared_ss_curve(fb, c.as_ref()));
        let args = ssab::ssab::ss::format::PartAnimationDataFloatValueArgs {
            index: part_animation_data_float_value_t.index,
            interpolation: part_animation_data_float_value_t.interpolation,
            value: part_animation_data_float_value_t.value,
            curve: curve_off,
            easing_rate: part_animation_data_float_value_t.easing_rate,
        };
        let offset = ssab::ssab::ss::format::PartAnimationDataFloatValue::create(fb, &args);
        self.part_animation_data_float_value_cache.insert(key, offset);
        offset
    }

    // PartAnimationDataFloatValueVecKey
    fn create_shared_part_animation_data_float_value_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_float_value_t_array: &[ssab::ssab::ss::format::PartAnimationDataFloatValueT],
    ) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>> {
        let key = PartAnimationDataFloatValueVecKey::from(part_animation_data_float_value_t_array);
        if let Some(&offset) = self.part_animation_data_float_value_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_float_value_t_array.iter().map(|t| self.create_shared_part_animation_data_float_value(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_float_value_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_int_value(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_int_value_t: &ssab::ssab::ss::format::PartAnimationDataIntValueT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataIntValue<'a>> {
        let key = PartAnimationDataIntValueKey::from(part_animation_data_int_value_t);
        if let Some(&offset) = self.part_animation_data_int_value_cache.get(&key) {
            return offset;
        }
        let curve_off = part_animation_data_int_value_t.curve.as_ref().map(|c| self.create_shared_ss_curve(fb, c.as_ref()));
        let args = ssab::ssab::ss::format::PartAnimationDataIntValueArgs {
            index: part_animation_data_int_value_t.index,
            interpolation: part_animation_data_int_value_t.interpolation,
            value: part_animation_data_int_value_t.value,
            curve: curve_off,
            easing_rate: part_animation_data_int_value_t.easing_rate,
        };
        let offset = ssab::ssab::ss::format::PartAnimationDataIntValue::create(fb, &args);
        self.part_animation_data_int_value_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_int_value_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_int_value_t_array: &[ssab::ssab::ss::format::PartAnimationDataIntValueT],
    ) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataIntValue<'a>>>> {
        let key = PartAnimationDataIntValueVecKey::from(part_animation_data_int_value_t_array);
        if let Some(&offset) = self.part_animation_data_int_value_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_int_value_t_array.iter().map(|t| self.create_shared_part_animation_data_int_value(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_int_value_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_uint_value(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_uint_value_t: &ssab::ssab::ss::format::PartAnimationDataUIntValueT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataUIntValue<'a>> {
        let key = PartAnimationDataUIntValueKey::from(part_animation_data_uint_value_t);
        if let Some(&offset) = self.part_animation_data_uint_value_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::PartAnimationDataUIntValueArgs { index: part_animation_data_uint_value_t.index, interpolation: part_animation_data_uint_value_t.interpolation, value: part_animation_data_uint_value_t.value };
        let offset = ssab::ssab::ss::format::PartAnimationDataUIntValue::create(fb, &args);
        self.part_animation_data_uint_value_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_uint_value_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_uint_value_t_array: &[ssab::ssab::ss::format::PartAnimationDataUIntValueT],
    ) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataUIntValue<'a>>>> {
        let key = PartAnimationDataUIntValueVecKey::from(part_animation_data_uint_value_t_array);
        if let Some(&offset) = self.part_animation_data_uint_value_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_uint_value_t_array.iter().map(|t| self.create_shared_part_animation_data_uint_value(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_uint_value_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_bool_value(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_bool_value_t: &ssab::ssab::ss::format::PartAnimationDataBoolValueT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'a>> {
        let key = PartAnimationDataBoolValueKey::from(part_animation_data_bool_value_t);
        if let Some(&offset) = self.part_animation_data_bool_value_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::PartAnimationDataBoolValueArgs { index: part_animation_data_bool_value_t.index, value: part_animation_data_bool_value_t.value };
        let offset = ssab::ssab::ss::format::PartAnimationDataBoolValue::create(fb, &args);
        self.part_animation_data_bool_value_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_bool_value_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_bool_value_t_array: &[ssab::ssab::ss::format::PartAnimationDataBoolValueT],
    ) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'a>>>> {
        let key = PartAnimationDataBoolValueVecKey::from(part_animation_data_bool_value_t_array);
        if let Some(&offset) = self.part_animation_data_bool_value_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_bool_value_t_array.iter().map(|t| self.create_shared_part_animation_data_bool_value(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_bool_value_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_cell_value(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_cell_value_t: &ssab::ssab::ss::format::PartAnimationDataCellValueT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataCellValue<'a>> {
        let key = PartAnimationDataCellValueKey::from(part_animation_data_cell_value_t);
        if let Some(&offset) = self.part_animation_data_cell_value_cache.get(&key) {
            return offset;
        }
        let args = ssab::ssab::ss::format::PartAnimationDataCellValueArgs { index: part_animation_data_cell_value_t.index, value: part_animation_data_cell_value_t.value.as_ref().map(|v| self.create_shared_part_attribute_cell(fb, v)) };

        let offset = ssab::ssab::ss::format::PartAnimationDataCellValue::create(fb, &args);
        self.part_animation_data_cell_value_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_cell_value_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_cell_value_t_array: &[ssab::ssab::ss::format::PartAnimationDataCellValueT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataCellValue<'a>>>> {
        let key = PartAnimationDataCellValueVecKey::from(part_animation_data_cell_value_t_array);
        if let Some(&offset) = self.part_animation_data_cell_value_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_cell_value_t_array.iter().map(|t| self.create_shared_part_animation_data_cell_value(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_cell_value_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_part_color(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_part_color: &ssab::ssab::ss::format::PartAnimationDataPartColorT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataPartColor<'a>> {
        let key = PartAnimationDataPartColorKey::from(part_animation_data_part_color);
        if let Some(&offset) = self.part_animation_data_part_color_cache.get(&key) {
            return offset;
        }
        let curve_off = part_animation_data_part_color.curve.as_ref().map(|c| self.create_shared_ss_curve(fb, c.as_ref()));
        let args = ssab::ssab::ss::format::PartAnimationDataPartColorArgs {
            index: part_animation_data_part_color.index,
            interpolation: part_animation_data_part_color.interpolation,
            value: part_animation_data_part_color.value.as_ref().map(|v| self.create_shared_part_attribute_part_color(fb, v)),
            curve: curve_off,
            easing_rate: part_animation_data_part_color.easing_rate,
        };
        let offset = ssab::ssab::ss::format::PartAnimationDataPartColor::create(fb, &args);
        self.part_animation_data_part_color_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_part_color_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_part_color_array: &[ssab::ssab::ss::format::PartAnimationDataPartColorT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataPartColor<'a>>>> {
        let key = PartAnimationDataPartColorVecKey::from(part_animation_data_part_color_array);
        if let Some(&offset) = self.part_animation_data_part_color_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_part_color_array.iter().map(|t| self.create_shared_part_animation_data_part_color(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_part_color_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_shader(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_shader: &ssab::ssab::ss::format::PartAnimationDataShaderT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataShader<'a>> {
        let key = PartAnimationDataShaderKey::from(part_animation_data_shader);
        if let Some(&offset) = self.part_animation_data_shader_cache.get(&key) {
            return offset;
        }
        let curve_off = part_animation_data_shader.curve.as_ref().map(|c| self.create_shared_ss_curve(fb, c.as_ref()));
        let args = ssab::ssab::ss::format::PartAnimationDataShaderArgs {
            index: part_animation_data_shader.index,
            interpolation: part_animation_data_shader.interpolation,
            value: part_animation_data_shader.value.as_ref().map(|v| self.create_shared_part_attribute_shader(fb, v)),
            curve: curve_off,
            easing_rate: part_animation_data_shader.easing_rate,
        };
        let offset = ssab::ssab::ss::format::PartAnimationDataShader::create(fb, &args);
        self.part_animation_data_shader_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_shader_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_shader: &[ssab::ssab::ss::format::PartAnimationDataShaderT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataShader<'a>>>> {
        let key = PartAnimationDataShaderVecKey::from(part_animation_data_shader);
        if let Some(&offset) = self.part_animation_data_shader_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_shader.iter().map(|t| self.create_shared_part_animation_data_shader(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_shader_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_vertex(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_vertex: &ssab::ssab::ss::format::PartAnimationDataVertexT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataVertex<'a>> {
        let key = PartAnimationDataVertexKey::from(part_animation_data_vertex);
        if let Some(&offset) = self.part_animation_data_vertex_cache.get(&key) {
            return offset;
        }
        let curve_off = part_animation_data_vertex.curve.as_ref().map(|c| self.create_shared_ss_curve(fb, c.as_ref()));
        let args = ssab::ssab::ss::format::PartAnimationDataVertexArgs {
            index: part_animation_data_vertex.index,
            interpolation: part_animation_data_vertex.interpolation,
            value: part_animation_data_vertex.value.as_ref().map(|v| self.create_shared_part_attribute_vertex(fb, v)),
            curve: curve_off,
            easing_rate: part_animation_data_vertex.easing_rate,
        };
        let offset = ssab::ssab::ss::format::PartAnimationDataVertex::create(fb, &args);
        self.part_animation_data_vertex_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_vertex_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_vertex_array: &[ssab::ssab::ss::format::PartAnimationDataVertexT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVertex<'a>>>> {
        let key = PartAnimationDataVertexVecKey::from(part_animation_data_vertex_array);
        if let Some(&offset) = self.part_animation_data_vertex_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_vertex_array.iter().map(|t| self.create_shared_part_animation_data_vertex(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_vertex_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_vec2(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_vec2: &ssab::ssab::ss::format::PartAnimationDataVec2T) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataVec2<'a>> {
        let key = PartAnimationDataVec2Key::from(part_animation_data_vec2);
        if let Some(&offset) = self.part_animation_data_vec2_cache.get(&key) {
            return offset;
        }
        let curve_off = part_animation_data_vec2.curve.as_ref().map(|c| self.create_shared_ss_curve(fb, c.as_ref()));
        let args = ssab::ssab::ss::format::PartAnimationDataVec2Args {
            index: part_animation_data_vec2.index,
            interpolation: part_animation_data_vec2.interpolation,
            value: part_animation_data_vec2.value.as_ref().map(|v| self.create_shared_vec2(fb, v)),
            curve: curve_off,
            easing_rate: part_animation_data_vec2.easing_rate,
        };
        let offset = ssab::ssab::ss::format::PartAnimationDataVec2::create(fb, &args);
        self.part_animation_data_vec2_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_vec2_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_vec2_array: &[ssab::ssab::ss::format::PartAnimationDataVec2T],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVec2<'a>>>> {
        let key = PartAnimationDataVec2VecKey::from(part_animation_data_vec2_array);
        if let Some(&offset) = self.part_animation_data_vec2_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_vec2_array.iter().map(|t| self.create_shared_part_animation_data_vec2(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_vec2_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_deform(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_deform: &ssab::ssab::ss::format::PartAnimationDataDeformT,
    ) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationDataDeform<'a>> {
        let key = PartAnimationDataDeformKey::from(part_animation_data_deform);
        if let Some(&offset) = self.part_animation_data_deform_cache.get(&key) {
            return offset;
        }
        let curve_off = part_animation_data_deform.curve.as_ref().map(|c| self.create_shared_ss_curve(fb, c.as_ref()));
        let args = ssab::ssab::ss::format::PartAnimationDataDeformArgs {
            index: part_animation_data_deform.index,
            interpolation: part_animation_data_deform.interpolation,
            value: part_animation_data_deform.value.as_ref().map(|v| self.create_shared_part_attribute_deform(fb, v)),
            curve: curve_off,
            easing_rate: part_animation_data_deform.easing_rate,
        };
        let offset = ssab::ssab::ss::format::PartAnimationDataDeform::create(fb, &args);
        self.part_animation_data_deform_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_deform_list(
        &mut self,
        fb: &mut flatbuffers::FlatBufferBuilder<'a>,
        part_animation_data_deform_array: &[ssab::ssab::ss::format::PartAnimationDataDeformT],
    ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataDeform<'a>>>> {
        let key = PartAnimationDataDeformVecKey::from(part_animation_data_deform_array);
        if let Some(&offset) = self.part_animation_data_deform_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_deform_array.iter().map(|t| self.create_shared_part_animation_data_deform(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_deform_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data: &ssab::ssab::ss::format::PartAnimationDataT) -> flatbuffers::WIPOffset<ssab::ssab::ss::format::PartAnimationData<'a>> {
        let key = PartAnimationDataKey::from(part_animation_data);
        if let Some(&offset) = self.part_animation_data_cache.get(&key) {
            return offset;
        }

        let mut p_args = ssab::ssab::ss::format::PartAnimationDataArgs::default();
        p_args.cell_array = part_animation_data.cell_array.as_ref().map(|cell_array| self.create_shared_part_animation_data_cell_value_list(fb, cell_array));

        p_args.position_x_array = part_animation_data.position_x_array.as_ref().map(|position_x_array| self.create_shared_part_animation_data_float_value_list(fb, position_x_array));

        p_args.position_y_array = part_animation_data.position_y_array.as_ref().map(|position_y_array| self.create_shared_part_animation_data_float_value_list(fb, position_y_array));
        p_args.position_z_array = part_animation_data.position_z_array.as_ref().map(|position_z_array| self.create_shared_part_animation_data_float_value_list(fb, position_z_array));
        p_args.rotation_x_array = part_animation_data.rotation_x_array.as_ref().map(|rotation_x_array| self.create_shared_part_animation_data_float_value_list(fb, rotation_x_array));
        p_args.rotation_y_array = part_animation_data.rotation_y_array.as_ref().map(|rotation_y_array| self.create_shared_part_animation_data_float_value_list(fb, rotation_y_array));
        p_args.rotation_z_array = part_animation_data.rotation_z_array.as_ref().map(|rotation_z_array| self.create_shared_part_animation_data_float_value_list(fb, rotation_z_array));
        p_args.scale_x_array = part_animation_data.scale_x_array.as_ref().map(|scale_x_array| self.create_shared_part_animation_data_float_value_list(fb, scale_x_array));
        p_args.scale_y_array = part_animation_data.scale_y_array.as_ref().map(|scale_y_array| self.create_shared_part_animation_data_float_value_list(fb, scale_y_array));
        p_args.local_scale_x_array = part_animation_data.local_scale_x_array.as_ref().map(|local_scale_x_array| self.create_shared_part_animation_data_float_value_list(fb, local_scale_x_array));
        p_args.local_scale_y_array = part_animation_data.local_scale_y_array.as_ref().map(|local_scale_y_array| self.create_shared_part_animation_data_float_value_list(fb, local_scale_y_array));
        p_args.alpha_array = part_animation_data.alpha_array.as_ref().map(|alpha_array| self.create_shared_part_animation_data_float_value_list(fb, alpha_array));
        p_args.local_alpha_array = part_animation_data.local_alpha_array.as_ref().map(|local_alpha_array| self.create_shared_part_animation_data_float_value_list(fb, local_alpha_array));
        p_args.priority_array = part_animation_data.priority_array.as_ref().map(|priority_array| self.create_shared_part_animation_data_int_value_list(fb, priority_array));
        p_args.flip_h_array = part_animation_data.flip_h_array.as_ref().map(|flip_h_array| self.create_shared_part_animation_data_bool_value_list(fb, flip_h_array));
        p_args.flip_v_array = part_animation_data.flip_v_array.as_ref().map(|flip_v_array| self.create_shared_part_animation_data_bool_value_list(fb, flip_v_array));
        p_args.hide_array = part_animation_data.hide_array.as_ref().map(|hide_array| self.create_shared_part_animation_data_bool_value_list(fb, hide_array));
        p_args.part_color_array = part_animation_data.part_color_array.as_ref().map(|part_color_array| self.create_shared_part_animation_data_part_color_list(fb, part_color_array));
        p_args.shader_array = part_animation_data.shader_array.as_ref().map(|shader_array| self.create_shared_part_animation_data_shader_list(fb, shader_array));
        p_args.vertex_array = part_animation_data.vertex_array.as_ref().map(|vertex_array| self.create_shared_part_animation_data_vertex_list(fb, vertex_array));
        p_args.pivot_x_array = part_animation_data.pivot_x_array.as_ref().map(|pivot_x_array| self.create_shared_part_animation_data_float_value_list(fb, pivot_x_array));
        p_args.pivot_y_array = part_animation_data.pivot_y_array.as_ref().map(|pivot_y_array| self.create_shared_part_animation_data_float_value_list(fb, pivot_y_array));
        p_args.anchor_x_array = part_animation_data.anchor_x_array.as_ref().map(|anchor_x_array| self.create_shared_part_animation_data_float_value_list(fb, anchor_x_array));
        p_args.anchor_y_array = part_animation_data.anchor_y_array.as_ref().map(|anchor_y_array| self.create_shared_part_animation_data_float_value_list(fb, anchor_y_array));
        p_args.size_x_array = part_animation_data.size_x_array.as_ref().map(|size_x_array| self.create_shared_part_animation_data_float_value_list(fb, size_x_array));
        p_args.size_y_array = part_animation_data.size_y_array.as_ref().map(|size_y_array| self.create_shared_part_animation_data_float_value_list(fb, size_y_array));
        p_args.img_flip_h_array = part_animation_data.img_flip_h_array.as_ref().map(|img_flip_h_array| self.create_shared_part_animation_data_bool_value_list(fb, img_flip_h_array));
        p_args.img_flip_v_array = part_animation_data.img_flip_v_array.as_ref().map(|img_flip_v_array| self.create_shared_part_animation_data_bool_value_list(fb, img_flip_v_array));
        p_args.uv_translation_x_array = part_animation_data.uv_translation_x_array.as_ref().map(|uv_translation_x_array| self.create_shared_part_animation_data_float_value_list(fb, uv_translation_x_array));
        p_args.uv_translation_y_array = part_animation_data.uv_translation_y_array.as_ref().map(|uv_translation_y_array| self.create_shared_part_animation_data_float_value_list(fb, uv_translation_y_array));
        p_args.uv_rotation_z_array = part_animation_data.uv_rotation_z_array.as_ref().map(|uv_rotation_z_array| self.create_shared_part_animation_data_float_value_list(fb, uv_rotation_z_array));
        p_args.uv_scale_x_array = part_animation_data.uv_scale_x_array.as_ref().map(|uv_scale_x_array| self.create_shared_part_animation_data_float_value_list(fb, uv_scale_x_array));
        p_args.uv_scale_y_array = part_animation_data.uv_scale_y_array.as_ref().map(|uv_scale_y_array| self.create_shared_part_animation_data_float_value_list(fb, uv_scale_y_array));
        p_args.bounding_radius_array = part_animation_data.bounding_radius_array.as_ref().map(|bounding_radius_array| self.create_shared_part_animation_data_float_value_list(fb, bounding_radius_array));
        p_args.mask_array = part_animation_data.mask_array.as_ref().map(|mask_array| self.create_shared_part_animation_data_float_value_list(fb, mask_array));
        p_args.skew_array = part_animation_data.skew_array.as_ref().map(|skew_array| self.create_shared_part_animation_data_vec2_list(fb, skew_array));
        p_args.deform_array = part_animation_data.deform_array.as_ref().map(|deform_array| self.create_shared_part_animation_data_deform_list(fb, deform_array));
        p_args.texture_array = part_animation_data.texture_array.as_ref().map(|texture_array| self.create_shared_part_animation_data_uint_value_list(fb, texture_array));

        let offset = ssab::ssab::ss::format::PartAnimationData::create(fb, &p_args);
        self.part_animation_data_cache.insert(key, offset);
        offset
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
        args.name = cell.name.as_ref().map(|name| fb.create_shared_string(name));
        args.rectangle = cell.rectangle.as_ref().map(|r| self.create_shared_rect(fb, r));
        args.pivot = cell.pivot.as_ref().map(|p| self.create_shared_vec2(fb, p));
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
        args.name = label_t.name.as_ref().map(|name| fb.create_shared_string(name));
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
        args.rgba = part_attribute_cell.rgba.as_ref().map(|rgba| self.create_shared_u8rect(fb, rgba));
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
        args.id = part_attribute_shader.id.as_ref().map(|id| fb.create_shared_string(id));
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
        args.rect = part_attribute_user_data.rect.as_ref().map(|rect| self.create_shared_rect(fb, rect));
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
        args.command_id = part_attribute_signal.command_id.as_ref().map(|command_id| fb.create_shared_string(command_id));
        args.params = part_attribute_signal.params.as_ref().map(|params| {
            let v = params
                .iter()
                .map(|param| {
                    let mut p_args = ssab::ssab::ss::format::SignalCommandParamArgs::default();
                    p_args.param_id = param.param_id.as_ref().map(|param_id| fb.create_shared_string(param_id));
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
        args.sound_name = part_attribute_audio.sound_name.as_ref().map(|sound_name| fb.create_shared_string(sound_name));
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
                p_args.shape_mask = v.shape_mask;
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
                p_args.text_mask = v.text_mask;
                p_args.text_width = v.text_width;
                p_args.text_height = v.text_height;
                p_args.e_anchor = v.e_anchor;
                let offset = ssab::ssab::ss::format::PartTypeText::create(fb, &p_args);
                offset.as_union_value()
            }
            ssab::ssab::ss::format::PartTypeT::PartTypeNines(v) => v.pack(fb).as_union_value(),
            ssab::ssab::ss::format::PartTypeT::PartTypeInstance(v) => {
                let mut p_args = ssab::ssab::ss::format::PartTypeInstanceArgs::default();
                p_args.ref_anime_hash = v.ref_anime_hash;
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
                args.name = part.name.as_ref().map(|s| fb.create_shared_string(s));
                args.parent_index = part.parent_index;
                args.bounds_type = part.bounds_type;
                args.blend_type = part.blend_type;
                args.mask_influence = part.mask_influence;
                args.mask_write = part.mask_write;
                args.visible_inside_mask = part.visible_inside_mask;
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
                            u_args.value = user.value.as_ref().map(|value| self.create_shared_user_data(fb, value));
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
                            s_args.value = signal.value.as_ref().map(|value| self.create_shared_signal(fb, value));
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
                            a_args.value = audio.value.as_ref().map(|value| self.create_shared_audio(fb, value));
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
                            i_args.value = instance.value.as_ref().map(|value| self.create_shared_part_instance(fb, value));

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
                            e_args.value = effect.value.as_ref().map(|value| self.create_shared_effect(fb, value));
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
                args.parts_initial_attributes = animation_data_t.parts_initial_attributes.as_ref().map(|parts_initial_attributes| self.create_shared_part_attributes_list(fb, parts_initial_attributes));
                args.parts_animation_data = animation_data_t.parts_animation_data.as_ref().map(|parts_animation_data| self.create_shared_parts_animation_data(fb, parts_animation_data));
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
                args.canvas = animation_data_t.canvas.as_ref().map(|c| self.create_shared_rect(fb, c));

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
                args.name = cell_map_t.name.as_ref().map(|name| fb.create_shared_string(name));
                args.image_path = cell_map_t.image_path.as_ref().map(|s| fb.create_shared_string(s));
                args.image_path_hash = cell_map_t.image_path_hash;
                args.size_original = cell_map_t.size_original.as_ref().map(|s| self.create_shared_vec2(fb, s));
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
                let offset = self.create_shared_rect_value_table(fb, v.as_ref());
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementRotation(v) => {
                let mut w_args = ssab::ssab::ss::format::EffectParticleElementRotationArgs::default();
                w_args.rotation_value = v.rotation_value.as_ref().map(|rotation_value| self.create_shared_vec2(fb, rotation_value));
                w_args.rotation_add_value = v.rotation_add_value.as_ref().map(|rotation_add_value| self.create_shared_vec2(fb, rotation_add_value));
                let offset = ssab::ssab::ss::format::EffectParticleElementRotation::create(fb, &w_args);
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementRotationTrans(v) => Some(v.pack(fb).as_union_value()),
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementInitColor(v) => Some(v.pack(fb).as_union_value()),
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementTransColor(v) => Some(v.pack(fb).as_union_value()),
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementSize(v) => {
                let mut w_args = ssab::ssab::ss::format::RectAndVec2ValueTableArgs::default();
                w_args.size = v.size.as_ref().map(|size| self.create_shared_rect(fb, size));
                w_args.scale_factor_value = v.scale_factor_value.as_ref().map(|scale_factor_value| self.create_shared_vec2(fb, scale_factor_value));
                let offset = ssab::ssab::ss::format::RectAndVec2ValueTable::create(fb, &w_args);
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementTransSize(v) => {
                let mut w_args = ssab::ssab::ss::format::RectAndVec2ValueTableArgs::default();
                w_args.size = v.size.as_ref().map(|size| self.create_shared_rect(fb, size));
                w_args.scale_factor_value = v.scale_factor_value.as_ref().map(|scale_factor_value| self.create_shared_vec2(fb, scale_factor_value));
                let offset = ssab::ssab::ss::format::RectAndVec2ValueTable::create(fb, &w_args);
                Some(offset.as_union_value())
            }
            ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticlePointGravity(v) => {
                let mut w_args = ssab::ssab::ss::format::EffectParticlePointGravityArgs::default();
                w_args.position = v.position.as_ref().map(|position| self.create_shared_vec2(fb, position));
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
                args.name = effect_t.name.as_ref().map(|name| fb.create_shared_string(name));
                args.fps = effect_t.fps;
                args.is_lock_rand_seed = effect_t.is_lock_rand_seed;
                args.lock_rand_seed = effect_t.lock_rand_seed;
                args.layout_scale = effect_t.layout_scale.as_ref().map(|v| self.create_shared_vec2(fb, v));
                args.effect_node = effect_t.effect_node.as_ref().map(|nodes| self.create_shared_effect_nodes(fb, nodes));

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
                            f_args.atlas = font.atlas.as_ref().map(|atlas| self.create_shared_rect(fb, atlas));
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

        args.name_hash = ss_anime_binary_t.name_hash;
        args.name = ss_anime_binary_t.name.as_ref().map(|s| fb.create_shared_string(s));
        args.parts = ss_anime_binary_t.parts.as_ref().map(|parts| self.create_shared_part_list(fb, parts));
        args.animations = ss_anime_binary_t.animations.as_ref().map(|animations| self.create_animation_data_list(fb, animations));
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
                    entry_args.anime_name_hash = entry.anime_name_hash;
                    entry_args.anime_pack_name_hash = entry.anime_pack_name_hash;
                    entry_args.anime_pack_name = entry.anime_pack_name.as_ref().map(|s| fb.create_shared_string(s));
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
