use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec2Key {
    pub v1: ordered_float::OrderedFloat<f32>,
    pub v2: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::Vec2T> for Vec2Key {
    fn from(v: &ssab::ssab::Vec2T) -> Self {
        Self {
            v1: ordered_float::OrderedFloat(v.v1),
            v2: ordered_float::OrderedFloat(v.v2),
        }
    }
}
impl Vec2Key {
    fn new(x: f32, y: f32) -> Self {
        Self {
            v1: ordered_float::OrderedFloat(x),
            v2: ordered_float::OrderedFloat(y),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec3Key {
    pub v1: ordered_float::OrderedFloat<f32>,
    pub v2: ordered_float::OrderedFloat<f32>,
    pub v3: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::Vec3T> for Vec3Key {
    fn from(v: &ssab::ssab::Vec3T) -> Self {
        Self {
            v1: ordered_float::OrderedFloat(v.v1),
            v2: ordered_float::OrderedFloat(v.v2),
            v3: ordered_float::OrderedFloat(v.v3),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RectKey {
    pub x1: ordered_float::OrderedFloat<f32>,
    pub y1: ordered_float::OrderedFloat<f32>,
    pub x2: ordered_float::OrderedFloat<f32>,
    pub y2: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::RectT> for RectKey {
    fn from(value: &ssab::ssab::RectT) -> Self {
        Self {
            x1: ordered_float::OrderedFloat(value.x1),
            y1: ordered_float::OrderedFloat(value.y1),
            x2: ordered_float::OrderedFloat(value.x2),
            y2: ordered_float::OrderedFloat(value.y2),
        }
    }
}
impl RectKey {
    fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            x1: ordered_float::OrderedFloat(x1),
            y1: ordered_float::OrderedFloat(y1),
            x2: ordered_float::OrderedFloat(x2),
            y2: ordered_float::OrderedFloat(y2),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct U8RectKey {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl From<&ssab::ssab::U8RectT> for U8RectKey {
    fn from(value: &ssab::ssab::U8RectT) -> Self {
        Self {
            a: value.a,
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct IntValueEntryKey {
    pub value: i32,
}
impl From<&ssab::ssab::IntValueEntryT> for IntValueEntryKey {
    fn from(int_value_table_t: &ssab::ssab::IntValueEntryT) -> Self {
        Self {
            value: int_value_table_t.value,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct FloatValueEntryKey {
    pub value: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::FloatValueEntryT> for FloatValueEntryKey {
    fn from(float_value_table_t: &ssab::ssab::FloatValueEntryT) -> Self {
        Self {
            value: ordered_float::OrderedFloat(float_value_table_t.value),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec2ValueEntryKey {
    pub value: Vec2Key,
}
impl From<&ssab::ssab::Vec2ValueEntryT> for Vec2ValueEntryKey {
    fn from(vec2_value_table_t: &ssab::ssab::Vec2ValueEntryT) -> Self {
        let value = vec2_value_table_t.value.clone().unwrap();
        let v1 = value.v1;
        let v2 = value.v2;
        Self {
            value: Vec2Key::new(v1, v2),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct RectValueEntryKey {
    pub value: RectKey,
}
impl From<&ssab::ssab::RectValueEntryT> for RectValueEntryKey {
    fn from(rect_value_table_t: &ssab::ssab::RectValueEntryT) -> Self {
        let value = rect_value_table_t.value.as_ref().unwrap();
        Self {
            value: RectKey::new(value.x1, value.y1, value.x2, value.y2),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataFloatValueKey {
    pub index: u16,
    pub interpolation: ssab::ssab::InterpolationType,
    pub value: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::PartAnimationDataFloatValueT> for PartAnimationDataFloatValueKey {
    fn from(value: &ssab::ssab::PartAnimationDataFloatValueT) -> Self {
        Self {
            index: value.index,
            interpolation: ssab::ssab::InterpolationType::from(value.interpolation),
            value: ordered_float::OrderedFloat(value.value),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataFloatValueVecKey {
    pub v: Vec<PartAnimationDataFloatValueKey>,
}
impl From<&[ssab::ssab::PartAnimationDataFloatValueT]> for PartAnimationDataFloatValueVecKey {
    fn from(value: &[ssab::ssab::PartAnimationDataFloatValueT]) -> Self {
        Self {
            v: value.iter().map(|v| PartAnimationDataFloatValueKey::from(v)).collect(),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataIntValueKey {
    pub index: u16,
    pub interpolation: ssab::ssab::InterpolationType,
    pub value: i32,
}
impl From<&ssab::ssab::PartAnimationDataIntValueT> for PartAnimationDataIntValueKey {
    fn from(value: &ssab::ssab::PartAnimationDataIntValueT) -> Self {
        Self {
            index: value.index,
            interpolation: ssab::ssab::InterpolationType::from(value.interpolation),
            value: value.value,
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataIntValueVecKey {
    pub v: Vec<PartAnimationDataIntValueKey>,
}
impl From<&[ssab::ssab::PartAnimationDataIntValueT]> for PartAnimationDataIntValueVecKey {
    fn from(value: &[ssab::ssab::PartAnimationDataIntValueT]) -> Self {
        Self {
            v: value.iter().map(|v| PartAnimationDataIntValueKey::from(v)).collect(),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataBoolValueKey {
    pub index: u16,
    pub value: bool,
}
impl From<&ssab::ssab::PartAnimationDataBoolValueT> for PartAnimationDataBoolValueKey {
    fn from(value: &ssab::ssab::PartAnimationDataBoolValueT) -> Self {
        Self {
            index: value.index,
            value: value.value,
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataBoolValueVecKey {
    pub v: Vec<PartAnimationDataBoolValueKey>,
}
impl From<&[ssab::ssab::PartAnimationDataBoolValueT]> for PartAnimationDataBoolValueVecKey {
    fn from(value: &[ssab::ssab::PartAnimationDataBoolValueT]) -> Self {
        Self {
            v: value.iter().map(|v| PartAnimationDataBoolValueKey::from(v)).collect(),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataCellValueKey {
    pub index: u16,
    pub value: Option<PartAttributeCellKey>,
}
impl From<&ssab::ssab::PartAnimationDataCellValueT> for PartAnimationDataCellValueKey {
    fn from(value: &ssab::ssab::PartAnimationDataCellValueT) -> Self {
        Self {
            index: value.index,
            value: value.value.as_ref().map(|v| PartAttributeCellKey::from(v.as_ref())),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataCellValueVecKey {
    pub v: Vec<PartAnimationDataCellValueKey>,
}
impl From<&[ssab::ssab::PartAnimationDataCellValueT]> for PartAnimationDataCellValueVecKey {
    fn from(value: &[ssab::ssab::PartAnimationDataCellValueT]) -> Self {
        Self {
            v: value.iter().map(|v| PartAnimationDataCellValueKey::from(v)).collect(),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataPartColorKey {
    pub index: u16,
    pub interpolation: ssab::ssab::InterpolationType,
    pub value: Option<PartAttributePartColorKey>,
}
impl From<&ssab::ssab::PartAnimationDataPartColorT> for PartAnimationDataPartColorKey {
    fn from(value: &ssab::ssab::PartAnimationDataPartColorT) -> Self {
        Self {
            index: value.index,
            interpolation: ssab::ssab::InterpolationType::from(value.interpolation),
            value: value.value.as_ref().map(|v| PartAttributePartColorKey::from(v.as_ref())),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAnimationDataPartColorVecKey {
    pub v: Vec<PartAnimationDataPartColorKey>,
}
impl From<&[ssab::ssab::PartAnimationDataPartColorT]> for PartAnimationDataPartColorVecKey {
    fn from(value: &[ssab::ssab::PartAnimationDataPartColorT]) -> Self {
        Self {
            v: value.iter().map(|v| PartAnimationDataPartColorKey::from(v)).collect(),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CellKey {
    pub name: String,
    pub rectangle: Option<RectKey>,
    pub pivot: Option<Vec2Key>,
    pub rotated: bool,
    pub table_coordinate: Option<Vec<Vec2Key>>,
    pub table_index_vertex: Option<Vec<u16>>,
}
impl From<&ssab::ssab::CellT> for CellKey {
    fn from(value: &ssab::ssab::CellT) -> Self {
        Self {
            name: value.name.clone(),
            rectangle: value.rectangle.as_ref().map(|r| RectKey::from(r.as_ref())),
            pivot: value.pivot.as_ref().map(|p| Vec2Key::from(p.as_ref())),
            rotated: value.rotated,
            table_coordinate: value.table_coordinate.as_ref().map(|t| t.iter().map(|v| Vec2Key::from(v)).collect::<Vec<Vec2Key>>()),
            table_index_vertex: value.table_index_vertex.as_ref().map(|t| t.iter().map(|v| *v).collect::<Vec<u16>>()),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct LabelKey {
    pub name: String,
    pub time: i32,
}
impl From<&ssab::ssab::LabelT> for LabelKey {
    fn from(value: &ssab::ssab::LabelT) -> Self {
        Self {
            name: value.name.clone(),
            time: value.time,
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeCellKey {
    pub map_id: i32,
    pub name: Option<String>,
}
impl From<&ssab::ssab::PartAttributeCellT> for PartAttributeCellKey {
    fn from(value: &ssab::ssab::PartAttributeCellT) -> Self {
        Self {
            map_id: value.map_id,
            name: value.name.clone(),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct SsAttributePartColorKeyValueColorKey {
    pub rgba: Option<U8RectKey>,
    pub rate: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::SsAttributePartColorKeyValueColorT> for SsAttributePartColorKeyValueColorKey {
    fn from(value: &ssab::ssab::SsAttributePartColorKeyValueColorT) -> Self {
        Self {
            rgba: value.rgba.as_ref().map(|r| U8RectKey::from(r.as_ref())),
            rate: ordered_float::OrderedFloat(value.rate),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributePartColorKey {
    pub blend_type: ssab::ssab::BlendType,
    pub lt: Option<SsAttributePartColorKeyValueColorKey>,
    pub rt: Option<SsAttributePartColorKeyValueColorKey>,
    pub lb: Option<SsAttributePartColorKeyValueColorKey>,
    pub rb: Option<SsAttributePartColorKeyValueColorKey>,
}
impl From<&ssab::ssab::PartAttributePartColorT> for PartAttributePartColorKey {
    fn from(value: &ssab::ssab::PartAttributePartColorT) -> Self {
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
    pub param0: ordered_float::OrderedFloat<f32>,
    pub param1: ordered_float::OrderedFloat<f32>,
    pub param2: ordered_float::OrderedFloat<f32>,
    pub param3: ordered_float::OrderedFloat<f32>,
    pub param4: ordered_float::OrderedFloat<f32>,
    pub param5: ordered_float::OrderedFloat<f32>,
    pub param6: ordered_float::OrderedFloat<f32>,
    pub param7: ordered_float::OrderedFloat<f32>,
    pub map0_id: i32,
    pub map0_name: Option<String>,
    pub map1_id: i32,
    pub map1_name: Option<String>,
}
impl From<&ssab::ssab::PartAttributeShaderT> for PartAttributeShaderKey {
    fn from(value: &ssab::ssab::PartAttributeShaderT) -> Self {
        Self {
            id: value.id.clone(),
            param0: ordered_float::OrderedFloat(value.param0),
            param1: ordered_float::OrderedFloat(value.param1),
            param2: ordered_float::OrderedFloat(value.param2),
            param3: ordered_float::OrderedFloat(value.param3),
            param4: ordered_float::OrderedFloat(value.param4),
            param5: ordered_float::OrderedFloat(value.param5),
            param6: ordered_float::OrderedFloat(value.param6),
            param7: ordered_float::OrderedFloat(value.param7),
            map0_id: value.map0_id,
            map0_name: value.map0_name.clone(),
            map1_id: value.map1_id,
            map1_name: value.map1_name.clone(),
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
impl From<&ssab::ssab::PartAttributeVertexT> for PartAttributeVertexKey {
    fn from(value: &ssab::ssab::PartAttributeVertexT) -> Self {
        Self {
            lt: value.lt.as_ref().map(|v| Vec2Key::from(v.as_ref())),
            rt: value.rt.as_ref().map(|v| Vec2Key::from(v.as_ref())),
            lb: value.lb.as_ref().map(|v| Vec2Key::from(v.as_ref())),
            rb: value.rb.as_ref().map(|v| Vec2Key::from(v.as_ref())),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeDeformKey {
    pub coordinate: Option<Vec<Vec2Key>>,
    pub index: Option<Vec<i16>>,
}
impl From<&ssab::ssab::PartAttributeDeformT> for PartAttributeDeformKey {
    fn from(value: &ssab::ssab::PartAttributeDeformT) -> Self {
        Self {
            coordinate: value.coordinate.as_ref().map(|c| c.iter().map(|v| Vec2Key::from(v)).collect()),
            index: value.index.as_ref().map(|i| i.iter().map(|v| *v as i16).collect()),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributesKey {
    pub cell: Option<PartAttributeCellKey>,
    pub position: Option<Vec3Key>,
    pub rotation: Option<Vec3Key>,
    pub scale: Option<Vec2Key>,
    pub local_scale: Option<Vec2Key>,
    pub opacity: ordered_float::OrderedFloat<f32>,
    pub localopacity: ordered_float::OrderedFloat<f32>,
    pub priority: i16,
    pub flip_v: bool,
    pub flip_h: bool,
    pub hide: bool,
    pub parts_color: Option<PartAttributePartColorKey>,
    pub shader: Option<PartAttributeShaderKey>,
    pub vertex: Option<PartAttributeVertexKey>,
    pub pivot: Option<Vec2Key>,
    pub size: Option<Vec2Key>,
    pub uv_move: Option<Vec3Key>,
    pub uv_scale: Option<Vec2Key>,
    pub bounding_radius: ordered_float::OrderedFloat<f32>,
    pub mask: ordered_float::OrderedFloat<f32>,
    pub skew: Option<Vec2Key>,
    pub deform: Option<PartAttributeDeformKey>,
    pub texture: i32,
}
impl From<&ssab::ssab::PartAttributesT> for PartAttributesKey {
    fn from(value: &ssab::ssab::PartAttributesT) -> Self {
        Self {
            cell: value.cell.as_ref().map(|c| PartAttributeCellKey::from(c.as_ref())),
            position: value.position.as_ref().map(|p| Vec3Key::from(p.as_ref())),
            rotation: value.rotation.as_ref().map(|r| Vec3Key::from(r.as_ref())),
            scale: value.scale.as_ref().map(|s| Vec2Key::from(s.as_ref())),
            local_scale: value.local_scale.as_ref().map(|s| Vec2Key::from(s.as_ref())),
            opacity: ordered_float::OrderedFloat(value.opacity),
            localopacity: ordered_float::OrderedFloat(value.localopacity),
            priority: value.priority,
            flip_v: value.flip_v,
            flip_h: value.flip_h,
            hide: value.hide,
            parts_color: value.parts_color.as_ref().map(|c| PartAttributePartColorKey::from(c.as_ref())),
            shader: value.shader.as_ref().map(|s| PartAttributeShaderKey::from(s.as_ref())),
            vertex: value.vertex.as_ref().map(|v| PartAttributeVertexKey::from(v.as_ref())),
            pivot: value.pivot.as_ref().map(|p| Vec2Key::from(p.as_ref())),
            size: value.size.as_ref().map(|s| Vec2Key::from(s.as_ref())),
            uv_move: value.uv_move.as_ref().map(|u| Vec3Key::from(u.as_ref())),
            uv_scale: value.uv_scale.as_ref().map(|s| Vec2Key::from(s.as_ref())),
            bounding_radius: ordered_float::OrderedFloat(value.bounding_radius),
            mask: ordered_float::OrderedFloat(value.mask),
            skew: value.skew.as_ref().map(|s| Vec2Key::from(s.as_ref())),
            deform: value.deform.as_ref().map(|d| PartAttributeDeformKey::from(d.as_ref())),
            texture: value.texture,
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributesVecKey {
    v: Vec<PartAttributesKey>,
}
impl From<&Vec<ssab::ssab::PartAttributesT>> for PartAttributesVecKey {
    fn from(value: &Vec<ssab::ssab::PartAttributesT>) -> Self {
        Self {
            v: value.iter().map(|v| PartAttributesKey::from(v)).collect(),
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartAttributeInstanceKey {
    pub pingpong: bool,
    pub independent: bool,
    pub loop_num: i32,
    pub start_label: Option<String>,
    pub start_offset: i32,
    pub end_label: Option<String>,
    pub end_offset: i32,
    pub speed: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::PartAttributeInstanceT> for PartAttributeInstanceKey {
    fn from(value: &ssab::ssab::PartAttributeInstanceT) -> Self {
        Self {
            pingpong: value.pingpong,
            independent: value.independent,
            loop_num: value.loop_num,
            start_label: value.start_label.clone(),
            start_offset: value.start_offset,
            end_label: value.end_label.clone(),
            end_offset: value.end_offset,
            speed: ordered_float::OrderedFloat(value.speed),
        }
    }
}


pub struct SsabCache<'a> {
    pub vec2_cache: HashMap<Vec2Key, flatbuffers::WIPOffset<ssab::ssab::Vec2<'a>>>,
    pub vec3_cache: HashMap<Vec3Key, flatbuffers::WIPOffset<ssab::ssab::Vec3<'a>>>,
    pub rect_cache: HashMap<RectKey, flatbuffers::WIPOffset<ssab::ssab::Rect<'a>>>,
    pub u8rect_cache: HashMap<U8RectKey, flatbuffers::WIPOffset<ssab::ssab::U8Rect<'a>>>,
    pub int_value_table_cache: HashMap<IntValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::IntValueEntry<'a>>>,
    pub float_value_table_cache: HashMap<FloatValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::FloatValueEntry<'a>>>,
    pub vec2_value_table_cache: HashMap<Vec2ValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::Vec2ValueEntry<'a>>>,
    pub rect_value_table_cache: HashMap<RectValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::RectValueEntry<'a>>>,

    pub cell_cache: HashMap<CellKey, flatbuffers::WIPOffset<ssab::ssab::Cell<'a>>>,
    pub label_cache: HashMap<LabelKey, flatbuffers::WIPOffset<ssab::ssab::Label<'a>>>,
    pub part_attribute_cell_cache: HashMap<PartAttributeCellKey, flatbuffers::WIPOffset<ssab::ssab::PartAttributeCell<'a>>>,
    pub attribute_part_color_key_value_color_cache: HashMap<SsAttributePartColorKeyValueColorKey, flatbuffers::WIPOffset<ssab::ssab::SsAttributePartColorKeyValueColor<'a>>>,
    pub part_attribute_part_color_cache: HashMap<PartAttributePartColorKey, flatbuffers::WIPOffset<ssab::ssab::PartAttributePartColor<'a>>>,
    pub part_attribute_shader_cache: HashMap<PartAttributeShaderKey, flatbuffers::WIPOffset<ssab::ssab::PartAttributeShader<'a>>>,
    pub part_attribute_vertex_cache: HashMap<PartAttributeVertexKey, flatbuffers::WIPOffset<ssab::ssab::PartAttributeVertex<'a>>>,
    pub part_attribute_deform_cache: HashMap<PartAttributeDeformKey, flatbuffers::WIPOffset<ssab::ssab::PartAttributeDeform<'a>>>,
    pub part_attributes_cache: HashMap<PartAttributesKey, flatbuffers::WIPOffset<ssab::ssab::PartAttributes<'a>>>,
    pub part_attributes_vec_cache: HashMap<PartAttributesVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::PartAttributes<'a>>>>>,
    pub part_attribute_instance_cache: HashMap<PartAttributeInstanceKey, flatbuffers::WIPOffset<ssab::ssab::PartAttributeInstance<'a>>>,

    pub part_type_none_cache: HashMap<String, flatbuffers::WIPOffset<ssab::ssab::PartTypeNone<'a>>>,

    pub part_animation_data_float_value_cache: HashMap<PartAnimationDataFloatValueKey, flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataFloatValue<'a>>>,
    pub part_animation_data_float_value_vec_cache: HashMap<PartAnimationDataFloatValueVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationDataFloatValue<'a>>>>>,
    pub part_animation_data_int_value_cache: HashMap<PartAnimationDataIntValueKey, flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataIntValue<'a>>>,
    pub part_animation_data_int_value_vec_cache: HashMap<PartAnimationDataIntValueVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationDataIntValue<'a>>>>>,
    pub part_animation_data_bool_value_cache: HashMap<PartAnimationDataBoolValueKey, flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataBoolValue<'a>>>,
    pub part_animation_data_bool_value_vec_cache: HashMap<PartAnimationDataBoolValueVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationDataBoolValue<'a>>>>>,
    pub part_animation_data_cell_value_cache: HashMap<PartAnimationDataCellValueKey, flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataCellValue<'a>>>,
    pub part_animation_data_cell_value_vec_cache: HashMap<PartAnimationDataCellValueVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationDataCellValue<'a>>>>>,
    pub part_animation_data_part_color_cache: HashMap<PartAnimationDataPartColorKey, flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataPartColor<'a>>>,
    pub part_animation_data_part_color_vec_cache: HashMap<PartAnimationDataPartColorVecKey, flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationDataPartColor<'a>>>>>,
}
impl<'a> SsabCache<'a> {
    pub fn new() -> Self {
        Self {
            vec2_cache: HashMap::new(),
            vec3_cache: HashMap::new(),
            rect_cache: HashMap::new(),
            int_value_table_cache: HashMap::new(),
            float_value_table_cache: HashMap::new(),
            vec2_value_table_cache: HashMap::new(),
            rect_value_table_cache: HashMap::new(),
            u8rect_cache: HashMap::new(),

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

            part_type_none_cache: HashMap::new(),

            part_animation_data_float_value_cache: HashMap::new(),
            part_animation_data_float_value_vec_cache: HashMap::new(),
            part_animation_data_int_value_cache: HashMap::new(),
            part_animation_data_int_value_vec_cache: HashMap::new(),
            part_animation_data_bool_value_cache: HashMap::new(),
            part_animation_data_bool_value_vec_cache: HashMap::new(),
            part_animation_data_cell_value_cache: HashMap::new(),
            part_animation_data_cell_value_vec_cache: HashMap::new(),
            part_animation_data_part_color_cache: HashMap::new(),
            part_animation_data_part_color_vec_cache: HashMap::new(),
        }
    }
    fn create_shared_vec2(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, vec2t: &ssab::ssab::Vec2T) -> flatbuffers::WIPOffset<ssab::ssab::Vec2<'a>> {
        let key = Vec2Key::from(vec2t);
        if let Some(&offset) = self.vec2_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::Vec2Args::default();
        args.v1 = vec2t.v1;
        args.v2 = vec2t.v2;
        let offset = ssab::ssab::Vec2::create(fb, &args);
        self.vec2_cache.insert(key, offset);
        offset
    }

    fn create_shared_vec3(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, vec3t: &ssab::ssab::Vec3T) -> flatbuffers::WIPOffset<ssab::ssab::Vec3<'a>> {
        let key = Vec3Key::from(vec3t);
        if let Some(&offset) = self.vec3_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::Vec3Args::default();
        args.v1 = vec3t.v1;
        args.v2 = vec3t.v2;
        args.v3 = vec3t.v3;
        let offset = ssab::ssab::Vec3::create(fb, &args);
        self.vec3_cache.insert(key, offset);
        offset
    }

    fn create_shared_rect(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, rectt: &ssab::ssab::RectT) -> flatbuffers::WIPOffset<ssab::ssab::Rect<'a>> {
        let key = RectKey::from(rectt);
        if let Some(&offset) = self.rect_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::RectArgs::default();
        args.x1 = rectt.x1;
        args.y1 = rectt.y1;
        args.x2 = rectt.x2;
        args.y2 = rectt.y2;
        let offset = ssab::ssab::Rect::create(fb, &args);
        self.rect_cache.insert(key, offset);
        offset
    }

    fn create_shared_u8rect(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, u8rectt: &ssab::ssab::U8RectT) -> flatbuffers::WIPOffset<ssab::ssab::U8Rect<'a>> {
        let key = U8RectKey::from(u8rectt);
        if let Some(&offset) = self.u8rect_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::U8RectArgs::default();
        args.a = u8rectt.a;
        args.r = u8rectt.r;
        args.g = u8rectt.g;
        args.b = u8rectt.b;
        let offset = ssab::ssab::U8Rect::create(fb, &args);
        self.u8rect_cache.insert(key, offset);
        offset
    }

    fn create_shared_int_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, int_value_table_t: &ssab::ssab::IntValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::IntValueEntry<'a>> {
        let key = IntValueEntryKey::from(int_value_table_t);
        if let Some(&offset) = self.int_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::IntValueEntryArgs::default();
        args.value = int_value_table_t.value;
        let offset = ssab::ssab::IntValueEntry::create(fb, &args);
        self.int_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_float_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, float_value_table_t: &ssab::ssab::FloatValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::FloatValueEntry<'a>> {
        let key = FloatValueEntryKey::from(float_value_table_t);
        if let Some(&offset) = self.float_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::FloatValueEntryArgs::default();
        args.value = float_value_table_t.value;
        let offset = ssab::ssab::FloatValueEntry::create(fb, &args);
        self.float_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_vec2_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, vec2_value_table_t: &ssab::ssab::Vec2ValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::Vec2ValueEntry<'a>> {
        let key = Vec2ValueEntryKey::from(vec2_value_table_t);
        if let Some(&offset) = self.vec2_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::Vec2ValueEntryArgs::default();
        let v = vec2_value_table_t.value.as_ref().unwrap();
        args.value = Some(self.create_shared_vec2(fb, &v));
        let offset = ssab::ssab::Vec2ValueEntry::create(fb, &args);
        self.vec2_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_rect_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, rect_value_table_t: &ssab::ssab::RectValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::RectValueEntry<'a>> {
        let key = RectValueEntryKey::from(rect_value_table_t);
        if let Some(&offset) = self.rect_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::RectValueEntryArgs::default();
        let v = rect_value_table_t.value.as_ref().unwrap();
        args.value = Some(self.create_shared_rect(fb, &v));
        let offset = ssab::ssab::RectValueEntry::create(fb, &args);
        self.rect_value_table_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_float_value(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_float_value_t: &ssab::ssab::PartAnimationDataFloatValueT) -> flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataFloatValue<'a>> {
        let key = PartAnimationDataFloatValueKey::from(part_animation_data_float_value_t);
        if let Some(&offset) = self.part_animation_data_float_value_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::PartAnimationDataFloatValueArgs::default();
        args.index = part_animation_data_float_value_t.index;
        args.value = part_animation_data_float_value_t.value;
        let offset = ssab::ssab::PartAnimationDataFloatValue::create(fb, &args);
        self.part_animation_data_float_value_cache.insert(key, offset);
        offset
    }

    // PartAnimationDataFloatValueVecKey
    fn create_shared_part_animation_data_float_value_vec(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_float_value_t_array: &[ssab::ssab::PartAnimationDataFloatValueT]) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationDataFloatValue<'a>>>> {
        let key = PartAnimationDataFloatValueVecKey::from(part_animation_data_float_value_t_array);
        if let Some(&offset) = self.part_animation_data_float_value_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_float_value_t_array.iter().map(|t| self.create_shared_part_animation_data_float_value(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_float_value_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_int_value(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_int_value_t: &ssab::ssab::PartAnimationDataIntValueT) -> flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataIntValue<'a>> {
        let key = PartAnimationDataIntValueKey::from(part_animation_data_int_value_t);
        if let Some(&offset) = self.part_animation_data_int_value_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::PartAnimationDataIntValueArgs::default();
        args.index = part_animation_data_int_value_t.index;
        args.value = part_animation_data_int_value_t.value;
        let offset = ssab::ssab::PartAnimationDataIntValue::create(fb, &args);
        self.part_animation_data_int_value_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_int_value_vec(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_int_value_t_array: &[ssab::ssab::PartAnimationDataIntValueT]) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationDataIntValue<'a>>>> {
        let key = PartAnimationDataIntValueVecKey::from(part_animation_data_int_value_t_array);
        if let Some(&offset) = self.part_animation_data_int_value_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_int_value_t_array.iter().map(|t| self.create_shared_part_animation_data_int_value(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_int_value_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_bool_value(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_bool_value_t: &ssab::ssab::PartAnimationDataBoolValueT) -> flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataBoolValue<'a>> {
        let key = PartAnimationDataBoolValueKey::from(part_animation_data_bool_value_t);
        if let Some(&offset) = self.part_animation_data_bool_value_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::PartAnimationDataBoolValueArgs::default();
        args.index = part_animation_data_bool_value_t.index;
        args.value = part_animation_data_bool_value_t.value;
        let offset = ssab::ssab::PartAnimationDataBoolValue::create(fb, &args);
        self.part_animation_data_bool_value_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_bool_value_vec(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_bool_value_t_array: &[ssab::ssab::PartAnimationDataBoolValueT]) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationDataBoolValue<'a>>>> {
        let key = PartAnimationDataBoolValueVecKey::from(part_animation_data_bool_value_t_array);
        if let Some(&offset) = self.part_animation_data_bool_value_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_bool_value_t_array.iter().map(|t| self.create_shared_part_animation_data_bool_value(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_bool_value_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_cell_value(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_cell_value_t: &ssab::ssab::PartAnimationDataCellValueT) -> flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataCellValue<'a>> {
        let key = PartAnimationDataCellValueKey::from(part_animation_data_cell_value_t);
        if let Some(&offset) = self.part_animation_data_cell_value_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::PartAnimationDataCellValueArgs::default();
        args.index = part_animation_data_cell_value_t.index;
        args.value = part_animation_data_cell_value_t.value.as_ref().map(|v| self.create_shared_part_attribute_cell(fb, v));

        let offset = ssab::ssab::PartAnimationDataCellValue::create(fb, &args);
        self.part_animation_data_cell_value_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_cell_value_vec(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_cell_value_t_array: &[ssab::ssab::PartAnimationDataCellValueT]) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationDataCellValue<'a>>>> {
        let key = PartAnimationDataCellValueVecKey::from(part_animation_data_cell_value_t_array);
        if let Some(&offset) = self.part_animation_data_cell_value_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_cell_value_t_array.iter().map(|t| self.create_shared_part_animation_data_cell_value(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_cell_value_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_animation_data_part_color(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_part_color: &ssab::ssab::PartAnimationDataPartColorT) -> flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataPartColor<'a>> {
        let key = PartAnimationDataPartColorKey::from(part_animation_data_part_color);
        if let Some(&offset) = self.part_animation_data_part_color_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::PartAnimationDataPartColorArgs::default();
        args.index = part_animation_data_part_color.index;
        args.interpolation = part_animation_data_part_color.interpolation;
        args.value = part_animation_data_part_color.value.as_ref().map(|v| self.create_shared_part_attribute_part_color(fb, v));
        let offset = ssab::ssab::PartAnimationDataPartColor::create(fb, &args);
        self.part_animation_data_part_color_cache.insert(key, offset);
        offset
    }
    
    fn create_shared_part_animation_data_part_color_vec(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_animation_data_part_color_array: &[ssab::ssab::PartAnimationDataPartColorT]) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationDataPartColor<'a>>>> {
        let key = PartAnimationDataPartColorVecKey::from(part_animation_data_part_color_array);
        if let Some(&offset) = self.part_animation_data_part_color_vec_cache.get(&key) {
            return offset;
        }
        let v = part_animation_data_part_color_array.iter().map(|t| self.create_shared_part_animation_data_part_color(fb, t)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_animation_data_part_color_vec_cache.insert(key, offset);
        offset
    }
}

impl<'a> SsabCache<'a> {
    fn create_shared_cell(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, cell: &ssab::ssab::CellT) -> flatbuffers::WIPOffset<ssab::ssab::Cell<'a>> {
        let key = CellKey::from(cell);
        if let Some(&offset) = self.cell_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::CellArgs::default();
        args.name = Some(fb.create_shared_string(&cell.name));
        args.rectangle = cell.rectangle.as_ref().map(|r| self.create_shared_rect(fb, r));
        args.pivot = cell.pivot.as_ref().map(|p| self.create_shared_vec2(fb, p));
        args.rotated = cell.rotated;
        args.table_coordinate = cell.table_coordinate.as_ref().map(|p| {
            let vec = p.iter().map(|v| self.create_shared_vec2(fb, v)).collect::<Vec<_>>();
            fb.create_vector(&vec)
        });
        args.table_index_vertex = cell.table_index_vertex.as_ref().map(|p| fb.create_vector(&p));

        let offset = ssab::ssab::Cell::create(fb, &args);
        self.cell_cache.insert(key, offset);
        offset
    }

    fn create_shared_label(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, label_t: &ssab::ssab::LabelT) -> flatbuffers::WIPOffset<ssab::ssab::Label<'a>> {
        let key = LabelKey::from(label_t);
        if let Some(&offset) = self.label_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::LabelArgs::default();
        args.name = Some(fb.create_shared_string(&label_t.name));
        args.time = label_t.time;

        let offset = ssab::ssab::Label::create(fb, &args);
        self.label_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attribute_cell(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_cell: &ssab::ssab::PartAttributeCellT) -> flatbuffers::WIPOffset<ssab::ssab::PartAttributeCell<'a>> {
        let key =  PartAttributeCellKey::from(part_attribute_cell);
        if let Some(&offset) = self.part_attribute_cell_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::PartAttributeCellArgs::default();
        args.map_id = part_attribute_cell.map_id;
        args.name = part_attribute_cell.name.as_ref().map(|s| fb.create_shared_string(s));

        let offset = ssab::ssab::PartAttributeCell::create(fb, &args);
        self.part_attribute_cell_cache.insert(key, offset);
        offset
    }

    fn create_shared_attribute_part_color_key_value_color(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_cell: &ssab::ssab::SsAttributePartColorKeyValueColorT) -> flatbuffers::WIPOffset<ssab::ssab::SsAttributePartColorKeyValueColor<'a>> {
        let key =  SsAttributePartColorKeyValueColorKey::from(part_attribute_cell);
        if let Some(&offset) = self.attribute_part_color_key_value_color_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::SsAttributePartColorKeyValueColorArgs::default();
        args.rgba = part_attribute_cell.rgba.as_ref().map(|rgba| self.create_shared_u8rect(fb, rgba));
        args.rate = part_attribute_cell.rate;

        let offset = ssab::ssab::SsAttributePartColorKeyValueColor::create(fb, &args);
        self.attribute_part_color_key_value_color_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attribute_part_color(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_part_color: &ssab::ssab::PartAttributePartColorT) -> flatbuffers::WIPOffset<ssab::ssab::PartAttributePartColor<'a>> {
        let key =  PartAttributePartColorKey::from(part_attribute_part_color);
        if let Some(&offset) = self.part_attribute_part_color_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::PartAttributePartColorArgs::default();
        args.blend_type = part_attribute_part_color.blend_type;
        args.lt = part_attribute_part_color.lt.as_ref().map(|lt| self.create_shared_attribute_part_color_key_value_color(fb, lt));
        args.rt = part_attribute_part_color.rt.as_ref().map(|rt| self.create_shared_attribute_part_color_key_value_color(fb, rt));
        args.lb = part_attribute_part_color.lb.as_ref().map(|lb| self.create_shared_attribute_part_color_key_value_color(fb, lb));
        args.rb = part_attribute_part_color.rb.as_ref().map(|rb| self.create_shared_attribute_part_color_key_value_color(fb, rb));

        let offset = ssab::ssab::PartAttributePartColor::create(fb, &args);
        self.part_attribute_part_color_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attribute_shader(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_shader: &ssab::ssab::PartAttributeShaderT) -> flatbuffers::WIPOffset<ssab::ssab::PartAttributeShader<'a>> {
        let key =  PartAttributeShaderKey::from(part_attribute_shader);
        if let Some(&offset) = self.part_attribute_shader_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::PartAttributeShaderArgs::default();
        args.id = part_attribute_shader.id.as_ref().map(|id| fb.create_shared_string(&id));
        args.param0 = part_attribute_shader.param0;
        args.param1 = part_attribute_shader.param1;
        args.param2 = part_attribute_shader.param2;
        args.param3 = part_attribute_shader.param3;
        args.param4 = part_attribute_shader.param4;
        args.param5 = part_attribute_shader.param5;
        args.param6 = part_attribute_shader.param6;
        args.param7 = part_attribute_shader.param7;
        args.map0_id = part_attribute_shader.map0_id;
        args.map0_name = part_attribute_shader.map0_name.as_ref().map(|s| fb.create_shared_string(s));
        args.map1_id = part_attribute_shader.map1_id;
        args.map1_name = part_attribute_shader.map1_name.as_ref().map(|s| fb.create_shared_string(s));

        let offset = ssab::ssab::PartAttributeShader::create(fb, &args);
        self.part_attribute_shader_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attribute_vertex(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_vertex: &ssab::ssab::PartAttributeVertexT) -> flatbuffers::WIPOffset<ssab::ssab::PartAttributeVertex<'a>> {
        let key =  PartAttributeVertexKey::from(part_attribute_vertex);
        if let Some(&offset) = self.part_attribute_vertex_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::PartAttributeVertexArgs::default();
        args.lt = part_attribute_vertex.lt.as_ref().map(|lt| self.create_shared_vec2(fb, lt));
        args.rt = part_attribute_vertex.rt.as_ref().map(|rt| self.create_shared_vec2(fb, rt));
        args.lb = part_attribute_vertex.lb.as_ref().map(|lb| self.create_shared_vec2(fb, lb));
        args.rb = part_attribute_vertex.rb.as_ref().map(|rb| self.create_shared_vec2(fb, rb));

        let offset = ssab::ssab::PartAttributeVertex::create(fb, &args);
        self.part_attribute_vertex_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attribute_deform(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_deform: &ssab::ssab::PartAttributeDeformT) -> flatbuffers::WIPOffset<ssab::ssab::PartAttributeDeform<'a>> {
        let key =  PartAttributeDeformKey::from(part_attribute_deform);
        if let Some(&offset) = self.part_attribute_deform_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::PartAttributeDeformArgs::default();
        args.coordinate = part_attribute_deform.coordinate.as_ref().map(|p| {
            let v = p.iter().map(|v| self.create_shared_vec2(fb, v)).collect::<Vec<_>>();
            fb.create_vector(&v)
        });
        args.index = part_attribute_deform.index.as_ref().map(|p| fb.create_vector(&p));

        let offset = ssab::ssab::PartAttributeDeform::create(fb, &args);
        self.part_attribute_deform_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attributes(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attributes: &ssab::ssab::PartAttributesT) -> flatbuffers::WIPOffset<ssab::ssab::PartAttributes<'a>> {
        let key =  PartAttributesKey::from(part_attributes);
        if let Some(&offset) = self.part_attributes_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::PartAttributesArgs::default();
        args.cell = part_attributes.cell.as_ref().map(|cell| { self.create_shared_part_attribute_cell(fb, cell) });
        args.position = part_attributes.position.as_ref().map(|position| self.create_shared_vec3(fb, &position));
        args.rotation = part_attributes.rotation.as_ref().map(|rotation| self.create_shared_vec3(fb, &rotation));
        args.scale = part_attributes.scale.as_ref().map(|scale| self.create_shared_vec2(fb, &scale));
        args.local_scale = part_attributes.local_scale.as_ref().map(|local_scale|self.create_shared_vec2(fb, &local_scale));
        args.opacity = part_attributes.opacity;
        args.localopacity = part_attributes.localopacity;
        args.priority = part_attributes.priority;
        args.flip_v = part_attributes.flip_v;
        args.flip_h = part_attributes.flip_h;
        args.hide = part_attributes.hide;
        args.parts_color = part_attributes.parts_color.as_ref().map(|parts_color| { self.create_shared_part_attribute_part_color(fb, parts_color) });
        args.shader = part_attributes.shader.as_ref().map(|shader| { self.create_shared_part_attribute_shader(fb, shader) });
        args.vertex = part_attributes.vertex.as_ref().map(|vertex| { self.create_shared_part_attribute_vertex(fb, vertex) });
        args.pivot = part_attributes.pivot.as_ref().map(|pivot| self.create_shared_vec2(fb, &pivot));
        args.size = part_attributes.size.as_ref().map(|size| self.create_shared_vec2(fb, &size));
        args.uv_move = part_attributes.uv_move.as_ref().map(|uv_move| self.create_shared_vec3(fb, &uv_move));
        args.uv_scale = part_attributes.uv_scale.as_ref().map(|uv_scale| self.create_shared_vec2(fb, &uv_scale));
        args.bounding_radius = part_attributes.bounding_radius;
        args.mask = part_attributes.mask;
        args.skew = part_attributes.skew.as_ref().map(|skew| self.create_shared_vec2(fb, &skew));
        args.deform = part_attributes.deform.as_ref().map(|deform| { self.create_shared_part_attribute_deform(fb, deform) });
        args.texture = part_attributes.texture;
        let offset = ssab::ssab::PartAttributes::create(fb, &args);

        self.part_attributes_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_attributes_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attributes_vec: &Vec<ssab::ssab::PartAttributesT>) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::PartAttributes<'a>>>> {
        let key = PartAttributesVecKey::from(part_attributes_vec);
        if let Some(&offset) = self.part_attributes_vec_cache.get(&key) {
            return offset;
        }

        let v = part_attributes_vec.iter().map(|part_attributes| self.create_shared_part_attributes(fb, part_attributes)).collect::<Vec<_>>();
        let offset = fb.create_vector(&v);
        self.part_attributes_vec_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_instance(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_attribute_instance: &ssab::ssab::PartAttributeInstanceT) -> flatbuffers::WIPOffset<ssab::ssab::PartAttributeInstance<'a>> {
        let key =  PartAttributeInstanceKey::from(part_attribute_instance);
        if let Some(&offset) = self.part_attribute_instance_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::PartAttributeInstanceArgs::default();
        args.pingpong = part_attribute_instance.pingpong;
        args.independent = part_attribute_instance.independent;
        args.loop_num = part_attribute_instance.loop_num;
        args.start_label = part_attribute_instance.start_label.as_ref().map(|start_label| fb.create_shared_string(&start_label));
        args.start_offset = part_attribute_instance.start_offset;
        args.end_label = part_attribute_instance.end_label.as_ref().map(|end_label| fb.create_shared_string(&end_label));
        args.end_offset = part_attribute_instance.end_offset;
        args.speed = part_attribute_instance.speed;
        let offset = ssab::ssab::PartAttributeInstance::create(fb, &args);

        self.part_attribute_instance_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_type(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_type: &ssab::ssab::PartTypeT) -> flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset> {
        match part_type {
            ssab::ssab::PartTypeT::PartTypeShape(v) => {
                /*
                let mut p_args = ssab::ssab::PartTypeShapeArgs::default();
                p_args.shape_type = v.shape_type;
                p_args.shape_mask = v.shape_mask;
                let offset = ssab::ssab::PartTypeShape::create(fb, &p_args);
                offset.as_union_value()
                 */
                v.pack(fb).as_union_value()
            },
            ssab::ssab::PartTypeT::PartTypeText(v) => {
                let mut p_args = ssab::ssab::PartTypeTextArgs::default();
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
                let offset = ssab::ssab::PartTypeText::create(fb, &p_args);
                offset.as_union_value()
            },
            ssab::ssab::PartTypeT::PartTypeNines(v) => {
                v.pack(fb).as_union_value()
            },
            ssab::ssab::PartTypeT::PartTypeInstance(v) => {
                let mut p_args = ssab::ssab::PartTypeInstanceArgs::default();
                p_args.ref_anime = v.ref_anime.as_ref().map(|s| fb.create_shared_string(s));
                let offset = ssab::ssab::PartTypeInstance::create(fb, &p_args);
                offset.as_union_value()
            },
            ssab::ssab::PartTypeT::PartTypeEffect(v) => {
                let mut p_args = ssab::ssab::PartTypeEffectArgs::default();
                p_args.ref_effect_name = v.ref_effect_name.as_ref().map(|s| fb.create_shared_string(s));
                let offset = ssab::ssab::PartTypeEffect::create(fb, &p_args);
                offset.as_union_value()
            },
            _ => {
                if let Some(&offset) = self.part_type_none_cache.get("dummy") {
                    return offset.as_union_value()
                };
                let p_args = ssab::ssab::PartTypeNoneArgs::default();
                let offset = ssab::ssab::PartTypeNone::create(fb, &p_args);
                self.part_type_none_cache.insert("dummy".to_string(), offset);
                offset.as_union_value()
            },
        }
    }

    fn create_shared_part_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, ps: &[ssab::ssab::PartDataT]) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::PartData<'a>>>> {
        let parts = ps.iter().map(|part| {
            let mut args = ssab::ssab::PartDataArgs::default();
            args.name = part.name.as_ref().map(|s| fb.create_shared_string(s));
            args.parent_index = part.parent_index;
            args.bounds_type = part.bounds_type;
            args.blend_type = part.blend_type;
            args.part_type_type = part.part_type.part_type_type();
            args.part_type = Some(self.create_shared_part_type(fb, &part.part_type));

            ssab::ssab::PartData::create(fb, &args)
        }).collect::<Vec<_>>();
        fb.create_vector(&parts)
    }

    fn create_shared_parts_animation_data(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, pa: &[ssab::ssab::PartAnimationDataT]) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationData<'a>>>> {
        let part_animation_data = pa.iter().map(|part_animation_data| {
            let mut p_args = ssab::ssab::PartAnimationDataArgs::default();
            p_args.cell_array = part_animation_data.cell_array.as_ref().map(|cell_array| {
                self.create_shared_part_animation_data_cell_value_vec(fb, cell_array)
            });

            p_args.pos_x_array = part_animation_data.pos_x_array.as_ref().map(|pos_x_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, pos_x_array)
            });

            p_args.pos_y_array = part_animation_data.pos_y_array.as_ref().map(|pos_y_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, pos_y_array)
            });
            p_args.pos_z_array = part_animation_data.pos_z_array.as_ref().map(|pos_z_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, pos_z_array)
            });
            p_args.rot_x_array = part_animation_data.rot_x_array.as_ref().map(|rot_x_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, rot_x_array)
            });
            p_args.rot_y_array = part_animation_data.rot_y_array.as_ref().map(|rot_y_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, rot_y_array)
            });
            p_args.rot_z_array = part_animation_data.rot_z_array.as_ref().map(|rot_z_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, rot_z_array)
            });
            p_args.scl_x_array = part_animation_data.scl_x_array.as_ref().map(|scl_x_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, scl_x_array)
            });
            p_args.scl_y_array = part_animation_data.scl_y_array.as_ref().map(|scl_y_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, scl_y_array)
            });
            p_args.loscl_x_array = part_animation_data.loscl_x_array.as_ref().map(|loscl_x_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, loscl_x_array)
            });
            p_args.loscl_y_array = part_animation_data.loscl_y_array.as_ref().map(|loscl_y_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, loscl_y_array)
            });
            p_args.alpha_array = part_animation_data.alpha_array.as_ref().map(|alpha_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, alpha_array)
            });
            p_args.lo_alpha_array = part_animation_data.lo_alpha_array.as_ref().map(|lo_alpha_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, lo_alpha_array)
            });
            p_args.prio_array = part_animation_data.prio_array.as_ref().map(|prio_array| {
                self.create_shared_part_animation_data_int_value_vec(fb, prio_array)
            });
            p_args.flip_h_array = part_animation_data.flip_h_array.as_ref().map(|flip_h_array| {
                self.create_shared_part_animation_data_bool_value_vec(fb, flip_h_array)
            });
            p_args.flip_v_array = part_animation_data.flip_v_array.as_ref().map(|flip_v_array| {
                self.create_shared_part_animation_data_bool_value_vec(fb, flip_v_array)
            });
            p_args.hide_array = part_animation_data.hide_array.as_ref().map(|hide_array| {
                self.create_shared_part_animation_data_bool_value_vec(fb, hide_array)
            });
            p_args.parts_color_array = part_animation_data.parts_color_array.as_ref().map(|parts_color_array| {
                self.create_shared_part_animation_data_part_color_vec(fb, parts_color_array)
            });
            p_args.shader_array = part_animation_data.shader_array.as_ref().map(|shader_array| {
                let v = shader_array.iter().map(|shader| {
                    let mut s_srgs = ssab::ssab::PartAnimationDataShaderArgs::default();
                    s_srgs.index = shader.index;
                    s_srgs.value = shader.value.as_ref().map(|value| { self.create_shared_part_attribute_shader(fb, value) });

                    ssab::ssab::PartAnimationDataShader::create(fb, &s_srgs)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.vertex_array = part_animation_data.vertex_array.as_ref().map(|vertex_array| {
                let v = vertex_array.iter().map(|vertex| {
                    let mut v_args = ssab::ssab::PartAnimationDataVertexArgs::default();
                    v_args.index = vertex.index;
                    v_args.interpolation = vertex.interpolation;
                    v_args.value = vertex.value.as_ref().map(|value| { self.create_shared_part_attribute_vertex(fb, value) });
                    ssab::ssab::PartAnimationDataVertex::create(fb, &v_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.pivot_x_array = part_animation_data.pivot_x_array.as_ref().map(|pivot_x_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, pivot_x_array)
            });
            p_args.pivot_y_array = part_animation_data.pivot_y_array.as_ref().map(|pivot_y_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, pivot_y_array)
            });
            p_args.anchor_x_array = part_animation_data.anchor_x_array.as_ref().map(|anchor_x_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, anchor_x_array)
            });
            p_args.anchor_y_array = part_animation_data.anchor_y_array.as_ref().map(|anchor_y_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, anchor_y_array)
            });
            p_args.size_x_array = part_animation_data.size_x_array.as_ref().map(|size_x_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, size_x_array)
            });
            p_args.size_y_array = part_animation_data.size_y_array.as_ref().map(|size_y_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, size_y_array)
            });
            p_args.img_flip_h_array = part_animation_data.img_flip_h_array.as_ref().map(|img_flip_h_array| {
                self.create_shared_part_animation_data_bool_value_vec(fb, img_flip_h_array)
            });
            p_args.img_flip_v_array = part_animation_data.img_flip_v_array.as_ref().map(|img_flip_v_array| {
                self.create_shared_part_animation_data_bool_value_vec(fb, img_flip_v_array)
            });
            p_args.uvt_x_array = part_animation_data.uvt_x_array.as_ref().map(|uvt_x_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, uvt_x_array)
            });
            p_args.uvt_y_array = part_animation_data.uvt_y_array.as_ref().map(|uvt_y_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, uvt_y_array)
            });
            p_args.uvr_z_array = part_animation_data.uvr_z_array.as_ref().map(|uvr_z_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, uvr_z_array)
            });
            p_args.uvs_x_array = part_animation_data.uvs_x_array.as_ref().map(|uvs_x_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, uvs_x_array)
            });
            p_args.uvs_y_array = part_animation_data.uvs_y_array.as_ref().map(|uvs_y_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, uvs_y_array)
            });
            p_args.boundr_array = part_animation_data.boundr_array.as_ref().map(|boundr_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, boundr_array)
            });
            p_args.mask_array = part_animation_data.mask_array.as_ref().map(|mask_array| {
                self.create_shared_part_animation_data_float_value_vec(fb, mask_array)
            });
            p_args.skew_array = part_animation_data.skew_array.as_ref().map(|skew_array| {
                let v = skew_array.iter().map(|skew| {
                    let mut s_args = ssab::ssab::PartAnimationDataVec2Args::default();
                    s_args.index = skew.index;
                    s_args.interpolation = skew.interpolation;
                    s_args.value = skew.value.as_ref().map(|value| self.create_shared_vec2(fb, value));

                    ssab::ssab::PartAnimationDataVec2::create(fb, &s_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.deform_array = part_animation_data.deform_array.as_ref().map(|deform_array| {
                let v = deform_array.iter().map(|deform| {
                    let mut d_args = ssab::ssab::PartAnimationDataDeformArgs::default();
                    d_args.index = deform.index;
                    d_args.interpolation = deform.interpolation;
                    d_args.value = deform.value.as_ref().map(|value| {
                        self.create_shared_part_attribute_deform(fb, value)
                    });

                    ssab::ssab::PartAnimationDataDeform::create(fb, &d_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.texture_array = part_animation_data.texture_array.as_ref().map(|texture_array| {
                self.create_shared_part_animation_data_int_value_vec(fb, texture_array)
            });

            ssab::ssab::PartAnimationData::create(fb, &p_args)
        }).collect::<Vec<_>>();
        fb.create_vector(&part_animation_data)
    }

    fn create_shared_parts_mesh_data(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, pm: &[ssab::ssab::PartMeshDataT]) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::PartMeshData<'a>>>> {
        let part_mesh_data = pm.iter().map(|part| {
            let mut p_args = ssab::ssab::PartMeshDataArgs::default();
            p_args.part_index = part.part_index;
            p_args.uv = part.uv.as_ref().map(|uv| fb.create_vector(uv));
            p_args.indices = part.indices.as_ref().map(|indices| fb.create_vector(indices));
            p_args.count_coordinate_mesh = part.count_coordinate_mesh;
            p_args.index_coordinate_deform = part.index_coordinate_deform.as_ref().map(|index_coordinate_deform| fb.create_vector(index_coordinate_deform));

            ssab::ssab::PartMeshData::create(fb, &p_args)
        }).collect::<Vec<_>>();
        fb.create_vector(&part_mesh_data)
    }

    fn create_shared_events_per_frame(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, es: &[ssab::ssab::EventsPerFrameT]) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::EventsPerFrame<'a>>>> {
        let v = es.iter().map(|event| {
            let mut e_args = ssab::ssab::EventsPerFrameArgs::default();
            e_args.frame_index = event.frame_index;
            e_args.users = event.users.as_ref().map(|users| {
                let v = users.iter().map(|user| {
                    let mut u_args = ssab::ssab::EventUserDataArgs::default();
                    u_args.part_index = user.part_index;
                    u_args.value = user.value.as_ref().map(|value| {
                        let mut u = ssab::ssab::PartAttributeUserDataArgs::default();
                        u.integer = value.integer.as_ref().map(|integer| self.create_shared_int_value_table(fb, &integer));
                        u.rect = value.rect.as_ref().map(|rect| self.create_shared_rect(fb, &rect));
                        u.point = value.point.as_ref().map(|point| self.create_shared_vec2(fb, &point));
                        u.str = value.str.as_ref().map(|s| fb.create_shared_string(s));

                        ssab::ssab::PartAttributeUserData::create(fb, &u)
                    });
                    ssab::ssab::EventUserData::create(fb, &u_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });

            e_args.signals = event.signals.as_ref().map(|signals| {
               let v = signals.iter().map(|signal| {
                   let mut s_args = ssab::ssab::EventSignalArgs::default();
                   s_args.part_index = signal.part_index;
                   s_args.value = signal.value.as_ref().map(|value| {
                       let mut s = ssab::ssab::PartAttributeSignalArgs::default();
                       s.active = value.active;
                       s.command_id = value.command_id.as_ref().map(|s| fb.create_shared_string(s));
                       s.params = value.params.as_ref().map(|params| {
                           let v = params.iter().map(|param| {
                               let mut p_args = ssab::ssab::SignalCommandParamArgs::default();
                               p_args.param_id = param.param_id.as_ref().map(|param_id| fb.create_shared_string(param_id));
                               p_args.type_ = param.type_;
                               p_args.value = param.value;

                               ssab::ssab::SignalCommandParam::create(fb, &p_args)
                           }).collect::<Vec<_>>();
                           fb.create_vector(&v)
                       });

                       ssab::ssab::PartAttributeSignal::create(fb, &s)
                   });

                   ssab::ssab::EventSignal::create(fb, &s_args)
               }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            e_args.audios = event.audios.as_ref().map(|audios| {
                let v = audios.iter().map(|audio| {
                    let mut a_args = ssab::ssab::EventAudioArgs::default();
                    a_args.part_index = audio.part_index;
                    a_args.value = audio.value.as_ref().map(|value| {
                        let mut pa_args = ssab::ssab::PartAttributeAudioArgs::default();
                        pa_args.sound_list_id = value.sound_list_id;
                        pa_args.sound_name = value.sound_name.as_ref().map(|sound_name| fb.create_shared_string(sound_name));
                        pa_args.loop_num = value.loop_num;

                        ssab::ssab::PartAttributeAudio::create(fb, &pa_args)
                    });
                    ssab::ssab::EventAudio::create(fb, &a_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            e_args.instances = event.instances.as_ref().map(|instances| {
                let v = instances.iter().map(|instance| {
                    let mut i_args = ssab::ssab::EventInstanceArgs::default();
                    i_args.part_index = instance.part_index;
                    i_args.value = instance.value.as_ref().map(|value| {
                        self.create_shared_part_instance(fb, value)
                    });

                    ssab::ssab::EventInstance::create(fb, &i_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            e_args.effects = event.effects.as_ref().map(|effects| {
                let v = effects.iter().map(|effect| {
                    let mut e_args = ssab::ssab::EventEffectArgs::default();
                    e_args.part_index = effect.part_index;
                    e_args.value = effect.value.as_ref().map(|value| {
                        let mut pe_args = ssab::ssab::PartAttributeEffectArgs::default();
                        pe_args.cur_keyframe = value.cur_keyframe;
                        pe_args.start_time = value.start_time;
                        pe_args.speed = value.speed;
                        pe_args.loopflag = value.loopflag;
                        ssab::ssab::PartAttributeEffect::create(fb, &pe_args)
                    });

                    ssab::ssab::EventEffect::create(fb, &e_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });

            ssab::ssab::EventsPerFrame::create(fb, &e_args)
        }).collect::<Vec<_>>();
        fb.create_vector(&v)
    }

    fn create_animation_data_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, animation_data_t_vec: &[ssab::ssab::AnimationDataT]) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::AnimationData<'a>>>> {
        let animation_data_vec = animation_data_t_vec.iter().map(|animation_data_t| {
            let mut args = ssab::ssab::AnimationDataArgs::default();
            args.name = Some(fb.create_shared_string(&animation_data_t.name));
            args.parts_initial_attributes = animation_data_t.parts_initial_attributes.as_ref().map(|parts_initial_attributes| self.create_shared_part_attributes_list(fb, parts_initial_attributes));
            args.parts_animation_data = animation_data_t.parts_animation_data.as_ref().map(|parts_animation_data| self.create_shared_parts_animation_data(fb, parts_animation_data));
            args.parts_mesh_data = animation_data_t.parts_mesh_data.as_ref().map(|parts_mesh_data| self.create_shared_parts_mesh_data(fb, parts_mesh_data));
            args.events = animation_data_t.events.as_ref().map(|events| { self.create_shared_events_per_frame(fb, events) });
            args.labels = animation_data_t.labels.as_ref().map(|labels| {
                let v = labels.iter().map(|label| {
                    self.create_shared_label(fb, label)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            args.start_frame = animation_data_t.start_frame;
            args.end_frame = animation_data_t.end_frame;
            args.total_frame = animation_data_t.total_frame;
            args.fps = animation_data_t.fps;
            args.canvas = animation_data_t.canvas.as_ref().map(|c| self.create_shared_rect(fb, c));

            ssab::ssab::AnimationData::create(fb, &args)
        }).collect::<Vec<_>>();

        fb.create_vector(&animation_data_vec)
    }

    fn create_shared_cell_map(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, cell_map_t_vec: &[ssab::ssab::CellMapT], ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::CellMap<'a>>>> {
        let cell_map = cell_map_t_vec.iter().map(|cell_map_t| {
            let mut args = ssab::ssab::CellMapArgs::default();
            args.name = Some(fb.create_shared_string(&cell_map_t.name));
            args.image_path = cell_map_t.image_path.as_ref().map(|s| fb.create_shared_string(s));
            args.size_original = cell_map_t.size_original.as_ref().map(|s| self.create_shared_vec2(fb, s));
            args.wrap_mode = cell_map_t.wrap_mode;
            args.filter_mode = cell_map_t.filter_mode;
            args.cells = cell_map_t.cells.as_ref().map(|c| {
                let a = c.iter().map(|cell| self.create_shared_cell(fb, cell)).collect::<Vec<_>>();
                fb.create_vector(&a)
            });

            ssab::ssab::CellMap::create(fb, &args)
        }).collect::<Vec<_>>();

        fb.create_vector(&cell_map)
    }

    fn create_shared_sound_files(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, table_data_t_vec: &[ssab::ssab::SoundFileT], ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::SoundFile<'a>>>> {
        let table_data = table_data_t_vec.iter().map(|table_data_t| {
            let mut sound_file_args = ssab::ssab::SoundFileArgs::default();
            sound_file_args.name = Some(fb.create_shared_string(&table_data_t.name));
            sound_file_args.time_total = table_data_t.time_total;

            ssab::ssab::SoundFile::create(fb, &sound_file_args)
        }).collect::<Vec<_>>();

        fb.create_vector(&table_data)
    }

    fn create_shared_effect_node_behavior(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, behavior: &ssab::ssab::EffectNodeBehaviorT) -> Option<flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset>> {
        match behavior {
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementBasic(v) => {
                Some(v.pack(fb).as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementRndSeedChange(v) |
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementDelay(v) |
            ssab::ssab::EffectNodeBehaviorT::EffectParticleInfiniteEmitEnabled(v) => {
                let offset = self.create_shared_int_value_table(fb, v.as_ref());
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementGravity(v) |
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTransSpeed(v) |
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTangentialAcceleration(v) |
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementAlphaFade(v) => {
                let offset = self.create_shared_vec2_value_table(fb, v.as_ref());
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementPosition(v) => {
                let offset = self.create_shared_rect_value_table(fb, v.as_ref());
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementRotation(v) => {
                let mut w_args = ssab::ssab::EffectParticleElementRotationArgs::default();
                w_args.rotation_value = v.rotation_value.as_ref().map(|rotation_value| self.create_shared_vec2(fb, &rotation_value));
                w_args.rotation_add_value = v.rotation_add_value.as_ref().map(|rotation_add_value| self.create_shared_vec2(fb, &rotation_add_value));
                let offset = ssab::ssab::EffectParticleElementRotation::create(fb, &w_args);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementRotationTrans(v) => {
                Some(v.pack(fb).as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementInitColor(v) => {
                Some(v.pack(fb).as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTransColor(v) => {
                Some(v.pack(fb).as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementSize(v) => {
                let mut w_args = ssab::ssab::RectAndVec2ValueTableArgs::default();
                w_args.size = v.size.as_ref().map(|size| self.create_shared_rect(fb, size));
                w_args.scale_factor_value = v.scale_factor_value.as_ref().map(|scale_factor_value| self.create_shared_vec2(fb, &scale_factor_value));
                let offset = ssab::ssab::RectAndVec2ValueTable::create(fb, &w_args);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTransSize(v) => {
                let mut w_args = ssab::ssab::RectAndVec2ValueTableArgs::default();
                w_args.size = v.size.as_ref().map(|size| self.create_shared_rect(fb, size));
                w_args.scale_factor_value = v.scale_factor_value.as_ref().map(|scale_factor_value| self.create_shared_vec2(fb, &scale_factor_value));
                let offset = ssab::ssab::RectAndVec2ValueTable::create(fb, &w_args);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticlePointGravity(v) => {
                let mut w_args = ssab::ssab::EffectParticlePointGravityArgs::default();
                w_args.position = v.position.as_ref().map(|position| self.create_shared_vec2(fb, position));
                w_args.power = v.power;
                let offset = ssab::ssab::EffectParticlePointGravity::create(fb, &w_args);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleTurnToDirectionEnabled(v) => {
                let offset = self.create_shared_float_value_table(fb, v.as_ref());
                Some(offset.as_union_value())
            },
            _ => None,
        }
    }

    fn create_shared_effect_nodes(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, ns: &[ssab::ssab::EffectNodeT], ) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::EffectNode<'a>>>> {
        let nodes = ns.iter().map(|node| {
            let mut n_args = ssab::ssab::EffectNodeArgs::default();
            n_args.index = node.index;
            n_args.parent_index = node.parent_index;
            n_args.effect_node_type = node.effect_node_type;
            n_args.cell_index = node.cell_index;
            n_args.blend_type = node.blend_type;
            n_args.behavior = node.behavior.as_ref().map(|behavior| {
                let v = behavior.iter().map(|b| {
                    let mut w_args = ssab::ssab::EffectNodeBehaviorWrapperArgs::default();
                    w_args.value_type = b.value.effect_node_behavior_type();
                    w_args.value = self.create_shared_effect_node_behavior(fb, &b.value);
                    ssab::ssab::EffectNodeBehaviorWrapper::create(fb, &w_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            ssab::ssab::EffectNode::create(fb, &n_args)
        }).collect::<Vec<_>>();
        fb.create_vector(&nodes)
    }

    fn create_shared_effect_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, effet_list_t_vec: &[ssab::ssab::EffectT], ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::Effect<'a>>>> {
        let sound_list = effet_list_t_vec.iter().map(|effect_t| {
            let mut args = ssab::ssab::EffectArgs::default();
            args.name = Some(fb.create_shared_string(&effect_t.name));
            args.fps = effect_t.fps;
            args.is_lock_rand_seed = effect_t.is_lock_rand_seed;
            args.lock_rand_seed = effect_t.lock_rand_seed;
            args.layout_scale = effect_t.layout_scale.as_ref().map(|v| self.create_shared_vec2(fb, v));
            args.effect_node = effect_t.effect_node.as_ref().map(|nodes| self.create_shared_effect_nodes(fb, &nodes));

            ssab::ssab::Effect::create(fb, &args)
        }).collect::<Vec<_>>();

        fb.create_vector(&sound_list)
    }

    fn create_shared_sound_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, sound_list_t_vec: &[ssab::ssab::SoundListT], ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::SoundList<'a>>>> {
        let sound_list = sound_list_t_vec.iter().map(|sound_list_t| {
            let mut args = ssab::ssab::SoundListArgs::default();
            args.name = Some(fb.create_shared_string(&sound_list_t.name));
            args.table_data = sound_list_t.table_data.as_ref().map(|table_data| self.create_shared_sound_files(fb, table_data));

            ssab::ssab::SoundList::create(fb, &args)
        }).collect::<Vec<_>>();

        fb.create_vector(&sound_list)
    }

    fn create_shared_font_bitmap(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, font_bitmap_t_vec: &[ssab::ssab::FontBitmapT], ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::FontBitmap<'a>>>> {
        let font_bitmap = font_bitmap_t_vec.iter().map(|font_bitmap_t| {
            let mut args = ssab::ssab::FontBitmapArgs::default();
            args.name = Some(fb.create_shared_string(&font_bitmap_t.name));
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
            args.table_data = font_bitmap_t.table_data.as_ref().map(|table_data| {
                let v = table_data.iter().map(|font| {
                    let mut f_args = ssab::ssab::FontGlyphBitmapArgs::default();
                    f_args.id = font.id;
                    f_args.atlas = font.atlas.as_ref().map(|atlas| self.create_shared_rect(fb, atlas));
                    f_args.offset = font.offset.as_ref().map(|offset| self.create_shared_vec2(fb, offset));
                    f_args.width_advance = font.width_advance;
                    f_args.text = font.text.as_ref().map(|text| fb.create_shared_string(text));
                    f_args.page = font.page;
                    f_args.channel = font.channel;

                    ssab::ssab::FontGlyphBitmap::create(fb, &f_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });

            ssab::ssab::FontBitmap::create(fb, &args)
        }).collect::<Vec<_>>();

        fb.create_vector(&font_bitmap)
    }

    pub fn create_shared_anime_binary(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, ss_anime_binary_t: &ssab::ssab::SsAnimeBinaryT, ) -> flatbuffers::WIPOffset<ssab::ssab::SsAnimeBinary<'a>> {
        let mut args = ssab::ssab::SsAnimeBinaryArgs::default();

        args.name = ss_anime_binary_t.name.as_ref().map(|s| fb.create_shared_string(s));
        args.parts = ss_anime_binary_t.parts.as_ref().map(|parts| self.create_shared_part_list(fb, parts));
        args.animations = ss_anime_binary_t.animations.as_ref().map(|animations| self.create_animation_data_list(fb, animations));
        args.cellmaps = ss_anime_binary_t.cellmaps.as_ref().map(|cell_map| self.create_shared_cell_map(fb, cell_map));
        args.effects = ss_anime_binary_t.effects.as_ref().map(|effect| self.create_shared_effect_list(fb, effect));
        args.sound_lists = ss_anime_binary_t.sound_lists.as_ref().map(|sound_list| self.create_shared_sound_list(fb, sound_list));
        args.font_bitmaps = ss_anime_binary_t.font_bitmaps.as_ref().map(|font_bitmap| self.create_shared_font_bitmap(fb, font_bitmap));
        args.external_instances = ss_anime_binary_t.external_instances.as_ref().map(|external_instances| {
            let vec = external_instances.iter().map(|s| fb.create_shared_string(s)).collect::<Vec<_>>();
            fb.create_vector(&vec)
        });

        ssab::ssab::SsAnimeBinary::create(fb, &args)
    }
}
