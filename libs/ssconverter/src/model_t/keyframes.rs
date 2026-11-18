//! The converter's keyframes, and the `Keyframes` columns they are written into.
//!
//! The attribute converters in `animedata` build a `Vec` of [`Key`]s per attribute
//! per part ([`PartKeys`]). [`KeyframesBuilder`] then lays every key of the pack
//! into one set of columns, gives each part the run that holds each attribute's
//! keys, and stores each distinct curve and compound value once.

use crate::cache::ssab_cache::{PartAttributeCellKey, PartAttributeDeformKey, PartAttributePartColorKey, PartAttributeShaderKey, PartAttributeVertexKey, Vec2Key};
use ssab::ssab::ss::format::{InterpolationType, KeyCurveT, KeyframesT, PartAnimationDataT, PartAttributeCellT, PartAttributeDeformT, PartAttributePartColorT, PartAttributeShaderT, PartAttributeVertexT, Vec2T};
use ssab::{NO_CURVE, NO_KEYS};
use std::collections::HashMap;
use std::hash::Hash;

/// One keyframe before it is laid out. `curve` is read for Hermite and Bezier,
/// `easing_rate` for the ease types that take a rate, and neither otherwise.
#[derive(Clone, Default, PartialEq)]
pub struct Key<V> {
    pub index: u16,
    pub interpolation: InterpolationType,
    pub value: V,
    pub curve: Option<KeyCurveT>,
    pub easing_rate: f32,
}

pub type FloatKey = Key<f32>;
pub type IntKey = Key<i32>;
pub type BoolKey = Key<bool>;
pub type TextureKey = Key<u32>;
pub type CellKey = Key<Box<PartAttributeCellT>>;
pub type PartColorKey = Key<Box<PartAttributePartColorT>>;
pub type ShaderKey = Key<Box<PartAttributeShaderT>>;
pub type VertexKey = Key<Box<PartAttributeVertexT>>;
pub type SkewKey = Key<Box<Vec2T>>;
pub type DeformKey = Key<Box<PartAttributeDeformT>>;

/// One part's keys in one animation, an attribute at a time, in frame order.
#[derive(Clone, Default)]
pub struct PartKeys {
    pub cell_array: Option<Vec<CellKey>>,
    pub position_x_array: Option<Vec<FloatKey>>,
    pub position_y_array: Option<Vec<FloatKey>>,
    pub position_z_array: Option<Vec<FloatKey>>,
    pub rotation_x_array: Option<Vec<FloatKey>>,
    pub rotation_y_array: Option<Vec<FloatKey>>,
    pub rotation_z_array: Option<Vec<FloatKey>>,
    pub scale_x_array: Option<Vec<FloatKey>>,
    pub scale_y_array: Option<Vec<FloatKey>>,
    pub local_scale_x_array: Option<Vec<FloatKey>>,
    pub local_scale_y_array: Option<Vec<FloatKey>>,
    pub alpha_array: Option<Vec<FloatKey>>,
    pub local_alpha_array: Option<Vec<FloatKey>>,
    pub priority_array: Option<Vec<IntKey>>,
    pub flip_h_array: Option<Vec<BoolKey>>,
    pub flip_v_array: Option<Vec<BoolKey>>,
    pub hide_array: Option<Vec<BoolKey>>,
    pub part_color_array: Option<Vec<PartColorKey>>,
    pub shader_array: Option<Vec<ShaderKey>>,
    pub vertex_array: Option<Vec<VertexKey>>,
    pub pivot_x_array: Option<Vec<FloatKey>>,
    pub pivot_y_array: Option<Vec<FloatKey>>,
    pub anchor_x_array: Option<Vec<FloatKey>>,
    pub anchor_y_array: Option<Vec<FloatKey>>,
    pub size_x_array: Option<Vec<FloatKey>>,
    pub size_y_array: Option<Vec<FloatKey>>,
    pub img_flip_h_array: Option<Vec<BoolKey>>,
    pub img_flip_v_array: Option<Vec<BoolKey>>,
    pub uv_translation_x_array: Option<Vec<FloatKey>>,
    pub uv_translation_y_array: Option<Vec<FloatKey>>,
    pub uv_rotation_z_array: Option<Vec<FloatKey>>,
    pub uv_scale_x_array: Option<Vec<FloatKey>>,
    pub uv_scale_y_array: Option<Vec<FloatKey>>,
    pub bounding_radius_array: Option<Vec<FloatKey>>,
    pub mask_array: Option<Vec<FloatKey>>,
    pub skew_array: Option<Vec<SkewKey>>,
    pub deform_array: Option<Vec<DeformKey>>,
    pub texture_array: Option<Vec<TextureKey>>,
}

/// Each distinct item once, addressed by the order it was first seen.
struct Pool<K, T> {
    index: HashMap<K, u32>,
    items: Vec<T>,
}

