//! Read side of `SsAnimeBinary.keyframes`: every key of the pack, one column per
//! field, and the tables its compound values live in.

use crate::core::framedata::ResolvedCellMeta;
use crate::runtime;
use flatbuffers::{ForwardsUOffset, Vector};
use ssab::ssab::ss::format::{CellMap, InterpolationType, KeyCurve, Keyframes, PartAttributeCell, PartAttributeDeform};

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

/// A cell a key shows, with what it resolves to in the pack's cellmaps.
#[derive(Clone, Copy)]
pub struct CellValue {
    pub cell: runtime::framedata::ss::runtime::PartAttributeCell,
    /// All zeros when the pack's cellmaps do not hold the cell.
    pub meta: ResolvedCellMeta,
}

/// The pack's keyframe columns, looked up once when the pack is loaded. The
/// fixed-size compound values its keys share are decoded then too, cells
/// resolved against the cellmaps, so every animation reads a key's value by
/// index instead of decoding it again. Deforms vary in length and are read
/// from the pack as they are.
pub struct KeyColumns<'a> {
    // The columns every frame reads, as the little-endian bytes FlatBuffers
    // stores them as: indexing one is a slice access, where `Vector::get` reads
    // the length back out of the buffer on every call.
    runs: &'a [U32Bytes],
    frame: &'a [U16Bytes],
    interpolation: &'a [u8],
    curve: &'a [U16Bytes],
    value: &'a [U32Bytes],
    curves: Vector<'a, KeyCurve>,
    easing_rates: Vector<'a, f32>,
    cells: alloc::boxed::Box<[CellValue]>,
    part_colors: alloc::boxed::Box<[runtime::framedata::ss::runtime::PartAttributePartColor]>,
    shaders: alloc::boxed::Box<[runtime::framedata::ss::runtime::PartAttributeShader]>,
    vertices: alloc::boxed::Box<[runtime::framedata::ss::runtime::PartAttributeVertex]>,
    /// `(x, y)`.
    skews: alloc::boxed::Box<[(f32, f32)]>,
    deforms: Vector<'a, ForwardsUOffset<PartAttributeDeform<'a>>>,
}

impl<'a> KeyColumns<'a> {
    /// `cellmaps` are the pack's, which its cells resolve against.
    pub fn new(keyframes: Keyframes<'a>, cellmaps: Option<Vector<'a, ForwardsUOffset<CellMap<'a>>>>) -> Self {
        let cell = |c: PartAttributeCell| CellValue {
            cell: runtime::framedata::ss::runtime::PartAttributeCell::new(c.cellmap_name_hash(), c.name_hash()),
            meta: cellmaps.as_ref().and_then(|maps| crate::core::framedata::resolve_cell(maps, c.cellmap_name_hash(), c.name_hash())).unwrap_or_default(),
        };
        Self {
            runs: u32s(keyframes.runs()),
            frame: u16s(keyframes.frame()),
            interpolation: keyframes.interpolation().bytes(),
            curve: u16s(keyframes.curve()),
            value: u32s(keyframes.value()),
            curves: keyframes.curves(),
            easing_rates: keyframes.easing_rates(),
            cells: keyframes.cells().iter().map(cell).collect(),
            part_colors: keyframes.part_colors().iter().map(|c| transfer_partcolor(&c)).collect(),
            shaders: keyframes.shaders().iter().map(|s| transfer_shader(&s)).collect(),
            vertices: keyframes.vertices().iter().map(|v| transfer_vertex(&v)).collect(),
            skews: keyframes.skews().iter().map(|s| (s.v1(), s.v2())).collect(),
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
        let at = |i: usize| u32::from_le_bytes(self.runs[i]);
        let first = at(r as usize);
        Some(KeyRun::new(first, at(r as usize + 1) - first))
    }

    /// The frames `run`'s keys sit on.
    #[inline]
    pub fn frames(&self, run: &KeyRun) -> Frames<'a> {
        Frames(&self.frame[run.first() as usize..][..run.count() as usize])
    }

    /// Key `k` of the pack.
    #[inline]
    pub fn key(&self, k: usize) -> Key<'_, 'a> {
        Key { columns: self, k }
    }

    /// The cell a cell key's [`Key::cell`] names.
    #[inline]
    pub fn cell(&self, cell: u32) -> &CellValue {
        &self.cells[cell as usize]
    }
}

/// One key of a [`KeyColumns`]. Which value accessor applies is the attribute's
/// to say: the run a key belongs to is read as one type throughout.
#[derive(Clone, Copy)]
pub struct Key<'c, 'a> {
    columns: &'c KeyColumns<'a>,
    k: usize,
}

