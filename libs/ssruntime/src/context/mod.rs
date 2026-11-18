//! [`Context`]: the playback state of one character, and the per-frame evaluation that turns it into frame data.

use alloc::vec::Vec;

use crate::mix::{LayerState, NO_EVENT_THRESHOLD, SourceState, TrackBlend, TrackFade, TrackState, count_attribute_capacities, reserve_total};
use crate::resource::{AnimationMetadata, AnimationResource, fnv1a_hash_str};
use crate::{error, eval};

mod buffers;
mod events;
mod frame;
mod overrides;
mod playback;
mod tracks;

/// Per-instance global buffers — the output of `resolve_global_state`.
/// World matrices, UVs, and shape vertices are composited from the resolved
/// (post-mix) pose; they are not per-layer.
pub(crate) struct GlobalBuffers {
    world_matrices: Vec<crate::eval::SsMatrix4x4>,
    inheritance_matrices: Vec<crate::eval::SsMatrix4x4>,
    pub(crate) local_uvs: Vec<f32>,
    shape_vertices: Vec<f32>,
    shape_vertex_box_coords: Vec<f32>,
    shape_vertex_counts: Vec<i32>,
    /// Skinned mesh world-space vertex positions, concatenated across all parts.
    /// Use `mesh_vertex_offsets` to slice per-part ranges:
    ///   part `p` occupies `mesh_vertex_offsets[p]..mesh_vertex_offsets[p + 1]`.
    /// Non-mesh parts contribute an empty range.
    mesh_vertices_x: Vec<f32>,
    mesh_vertices_y: Vec<f32>,
    /// CSR-style prefix sum, length = `num_parts + 1`.
    mesh_vertex_offsets: Vec<u32>,
    /// The mesh layout (`AnimationMetadata::mesh_layout`) whose static UVs and
    /// triangle indices the getters hand out, and that `mesh_vertex_offsets`
    /// and the vertex buffers are sized for; [`NO_MESH_LAYOUT`] until
    /// `compute_mesh_vertices` sets them up, and in SceneGraph mode. The buffers
    /// themselves are the animation's, shared by every instance.
    mesh_layout: usize,
    /// Per-frame Nine-Slice grids, CSR-sliced per part. Written by
    /// `compose_frame_data` (the draw plan needs the counts), not by
    /// `resolve_global_state`.
    nines: crate::eval::framedata::NinesBuffers,
}

impl GlobalBuffers {
    fn new() -> Self {
        Self {
            world_matrices: Vec::new(),
            inheritance_matrices: Vec::new(),
            local_uvs: Vec::new(),
            shape_vertices: Vec::new(),
            shape_vertex_box_coords: Vec::new(),
            shape_vertex_counts: Vec::new(),
            mesh_vertices_x: Vec::new(),
            mesh_vertices_y: Vec::new(),
            mesh_vertex_offsets: Vec::new(),
            mesh_layout: NO_MESH_LAYOUT,
            nines: crate::eval::framedata::NinesBuffers::default(),
        }
    }
}

/// `GlobalBuffers::mesh_layout` before the static mesh buffers are set up.
const NO_MESH_LAYOUT: usize = usize::MAX;

/// Static / shareable evaluation context. The resource borrow and the
/// FlatBufferBuilder workspace live here because they are what Animation
/// Instancing (ROADMAP.md) would share: one resource and one fb workspace
/// driving N instances serially.
pub(crate) struct EvaluationContext<'a> {
    resource: Option<&'a AnimationResource<'a>>,
    fb: flatbuffers::FlatBufferBuilder<'a>,
    /// Whether `fb` currently holds a finished frame-data buffer. Guards the
    /// zero-copy accessors: `finished_data()` asserts on an unfinished builder,
    /// and `panic = "abort"` makes that assert fatal.
    frame_data_ready: bool,
    /// The largest frame data `fb` has finished, and the capacity it was last
    /// given. The builder grows by doubling, so a frame that outgrows the
    /// capacity leaves up to half of it unused; the next frame starts on a
    /// builder sized to the largest frame instead.
    fb_need: usize,
    pub(crate) fb_capacity: usize,

    // Workspace buffers to avoid per-frame allocations
    draw_order_workspace: Vec<u16>,
    batches_workspace: Vec<crate::runtime::framedata::ss::runtime::DrawBatch>,
    mask_indices_workspace: Vec<u16>,
    deform_interpolation_start_workspace: Vec<f32>,
    deform_interpolation_end_workspace: Vec<f32>,
    deform_interpolation_out_workspace: Vec<f32>,
    deform_result_coordinate_workspace: Vec<crate::runtime::framedata::ss::runtime::Vec2>,
    deform_result_index_workspace: Vec<i16>,
    inheritance_alphas_workspace: Vec<f32>,
    /// Scratch for the Override Layer's force-hide cascade forward pass.
    override_hide_workspace: Vec<bool>,
    /// Scratch copy of the primary controller's traversed frames, so the seam
    /// can read them while the override table is borrowed mutably.
    override_passed_frames_workspace: Vec<i32>,
}