impl<K, T> Default for Pool<K, T> {
    fn default() -> Self {
        Self { index: HashMap::new(), items: Vec::new() }
    }
}

impl<K: Eq + Hash, T: Clone> Pool<K, T> {
    fn insert(&mut self, key: K, item: &T) -> u32 {
        let next = self.items.len() as u32;
        *self.index.entry(key).or_insert_with(|| {
            self.items.push(item.clone());
            next
        })
    }
}

/// The tables a compound value is stored in, so that `Keyframes.value` can name it.
#[derive(Default)]
pub struct ValuePools {
    cells: Pool<PartAttributeCellKey, PartAttributeCellT>,
    part_colors: Pool<PartAttributePartColorKey, PartAttributePartColorT>,
    shaders: Pool<PartAttributeShaderKey, PartAttributeShaderT>,
    vertices: Pool<PartAttributeVertexKey, PartAttributeVertexT>,
    skews: Pool<Vec2Key, Vec2T>,
    deforms: Pool<PartAttributeDeformKey, PartAttributeDeformT>,
}

/// How a key's value becomes its `Keyframes.value` word.
pub trait KeyValue {
    fn encode(&self, pools: &mut ValuePools) -> u32;
}

impl KeyValue for f32 {
    fn encode(&self, _: &mut ValuePools) -> u32 {
        self.to_bits()
    }
}

impl KeyValue for i32 {
    fn encode(&self, _: &mut ValuePools) -> u32 {
        *self as u32
    }
}

impl KeyValue for bool {
    fn encode(&self, _: &mut ValuePools) -> u32 {
        u32::from(*self)
    }
}

impl KeyValue for u32 {
    fn encode(&self, _: &mut ValuePools) -> u32 {
        *self
    }
}

impl KeyValue for Box<PartAttributeCellT> {
    fn encode(&self, pools: &mut ValuePools) -> u32 {
        pools.cells.insert(PartAttributeCellKey::from(self.as_ref()), self)
    }
}

impl KeyValue for Box<PartAttributePartColorT> {
    fn encode(&self, pools: &mut ValuePools) -> u32 {
        pools.part_colors.insert(PartAttributePartColorKey::from(self.as_ref()), self)
    }
}

impl KeyValue for Box<PartAttributeShaderT> {
    fn encode(&self, pools: &mut ValuePools) -> u32 {
        pools.shaders.insert(PartAttributeShaderKey::from(self.as_ref()), self)
    }
}

impl KeyValue for Box<PartAttributeVertexT> {
    fn encode(&self, pools: &mut ValuePools) -> u32 {
        pools.vertices.insert(PartAttributeVertexKey::from(self.as_ref()), self)
    }
}

impl KeyValue for Box<Vec2T> {
    fn encode(&self, pools: &mut ValuePools) -> u32 {
        pools.skews.insert(Vec2Key::from(self.as_ref()), self)
    }
}

impl KeyValue for Box<PartAttributeDeformT> {
    fn encode(&self, pools: &mut ValuePools) -> u32 {
        pools.deforms.insert(PartAttributeDeformKey::from(self.as_ref()), self)
    }
}

/// Lays the keys of every animation in a pack into one set of `Keyframes` columns.
///
/// A run that repeats one already laid out — the setup pose every animation is
/// seeded with, a part no animation moves — is given the earlier run rather
/// than a copy.
pub struct KeyframesBuilder<'p> {
    pack: &'p str,
    keyframes: KeyframesT,
    runs: HashMap<Vec<(u16, u8, u16, u32)>, u32>,
    curves: Pool<[u32; 4], KeyCurveT>,
    easing_rates: Pool<u32, f32>,
    values: ValuePools,
}

impl<'p> KeyframesBuilder<'p> {
    pub fn new(pack: &'p str) -> Self {
        Self { pack, keyframes: KeyframesT::default(), runs: HashMap::new(), curves: Pool::default(), easing_rates: Pool::default(), values: ValuePools::default() }
    }

