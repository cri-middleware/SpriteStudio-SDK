//! The track stack: its sources and tracks, the pose buffers each source is extracted into, and the mix that folds them into one pose.

use alloc::vec::Vec;

use crate::resource::AnimationMetadata;
use crate::{core, runtime};

/// Per-layer pose buffers — the immediate output of `extract_pose`,
/// before any cross-layer composition. SRT / Color / Cell ID / Vertex /
/// Deform / Cursor live here, as do the pose-derived per-part buffers
/// (`cell_meta_per_part`, `cell_texture_hashes`, `local_vertices`) that
/// are written in lockstep with the pose evaluation.
///
/// An instance has one per source, and the resolved one the mix writes
/// (`InstanceState::resolved_layer`). While one source plays alone, its pose
/// is extracted straight into the resolved layer, and its own layer stays
/// unallocated until a mix needs it.
pub(crate) struct LayerState<'a> {
    pub(crate) parts_vec: Vec<crate::runtime::framedata::ss::runtime::PartState>,
    pub(crate) cell_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
    pub(crate) parts_color_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    pub(crate) shader_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    pub(crate) vertex_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    pub(crate) deform_vec: Vec<flatbuffers::WIPOffset<crate::runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
    pub(crate) cursors: Vec<u32>,
    pub(crate) cell_meta_per_part: Vec<f32>,
    pub(crate) cell_texture_hashes: Vec<u32>,
    /// Each part's cell, as an index for `KeyColumns::cell`: the one its UVs sample.
    pub(crate) cell_ids: Vec<Option<u32>>,
    pub(crate) local_vertices: Vec<f32>,
    /// Per-part mesh deform retained from Phase 1 for the Phase 2 skinning pass.
    pub(crate) mesh_deform: Vec<crate::core::framedata::DeformSlot<crate::core::framedata::PartDeformFrame>>,
    pub(crate) raw_vertex_deform: Vec<crate::core::framedata::DeformSlot<crate::core::framedata::RawVertexDeform>>,
    /// Per-part attribute coverage of the animation this layer was extracted
    /// from (`CachedPartData::coverage`), consulted by [`mix_tracks`]. Copied in
    /// after pose extraction; empty means none was recorded (`layer_coverage`).
    pub(crate) coverage_vec: Vec<crate::runtime::framedata::ss::runtime::UpdateAttributeFlags>,
}

impl<'a> LayerState<'a> {
    pub(crate) fn new() -> Self {
        Self {
            parts_vec: Vec::new(),
            cell_vec: Vec::new(),
            parts_color_vec: Vec::new(),
            shader_vec: Vec::new(),
            vertex_vec: Vec::new(),
            deform_vec: Vec::new(),
            cursors: Vec::new(),
            cell_meta_per_part: Vec::new(),
            cell_texture_hashes: Vec::new(),
            cell_ids: Vec::new(),
            local_vertices: Vec::new(),
            mesh_deform: Vec::new(),
            raw_vertex_deform: Vec::new(),
            coverage_vec: Vec::new(),
        }
    }

    pub(crate) fn resize_for_parts(&mut self, num_parts: usize, cursor_c: usize, cell_c: usize, color_c: usize, shader_c: usize, deform_c: usize) {
        self.parts_vec.clear();
        self.parts_vec.reserve(num_parts);
        self.cell_vec.clear();
        self.cell_vec.reserve(cell_c);
        self.parts_color_vec.clear();
        self.parts_color_vec.reserve(color_c);
        self.shader_vec.clear();
        self.shader_vec.reserve(shader_c);
        self.deform_vec.clear();
        self.deform_vec.reserve(deform_c);

        self.cursors.clear();
        self.cursors.resize(cursor_c, 0);
        self.cell_meta_per_part.clear();
        self.cell_meta_per_part.resize(num_parts * 6, 0.0);
        self.cell_texture_hashes.clear();
        self.cell_texture_hashes.resize(num_parts, 0);
        self.cell_ids.clear();
        self.cell_ids.reserve(num_parts);
        self.local_vertices.clear();
        self.local_vertices.resize(num_parts * crate::core::framedata::LOCAL_VERTICES_STRIDE, 0.0);
        self.raw_vertex_deform.clear();
        self.raw_vertex_deform.resize_with(num_parts, || None);
    }