impl<'a> EvaluationContext<'a> {
    fn new() -> Self {
        Self {
            resource: None,
            fb: flatbuffers::FlatBufferBuilder::new(),
            frame_data_ready: false,
            fb_need: 0,
            fb_capacity: 0,
            draw_order_workspace: Vec::new(),
            batches_workspace: Vec::new(),
            mask_indices_workspace: Vec::new(),
            deform_interpolation_start_workspace: Vec::new(),
            deform_interpolation_end_workspace: Vec::new(),
            deform_interpolation_out_workspace: Vec::new(),
            deform_result_coordinate_workspace: Vec::new(),
            deform_result_index_workspace: Vec::new(),
            inheritance_alphas_workspace: Vec::new(),
            override_hide_workspace: Vec::new(),
            override_passed_frames_workspace: Vec::new(),
        }
    }
}

/// The transport of the whole character. A source's own controller keeps its
/// playhead, speed, loop count and section, but whether it moves at all is
/// decided here, so a source attached mid-playback moves with the rest and
/// pause / stop reach every track. Track 0 finishing does not stop it: the
/// other tracks and the fades carry on.
#[derive(Debug, Clone, Copy, Default)]
struct CharacterClock {
    /// Set by `play`, cleared by `stop` and by a new setup.
    playing: bool,
    /// A hold on a playing clock. Never set while `playing` is false.
    pausing: bool,
}

impl CharacterClock {
    fn is_running(&self) -> bool {
        self.playing && !self.pausing
    }
}

/// Per-instance playback state. Owns one playback source per blend layer,
/// the per-source pose buffers (`layers`), the post-mix pose
/// (`resolved_layer`) that feeds global resolution, the resolved global
/// buffers, and the configuration knobs that influence frame evaluation.
/// Kept apart from `EvaluationContext` so that many of these can share one.
///
/// `sources` and `layers` are kept length-parallel and always contain at
/// least one slot (the primary). The `primary` field designates the
/// authoritative source for events and non-blendable discrete attributes.
pub(crate) struct InstanceState<'a> {
    /// Whether the character is playing, and held. Every source on every track
    /// advances on this one clock, and every fade with it.
    clock: CharacterClock,
    pub(crate) sources: Vec<SourceState>,
    /// The track stack, ascending. Always holds at least track 0, which carries
    /// the structure (discrete attributes and events) for the whole instance.
    pub(crate) tracks: Vec<TrackState>,
    pub(crate) layers: Vec<LayerState<'a>>,
    pub(crate) resolved_layer: LayerState<'a>,
    pub(crate) primary: u32,
    z_order: crate::eval::z_order::ZOrderTable,
    pub(crate) globals: GlobalBuffers,
    collected_event_indices: Vec<i32>,
    /// Parallel to `collected_event_indices`: source handle that fired each
    /// collected event. At the default event threshold every entry equals
    /// `primary` — "track 0's animation fires events". When other sources
    /// opt in (lower threshold), entries record the firing source.
    collected_event_source_handles: Vec<u32>,
    pub(crate) coordinate_system: crate::eval::SsCoordinateSystem,
    pub(crate) origin: crate::eval::SsOrigin,
    pub(crate) unit: crate::eval::SsUnit,
    render_mode: crate::eval::RenderMode,
    /// Per-part runtime Override Layer table. Dense by `part_index`,
    /// `None` = no override. Created at the part count by the first override
    /// set on the instance, and dropped again when a new animation leaves no
    /// override in it, so an instance that overrides nothing never allocates it.
    pub(crate) part_overrides: Vec<Option<crate::eval::override_layer::PartOverride>>,
    /// Per-part resolved cell meta recorded by the seam when a cell override is
    /// applied, so `compute_local_uvs` samples the overridden cell (not just the
    /// geometry/texture channels). Empty when no override is active this frame.
    override_cell_uv_meta: Vec<Option<crate::eval::framedata::ResolvedCellMeta>>,
    /// How many times `Context::update` has run: the update an evaluation
    /// belongs to, which an OverwriteOnNextKeyframe override's first evaluation records.
    updates: u64,
}

