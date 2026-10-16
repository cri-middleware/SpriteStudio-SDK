//! [`Context`]: the playback state of one character, and the per-frame evaluation that turns it into frame data.

use alloc::vec::Vec;

use crate::mix::{LayerState, NO_EVENT_THRESHOLD, NO_SOURCE, SourceState, TrackBlend, TrackFade, TrackState, TrackSync, count_attribute_capacities, mix_tracks, reserve_total};
use crate::resource::{AnimationMetadata, AnimationResource, StaticMeshBuffers, fnv1a_hash_str};
use crate::{core, error};

/// Per-instance global buffers — the output of `resolve_global_state`.
/// World matrices, UVs, and shape vertices are composited from the resolved
/// (post-mix) pose; they are not per-layer.
pub(crate) struct GlobalBuffers {
    world_matrices: Vec<crate::core::SsMatrix4x4>,
    inheritance_matrices: Vec<crate::core::SsMatrix4x4>,
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
    nines: crate::core::framedata::NinesBuffers,
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
            nines: crate::core::framedata::NinesBuffers::default(),
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
    z_order: crate::core::z_order::ZOrderTable,
    pub(crate) globals: GlobalBuffers,
    collected_event_indices: Vec<i32>,
    /// Parallel to `collected_event_indices`: source handle that fired each
    /// collected event. At the default event threshold every entry equals
    /// `primary` — "track 0's animation fires events". When other sources
    /// opt in (lower threshold), entries record the firing source.
    collected_event_source_handles: Vec<u32>,
    pub(crate) coordinate_system: crate::core::SsCoordinateSystem,
    pub(crate) origin: crate::core::SsOrigin,
    pub(crate) unit: crate::core::SsUnit,
    render_mode: crate::core::RenderMode,
    /// Per-part runtime Override Layer table. Dense by `part_index`,
    /// `None` = no override. Created at the part count by the first override
    /// set on the instance, and dropped again when a new animation leaves no
    /// override in it, so an instance that overrides nothing never allocates it.
    pub(crate) part_overrides: Vec<Option<crate::core::override_layer::PartOverride>>,
    /// Per-part resolved cell meta recorded by the seam when a cell override is
    /// applied, so `compute_local_uvs` samples the overridden cell (not just the
    /// geometry/texture channels). Empty when no override is active this frame.
    override_cell_uv_meta: Vec<Option<crate::core::framedata::ResolvedCellMeta>>,
    /// How many times `Context::update` has run: the update an evaluation
    /// belongs to, which a NON override's first evaluation records.
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
            z_order: crate::core::z_order::ZOrderTable::new(),
            globals: GlobalBuffers::new(),
            collected_event_indices: Vec::new(),
            collected_event_source_handles: Vec::new(),
            coordinate_system: crate::core::SsCoordinateSystem::default(),
            origin: crate::core::SsOrigin::default(),
            unit: crate::core::SsUnit::default(),
            render_mode: crate::core::RenderMode::default(),
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

    /// On a new animation within the same SSAB: drop Visibility and NON/NOW
    /// priority Color/Cell overrides, keep PERMANENT ones, then resize to the
    /// new part count (part identity is stable within an SSAB; parts beyond the
    /// new count are dropped by the resize — the PERMANENT re-validation).
    fn retain_permanent_overrides_on_setup(&mut self, num_parts: usize) {
        use crate::core::override_layer::PriorityMode;
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
    fn set_track_policy(&mut self, track: u32, policy: impl FnOnce(&mut core::playback_controller::PlaybackPolicy), controller: impl Fn(&mut core::playback_controller::PlaybackController)) {
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
    fn track_main_controller(&self, track: u32) -> Option<&core::playback_controller::PlaybackController> {
        let main = self.tracks.get(track as usize)?.main;
        let source = self.sources.get(main as usize)?;
        source.has_real_animation().then_some(&source.controller)
    }
}

pub struct Context<'a> {
    pub(crate) eval: EvaluationContext<'a>,
    pub(crate) instance: InstanceState<'a>,
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context { eval: EvaluationContext::new(), instance: InstanceState::new() }
    }

    pub fn reset(&mut self) {
        self.eval.resource = None;
        self.eval.fb.reset();
        self.eval.frame_data_ready = false;
        self.instance.collapse_to_track_zero();
        // A teardown, not an animation change: the host's playback policy goes
        // with it, which `collapse_to_track_zero` alone would have carried over.
        self.instance.tracks[0].policy = core::playback_controller::PlaybackPolicy::DEFAULT;
        self.instance.tracks[0].outgoing_event_threshold = NO_EVENT_THRESHOLD;
        self.instance.primary_source_mut().controller.reset_setup();
        self.instance.layers[0].cursors.clear();
        self.instance.resolved_layer.cursors.clear();
        self.instance.coordinate_system = crate::core::SsCoordinateSystem::default();
        self.instance.origin = crate::core::SsOrigin::default();
        self.instance.unit = crate::core::SsUnit::default();
        self.instance.render_mode = crate::core::RenderMode::default();
        self.instance.z_order = crate::core::z_order::ZOrderTable::new();
        self.instance.part_overrides.clear();
    }

    pub fn set_render_mode(&mut self, mode: crate::core::RenderMode) {
        self.instance.render_mode = mode;
    }

