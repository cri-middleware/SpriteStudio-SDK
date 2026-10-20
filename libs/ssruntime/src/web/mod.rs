//! Wasm bindings for ssruntime.
//!
//! This module mirrors the C FFI surface in `src/ffi/mod.rs` 1:1 for JS/TS
//! Players. The FFI side is authoritative — when adding or modifying public
//! API, update `src/ffi/mod.rs` first and reflect the change here. A fallible
//! one reports through `js_error` — never by formatting the error itself, which
//! is how twenty-eight call sites came to hand JS the Rust variant name. POD return
//! types (`ss_event_instance_info`, `ss_effect_event_info`) are exposed as
//! wasm-bindgen wrapper classes (`WebEventInstanceInfo`,
//! `WebEffectEventInfo`) because wasm-bindgen cannot expose `#[repr(C)]`
//! structs by value.
//!
//! Not yet exposed (deferred):
//!
//! - `ss_runtime_set_log_callback`, which requires bridging a JS `Function`
//!   and a `js-sys` dependency, held off until the JS/TS Player needs it.
//! - `ss_runtime_ssab_schema_version`.

use crate::owning::{OwningContext, OwningEffectSlot};
use crate::{SsabData, SsabResource};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebCoordinateSystem {
    YUp = 0,
    YDown = 1,
}

impl From<crate::eval::SsCoordinateSystem> for WebCoordinateSystem {
    fn from(cs: crate::eval::SsCoordinateSystem) -> Self {
        match cs {
            crate::eval::SsCoordinateSystem::YUp => WebCoordinateSystem::YUp,
            crate::eval::SsCoordinateSystem::YDown => WebCoordinateSystem::YDown,
        }
    }
}