impl<'a> InstanceState<'a> {
    fn new() -> Self {
        Self {
            clock: CharacterClock::default(),
            sources: alloc::vec![SourceState::new()],
            tracks: alloc::vec![TrackState::new()],
            layers: alloc::vec![LayerState::new()],
            resolved_layer: LayerState::new(),
            primary: 0,
            z_order: crate::eval::z_order::ZOrderTable::new(),
            globals: GlobalBuffers::new(),
            collected_event_indices: Vec::new(),
            collected_event_source_handles: Vec::new(),
            coordinate_system: crate::eval::SsCoordinateSystem::default(),
            origin: crate::eval::SsOrigin::default(),
            unit: crate::eval::SsUnit::default(),
            render_mode: crate::eval::RenderMode::default(),
            part_overrides: Vec::new(),
            override_cell_uv_meta: Vec::new(),
            updates: 0,
        }
    }

    /// `true` when at least one part carries an active override — gates the
    /// per-frame seam work in `get_frame_data`.
    pub(crate) fn has_any_override(&self) -> bool {
        self.part_overrides.iter().any(|o| o.as_ref().is_some_and(|p| !p.is_empty()))
    }

    /// Drop the slot back to `None` if the entry no longer carries any override,
    /// keeping [`has_any_override`](Self::has_any_override) cheap and the
    /// lifecycle bookkeeping simple.
    fn compact_override(&mut self, idx: usize) {
        if let Some(entry) = self.part_overrides.get_mut(idx)
            && entry.as_ref().is_some_and(|p| p.is_empty())
        {
            *entry = None;
        }
    }

    /// On a new animation within the same SSAB: drop Visibility and OverwriteOnNextKeyframe and
    /// HoldUntilNextAnimation Color/Cell overrides, keep Permanent ones, then resize to the
    /// new part count (part identity is stable within an SSAB; parts beyond the
    /// new count are dropped by the resize — the Permanent re-validation).
    fn retain_permanent_overrides_on_setup(&mut self, num_parts: usize) {
        use crate::eval::override_layer::PriorityMode;
        if self.part_overrides.is_empty() {
            return;
        }
        for entry in self.part_overrides.iter_mut() {
            let Some(p) = entry.as_mut() else { continue };
            p.visibility = None;
            if p.color.as_ref().is_some_and(|c| c.priority != PriorityMode::Permanent) {
                p.color = None;
            }
            if p.cell.as_ref().is_some_and(|c| c.priority != PriorityMode::Permanent) {
                p.cell = None;
            }
            if p.is_empty() {
                *entry = None;
            }
        }
        if self.part_overrides.iter().all(Option::is_none) {
            self.part_overrides = Vec::new();
        } else {
            self.part_overrides.resize(num_parts, None);
        }
    }

    fn primary_idx(&self) -> usize {
        self.primary as usize
    }

