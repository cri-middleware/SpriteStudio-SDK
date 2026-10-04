//! Read side of `SsAnimeBinary.keyframes`: every key of the pack, one column per
//! field, and the tables its compound values live in.

use flatbuffers::{ForwardsUOffset, Vector};
use ssab::ssab::ss::format::{InterpolationType, KeyCurve, Keyframes, PartAttributeCell, PartAttributeDeform, PartAttributePartColor, PartAttributeShader, PartAttributeVertex, Vec2};

use ssab::{NO_CURVE, NO_KEYS};

/// One attribute's keys on one part in one animation: `count` keys from `first`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyRun {
    first: u32,
    count: u32,
}

impl KeyRun {
    pub const fn new(first: u32, count: u32) -> Self {
        Self { first, count }
    }

    #[inline]
    pub fn first(&self) -> u32 {
        self.first
    }

    #[inline]
    pub fn count(&self) -> u32 {
        self.count
    }
}

/// The pack's keyframe columns, looked up once when the pack is loaded.
#[derive(Clone, Copy)]
pub struct KeyColumns<'a> {
    runs: Vector<'a, u32>,
    frame: Vector<'a, u16>,
    interpolation: Vector<'a, InterpolationType>,
    curve: Vector<'a, u16>,
    value: Vector<'a, u32>,
    curves: Vector<'a, KeyCurve>,
    easing_rates: Vector<'a, f32>,
    cells: Vector<'a, ForwardsUOffset<PartAttributeCell<'a>>>,
    part_colors: Vector<'a, ForwardsUOffset<PartAttributePartColor<'a>>>,
    shaders: Vector<'a, ForwardsUOffset<PartAttributeShader<'a>>>,
    vertices: Vector<'a, ForwardsUOffset<PartAttributeVertex<'a>>>,
    skews: Vector<'a, ForwardsUOffset<Vec2<'a>>>,
    deforms: Vector<'a, ForwardsUOffset<PartAttributeDeform<'a>>>,
}

impl<'a> KeyColumns<'a> {
    pub fn new(keyframes: Keyframes<'a>) -> Self {
        Self {
            runs: keyframes.runs(),
            frame: keyframes.frame(),
            interpolation: keyframes.interpolation(),
            curve: keyframes.curve(),
            value: keyframes.value(),
            curves: keyframes.curves(),
            easing_rates: keyframes.easing_rates(),
            cells: keyframes.cells(),
            part_colors: keyframes.part_colors(),
            shaders: keyframes.shaders(),
            vertices: keyframes.vertices(),
            skews: keyframes.skews(),
            deforms: keyframes.deforms(),
        }
    }

    /// The keys of run `r` — a `PartAnimationData` field — or `None` for
    /// [`NO_KEYS`].
    #[inline]
    pub fn run(&self, r: u32) -> Option<KeyRun> {
        if r == NO_KEYS {
            return None;
        }
        let first = self.runs.get(r as usize);
        Some(KeyRun::new(first, self.runs.get(r as usize + 1) - first))
    }

    /// Key `k` of the pack.
    #[inline]
    pub fn key(&self, k: usize) -> Key<'_, 'a> {
        Key { columns: self, k }
    }
}

/// One key of a [`KeyColumns`]. Which value accessor applies is the attribute's
/// to say: the run a key belongs to is read as one type throughout.
#[derive(Clone, Copy)]
pub struct Key<'c, 'a> {
    columns: &'c KeyColumns<'a>,
    k: usize,
}