impl<'c, 'a> Key<'c, 'a> {
    /// The frame the key sits on.
    #[inline]
    pub fn index(&self) -> u16 {
        u16::from_le_bytes(self.columns.frame[self.k])
    }

    #[inline]
    pub fn interpolation(&self) -> InterpolationType {
        InterpolationType(self.columns.interpolation[self.k])
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
        let slot = u16::from_le_bytes(self.columns.curve[self.k]);
        (slot != NO_CURVE).then_some(slot as usize)
    }

    #[inline]
    fn bits(&self) -> u32 {
        u32::from_le_bytes(self.columns.value[self.k])
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

    /// The cell the key shows, as an index for [`KeyColumns::cell`].
    #[inline]
    pub fn cell(&self) -> u32 {
        self.bits()
    }

    #[inline]
    pub fn part_color(&self) -> &'c runtime::framedata::ss::runtime::PartAttributePartColor {
        &self.columns.part_colors[self.bits() as usize]
    }

    #[inline]
    pub fn shader(&self) -> &'c runtime::framedata::ss::runtime::PartAttributeShader {
        &self.columns.shaders[self.bits() as usize]
    }

    #[inline]
    pub fn vertex(&self) -> &'c runtime::framedata::ss::runtime::PartAttributeVertex {
        &self.columns.vertices[self.bits() as usize]
    }

    #[inline]
    pub fn skew(&self) -> (f32, f32) {
        self.columns.skews[self.bits() as usize]
    }

    #[inline]
    pub fn deform(&self) -> PartAttributeDeform<'a> {
        self.columns.deforms.get(self.bits() as usize)
    }
}

/// The frames a run's keys sit on, in order, as [`KeyColumns::frames`] gives
/// them: what a key search looks through.
#[derive(Clone, Copy)]
pub struct Frames<'a>(&'a [U16Bytes]);

impl Frames<'_> {
    /// How many keys [`locate`](Self::locate) steps from the hint, either way,
    /// before it searches instead.
    const NEAR: usize = 4;

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    fn at(&self, i: usize) -> f32 {
        u16::from_le_bytes(self.0[i]) as f32
    }

    /// Where `frame_no` falls: the position of the last key at or before it, or
    /// `None` before the first. The answer depends on `frame_no` alone, never on
    /// `hint`, which is only where to look first, counted from the run's first
    /// key. It is left on the position found, so playback that moves a few keys
    /// either way settles within a few steps of it, and a longer seek falls back
    /// to a binary search.
    #[inline]
    pub fn locate(&self, frame_no: f32, hint: &mut u32) -> Option<usize> {
        let found = self.near(frame_no, *hint as usize).unwrap_or_else(|| self.0.partition_point(|f| u16::from_le_bytes(*f) as f32 <= frame_no).checked_sub(1));
        *hint = found.unwrap_or(0) as u32;
        found
    }

    /// [`locate`](Self::locate)'s answer when it lies within a few keys of
    /// `hint`, or `None` to search for it.
    #[inline]
    fn near(&self, frame_no: f32, hint: usize) -> Option<Option<usize>> {
        let last = self.len().checked_sub(1)?;
        let mut i = hint.min(last);
        if self.at(i) <= frame_no {
            // Forward playback: the key is the hint's or a few after it.
            for _ in 0..=Self::NEAR {
                if i == last || frame_no < self.at(i + 1) {
                    return Some(Some(i));
                }
                i += 1;
            }
        } else {
            // Reverse playback, or a loop back to the start.
            for _ in 0..Self::NEAR {
                if i == 0 {
                    return Some(None);
                }
                i -= 1;
                if self.at(i) <= frame_no {
                    return Some(Some(i));
                }
            }
        }
        None
    }
}

/// A `u16` as FlatBuffers stores it.
type U16Bytes = [u8; size_of::<u16>()];

/// A `u32` as FlatBuffers stores it.
type U32Bytes = [u8; size_of::<u32>()];

// A column's element type is spelled in each of these, so a schema change to it
// stops the build here rather than misreading the bytes.

fn u16s(column: Vector<'_, u16>) -> &[U16Bytes] {
    column.bytes().as_chunks().0
}

fn u32s(column: Vector<'_, u32>) -> &[U32Bytes] {
    column.bytes().as_chunks().0
}

// The runtime form of each compound value, which `KeyColumns::new` decodes the
// pack's into once. A corner the pack leaves out reads as zeros.