    /// Reset the stack to a single track carrying a single source. Called when a
    /// new animation is set up: an instance's tracks belong to the animation that
    /// established them, so they do not survive a re-setup.
    fn collapse_to_track_zero(&mut self) {
        // The collected events read against these sources, and their handles would
        // name whatever the slots hold next.
        self.collected_event_indices.clear();
        self.collected_event_source_handles.clear();
        self.sources.truncate(1);
        self.layers.truncate(1);
        // Rebuilding the slot must not read as a reset of the host's playback
        // policy: this runs inside every `setup_animation`, and the speed /
        // frame-skip / frame-rate override deliberately outlive an animation
        // change. They are track 0's, not slot 0's: after a crossfade slot 0
        // may be a released source at the defaults. `Context::reset` clears
        // them explicitly afterwards, because that one really is a teardown.
        let policy = self.tracks[0].policy;
        let outgoing_event_threshold = self.tracks[0].outgoing_event_threshold;
        self.sources[0] = SourceState::new();
        self.sources[0].controller.adopt_policy(&policy);
        self.tracks.truncate(1);
        self.tracks[0] = TrackState::new();
        self.tracks[0].policy = policy;
        self.tracks[0].outgoing_event_threshold = outgoing_event_threshold;
        self.tracks[0].main = 0;
        self.primary = 0;
    }

    pub(crate) fn primary_source(&self) -> &SourceState {
        &self.sources[self.primary_idx()]
    }

    fn primary_source_mut(&mut self) -> &mut SourceState {
        let idx = self.primary_idx();
        &mut self.sources[idx]
    }

    /// `true` when the primary alone decides the pose: it is active, its track
    /// mixes at full alpha, and no other active source is on an audible track.
    /// The mix would yield the primary's pose verbatim, so the pose is
    /// extracted straight into the resolved layer instead, which skips the
    /// per-part vec clone + 27 per-part PartState setter calls and the
    /// primary's own layer.
    fn is_single_full_track(&self) -> bool {
        let primary_idx = self.primary_idx();
        let primary = &self.sources[primary_idx];
        if !primary.is_active() {
            return false;
        }
        // Any second audible source, or a track that would lerp/offset rather than
        // pass its source through, means the fold has real work to do.
        let track = &self.tracks[primary.track as usize];
        if track.alpha != 1.0 || track.blend != TrackBlend::Mix {
            return false;
        }
        !self.sources.iter().enumerate().any(|(i, s)| i != primary_idx && s.is_active() && self.tracks[s.track as usize].is_audible())
    }

    /// Change one setting of `track`'s playback policy, and the same setting on
    /// every source the track carries now (both sides of a crossfade included),
    /// so the policy and what is playing never disagree. The caller has made
    /// sure the track exists.
    fn set_track_policy(&mut self, track: u32, policy: impl FnOnce(&mut eval::playback_controller::PlaybackPolicy), controller: impl Fn(&mut eval::playback_controller::PlaybackController)) {
        policy(&mut self.tracks[track as usize].policy);
        for source in self.sources.iter_mut() {
            if source.track as u32 == track && source.has_real_animation() {
                controller(&mut source.controller);
            }
        }
    }

    /// Whether source `i`, which is not the primary, fires the events it passes.
    /// On the outgoing side of a crossfade it is the side's share of the track
    /// that is measured, against the track's threshold, so a side of several
    /// sources goes quiet at once; anywhere else, its own weight against its own
    /// threshold.
    fn source_fires_events(&self, i: usize) -> bool {
        let source = &self.sources[i];
        let track = &self.tracks[source.track as usize];
        if let Some(fade @ TrackFade::Cross { to, .. }) = track.fade
            && to as usize != i
            && source.outgoing_share > 0.0
        {
            return 1.0 - fade.progress() >= track.outgoing_event_threshold;
        }
        source.weight >= source.event_threshold
    }

    /// The controller of the source carrying `track`'s animation.
    fn track_main_controller(&self, track: u32) -> Option<&eval::playback_controller::PlaybackController> {
        let main = self.tracks.get(track as usize)?.main;
        let source = self.sources.get(main as usize)?;
        source.has_real_animation().then_some(&source.controller)
    }
}

/// The playback state of one animated object: a stack of tracks, each playing an
/// animation of the bound resource, layered onto the setup pose. Bind a resource
/// with [`Self::bind_resource`] and select an animation with
/// [`Self::setup_animation`] before playing.
pub struct Context<'a> {
    eval: EvaluationContext<'a>,
    instance: InstanceState<'a>,
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Read-only access to the internals, for the unit tests in `crate::tests`.
#[cfg(all(test, feature = "std"))]
impl<'a> Context<'a> {
    pub(crate) fn eval(&self) -> &EvaluationContext<'a> {
        &self.eval
    }

    pub(crate) fn instance(&self) -> &InstanceState<'a> {
        &self.instance
    }
}