    /// Size the buffers for `metadata`'s animation unless they already are: the
    /// first time a source's own layer is extracted into, or after the layer
    /// last held a different part count.
    pub(crate) fn fit(&mut self, metadata: &AnimationMetadata<'_>) {
        let num_parts = metadata.cached_parts().len();
        let sized = self.raw_vertex_deform.len() == num_parts && self.cell_texture_hashes.len() == num_parts && self.cell_meta_per_part.len() == num_parts * 6 && self.local_vertices.len() == num_parts * crate::core::framedata::LOCAL_VERTICES_STRIDE;
        if !sized {
            let (cell_c, color_c, shader_c, deform_c) = count_attribute_capacities(metadata);
            self.resize_for_parts(num_parts, crate::core::framedata::cursor_count(metadata.cached_parts()), cell_c, color_c, shader_c, deform_c);
        }
    }
}

pub(crate) struct SourceState {
    pub(crate) animation_idx: i32,
    pub(crate) controller: core::playback_controller::PlaybackController,
    /// Which track this source plays on. Sources sharing a track are mixed by
    /// weight; tracks fold onto each other by `TrackState::alpha`.
    pub(crate) track: u16,
    pub(crate) weight: f32,
    /// Minimum weight at which this source's events should fire (only meaningful
    /// for non-primary sources). The default, [`NO_EVENT_THRESHOLD`], keeps
    /// "track 0's animation fires events; other sources never do" — the caller
    /// opts a source in by lowering the threshold to <= its current weight.
    pub(crate) event_threshold: f32,
    /// This source's part of the outgoing side while its track crossfades: its
    /// weight when the fade began, normalised against every source the track
    /// carried then. The fade scales it by `1 - t`, so the outgoing side keeps
    /// its proportions and the track's weights sum to 1 throughout. Only read
    /// while the track's crossfade runs; the next crossfade overwrites it.
    pub(crate) outgoing_share: f32,
}

/// The default event threshold: no weight reaches it, so a source fires only
/// once the host lowers it. It used to be `1.0`, which the weight of a track's
/// lone source meets, so an overlay fired every event it passed.
pub(crate) const NO_EVENT_THRESHOLD: f32 = f32::INFINITY;

impl SourceState {
    pub(crate) fn new() -> Self {
        Self { animation_idx: -1, controller: core::playback_controller::PlaybackController::new(), track: 0, weight: 1.0, event_threshold: NO_EVENT_THRESHOLD, outgoing_share: 0.0 }
    }

    pub(crate) fn is_active(&self) -> bool {
        self.animation_idx != -1 && self.weight > 0.0
    }

    /// Release the slot. It keeps its position so live handles never shift, and
    /// `attach_source` may hand it out again.
    pub(crate) fn deactivate(&mut self) {
        self.controller.reset_setup();
        self.animation_idx = -1;
        self.weight = 0.0;
        self.event_threshold = NO_EVENT_THRESHOLD;
        self.outgoing_share = 0.0;
    }

    /// True for slots that carry an animation, not the released `-1`. Use this
    /// whenever a method needs a valid `metadata`/`controller`-backed source.
    pub(crate) fn has_real_animation(&self) -> bool {
        self.animation_idx >= 0
    }
}

/// How a track folds onto the pose the tracks below it produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackBlend {
    /// `resolved = lerp(below, track_pose, alpha)`. The default.
    Mix,
    /// `resolved = below + alpha * (track_pose - setup_pose)`. For overlays that
    /// offset what is underneath instead of replacing it.
    Additive,
}

/// How the animation laid on a track keeps time with track 0. Sync means the
/// normalised phase: how far through its section an animation stands, from
/// `0.0` at the section start towards `1.0` at its end, so animations of
/// different lengths line up by proportion rather than by frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackSync {
    /// The animation starts from the head of its section and keeps its own
    /// time. The default; right for one-off motions such as a blink.
    Free,
    /// The animation starts at the phase track 0's main animation stands at
    /// when it is laid on, then runs on the shared clock. Aligned once, not
    /// held: lengths or speeds that differ drift apart again.
    Start,
}

