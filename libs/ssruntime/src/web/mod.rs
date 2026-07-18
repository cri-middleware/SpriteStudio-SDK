//! Wasm bindings for ssruntime.
//!
//! This module mirrors the C FFI surface in `src/ffi/mod.rs` 1:1 for JS/TS
//! Players. The FFI side is authoritative — when adding or modifying public
//! API, update `src/ffi/mod.rs` first and reflect the change here. POD return
//! types (`ss_event_instance_info`, `ss_effect_event_info`) are exposed as
//! wasm-bindgen wrapper classes (`WebEventInstanceInfo`,
//! `WebEffectEventInfo`) because wasm-bindgen cannot expose `#[repr(C)]`
//! structs by value.
//!
//! Not yet exposed (deferred): `ss_runtime_set_log_callback` requires
//! bridging a JS `Function` and a `js-sys` dependency, which is held off
//! until the JS/TS Player needs it.

use crate::{Context, SsabData, SsabResource};
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

impl From<crate::core::SsCoordinateSystem> for WebCoordinateSystem {
    fn from(cs: crate::core::SsCoordinateSystem) -> Self {
        match cs {
            crate::core::SsCoordinateSystem::YUp => WebCoordinateSystem::YUp,
            crate::core::SsCoordinateSystem::YDown => WebCoordinateSystem::YDown,
        }
    }
}