impl<'a> Context<'a> {
    /// An empty context.
    pub fn new() -> Self {
        Context { eval: EvaluationContext::new(), instance: InstanceState::new() }
    }

    /// Back to the just-created state: the resource is unbound, and the overrides
    /// and the playback and coordinate settings are cleared.
    pub fn reset(&mut self) {
        self.eval.resource = None;
        self.eval.fb.reset();
        self.eval.frame_data_ready = false;
        self.instance.collapse_to_track_zero();
        // A teardown, not an animation change: the host's playback policy goes
        // with it, which `collapse_to_track_zero` alone would have carried over.
        self.instance.tracks[0].policy = eval::playback_controller::PlaybackPolicy::DEFAULT;
        self.instance.tracks[0].outgoing_event_threshold = NO_EVENT_THRESHOLD;
        self.instance.primary_source_mut().controller.reset_setup();
        self.instance.layers[0].cursors.clear();
        self.instance.resolved_layer.cursors.clear();
        self.instance.coordinate_system = crate::eval::SsCoordinateSystem::default();
        self.instance.origin = crate::eval::SsOrigin::default();
        self.instance.unit = crate::eval::SsUnit::default();
        self.instance.render_mode = crate::eval::RenderMode::default();
        self.instance.z_order = crate::eval::z_order::ZOrderTable::new();
        self.instance.part_overrides.clear();
        self.instance.globals.mesh_layout = NO_MESH_LAYOUT;
    }

    /// What the frame computes; see [`RenderMode`](crate::eval::RenderMode).
    pub fn set_render_mode(&mut self, mode: crate::eval::RenderMode) {
        self.instance.render_mode = mode;
    }

    /// What [`Self::set_render_mode`] stored.
    pub fn render_mode(&self) -> crate::eval::RenderMode {
        self.instance.render_mode
    }

    /// Bind `resource`, dropping the tracks and the overrides. The coordinate
    /// settings stay. [`Self::setup_animation`] comes next.
    pub fn bind_resource(&mut self, resource: &'a AnimationResource<'a>) {
        self.eval.resource = Some(resource);
        self.eval.fb.reset();
        self.eval.frame_data_ready = false;
        self.instance.collapse_to_track_zero();
        self.instance.clock = CharacterClock::default();
        self.instance.layers[0].cursors.clear();
        self.instance.resolved_layer.cursors.clear();
        // A new SSAB invalidates part identity — drop all overrides.
        self.instance.part_overrides.clear();
        // The layout indexes the old pack's static mesh buffers.
        self.instance.globals.mesh_layout = NO_MESH_LAYOUT;
        // Configuration (coordinate_system, origin, unit) is preserved.
    }

    /// The bound resource's `.ssab` bytes.
    pub fn get_ssab_data(&self) -> Option<&[u8]> {
        self.eval.resource.map(|r| r.get_ssab_data())
    }

    /// Select the animation `name` and move to its first frame. This does not start
    /// it; [`Self::play`] does. It drops every track but 0, the section and the loop
    /// count, and keeps the speed, the frame rate, the frame skip and the direction.
    pub fn setup_animation(&mut self, name: &str) -> Result<(), error::SSRuntimeError> {
        let hash = fnv1a_hash_str(name);
        self.setup_animation_by_hash(hash)
    }