/// One track of the stack. Tracks fold in ascending index order onto the setup
/// pose; `alpha` is the strength knob (unlike a source's `weight`, which only
/// decides proportions *within* a track).
#[derive(Debug, Clone, Copy)]
pub(crate) struct TrackState {
    pub(crate) alpha: f32,
    pub(crate) blend: TrackBlend,
    /// Where the animations laid on this track start.
    pub(crate) sync: TrackSync,
    /// How the animations laid on this track are played: the speed, the
    /// frame-rate override, the frame skip, the direction and style. Every source
    /// laid on the track takes it, and changing it reaches the sources already
    /// there, so it survives a crossfade or a `set_track_animation`.
    pub(crate) policy: core::playback_controller::PlaybackPolicy,
    /// How far into a crossfade the outgoing side keeps firing events: it fires
    /// while its share of the track, `1 - t`, is at least this. The default,
    /// [`NO_EVENT_THRESHOLD`], keeps it silent from the moment the fade begins.
    pub(crate) outgoing_event_threshold: f32,
    /// Handle of the source carrying this track's animation, or [`NO_SOURCE`].
    /// Extra sources added for weighted blending sit alongside it.
    pub(crate) main: u32,
    pub(crate) fade: Option<TrackFade>,
}

/// `TrackState::main` when a track has no animation on it.
pub(crate) const NO_SOURCE: u32 = u32::MAX;

impl TrackState {
    pub(crate) fn new() -> Self {
        Self { alpha: 1.0, blend: TrackBlend::Mix, sync: TrackSync::Free, policy: core::playback_controller::PlaybackPolicy::DEFAULT, outgoing_event_threshold: NO_EVENT_THRESHOLD, main: NO_SOURCE, fade: None }
    }

    /// A track contributes nothing at alpha 0 — `Mix` returns the pose below and
    /// `Additive` adds a zero-scaled delta.
    pub(crate) fn is_audible(&self) -> bool {
        self.alpha > 0.0
    }
}

/// A fade in flight on one track. Advanced by [`Context::update`](crate::Context::update).
#[derive(Debug, Clone, Copy)]
pub(crate) enum TrackFade {
    /// Raise `to` from weight 0 to 1 while every other source the track carried
    /// when the fade began sinks from its `outgoing_share` to 0, then retire them.
    Cross { to: u32, elapsed: f32, duration: f32 },
    /// Drive the track's `alpha` from `start` to `target`.
    Alpha { start: f32, target: f32, elapsed: f32, duration: f32 },
}

impl TrackFade {
    pub(crate) fn progress(&self) -> f32 {
        let (elapsed, duration) = match self {
            TrackFade::Cross { elapsed, duration, .. } | TrackFade::Alpha { elapsed, duration, .. } => (*elapsed, *duration),
        };
        (elapsed / duration).clamp(0.0, 1.0)
    }
}

pub(crate) fn count_attribute_capacities(metadata: &AnimationMetadata<'_>) -> (usize, usize, usize, usize) {
    let mut cell_c = 0;
    let mut color_c = 0;
    let mut shader_c = 0;
    let mut deform_c = 0;
    for p in metadata.cached_parts() {
        if p.run(crate::core::framedata::IDX_ATTRIBUTECELL).is_some() {
            cell_c += 1;
        }
        if p.run(crate::core::framedata::IDX_ATTRIBUTEPARTCOLOR).is_some() {
            color_c += 1;
        }
        if p.run(crate::core::framedata::IDX_ATTRIBUTESHADER).is_some() {
            shader_c += 1;
        }
        if p.run(crate::core::framedata::IDX_ATTRIBUTEDEFORM).is_some() {
            deform_c += 1;
        }
    }
    (cell_c, color_c, shader_c, deform_c)
}

/// Make room for `total` elements in all, whatever `v` holds now.
pub(crate) fn reserve_total<T>(v: &mut Vec<T>, total: usize) {
    v.reserve(total.saturating_sub(v.len()));
}

