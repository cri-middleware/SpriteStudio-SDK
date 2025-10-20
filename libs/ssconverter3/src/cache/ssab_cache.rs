use std::collections::HashMap;
use flatbuffers::{ForwardsUOffset, Vector, WIPOffset};
use ordered_float::OrderedFloat;
use ssab::ssab::{SsAnimeBinary, SsAnimeBinaryArgs, SsAnimeBinaryT};
use ssab::ssab::{Rect, RectArgs, RectT, Vec2, Vec2Args, Vec2T, Vec3, Vec3Args, Vec3T, Cell, CellArgs, CellMap, CellMapArgs, CellMapT, CellT, FontBitmap, FontBitmapArgs, FontBitmapT, SoundFile, SoundFileArgs, SoundFileT, SoundList, SoundListArgs, SoundListT, };

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec2Key {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
}
impl From<&Vec2T> for Vec2Key {
    fn from(v: &Vec2T) -> Self {
        Self {
            x: OrderedFloat(v.x),
            y: OrderedFloat(v.y),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec3Key {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
    pub z: OrderedFloat<f32>,
}
impl From<&Vec3T> for Vec3Key {
    fn from(value: &Vec3T) -> Self {
        Self {
            x: OrderedFloat(value.x),
            y: OrderedFloat(value.y),
            z: OrderedFloat(value.z),
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
    pub vec2t_cache: HashMap<Vec2Key, WIPOffset<Vec2<'bldr>>>,
    pub vec3t_cache: HashMap<Vec3Key, WIPOffset<Vec3<'bldr>>>,
    pub rectt_cache: HashMap<RectKey, WIPOffset<Rect<'bldr>>>,

    pub cell_cache: HashMap<CellKey, WIPOffset<Cell<'bldr>>>,
}
impl<'bldr> SsabCache<'bldr> {
    pub fn new() -> Self {
        Self {
            vec2t_cache: HashMap::new(),
            vec3t_cache: HashMap::new(),
            rectt_cache: HashMap::new(),
            
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
        if let Some(&offset) = self.vec2t_cache.get(&key) {
            return offset;
        }
        let mut vec2_args = Vec2Args::default();
        vec2_args.x = vec2t.x;
        vec2_args.y = vec2t.y;
        let offset = Vec2::create(fb, &vec2_args);
        self.vec2t_cache.insert(key, offset);
        offset
    }

    fn create_shared_vec3<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        vec3t: &Vec3T
    ) -> WIPOffset<Vec3<'bldr>>
    {
        let key = Vec3Key::from(vec3t);
        if let Some(&offset) = self.vec3t_cache.get(&key) {
            return offset;
        }
        let mut vec3_args = Vec3Args::default();
        vec3_args.x = vec3t.x;
        vec3_args.y = vec3t.y;
        vec3_args.z = vec3t.z;
        let offset = Vec3::create(fb, &vec3_args);
        self.vec3t_cache.insert(key, offset);
        offset
    }

    fn create_shared_rectt<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        rectt: &RectT,
    ) -> WIPOffset<Rect<'bldr>>
    {
        let key = RectKey::from(rectt);
        if let Some(&offset) = self.rectt_cache.get(&key) {
            return offset;
        }
        let mut rect_args = RectArgs::default();
        rect_args.x1 = rectt.x1;
        rect_args.y1 = rectt.y1;
        rect_args.x2 = rectt.x2;
        rect_args.y2 = rectt.y2;
        let offset = Rect::create(fb, &rect_args);
        self.rectt_cache.insert(key, offset);
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

        let mut cell_args = CellArgs::default();
        cell_args.name = Some(fb.create_shared_string(&cell.name));
        cell_args.rectangle = cell.rectangle.as_ref().map(|r|self.create_shared_rectt(fb, r));
        cell_args.pivot = cell.pivot.as_ref().map(|p| self.create_shared_vec2(fb, p));
        cell_args.rotated = cell.rotated;
        cell_args.table_coordinate = cell.table_coordinate.as_ref().map(|p| {
            let vec = p.iter().map(|v| self.create_shared_vec2(fb, v)).collect::<Vec<WIPOffset<Vec2<'bldr>>>>();
            fb.create_vector(&vec)
        });
        cell_args.table_index_vertex = cell.table_index_vertex.as_ref().map(|p| {
            fb.create_vector(&p)
        });

        let offset = Cell::create(fb, &cell_args);
        self.cell_cache.insert(key, offset);
        offset
    }

    fn create_shared_cell_map<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        cell_map_t_vec: &[CellMapT]
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<CellMap<'bldr>>>> {
        let cell_map = cell_map_t_vec.iter().map(|cell_map_t| {
            let mut cell_map_args = CellMapArgs::default();
            cell_map_args.name = Some(fb.create_shared_string(&cell_map_t.name));
            cell_map_args.image_path = cell_map_t.image_path.as_ref().map(|s| fb.create_shared_string(s));
            cell_map_args.size_original = cell_map_t.size_original.as_ref().map(|s| self.create_shared_vec2(fb, s));
            cell_map_args.wrap_mode = cell_map_t.wrap_mode;
            cell_map_args.filter_mode = cell_map_t.filter_mode;
            cell_map_args.cells = cell_map_t.cells.as_ref().map(|c| {
                let a = c.iter().map(|cell| self.create_shared_cell(fb, cell)).collect::<Vec<WIPOffset<Cell<'bldr>>>>();
                fb.create_vector(&a)
            });

            CellMap::create(fb, &cell_map_args)
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
            sound_file_args.path = table_data_t.path.as_ref().map(|s| fb.create_shared_string(s));
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

        anime_binary_args.animations = None; // TODO: implement
        anime_binary_args.cell_map = ss_anime_binary_t.cell_map.as_ref().map(|cell_map| self.create_shared_cell_map(fb, cell_map));
        anime_binary_args.effect = None;
        anime_binary_args.sound_list = ss_anime_binary_t.sound_list.as_ref().map(|sound_list| self.create_shared_sound_list(fb, sound_list));
        anime_binary_args.font_bitmap = ss_anime_binary_t.font_bitmap.as_ref().map(|font_bitmap| self.create_shared_font_bitmap(fb, font_bitmap));

        anime_binary_args.instances = None;

        SsAnimeBinary::create(fb, &anime_binary_args)
    }
}