    /// Lays out one part's keys and returns the runs that address them.
    pub fn part(&mut self, keys: &PartKeys) -> Result<PartAnimationDataT, crate::ConverterError> {
        let mut part = PartAnimationDataT::default();
        part.cell_keys = self.run(&keys.cell_array)?;
        part.position_x_keys = self.run(&keys.position_x_array)?;
        part.position_y_keys = self.run(&keys.position_y_array)?;
        part.position_z_keys = self.run(&keys.position_z_array)?;
        part.rotation_x_keys = self.run(&keys.rotation_x_array)?;
        part.rotation_y_keys = self.run(&keys.rotation_y_array)?;
        part.rotation_z_keys = self.run(&keys.rotation_z_array)?;
        part.scale_x_keys = self.run(&keys.scale_x_array)?;
        part.scale_y_keys = self.run(&keys.scale_y_array)?;
        part.local_scale_x_keys = self.run(&keys.local_scale_x_array)?;
        part.local_scale_y_keys = self.run(&keys.local_scale_y_array)?;
        part.alpha_keys = self.run(&keys.alpha_array)?;
        part.local_alpha_keys = self.run(&keys.local_alpha_array)?;
        part.priority_keys = self.run(&keys.priority_array)?;
        part.flip_h_keys = self.run(&keys.flip_h_array)?;
        part.flip_v_keys = self.run(&keys.flip_v_array)?;
        part.hide_keys = self.run(&keys.hide_array)?;
        part.part_color_keys = self.run(&keys.part_color_array)?;
        part.shader_keys = self.run(&keys.shader_array)?;
        part.vertex_keys = self.run(&keys.vertex_array)?;
        part.pivot_x_keys = self.run(&keys.pivot_x_array)?;
        part.pivot_y_keys = self.run(&keys.pivot_y_array)?;
        part.anchor_x_keys = self.run(&keys.anchor_x_array)?;
        part.anchor_y_keys = self.run(&keys.anchor_y_array)?;
        part.size_x_keys = self.run(&keys.size_x_array)?;
        part.size_y_keys = self.run(&keys.size_y_array)?;
        part.img_flip_h_keys = self.run(&keys.img_flip_h_array)?;
        part.img_flip_v_keys = self.run(&keys.img_flip_v_array)?;
        part.uv_translation_x_keys = self.run(&keys.uv_translation_x_array)?;
        part.uv_translation_y_keys = self.run(&keys.uv_translation_y_array)?;
        part.uv_rotation_z_keys = self.run(&keys.uv_rotation_z_array)?;
        part.uv_scale_x_keys = self.run(&keys.uv_scale_x_array)?;
        part.uv_scale_y_keys = self.run(&keys.uv_scale_y_array)?;
        part.bounding_radius_keys = self.run(&keys.bounding_radius_array)?;
        part.mask_keys = self.run(&keys.mask_array)?;
        part.skew_keys = self.run(&keys.skew_array)?;
        part.deform_keys = self.run(&keys.deform_array)?;
        part.texture_keys = self.run(&keys.texture_array)?;
        Ok(part)
    }

    pub fn finish(self) -> KeyframesT {
        let mut keyframes = self.keyframes;
        keyframes.runs.push(keyframes.frame.len() as u32);
        keyframes.curves = self.curves.items;
        keyframes.easing_rates = self.easing_rates.items;
        keyframes.cells = self.values.cells.items;
        keyframes.part_colors = self.values.part_colors.items;
        keyframes.shaders = self.values.shaders.items;
        keyframes.vertices = self.values.vertices.items;
        keyframes.skews = self.values.skews.items;
        keyframes.deforms = self.values.deforms.items;
        keyframes
    }

    fn run<V: KeyValue>(&mut self, keys: &Option<Vec<Key<V>>>) -> Result<u32, crate::ConverterError> {
        let Some(keys) = keys else { return Ok(NO_KEYS) };
        let mut encoded = Vec::with_capacity(keys.len());
        for key in keys {
            let curve = self.curve(key)?;
            encoded.push((key.index, key.interpolation.0, curve, key.value.encode(&mut self.values)));
        }
        let columns = &mut self.keyframes;
        Ok(*self.runs.entry(encoded).or_insert_with_key(|encoded| {
            columns.runs.push(columns.frame.len() as u32);
            for &(frame, interpolation, curve, value) in encoded {
                columns.frame.push(frame);
                columns.interpolation.push(InterpolationType(interpolation));
                columns.curve.push(curve);
                columns.value.push(value);
            }
            (columns.runs.len() - 1) as u32
        }))
    }

    fn curve<V>(&mut self, key: &Key<V>) -> Result<u16, crate::ConverterError> {
        let index = match key.interpolation {
            InterpolationType::Hermite | InterpolationType::Bezier => match &key.curve {
                Some(c) => self.curves.insert([c.start_time.to_bits(), c.start_value.to_bits(), c.end_time.to_bits(), c.end_value.to_bits()], c),
                None => return Ok(NO_CURVE),
            },
            InterpolationType::EaseIn | InterpolationType::EaseOut | InterpolationType::EaseInOut | InterpolationType::EaseElasticIn | InterpolationType::EaseElasticOut | InterpolationType::EaseElasticInOut => {
                self.easing_rates.insert(key.easing_rate.to_bits(), &key.easing_rate)
            }
            _ => return Ok(NO_CURVE),
        };
        u16::try_from(index).ok().filter(|&index| index != NO_CURVE).ok_or_else(|| crate::ConverterError::ConversionFailed(format!("{}: more than {} distinct curves, or easing rates, in one pack", self.pack, NO_CURVE)))
    }
}