// Indices into the mix's per-attribute weight / accumulator arrays. One entry
// per continuous value the mix blends; `MIX_ATTR_FLAGS` maps each to the
// coverage bit that decides whether a source votes on it.
const MIX_POS_X: usize = 0;
const MIX_POS_Y: usize = 1;
const MIX_POS_Z: usize = 2;
const MIX_ROT_X: usize = 3;
const MIX_ROT_Y: usize = 4;
const MIX_ROT_Z: usize = 5;
const MIX_SCL_X: usize = 6;
const MIX_SCL_Y: usize = 7;
const MIX_LOSCL_X: usize = 8;
const MIX_LOSCL_Y: usize = 9;
const MIX_ALPHA: usize = 10;
const MIX_LOCAL_ALPHA: usize = 11;
const MIX_PIVOT_X: usize = 12;
const MIX_PIVOT_Y: usize = 13;
const MIX_ANCHOR_X: usize = 14;
const MIX_ANCHOR_Y: usize = 15;
const MIX_SIZE_X: usize = 16;
const MIX_SIZE_Y: usize = 17;
const MIX_UV_TX: usize = 18;
const MIX_UV_TY: usize = 19;
const MIX_UV_RZ: usize = 20;
const MIX_UV_SX: usize = 21;
const MIX_UV_SY: usize = 22;
const MIX_BOUND_R: usize = 23;
const MIX_MASK: usize = 24;
const MIX_SKEW_X: usize = 25;
const MIX_SKEW_Y: usize = 26;
const MIX_ATTR_COUNT: usize = 27;

/// Coverage bit gating each mixed attribute, in `MIX_*` index order.
/// `skew_x` / `skew_y` come from one source array and therefore share one bit.
pub(crate) const MIX_ATTR_FLAGS: [runtime::framedata::ss::runtime::UpdateAttributeFlags; MIX_ATTR_COUNT] = {
    use runtime::framedata::ss::runtime::UpdateAttributeFlags as F;
    [
        F::AttributePosX,
        F::AttributePosY,
        F::AttributePosZ,
        F::AttributeRotX,
        F::AttributeRotY,
        F::AttributeRotZ,
        F::AttributeSclX,
        F::AttributeSclY,
        F::AttributeLosclX,
        F::AttributeLosclY,
        F::AttributeAlpha,
        F::AttributeLoAlpha,
        F::AttributePivotX,
        F::AttributePivotY,
        F::AttributeAnchorX,
        F::AttributeAnchorY,
        F::AttributeSizeX,
        F::AttributeSizeY,
        F::AttributeUvtX,
        F::AttributeUvtY,
        F::AttributeUvrZ,
        F::AttributeUvsX,
        F::AttributeUvsY,
        F::AttributeBoundr,
        F::AttributeMask,
        F::AttributeSkew,
        F::AttributeSkew,
    ]
};

/// Which attributes `layer`'s source animates on `part`.
///
/// An empty `coverage_vec` means no coverage was recorded for this layer, which
/// reads as "covers everything" — the pre-coverage behavior. That is what keeps
/// hand-built layers (unit tests) working unchanged, and it fails safe: an
/// unrecorded layer contributes as it always did rather than silently dropping
/// out of the mix.
#[inline]
fn layer_coverage(layer: &LayerState<'_>, part: usize) -> runtime::framedata::ss::runtime::UpdateAttributeFlags {
    layer.coverage_vec.get(part).copied().unwrap_or(runtime::framedata::ss::runtime::UpdateAttributeFlags::all())
}

/// Read a `PartState`'s mixed attributes into `out`, in `MIX_*` index order.
#[inline]
fn read_attrs(s: &runtime::framedata::ss::runtime::PartState, out: &mut [f32; MIX_ATTR_COUNT]) {
    out[MIX_POS_X] = s.position_x();
    out[MIX_POS_Y] = s.position_y();
    out[MIX_POS_Z] = s.position_z();
    out[MIX_ROT_X] = s.rotation_x();
    out[MIX_ROT_Y] = s.rotation_y();
    out[MIX_ROT_Z] = s.rotation_z();
    out[MIX_SCL_X] = s.scale_x();
    out[MIX_SCL_Y] = s.scale_y();
    out[MIX_LOSCL_X] = s.local_scale_x();
    out[MIX_LOSCL_Y] = s.local_scale_y();
    out[MIX_ALPHA] = s.alpha();
    out[MIX_LOCAL_ALPHA] = s.local_alpha();
    out[MIX_PIVOT_X] = s.pivot_x();
    out[MIX_PIVOT_Y] = s.pivot_y();
    out[MIX_ANCHOR_X] = s.anchor_x();
    out[MIX_ANCHOR_Y] = s.anchor_y();
    out[MIX_SIZE_X] = s.size_x();
    out[MIX_SIZE_Y] = s.size_y();
    out[MIX_UV_TX] = s.uv_translation_x();
    out[MIX_UV_TY] = s.uv_translation_y();
    out[MIX_UV_RZ] = s.uv_rotation_z();
    out[MIX_UV_SX] = s.uv_scale_x();
    out[MIX_UV_SY] = s.uv_scale_y();
    out[MIX_BOUND_R] = s.bounding_radius();
    out[MIX_MASK] = s.mask();
    out[MIX_SKEW_X] = s.skew_x();
    out[MIX_SKEW_Y] = s.skew_y();
}

