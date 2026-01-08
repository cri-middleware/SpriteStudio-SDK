use std::collections::HashMap;
use ssab::ssab::{BlendType, FontGlyphBitmap, Rect, SsAttributePartColorKeyValueColor, Vec2};

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
            interpolation: value.interpolation,
            value: ordered_float::OrderedFloat(value.value),
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

    pub part_type_none_cache: HashMap<String, flatbuffers::WIPOffset<ssab::ssab::PartTypeNone<'a>>>,

    pub part_animation_data_float_value_cache: HashMap<PartAnimationDataFloatValueKey, flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataFloatValue<'a>>>,
    pub part_animation_data_int_value_cache: HashMap<PartAnimationDataIntValueKey, flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataIntValue<'a>>>,
    pub part_animation_data_bool_value_cache: HashMap<PartAnimationDataBoolValueKey, flatbuffers::WIPOffset<ssab::ssab::PartAnimationDataBoolValue<'a>>>,
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

            part_type_none_cache: HashMap::new(),

            part_animation_data_float_value_cache: HashMap::new(),
            part_animation_data_int_value_cache: HashMap::new(),
            part_animation_data_bool_value_cache: HashMap::new(),
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
                let v = cell_array.iter().map(|cell| {
                    let mut c_args = ssab::ssab::PartAnimationDataCellValueArgs::default();
                    c_args.index = cell.index;
                    c_args.value = cell.value.as_ref().map(|value| {
                        let mut pc_args = ssab::ssab::PartAttributeCellArgs::default();
                        pc_args.map_id = value.map_id;
                        pc_args.name = value.name.as_ref().map(|name| fb.create_shared_string(name));

                        ssab::ssab::PartAttributeCell::create(fb, &pc_args)
                    });
                    ssab::ssab::PartAnimationDataCellValue::create(fb, &c_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });

            p_args.pos_x_array = part_animation_data.pos_x_array.as_ref().map(|pos_x_array| {
                let v = pos_x_array.iter().map(|pos_x| {
                    self.create_shared_part_animation_data_float_value(fb, &pos_x)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });

            p_args.pos_y_array = part_animation_data.pos_y_array.as_ref().map(|pos_y_array| {
                let v = pos_y_array.iter().map(|pos_y| {
                    self.create_shared_part_animation_data_float_value(fb, &pos_y)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.pos_z_array = part_animation_data.pos_z_array.as_ref().map(|pos_z_array| {
                let v = pos_z_array.iter().map(|pos_z| {
                    self.create_shared_part_animation_data_float_value(fb, &pos_z)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.rot_x_array = part_animation_data.rot_x_array.as_ref().map(|rot_x_array| {
                let v = rot_x_array.iter().map(|rot_x| {
                    self.create_shared_part_animation_data_float_value(fb, &rot_x)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.rot_y_array = part_animation_data.rot_y_array.as_ref().map(|rot_y_array| {
                let v = rot_y_array.iter().map(|rot_y| {
                    self.create_shared_part_animation_data_float_value(fb, &rot_y)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.rot_z_array = part_animation_data.rot_z_array.as_ref().map(|rot_z_array| {
                let v = rot_z_array.iter().map(|rot_z| {
                    self.create_shared_part_animation_data_float_value(fb, &rot_z)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.scl_x_array = part_animation_data.scl_x_array.as_ref().map(|scl_x_array| {
                let v = scl_x_array.iter().map(|scl_x| {
                    self.create_shared_part_animation_data_float_value(fb, &scl_x)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.scl_y_array = part_animation_data.scl_y_array.as_ref().map(|scl_y_array| {
                let v = scl_y_array.iter().map(|scl_y| {
                    self.create_shared_part_animation_data_float_value(fb, &scl_y)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.loscl_x_array = part_animation_data.loscl_x_array.as_ref().map(|loscl_x_array| {
                let v = loscl_x_array.iter().map(|loscl_x| {
                    self.create_shared_part_animation_data_float_value(fb, &loscl_x)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.loscl_y_array = part_animation_data.loscl_y_array.as_ref().map(|loscl_y_array| {
                let v = loscl_y_array.iter().map(|loscl_y| {
                    self.create_shared_part_animation_data_float_value(fb, &loscl_y)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.alpha_array = part_animation_data.alpha_array.as_ref().map(|alpha_array| {
                let v = alpha_array.iter().map(|alpha| {
                    self.create_shared_part_animation_data_float_value(fb, &alpha)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.lo_alpha_array = part_animation_data.lo_alpha_array.as_ref().map(|lo_alpha_array| {
                let v = lo_alpha_array.iter().map(|lo_alpha| {
                    self.create_shared_part_animation_data_float_value(fb, &lo_alpha)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.prio_array = part_animation_data.prio_array.as_ref().map(|prio_array| {
                let v = prio_array.iter().map(|prio| {
                    self.create_shared_part_animation_data_int_value(fb, &prio)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.flip_h_array = part_animation_data.flip_h_array.as_ref().map(|flip_h_array| {
                let v = flip_h_array.iter().map(|flip_h| {
                    self.create_shared_part_animation_data_bool_value(fb, &flip_h)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.flip_v_array = part_animation_data.flip_v_array.as_ref().map(|flip_v_array| {
                let v = flip_v_array.iter().map(|flip_v| {
                    self.create_shared_part_animation_data_bool_value(fb, &flip_v)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.hide_array = part_animation_data.hide_array.as_ref().map(|hide_array| {
                let v = hide_array.iter().map(|hide| {
                    self.create_shared_part_animation_data_bool_value(fb, &hide)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.parts_color_array = part_animation_data.parts_color_array.as_ref().map(|parts_color_array| {
                let v = parts_color_array.iter().map(|parts_color| {
                    let mut p_args = ssab::ssab::PartAnimationDataPartColorArgs::default();
                    p_args.index = parts_color.index;
                    p_args.interpolation = parts_color.interpolation;
                    p_args.value = parts_color.value.as_ref().map(|value| {
                        let mut v_args = ssab::ssab::PartAttributePartColorArgs::default();
                        v_args.blend_type = value.blend_type;
                        v_args.lt = value.lt.as_ref().map(|lt| {
                            let mut args = ssab::ssab::SsAttributePartColorKeyValueColorArgs::default();
                            args.rgba = lt.rgba.as_ref().map(|rgba| self.create_shared_u8rect(fb, rgba));
                            args.rate = lt.rate;

                            ssab::ssab::SsAttributePartColorKeyValueColor::create(fb, &args)
                        });
                        v_args.rt = value.rt.as_ref().map(|lt| {
                            let mut args = ssab::ssab::SsAttributePartColorKeyValueColorArgs::default();
                            args.rgba = lt.rgba.as_ref().map(|rgba| self.create_shared_u8rect(fb, rgba));
                            args.rate = lt.rate;

                            ssab::ssab::SsAttributePartColorKeyValueColor::create(fb, &args)
                        });
                        v_args.lb = value.lb.as_ref().map(|lt| {
                            let mut args = ssab::ssab::SsAttributePartColorKeyValueColorArgs::default();
                            args.rgba = lt.rgba.as_ref().map(|rgba| self.create_shared_u8rect(fb, rgba));
                            args.rate = lt.rate;

                            ssab::ssab::SsAttributePartColorKeyValueColor::create(fb, &args)
                        });
                        v_args.rb = value.rb.as_ref().map(|lt| {
                            let mut args = ssab::ssab::SsAttributePartColorKeyValueColorArgs::default();
                            args.rgba = lt.rgba.as_ref().map(|rgba| self.create_shared_u8rect(fb, rgba));
                            args.rate = lt.rate;

                            ssab::ssab::SsAttributePartColorKeyValueColor::create(fb, &args)
                        });
                        ssab::ssab::PartAttributePartColor::create(fb, &v_args)
                    });

                    ssab::ssab::PartAnimationDataPartColor::create(fb, &p_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.shader_array = part_animation_data.shader_array.as_ref().map(|shader_array| {
                let v = shader_array.iter().map(|shader| {
                    let mut s_srgs = ssab::ssab::PartAnimationDataShaderArgs::default();
                    s_srgs.index = shader.index;
                    s_srgs.value = shader.value.as_ref().map(|value| {
                        let mut v_args = ssab::ssab::PartAttributeShaderArgs::default();
                        v_args.id = value.id.as_ref().map(|id| fb.create_shared_string(&id));
                        v_args.param0 = value.param0;
                        v_args.param1 = value.param1;
                        v_args.param2 = value.param2;
                        v_args.param3 = value.param3;
                        v_args.param4 = value.param4;
                        v_args.param5 = value.param5;
                        v_args.param6 = value.param6;
                        v_args.param7 = value.param7;
                        v_args.map0_id = value.map0_id;
                        v_args.map0_name = value.map0_name.as_ref().map(|map0_name| fb.create_shared_string(&map0_name));
                        v_args.map1_id = value.map1_id;
                        v_args.map1_name = value.map1_name.as_ref().map(|map1_name| fb.create_shared_string(&map1_name));

                        ssab::ssab::PartAttributeShader::create(fb, &v_args)
                    });

                    ssab::ssab::PartAnimationDataShader::create(fb, &s_srgs)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.vertex_array = part_animation_data.vertex_array.as_ref().map(|vertex_array| {
                let v = vertex_array.iter().map(|vertex| {
                    let mut v_args = ssab::ssab::PartAnimationDataVertexArgs::default();
                    v_args.index = vertex.index;
                    v_args.interpolation = vertex.interpolation;
                    v_args.value = vertex.value.as_ref().map(|value| {
                        let mut args = ssab::ssab::PartAttributeVertexArgs::default();
                        args.lt = value.lt.as_ref().map(|lt| self.create_shared_vec2(fb, lt));
                        args.rt = value.rt.as_ref().map(|rt| self.create_shared_vec2(fb, rt));
                        args.lb = value.lb.as_ref().map(|lb| self.create_shared_vec2(fb, lb));
                        args.rb = value.rb.as_ref().map(|rb| self.create_shared_vec2(fb, rb));
                        ssab::ssab::PartAttributeVertex::create(fb, &args)
                    });
                    ssab::ssab::PartAnimationDataVertex::create(fb, &v_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.pivot_x_array = part_animation_data.pivot_x_array.as_ref().map(|pivot_x_array| {
                let v = pivot_x_array.iter().map(|pivot_x| {
                    self.create_shared_part_animation_data_float_value(fb, &pivot_x)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.pivot_y_array = part_animation_data.pivot_y_array.as_ref().map(|pivot_y_array| {
                let v = pivot_y_array.iter().map(|pivot_y| {
                    self.create_shared_part_animation_data_float_value(fb, &pivot_y)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.anchor_x_array = part_animation_data.anchor_x_array.as_ref().map(|anchor_x_array| {
                let v = anchor_x_array.iter().map(|anchor_x| {
                    self.create_shared_part_animation_data_float_value(fb, &anchor_x)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.anchor_y_array = part_animation_data.anchor_y_array.as_ref().map(|anchor_y_array| {
                let v = anchor_y_array.iter().map(|anchor_y| {
                    self.create_shared_part_animation_data_float_value(fb, &anchor_y)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.size_x_array = part_animation_data.size_x_array.as_ref().map(|size_x_array| {
                let v = size_x_array.iter().map(|size_x| {
                    self.create_shared_part_animation_data_float_value(fb, &size_x)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.size_y_array = part_animation_data.size_y_array.as_ref().map(|size_y_array| {
                let v = size_y_array.iter().map(|size_y| {
                    self.create_shared_part_animation_data_float_value(fb, &size_y)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.img_flip_h_array = part_animation_data.img_flip_h_array.as_ref().map(|img_flip_h_array| {
                let v = img_flip_h_array.iter().map(|img_flip_h| {
                    self.create_shared_part_animation_data_bool_value(fb, &img_flip_h)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.img_flip_v_array = part_animation_data.img_flip_v_array.as_ref().map(|img_flip_v_array| {
                let v = img_flip_v_array.iter().map(|img_flip_v| {
                    self.create_shared_part_animation_data_bool_value(fb, &img_flip_v)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.uvt_x_array = part_animation_data.uvt_x_array.as_ref().map(|uvt_x_array| {
                let v = uvt_x_array.iter().map(|uvt_x| {
                    self.create_shared_part_animation_data_float_value(fb, &uvt_x)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.uvt_y_array = part_animation_data.uvt_y_array.as_ref().map(|uvt_y_array| {
                let v = uvt_y_array.iter().map(|uvt_y| {
                    self.create_shared_part_animation_data_float_value(fb, &uvt_y)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.uvr_z_array = part_animation_data.uvr_z_array.as_ref().map(|uvr_z_array| {
                let v = uvr_z_array.iter().map(|uvr_z| {
                    self.create_shared_part_animation_data_float_value(fb, &uvr_z)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.uvs_x_array = part_animation_data.uvs_x_array.as_ref().map(|uvs_x_array| {
                let v = uvs_x_array.iter().map(|uvs_x| {
                    self.create_shared_part_animation_data_float_value(fb, &uvs_x)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.uvs_y_array = part_animation_data.uvs_y_array.as_ref().map(|uvs_y_array| {
                let v = uvs_y_array.iter().map(|uvs_y| {
                    self.create_shared_part_animation_data_float_value(fb, &uvs_y)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.boundr_array = part_animation_data.boundr_array.as_ref().map(|boundr_array| {
                let v = boundr_array.iter().map(|boundr| {
                    self.create_shared_part_animation_data_float_value(fb, &boundr)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.mask_array = part_animation_data.mask_array.as_ref().map(|mask_array| {
                let v = mask_array.iter().map(|mask| {
                    self.create_shared_part_animation_data_float_value(fb, &mask)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
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
                        let mut v_args = ssab::ssab::PartAttributeDeformArgs::default();
                        v_args.coordinate = value.coordinate.as_ref().map(|coordinate| {
                            let cv = coordinate.iter().map(|cv| self.create_shared_vec2(fb, cv)).collect::<Vec<_>>();
                            fb.create_vector(&cv)
                        });
                        v_args.index = value.index.as_ref().map(|index| fb.create_vector(&index));
                        ssab::ssab::PartAttributeDeform::create(fb, &v_args)
                    });

                    ssab::ssab::PartAnimationDataDeform::create(fb, &d_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            p_args.texture_array = part_animation_data.texture_array.as_ref().map(|texture_array| {
                let v = texture_array.iter().map(|texture| {
                    self.create_shared_part_animation_data_int_value(fb, &texture)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
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
                        let mut pi_args = ssab::ssab::PartAttributeInstanceArgs::default();
                        pi_args.pingpong = value.pingpong;
                        pi_args.independent = value.independent;
                        pi_args.loop_num = value.loop_num;
                        pi_args.start_label = value.start_label.as_ref().map(|start_label| fb.create_shared_string(start_label));
                        pi_args.start_offset = value.start_offset;
                        pi_args.end_label = value.end_label.as_ref().map(|end_label| fb.create_shared_string(end_label));
                        pi_args.end_offset = value.end_offset;
                        pi_args.speed = value.speed;
                        ssab::ssab::PartAttributeInstance::create(fb, &pi_args)
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

    fn create_parts_initial_attributes(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, parts_initial_attributes: &[ssab::ssab::PartAttributesT]) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::PartAttributes<'a>>>> {
        let v = parts_initial_attributes.iter().map(|attribute| {
            let mut p_args = ssab::ssab::PartAttributesArgs::default();
            p_args.cell = attribute.cell.as_ref().map(|cell| {
                let mut c_args = ssab::ssab::PartAttributeCellArgs::default();
                c_args.map_id = cell.map_id;
                c_args.name = cell.name.as_ref().map(|name| fb.create_shared_string(name));
                ssab::ssab::PartAttributeCell::create(fb, &c_args)
            });
            p_args.position = attribute.position.as_ref().map(|position| self.create_shared_vec3(fb, &position));
            p_args.rotation = attribute.rotation.as_ref().map(|rotation| self.create_shared_vec3(fb, &rotation));
            p_args.scale = attribute.scale.as_ref().map(|scale| self.create_shared_vec2(fb, &scale));
            p_args.local_scale = attribute.local_scale.as_ref().map(|local_scale|self.create_shared_vec2(fb, &local_scale));
            p_args.opacity = attribute.opacity;
            p_args.localopacity = attribute.localopacity;
            p_args.priority = attribute.priority;
            p_args.flip_v = attribute.flip_v;
            p_args.flip_h = attribute.flip_h;
            p_args.hide = attribute.hide;
            p_args.parts_color = attribute.parts_color.as_ref().map(|parts_color| {
                let mut pc_args = ssab::ssab::PartAttributePartColorArgs::default();
                pc_args.blend_type = parts_color.blend_type;
                pc_args.lt = parts_color.lt.as_ref().map(|lt| {
                    let mut args = ssab::ssab::SsAttributePartColorKeyValueColorArgs::default();
                    args.rgba = lt.rgba.as_ref().map(|rgba| self.create_shared_u8rect(fb, rgba));
                    args.rate = lt.rate;

                    ssab::ssab::SsAttributePartColorKeyValueColor::create(fb, &args)
                });
                pc_args.rt = parts_color.rt.as_ref().map(|lt| {
                    let mut args = ssab::ssab::SsAttributePartColorKeyValueColorArgs::default();
                    args.rgba = lt.rgba.as_ref().map(|rgba| self.create_shared_u8rect(fb, rgba));
                    args.rate = lt.rate;

                    ssab::ssab::SsAttributePartColorKeyValueColor::create(fb, &args)
                });
                pc_args.lb = parts_color.lb.as_ref().map(|lt| {
                    let mut args = ssab::ssab::SsAttributePartColorKeyValueColorArgs::default();
                    args.rgba = lt.rgba.as_ref().map(|rgba| self.create_shared_u8rect(fb, rgba));
                    args.rate = lt.rate;

                    ssab::ssab::SsAttributePartColorKeyValueColor::create(fb, &args)
                });
                pc_args.rb = parts_color.rb.as_ref().map(|lt| {
                    let mut args = ssab::ssab::SsAttributePartColorKeyValueColorArgs::default();
                    args.rgba = lt.rgba.as_ref().map(|rgba| self.create_shared_u8rect(fb, rgba));
                    args.rate = lt.rate;

                    ssab::ssab::SsAttributePartColorKeyValueColor::create(fb, &args)
                });
                ssab::ssab::PartAttributePartColor::create(fb, &pc_args)
            });
            p_args.shader = attribute.shader.as_ref().map(|shader| {
                let mut v_args = ssab::ssab::PartAttributeShaderArgs::default();
                v_args.id = shader.id.as_ref().map(|id| fb.create_shared_string(&id));
                v_args.param0 = shader.param0;
                v_args.param1 = shader.param1;
                v_args.param2 = shader.param2;
                v_args.param3 = shader.param3;
                v_args.param4 = shader.param4;
                v_args.param5 = shader.param5;
                v_args.param6 = shader.param6;
                v_args.param7 = shader.param7;
                v_args.map0_id = shader.map0_id;
                v_args.map0_name = shader.map0_name.as_ref().map(|map0_name| fb.create_shared_string(&map0_name));
                v_args.map1_id = shader.map1_id;
                v_args.map1_name = shader.map1_name.as_ref().map(|map1_name| fb.create_shared_string(&map1_name));

                ssab::ssab::PartAttributeShader::create(fb, &v_args)
            });
            p_args.vertex = attribute.vertex.as_ref().map(|vertex| {
                let mut args = ssab::ssab::PartAttributeVertexArgs::default();
                args.lt = vertex.lt.as_ref().map(|lt| self.create_shared_vec2(fb, lt));
                args.rt = vertex.rt.as_ref().map(|rt| self.create_shared_vec2(fb, rt));
                args.lb = vertex.lb.as_ref().map(|lb| self.create_shared_vec2(fb, lb));
                args.rb = vertex.rb.as_ref().map(|rb| self.create_shared_vec2(fb, rb));
                ssab::ssab::PartAttributeVertex::create(fb, &args)
            });
            p_args.pivot = attribute.pivot.as_ref().map(|pivot| self.create_shared_vec2(fb, &pivot));
            p_args.size = attribute.size.as_ref().map(|size| self.create_shared_vec2(fb, &size));
            p_args.uv_move = attribute.uv_move.as_ref().map(|uv_move| self.create_shared_vec3(fb, &uv_move));
            p_args.uv_scale = attribute.uv_scale.as_ref().map(|uv_scale| self.create_shared_vec2(fb, &uv_scale));
            p_args.bounding_radius = attribute.bounding_radius;
            p_args.mask = attribute.mask;
            p_args.skew = attribute.skew.as_ref().map(|skew| self.create_shared_vec2(fb, &skew));
            p_args.deform = attribute.deform.as_ref().map(|deform| {
                let mut v_args = ssab::ssab::PartAttributeDeformArgs::default();
                v_args.coordinate = deform.coordinate.as_ref().map(|coordinate| {
                    let cv = coordinate.iter().map(|cv| self.create_shared_vec2(fb, cv)).collect::<Vec<_>>();
                    fb.create_vector(&cv)
                });
                v_args.index = deform.index.as_ref().map(|index| fb.create_vector(&index));
                ssab::ssab::PartAttributeDeform::create(fb, &v_args)
            });
            p_args.texture = attribute.texture;

            ssab::ssab::PartAttributes::create(fb, &p_args)
        }).collect::<Vec<_>>();
        fb.create_vector(&v)
    }

    fn create_animation_data_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, animation_data_t_vec: &[ssab::ssab::AnimationDataT]) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::AnimationData<'a>>>> {
        let animation_data_vec = animation_data_t_vec.iter().map(|animation_data_t| {
            let mut args = ssab::ssab::AnimationDataArgs::default();
            args.name = Some(fb.create_shared_string(&animation_data_t.name));
            args.parts_initial_attributes = animation_data_t.parts_initial_attributes.as_ref().map(|parts_initial_attributes| self.create_parts_initial_attributes(fb, parts_initial_attributes));
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