impl<'a> Key<'_, 'a> {
    /// The frame the key sits on.
    #[inline]
    pub fn index(&self) -> u16 {
        self.columns.frame.get(self.k)
    }

    #[inline]
    pub fn interpolation(&self) -> InterpolationType {
        self.columns.interpolation.get(self.k)
    }

    /// The control values of a Hermite or Bezier key; `None` for any other.
    #[inline]
    pub fn curve(&self) -> Option<&'a KeyCurve> {
        match self.interpolation() {
            InterpolationType::Hermite | InterpolationType::Bezier => self.slot().map(|i| self.columns.curves.get(i)),
            _ => None,
        }
    }

    /// The rate of an ease that takes one; `0.0` for any other key.
    #[inline]
    pub fn easing_rate(&self) -> f32 {
        match self.interpolation() {
            InterpolationType::EaseIn | InterpolationType::EaseOut | InterpolationType::EaseInOut | InterpolationType::EaseElasticIn | InterpolationType::EaseElasticOut | InterpolationType::EaseElasticInOut => {
                self.slot().map_or(0.0, |i| self.columns.easing_rates.get(i))
            }
            _ => 0.0,
        }
    }

    #[inline]
    fn slot(&self) -> Option<usize> {
        let slot = self.columns.curve.get(self.k);
        (slot != NO_CURVE).then_some(slot as usize)
    }

    #[inline]
    fn bits(&self) -> u32 {
        self.columns.value.get(self.k)
    }

    #[inline]
    pub fn float(&self) -> f32 {
        f32::from_bits(self.bits())
    }

    #[inline]
    pub fn int(&self) -> i32 {
        self.bits() as i32
    }

    #[inline]
    pub fn bool(&self) -> bool {
        self.bits() != 0
    }

    #[inline]
    pub fn uint(&self) -> u32 {
        self.bits()
    }

    #[inline]
    pub fn cell(&self) -> PartAttributeCell<'a> {
        self.columns.cells.get(self.bits() as usize)
    }

    #[inline]
    pub fn part_color(&self) -> PartAttributePartColor<'a> {
        self.columns.part_colors.get(self.bits() as usize)
    }

    #[inline]
    pub fn shader(&self) -> PartAttributeShader<'a> {
        self.columns.shaders.get(self.bits() as usize)
    }

    #[inline]
    pub fn vertex(&self) -> PartAttributeVertex<'a> {
        self.columns.vertices.get(self.bits() as usize)
    }

    #[inline]
    pub fn skew(&self) -> Vec2<'a> {
        self.columns.skews.get(self.bits() as usize)
    }

    #[inline]
    pub fn deform(&self) -> PartAttributeDeform<'a> {
        self.columns.deforms.get(self.bits() as usize)
    }
}

/// A `Keyframes` table with every column present and empty, for tests that
/// need one but read no key.
#[cfg(test)]
pub(crate) fn create_empty_keyframes<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>) -> flatbuffers::WIPOffset<Keyframes<'a>> {
    use flatbuffers::WIPOffset;
    let args = ssab::ssab::ss::format::KeyframesArgs {
        runs: Some(fb.create_vector::<u32>(&[0])),
        frame: Some(fb.create_vector::<u16>(&[])),
        interpolation: Some(fb.create_vector::<InterpolationType>(&[])),
        curve: Some(fb.create_vector::<u16>(&[])),
        value: Some(fb.create_vector::<u32>(&[])),
        curves: Some(fb.create_vector::<KeyCurve>(&[])),
        easing_rates: Some(fb.create_vector::<f32>(&[])),
        cells: Some(fb.create_vector::<WIPOffset<PartAttributeCell>>(&[])),
        part_colors: Some(fb.create_vector::<WIPOffset<PartAttributePartColor>>(&[])),
        shaders: Some(fb.create_vector::<WIPOffset<PartAttributeShader>>(&[])),
        vertices: Some(fb.create_vector::<WIPOffset<PartAttributeVertex>>(&[])),
        skews: Some(fb.create_vector::<WIPOffset<Vec2>>(&[])),
        deforms: Some(fb.create_vector::<WIPOffset<PartAttributeDeform>>(&[])),
    };
    Keyframes::create(fb, &args)
}

/// [`create_empty_keyframes`] as a buffer of its own.
#[cfg(test)]
pub(crate) fn empty_keyframes() -> alloc::vec::Vec<u8> {
    let mut fb = flatbuffers::FlatBufferBuilder::new();
    let root = create_empty_keyframes(&mut fb);
    fb.finish(root, None);
    fb.finished_data().to_vec()
}