    /// [`Self::setup_animation`] by the FNV-1a hash of the name.
    pub fn setup_animation_by_hash(&mut self, hash: u32) -> Result<(), error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };

        let Some(index) = resource.find_animation_index_by_hash(hash) else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };

        let metadata = &resource.animations[index];
        self.instance.collapse_to_track_zero();
        // Selecting an animation does not start it; `play` does.
        self.instance.clock = CharacterClock::default();
        let primary = self.instance.primary_source_mut();
        primary.controller.setup(metadata.start_frame, metadata.end_frame, metadata.fps);
        primary.animation_idx = index as i32;
        primary.weight = 1.0;

        let num_parts = metadata.cached_parts().len();
        let (cell_c, color_c, shader_c, deform_c) = count_attribute_capacities(metadata);
        let cursor_c = crate::eval::framedata::cursor_count(metadata.cached_parts());

        // A new setup plays one source alone, extracted straight into the
        // resolved layer; the source's own layer is sized when a mix needs it.
        for layer in self.instance.layers.iter_mut() {
            *layer = LayerState::new();
        }
        self.instance.resolved_layer.resize_for_parts(num_parts, cursor_c, cell_c, color_c, shader_c, deform_c);

        // Override Layer: a new animation clears Visibility and OverwriteOnNextKeyframe and
        // HoldUntilNextAnimation Color/Cell overrides but keeps Permanent ones.
        self.instance.retain_permanent_overrides_on_setup(num_parts);

        let globals = &mut self.instance.globals;
        globals.world_matrices.clear();
        globals.world_matrices.resize(num_parts, crate::eval::SsMatrix4x4::IDENTITY);
        globals.inheritance_matrices.clear();
        globals.inheritance_matrices.resize(num_parts, crate::eval::SsMatrix4x4::IDENTITY);
        globals.local_uvs.clear();
        globals.local_uvs.resize(num_parts * 10, 0.0);
        // Shape buffers keep their `part_index * SHAPE_VERTICES_STRIDE` layout, but
        // only a pack with a shape part has anything to put in them.
        let shape_len = if metadata.cached_parts().iter().any(|p| p.shape_kind.is_some()) { num_parts * crate::eval::framedata::SHAPE_VERTICES_STRIDE } else { 0 };
        globals.shape_vertices = alloc::vec![0.0; shape_len];
        globals.shape_vertex_box_coords = alloc::vec![0.0; shape_len];
        globals.shape_vertex_counts.clear();
        globals.shape_vertex_counts.resize(num_parts, 0);
        // Have compute_mesh_vertices set up the newly selected animation's
        // static mesh buffers.
        globals.mesh_layout = NO_MESH_LAYOUT;

        self.instance.z_order.setup(index as i32, metadata.cached_parts());
        self.prepare_part_bases();
        self.reserve_frame_workspaces(metadata);

        Ok(())
    }

    /// Size up front what the first frame of `metadata`'s animation would
    /// otherwise grow on its way: the buffers whose size the parts decide, by
    /// their count or by their meshes' vertices. What a frame's contents decide
    /// — the frame data, the draw batches, a deform, an effect's particles, the
    /// frames an update passes — still grows on first use.
    fn reserve_frame_workspaces(&mut self, metadata: &AnimationMetadata<'a>) {
        let parts = metadata.cached_parts();
        let num_parts = parts.len();

        let resolved = &mut self.instance.resolved_layer;
        // One vertex attribute per part that shows a cell, whether or not it
        // keys a vertex deform.
        resolved.vertex_vec.clear();
        resolved.vertex_vec.reserve(parts.iter().filter(|p| p.run(crate::eval::framedata::IDX_ATTRIBUTECELL).is_some()).count());
        if resolved.mesh_deform.len() != num_parts {
            resolved.mesh_deform.resize_with(num_parts, || None);
        }

        let eval = &mut self.eval;
        reserve_total(&mut eval.inheritance_alphas_workspace, num_parts);
        reserve_total(&mut eval.draw_order_workspace, num_parts);
        reserve_total(&mut eval.mask_indices_workspace, parts.iter().filter(|p| p.is_mask_contributor).count());

        let globals = &mut self.instance.globals;
        reserve_total(&mut globals.nines.vertex_offsets, num_parts + 1);
        reserve_total(&mut globals.nines.index_offsets, num_parts + 1);
        // Room only: the static mesh buffers are still handed out from the
        // first frame on, and only a Direct instance positions mesh vertices.
        reserve_total(&mut globals.mesh_vertex_offsets, num_parts + 1);
        if self.instance.render_mode == crate::eval::RenderMode::Direct {
            let total_vertices = parts.iter().map(|p| p.mesh_cell.as_ref().map_or(0, |m| m.num_vertices())).sum();
            reserve_total(&mut globals.mesh_vertices_x, total_vertices);
            reserve_total(&mut globals.mesh_vertices_y, total_vertices);
        }
    }
}