    pub fn render_mode(&self) -> crate::core::RenderMode {
        self.instance.render_mode
    }

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
        // Configuration (coordinate_system, origin, unit) is preserved.
    }

    pub fn get_ssab_data(&self) -> Option<&[u8]> {
        self.eval.resource.map(|r| r.get_ssab_data())
    }

    // --- Override Layer API ---

    /// Resolve a part name to its `part_index` in the current primary animation,
    /// or `-1` if not loaded / not found. Indices are stable within an SSAB, so
    /// callers can resolve once and reuse the index across animation changes.
    pub fn find_part_index(&self, name: &str) -> i32 {
        let Some(resource) = self.eval.resource else {
            return -1;
        };
        let anim_idx = self.instance.primary_source().animation_idx;
        if anim_idx < 0 {
            return -1;
        }
        let hash = fnv1a_hash_str(name);
        resource.animations[anim_idx as usize].cached_parts().iter().position(|p| p.name_hash == hash).map(|i| i as i32).unwrap_or(-1)
    }

    /// The integer frame the primary playhead is on right now — baked into a NON
    /// override at set time so its own set-frame keyframe can be skipped through
    /// the update of its first evaluation (see `PriorityMode::OverwriteOnNextKeyframe`).
    fn override_set_frame(&self) -> i32 {
        crate::core::math::floorf(self.instance.primary_source().controller.get_frame_no()) as i32
    }

    /// The primary animation's part count — the range an override's part index
    /// must fall in — or 0 before an animation is set up.
    pub(crate) fn override_part_count(&self) -> usize {
        let anim_idx = self.instance.primary_source().animation_idx;
        match self.eval.resource {
            Some(resource) if anim_idx >= 0 => resource.animations[anim_idx as usize].cached_parts().len(),
            _ => 0,
        }
    }

    /// The override slot of `part`, creating the table the first time an
    /// override is set.
    fn override_slot(&mut self, part: u32) -> Result<&mut Option<crate::core::override_layer::PartOverride>, error::SSRuntimeError> {
        let (idx, num_parts) = (part as usize, self.override_part_count());
        if idx >= num_parts {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        if self.instance.part_overrides.is_empty() {
            self.instance.part_overrides.resize(num_parts, None);
        }
        Ok(&mut self.instance.part_overrides[idx])
    }

    /// Clear one kind of override on `part`. Nothing to do when the table was
    /// never created.
    fn clear_override(&mut self, part: u32, clear: impl FnOnce(&mut crate::core::override_layer::PartOverride)) -> Result<(), error::SSRuntimeError> {
        let idx = part as usize;
        if idx >= self.override_part_count() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        if let Some(entry) = self.instance.part_overrides.get_mut(idx).and_then(|e| e.as_mut()) {
            clear(entry);
        }
        self.instance.compact_override(idx);
        Ok(())
    }

    /// Set or clear a per-part visibility override.
    /// `force`: `0` = clear/revert to animation, `1` = force-hide,
    /// `2` = force-visible (reserved, not implemented — rejected).
    /// `cascade` propagates a force-hide to descendants.
    pub fn set_part_visibility_override(&mut self, part: u32, force: u8, cascade: bool) -> Result<(), error::SSRuntimeError> {
        match force {
            0 => self.clear_override(part, |entry| entry.visibility = None)?,
            1 => {
                let entry = self.override_slot(part)?.get_or_insert_with(Default::default);
                entry.visibility = Some(crate::core::override_layer::VisibilityOverride { cascade });
            }
            _ => return Err(error::SSRuntimeError::InvalidParameter),
        }
        Ok(())
    }

    /// Clear the visibility override on a part (revert to animation).
    pub fn clear_part_visibility_override(&mut self, part: u32) -> Result<(), error::SSRuntimeError> {
        self.set_part_visibility_override(part, 0, false)
    }

    /// Set a per-part color override with a single RGBA (fills all four corners).
    /// `blend_op`: `0=Mix 1=Mul 2=Add 3=Sub`. `rgba`: packed `0xRRGGBBAA`.
    /// `priority`: `0=NON 1=NOW_ANIMATION 2=PERMANENT`. Normal parts only.
    pub fn set_part_color_override(&mut self, part: u32, blend_op: u8, rgba: u32, priority: u8) -> Result<(), error::SSRuntimeError> {
        let corner = rgba.to_be_bytes(); // 0xRRGGBBAA -> [r, g, b, a]
        self.set_part_color_override_corners(part, blend_op, [corner; 4], priority)
    }

    /// Set a per-part color override with distinct corners in ssruntime order
    /// `lt, rt, lb, rb`, each `[r, g, b, a]`.
    pub fn set_part_color_override_corners(&mut self, part: u32, blend_op: u8, corners_rgba: [[u8; 4]; 4], priority: u8) -> Result<(), error::SSRuntimeError> {
        if blend_op >= 4 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let Some(priority) = crate::core::override_layer::PriorityMode::from_u8(priority) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        // Anchor the NON one-shot to "from the moment of override": remember the
        // set frame (its own keyframe is skipped through the update of the first
        // evaluation).
        let set_frame = self.override_set_frame();
        let entry = self.override_slot(part)?.get_or_insert_with(Default::default);
        entry.color = Some(crate::core::override_layer::ColorOverride { blend_op, corners_rgba, rate: 1.0, priority, set_frame, first_eval_update: None });
        Ok(())
    }

    /// Clear the color override on a part.
    pub fn clear_part_color_override(&mut self, part: u32) -> Result<(), error::SSRuntimeError> {
        self.clear_override(part, |entry| entry.color = None)
    }

    /// Set a per-part cell override, identified by `(cellmap_name_hash,
    /// cell_name_hash)` (FNV-1a of the cellmap name without `.ssce` and the cell
    /// name). `(0, 0)` reverts to the animation. `priority`: `0=NON 1=NOW 2=
    /// PERMANENT`. Normal + Mask parts only. The cell is resolved against the
    /// bound SSAB now and cached; errors if it cannot be resolved.
    pub fn set_part_cell_override(&mut self, part: u32, cellmap_name_hash: u32, cell_name_hash: u32, priority: u8) -> Result<(), error::SSRuntimeError> {
        if cellmap_name_hash == 0 && cell_name_hash == 0 {
            return self.clear_part_cell_override(part);
        }
        let Some(priority) = crate::core::override_layer::PriorityMode::from_u8(priority) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        if part as usize >= self.override_part_count() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(resource.get_ssab_data()) };
        let Some(resolved) = ssab.cellmaps().and_then(|cellmaps| crate::core::framedata::resolve_cell(&cellmaps, cellmap_name_hash, cell_name_hash)) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        let set_frame = self.override_set_frame();
        let entry = self.override_slot(part)?.get_or_insert_with(Default::default);
        entry.cell = Some(crate::core::override_layer::CellOverride { cellmap_name_hash, cell_name_hash, resolved, priority, set_frame, first_eval_update: None });
        Ok(())
    }

    /// Clear the cell override on a part.
    pub fn clear_part_cell_override(&mut self, part: u32) -> Result<(), error::SSRuntimeError> {
        self.clear_override(part, |entry| entry.cell = None)
    }

    /// Clear every per-part override.
    pub fn clear_all_part_overrides(&mut self) {
        for entry in self.instance.part_overrides.iter_mut() {
            *entry = None;
        }
    }

    pub fn setup_animation(&mut self, name: &str) -> Result<(), error::SSRuntimeError> {
        let hash = fnv1a_hash_str(name);
        self.setup_animation_by_hash(hash)
    }

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
        let cursor_c = crate::core::framedata::cursor_count(metadata.cached_parts());

        // A new setup plays one source alone, extracted straight into the
        // resolved layer; the source's own layer is sized when a mix needs it.
        for layer in self.instance.layers.iter_mut() {
            *layer = LayerState::new();
        }
        self.instance.resolved_layer.resize_for_parts(num_parts, cursor_c, cell_c, color_c, shader_c, deform_c);

        // Override Layer: a new animation clears Visibility and NON/NOW
        // Color/Cell overrides but keeps PERMANENT ones.
        self.instance.retain_permanent_overrides_on_setup(num_parts);

        let globals = &mut self.instance.globals;
        globals.world_matrices.clear();
        globals.world_matrices.resize(num_parts, crate::core::SsMatrix4x4::IDENTITY);
        globals.inheritance_matrices.clear();
        globals.inheritance_matrices.resize(num_parts, crate::core::SsMatrix4x4::IDENTITY);
        globals.local_uvs.clear();
        globals.local_uvs.resize(num_parts * 10, 0.0);
        // Shape buffers keep their `part_index * SHAPE_VERTICES_STRIDE` layout, but
        // only a pack with a shape part has anything to put in them.
        let shape_len = if metadata.cached_parts().iter().any(|p| p.shape_kind.is_some()) { num_parts * crate::core::framedata::SHAPE_VERTICES_STRIDE } else { 0 };
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
        resolved.vertex_vec.reserve(parts.iter().filter(|p| p.run(crate::core::framedata::IDX_ATTRIBUTECELL).is_some()).count());
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
        if self.instance.render_mode == crate::core::RenderMode::Direct {
            let total_vertices = parts.iter().map(|p| p.mesh_cell.as_ref().map_or(0, |m| m.num_vertices())).sum();
            reserve_total(&mut globals.mesh_vertices_x, total_vertices);
            reserve_total(&mut globals.mesh_vertices_y, total_vertices);
        }
    }

    /// Put `name` on `track`, replacing whatever it was playing.
    ///
    /// Tracks are addressed by index and grow on demand: setting track 3 creates
    /// 0..=3, the new ones silent. Track 0 is the structure track — its animation
    /// supplies every discrete attribute and drives events — so it must be set up
    /// first (`setup_animation` does exactly that), and this errors with
    /// `AnimationNotFound` until it has been.
    ///
    /// The animation must live in the currently bound `AnimationResource`
    /// (same-SSAB constraint). Cancels any fade in flight on the track.
    pub fn set_track_animation(&mut self, track: u32, name: &str) -> Result<(), error::SSRuntimeError> {
        self.set_track_animation_by_hash(track, fnv1a_hash_str(name))
    }

    pub fn set_track_animation_by_hash(&mut self, track: u32, hash: u32) -> Result<(), error::SSRuntimeError> {
        let index = self.resolve_animation(hash)?;
        self.ensure_track(track)?;
        let handle = self.attach_source(track, index, 1.0)?;
        self.replace_track_main(track, handle);
        Ok(())
    }

    /// The strength of a track, i.e. how much of it survives onto the pose the
    /// tracks below produced. **This is the knob that always means something**: a
    /// source's `weight` only decides proportions among the sources sharing one
    /// track, and is silent when a source is the only one animating an attribute.
    ///
    /// Alpha is not clamped — values above 1 extrapolate, which is occasionally
    /// what an overlay wants — but `alpha <= 0` makes the track contribute
    /// nothing. Track 0 at alpha 0 resolves to the setup pose, which is how an
    /// animation fades out to rest.
    pub fn set_track_alpha(&mut self, track: u32, alpha: f32) -> Result<(), error::SSRuntimeError> {
        let t = self.instance.tracks.get_mut(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if alpha.is_nan() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        t.alpha = alpha;
        Ok(())
    }

    pub fn get_track_alpha(&self, track: u32) -> Option<f32> {
        self.instance.tracks.get(track as usize).map(|t| t.alpha)
    }

    /// How the track folds onto what is below: [`TrackBlend::Mix`] lerps toward
    /// the track's pose, [`TrackBlend::Additive`] adds the track's offset from the
    /// setup pose. Additive is for overlays that displace a performance rather
    /// than replace it.
    pub fn set_track_blend(&mut self, track: u32, blend: TrackBlend) -> Result<(), error::SSRuntimeError> {
        let t = self.instance.tracks.get_mut(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        t.blend = blend;
        Ok(())
    }

    pub fn get_track_blend(&self, track: u32) -> Option<TrackBlend> {
        self.instance.tracks.get(track as usize).map(|t| t.blend)
    }

    /// Where the animations laid on `track` start: [`TrackSync::Free`] from the
    /// head of their section, [`TrackSync::Start`] at the phase track 0 stands
    /// at. It applies to everything laid on the track from then on —
    /// `set_track_animation`, `crossfade` and `add_track_source` — so set it
    /// before laying, and a track with nothing on it yet is created for it.
    /// Like the rest of a track it goes with the stack on the next setup.
    pub fn set_track_sync(&mut self, track: u32, sync: TrackSync) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.tracks[track as usize].sync = sync;
        Ok(())
    }

    pub fn get_track_sync(&self, track: u32) -> Option<TrackSync> {
        self.instance.tracks.get(track as usize).map(|t| t.sync)
    }

    /// Number of tracks that exist, including silent ones. Always at least 1.
    pub fn get_track_count(&self) -> u32 {
        self.instance.tracks.len() as u32
    }

    /// Silence a track: every source on it is deactivated and its fade cancelled.
    /// Track 0 cannot be cleared — it carries the structure, and an instance with
    /// no structure has no pose to emit. Fade it to alpha 0 instead.
    pub fn clear_track(&mut self, track: u32) -> Result<(), error::SSRuntimeError> {
        if track == 0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let t = self.instance.tracks.get_mut(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        t.main = NO_SOURCE;
        t.fade = None;
        for source in self.instance.sources.iter_mut() {
            if source.track as u32 == track {
                source.deactivate();
            }
        }
        Ok(())
    }

    /// Fade `track` to `name` over `duration_ms` (the same time unit as
    /// [`Context::update`]'s `delta`). The fade starts from whatever the track is
    /// already playing, so there are no handles to pair up and nothing to clean up
    /// afterwards: on completion the outgoing sources are deactivated.
    ///
    /// **The incoming animation becomes the track's main one the moment the fade
    /// begins**, at weight 0 — on track 0 that makes it the primary, so the
    /// frame, `is_playing` / `is_finished` / `just_looped`, the playback settings,
    /// the discrete attributes and the events all speak for it from the call on.
    /// The host has said where the character is going, and the outgoing side is
    /// only its pose fading away: it neither reports state nor fires events
    /// (unless its event threshold is lowered). Handing over on completion
    /// instead let an outgoing animation that ran out mid-fade report the
    /// character finished, and then playing again once the fade completed.
    ///
    /// **Everything the track carries fades out together**, in the proportions it
    /// carries them: the main animation, any `add_track_source` sources, and
    /// whatever a cancelled fade left behind. Their weights are normalised to sum
    /// to 1 when the fade begins and scaled down by `1 - t` from there, so the
    /// pose does not jump when the fade begins or when it completes.
    ///
    /// Starting a crossfade while one is running on the same track folds the one
    /// in flight into the new outgoing side, at the weights it has reached. There
    /// is never more than one outgoing side, so fades restarted faster than they
    /// run do not pile up, and every outgoing source is released on completion.
    pub fn crossfade(&mut self, track: u32, name: &str, duration_ms: f32) -> Result<(), error::SSRuntimeError> {
        self.crossfade_by_hash(track, fnv1a_hash_str(name), duration_ms)
    }

    pub fn crossfade_by_hash(&mut self, track: u32, hash: u32, duration_ms: f32) -> Result<(), error::SSRuntimeError> {
        if duration_ms.is_nan() || duration_ms <= 0.0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let index = self.resolve_animation(hash)?;
        self.ensure_track(track)?;

        let to = self.attach_source(track, index, 0.0)?;
        let outgoing: f32 = self.instance.sources.iter().enumerate().filter(|&(i, s)| i as u32 != to && s.track as u32 == track && s.has_real_animation()).map(|(_, s)| s.weight.max(0.0)).sum();
        if outgoing <= 0.0 {
            // Nothing audible to fade from — this is just "start playing".
            self.instance.sources[to as usize].weight = 1.0;
            self.replace_track_main(track, to);
            return Ok(());
        }

        // Normalise the outgoing side once, here, so the fade only has to scale it.
        // Left as they were, a cancelled fade's leftovers or a host's weights of 2
        // would change their share of the pose the moment the incoming one rose.
        for (i, source) in self.instance.sources.iter_mut().enumerate() {
            if i as u32 != to && source.track as u32 == track && source.has_real_animation() {
                source.outgoing_share = source.weight.max(0.0) / outgoing;
                source.weight = source.outgoing_share;
            }
        }
        let t = &mut self.instance.tracks[track as usize];
        t.main = to;
        t.fade = Some(TrackFade::Cross { to, elapsed: 0.0, duration: duration_ms });
        if track == 0 {
            self.instance.primary = to;
        }
        Ok(())
    }

    /// Drive a track's `alpha` to `target` over `duration_ms`. Fading track 0 to 0
    /// resolves the instance to its setup pose, which is what "fade out to rest"
    /// means here — no placeholder animation required.
    pub fn fade_track(&mut self, track: u32, target: f32, duration_ms: f32) -> Result<(), error::SSRuntimeError> {
        if duration_ms.is_nan() || duration_ms <= 0.0 || target.is_nan() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let t = self.instance.tracks.get_mut(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        t.fade = Some(TrackFade::Alpha { start: t.alpha, target, elapsed: 0.0, duration: duration_ms });
        Ok(())
    }

    /// Progress of the fade in flight on `track`, in `[0.0, 1.0]`; `None` when the
    /// track is not fading.
    pub fn get_track_fade_progress(&self, track: u32) -> Option<f32> {
        self.instance.tracks.get(track as usize)?.fade.as_ref().map(TrackFade::progress)
    }

    /// Cancel the fade on `track`, leaving alpha and weights where they stand.
    pub fn cancel_track_fade(&mut self, track: u32) -> Result<(), error::SSRuntimeError> {
        let t = self.instance.tracks.get_mut(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        t.fade = None;
        Ok(())
    }

    /// Seek the track's current animation. This is how a modulation axis is
    /// driven: author the axis as an animation and scrub it with the tracked
    /// value.
    pub fn set_track_frame(&mut self, track: u32, frame: f32) -> Result<(), error::SSRuntimeError> {
        let handle = self.track_main(track)?;
        self.instance.sources[handle as usize].controller.set_frame_no(frame);
        Ok(())
    }

    pub fn get_track_frame(&self, track: u32) -> Option<f32> {
        let handle = self.instance.tracks.get(track as usize)?.main;
        let source = self.instance.sources.get(handle as usize)?;
        (source.animation_idx >= 0).then(|| source.controller.get_frame_no())
    }

    /// The speed of `track`, as a multiple of each animation's frame rate. Kept
    /// by the track: it reaches every source on it now and every one laid on it
    /// later, so a crossfade or a `set_track_animation` does not reset it.
    /// Track 0's is [`Self::set_speed_scale`]. Like every track setting it goes
    /// with the track when `setup_animation` collapses the stack, except on
    /// track 0.
    pub fn set_track_speed(&mut self, track: u32, speed: f32) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.set_track_policy(track, |p| p.set_speed_scale(speed), |c| c.set_speed_scale(speed));
        Ok(())
    }

    /// What [`Self::set_track_speed`] stored: a negative speed reads back as 0.
    pub fn get_track_speed(&self, track: u32) -> Option<f32> {
        Some(self.instance.tracks.get(track as usize)?.policy.speed_scale)
    }

    /// The frame rate `track` plays its animations at instead of the rate they
    /// were authored at; `0` or a negative rate clears it. Kept by the track,
    /// as [`Self::set_track_speed`] is.
    pub fn set_track_frame_rate(&mut self, track: u32, frame_rate: i32) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.set_track_policy(track, |p| p.set_frame_rate(frame_rate), |c| c.set_frame_rate(frame_rate));
        Ok(())
    }

    /// The rate `track`'s animation plays at: the override when one is set,
    /// otherwise the rate it was authored at. `None` for a track that does not
    /// exist; `0` for one with no animation and no override.
    pub fn get_track_frame_rate(&self, track: u32) -> Option<i32> {
        let policy = self.instance.tracks.get(track as usize)?.policy;
        Some(self.instance.track_main_controller(track).map_or(policy.frame_rate_override.unwrap_or(0), |c| c.get_frame_rate()))
    }

    /// Whether `track` skips frames to keep up. Kept by the track, as
    /// [`Self::set_track_speed`] is.
    pub fn set_track_skip_frames(&mut self, track: u32, skip: bool) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.set_track_policy(track, |p| p.is_skip_frames = skip, |c| c.set_skip_frames(skip));
        Ok(())
    }

    pub fn get_track_skip_frames(&self, track: u32) -> Option<bool> {
        Some(self.instance.tracks.get(track as usize)?.policy.is_skip_frames)
    }

    /// The direction and style `track` plays its animations in. Kept by the
    /// track, as [`Self::set_track_speed`] is.
    pub fn set_track_playback_direction(&mut self, track: u32, direction: core::playback_controller::PlaybackDirection, style: core::playback_controller::PlaybackStyle) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.set_track_policy(
            track,
            |p| {
                p.playback_direction = direction;
                p.playback_style = style;
            },
            |c| c.set_playback_direction(direction, style),
        );
        Ok(())
    }

    pub fn get_track_playback_direction(&self, track: u32) -> Option<core::playback_controller::PlaybackDirection> {
        Some(self.instance.tracks.get(track as usize)?.policy.playback_direction)
    }

    pub fn get_track_playback_style(&self, track: u32) -> Option<core::playback_controller::PlaybackStyle> {
        Some(self.instance.tracks.get(track as usize)?.policy.playback_style)
    }

    /// How many times `track`'s animation plays before it holds its last frame;
    /// `0` or a negative count loops forever. Unlike the speed this belongs to
    /// the animation on the track, not the track: laying another animation
    /// there starts it looping forever again, so state the count after laying
    /// it. Track 0's is [`Self::set_loop_count`].
    pub fn set_track_loop_count(&mut self, track: u32, loop_count: i32) -> Result<(), error::SSRuntimeError> {
        let handle = self.track_main(track)?;
        self.instance.sources[handle as usize].controller.set_loop_count(loop_count);
        Ok(())
    }

    /// `-1` for a track that loops forever; `None` for one with no animation.
    pub fn get_track_loop_count(&self, track: u32) -> Option<i32> {
        Some(self.instance.track_main_controller(track)?.get_loop_count())
    }

    /// `true` once `track`'s animation has played its loop count out and holds
    /// its last frame, as [`Self::is_finished`] is for track 0. A finished track
    /// keeps mixing that frame in until the host fades or clears it. `false` for
    /// a track with no animation.
    pub fn is_track_finished(&self, track: u32) -> bool {
        self.instance.track_main_controller(track).is_some_and(|c| c.is_finished())
    }

    /// `true` for the update in which `track`'s animation wrapped back to its
    /// start, as [`Self::just_looped`] is for track 0.
    pub fn track_just_looped(&self, track: u32) -> bool {
        self.instance.track_main_controller(track).is_some_and(|c| c.just_looped())
    }

    /// Handle of the source carrying `track`'s animation. Needed to give that
    /// animation a weight of its own once a track holds several — the animation
    /// put there by `set_track_animation` starts at weight 1.
    pub fn get_track_main_source(&self, track: u32) -> Option<u32> {
        let main = self.instance.tracks.get(track as usize)?.main;
        (main != NO_SOURCE).then_some(main)
    }

    /// Add another weighted source to `track`, for the case a track needs several
    /// animations blended by proportion (locomotion: walk + run). Returns a stable
    /// handle for [`Self::set_source_weight`].
    ///
    /// Most tracks never need this — one animation plus the track's `alpha`
    /// covers layering, and `crossfade` covers transitions.
    pub fn add_track_source(&mut self, track: u32, name: &str, weight: f32) -> Result<u32, error::SSRuntimeError> {
        self.add_track_source_by_hash(track, fnv1a_hash_str(name), weight)
    }

    pub fn add_track_source_by_hash(&mut self, track: u32, hash: u32, weight: f32) -> Result<u32, error::SSRuntimeError> {
        let index = self.resolve_animation(hash)?;
        self.ensure_track(track)?;
        let handle = self.attach_source(track, index, weight)?;
        // The first animation to land on a track becomes its main one, so callers
        // may build a track entirely out of `add_track_source` calls.
        if self.instance.tracks[track as usize].main == NO_SOURCE {
            self.replace_track_main(track, handle);
        }
        Ok(handle)
    }

    /// Proportion of one source **within its track**. See [`Self::set_track_alpha`]
    /// for the strength of the track as a whole.
    pub fn set_source_weight(&mut self, handle: u32, weight: f32) -> Result<(), error::SSRuntimeError> {
        let source = self.instance.sources.get_mut(handle as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if source.animation_idx < 0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        source.weight = weight;
        Ok(())
    }

    pub fn get_source_weight(&self, handle: u32) -> Option<f32> {
        let source = self.instance.sources.get(handle as usize)?;
        (source.animation_idx >= 0).then_some(source.weight)
    }

    pub fn set_source_frame(&mut self, handle: u32, frame: f32) -> Result<(), error::SSRuntimeError> {
        let source = self.instance.sources.get_mut(handle as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if source.animation_idx < 0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        source.controller.set_frame_no(frame);
        Ok(())
    }

    /// Which animation `handle` plays, as an index into the pack's animations:
    /// the order of [`AnimationResource::get_animation_names`] and of the
    /// ssab's `animations()`. `None` for a handle that names no source.
    ///
    /// An index from [`Self::get_passed_event_indices`] addresses the `events()`
    /// of the animation of the source that fired it, so this is how a Player
    /// decodes an event. Track 0's included: a crossfade on track 0 hands it to
    /// another animation, and a table kept from `setup_animation` is then the
    /// wrong one.
    pub fn get_source_animation_index(&self, handle: u32) -> Option<u32> {
        let source = self.instance.sources.get(handle as usize)?;
        source.has_real_animation().then_some(source.animation_idx as u32)
    }

    /// Which animation track 0's main source plays, as an index into the pack's
    /// animations: the one `setup_animation` set up, until a `crossfade` or a
    /// `set_track_animation` on track 0 hands the track to another, from the
    /// call on. `None` before anything is set up.
    pub fn get_current_animation_index(&self) -> Option<u32> {
        self.get_source_animation_index(self.instance.primary)
    }

    pub fn get_source_frame(&self, handle: u32) -> Option<f32> {
        let source = self.instance.sources.get(handle as usize)?;
        (source.animation_idx >= 0).then(|| source.controller.get_frame_no())
    }

    /// Per-source event firing threshold. A source fires events once its weight
    /// reaches the threshold. The default is `+∞`, so by default no source
    /// fires but track 0's main source, which always does regardless of this
    /// setting. Distinguish sources with
    /// [`Self::get_passed_event_source_handles`].
    pub fn set_source_event_threshold(&mut self, handle: u32, threshold: f32) -> Result<(), error::SSRuntimeError> {
        let source = self.instance.sources.get_mut(handle as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if !source.has_real_animation() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        source.event_threshold = threshold;
        Ok(())
    }

    pub fn get_source_event_threshold(&self, handle: u32) -> Option<f32> {
        let source = self.instance.sources.get(handle as usize)?;
        source.has_real_animation().then_some(source.event_threshold)
    }

    /// How far into a crossfade on `track` the outgoing side keeps firing
    /// events. It fires while its share of the track, `1 - t`, is at least
    /// `threshold`, however many sources it holds: `0.5` keeps it firing for the
    /// first half of the fade. The default, `+∞`, keeps it silent from the moment
    /// the fade begins, which is where the incoming animation starts firing.
    /// The outgoing sources' own thresholds are not consulted while they fade.
    ///
    /// A setting of the track rather than of a source, since the outgoing side
    /// can be several sources, released when the fade completes; it applies to
    /// every crossfade on the track until it is changed. Track 0 keeps it
    /// across `setup_animation`, as it keeps its playback settings.
    pub fn set_track_outgoing_event_threshold(&mut self, track: u32, threshold: f32) -> Result<(), error::SSRuntimeError> {
        self.ensure_track(track)?;
        self.instance.tracks[track as usize].outgoing_event_threshold = threshold;
        Ok(())
    }

    pub fn get_track_outgoing_event_threshold(&self, track: u32) -> Option<f32> {
        Some(self.instance.tracks.get(track as usize)?.outgoing_event_threshold)
    }

    // ---- internals shared by the track API ---------------------------------

    /// Build the part bases for this instance's coordinate setup now, not on the
    /// first frame. Runs when the animation or a coordinate setting changes.
    fn prepare_part_bases(&mut self) {
        let Some(resource) = self.eval.resource else {
            return;
        };
        let anim_idx = self.instance.primary_source().animation_idx;
        if anim_idx < 0 {
            return;
        }
        resource.animations[anim_idx as usize].part_bases(self.instance.coordinate_system, self.instance.origin, self.instance.unit);
    }

    fn resolve_animation(&self, hash: u32) -> Result<usize, error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };
        resource.find_animation_index_by_hash(hash).ok_or(error::SSRuntimeError::AnimationNotFound)
    }

    /// Grow the stack so `track` exists. Every track but 0 may be created this
    /// way; track 0 exists from construction, but until `setup_animation` has run
    /// there is no structure to layer onto, so this reports `AnimationNotFound`.
    fn ensure_track(&mut self, track: u32) -> Result<(), error::SSRuntimeError> {
        if track != 0 && !self.instance.sources[self.instance.primary_idx()].has_real_animation() {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }
        if track as usize >= self.instance.tracks.len() {
            self.instance.tracks.resize(track as usize + 1, TrackState::new());
        }
        Ok(())
    }

    /// Park a new source on `track`, reusing a slot left behind by a finished
    /// fade when one is free so a long-lived instance does not grow a slot per
    /// transition. Handles stay stable: a slot is only reused once inactive.
    fn attach_source(&mut self, track: u32, animation_idx: usize, weight: f32) -> Result<u32, error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };
        let metadata = &resource.animations[animation_idx];
        let num_parts = metadata.cached_parts().len();
        let (cell_c, color_c, shader_c, deform_c) = count_attribute_capacities(metadata);
        let cursor_c = crate::core::framedata::cursor_count(metadata.cached_parts());

        metadata.part_bases(self.instance.coordinate_system, self.instance.origin, self.instance.unit);
        let mut source = SourceState::new();
        // The track's policy, not the defaults: a laid animation is played the
        // way the host plays that track. Before `setup`, which derives the
        // heading and the frame interval from it.
        source.controller.adopt_policy(&self.instance.tracks[track as usize].policy);
        source.controller.setup(metadata.start_frame, metadata.end_frame, metadata.fps);
        // A source laid on a track plays from the moment it is attached; the
        // character's clock decides whether it actually moves. Left stopped, an
        // overlay sat on its first frame and a completed crossfade froze the
        // character, since nothing but `play` ever started a controller.
        source.controller.play();
        if self.instance.tracks[track as usize].sync == TrackSync::Start
            && let Some(phase) = self.primary_phase()
        {
            let c = &mut source.controller;
            let frame = c.get_start_frame() as f32 + phase * (c.get_end_frame() - c.get_start_frame() + 1) as f32;
            c.set_frame_no(frame);
        }
        source.animation_idx = animation_idx as i32;
        source.weight = weight;
        source.track = track as u16;

        let free = self.instance.sources.iter().position(|s| s.animation_idx < 0);
        let handle = match free {
            Some(i) => {
                self.instance.sources[i] = source;
                self.instance.layers[i].resize_for_parts(num_parts, cursor_c, cell_c, color_c, shader_c, deform_c);
                i
            }
            None => {
                self.instance.sources.push(source);
                let mut layer = LayerState::new();
                layer.resize_for_parts(num_parts, cursor_c, cell_c, color_c, shader_c, deform_c);
                self.instance.layers.push(layer);
                self.instance.sources.len() - 1
            }
        };
        Ok(handle as u32)
    }

    /// How far through its section track 0's main animation stands, in
    /// `[0.0, 1.0)`; `None` with nothing set up. Measured by position, not by
    /// progress, so a backward or ping-pong return leg reads where it stands.
    /// During a crossfade on track 0 this is the incoming animation, which takes
    /// the main role when the fade begins; the source a crossfade is laying
    /// is attached before that, so it reads the outgoing one it is replacing.
    fn primary_phase(&self) -> Option<f32> {
        let source = self.instance.primary_source();
        if !source.has_real_animation() {
            return None;
        }
        let c = &source.controller;
        let length = (c.get_end_frame() - c.get_start_frame() + 1) as f32;
        Some(((c.get_frame_no() - c.get_start_frame() as f32) / length).clamp(0.0, 1.0))
    }

    /// Make `handle` the track's animation, retiring whatever held that role.
    fn replace_track_main(&mut self, track: u32, handle: u32) {
        let previous = self.instance.tracks[track as usize].main;
        if previous != NO_SOURCE && previous != handle {
            self.instance.sources[previous as usize].deactivate();
        }
        // Any other source on this track belonged to the animation being replaced.
        for (i, source) in self.instance.sources.iter_mut().enumerate() {
            if i as u32 != handle && source.track as u32 == track {
                source.deactivate();
            }
        }
        let t = &mut self.instance.tracks[track as usize];
        t.main = handle;
        t.fade = None;
        if track == 0 {
            self.instance.primary = handle;
        }
    }

    fn track_main(&self, track: u32) -> Result<u32, error::SSRuntimeError> {
        let t = self.instance.tracks.get(track as usize).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if t.main == NO_SOURCE {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        Ok(t.main)
    }

    pub fn get_frame_data(&mut self, frame_no: f32) -> Result<&[u8], error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };

        let primary_anim_idx = self.instance.primary_source().animation_idx;
        if primary_anim_idx < 0 {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }

        let primary_metadata = &resource.animations[primary_anim_idx as usize];
        if !((primary_metadata.start_frame as f32) <= frame_no && frame_no < (primary_metadata.end_frame as f32 + 1.0)) {
            return Err(error::SSRuntimeError::InvalidParameter);
        }

        if self.eval.fb_need > self.eval.fb_capacity {
            // The last frame outgrew the builder, which doubled. Size it to the
            // largest frame so far, with a sixteenth to spare for frames that
            // carry a little more; the old buffer is released. The builder
            // writes back to front from the end of its buffer, so the size sets
            // where the frame data starts: a multiple of 8 keeps it on the
            // boundary `PartState` needs, as the doubled sizes did.
            self.eval.fb_capacity = (self.eval.fb_need + self.eval.fb_need / 16).next_multiple_of(8);
            self.eval.fb = flatbuffers::FlatBufferBuilder::with_capacity(self.eval.fb_capacity);
        }
        self.eval.fb.reset();
        self.eval.frame_data_ready = false;

        let coordinate_system = self.instance.coordinate_system;
        let origin = self.instance.origin;
        let unit = self.instance.unit;

        // Stage A: per-source pose extraction. Each source uses its own
        // controller-driven frame; track 0's main source uses the explicit
        // `frame_no` argument to preserve single-animation behavior. That source
        // is always extracted regardless of its weight — `mix_tracks` uses it as
        // the discrete-attribute / structure source even when its continuous
        // contribution is zero.
        let primary_idx = self.instance.primary_idx();
        let primary_num_parts = primary_metadata.cached_parts().len();
        let num_sources = self.instance.sources.len();
        // One audible source at full alpha resolves to its own pose verbatim, so
        // it is extracted straight into the resolved layer and the two-level fold
        // is skipped. The floor is unobservable here: an attribute nothing keys
        // reads the same from the setup pose as from the extracted pose.
        let single = self.instance.is_single_full_track();
        for i in 0..num_sources {
            let source_anim_idx = self.instance.sources[i].animation_idx;
            if source_anim_idx < 0 {
                continue;
            }
            if i != primary_idx && self.instance.sources[i].weight <= 0.0 {
                continue;
            }
            let frame_for_source = if i == primary_idx { frame_no } else { self.instance.sources[i].controller.get_frame_no() };
            let metadata = &resource.animations[source_anim_idx as usize];
            let into_resolved = single && i == primary_idx;
            let layer = if into_resolved { &mut self.instance.resolved_layer } else { &mut self.instance.layers[i] };
            layer.fit(metadata);
            core::framedata::extract_pose(
                &mut self.eval.fb,
                metadata.cached_parts(),
                &resource.source.keys,
                frame_for_source,
                &mut layer.parts_vec,
                &mut layer.cell_vec,
                &mut layer.parts_color_vec,
                &mut layer.shader_vec,
                &mut layer.deform_vec,
                &mut layer.raw_vertex_deform,
                &mut layer.cursors,
                Some(metadata.part_bases(coordinate_system, origin, unit)),
                &mut layer.cell_meta_per_part,
                &mut layer.cell_texture_hashes,
                &mut layer.cell_ids,
                &mut layer.mesh_deform,
                coordinate_system,
                origin,
                unit,
                metadata.canvas_width,
                metadata.canvas_height,
                &mut self.eval.deform_interpolation_start_workspace,
                &mut self.eval.deform_interpolation_end_workspace,
                &mut self.eval.deform_interpolation_out_workspace,
                &mut self.eval.deform_result_coordinate_workspace,
                &mut self.eval.deform_result_index_workspace,
                &mut self.eval.inheritance_alphas_workspace,
            );

            // Record which attributes this animation actually carries, per part,
            // for the mix's coverage rule. Animation-level and therefore constant
            // for the source's lifetime; copied per frame because a source can be
            // repointed at another animation at any time. The resolved layer is
            // never mixed from, so it needs none.
            if !into_resolved {
                layer.coverage_vec.clear();
                layer.coverage_vec.extend(metadata.cached_parts().iter().map(|p| p.coverage));
            }
        }

        if !single {
            mix_tracks(&self.instance.sources, &self.instance.layers, &self.instance.tracks, primary_idx, primary_metadata.setup_pose(coordinate_system, origin, unit), &mut self.instance.resolved_layer);
        }

        // ⟵ SEAM: apply the per-part runtime Override Layer on the resolved
        // pose — after the mix, before compose — so overrides win over both
        // keyframe values and blend results.
        if self.instance.has_any_override() {
            // (Re)size + clear the cell-UV meta so the seam records only the
            // parts it overrides this frame; compute_local_uvs consults it.
            if self.instance.override_cell_uv_meta.len() != primary_num_parts {
                self.instance.override_cell_uv_meta.resize(primary_num_parts, None);
            }
            for slot in self.instance.override_cell_uv_meta.iter_mut() {
                *slot = None;
            }
            // NON handover is decided from the frames the primary actually
            // traversed this update (section / loop / reverse / ping-pong
            // correct). Copy them out before the mutable borrows below so the
            // controller borrow is released.
            self.eval.override_passed_frames_workspace.clear();
            self.eval.override_passed_frames_workspace.extend_from_slice(self.instance.primary_source().controller.get_passed_frames());
            crate::core::override_layer::apply_part_overrides(
                &mut self.instance.part_overrides,
                primary_metadata.cached_parts(),
                &resource.source.keys,
                &mut self.instance.resolved_layer.parts_vec,
                &mut self.instance.resolved_layer.parts_color_vec,
                &mut self.instance.resolved_layer.cell_vec,
                &mut self.instance.resolved_layer.cell_meta_per_part,
                &mut self.instance.resolved_layer.cell_texture_hashes,
                &mut self.instance.override_cell_uv_meta,
                &mut self.eval.override_hide_workspace,
                &self.eval.override_passed_frames_workspace,
                self.instance.updates,
            );
        } else {
            // No overrides active: leave the buffer empty so compute_local_uvs
            // falls back to the keyframe cell for every part.
            self.instance.override_cell_uv_meta.clear();
        }

        // Stage B (`compute_local_vertices_for_parts`), Stage C
        // (`serialize_pose_to_fb`) and the draw plan, on the resolved pose.
        let nines = &mut self.instance.globals.nines;
        let resolved = &mut self.instance.resolved_layer;
        // A track-0 change can hand the primary role to another animation.
        self.instance.z_order.fit(primary_anim_idx, primary_metadata.cached_parts());
        let offset = core::framedata::compose_frame_data(
            &mut self.eval.fb,
            primary_metadata.cached_parts(),
            &mut self.instance.z_order,
            &mut resolved.parts_vec,
            &resolved.cell_vec,
            &resolved.parts_color_vec,
            &resolved.shader_vec,
            &mut resolved.vertex_vec,
            &resolved.deform_vec,
            &resolved.raw_vertex_deform,
            &resolved.cell_meta_per_part,
            &resolved.cell_texture_hashes,
            &mut resolved.local_vertices,
            nines,
            coordinate_system,
            self.instance.render_mode,
            &mut self.eval.draw_order_workspace,
            &mut self.eval.batches_workspace,
            &mut self.eval.mask_indices_workspace,
        );

        // Phase 2: resolve global state (matrices / UVs / shape vertices)
        // from the (post-mix) local pose.
        Self::resolve_global_state(&mut self.instance, primary_metadata);

        self.eval.fb.finish_minimal(offset);
        self.eval.frame_data_ready = true;
        self.eval.fb_need = self.eval.fb_need.max(self.eval.fb.finished_data().len());
        Ok(self.eval.fb.finished_data())
    }

    /// Phase 2 of the frame pipeline. Consumes the (post-mix) local pose
    /// in `instance.resolved_layer` and writes per-instance global outputs
    /// in `instance.globals` — world / inheritance matrices, local UVs,
    /// and shape vertex buffers.
    fn resolve_global_state(instance: &mut InstanceState<'a>, metadata: &AnimationMetadata<'a>) {
        if instance.render_mode == crate::core::RenderMode::Direct {
            Self::compute_matrices(instance, metadata.cached_parts());
            // Skinned mesh skinning depends on world matrices, so it lives in the
            // Direct path. Mesh parts in SceneGraph mode get empty per-part ranges.
            Self::compute_mesh_vertices(instance, metadata);
        } else {
            Self::clear_mesh_vertices(instance, metadata.cached_parts().len());
        }
        Self::compute_local_uvs(instance, metadata);
        Self::compute_shape_vertices(instance, metadata.cached_parts());
    }

    /// Writes per-part mesh world-space vertex positions to
    /// `globals.mesh_vertices_x` / `_y` for the current frame — skeletal
    /// (bone-weighted) and Deform-only meshes alike, with the frame's deform
    /// deltas applied.
    ///
    /// Layout: x/y are SoA, sliced per part by `mesh_vertex_offsets` (a CSR
    /// prefix sum of length `num_parts + 1`; non-mesh parts get an empty
    /// range). The CSR offsets and the static UV / index buffers are the
    /// animation's (`StaticMeshBuffers`); only the vertex positions are
    /// recomputed each frame.
    #[allow(clippy::needless_range_loop)]
    fn compute_mesh_vertices(instance: &mut InstanceState<'a>, metadata: &AnimationMetadata<'a>) {
        let num_parts = instance.resolved_layer.parts_vec.len();
        let cached_parts = metadata.cached_parts();

        // The CSR offsets and the static UV / index buffers depend only on the
        // animation's per-part Cells (fixed by the SS mesh spec). Take the
        // offsets and size the SoA position buffers once per layout;
        // `setup_animation` resets `mesh_layout` so a new animation sets up again.
        if instance.globals.mesh_layout != metadata.mesh_layout {
            Self::setup_static_mesh_buffers(instance, metadata);
        }

        let layer = &instance.resolved_layer;
        // SSAB bind coordinates are authored Y-up; flip when the instance
        // (hence the bone world matrices) runs Y-down.
        let flip_y = instance.coordinate_system == crate::core::SsCoordinateSystem::YDown;
        // The Deform-only mesh rest comes from the cell's `table_coord_x` /
        // `table_coord_y`, which ssconverter copies verbatim from the `.ssce`
        // meshPointList — texture-pixel space, i.e. already Y-down. So it must be
        // flipped for a Y-up target, the OPPOSITE of the Y-up-native bind
        // coordinates above.
        let flip_rest = instance.coordinate_system == crate::core::SsCoordinateSystem::YUp;
        let globals = &mut instance.globals;

        let total_vertices = globals.mesh_vertices_x.len();
        if total_vertices == 0 {
            return;
        }

        for i in 0..num_parts {
            let Some(cached) = cached_parts.get(i) else {
                continue;
            };
            let start = globals.mesh_vertex_offsets[i] as usize;
            let end = globals.mesh_vertex_offsets[i + 1] as usize;
            if end <= start {
                continue; // not a mesh part
            }

            // Skeletal mesh: a bone-weighted binding drives the vertices.
            let skeletal = cached.mesh_binding.as_ref().map(|b| !b.bone_indices.is_empty()).unwrap_or(false);
            if skeletal {
                let binding = cached.mesh_binding.as_ref().unwrap();
                // The SpriteStudio `.ssae` meshList carries the N rendered mesh vertices
                // followed by 4 trailing placeholder entries (the cell-quad
                // corners, bound to no bone), so a well-formed binding has
                // `cell_count + 4` vertices. `compute_skinned_mesh_vertices`
                // skins exactly `out_x.len()` (= cell count) vertices from the
                // front of the binding, so a longer binding is fine — the
                // trailing extras are simply never read. Only a binding with
                // *fewer* vertices than the cell is unusable.
                if binding.num_vertices() < end - start {
                    continue; // bind has fewer vertices than the cell — cannot skin
                }
                let deform = layer.mesh_deform.get(i).and_then(|d| d.as_deref()).filter(|d| !d.indices.is_empty()).map(|d| (d.indices.as_slice(), d.dx.as_slice(), d.dy.as_slice(), &globals.world_matrices[i]));
                let (out_x, out_y) = (&mut globals.mesh_vertices_x[start..end], &mut globals.mesh_vertices_y[start..end]);
                crate::core::geometry::compute_skinned_mesh_vertices(&binding.influence_offsets, &binding.bone_indices, &binding.weights, &binding.coords_x, &binding.coords_y, &globals.world_matrices, deform, flip_y, out_x, out_y);
                continue;
            }

            // Deform-only mesh: no bones — rigidly transform the cell's rest
            // mesh by the part's own world matrix.
            let Some(mesh_cell) = cached.mesh_cell.as_ref() else {
                continue;
            };
            if mesh_cell.rest_x.len() != (end - start) || i >= globals.world_matrices.len() {
                continue;
            }
            // Rest transform (W · rest) — SIMD over 4 vertices at a time.
            {
                let (out_x, out_y) = (&mut globals.mesh_vertices_x[start..end], &mut globals.mesh_vertices_y[start..end]);
                crate::core::geometry::compute_rigid_mesh_vertices(&mesh_cell.rest_x, &mesh_cell.rest_y, &globals.world_matrices[i], flip_rest, out_x, out_y);
            }
            // Apply deform deltas as-is — evaluate_part has already applied any
            // coordinate-system Y conversion. Sparse scatter, kept scalar.
            if let Some(deform) = layer.mesh_deform.get(i).and_then(|d| d.as_deref()) {
                let vert_count = end - start;
                let mat = &globals.world_matrices[i];
                let (m0, m1, m4, m5) = (mat.m[0], mat.m[1], mat.m[4], mat.m[5]);
                for k in 0..deform.indices.len() {
                    let v = deform.indices[k] as usize;
                    if v < vert_count {
                        globals.mesh_vertices_x[start + v] += m0 * deform.dx[k] + m4 * deform.dy[k];
                        globals.mesh_vertices_y[start + v] += m1 * deform.dx[k] + m5 * deform.dy[k];
                    }
                }
            }
        }
    }

    /// Points the instance at the animation's static mesh buffers: copies their
    /// vertex offsets, and sizes and zeroes the SoA position buffers. A
    /// statically malformed mesh part keeps the zeros written here, so the
    /// per-frame pass needs no clear.
    fn setup_static_mesh_buffers(instance: &mut InstanceState<'a>, metadata: &AnimationMetadata<'a>) {
        let globals = &mut instance.globals;
        let mesh = metadata.static_mesh();
        globals.mesh_vertex_offsets.clone_from(&mesh.vertex_offsets);
        let total_vertices = *mesh.vertex_offsets.last().unwrap_or(&0) as usize;
        globals.mesh_vertices_x.clear();
        globals.mesh_vertices_x.resize(total_vertices, 0.0);
        globals.mesh_vertices_y.clear();
        globals.mesh_vertices_y.resize(total_vertices, 0.0);
        globals.mesh_layout = metadata.mesh_layout;
    }

    fn clear_mesh_vertices(instance: &mut InstanceState<'a>, num_parts: usize) {
        let globals = &mut instance.globals;
        if globals.mesh_vertex_offsets.len() != num_parts + 1 {
            globals.mesh_vertex_offsets.resize(num_parts + 1, 0);
        }
        for slot in globals.mesh_vertex_offsets.iter_mut() {
            *slot = 0;
        }
        globals.mesh_vertices_x.clear();
        globals.mesh_vertices_y.clear();
        globals.mesh_layout = NO_MESH_LAYOUT;
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_matrices(instance: &mut InstanceState<'a>, cached_parts: &[crate::core::framedata::CachedPartData]) {
        let layer = &instance.resolved_layer;
        let globals = &mut instance.globals;
        for i in 0..layer.parts_vec.len() {
            let parent_idx = cached_parts[i].parent_index;
            let valid_parent_idx = (parent_idx >= 0 && (parent_idx as usize) < i).then_some(parent_idx as usize);

            let parent_world = valid_parent_idx.map(|p_idx| &globals.inheritance_matrices[p_idx]);
            let parent_size = valid_parent_idx.map(|p_idx| {
                let p_state = &layer.parts_vec[p_idx];
                (p_state.size_x(), p_state.size_y())
            });

            let part_state = &layer.parts_vec[i];
            let (inh_m, draw_m) = crate::core::SsMatrix4x4::compute_world_matrix(part_state, parent_world, parent_size);
            globals.inheritance_matrices[i] = inh_m;
            globals.world_matrices[i] = draw_m;
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_local_uvs(instance: &mut InstanceState<'a>, metadata: &AnimationMetadata<'a>) {
        let keys = &metadata.source.keys;
        let layer = &instance.resolved_layer;
        let override_cell_uv_meta = &instance.override_cell_uv_meta;
        let globals = &mut instance.globals;
        let num_parts = layer.parts_vec.len();
        if globals.local_uvs.len() != num_parts * 10 {
            globals.local_uvs.resize(num_parts * 10, 0.0);
        }
        for slot in globals.local_uvs.iter_mut() {
            *slot = 0.0;
        }

        for i in 0..num_parts {
            let state = &layer.parts_vec[i];
            if state.cell() < 0 {
                continue;
            }
            // A cell override recorded at the seam wins for UVs too, so the
            // sampled image (not just the quad size/texture) reflects the swap.
            let meta = match override_cell_uv_meta.get(i).and_then(|m| m.as_ref()) {
                Some(m) => m,
                None => match layer.cell_ids.get(i).copied().flatten() {
                    Some(cell) => &keys.cell(cell).meta,
                    None => continue,
                },
            };
            if meta.size_w == 0.0 && meta.size_h == 0.0 {
                continue;
            }

            let rect_right = meta.rect_left + meta.size_w;
            let rect_bottom = meta.rect_top + meta.size_h;

            let (tu, tv, rot_deg, su, sv, fh, fv, ifh, ifv) =
                (state.uv_translation_x() * meta.tex_w, state.uv_translation_y() * meta.tex_h, state.uv_rotation_z(), state.uv_scale_x(), state.uv_scale_y(), state.flip_h(), state.flip_v(), state.img_flip_h(), state.img_flip_v());

            let (us, vs) = crate::core::geometry::compute_local_uvs((meta.rect_left, meta.rect_top, rect_right, rect_bottom), (tu, tv, rot_deg, su, sv), (fh, fv), (ifh, ifv), meta.rotated);

            let offset = i * 10;
            for k in 0..5 {
                globals.local_uvs[offset + k * 2] = us[k];
                globals.local_uvs[offset + k * 2 + 1] = vs[k];
            }
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_shape_vertices(instance: &mut InstanceState<'a>, cached_parts: &[crate::core::framedata::CachedPartData]) {
        let coordinate_system = instance.coordinate_system;
        let layer = &instance.resolved_layer;
        let globals = &mut instance.globals;
        let num_parts = layer.parts_vec.len();
        let stride = crate::core::framedata::SHAPE_VERTICES_STRIDE;
        if globals.shape_vertex_counts.len() != num_parts {
            globals.shape_vertex_counts.resize(num_parts, 0);
        }
        globals.shape_vertex_counts.fill(0);
        // `setup_animation` leaves the vertex buffers empty when no part is a shape.
        if globals.shape_vertices.len() != num_parts * stride {
            return;
        }
        globals.shape_vertices.fill(0.0);
        globals.shape_vertex_box_coords.fill(0.0);

        for i in 0..num_parts {
            let Some(shape_kind) = cached_parts[i].shape_kind else { continue };
            let state = &layer.parts_vec[i];
            let size_x = state.size_x();
            let size_y = state.size_y();
            if size_x <= 0.0 || size_y <= 0.0 {
                continue;
            }

            let (count, vx, vy) = crate::core::geometry::compute_shape_vertices(shape_kind, (size_x, size_y));
            if count < 3 {
                continue;
            }

            let mut effective_pivot_y = state.pivot_y();
            if coordinate_system == crate::core::SsCoordinateSystem::YDown {
                effective_pivot_y = -effective_pivot_y;
            }
            let tx = -state.pivot_x() * size_x;
            let ty = -effective_pivot_y * size_y;
            let hw = size_x * 0.5;
            let hh = size_y * 0.5;

            // Vertex deform: the 4 box-corner offsets ([LT, RT, LB, RB], in the
            // part's Y-up local space) are spread across every shape vertex by
            // bilinear interpolation over its box coordinates (fx, fy) — the same
            // weighting the renderer uses for per-corner PartColor. Offsets are
            // added in Y-up space, before the YDown negation, so the dy sign
            // matches the 4-corner deform path in `compute_local_vertices`.
            let deform = crate::core::framedata::vertex_deform_at(&layer.raw_vertex_deform, i);
            // Skew shares the Vertex-deform slot; applied as a shear about the
            // pivot origin in Y-up, before the Y-down flip, like the deform.
            let skew = crate::core::geometry::skew_local_tans(state.skew_x(), state.skew_y());

            let offset = i * stride;
            for v in 0..count {
                let fx = (vx[v] + hw) / size_x;
                let fy = (hh - vy[v]) / size_y;

                let (mut dvx, mut dvy) = (vx[v] + tx, vy[v] + ty);
                if let Some(d) = deform {
                    let (ddx, ddy) = crate::core::geometry::bilinear_box_deform(&d.dx, &d.dy, fx, fy);
                    dvx += ddx;
                    dvy += ddy;
                }
                if let Some((tan_x, tan_y)) = skew {
                    let (nx, ny) = crate::core::geometry::skew_point_yup(dvx, dvy, tan_x, tan_y);
                    dvx = nx;
                    dvy = ny;
                }

                let mut y = dvy;
                if coordinate_system == crate::core::SsCoordinateSystem::YDown {
                    y = -y;
                }
                globals.shape_vertices[offset + 2 * v] = dvx;
                globals.shape_vertices[offset + 2 * v + 1] = y;

                globals.shape_vertex_box_coords[offset + 2 * v] = fx;
                globals.shape_vertex_box_coords[offset + 2 * v + 1] = fy;
            }
            globals.shape_vertex_counts[i] = count as i32;
        }
    }

    // Playhead and per-frame buffer getters

    pub fn get_frame_no(&self) -> f32 {
        self.instance.primary_source().controller.get_frame_no()
    }

    /// `true` once the configured loops have completed, and sticky until the
    /// next `play` / `setup_animation`. Never `true` under an infinite loop
    /// count, and never raised by `stop`.
    ///
    /// This is a *state*, not a pulse: to notify once, take its rising edge.
    pub fn is_finished(&self) -> bool {
        self.instance.primary_source().controller.is_finished()
    }

    pub fn just_looped(&self) -> bool {
        self.instance.primary_source().controller.just_looped()
    }

    /// Pointer to the frame data built by the last [`get_frame_data`] call,
    /// without re-evaluating it. Null until the first call, and again after
    /// `reset` / `bind_resource`; valid until the next `get_frame_data`.
    ///
    /// [`get_frame_data`]: Self::get_frame_data
    pub fn get_frame_data_ptr(&self) -> *const u8 {
        if !self.eval.frame_data_ready {
            return ::core::ptr::null();
        }
        self.eval.fb.finished_data().as_ptr()
    }

    /// Byte length of the buffer [`get_frame_data_ptr`] points at.
    ///
    /// [`get_frame_data_ptr`]: Self::get_frame_data_ptr
    pub fn get_frame_data_len(&self) -> usize {
        if !self.eval.frame_data_ready {
            return 0;
        }
        self.eval.fb.finished_data().len()
    }

    pub fn get_world_matrices_ptr(&self) -> *const f32 {
        self.instance.globals.world_matrices.as_ptr() as *const f32
    }

    pub fn get_world_matrices_len(&self) -> usize {
        self.instance.globals.world_matrices.len() * 16
    }

    pub fn get_local_uvs_ptr(&self) -> *const f32 {
        self.instance.globals.local_uvs.as_ptr()
    }

    pub fn get_local_uvs_len(&self) -> usize {
        self.instance.globals.local_uvs.len()
    }

    pub fn get_shape_vertices_ptr(&self) -> *const f32 {
        self.instance.globals.shape_vertices.as_ptr()
    }

    pub fn get_shape_vertices_len(&self) -> usize {
        self.instance.globals.shape_vertices.len()
    }

    pub fn get_shape_vertex_box_coords_ptr(&self) -> *const f32 {
        self.instance.globals.shape_vertex_box_coords.as_ptr()
    }

    pub fn get_shape_vertex_box_coords_len(&self) -> usize {
        self.instance.globals.shape_vertex_box_coords.len()
    }

    pub fn get_shape_vertex_counts_ptr(&self) -> *const i32 {
        self.instance.globals.shape_vertex_counts.as_ptr()
    }

    pub fn get_mesh_vertices_x_ptr(&self) -> *const f32 {
        self.instance.globals.mesh_vertices_x.as_ptr()
    }

    pub fn get_mesh_vertices_x_len(&self) -> usize {
        self.instance.globals.mesh_vertices_x.len()
    }

    pub fn get_mesh_vertices_y_ptr(&self) -> *const f32 {
        self.instance.globals.mesh_vertices_y.as_ptr()
    }

    pub fn get_mesh_vertices_y_len(&self) -> usize {
        self.instance.globals.mesh_vertices_y.len()
    }

    pub fn get_mesh_vertex_offsets_ptr(&self) -> *const u32 {
        self.instance.globals.mesh_vertex_offsets.as_ptr()
    }

    pub fn get_mesh_vertex_offsets_len(&self) -> usize {
        self.instance.globals.mesh_vertex_offsets.len()
    }

    /// The static mesh buffers this instance draws with: none before the first
    /// Direct-mode `get_frame_data`, and none in SceneGraph mode.
    fn static_mesh(&self) -> Option<&StaticMeshBuffers> {
        self.eval.resource?.static_mesh(self.instance.globals.mesh_layout)
    }

    pub fn get_mesh_uvs_ptr(&self) -> *const f32 {
        self.static_mesh().map_or(::core::ptr::NonNull::dangling().as_ptr(), |m| m.uvs.as_ptr())
    }

    pub fn get_mesh_uvs_len(&self) -> usize {
        self.static_mesh().map_or(0, |m| m.uvs.len())
    }

    pub fn get_mesh_indices_ptr(&self) -> *const i32 {
        self.static_mesh().map_or(::core::ptr::NonNull::dangling().as_ptr(), |m| m.indices.as_ptr())
    }

    pub fn get_mesh_indices_len(&self) -> usize {
        self.static_mesh().map_or(0, |m| m.indices.len())
    }

    pub fn get_mesh_index_offsets_ptr(&self) -> *const u32 {
        self.static_mesh().map_or(::core::ptr::NonNull::dangling().as_ptr(), |m| m.index_offsets.as_ptr())
    }

    pub fn get_mesh_index_offsets_len(&self) -> usize {
        self.static_mesh().map_or(0, |m| m.index_offsets.len())
    }

    pub fn get_nines_vertices_ptr(&self) -> *const f32 {
        self.instance.globals.nines.vertices.as_ptr()
    }

    pub fn get_nines_vertices_len(&self) -> usize {
        self.instance.globals.nines.vertices.len()
    }

    pub fn get_nines_uvs_ptr(&self) -> *const f32 {
        self.instance.globals.nines.uvs.as_ptr()
    }

    pub fn get_nines_uvs_len(&self) -> usize {
        self.instance.globals.nines.uvs.len()
    }

    pub fn get_nines_box_coords_ptr(&self) -> *const f32 {
        self.instance.globals.nines.box_coords.as_ptr()
    }

    pub fn get_nines_box_coords_len(&self) -> usize {
        self.instance.globals.nines.box_coords.len()
    }

    pub fn get_nines_indices_ptr(&self) -> *const i32 {
        self.instance.globals.nines.indices.as_ptr()
    }

    pub fn get_nines_indices_len(&self) -> usize {
        self.instance.globals.nines.indices.len()
    }

    pub fn get_nines_vertex_offsets_ptr(&self) -> *const u32 {
        self.instance.globals.nines.vertex_offsets.as_ptr()
    }

    pub fn get_nines_vertex_offsets_len(&self) -> usize {
        self.instance.globals.nines.vertex_offsets.len()
    }

    pub fn get_nines_index_offsets_ptr(&self) -> *const u32 {
        self.instance.globals.nines.index_offsets.as_ptr()
    }

    pub fn get_nines_index_offsets_len(&self) -> usize {
        self.instance.globals.nines.index_offsets.len()
    }

    pub fn get_shape_vertex_counts_len(&self) -> usize {
        self.instance.globals.shape_vertex_counts.len()
    }

    pub fn get_cell_meta_ptr(&self) -> *const f32 {
        self.instance.resolved_layer.cell_meta_per_part.as_ptr()
    }

    pub fn get_cell_meta_len(&self) -> usize {
        self.instance.resolved_layer.cell_meta_per_part.len()
    }

    pub fn get_local_vertices_ptr(&self) -> *const f32 {
        self.instance.resolved_layer.local_vertices.as_ptr()
    }

    pub fn get_local_vertices_len(&self) -> usize {
        self.instance.resolved_layer.local_vertices.len()
    }

    // Playback state and setting getters

    /// `true` while the clock runs and track 0 has not finished. A held
    /// character is still playing.
    pub fn is_playing(&self) -> bool {
        self.instance.clock.playing && self.instance.primary_source().controller.is_playing()
    }

    pub fn is_playing_forward(&self) -> bool {
        self.instance.primary_source().controller.is_playing_forward()
    }

    /// `true` while a playing character is held. Implies [`Self::is_playing`],
    /// so the pair reads as three states. Once track 0 has finished, a pause
    /// still holds the other tracks and the fades, but this reads `false`
    /// along with `is_playing`.
    pub fn is_pausing(&self) -> bool {
        self.instance.clock.pausing && self.is_playing()
    }

    /// `true` from `play` until `stop` or the next setup: the character's
    /// clock, which [`Self::is_playing`] reads only while track 0 has not
    /// finished. A host that stops ticking a character with nothing to move
    /// reads this, since the other tracks and the fades run on after track 0
    /// ends. A held clock is still playing.
    pub fn is_clock_playing(&self) -> bool {
        self.instance.clock.playing
    }

    /// `true` while [`Self::pause`] holds a playing clock. Implies
    /// [`Self::is_clock_playing`], so the pair reads as three states, as
    /// [`Self::is_playing`] and [`Self::is_pausing`] do; unlike `is_pausing`,
    /// it still reads the hold after track 0 has finished.
    pub fn is_clock_pausing(&self) -> bool {
        self.instance.clock.pausing
    }

    /// `true` while the track stack would move the pose on a running clock for a
    /// reason other than track 0's own animation stepping: a fade on any track,
    /// or a source — other than track 0's main one — that plays, at a speed above
    /// 0 and a weight above 0, on a track that is heard. A cleared, finished,
    /// silenced or scrubbed track does not count, nor does a source a crossfade
    /// has retired; one a cancelled crossfade left behind does.
    ///
    /// It does not read the clock: a Player gating its redraw combines it with
    /// [`Self::is_clock_playing`] and [`Self::is_clock_pausing`], and compares
    /// track 0's frame itself. With this, a Player keeps no copy of which tracks
    /// and sources it has laid, which goes wrong the moment one is released
    /// inside the runtime.
    pub fn is_stack_in_motion(&self) -> bool {
        let instance = &self.instance;
        if instance.tracks.iter().any(|t| t.fade.is_some()) {
            return true;
        }
        let primary = instance.primary_idx();
        instance
            .sources
            .iter()
            .enumerate()
            .any(|(i, source)| i != primary && source.has_real_animation() && source.weight > 0.0 && source.controller.is_playing() && instance.tracks.get(source.track as usize).is_some_and(|t| t.is_audible() && t.policy.speed_scale > 0.0))
    }

    // The track 0 settings below read the track, which their setters write:
    // set before an animation is laid, there is no controller holding them yet.

    /// Track 0's [`Self::get_track_skip_frames`].
    pub fn get_skip_frames(&self) -> bool {
        self.instance.tracks[0].policy.is_skip_frames
    }

    /// Track 0's [`Self::get_track_playback_direction`].
    pub fn get_playback_direction(&self) -> core::playback_controller::PlaybackDirection {
        self.instance.tracks[0].policy.playback_direction
    }

    /// Track 0's [`Self::get_track_playback_style`].
    pub fn get_playback_style(&self) -> core::playback_controller::PlaybackStyle {
        self.instance.tracks[0].policy.playback_style
    }

    pub fn get_loop_count(&self) -> i32 {
        self.instance.primary_source().controller.get_loop_count()
    }

    pub fn get_remaining_loops(&self) -> i32 {
        self.instance.primary_source().controller.get_remaining_loops()
    }

    pub fn get_start_frame(&self) -> i32 {
        self.instance.primary_source().controller.get_start_frame()
    }

    pub fn get_end_frame(&self) -> i32 {
        self.instance.primary_source().controller.get_end_frame()
    }

    pub fn get_anime_data_start_frame(&self) -> i32 {
        self.instance.primary_source().controller.get_anime_data_start_frame()
    }

    pub fn get_anime_data_end_frame(&self) -> i32 {
        self.instance.primary_source().controller.get_anime_data_end_frame()
    }

    /// Track 0's [`Self::get_track_frame_rate`].
    pub fn get_frame_rate(&self) -> i32 {
        let policy = &self.instance.tracks[0].policy;
        self.instance.track_main_controller(0).map_or(policy.frame_rate_override.unwrap_or(0), |c| c.get_frame_rate())
    }

    /// Track 0's [`Self::get_track_speed`].
    pub fn get_speed_scale(&self) -> f32 {
        self.instance.tracks[0].policy.speed_scale
    }

    // Playback setup

    /// Track 0's [`Self::set_track_skip_frames`].
    pub fn set_skip_frames(&mut self, skip: bool) {
        self.instance.set_track_policy(0, |p| p.is_skip_frames = skip, |c| c.set_skip_frames(skip));
    }

    pub fn set_loop_count(&mut self, loop_count: i32) {
        self.instance.primary_source_mut().controller.set_loop_count(loop_count);
    }

    /// Track 0's [`Self::set_track_frame_rate`].
    pub fn set_frame_rate(&mut self, frame_rate: i32) {
        self.instance.set_track_policy(0, |p| p.set_frame_rate(frame_rate), |c| c.set_frame_rate(frame_rate));
    }

    /// Track 0's [`Self::set_track_playback_direction`].
    pub fn set_playback_direction(&mut self, direction: core::playback_controller::PlaybackDirection, style: core::playback_controller::PlaybackStyle) {
        self.instance.set_track_policy(
            0,
            |p| {
                p.playback_direction = direction;
                p.playback_style = style;
            },
            |c| c.set_playback_direction(direction, style),
        );
    }

    /// Track 0's [`Self::set_track_speed`].
    pub fn set_speed_scale(&mut self, speed_scale: f32) {
        self.instance.set_track_policy(0, |p| p.set_speed_scale(speed_scale), |c| c.set_speed_scale(speed_scale));
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.instance.primary_source_mut().controller.set_animation_section(start_frame, end_frame);
    }

    pub fn set_coordinate_system(&mut self, coordinate_system: crate::core::SsCoordinateSystem) {
        self.instance.coordinate_system = coordinate_system;
        // The part bases live in this space, so they have to move with it.
        self.prepare_part_bases();
    }

    pub fn get_coordinate_system(&self) -> crate::core::SsCoordinateSystem {
        self.instance.coordinate_system
    }

    pub fn set_origin(&mut self, origin: crate::core::SsOrigin) {
        self.instance.origin = origin;
        // The part bases live in this space, so they have to move with it.
        self.prepare_part_bases();
    }

    pub fn get_origin(&self) -> crate::core::SsOrigin {
        self.instance.origin
    }

    pub fn set_unit(&mut self, unit: crate::core::SsUnit) {
        self.instance.unit = unit;
        // The part bases live in this space, so they have to move with it.
        self.prepare_part_bases();
    }

    pub fn get_unit(&self) -> crate::core::SsUnit {
        self.instance.unit
    }

    // Playback control

    /// Advance every source's controller by `delta` milliseconds and return the
    /// frame to hand to [`Self::get_frame_data`]: that of track 0's main
    /// animation, which `get_frame_data` validates against. A crossfade on
    /// track 0 hands that role over when it begins, so across one this is the
    /// incoming animation's frame.
    ///
    /// Nothing moves while the character's clock is stopped or held, fades
    /// included. The controllers are still updated with no elapsed time, which
    /// is what reports a frame a seek has just landed on.
    pub fn update(&mut self, delta: f32) -> f32 {
        self.instance.updates += 1;
        let running = self.instance.clock.is_running();
        let elapsed = if running { delta } else { 0.0 };
        for source in self.instance.sources.iter_mut() {
            if !source.has_real_animation() {
                continue;
            }
            source.controller.update(elapsed);
        }

        if running {
            self.advance_track_fades(delta);
        }
        self.collect_events();

        self.get_frame_no()
    }

    /// Advance every track's fade by `delta`.
    ///
    /// A crossfade raises the incoming source's weight linearly while the
    /// outgoing side sinks by the same amount, and on completion releases every
    /// source on the track but the incoming one, which has been its main one
    /// since the fade began. An alpha fade drives the track's `alpha` to its
    /// target. A crossfade whose incoming source was released elsewhere cancels
    /// itself.
    fn advance_track_fades(&mut self, delta: f32) {
        for track in 0..self.instance.tracks.len() {
            let Some(fade) = self.instance.tracks[track].fade else {
                continue;
            };
            match fade {
                TrackFade::Cross { to, mut elapsed, duration } => {
                    if self.instance.sources.get(to as usize).is_none_or(|s| !s.has_real_animation()) {
                        self.instance.tracks[track].fade = None;
                        continue;
                    }
                    elapsed += delta;
                    let t = (elapsed / duration).clamp(0.0, 1.0);
                    for (i, source) in self.instance.sources.iter_mut().enumerate() {
                        // A source added to the track after the fade began has no
                        // share, keeps the weight it was given, and is released with
                        // the rest on completion.
                        if i as u32 != to && source.track as usize == track && source.has_real_animation() && source.outgoing_share > 0.0 {
                            source.weight = source.outgoing_share * (1.0 - t);
                        }
                    }
                    self.instance.sources[to as usize].weight = t;
                    if elapsed >= duration {
                        self.replace_track_main(track as u32, to);
                    } else {
                        self.instance.tracks[track].fade = Some(TrackFade::Cross { to, elapsed, duration });
                    }
                }
                TrackFade::Alpha { start, target, mut elapsed, duration } => {
                    elapsed += delta;
                    let t = (elapsed / duration).clamp(0.0, 1.0);
                    self.instance.tracks[track].alpha = start + (target - start) * t;
                    if elapsed >= duration {
                        self.instance.tracks[track].alpha = target;
                        self.instance.tracks[track].fade = None;
                    } else {
                        self.instance.tracks[track].fade = Some(TrackFade::Alpha { start, target, elapsed, duration });
                    }
                }
            }
        }
    }

    fn collect_events(&mut self) {
        self.instance.collected_event_indices.clear();
        self.instance.collected_event_source_handles.clear();
        let Some(resource) = self.eval.resource else {
            return;
        };
        let primary_idx = self.instance.primary_idx();
        let num_sources = self.instance.sources.len();

        for i in 0..num_sources {
            let source = &self.instance.sources[i];
            if !source.has_real_animation() {
                continue;
            }
            let is_primary = i == primary_idx;
            if !is_primary && !self.instance.source_fires_events(i) {
                continue;
            }
            let metadata = &resource.animations[source.animation_idx as usize];
            let Some(events_vec) = metadata.events else { continue };
            let source_handle = i as u32;

            for &frame_no in source.controller.get_passed_frames() {
                if frame_no < 0 || frame_no > u16::MAX as i32 {
                    continue;
                }
                let target_frame = frame_no as u16;
                let mut low = 0;
                let mut high = events_vec.len();
                while low < high {
                    let mid = low + (high - low) / 2;
                    match events_vec.get(mid).frame_index().cmp(&target_frame) {
                        ::core::cmp::Ordering::Equal => {
                            self.instance.collected_event_indices.push(mid as i32);
                            self.instance.collected_event_source_handles.push(source_handle);
                            break;
                        }
                        ::core::cmp::Ordering::Less => low = mid + 1,
                        ::core::cmp::Ordering::Greater => high = mid,
                    }
                }
            }
        }
    }

    pub fn get_passed_event_indices(&self) -> &[i32] {
        &self.instance.collected_event_indices
    }

    /// Parallel to [`Self::get_passed_event_indices`]: source handle that fired
    /// each collected event. With every source and every track's outgoing side
    /// at the default event threshold (`+∞`), entries are uniformly
    /// [`Self::get_track_main_source`] of track 0. Diverges once secondaries opt
    /// in to event firing via [`Self::set_source_event_threshold`], or a
    /// crossfade's outgoing side via [`Self::set_track_outgoing_event_threshold`].
    pub fn get_passed_event_source_handles(&self) -> &[u32] {
        &self.instance.collected_event_source_handles
    }

    /// Resolve the most-recent `EventInstance` whose `frame_index <= current frame`
    /// for `slot_index` on the *currently selected animation*. Returns
    /// `synthetic_default()` when no event has fired yet (SS6 default semantics).
    /// Designed to live behind the `ss_runtime_get_active_event_instance` FFI.
    pub fn get_active_event_instance(&self, slot_index: u32) -> crate::types::ss_event_instance_info {
        let mut info = crate::types::ss_event_instance_info::synthetic_default();
        let Some(resource) = self.eval.resource else { return info };
        if self.instance.primary_source().animation_idx < 0 {
            return info;
        }
        let metadata = &resource.animations[self.instance.primary_source().animation_idx as usize];
        let Some(events) = metadata.events else { return info };

        let frame_no = self.instance.primary_source().controller.get_frame_no();
        let frame_int = if frame_no < 0.0 {
            -1
        } else if frame_no >= u16::MAX as f32 {
            u16::MAX as i32
        } else {
            frame_no as i32
        };

        // Forward scan: events are sorted by frame_index. Later matches override
        // earlier ones, so the loop naturally lands on the most-recent
        // EventInstance for `slot_index`.
        for i in 0..events.len() {
            let epf = events.get(i);
            if (epf.frame_index() as i32) > frame_int {
                break;
            }
            let Some(instances) = epf.instances() else { continue };
            for j in 0..instances.len() {
                let ev = instances.get(j);
                if ev.part_index() as u32 != slot_index {
                    continue;
                }
                let attr = ev.value();
                info = crate::types::ss_event_instance_info {
                    event_frame: epf.frame_index() as i32,
                    start_label_hash: attr.start_label_hash(),
                    start_offset: attr.start_offset(),
                    end_label_hash: attr.end_label_hash(),
                    end_offset: attr.end_offset(),
                    loop_num: attr.loop_num(),
                    speed: attr.speed(),
                    independent: attr.independent(),
                    reverse: attr.reverse(),
                    pingpong: attr.pingpong(),
                    is_synthetic_default: false,
                };
            }
        }
        info
    }

    /// Resolve the most-recent `EventEffect` whose `frame_index <= current frame`
    /// for `slot_index` on the *currently selected animation*. When no event has
    /// fired yet, falls back to `InitialEvents.effect` for that slot (still flagged
    /// `is_synthetic_default = true`); if neither exists returns full SS6 defaults.
    /// Designed to live behind the `ss_runtime_get_active_effect_event` FFI.
    pub fn get_active_effect_event(&self, slot_index: u32) -> crate::types::ss_effect_event_info {
        let mut info = crate::types::ss_effect_event_info::synthetic_default();
        let Some(resource) = self.eval.resource else { return info };
        if self.instance.primary_source().animation_idx < 0 {
            return info;
        }
        let metadata = &resource.animations[self.instance.primary_source().animation_idx as usize];

        // Fallback first: InitialEvents.effect for this slot. Real EventEffect
        // matches override below; the synthetic-default flag stays `true` for
        // initial-only state so callers don't treat it as an event edge.
        if let Some(inits) = metadata.initial_events
            && (slot_index as usize) < inits.len()
        {
            let entry = inits.get(slot_index as usize);
            if let Some(eff) = entry.effect() {
                info.event_frame = 0;
                info.start_time = eff.start_time();
                let s = eff.speed();
                info.speed = if s == 0.0 { 1.0 } else { s };
                // The implicit frame-0 (Setup) key is never an independent-playback
                // start point: only a real authored EventEffect may begin independent
                // playback. Forced to false so this mirrors the Instance synthetic
                // default (`ss_event_instance_info::synthetic_default`, independent =
                // false) and the two part types stay symmetric.
                info.independent = false;
                info.is_synthetic_default = true;
            }
        }

        let Some(events) = metadata.events else { return info };

        let frame_no = self.instance.primary_source().controller.get_frame_no();
        let frame_int = if frame_no < 0.0 {
            -1
        } else if frame_no >= u16::MAX as f32 {
            u16::MAX as i32
        } else {
            frame_no as i32
        };

        for i in 0..events.len() {
            let epf = events.get(i);
            if (epf.frame_index() as i32) > frame_int {
                break;
            }
            let Some(effects) = epf.effects() else { continue };
            for j in 0..effects.len() {
                let ev = effects.get(j);
                if ev.part_index() as u32 != slot_index {
                    continue;
                }
                let attr = ev.value();
                let s = attr.speed();
                info = crate::types::ss_effect_event_info { event_frame: epf.frame_index() as i32, start_time: attr.start_time(), speed: if s == 0.0 { 1.0 } else { s }, independent: attr.independent(), is_synthetic_default: false };
            }
        }
        info
    }

    /// Look up `label_hash` in the currently selected animation's `labels`
    /// table and return its `time`. Returns `fallback` when `label_hash == 0`
    /// (caller convention for "unset"), no animation is bound, or the hash
    /// is not found. Linear scan — labels per animation are typically <50.
    pub fn resolve_label_time(&self, label_hash: u32, fallback: i32) -> i32 {
        if label_hash == 0 {
            return fallback;
        }
        let Some(resource) = self.eval.resource else { return fallback };
        if self.instance.primary_source().animation_idx < 0 {
            return fallback;
        }
        let metadata = &resource.animations[self.instance.primary_source().animation_idx as usize];
        let Some(labels) = metadata.labels else { return fallback };
        for i in 0..labels.len() {
            let label = labels.get(i);
            if label.name_hash() == label_hash {
                return label.time();
            }
        }
        fallback
    }

    /// Heads track 0 out — back to its section start, loop count re-armed — and
    /// starts the character's clock. The other tracks keep their playheads, as
    /// they do when track 0 loops or ends on its own.
    pub fn play(&mut self) {
        self.instance.primary_source_mut().controller.play();
        self.start_clock();
    }

    pub fn play_with_start_frame(&mut self, frame_no: f32) {
        self.instance.primary_source_mut().controller.play_with_start_frame(frame_no);
        self.start_clock();
    }

    fn start_clock(&mut self) {
        self.instance.clock = CharacterClock { playing: true, pausing: false };
    }

    /// Holds the whole character in place: every track and every fade.
    /// Idempotent, and ignored while the clock is stopped; `is_playing` stays
    /// `true`.
    pub fn pause(&mut self) {
        if self.instance.clock.playing {
            self.instance.clock.pausing = true;
        }
    }

    /// Lifts a [`Self::pause`]. Idempotent, and not a way to start a stopped
    /// animation — `play` rewinds, so it cannot serve as a resume.
    pub fn resume(&mut self) {
        self.instance.clock.pausing = false;
    }

    /// Stops the whole character where it stands: every track and every fade.
    /// Nothing rewinds; only `play` starts it again.
    pub fn stop(&mut self) {
        self.instance.clock = CharacterClock::default();
    }

    pub fn set_frame_no(&mut self, frame: f32) {
        self.instance.primary_source_mut().controller.set_frame_no(frame);
    }

    /// Sets the playback position based on `diff` (frames elapsed since
    /// playback started, with per-instance speed already applied), applying
    /// the configured section / loops / style / direction. Used by parent
    /// Players to drive child Instance Players deterministically
    /// across reverse, seek, and frame-skip operations.
    pub fn set_frame_relative(&mut self, diff: f32) {
        self.instance.primary_source_mut().controller.set_frame_relative(diff);
    }

    pub fn next_frame(&mut self) {
        self.instance.primary_source_mut().controller.next_frame();
    }

    pub fn prev_frame(&mut self) {
        self.instance.primary_source_mut().controller.prev_frame();
    }
}
