use flatbuffers::{ForwardsUOffset, Vector, WIPOffset};
use ordered_float::OrderedFloat;
use ssab::ssab::{
    AnimationData, AnimationDataArgs, AnimationDataT, Effect, EffectArgs, EffectT, FloatValueEntry,
    FloatValueEntryArgs, FloatValueEntryT, IntValueEntry, IntValueEntryArgs, IntValueEntryT,
    RectValueEntry, RectValueEntryArgs, RectValueEntryT, SsAnimeBinary, SsAnimeBinaryArgs,
    SsAnimeBinaryT, Vec2ValueEntry, Vec2ValueEntryArgs, Vec2ValueEntryT,
};
use ssab::ssab::{
    Cell, CellArgs, CellMap, CellMapArgs, CellMapT, CellT, FontBitmap, FontBitmapArgs, FontBitmapT,
    Rect, RectArgs, RectT, SoundFile, SoundFileArgs, SoundFileT, SoundList, SoundListArgs,
    SoundListT, Vec2, Vec2Args, Vec2T,
};
use std::collections::HashMap;

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
impl Vec2Key {
    fn new(x: f32, y: f32) -> Self {
        Self {
            v1: OrderedFloat(x),
            v2: OrderedFloat(y),
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
impl RectKey {
    fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            x1: OrderedFloat(x1),
            y1: OrderedFloat(y1),
            x2: OrderedFloat(x2),
            y2: OrderedFloat(y2),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct IntValueEntryKey {
    pub value: i32,
}
impl From<&IntValueEntryT> for IntValueEntryKey {
    fn from(int_value_table_t: &IntValueEntryT) -> Self {
        Self {
            value: int_value_table_t.value,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct FloatValueEntryKey {
    pub value: OrderedFloat<f32>,
}
impl From<&FloatValueEntryT> for FloatValueEntryKey {
    fn from(float_value_table_t: &FloatValueEntryT) -> Self {
        Self {
            value: OrderedFloat(float_value_table_t.value),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec2ValueEntryKey {
    pub value: Vec2Key,
}
impl From<&Vec2ValueEntryT> for Vec2ValueEntryKey {
    fn from(vec2_value_table_t: &Vec2ValueEntryT) -> Self {
        let value = vec2_value_table_t.value.clone().unwrap();
        let v1 = value.v1;
        let v2 = value.v2;
        Self {
            value: Vec2Key::new(v1, v2),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RectValueEntryKey {
    pub value: RectKey,
}
impl From<&RectValueEntryT> for RectValueEntryKey {
    fn from(rect_value_table_t: &RectValueEntryT) -> Self {
        let value = rect_value_table_t.value.clone().unwrap();
        let x1 = value.x1;
        let y1 = value.y1;
        let x2 = value.x2;
        let y2 = value.y2;
        Self {
            value: RectKey::new(x1, y1, x2, y2),
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
            table_coordinate: value
                .table_coordinate
                .as_ref()
                .map(|t| t.iter().map(|v| Vec2Key::from(v)).collect::<Vec<Vec2Key>>()),
            table_index_vertex: value
                .table_index_vertex
                .as_ref()
                .map(|t| t.iter().map(|v| *v).collect::<Vec<u16>>()),
        }
    }
}

pub struct SsabCache<'bldr> {
    pub vec2_cache: HashMap<Vec2Key, WIPOffset<Vec2<'bldr>>>,
    pub rect_cache: HashMap<RectKey, WIPOffset<Rect<'bldr>>>,
    pub int_value_table_cache: HashMap<IntValueEntryKey, WIPOffset<IntValueEntry<'bldr>>>,
    pub float_value_table_cache: HashMap<FloatValueEntryKey, WIPOffset<FloatValueEntry<'bldr>>>,
    pub vec2_value_table_cache: HashMap<Vec2ValueEntryKey, WIPOffset<Vec2ValueEntry<'bldr>>>,
    pub rect_value_table_cache: HashMap<RectValueEntryKey, WIPOffset<RectValueEntry<'bldr>>>,

    pub cell_cache: HashMap<CellKey, WIPOffset<Cell<'bldr>>>,
}
impl<'bldr> SsabCache<'bldr> {
    pub fn new() -> Self {
        Self {
            vec2_cache: HashMap::new(),
            rect_cache: HashMap::new(),
            int_value_table_cache: HashMap::new(),
            float_value_table_cache: HashMap::new(),
            vec2_value_table_cache: HashMap::new(),
            rect_value_table_cache: HashMap::new(),

            cell_cache: HashMap::new(),
        }
    }
    fn create_shared_vec2<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        vec2t: &Vec2T,
    ) -> WIPOffset<Vec2<'bldr>> {
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
    ) -> WIPOffset<Rect<'bldr>> {
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

    #[allow(dead_code)]
    fn create_shared_int_value_table<
        'args: 'mut_bldr,
        'mut_bldr,
        A: flatbuffers::Allocator + 'bldr,
    >(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        int_value_table_t: &IntValueEntryT,
    ) -> WIPOffset<IntValueEntry<'bldr>> {
        let key = IntValueEntryKey::from(int_value_table_t);
        if let Some(&offset) = self.int_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = IntValueEntryArgs::default();
        args.value = int_value_table_t.value;
        let offset = IntValueEntry::create(fb, &args);
        offset
    }

    #[allow(dead_code)]
    fn create_shared_float_value_table<
        'args: 'mut_bldr,
        'mut_bldr,
        A: flatbuffers::Allocator + 'bldr,
    >(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        float_value_table_t: &FloatValueEntryT,
    ) -> WIPOffset<FloatValueEntry<'bldr>> {
        let key = FloatValueEntryKey::from(float_value_table_t);
        if let Some(&offset) = self.float_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = FloatValueEntryArgs::default();
        args.value = float_value_table_t.value;
        let offset = FloatValueEntry::create(fb, &args);
        offset
    }

    #[allow(dead_code)]
    fn create_shared_vec2_value_table<
        'args: 'mut_bldr,
        'mut_bldr,
        A: flatbuffers::Allocator + 'bldr,
    >(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        vec2_value_table_t: &Vec2ValueEntryT,
    ) -> WIPOffset<Vec2ValueEntry<'bldr>> {
        let key = Vec2ValueEntryKey::from(vec2_value_table_t);
        if let Some(&offset) = self.vec2_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = Vec2ValueEntryArgs::default();
        let v = vec2_value_table_t.value.as_ref().unwrap();
        args.value = Some(self.create_shared_vec2(fb, &v));
        let offset = Vec2ValueEntry::create(fb, &args);
        offset
    }

    #[allow(dead_code)]
    fn create_shared_rect_value_table<
        'args: 'mut_bldr,
        'mut_bldr,
        A: flatbuffers::Allocator + 'bldr,
    >(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        rect_value_table_t: &RectValueEntryT,
    ) -> WIPOffset<RectValueEntry<'bldr>> {
        let key = RectValueEntryKey::from(rect_value_table_t);
        if let Some(&offset) = self.rect_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = RectValueEntryArgs::default();
        let v = rect_value_table_t.value.as_ref().unwrap();
        args.value = Some(self.create_shared_rect(fb, &v));
        let offset = RectValueEntry::create(fb, &args);
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
        args.rectangle = cell
            .rectangle
            .as_ref()
            .map(|r| self.create_shared_rect(fb, r));
        args.pivot = cell.pivot.as_ref().map(|p| self.create_shared_vec2(fb, p));
        args.rotated = cell.rotated;
        args.table_coordinate = cell.table_coordinate.as_ref().map(|p| {
            let vec = p
                .iter()
                .map(|v| self.create_shared_vec2(fb, v))
                .collect::<Vec<WIPOffset<Vec2<'bldr>>>>();
            fb.create_vector(&vec)
        });
        args.table_index_vertex = cell
            .table_index_vertex
            .as_ref()
            .map(|p| fb.create_vector(&p));

        let offset = Cell::create(fb, &args);
        self.cell_cache.insert(key, offset);
        offset
    }

    fn create_animation_data_list<
        'args: 'mut_bldr,
        'mut_bldr,
        A: flatbuffers::Allocator + 'bldr,
    >(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        animation_data_t_vec: &[AnimationDataT],
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<AnimationData<'bldr>>>> {
        let animation_data_vec = animation_data_t_vec
            .iter()
            .map(|animation_data_t| {
                let mut args = AnimationDataArgs::default();
                args.name = Some(fb.create_shared_string(&animation_data_t.name));
                args.initial_data = None; // TODO; impl
                args.part_animation_data = None; // TODO; impl
                args.part_mesh_data = None; // TODO: impl
                args.events = None; // TODO; impl
                args.label_data = None; // TODO; impl
                args.start_frames = animation_data_t.start_frames;
                args.end_frames = animation_data_t.end_frames;
                args.total_frames = animation_data_t.total_frames;
                args.fps = animation_data_t.fps;
                args.canvas = animation_data_t
                    .canvas
                    .as_ref()
                    .map(|c| self.create_shared_rect(fb, c));

                AnimationData::create(fb, &args)
            })
            .collect::<Vec<WIPOffset<AnimationData<'bldr>>>>();

        fb.create_vector(&animation_data_vec)
    }

    fn create_shared_cell_map<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        cell_map_t_vec: &[CellMapT],
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<CellMap<'bldr>>>> {
        let cell_map = cell_map_t_vec
            .iter()
            .map(|cell_map_t| {
                let mut args = CellMapArgs::default();
                args.name = Some(fb.create_shared_string(&cell_map_t.name));
                args.image_path = cell_map_t
                    .image_path
                    .as_ref()
                    .map(|s| fb.create_shared_string(s));
                args.size_original = cell_map_t
                    .size_original
                    .as_ref()
                    .map(|s| self.create_shared_vec2(fb, s));
                args.wrap_mode = cell_map_t.wrap_mode;
                args.filter_mode = cell_map_t.filter_mode;
                args.cells = cell_map_t.cells.as_ref().map(|c| {
                    let a = c
                        .iter()
                        .map(|cell| self.create_shared_cell(fb, cell))
                        .collect::<Vec<WIPOffset<Cell<'bldr>>>>();
                    fb.create_vector(&a)
                });

                CellMap::create(fb, &args)
            })
            .collect::<Vec<WIPOffset<CellMap<'bldr>>>>();

        fb.create_vector(&cell_map)
    }

    fn create_shared_table_data<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        table_data_t_vec: &[SoundFileT],
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<SoundFile<'bldr>>>> {
        let table_data = table_data_t_vec
            .iter()
            .map(|table_data_t| {
                let mut sound_file_args = SoundFileArgs::default();
                sound_file_args.name = Some(fb.create_shared_string(&table_data_t.name));
                sound_file_args.time_total = table_data_t.time_total;

                SoundFile::create(fb, &sound_file_args)
            })
            .collect::<Vec<WIPOffset<SoundFile<'bldr>>>>();

        fb.create_vector(&table_data)
    }

    fn create_shared_effect_list<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        effet_list_t_vec: &[EffectT],
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<Effect<'bldr>>>> {
        let sound_list = effet_list_t_vec
            .iter()
            .map(|effect_t| {
                let mut effect_args = EffectArgs::default();
                effect_args.name = Some(fb.create_shared_string(&effect_t.name));
                effect_args.fps = effect_t.fps;
                effect_args.is_lock_rand_seed = effect_t.is_lock_rand_seed;
                effect_args.lock_rand_seed = effect_t.lock_rand_seed;
                effect_args.layout_scale = effect_t
                    .layout_scale
                    .as_ref()
                    .map(|v| self.create_shared_vec2(fb, v));
                // TODO: impl
                // effect_args.effect_node: Option<Vec<EffectNodeT>>,

                Effect::create(fb, &effect_args)
            })
            .collect::<Vec<WIPOffset<Effect<'bldr>>>>();

        fb.create_vector(&sound_list)
    }

    fn create_shared_sound_list<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        sound_list_t_vec: &[SoundListT],
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<SoundList<'bldr>>>> {
        let sound_list = sound_list_t_vec
            .iter()
            .map(|sound_list_t| {
                let mut sound_list_args = SoundListArgs::default();
                sound_list_args.name = Some(fb.create_shared_string(&sound_list_t.name));
                //sound_list_args.table_data = self.create_shared_table_data(fb, &sound_list_t.table_data);
                sound_list_args.table_data = sound_list_t
                    .table_data
                    .as_ref()
                    .map(|table_data| self.create_shared_table_data(fb, table_data));

                SoundList::create(fb, &sound_list_args)
            })
            .collect::<Vec<WIPOffset<SoundList<'bldr>>>>();

        fb.create_vector(&sound_list)
    }

    fn create_shared_font_bitmap<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        font_bitmap_t_vec: &[FontBitmapT],
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<FontBitmap<'bldr>>>> {
        let font_bitmap = font_bitmap_t_vec
            .iter()
            .map(|font_bitmap_t| {
                let mut font_bitmap_args = FontBitmapArgs::default();
                font_bitmap_args.name = Some(fb.create_shared_string(&font_bitmap_t.name));
                font_bitmap_args.name_face = font_bitmap_t
                    .name_face
                    .as_ref()
                    .map(|s| fb.create_shared_string(s));
                font_bitmap_args.size = font_bitmap_t.size;
                font_bitmap_args.padding_l = font_bitmap_t.padding_l;
                font_bitmap_args.padding_r = font_bitmap_t.padding_r;
                font_bitmap_args.padding_u = font_bitmap_t.padding_u;
                font_bitmap_args.padding_d = font_bitmap_t.padding_d;
                font_bitmap_args.base = font_bitmap_t.base;
                font_bitmap_args.height_line = font_bitmap_t.height_line;
                font_bitmap_args.scale = font_bitmap_t
                    .scale
                    .as_ref()
                    .map(|s| self.create_shared_vec2(fb, s));
                font_bitmap_args.spacing = font_bitmap_t
                    .spacing
                    .as_ref()
                    .map(|s| self.create_shared_vec2(fb, s));
                font_bitmap_args.size_texture_original = font_bitmap_t
                    .size_texture_original
                    .as_ref()
                    .map(|s| self.create_shared_vec2(fb, s));
                font_bitmap_args.name_texture = font_bitmap_t.name_texture.as_ref().map(|s| {
                    let a = s
                        .iter()
                        .map(|s| fb.create_shared_string(s))
                        .collect::<Vec<_>>();
                    fb.create_vector(&a)
                });
                font_bitmap_args.table_data = None;

                FontBitmap::create(fb, &font_bitmap_args)
            })
            .collect::<Vec<WIPOffset<FontBitmap<'bldr>>>>();

        fb.create_vector(&font_bitmap)
    }

    pub fn create_shared_anime_binary<
        'args: 'mut_bldr,
        'mut_bldr,
        A: flatbuffers::Allocator + 'bldr,
    >(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        ss_anime_binary_t: &SsAnimeBinaryT,
    ) -> WIPOffset<SsAnimeBinary<'bldr>> {
        let mut anime_binary_args = SsAnimeBinaryArgs::default();

        anime_binary_args.animations = ss_anime_binary_t
            .animations
            .as_ref()
            .map(|animations| self.create_animation_data_list(fb, animations));
        anime_binary_args.cell_map = ss_anime_binary_t
            .cell_map
            .as_ref()
            .map(|cell_map| self.create_shared_cell_map(fb, cell_map));
        anime_binary_args.effect = ss_anime_binary_t
            .effect
            .as_ref()
            .map(|effect| self.create_shared_effect_list(fb, effect));
        anime_binary_args.sound_list = ss_anime_binary_t
            .sound_list
            .as_ref()
            .map(|sound_list| self.create_shared_sound_list(fb, sound_list));
        anime_binary_args.font_bitmap = ss_anime_binary_t
            .font_bitmap
            .as_ref()
            .map(|font_bitmap| self.create_shared_font_bitmap(fb, font_bitmap));

        anime_binary_args.instances = None; // TODO: implement

        SsAnimeBinary::create(fb, &anime_binary_args)
    }
}