/// Write back the attributes some track spoke for; the rest keep what the
/// primary's extracted pose already held.
#[inline]
fn write_attrs(s: &mut runtime::framedata::ss::runtime::PartState, v: &[f32; MIX_ATTR_COUNT], touched: &[bool; MIX_ATTR_COUNT]) {
    macro_rules! w {
        ($idx:expr, $setter:ident) => {
            if touched[$idx] {
                s.$setter(v[$idx]);
            }
        };
    }
    w!(MIX_POS_X, set_position_x);
    w!(MIX_POS_Y, set_position_y);
    w!(MIX_POS_Z, set_position_z);
    w!(MIX_ROT_X, set_rotation_x);
    w!(MIX_ROT_Y, set_rotation_y);
    w!(MIX_ROT_Z, set_rotation_z);
    w!(MIX_SCL_X, set_scale_x);
    w!(MIX_SCL_Y, set_scale_y);
    w!(MIX_LOSCL_X, set_local_scale_x);
    w!(MIX_LOSCL_Y, set_local_scale_y);
    w!(MIX_ALPHA, set_alpha);
    w!(MIX_LOCAL_ALPHA, set_local_alpha);
    w!(MIX_PIVOT_X, set_pivot_x);
    w!(MIX_PIVOT_Y, set_pivot_y);
    w!(MIX_ANCHOR_X, set_anchor_x);
    w!(MIX_ANCHOR_Y, set_anchor_y);
    w!(MIX_SIZE_X, set_size_x);
    w!(MIX_SIZE_Y, set_size_y);
    w!(MIX_UV_TX, set_uv_translation_x);
    w!(MIX_UV_TY, set_uv_translation_y);
    w!(MIX_UV_RZ, set_uv_rotation_z);
    w!(MIX_UV_SX, set_uv_scale_x);
    w!(MIX_UV_SY, set_uv_scale_y);
    w!(MIX_BOUND_R, set_bounding_radius);
    w!(MIX_MASK, set_mask);
    w!(MIX_SKEW_X, set_skew_x);
    w!(MIX_SKEW_Y, set_skew_y);
}