impl From<WebCoordinateSystem> for crate::eval::SsCoordinateSystem {
    fn from(cs: WebCoordinateSystem) -> Self {
        match cs {
            WebCoordinateSystem::YUp => crate::eval::SsCoordinateSystem::YUp,
            WebCoordinateSystem::YDown => crate::eval::SsCoordinateSystem::YDown,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebOrigin {
    Center = 0,
    TopLeft = 1,
}

impl From<crate::eval::SsOrigin> for WebOrigin {
    fn from(origin: crate::eval::SsOrigin) -> Self {
        match origin {
            crate::eval::SsOrigin::Center => WebOrigin::Center,
            crate::eval::SsOrigin::TopLeft => WebOrigin::TopLeft,
        }
    }
}

impl From<WebOrigin> for crate::eval::SsOrigin {
    fn from(origin: WebOrigin) -> Self {
        match origin {
            WebOrigin::Center => crate::eval::SsOrigin::Center,
            WebOrigin::TopLeft => crate::eval::SsOrigin::TopLeft,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebUnit {
    Pixel = 0,
    Normalized = 1,
}

impl From<crate::eval::SsUnit> for WebUnit {
    fn from(unit: crate::eval::SsUnit) -> Self {
        match unit {
            crate::eval::SsUnit::Pixel => WebUnit::Pixel,
            crate::eval::SsUnit::Normalized => WebUnit::Normalized,
        }
    }
}

impl From<WebUnit> for crate::eval::SsUnit {
    fn from(unit: WebUnit) -> Self {
        match unit {
            WebUnit::Pixel => crate::eval::SsUnit::Pixel,
            WebUnit::Normalized => crate::eval::SsUnit::Normalized,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebPlaybackDirection {
    Forward = 0,
    Backward = 1,
}

impl From<crate::eval::playback_controller::PlaybackDirection> for WebPlaybackDirection {
    fn from(dir: crate::eval::playback_controller::PlaybackDirection) -> Self {
        match dir {
            crate::eval::playback_controller::PlaybackDirection::Forward => WebPlaybackDirection::Forward,
            crate::eval::playback_controller::PlaybackDirection::Backward => WebPlaybackDirection::Backward,
        }
    }
}

impl From<WebPlaybackDirection> for crate::eval::playback_controller::PlaybackDirection {
    fn from(dir: WebPlaybackDirection) -> Self {
        match dir {
            WebPlaybackDirection::Forward => crate::eval::playback_controller::PlaybackDirection::Forward,
            WebPlaybackDirection::Backward => crate::eval::playback_controller::PlaybackDirection::Backward,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebPlaybackStyle {
    Normal = 0,
    PingPong = 1,
}

impl From<crate::eval::playback_controller::PlaybackStyle> for WebPlaybackStyle {
    fn from(style: crate::eval::playback_controller::PlaybackStyle) -> Self {
        match style {
            crate::eval::playback_controller::PlaybackStyle::Normal => WebPlaybackStyle::Normal,
            crate::eval::playback_controller::PlaybackStyle::PingPong => WebPlaybackStyle::PingPong,
        }
    }
}

impl From<WebPlaybackStyle> for crate::eval::playback_controller::PlaybackStyle {
    fn from(style: WebPlaybackStyle) -> Self {
        match style {
            WebPlaybackStyle::Normal => crate::eval::playback_controller::PlaybackStyle::Normal,
            WebPlaybackStyle::PingPong => crate::eval::playback_controller::PlaybackStyle::PingPong,
        }
    }
}

/// JS-friendly mirror of `ss_event_instance_info` (`src/types.rs`).
///
/// Returned by `SSRuntimeContext::get_active_event_instance` and consumed by
/// `SSRuntimeInstanceSlot::step`. Construct directly via the
/// `WebEventInstanceInfo` constructor when bridging from raw values, or call
/// `synthetic_default` to obtain SS6 defaults.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct WebEventInstanceInfo {
    pub event_frame: i32,
    pub start_label_hash: u32,
    pub start_offset: i32,
    pub end_label_hash: u32,
    pub end_offset: i32,
    pub loop_num: i32,
    pub speed: f32,
    pub independent: bool,
    pub reverse: bool,
    pub pingpong: bool,
    pub is_synthetic_default: bool,
}

#[wasm_bindgen]
impl WebEventInstanceInfo {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(event_frame: i32, start_label_hash: u32, start_offset: i32, end_label_hash: u32, end_offset: i32, loop_num: i32, speed: f32, independent: bool, reverse: bool, pingpong: bool, is_synthetic_default: bool) -> WebEventInstanceInfo {
        WebEventInstanceInfo { event_frame, start_label_hash, start_offset, end_label_hash, end_offset, loop_num, speed, independent, reverse, pingpong, is_synthetic_default }
    }

    pub fn synthetic_default() -> WebEventInstanceInfo {
        crate::types::ss_event_instance_info::synthetic_default().into()
    }
}

impl From<crate::types::ss_event_instance_info> for WebEventInstanceInfo {
    fn from(v: crate::types::ss_event_instance_info) -> Self {
        WebEventInstanceInfo {
            event_frame: v.event_frame,
            start_label_hash: v.start_label_hash,
            start_offset: v.start_offset,
            end_label_hash: v.end_label_hash,
            end_offset: v.end_offset,
            loop_num: v.loop_num,
            speed: v.speed,
            independent: v.independent,
            reverse: v.reverse,
            pingpong: v.pingpong,
            is_synthetic_default: v.is_synthetic_default,
        }
    }
}

impl From<WebEventInstanceInfo> for crate::types::ss_event_instance_info {
    fn from(v: WebEventInstanceInfo) -> Self {
        crate::types::ss_event_instance_info {
            event_frame: v.event_frame,
            start_label_hash: v.start_label_hash,
            start_offset: v.start_offset,
            end_label_hash: v.end_label_hash,
            end_offset: v.end_offset,
            loop_num: v.loop_num,
            speed: v.speed,
            independent: v.independent,
            reverse: v.reverse,
            pingpong: v.pingpong,
            is_synthetic_default: v.is_synthetic_default,
        }
    }
}

/// JS-friendly mirror of `ss_effect_event_info` (`src/types.rs`).
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct WebEffectEventInfo {
    pub event_frame: i32,
    pub start_time: i32,
    pub speed: f32,
    pub independent: bool,
    pub is_synthetic_default: bool,
}

#[wasm_bindgen]
impl WebEffectEventInfo {
    #[wasm_bindgen(constructor)]
    pub fn new(event_frame: i32, start_time: i32, speed: f32, independent: bool, is_synthetic_default: bool) -> WebEffectEventInfo {
        WebEffectEventInfo { event_frame, start_time, speed, independent, is_synthetic_default }
    }

    pub fn synthetic_default() -> WebEffectEventInfo {
        crate::types::ss_effect_event_info::synthetic_default().into()
    }
}

impl From<crate::types::ss_effect_event_info> for WebEffectEventInfo {
    fn from(v: crate::types::ss_effect_event_info) -> Self {
        WebEffectEventInfo { event_frame: v.event_frame, start_time: v.start_time, speed: v.speed, independent: v.independent, is_synthetic_default: v.is_synthetic_default }
    }
}

impl From<WebEffectEventInfo> for crate::types::ss_effect_event_info {
    fn from(v: WebEffectEventInfo) -> Self {
        crate::types::ss_effect_event_info { event_frame: v.event_frame, start_time: v.start_time, speed: v.speed, independent: v.independent, is_synthetic_default: v.is_synthetic_default }
    }
}

/// How every fallible call here reports: the Brain's own wording, as the string
/// of a JS exception.
///
/// `Display`, not `Debug`. `Debug` prints the Rust variant name — a JS caller
/// was being handed `AnimationNotFound`, which is an identifier from a language
/// they are not writing, and it reaches their user through whatever the Player
/// puts in front of them. `Display` also carries what a variant knows:
/// `InvalidVersion` names the version expected and the one that arrived, which
/// is the whole of what a stale pack needs someone to be told.
///
/// One function rather than a closure per call site, because there were
/// twenty-eight of them and they have to agree.
fn js_error(e: crate::error::SSRuntimeError) -> JsValue {
    JsValue::from_str(&alloc::format!("{}", e))
}

#[wasm_bindgen]
pub struct SSRuntimeResource {
    inner: Arc<SsabResource>,
}

#[wasm_bindgen]
impl SSRuntimeResource {
    /// Copies and parses a `.ssab`. The runtime does not verify the bytes — a malformed or
    /// crafted pack causes out-of-bounds reads within this module's memory — so pass only
    /// packs you trust. A schema-version mismatch is reported as an error.
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<SSRuntimeResource, JsValue> {
        let ssab_data = SsabData::load_copy(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
        // SAFETY: the JS caller guarantees a valid `.ssab`, as the C ABI's caller does; nothing
        // here can check it (crate docs, "Limitations to design around").
        let inner = unsafe { SsabResource::new(ssab_data) };
        if inner.resource().is_none() {
            // The reason, not just the fact: a version mismatch names both
            // versions, which is the whole of what the user needs to act.
            let reason = inner.parse_error().map_or_else(|| alloc::string::String::from("Failed to parse SSAB data"), |e| alloc::format!("{}", e));
            return Err(JsValue::from_str(&reason));
        }
        Ok(SSRuntimeResource { inner })
    }
}

#[wasm_bindgen]
pub struct SSRuntimeContext {
    inner: OwningContext,
}

#[wasm_bindgen]
impl SSRuntimeContext {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Option<Box<[u8]>>) -> Result<SSRuntimeContext, JsValue> {
        let mut ctx = SSRuntimeContext { inner: OwningContext::new() };
        if let Some(d) = data {
            let res = SSRuntimeResource::new(&d)?;
            ctx.bind_resource(&res);
        }
        Ok(ctx)
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }

    pub fn bind_resource(&mut self, resource: &SSRuntimeResource) -> bool {
        self.inner.bind(resource.inner.clone()).is_ok()
    }

    pub fn load_ssab_copy(&mut self, data: &[u8]) -> bool {
        if let Ok(ssab_data) = SsabData::load_copy(data) {
            // SAFETY: the JS caller guarantees a valid `.ssab`, as for `SSRuntimeResource::new`.
            let res = SSRuntimeResource { inner: unsafe { SsabResource::new(ssab_data) } };
            if res.inner.resource().is_some() {
                return self.bind_resource(&res);
            }
        }
        false
    }

    pub fn get_render_mode(&self) -> u8 {
        self.inner.render_mode() as u8
    }

    pub fn set_render_mode(&mut self, mode: u8) {
        self.inner.set_render_mode(crate::eval::RenderMode::from_u8(mode));
    }

    pub fn setup_animation(&mut self, name: &str) -> Result<(), JsValue> {
        self.inner.setup_animation(name).map_err(js_error)
    }

    pub fn setup_animation_by_hash(&mut self, name_hash: u32) -> Result<(), JsValue> {
        self.inner.setup_animation_by_hash(name_hash).map_err(js_error)
    }

    /// Build the frame data for `frame_no` and return its byte length. Read the
    /// bytes with `get_frame_data_buf`, which does not evaluate again.
    ///
    /// Evaluating the frame is the expensive half of a tick, so it happens here
    /// once. The buf/len pair is pure, like every other one on this type.
    pub fn prepare_frame_data(&mut self, frame_no: f32) -> Result<usize, JsValue> {
        self.inner.get_frame_data(frame_no).map(|s| s.len()).map_err(js_error)
    }

    // Zero-copy getters

    /// Pointer to what the last `prepare_frame_data` built. Null before the
    /// first call; valid until the next one.
    pub fn get_frame_data_buf(&self) -> *const u8 {
        self.inner.get_frame_data_ptr()
    }

    pub fn get_frame_data_len(&self) -> usize {
        self.inner.get_frame_data_len()
    }

    pub fn get_world_matrices_buf(&self) -> *const f32 {
        self.inner.get_world_matrices_ptr()
    }

    pub fn get_world_matrices_len(&self) -> usize {
        self.inner.get_world_matrices_len()
    }

    pub fn get_mesh_vertices_x_buf(&self) -> *const f32 {
        self.inner.get_mesh_vertices_x_ptr()
    }

    pub fn get_mesh_vertices_x_len(&self) -> usize {
        self.inner.get_mesh_vertices_x_len()
    }

    pub fn get_mesh_vertices_y_buf(&self) -> *const f32 {
        self.inner.get_mesh_vertices_y_ptr()
    }

    pub fn get_mesh_vertices_y_len(&self) -> usize {
        self.inner.get_mesh_vertices_y_len()
    }

    pub fn get_mesh_uvs_buf(&self) -> *const f32 {
        self.inner.get_mesh_uvs_ptr()
    }

    pub fn get_mesh_uvs_len(&self) -> usize {
        self.inner.get_mesh_uvs_len()
    }

    pub fn get_mesh_indices_buf(&self) -> *const i32 {
        self.inner.get_mesh_indices_ptr()
    }

    pub fn get_mesh_indices_len(&self) -> usize {
        self.inner.get_mesh_indices_len()
    }

    pub fn get_mesh_vertex_offsets_buf(&self) -> *const u32 {
        self.inner.get_mesh_vertex_offsets_ptr()
    }

    pub fn get_mesh_vertex_offsets_len(&self) -> usize {
        self.inner.get_mesh_vertex_offsets_len()
    }

    pub fn get_mesh_index_offsets_buf(&self) -> *const u32 {
        self.inner.get_mesh_index_offsets_ptr()
    }

    pub fn get_mesh_index_offsets_len(&self) -> usize {
        self.inner.get_mesh_index_offsets_len()
    }

    pub fn get_nines_vertices_buf(&self) -> *const f32 {
        self.inner.get_nines_vertices_ptr()
    }

    pub fn get_nines_vertices_len(&self) -> usize {
        self.inner.get_nines_vertices_len()
    }

    pub fn get_nines_uvs_buf(&self) -> *const f32 {
        self.inner.get_nines_uvs_ptr()
    }

    pub fn get_nines_uvs_len(&self) -> usize {
        self.inner.get_nines_uvs_len()
    }

    pub fn get_nines_box_coords_buf(&self) -> *const f32 {
        self.inner.get_nines_box_coords_ptr()
    }

    pub fn get_nines_box_coords_len(&self) -> usize {
        self.inner.get_nines_box_coords_len()
    }

    pub fn get_nines_indices_buf(&self) -> *const i32 {
        self.inner.get_nines_indices_ptr()
    }

    pub fn get_nines_indices_len(&self) -> usize {
        self.inner.get_nines_indices_len()
    }

    pub fn get_nines_vertex_offsets_buf(&self) -> *const u32 {
        self.inner.get_nines_vertex_offsets_ptr()
    }

    pub fn get_nines_vertex_offsets_len(&self) -> usize {
        self.inner.get_nines_vertex_offsets_len()
    }

    pub fn get_nines_index_offsets_buf(&self) -> *const u32 {
        self.inner.get_nines_index_offsets_ptr()
    }

    pub fn get_nines_index_offsets_len(&self) -> usize {
        self.inner.get_nines_index_offsets_len()
    }

    pub fn get_local_uvs_buf(&self) -> *const f32 {
        self.inner.get_local_uvs_ptr()
    }

    pub fn get_local_uvs_len(&self) -> usize {
        self.inner.get_local_uvs_len()
    }

    pub fn get_cell_meta_buf(&self) -> *const f32 {
        self.inner.get_cell_meta_ptr()
    }

    pub fn get_cell_meta_len(&self) -> usize {
        self.inner.get_cell_meta_len()
    }

    pub fn get_local_vertices_buf(&self) -> *const f32 {
        self.inner.get_local_vertices_ptr()
    }

    pub fn get_local_vertices_len(&self) -> usize {
        self.inner.get_local_vertices_len()
    }

    pub fn get_shape_vertices_buf(&self) -> *const f32 {
        self.inner.get_shape_vertices_ptr()
    }

    pub fn get_shape_vertices_len(&self) -> usize {
        self.inner.get_shape_vertices_len()
    }

    pub fn get_shape_vertex_box_coords_buf(&self) -> *const f32 {
        self.inner.get_shape_vertex_box_coords_ptr()
    }

    pub fn get_shape_vertex_box_coords_len(&self) -> usize {
        self.inner.get_shape_vertex_box_coords_len()
    }

    pub fn get_shape_vertex_counts_buf(&self) -> *const i32 {
        self.inner.get_shape_vertex_counts_ptr()
    }

    pub fn get_shape_vertex_counts_len(&self) -> usize {
        self.inner.get_shape_vertex_counts_len()
    }

    pub fn get_ssab_buf(&self) -> Result<*const u8, JsValue> {
        self.inner.get_ssab_data().map(|s| s.as_ptr()).ok_or_else(|| JsValue::from_str("SSAB data not found"))
    }

    pub fn get_ssab_len(&self) -> Result<usize, JsValue> {
        self.inner.get_ssab_data().map(|s| s.len()).ok_or_else(|| JsValue::from_str("SSAB data not found"))
    }

    // Frame Number getter functions
    pub fn get_frame_no(&self) -> f32 {
        self.inner.get_frame_no()
    }

    /// `true` once the configured loops have completed, and sticky until the
    /// next `play` / `setup_animation`. Never `true` under an infinite loop
    /// count, and never raised by `stop`.
    ///
    /// This is a *state*, not a pulse: to notify once, take its rising edge.
    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }

    pub fn just_looped(&self) -> bool {
        self.inner.just_looped()
    }

    // Playback state and setting getters
    pub fn is_playing(&self) -> bool {
        self.inner.is_playing()
    }

    pub fn is_playing_forward(&self) -> bool {
        self.inner.is_playing_forward()
    }

    pub fn is_pausing(&self) -> bool {
        self.inner.is_pausing()
    }

    /// The character's clock: `true` from `play` until `stop` or the next
    /// setup, after track 0 has finished as well.
    pub fn is_clock_playing(&self) -> bool {
        self.inner.is_clock_playing()
    }

    /// `true` while `pause` holds a playing clock, after track 0 has finished
    /// as well. Implies `is_clock_playing`.
    pub fn is_clock_pausing(&self) -> bool {
        self.inner.is_clock_pausing()
    }

    /// `true` while the track stack would move the pose on a running clock for
    /// a reason other than track 0's own animation stepping: a fade on any
    /// track, or another source still playing on a track that is heard.
    /// Independent of the clock; read it with `is_clock_playing` and
    /// `is_clock_pausing`.
    pub fn is_stack_in_motion(&self) -> bool {
        self.inner.is_stack_in_motion()
    }

    pub fn get_skip_frames(&self) -> bool {
        self.inner.get_skip_frames()
    }

    pub fn get_playback_direction(&self) -> WebPlaybackDirection {
        self.inner.get_playback_direction().into()
    }

    pub fn get_playback_style(&self) -> WebPlaybackStyle {
        self.inner.get_playback_style().into()
    }

    pub fn get_loop_count(&self) -> i32 {
        self.inner.get_loop_count()
    }

    pub fn get_remaining_loops(&self) -> i32 {
        self.inner.get_remaining_loops()
    }

    pub fn get_start_frame(&self) -> i32 {
        self.inner.get_start_frame()
    }

    pub fn get_end_frame(&self) -> i32 {
        self.inner.get_end_frame()
    }

    pub fn get_frame_rate(&self) -> i32 {
        self.inner.get_frame_rate()
    }

    pub fn get_speed_scale(&self) -> f32 {
        self.inner.get_speed_scale()
    }

    // Playback setup
    pub fn set_skip_frames(&mut self, skip: bool) {
        self.inner.set_skip_frames(skip);
    }

    pub fn set_loop_count(&mut self, loop_count: i32) {
        self.inner.set_loop_count(loop_count);
    }

    pub fn set_frame_rate(&mut self, frame_rate: i32) {
        self.inner.set_frame_rate(frame_rate);
    }

    pub fn set_playback_direction(&mut self, direction: WebPlaybackDirection, style: WebPlaybackStyle) {
        self.inner.set_playback_direction(direction.into(), style.into());
    }

    pub fn set_speed_scale(&mut self, speed_scale: f32) {
        self.inner.set_speed_scale(speed_scale);
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.inner.set_animation_section(start_frame, end_frame);
    }

    pub fn set_coordinate_system(&mut self, coordinate_system: WebCoordinateSystem) {
        self.inner.set_coordinate_system(coordinate_system.into());
    }

    pub fn get_coordinate_system(&self) -> WebCoordinateSystem {
        self.inner.get_coordinate_system().into()
    }

    pub fn set_origin(&mut self, origin: WebOrigin) {
        self.inner.set_origin(origin.into());
    }

    pub fn get_origin(&self) -> WebOrigin {
        self.inner.get_origin().into()
    }

    pub fn set_unit(&mut self, unit: WebUnit) {
        self.inner.set_unit(unit.into());
    }

    pub fn get_unit(&self) -> WebUnit {
        self.inner.get_unit().into()
    }

    pub fn update(&mut self, delta: f32) -> f32 {
        self.inner.update(delta)
    }

    pub fn play(&mut self) {
        self.inner.play();
    }

    pub fn play_with_start_frame(&mut self, frame_no: f32) {
        self.inner.play_with_start_frame(frame_no);
    }

    /// Holds playback in place (idempotent). `is_playing` stays true; read
    /// `is_pausing` to tell a held animation from a running one.
    pub fn pause(&mut self) {
        self.inner.pause();
    }

    /// Lifts a `pause` (idempotent). `play` rewinds, so it is not a resume.
    pub fn resume(&mut self) {
        self.inner.resume();
    }

    pub fn stop(&mut self) {
        self.inner.stop();
    }

    pub fn set_frame_no(&mut self, frame: f32) {
        self.inner.set_frame_no(frame);
    }

    pub fn next_frame(&mut self) {
        self.inner.next_frame();
    }

    pub fn prev_frame(&mut self) {
        self.inner.prev_frame();
    }

    // Animation mixing — the track stack, mirroring the C FFI surface. Tracks
    // fold onto the setup pose in ascending order: a track's `alpha` is its
    // strength, while a source's `weight` only sets proportions among several
    // sources sharing one track. Track 0 carries the structure and events.

    pub fn set_track_animation(&mut self, track: u32, name: &str) -> Result<(), JsValue> {
        self.inner.set_track_animation(track, name).map_err(js_error)
    }

    pub fn set_track_animation_by_hash(&mut self, track: u32, hash: u32) -> Result<(), JsValue> {
        self.inner.set_track_animation_by_hash(track, hash).map_err(js_error)
    }

    pub fn set_track_alpha(&mut self, track: u32, alpha: f32) -> Result<(), JsValue> {
        self.inner.set_track_alpha(track, alpha).map_err(js_error)
    }

    pub fn get_track_alpha(&self, track: u32) -> Option<f32> {
        self.inner.get_track_alpha(track)
    }

    /// `blend`: 0 = mix (lerp onto what is below), 1 = additive (offset from the setup pose).
    pub fn set_track_blend(&mut self, track: u32, blend: u32) -> Result<(), JsValue> {
        let blend = match blend {
            0 => crate::TrackBlend::Mix,
            1 => crate::TrackBlend::Additive,
            _ => return Err(JsValue::from_str("blend must be 0 (mix) or 1 (additive)")),
        };
        self.inner.set_track_blend(track, blend).map_err(js_error)
    }

    pub fn get_track_blend(&self, track: u32) -> Option<u32> {
        self.inner.get_track_blend(track).map(|b| match b {
            crate::TrackBlend::Mix => 0,
            crate::TrackBlend::Additive => 1,
        })
    }

    /// `sync`: 0 = free (start from the head), 1 = start (start at track 0's phase).
    /// Set it before laying an animation on the track.
    pub fn set_track_sync(&mut self, track: u32, sync: u32) -> Result<(), JsValue> {
        let sync = match sync {
            0 => crate::TrackSync::Free,
            1 => crate::TrackSync::Start,
            _ => return Err(JsValue::from_str("sync must be 0 (free) or 1 (start)")),
        };
        self.inner.set_track_sync(track, sync).map_err(js_error)
    }

    pub fn get_track_sync(&self, track: u32) -> Option<u32> {
        self.inner.get_track_sync(track).map(|s| match s {
            crate::TrackSync::Free => 0,
            crate::TrackSync::Start => 1,
        })
    }

    pub fn get_track_count(&self) -> u32 {
        self.inner.get_track_count()
    }

    pub fn clear_track(&mut self, track: u32) -> Result<(), JsValue> {
        self.inner.clear_track(track).map_err(js_error)
    }

    pub fn crossfade(&mut self, track: u32, name: &str, duration_ms: f32) -> Result<(), JsValue> {
        self.inner.crossfade(track, name, duration_ms).map_err(js_error)
    }

    pub fn crossfade_by_hash(&mut self, track: u32, hash: u32, duration_ms: f32) -> Result<(), JsValue> {
        self.inner.crossfade_by_hash(track, hash, duration_ms).map_err(js_error)
    }

    pub fn fade_track(&mut self, track: u32, target_alpha: f32, duration_ms: f32) -> Result<(), JsValue> {
        self.inner.fade_track(track, target_alpha, duration_ms).map_err(js_error)
    }

    pub fn get_track_fade_progress(&self, track: u32) -> Option<f32> {
        self.inner.get_track_fade_progress(track)
    }

    pub fn cancel_track_fade(&mut self, track: u32) -> Result<(), JsValue> {
        self.inner.cancel_track_fade(track).map_err(js_error)
    }

    pub fn set_track_frame(&mut self, track: u32, frame: f32) -> Result<(), JsValue> {
        self.inner.set_track_frame(track, frame).map_err(js_error)
    }

    pub fn get_track_frame(&self, track: u32) -> Option<f32> {
        self.inner.get_track_frame(track)
    }

    pub fn set_track_speed(&mut self, track: u32, speed: f32) -> Result<(), JsValue> {
        self.inner.set_track_speed(track, speed).map_err(js_error)
    }

    pub fn get_track_speed(&self, track: u32) -> Option<f32> {
        self.inner.get_track_speed(track)
    }

    pub fn set_track_frame_rate(&mut self, track: u32, frame_rate: i32) -> Result<(), JsValue> {
        self.inner.set_track_frame_rate(track, frame_rate).map_err(js_error)
    }

    pub fn get_track_frame_rate(&self, track: u32) -> Option<i32> {
        self.inner.get_track_frame_rate(track)
    }

    pub fn set_track_skip_frames(&mut self, track: u32, skip: bool) -> Result<(), JsValue> {
        self.inner.set_track_skip_frames(track, skip).map_err(js_error)
    }

    pub fn get_track_skip_frames(&self, track: u32) -> Option<bool> {
        self.inner.get_track_skip_frames(track)
    }

    pub fn set_track_playback_direction(&mut self, track: u32, direction: WebPlaybackDirection, style: WebPlaybackStyle) -> Result<(), JsValue> {
        self.inner.set_track_playback_direction(track, direction.into(), style.into()).map_err(js_error)
    }

    pub fn get_track_playback_direction(&self, track: u32) -> Option<WebPlaybackDirection> {
        self.inner.get_track_playback_direction(track).map(Into::into)
    }

    pub fn get_track_playback_style(&self, track: u32) -> Option<WebPlaybackStyle> {
        self.inner.get_track_playback_style(track).map(Into::into)
    }

    /// Belongs to the animation on the track: laying another one there starts
    /// it looping forever again, so state the count after laying it.
    pub fn set_track_loop_count(&mut self, track: u32, loop_count: i32) -> Result<(), JsValue> {
        self.inner.set_track_loop_count(track, loop_count).map_err(js_error)
    }

    pub fn get_track_loop_count(&self, track: u32) -> Option<i32> {
        self.inner.get_track_loop_count(track)
    }

    /// `is_finished` for one track; `false` for a track with no animation.
    pub fn is_track_finished(&self, track: u32) -> bool {
        self.inner.is_track_finished(track)
    }

    /// `just_looped` for one track.
    pub fn track_just_looped(&self, track: u32) -> bool {
        self.inner.track_just_looped(track)
    }

    /// The outgoing side of a crossfade on the track fires events while its
    /// share, `1 - t`, is at least this. Silent from the start by default.
    pub fn set_track_outgoing_event_threshold(&mut self, track: u32, threshold: f32) -> Result<(), JsValue> {
        self.inner.set_track_outgoing_event_threshold(track, threshold).map_err(js_error)
    }

    pub fn get_track_outgoing_event_threshold(&self, track: u32) -> Option<f32> {
        self.inner.get_track_outgoing_event_threshold(track)
    }

    pub fn get_track_main_source(&self, track: u32) -> Option<u32> {
        self.inner.get_track_main_source(track)
    }

    pub fn add_track_source(&mut self, track: u32, name: &str, weight: f32) -> Result<u32, JsValue> {
        self.inner.add_track_source(track, name, weight).map_err(js_error)
    }

    pub fn add_track_source_by_hash(&mut self, track: u32, hash: u32, weight: f32) -> Result<u32, JsValue> {
        self.inner.add_track_source_by_hash(track, hash, weight).map_err(js_error)
    }

    pub fn set_source_weight(&mut self, handle: u32, weight: f32) -> Result<(), JsValue> {
        self.inner.set_source_weight(handle, weight).map_err(js_error)
    }

    pub fn get_source_weight(&self, handle: u32) -> Option<f32> {
        self.inner.get_source_weight(handle)
    }

    pub fn set_source_frame(&mut self, handle: u32, frame: f32) -> Result<(), JsValue> {
        self.inner.set_source_frame(handle, frame).map_err(js_error)
    }

    pub fn get_source_frame(&self, handle: u32) -> Option<f32> {
        self.inner.get_source_frame(handle)
    }

    /// Which animation the source plays, as an index into the ssab's animations;
    /// the table a passed event index addresses.
    pub fn get_source_animation_index(&self, handle: u32) -> Option<u32> {
        self.inner.get_source_animation_index(handle)
    }

    /// Which animation track 0's main source plays, as an index into the
    /// ssab's animations; a crossfade or a track animation on track 0 changes
    /// it from the call on.
    pub fn get_current_animation_index(&self) -> Option<u32> {
        self.inner.get_current_animation_index()
    }

    pub fn set_source_event_threshold(&mut self, handle: u32, threshold: f32) -> Result<(), JsValue> {
        self.inner.set_source_event_threshold(handle, threshold).map_err(js_error)
    }

    pub fn get_source_event_threshold(&self, handle: u32) -> Option<f32> {
        self.inner.get_source_event_threshold(handle)
    }

    pub fn get_passed_event_source_handle(&self, index: i32) -> u32 {
        if index < 0 {
            return u32::MAX;
        }
        self.inner.get_passed_event_source_handles().get(index as usize).copied().unwrap_or(u32::MAX)
    }

    pub fn get_passed_event_count(&self) -> i32 {
        self.inner.get_passed_event_indices().len() as i32
    }

    pub fn get_passed_event_index(&self, index: i32) -> i32 {
        self.inner.get_passed_event_indices().get(index as usize).copied().unwrap_or(-1)
    }

    /// Mirror of `ss_runtime_get_active_event_instance`. Returns SS6 defaults
    /// (`is_synthetic_default = true`, `loop_num = 1`, `speed = 1.0`) when no
    /// event is active. Most Players should pass the returned value directly
    /// to `SSRuntimeInstanceSlot::step`, which resolves the labels internally.
    pub fn get_active_event_instance(&mut self, slot_index: u32) -> WebEventInstanceInfo {
        self.inner.get_active_event_instance(slot_index).into()
    }

    /// Mirror of `ss_runtime_get_active_effect_event`.
    pub fn get_active_effect_event(&mut self, slot_index: u32) -> WebEffectEventInfo {
        self.inner.get_active_effect_event(slot_index).into()
    }

    // ---- Override Layer: per-part runtime overrides ----------------------
    // Mirrors the `ss_runtime_*_override` C FFI in `src/ffi/mod.rs`.

    /// Mirror of `ss_runtime_find_part_index`. `-1` if not loaded / not found.
    pub fn find_part_index(&self, part_name: &str) -> i32 {
        self.inner.find_part_index(part_name)
    }

    /// Mirror of `ss_runtime_set_part_visibility_override`.
    /// `force`: 0=clear/revert, 1=force-hide, 2=force-visible (reserved).
    pub fn set_part_visibility_override(&mut self, part: u32, force: u8, cascade: bool) -> Result<(), JsValue> {
        self.inner.set_part_visibility_override(part, force, cascade).map_err(js_error)
    }

    /// Mirror of `ss_runtime_clear_part_visibility_override`.
    pub fn clear_part_visibility_override(&mut self, part: u32) -> Result<(), JsValue> {
        self.inner.clear_part_visibility_override(part).map_err(js_error)
    }

    /// Mirror of `ss_runtime_set_part_color_override`. `rgba` packed `0xRRGGBBAA`
    /// (fills all four corners); `blend_op`: 0=Mix 1=Mul 2=Add 3=Sub;
    /// `priority_mode`: 0=OverwriteOnNextKeyframe 1=HoldUntilNextAnimation 2=Permanent.
    pub fn set_part_color_override(&mut self, part: u32, blend_op: u8, rgba: u32, priority_mode: u8) -> Result<(), JsValue> {
        self.inner.set_part_color_override(part, blend_op, rgba, priority_mode).map_err(js_error)
    }

    /// Mirror of `ss_runtime_set_part_color_override_corners`. `rgba_ltrtlbrb` =
    /// 4 packed RGBA in ssruntime corner order `lt, rt, lb, rb`.
    pub fn set_part_color_override_corners(&mut self, part: u32, blend_op: u8, rgba_ltrtlbrb: &[u32], priority_mode: u8) -> Result<(), JsValue> {
        if rgba_ltrtlbrb.len() < 4 {
            return Err(JsValue::from_str("rgba_ltrtlbrb must have 4 elements"));
        }
        let corners = [rgba_ltrtlbrb[0].to_be_bytes(), rgba_ltrtlbrb[1].to_be_bytes(), rgba_ltrtlbrb[2].to_be_bytes(), rgba_ltrtlbrb[3].to_be_bytes()];
        self.inner.set_part_color_override_corners(part, blend_op, corners, priority_mode).map_err(js_error)
    }

    /// Mirror of `ss_runtime_clear_part_color_override`.
    pub fn clear_part_color_override(&mut self, part: u32) -> Result<(), JsValue> {
        self.inner.clear_part_color_override(part).map_err(js_error)
    }

    /// Mirror of `ss_runtime_set_part_cell_override`. `(0, 0)` reverts to the
    /// animation; Normal + Mask parts only; errors if the cell is unresolvable.
    pub fn set_part_cell_override(&mut self, part: u32, cellmap_name_hash: u32, cell_name_hash: u32, priority_mode: u8) -> Result<(), JsValue> {
        self.inner.set_part_cell_override(part, cellmap_name_hash, cell_name_hash, priority_mode).map_err(js_error)
    }

    /// Mirror of `ss_runtime_clear_part_cell_override`.
    pub fn clear_part_cell_override(&mut self, part: u32) -> Result<(), JsValue> {
        self.inner.clear_part_cell_override(part).map_err(js_error)
    }

    /// Mirror of `ss_runtime_clear_all_part_overrides`.
    pub fn clear_all_part_overrides(&mut self) {
        self.inner.clear_all_part_overrides();
    }
}

/// JS-friendly wrapper of `EffectSlot`. Same conceptual flow as the C FFI:
/// 1) construct via `create`, 2) call `step` per frame with the resolved
/// active event and timing, 3) read `get_draw_plan_buf`/`get_draw_plan_len`
/// only when `step` returned `true` and only before the next call.
#[wasm_bindgen]
pub struct SSRuntimeEffectSlot {
    inner: OwningEffectSlot,
    last_draw_plan_ptr: *const u8,
    last_draw_plan_len: usize,
}

#[wasm_bindgen]
impl SSRuntimeEffectSlot {
    pub fn create(resource: &SSRuntimeResource, effect_name_hash: u32, seed: u32) -> Result<SSRuntimeEffectSlot, JsValue> {
        match OwningEffectSlot::new(resource.inner.clone(), effect_name_hash, seed) {
            Ok(inner) => Ok(SSRuntimeEffectSlot { inner, last_draw_plan_ptr: core::ptr::null(), last_draw_plan_len: 0 }),
            Err(e) => Err(js_error(e)),
        }
    }

    /// Advance one frame. `ev` is the event info returned by
    /// `SSRuntimeContext::get_active_effect_event` (or built manually via
    /// `WebEffectEventInfo`).
    ///
    /// `y_flip = true` for Y-down rendering (pixi.js); `vert_stride = 2` packs
    /// (x,y) verts, `3` packs (x,y,0) for engines requiring 3-component verts.
    /// `draw` is whether the part is in this frame's `draw_order`: a hidden
    /// part steps with `false`, which moves its clock and builds no plan. Pass
    /// `true` when stepping before this frame's `draw_order` is known.
    /// Returns `true` if visible; on `true` callers read the plan via
    /// `get_draw_plan_buf` / `get_draw_plan_len` before the next step.
    #[allow(clippy::too_many_arguments)]
    pub fn step(&mut self, ev: &WebEffectEventInfo, parent_frame: f32, delta_seconds: f32, fps: f32, parent_looped: bool, y_flip: bool, vert_stride: u8, draw: bool) -> bool {
        let r = self.inner.step((*ev).into(), parent_frame, delta_seconds, fps, parent_looped, y_flip, vert_stride, draw);
        self.last_draw_plan_ptr = r.draw_plan_buf;
        self.last_draw_plan_len = r.draw_plan_len;
        r.visible
    }

    pub fn get_draw_plan_buf(&self) -> *const u8 {
        self.last_draw_plan_ptr
    }

    pub fn get_draw_plan_len(&self) -> usize {
        self.last_draw_plan_len
    }

    pub fn get_seed(&self) -> u32 {
        self.inner.get_seed()
    }

    pub fn is_independent(&self) -> bool {
        self.inner.is_independent()
    }

    /// Reset the slot's playback / "last started key" memory to its just-created
    /// state without destroying it. Call on an explicit head-out (replay) so a
    /// re-played animation re-fires its effect from scratch instead of debouncing
    /// the first key as the same one the slot last started. Mirrors the C-API
    /// `ss_effect_slot_reset`.
    pub fn reset(&mut self) {
        self.inner.reset();
        self.last_draw_plan_ptr = core::ptr::null();
        self.last_draw_plan_len = 0;
    }
}

/// JS-friendly wrapper of `InstanceSlot`. Same conceptual flow as the C FFI:
/// 1) construct via `create`, 2) call `step` per frame with the resolved
/// active event, child context, and timing, 3) on `transitioned == true`
/// call `play` on the child context: the slot configures the child for the
/// new key without starting it, and an independent child never advances
/// otherwise.
#[wasm_bindgen]
pub struct SSRuntimeInstanceSlot {
    inner: crate::InstanceSlot,
    last_visible: bool,
    last_transitioned: bool,
    last_independent: bool,
    last_child_frame_no: f32,
    last_child_looped: bool,
}

#[wasm_bindgen]
impl SSRuntimeInstanceSlot {
    pub fn create() -> SSRuntimeInstanceSlot {
        SSRuntimeInstanceSlot { inner: crate::InstanceSlot::new(), last_visible: false, last_transitioned: false, last_independent: false, last_child_frame_no: 0.0, last_child_looped: false }
    }

    /// Advance one tick. `ev` is the event info returned by
    /// `SSRuntimeContext::get_active_event_instance` (or built manually via
    /// `WebEventInstanceInfo`). `child_ctx` must be the *child* runtime —
    /// label resolution and frame stepping run against it.
    /// Returns `true` if visible. Inspect `transitioned`, `independent`,
    /// `child_frame_no`, `child_looped` via the corresponding getters.
    pub fn step(&mut self, ev: &WebEventInstanceInfo, child_ctx: &mut SSRuntimeContext, parent_frame: f32, delta_seconds: f32, parent_looped: bool) -> bool {
        let r = self.inner.step((*ev).into(), &mut child_ctx.inner, parent_frame, delta_seconds, parent_looped);
        self.last_visible = r.visible;
        self.last_transitioned = r.transitioned;
        self.last_independent = r.independent;
        self.last_child_frame_no = r.child_frame_no;
        self.last_child_looped = r.child_looped;
        r.visible
    }

    pub fn transitioned(&self) -> bool {
        self.last_transitioned
    }

    pub fn independent(&self) -> bool {
        self.last_independent
    }

    pub fn child_frame_no(&self) -> f32 {
        self.last_child_frame_no
    }

    pub fn child_looped(&self) -> bool {
        self.last_child_looped
    }

    pub fn is_independent(&self) -> bool {
        self.inner.is_independent()
    }

    /// The speed the last `step` drove the child at. Scale the elapsed time
    /// handed to the child's own slots by its absolute value, not by the event's
    /// `speed`, which reads `1.0` while the slot ignores the implicit key. Mirrors
    /// the C-API `ss_instance_slot_get_speed`.
    pub fn get_speed(&self) -> f32 {
        self.inner.get_speed()
    }

    /// Reset the slot's "last started key" memory to its just-created state
    /// without destroying it. Call on an explicit head-out (replay) so a
    /// re-played animation re-fires the instance child from scratch instead of
    /// debouncing the first key as the same one the slot last started. Mirrors
    /// the C-API `ss_instance_slot_reset`.
    pub fn reset(&mut self) {
        self.inner.reset();
        self.last_visible = false;
        self.last_transitioned = false;
        self.last_independent = false;
        self.last_child_frame_no = 0.0;
        self.last_child_looped = false;
    }
}

#[wasm_bindgen]
pub fn ss_runtime_hash_string(s: &str) -> u32 {
    crate::fnv1a_hash_str(s)
}
