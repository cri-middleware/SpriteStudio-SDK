use std::collections::HashMap;
use flatbuffers::{ForwardsUOffset, Vector, WIPOffset};
use ordered_float::OrderedFloat;
use crate::ssab::{CellMap, CellMapArgs, CellMapT, SsAnimeBinary, SsAnimeBinaryArgs, SsAnimeBinaryT, Vec2, Vec2Args, Vec2T, Vec3, Vec3Args, Vec3T};

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

pub struct SsabCache<'bldr> {
    pub vec2t_cache: HashMap<Vec2Key, WIPOffset<Vec2<'bldr>>>,
    pub vec3t_cache: HashMap<Vec3Key, WIPOffset<Vec3<'bldr>>>,
}
impl<'bldr> SsabCache<'bldr> {
    pub fn new() -> Self {
        Self {
            vec2t_cache: HashMap::new(),
            vec3t_cache: HashMap::new(),
        }
    }
    fn create_shared_vec2<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        vec2t: &Vec2T
    ) -> WIPOffset<Vec2<'bldr>>
    {
        if let Some(&offset) = self.vec2t_cache.get(&Vec2Key::from(vec2t)) {
            return offset;
        }
        let mut vec2_args = Vec2Args::default();
        vec2_args.x = vec2t.x;
        vec2_args.y = vec2t.y;
        let offset = Vec2::create(fb, &vec2_args);
        self.vec2t_cache.insert(Vec2Key::from(vec2t), offset);
        offset
    }

    fn create_shared_vec3<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        vec3t: &Vec3T
    ) -> WIPOffset<Vec3<'bldr>>
    {
        if let Some(&offset) = self.vec3t_cache.get(&Vec3Key::from(vec3t)) {
            return offset;
        }
        let mut vec3_args = Vec3Args::default();
        vec3_args.x = vec3t.x;
        vec3_args.y = vec3t.y;
        vec3_args.z = vec3t.z;
        let offset = Vec3::create(fb, &vec3_args);
        self.vec3t_cache.insert(Vec3Key::from(vec3t), offset);
        offset
    }
}

impl<'bldr> SsabCache<'bldr> {
    fn create_shared_cell_map<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        cell_map: &Vec<CellMapT>
    ) -> WIPOffset<Vector<'bldr, ForwardsUOffset<CellMap<'bldr>>>> {
        let a = cell_map.iter().map(|cell_map_t| {
            let mut cell_map_args = CellMapArgs::default();
            cell_map_args.name = Some(fb.create_shared_string(&cell_map_t.name));
            cell_map_args.image_path = cell_map_t.image_path.as_ref().map(|s| fb.create_shared_string(s));
            cell_map_args.size_original = cell_map_t.size_original.as_ref().map(|s| self.create_shared_vec2(fb, s));
            cell_map_args.wrap_mode = cell_map_t.wrap_mode;
            cell_map_args.filter_mode = cell_map_t.filter_mode;
            cell_map_args.cells = None; // TODO: implement

            CellMap::create(fb, &cell_map_args)
        }).collect::<Vec<WIPOffset<CellMap<'bldr>>>>();

        fb.create_vector(&a)
    }

    pub fn create_shared_anime_binary<'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        &mut self,
        fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        ss_anime_binary_t: &SsAnimeBinaryT
    ) -> WIPOffset<SsAnimeBinary<'bldr>> {
        let mut anime_binary_args = SsAnimeBinaryArgs::default();

        anime_binary_args.animation_pack = None; // TODO: implement
        anime_binary_args.cell_map = ss_anime_binary_t.cell_map.as_ref().map(|cell_map| self.create_shared_cell_map(fb, cell_map));
        anime_binary_args.effect = None;
        anime_binary_args.sound_list = None;
        anime_binary_args.font_bitmap = None;
        anime_binary_args.dependencies = None;

        SsAnimeBinary::create(fb, &anime_binary_args)
    }
}

