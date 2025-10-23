use std::collections::HashMap;
use flatbuffers::{ForwardsUOffset, Vector, WIPOffset};
use ordered_float::OrderedFloat;
use ssab::ssab::{AnimationData, AnimationDataArgs, AnimationDataT, BoolValueTable, BoolValueTableArgs, BoolValueTableT, FloatValueTable, FloatValueTableArgs, FloatValueTableT, IntValueTable, IntValueTableArgs, IntValueTableT, Label, MeshDataIndices, MeshDataUV, PartState, SsAnimeBinary, SsAnimeBinaryArgs, SsAnimeBinaryT, };
use ssab::ssab::{Rect, RectArgs, RectT, Vec2, Vec2Args, Vec2T, Cell, CellArgs, CellMap, CellMapArgs, CellMapT, CellT, FontBitmap, FontBitmapArgs, FontBitmapT, SoundFile, SoundFileArgs, SoundFileT, SoundList, SoundListArgs, SoundListT, };

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec2Key {
    pub v1: OrderedFloat<f32>,
    pub v2: OrderedFloat<f32>,
}
impl From<&Vec2T> for Vec2Key {
    fn from(v: &Vec2T) -> Self {
        Self {
            v1: OrderedFloat(v.v1),
            v2: OrderedFloat(v.v2),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RectKey {
    pub x1: OrderedFloat<f32>,
    pub y1: OrderedFloat<f32>,
    pub x2: OrderedFloat<f32>,
    pub y2: OrderedFloat<f32>,
}
impl From<&ssab::ssab::RectT> for RectKey {
    fn from(value: &ssab::ssab::RectT) -> Self {
        Self {
            x1: OrderedFloat(value.x1),
            y1: OrderedFloat(value.y1),
            x2: OrderedFloat(value.x2),
            y2: OrderedFloat(value.y2),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct IntValueTableKey {
    pub value: i32,
}
impl From<&IntValueTableT> for IntValueTableKey {
    fn from(int_value_table_t: &IntValueTableT) -> Self {
        Self {
            value: int_value_table_t.value,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct BoolValueTableKey {
    pub value: bool,
}
impl From<&BoolValueTableT> for BoolValueTableKey {
    fn from(int_value_table_t: &BoolValueTableT) -> Self {
        Self {
            value: int_value_table_t.value,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct FloatValueTableKey {
    pub value: OrderedFloat<f32>,
}
impl From<&FloatValueTableT> for FloatValueTableKey {
    fn from(float_value_table_t: &FloatValueTableT) -> Self {
        Self {
            value: OrderedFloat(float_value_table_t.value),
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
impl From<&CellT> for CellKey {
    fn from(value: &CellT) -> Self {
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

pub struct SsabCache<'bldr> {
    pub vec2_cache: HashMap<Vec2Key, WIPOffset<Vec2<'bldr>>>,
    pub rect_cache: HashMap<RectKey, WIPOffset<Rect<'bldr>>>,
    pub int_value_table_cache: HashMap<IntValueTableKey, WIPOffset<IntValueTable<'bldr>>>,
    pub bool_value_table_cache: HashMap<BoolValueTableKey, WIPOffset<BoolValueTable<'bldr>>>,
    pub float_value_table_cache: HashMap<FloatValueTableKey, WIPOffset<FloatValueTable<'bldr>>>,

    pub cell_cache: HashMap<CellKey, WIPOffset<Cell<'bldr>>>,
}
impl<'bldr> SsabCache<'bldr> {
    pub fn new() -> Self {
        Self {
            vec2_cache: HashMap::new(),
            rect_cache: HashMap::new(),
            int_value_table_cache: HashMap::new(),
            bool_value_table_cache: HashMap::new(),
            float_value_table_cache: HashMap::new(),

            cell_cache: HashMap::new(),
        }
    }
    fn create_shared_vec2<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        vec2t: &Vec2T
    ) -> WIPOffset<Vec2<'bldr>>
    {
        let key = Vec2Key::from(vec2t);
        if let Some(&offset) = self.vec2_cache.get(&key) {
            return offset;
        }
        let mut args = Vec2Args::default();
        args.v1 = vec2t.v1;
        args.v2 = vec2t.v2;
        let offset = Vec2::create(fb, &args);
        self.vec2_cache.insert(key, offset);
        offset
    }

    fn create_shared_rect<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        rectt: &RectT,
    ) -> WIPOffset<Rect<'bldr>>
    {
        let key = RectKey::from(rectt);
        if let Some(&offset) = self.rect_cache.get(&key) {
            return offset;
        }
        let mut args = RectArgs::default();
        args.x1 = rectt.x1;
        args.y1 = rectt.y1;
        args.x2 = rectt.x2;
        args.y2 = rectt.y2;
        let offset = Rect::create(fb, &args);
        self.rect_cache.insert(key, offset);
        offset
    }

    fn create_shared_int_value_table<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        int_value_table_t: &IntValueTableT,
    ) -> WIPOffset<IntValueTable<'bldr>>
    {
        let key = IntValueTableKey::from(int_value_table_t);
        if let Some(&offset) = self.int_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = IntValueTableArgs::default();
        args.value = int_value_table_t.value;
        let offset = IntValueTable::create(fb, &args);
        offset
    }

    fn create_shared_bool_value_table<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        bool_value_table_t: &BoolValueTableT,
    ) -> WIPOffset<BoolValueTable<'bldr>>
    {
        let key = BoolValueTableKey::from(bool_value_table_t);
        if let Some(&offset) = self.bool_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = BoolValueTableArgs::default();
        args.value = bool_value_table_t.value;
        let offset = BoolValueTable::create(fb, &args);
        offset
    }

    fn create_shared_float_value_table<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        float_value_table_t: &FloatValueTableT,
    ) -> WIPOffset<FloatValueTable<'bldr>>
    {
        let key = FloatValueTableKey::from(float_value_table_t);
        if let Some(&offset) = self.float_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = FloatValueTableArgs::default();
        args.value = float_value_table_t.value;
        let offset = FloatValueTable::create(fb, &args);
        offset
    }
}

impl<'bldr> SsabCache<'bldr> {
    fn create_shared_cell<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        cell: &CellT,
    ) -> WIPOffset<Cell<'bldr>> {
        let key = CellKey::from(cell);
        if let Some(&offset) = self.cell_cache.get(&key) {
            return offset;
        }

        let mut args = CellArgs::default();
        args.name = Some(fb.create_shared_string(&cell.name));
        args.rectangle = cell.rectangle.as_ref().map(|r|self.create_shared_rect(fb, r));
        args.pivot = cell.pivot.as_ref().map(|p| self.create_shared_vec2(fb, p));
        args.rotated = cell.rotated;
        args.table_coordinate = cell.table_coordinate.as_ref().map(|p| {
            let vec = p.iter().map(|v| self.create_shared_vec2(fb, v)).collect::<Vec<WIPOffset<Vec2<'bldr>>>>();
            fb.create_vector(&vec)
        });
        args.table_index_vertex = cell.table_index_vertex.as_ref().map(|p| {
            fb.create_vector(&p)
        });

        let offset = Cell::create(fb, &args);
        self.cell_cache.insert(key, offset);
        offset
    }

    fn create_animation_data_list<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        animation_data_t_vec: &[AnimationDataT]
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<AnimationData<'bldr>>>> {
        let animation_data_vec = animation_data_t_vec.iter().map(|animation_data_t| {
            let mut args = AnimationDataArgs::default();
            args.name = Some(fb.create_shared_string(&animation_data_t.name));
            args.setup_animation = None; // TODO; impl
            args.frame_data = None; // TODO; impl
            args.events = None; // TODO; impl
            args.label_data = None; // TODO; impl
            args.meshs_data_uv = None; // TODO; impl
            args.meshs_data_indices = None; // TODO; impl
            args.start_frames = animation_data_t.start_frames;
            args.end_frames = animation_data_t.end_frames;
            args.total_frames = animation_data_t.total_frames;
            args.fps = animation_data_t.fps;
            args.canvas = animation_data_t.canvas.as_ref().map(|c| self.create_shared_rect(fb, c));

            AnimationData::create(fb, &args)
        }).collect::<Vec<WIPOffset<AnimationData<'bldr>>>>();

        fb.create_vector(&animation_data_vec)
    }

    fn create_shared_cell_map<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        cell_map_t_vec: &[CellMapT]
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<CellMap<'bldr>>>> {
        let cell_map = cell_map_t_vec.iter().map(|cell_map_t| {
            let mut args = CellMapArgs::default();
            args.name = Some(fb.create_shared_string(&cell_map_t.name));
            args.image_path = cell_map_t.image_path.as_ref().map(|s| fb.create_shared_string(s));
            args.size_original = cell_map_t.size_original.as_ref().map(|s| self.create_shared_vec2(fb, s));
            args.wrap_mode = cell_map_t.wrap_mode;
            args.filter_mode = cell_map_t.filter_mode;
            args.cells = cell_map_t.cells.as_ref().map(|c| {
                let a = c.iter().map(|cell| self.create_shared_cell(fb, cell)).collect::<Vec<WIPOffset<Cell<'bldr>>>>();
                fb.create_vector(&a)
            });

            CellMap::create(fb, &args)
        }).collect::<Vec<WIPOffset<CellMap<'bldr>>>>();

        fb.create_vector(&cell_map)
    }

    fn create_shared_table_data<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        table_data_t_vec: &[SoundFileT],
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<SoundFile<'bldr>>>> {
        let table_data = table_data_t_vec.iter().map(|table_data_t| {
            let mut sound_file_args = SoundFileArgs::default();
            sound_file_args.name = Some(fb.create_shared_string(&table_data_t.name));
            sound_file_args.time_total = table_data_t.time_total;

            SoundFile::create(fb, &sound_file_args)
        }).collect::<Vec<WIPOffset<SoundFile<'bldr>>>>();

        fb.create_vector(&table_data)
    }

    fn create_shared_sound_list<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        sound_list_t_vec: &[SoundListT]
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<SoundList<'bldr>>>> {
        let sound_list = sound_list_t_vec.iter().map(|sound_list_t| {
            let mut sound_list_args = SoundListArgs::default();
            sound_list_args.name = Some(fb.create_shared_string(&sound_list_t.name));
            //sound_list_args.table_data = self.create_shared_table_data(fb, &sound_list_t.table_data);
            sound_list_args.table_data = sound_list_t.table_data.as_ref().map(|table_data| { self.create_shared_table_data(fb, table_data) });

            SoundList::create(fb, &sound_list_args)
        }).collect::<Vec<WIPOffset<SoundList<'bldr>>>>();

        fb.create_vector(&sound_list)
    }

    fn create_shared_font_bitmap<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        font_bitmap_t_vec: &[FontBitmapT],
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<FontBitmap<'bldr>>>> {
        let font_bitmap = font_bitmap_t_vec.iter().map(|font_bitmap_t| {
            let mut font_bitmap_args = FontBitmapArgs::default();
            font_bitmap_args.name = Some(fb.create_shared_string(&font_bitmap_t.name));
            font_bitmap_args.name_face = font_bitmap_t.name_face.as_ref().map(|s| fb.create_shared_string(s));
            font_bitmap_args.size = font_bitmap_t.size;
            font_bitmap_args.padding_l = font_bitmap_t.padding_l;
            font_bitmap_args.padding_r = font_bitmap_t.padding_r;
            font_bitmap_args.padding_u = font_bitmap_t.padding_u;
            font_bitmap_args.padding_d = font_bitmap_t.padding_d;
            font_bitmap_args.base = font_bitmap_t.base;
            font_bitmap_args.height_line = font_bitmap_t.height_line;
            font_bitmap_args.scale = font_bitmap_t.scale.as_ref().map(|s| self.create_shared_vec2(fb, s));
            font_bitmap_args.spacing = font_bitmap_t.spacing.as_ref().map(|s| self.create_shared_vec2(fb, s));
            font_bitmap_args.size_texture_original = font_bitmap_t.size_texture_original.as_ref().map(|s| self.create_shared_vec2(fb, s));
            font_bitmap_args.name_texture = font_bitmap_t.name_texture.as_ref().map(|s| {
                let a = s.iter().map(|s| fb.create_shared_string(s)).collect::<Vec<_>>();
                fb.create_vector(&a)
            });
            font_bitmap_args.table_data =  None;

            FontBitmap::create(fb, &font_bitmap_args)
        }).collect::<Vec<WIPOffset<FontBitmap<'bldr>>>>();

        fb.create_vector(&font_bitmap)
    }

    pub fn create_shared_anime_binary<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        ss_anime_binary_t: &SsAnimeBinaryT
    ) -> WIPOffset<SsAnimeBinary<'bldr>> {
        let mut anime_binary_args = SsAnimeBinaryArgs::default();

        anime_binary_args.animations = ss_anime_binary_t.animations.as_ref().map(|animations| self.create_animation_data_list(fb, animations));
        anime_binary_args.cell_map = ss_anime_binary_t.cell_map.as_ref().map(|cell_map| self.create_shared_cell_map(fb, cell_map));
        anime_binary_args.effect = None; // TODO: implement
        anime_binary_args.sound_list = ss_anime_binary_t.sound_list.as_ref().map(|sound_list| self.create_shared_sound_list(fb, sound_list));
        anime_binary_args.font_bitmap = ss_anime_binary_t.font_bitmap.as_ref().map(|font_bitmap| self.create_shared_font_bitmap(fb, font_bitmap));

        anime_binary_args.instances = None; // TODO: implement

        SsAnimeBinary::create(fb, &anime_binary_args)
    }
}