#[inline]
fn transfer_blend_type(bt: ssab::ssab::ss::format::BlendType) -> runtime::framedata::ss::runtime::BlendType {
    match bt {
        ssab::ssab::ss::format::BlendType::Mix => runtime::framedata::ss::runtime::BlendType::Mix,
        ssab::ssab::ss::format::BlendType::Mul => runtime::framedata::ss::runtime::BlendType::Mul,
        ssab::ssab::ss::format::BlendType::Add => runtime::framedata::ss::runtime::BlendType::Add,
        ssab::ssab::ss::format::BlendType::Sub => runtime::framedata::ss::runtime::BlendType::Sub,
        ssab::ssab::ss::format::BlendType::Mulalpha => runtime::framedata::ss::runtime::BlendType::Mulalpha,
        ssab::ssab::ss::format::BlendType::Screen => runtime::framedata::ss::runtime::BlendType::Screen,
        ssab::ssab::ss::format::BlendType::Exclusion => runtime::framedata::ss::runtime::BlendType::Exclusion,
        ssab::ssab::ss::format::BlendType::Invert => runtime::framedata::ss::runtime::BlendType::Invert,
        ssab::ssab::ss::format::BlendType::Mul2 => runtime::framedata::ss::runtime::BlendType::Mul2,
        ssab::ssab::ss::format::BlendType::Div2 => runtime::framedata::ss::runtime::BlendType::Div2,
        ssab::ssab::ss::format::BlendType::Screen2 => runtime::framedata::ss::runtime::BlendType::Screen2,
        ssab::ssab::ss::format::BlendType::Overlay2 => runtime::framedata::ss::runtime::BlendType::Overlay2,
        _ => runtime::framedata::ss::runtime::BlendType::Mix,
    }
}

#[inline]
fn transfer_partcolor(source: &ssab::ssab::ss::format::PartAttributePartColor) -> runtime::framedata::ss::runtime::PartAttributePartColor {
    let mut destination = runtime::framedata::ss::runtime::PartAttributePartColor::default();
    let s = source;

    destination.set_blend_type(transfer_blend_type(s.blend_type()));

    fn convert_u8color(rgba: &ssab::ssab::ss::format::U8Color) -> runtime::framedata::ss::runtime::U8Color {
        runtime::framedata::ss::runtime::U8Color::new(rgba.a(), rgba.r(), rgba.g(), rgba.b())
    }

    if let Some(lt) = s.lt()
        && let Some(rgba) = lt.rgba()
    {
        destination.set_lt(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8color(&rgba), lt.rate()));
    }
    if let Some(rt) = s.rt()
        && let Some(rgba) = rt.rgba()
    {
        destination.set_rt(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8color(&rgba), rt.rate()));
    }
    if let Some(rb) = s.rb()
        && let Some(rgba) = rb.rgba()
    {
        destination.set_rb(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8color(&rgba), rb.rate()));
    }
    if let Some(lb) = s.lb()
        && let Some(rgba) = lb.rgba()
    {
        destination.set_lb(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8color(&rgba), lb.rate()));
    }

    destination
}

#[inline]
fn transfer_shader(source: &ssab::ssab::ss::format::PartAttributeShader) -> runtime::framedata::ss::runtime::PartAttributeShader {
    let s = source;

    runtime::framedata::ss::runtime::PartAttributeShader::new(
        s.id_hash(),
        s.param0(),
        s.param1(),
        s.param2(),
        s.param3(),
        s.param4(),
        s.param5(),
        s.param6(),
        s.param7(),
        s.map0_cellmap_name_hash(),
        s.map0_name_hash(),
        s.map1_cellmap_name_hash(),
        s.map1_name_hash(),
    )
}

#[inline]
fn transfer_vertex(source: &ssab::ssab::ss::format::PartAttributeVertex) -> runtime::framedata::ss::runtime::PartAttributeVertex {
    let s = source;

    let lt = s.lt().map(|lt| runtime::framedata::ss::runtime::Vec2::new(lt.v1(), lt.v2())).unwrap_or_default();
    let rt = s.rt().map(|rt| runtime::framedata::ss::runtime::Vec2::new(rt.v1(), rt.v2())).unwrap_or_default();
    let lb = s.lb().map(|lb| runtime::framedata::ss::runtime::Vec2::new(lb.v1(), lb.v2())).unwrap_or_default();
    let rb = s.rb().map(|rb| runtime::framedata::ss::runtime::Vec2::new(rb.v1(), rb.v2())).unwrap_or_default();

    // The center vertex is filled later by get_parts after pivot/size/flip/deform are applied.
    runtime::framedata::ss::runtime::PartAttributeVertex::new(&lt, &rt, &lb, &rb, &runtime::framedata::ss::runtime::Vec2::default())
}

/// A `Keyframes` table with every column present and empty, for tests that
/// need one but read no key.
#[cfg(test)]
pub(crate) fn create_empty_keyframes<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>) -> flatbuffers::WIPOffset<Keyframes<'a>> {
    use flatbuffers::WIPOffset;
    use ssab::ssab::ss::format::{PartAttributePartColor, PartAttributeShader, PartAttributeVertex, Vec2};
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