/// Resolve every track onto one pose in `resolved_layer`.
///
/// Two levels. **Within a track**, its sources are weighted-summed, normalized
/// **per attribute** so a source that does not animate an attribute neither
/// contributes to it nor dilutes the sources that do (the coverage rule).
/// **Between tracks**, each track folds onto the pose below it in ascending
/// order — `Mix` lerps by the track's `alpha`, `Additive` adds `alpha` times the
/// track's offset from the setup pose. The stack's floor is the setup pose, so a
/// track's `alpha` always means something even where nothing below it has an
/// opinion; a source's `weight`, by contrast, only decides proportions among the
/// sources of one track.
///
/// Attributes no track speaks for keep `layers[primary_idx]`'s value, and every
/// discrete attribute (cell / part_color / shader / vertex / deform indices, flip
/// flags, hide, priority, texture, cell_meta, mesh_deform) is copied from there
/// verbatim — tracks never blend those. `update_flag` is OR-merged across every
/// contributing source so Stage B sees each attribute that took part.
///
/// A source is "active" when `animation_idx >= 0 && weight > 0.0`; a track is
/// audible when `alpha > 0.0`. With neither, the resolved pose equals
/// `layers[primary_idx]`.
pub(crate) fn mix_tracks<'b>(sources: &[SourceState], layers: &[LayerState<'b>], tracks: &[TrackState], primary_idx: usize, setup_pose: &[runtime::framedata::ss::runtime::PartState], resolved_layer: &mut LayerState<'b>) {
    let primary = &layers[primary_idx];
    let num_parts = primary.parts_vec.len();

    // Discrete structure: copied from primary verbatim.
    resolved_layer.cell_vec.clone_from(&primary.cell_vec);
    resolved_layer.parts_color_vec.clone_from(&primary.parts_color_vec);
    resolved_layer.shader_vec.clone_from(&primary.shader_vec);
    resolved_layer.deform_vec.clone_from(&primary.deform_vec);
    resolved_layer.cell_meta_per_part.clone_from(&primary.cell_meta_per_part);
    resolved_layer.cell_texture_hashes.clone_from(&primary.cell_texture_hashes);
    // The cell each part shows is the primary's; the UVs sample it.
    resolved_layer.cell_ids.clone_from(&primary.cell_ids);

    // mesh_deform holds variable-size per-part deltas; deep-copy from primary.
    if resolved_layer.mesh_deform.len() != num_parts {
        resolved_layer.mesh_deform.resize_with(num_parts, || None);
    }
    for (dst, src) in resolved_layer.mesh_deform.iter_mut().zip(primary.mesh_deform.iter()) {
        match src.as_deref() {
            Some(src) => {
                let dst = dst.get_or_insert_with(Default::default);
                dst.indices.clear();
                dst.indices.extend_from_slice(&src.indices);
                dst.dx.clear();
                dst.dx.extend_from_slice(&src.dx);
                dst.dy.clear();
                dst.dy.extend_from_slice(&src.dy);
            }
            None => {
                if let Some(dst) = dst.as_deref_mut() {
                    dst.clear();
                }
            }
        }
    }

    // Stage B fills these; clear them so stale post-mix data is never read.
    resolved_layer.vertex_vec.clear();
    for v in resolved_layer.local_vertices.iter_mut() {
        *v = 0.0;
    }

    resolved_layer.parts_vec.clear();
    resolved_layer.parts_vec.reserve(num_parts);
    if resolved_layer.raw_vertex_deform.len() != num_parts {
        resolved_layer.raw_vertex_deform.clear();
        resolved_layer.raw_vertex_deform.resize_with(num_parts, || None);
    } else {
        for slot in resolved_layer.raw_vertex_deform.iter_mut().flatten() {
            slot.present = false;
        }
    }

    for i in 0..num_parts {
        let primary_state = primary.parts_vec[i];
        let mut mixed = primary_state;

        // The stack starts at the setup pose and each track folds onto it in
        // ascending order. `cur` carries the running resolved value per attribute;
        // `touched` records which ones any track actually spoke for, so the rest
        // keep the primary's value exactly as they did before tracks existed.
        let floor = setup_pose.get(i).copied().unwrap_or(primary_state);
        let mut cur = [0.0f32; MIX_ATTR_COUNT];
        let mut base = [0.0f32; MIX_ATTR_COUNT];
        read_attrs(&floor, &mut cur);
        base.copy_from_slice(&cur);
        let mut touched = [false; MIX_ATTR_COUNT];

        let mut cur_dx = wide::f32x4::splat(0.0);
        let mut cur_dy = wide::f32x4::splat(0.0);
        let mut deform_present = false;
        let mut flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();

        for (t, track) in tracks.iter().enumerate() {
            // A silent track is not skipped: its sources still decide which
            // attributes the stack speaks for, and folding at `alpha == 0` is a
            // no-op that leaves the floor showing. That is what makes fading a
            // track to 0 resolve to the setup pose instead of freezing the pose
            // the source last produced.

            // Level 1: weighted mix of this track's own sources, per attribute
            // (the coverage rule). Without it, a source that carries no keyframe
            // array for an attribute would vote `evaluate_part`'s hardcoded default
            // and drag the result toward a number that means nothing — up to
            // thousands of pixels on real assets. Coverage is per animation
            // (`CachedPartData::coverage`), never per frame.
            let mut wsum = [0.0f32; MIX_ATTR_COUNT];
            let mut acc = [0.0f32; MIX_ATTR_COUNT];
            let mut deform_weight = 0.0_f32;
            let mut dx_acc = wide::f32x4::splat(0.0);
            let mut dy_acc = wide::f32x4::splat(0.0);

            for (j, src) in sources.iter().enumerate() {
                if src.track as usize != t || !src.is_active() {
                    continue;
                }
                // A source whose layer is shorter than the primary's part list does
                // not cover the trailing indices at all (mismatched part counts,
                // parts correlated positionally). Skipping it here instead of
                // indexing out of bounds is what keeps such a blend from panicking
                // and aborting the host.
                let Some(state) = layers[j].parts_vec.get(i) else {
                    continue;
                };

                // Per-attribute weights: `w` where this source covers the attribute,
                // 0 where it does not. Materializing them keeps the accumulation
                // branch-free and straight-line, which is what lets LLVM keep
                // vectorizing it.
                //
                // This table costs ~22% of the mix stage (65 parts, 2 sources:
                // 9.9us -> 12.1us per get_frame_data). Two attempts to win it back
                // measured no better and were dropped: a "coverage is uniform across
                // sources" fast path (almost never fires — a part is only uniform
                // when all 26 bits agree) and a branch-free bit-mask form (identical
                // timing; the cost is the stores, not the branches). Single-source
                // playback does not reach here at all — `get_frame_data` extracts
                // straight into the resolved layer instead — so nothing that ships
                // today pays it.
                let w = src.weight;
                let cov = layer_coverage(&layers[j], i);
                let mut wk = [0.0f32; MIX_ATTR_COUNT];
                for (k, f) in MIX_ATTR_FLAGS.iter().enumerate() {
                    if cov.contains(*f) {
                        wk[k] = w;
                        wsum[k] += w;
                    }
                }

                let mut v = [0.0f32; MIX_ATTR_COUNT];
                read_attrs(state, &mut v);
                for k in 0..MIX_ATTR_COUNT {
                    acc[k] += wk[k] * v[k];
                }
                flag |= state.update_flag();

                // `raw_vertex_deform` carries its own presence flag, which is the
                // same signal as coverage for the other attributes: normalize over
                // the sources that actually have a deform, so a source without one
                // does not pull the deltas toward zero.
                if let Some(rvd) = crate::core::framedata::vertex_deform_at(&layers[j].raw_vertex_deform, i) {
                    deform_weight += w;
                    let w_v = wide::f32x4::splat(w);
                    dx_acc = w_v.mul_add(wide::f32x4::from(rvd.dx), dx_acc);
                    dy_acc = w_v.mul_add(wide::f32x4::from(rvd.dy), dy_acc);
                }
            }

            // Level 2: fold this track's pose onto everything below it. Attributes
            // the track says nothing about pass through untouched — that is what
            // lets a track that only animates part of the rig act as an overlay
            // without an explicit mask.
            let alpha = track.alpha;
            for k in 0..MIX_ATTR_COUNT {
                if wsum[k] <= 0.0 {
                    continue;
                }
                let v = acc[k] / wsum[k];
                cur[k] = match track.blend {
                    TrackBlend::Mix => cur[k] + (v - cur[k]) * alpha,
                    TrackBlend::Additive => cur[k] + (v - base[k]) * alpha,
                };
                touched[k] = true;
            }

            if deform_weight > 0.0 {
                let inv = wide::f32x4::splat(1.0 / deform_weight);
                let (dx, dy) = (dx_acc * inv, dy_acc * inv);
                let a = wide::f32x4::splat(alpha);
                match track.blend {
                    TrackBlend::Mix => {
                        cur_dx += (dx - cur_dx) * a;
                        cur_dy += (dy - cur_dy) * a;
                    }
                    // The setup pose carries no deform, so the additive reference is zero.
                    TrackBlend::Additive => {
                        cur_dx += dx * a;
                        cur_dy += dy * a;
                    }
                }
                deform_present = true;
            }
        }

        if touched.iter().any(|t| *t) {
            write_attrs(&mut mixed, &cur, &touched);
            mixed.set_update_flag(flag);
        }
        if deform_present {
            **resolved_layer.raw_vertex_deform[i].get_or_insert_with(Default::default) = crate::core::framedata::RawVertexDeform { present: true, dx: cur_dx.to_array(), dy: cur_dy.to_array() };
        }

        resolved_layer.parts_vec.push(mixed);
    }
}