impl From<WebCoordinateSystem> for crate::core::SsCoordinateSystem {
    fn from(cs: WebCoordinateSystem) -> Self {
        match cs {
            WebCoordinateSystem::YUp => crate::core::SsCoordinateSystem::YUp,
            WebCoordinateSystem::YDown => crate::core::SsCoordinateSystem::YDown,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebOrigin {
    Center = 0,
    TopLeft = 1,
}

impl From<crate::core::SsOrigin> for WebOrigin {
    fn from(origin: crate::core::SsOrigin) -> Self {
        match origin {
            crate::core::SsOrigin::Center => WebOrigin::Center,
            crate::core::SsOrigin::TopLeft => WebOrigin::TopLeft,
        }
    }
}

impl From<WebOrigin> for crate::core::SsOrigin {
    fn from(origin: WebOrigin) -> Self {
        match origin {
            WebOrigin::Center => crate::core::SsOrigin::Center,
            WebOrigin::TopLeft => crate::core::SsOrigin::TopLeft,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebUnit {
    Pixel = 0,
    Normalized = 1,
}

impl From<crate::core::SsUnit> for WebUnit {
    fn from(unit: crate::core::SsUnit) -> Self {
        match unit {
            crate::core::SsUnit::Pixel => WebUnit::Pixel,
            crate::core::SsUnit::Normalized => WebUnit::Normalized,
        }
    }
}

impl From<WebUnit> for crate::core::SsUnit {
    fn from(unit: WebUnit) -> Self {
        match unit {
            WebUnit::Pixel => crate::core::SsUnit::Pixel,
            WebUnit::Normalized => crate::core::SsUnit::Normalized,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebPlaybackDirection {
    Forward = 0,
    Backward = 1,
}

impl From<crate::core::playback_controller::PlaybackDirection> for WebPlaybackDirection {
    fn from(dir: crate::core::playback_controller::PlaybackDirection) -> Self {
        match dir {
            crate::core::playback_controller::PlaybackDirection::Forward => WebPlaybackDirection::Forward,
            crate::core::playback_controller::PlaybackDirection::Backward => WebPlaybackDirection::Backward,
        }
    }
}

impl From<WebPlaybackDirection> for crate::core::playback_controller::PlaybackDirection {
    fn from(dir: WebPlaybackDirection) -> Self {
        match dir {
            WebPlaybackDirection::Forward => crate::core::playback_controller::PlaybackDirection::Forward,
            WebPlaybackDirection::Backward => crate::core::playback_controller::PlaybackDirection::Backward,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebPlaybackStyle {
    Normal = 0,
    PingPong = 1,
}

impl From<crate::core::playback_controller::PlaybackStyle> for WebPlaybackStyle {
    fn from(style: crate::core::playback_controller::PlaybackStyle) -> Self {
        match style {
            crate::core::playback_controller::PlaybackStyle::Normal => WebPlaybackStyle::Normal,
            crate::core::playback_controller::PlaybackStyle::PingPong => WebPlaybackStyle::PingPong,
        }
    }
}

impl From<WebPlaybackStyle> for crate::core::playback_controller::PlaybackStyle {
    fn from(style: WebPlaybackStyle) -> Self {
        match style {
            WebPlaybackStyle::Normal => crate::core::playback_controller::PlaybackStyle::Normal,
            WebPlaybackStyle::PingPong => crate::core::playback_controller::PlaybackStyle::PingPong,
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

#[wasm_bindgen]
pub struct SSRuntimeResource {
    pub(crate) inner: Arc<SsabResource>,
}

#[wasm_bindgen]
impl SSRuntimeResource {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<SSRuntimeResource, JsValue> {
        let ssab_data = SsabData::load_copy(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let inner = SsabResource::new(ssab_data);
        if inner.resource().is_none() {
            return Err(JsValue::from_str("Failed to parse SSAB data"));
        }
        Ok(SSRuntimeResource { inner })
    }
}

#[wasm_bindgen]
pub struct SSRuntimeContext {
    inner: Context<'static>,
    // Hold the resource to keep it alive
    bound_resource: Option<Arc<SsabResource>>,
}

#[wasm_bindgen]
impl SSRuntimeContext {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Option<Box<[u8]>>) -> Result<SSRuntimeContext, JsValue> {
        let mut ctx = SSRuntimeContext { inner: Context::new(), bound_resource: None };
        if let Some(d) = data {
            let res = SSRuntimeResource::new(&d)?;
            ctx.bind_resource(&res);
        }
        Ok(ctx)
    }

    pub fn reset(&mut self) {
        self.inner.reset();
        self.bound_resource = None;
    }

    pub fn bind_resource(&mut self, resource: &SSRuntimeResource) -> bool {
        let res_ptr = resource.inner.resource_static_ptr();
        if res_ptr.is_null() {
            return false;
        }
        // Safety: We clone the Arc into bound_resource, keeping the buffer alive.
        self.bound_resource = Some(resource.inner.clone());
        self.inner.bind_resource(unsafe { &*res_ptr });
        true
    }

    pub fn load_ssab_copy(&mut self, data: &[u8]) -> bool {
        if let Ok(ssab_data) = SsabData::load_copy(data) {
            let res = SSRuntimeResource { inner: SsabResource::new(ssab_data) };
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
        self.inner.set_render_mode(crate::core::RenderMode::from_u8(mode));
    }

    pub fn setup_animation(&mut self, name: &str) -> Result<(), JsValue> {
        self.inner.setup_animation(name).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn setup_animation_by_hash(&mut self, name_hash: u32) -> Result<(), JsValue> {
        self.inner.setup_animation_by_hash(name_hash).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    // Zero-copy getters
    pub fn get_frame_data_buf(&mut self, frame_no: f32) -> Result<*const u8, JsValue> {
        self.inner.get_frame_data(frame_no).map(|s| s.as_ptr()).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn get_frame_data_len(&mut self, frame_no: f32) -> Result<usize, JsValue> {
        self.inner.get_frame_data(frame_no).map(|s| s.len()).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
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

    pub fn is_looped(&self) -> bool {
        self.inner.is_looped()
    }

    // setting getter functions
    pub fn is_playing(&self) -> bool {
        self.inner.is_playing()
    }

    pub fn is_playing_forward(&self) -> bool {
        self.inner.is_playing_forward()
    }

    pub fn is_pausing(&self) -> bool {
        self.inner.is_pausing()
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

    pub fn get_loops(&self) -> i32 {
        self.inner.get_loops()
    }

    pub fn get_remain_loops(&self) -> i32 {
        self.inner.get_remain_loops()
    }

    pub fn get_start_frame(&self) -> i32 {
        self.inner.get_start_frame()
    }

    pub fn get_end_frame(&self) -> i32 {
        self.inner.get_end_frame()
    }

    pub fn get_fps(&self) -> i32 {
        self.inner.get_fps()
    }

    // Playback setup
    pub fn set_skip_frames(&mut self, skip: bool) {
        self.inner.set_skip_frames(skip);
    }

    pub fn set_loop(&mut self, loops: i32) {
        self.inner.set_loop(loops);
    }

    pub fn set_frame_rate(&mut self, fps: i32) {
        self.inner.set_frame_rate(fps);
    }

    pub fn set_playback_direction(&mut self, direction: WebPlaybackDirection, style: WebPlaybackStyle) {
        self.inner.set_playback_direction(direction.into(), style.into());
    }

    pub fn set_animation_speed(&mut self, speed_rate: f32) {
        self.inner.set_animation_speed(speed_rate);
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

    pub fn pause(&mut self) {
        self.inner.pause();
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

    // Animation blending (Phase 3). Mirrors the C FFI surface; see
    // `Context::add_blend_source` for the "Primary owns the structure,
    // secondaries contribute continuous values" model.

    pub fn add_blend_source(&mut self, name: &str, weight: f32) -> Result<u32, JsValue> {
        self.inner.add_blend_source(name, weight).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn add_blend_source_by_hash(&mut self, hash: u32, weight: f32) -> Result<u32, JsValue> {
        self.inner.add_blend_source_by_hash(hash, weight).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn remove_blend_source(&mut self, handle: u32) -> Result<(), JsValue> {
        self.inner.remove_blend_source(handle).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn clear_blend_sources(&mut self) {
        self.inner.clear_blend_sources();
    }

    pub fn blend_source_count(&self) -> u32 {
        self.inner.blend_source_count()
    }

    pub fn set_source_weight(&mut self, handle: u32, weight: f32) -> Result<(), JsValue> {
        self.inner.set_source_weight(handle, weight).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn source_weight(&self, handle: u32) -> Option<f32> {
        self.inner.source_weight(handle)
    }

    pub fn set_primary(&mut self, handle: u32) -> Result<(), JsValue> {
        self.inner.set_primary(handle).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn primary_handle(&self) -> u32 {
        self.inner.primary_handle()
    }

    pub fn set_source_frame(&mut self, handle: u32, frame: f32) -> Result<(), JsValue> {
        self.inner.set_source_frame(handle, frame).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn source_frame(&self, handle: u32) -> Option<f32> {
        self.inner.source_frame(handle)
    }

    pub fn set_source_speed(&mut self, handle: u32, speed: f32) -> Result<(), JsValue> {
        self.inner.set_source_speed(handle, speed).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn add_empty_blend_source(&mut self, weight: f32) -> Result<u32, JsValue> {
        self.inner.add_empty_blend_source(weight).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn set_source_event_threshold(&mut self, handle: u32, threshold: f32) -> Result<(), JsValue> {
        self.inner.set_source_event_threshold(handle, threshold).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn source_event_threshold(&self, handle: u32) -> Option<f32> {
        self.inner.source_event_threshold(handle)
    }

    pub fn crossfade(&mut self, from: u32, to: u32, duration_ms: f32) -> Result<(), JsValue> {
        self.inner.crossfade(from, to, duration_ms).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn crossfade_progress(&self) -> Option<f32> {
        self.inner.crossfade_progress()
    }

    pub fn cancel_crossfade(&mut self) {
        self.inner.cancel_crossfade();
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

    // ---- Override Layer (Phase 2): per-part runtime overrides -------------
    // Mirrors the `ss_runtime_*_override` C FFI in `src/ffi/mod.rs`.

    /// Mirror of `ss_runtime_find_part_index`. `-1` if not loaded / not found.
    pub fn find_part_index(&self, part_name: &str) -> i32 {
        self.inner.find_part_index(part_name)
    }

    /// Mirror of `ss_runtime_set_part_visibility_override`.
    /// `force`: 0=clear/revert, 1=force-hide, 2=force-visible (reserved).
    pub fn set_part_visibility_override(&mut self, part: u32, force: u8, cascade: bool) -> Result<(), JsValue> {
        self.inner.set_part_visibility_override(part, force, cascade).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    /// Mirror of `ss_runtime_clear_part_visibility_override`.
    pub fn clear_part_visibility_override(&mut self, part: u32) -> Result<(), JsValue> {
        self.inner.clear_part_visibility_override(part).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    /// Mirror of `ss_runtime_set_part_color_override`. `rgba` packed `0xRRGGBBAA`
    /// (fills all four corners); `blend_op`: 0=Mix 1=Mul 2=Add 3=Sub;
    /// `priority_mode`: 0=OverwriteOnNextKeyframe 1=HoldUntilNextAnimation 2=Permanent.
    pub fn set_part_color_override(&mut self, part: u32, blend_op: u8, rgba: u32, priority_mode: u8) -> Result<(), JsValue> {
        self.inner.set_part_color_override(part, blend_op, rgba, priority_mode).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    /// Mirror of `ss_runtime_set_part_color_override_corners`. `rgba_ltrtlbrb` =
    /// 4 packed RGBA in ssruntime corner order `lt, rt, lb, rb`.
    pub fn set_part_color_override_corners(&mut self, part: u32, blend_op: u8, rgba_ltrtlbrb: &[u32], priority_mode: u8) -> Result<(), JsValue> {
        if rgba_ltrtlbrb.len() < 4 {
            return Err(JsValue::from_str("rgba_ltrtlbrb must have 4 elements"));
        }
        let corners = [rgba_ltrtlbrb[0].to_be_bytes(), rgba_ltrtlbrb[1].to_be_bytes(), rgba_ltrtlbrb[2].to_be_bytes(), rgba_ltrtlbrb[3].to_be_bytes()];
        self.inner.set_part_color_override_corners(part, blend_op, corners, priority_mode).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    /// Mirror of `ss_runtime_clear_part_color_override`.
    pub fn clear_part_color_override(&mut self, part: u32) -> Result<(), JsValue> {
        self.inner.clear_part_color_override(part).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    /// Mirror of `ss_runtime_set_part_cell_override`. `(0, 0)` reverts to the
    /// animation; Normal + Mask parts only; errors if the cell is unresolvable.
    pub fn set_part_cell_override(&mut self, part: u32, cellmap_name_hash: u32, cell_name_hash: u32, priority_mode: u8) -> Result<(), JsValue> {
        self.inner.set_part_cell_override(part, cellmap_name_hash, cell_name_hash, priority_mode).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    /// Mirror of `ss_runtime_clear_part_cell_override`.
    pub fn clear_part_cell_override(&mut self, part: u32) -> Result<(), JsValue> {
        self.inner.clear_part_cell_override(part).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    /// Mirror of `ss_runtime_clear_all_part_overrides`.
    pub fn clear_all_part_overrides(&mut self) {
        self.inner.clear_all_part_overrides();
    }
}

/// JS-friendly wrapper of `EffectSlot`. Same conceptual flow as the C FFI:
/// 1) construct via `create`, 2) call `step` per frame with the resolved
/// active event and timing, 3) read `draw_plan_buf`/`draw_plan_len` only
/// when `step` returned `true` and only before the next call.
#[wasm_bindgen]
pub struct SSRuntimeEffectSlot {
    inner: crate::EffectSlot<'static>,
    #[allow(dead_code)]
    resource_handle: Arc<SsabResource>,
    last_draw_plan_ptr: *const u8,
    last_draw_plan_len: usize,
}

#[wasm_bindgen]
impl SSRuntimeEffectSlot {
    pub fn create(resource: &SSRuntimeResource, effect_name_hash: u32, seed: u32) -> Result<SSRuntimeEffectSlot, JsValue> {
        let res_ptr = resource.inner.resource_static_ptr();
        if res_ptr.is_null() {
            return Err(JsValue::from_str("Resource not initialized"));
        }
        let resource_handle = resource.inner.clone();
        match crate::EffectSlot::new(unsafe { &*res_ptr }, effect_name_hash, seed) {
            Ok(inner) => Ok(SSRuntimeEffectSlot { inner, resource_handle, last_draw_plan_ptr: core::ptr::null(), last_draw_plan_len: 0 }),
            Err(e) => Err(JsValue::from_str(&alloc::format!("{:?}", e))),
        }
    }

    /// Advance one frame. `ev` is the event info returned by
    /// `SSRuntimeContext::get_active_effect_event` (or built manually via
    /// `WebEffectEventInfo`).
    ///
    /// `y_flip = true` for Y-down rendering (pixi.js); `vert_stride = 2` packs
    /// (x,y) verts, `3` packs (x,y,0) for engines requiring 3-component verts.
    /// Returns `true` if visible; on `true` callers read the plan via
    /// `get_draw_plan_buf` / `get_draw_plan_len` before the next step.
    #[allow(clippy::too_many_arguments)]
    pub fn step(&mut self, ev: &WebEffectEventInfo, parent_frame: f32, delta_seconds: f32, fps: f32, parent_looped: bool, y_flip: bool, vert_stride: u8) -> bool {
        let r = self.inner.step((*ev).into(), parent_frame, delta_seconds, fps, parent_looped, y_flip, vert_stride);
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
/// fire any host-side `play()` side effects (event sinks etc.).
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
