use crate::{Context, SsabData, SsabResource};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use wasm_bindgen::prelude::*;

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

#[wasm_bindgen]
pub struct UpdateResult {
    pub frame: i32,
    pub decimal: f32,
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
        if inner.resource.is_none() {
            return Err(JsValue::from_str("Failed to parse SSAB data"));
        }
        Ok(SSRuntimeResource { inner })
    }

    pub fn create_borrow(data: &[u8]) -> Result<SSRuntimeResource, JsValue> {
        let ssab_data = unsafe { SsabData::load_borrow(data.as_ptr(), data.len()) }.map_err(|e| JsValue::from_str(&e.to_string()))?;
        let inner = SsabResource::new(ssab_data);
        if inner.resource.is_none() {
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

    pub fn bind_resource(&mut self, resource: &SSRuntimeResource) {
        if let Some(res) = &resource.inner.resource {
            // Safety: We hold the Arc in bound_resource, so the reference stays valid.
            let res_ptr = res as *const crate::AnimationResource<'static>;
            self.inner.bind_resource(unsafe { &*res_ptr });
            self.bound_resource = Some(resource.inner.clone());
        }
    }

    pub fn load_ssab_copy(&mut self, data: &[u8]) -> bool {
        if let Ok(ssab_data) = SsabData::load_copy(data) {
            let res = SSRuntimeResource { inner: SsabResource::new(ssab_data) };
            if res.inner.resource.is_some() {
                self.bind_resource(&res);
                return true;
            }
        }
        false
    }

    pub fn load_ssab_borrow(&mut self, data: &[u8]) -> bool {
        if let Ok(ssab_data) = unsafe { SsabData::load_borrow(data.as_ptr(), data.len()) } {
            let res = SSRuntimeResource { inner: SsabResource::new(ssab_data) };
            if res.inner.resource.is_some() {
                self.bind_resource(&res);
                return true;
            }
        }
        false
    }

    pub fn setup_animation(&mut self, name: &str) -> Result<(), JsValue> {
        self.inner.setup_animation(name).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    // Zero-copy getters
    pub fn get_frame_data_buf(&mut self, frame_no: i32) -> Result<*const u8, JsValue> {
        self.inner.get_frame_data(frame_no).map(|s| s.as_ptr()).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn get_frame_data_len(&mut self, frame_no: i32) -> Result<usize, JsValue> {
        self.inner.get_frame_data(frame_no).map(|s| s.len()).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn get_ssab_buf(&self) -> Result<*const u8, JsValue> {
        self.inner.get_ssab_data().map(|s| s.as_ptr()).ok_or_else(|| JsValue::from_str("SSAB data not found"))
    }

    pub fn get_ssab_len(&self) -> Result<usize, JsValue> {
        self.inner.get_ssab_data().map(|s| s.len()).ok_or_else(|| JsValue::from_str("SSAB data not found"))
    }

    // Frame Number getter functions
    pub fn get_frame_no(&self) -> i32 {
        self.inner.get_frame_no()
    }

    pub fn get_frame_no_decimal(&self) -> f32 {
        self.inner.get_frame_no_decimal()
    }

    pub fn is_end_frame_reached(&self) -> bool {
        self.inner.is_end_frame_reached()
    }

    // setting getter functions
    pub fn is_playing(&self) -> bool {
        self.inner.is_playing()
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

    pub fn update(&mut self, delta: f32) -> UpdateResult {
        let (frame, decimal) = self.inner.update(delta);
        UpdateResult { frame, decimal }
    }

    pub fn play(&mut self) {
        self.inner.play();
    }

    pub fn play_with_start_frame(&mut self, frame_no: i32) {
        self.inner.play_with_start_frame(frame_no);
    }

    pub fn pause(&mut self) {
        self.inner.pause();
    }

    pub fn stop(&mut self) {
        self.inner.stop();
    }

    pub fn set_frame_no(&mut self, frame: i32) {
        self.inner.set_frame_no(frame);
    }

    pub fn next_frame(&mut self) {
        self.inner.next_frame();
    }

    pub fn prev_frame(&mut self) {
        self.inner.prev_frame();
    }

    pub fn get_passed_event_count(&self) -> i32 {
        self.inner.get_passed_event_frames().len() as i32
    }

    pub fn get_passed_event_frame_no(&self, index: i32) -> i32 {
        self.inner.get_passed_event_frames().get(index as usize).copied().unwrap_or(-1)
    }
}

#[wasm_bindgen]
pub struct SSRuntimeEffectContext {
    inner: crate::EffectContext<'static>,
    #[allow(dead_code)]
    resource_handle: Arc<SsabResource>,
}

#[wasm_bindgen]
impl SSRuntimeEffectContext {
    pub fn create(resource: &SSRuntimeResource, effect_name_hash: u32, seed: u32) -> Result<SSRuntimeEffectContext, JsValue> {
        if let Some(res) = &resource.inner.resource {
            // Safety: We hold the Arc in resource_handle, so the reference stays valid.
            let res_ptr = res as *const crate::AnimationResource<'static>;
            match crate::EffectContext::new(unsafe { &*res_ptr }, effect_name_hash, seed) {
                Ok(inner) => Ok(SSRuntimeEffectContext { inner, resource_handle: resource.inner.clone() }),
                Err(e) => Err(JsValue::from_str(&alloc::format!("{:?}", e))),
            }
        } else {
            Err(JsValue::from_str("Resource not initialized"))
        }
    }

    pub fn update(&mut self, frame: f32) {
        self.inner.update(frame);
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }

    pub fn get_lifetime(&self) -> i32 {
        self.inner.get_lifetime()
    }

    pub fn is_infinite(&self) -> bool {
        self.inner.is_infinite()
    }

    pub fn get_state_buf(&mut self) -> Result<*const u8, JsValue> {
        self.inner.get_state().map(|s| s.as_ptr()).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn get_state_len(&mut self) -> Result<usize, JsValue> {
        self.inner.get_state().map(|s| s.len()).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }
}

#[wasm_bindgen]
pub struct SSRuntimeUtils;

#[wasm_bindgen]
impl SSRuntimeUtils {
    pub fn calculate_instance_frame(parent_frame: f32, key_frame: i32, start_offset: i32, speed: f32, independent: bool, accumulated_time: f32) -> f32 {
        crate::core::util::calculate_instance_frame(parent_frame, key_frame, start_offset, speed, independent, accumulated_time)
    }

    pub fn get_part_world_matrix(context: &SSRuntimeContext, part_index: u16, frame_no: i32) -> Option<Box<[f32]>> {
        if let Some(res) = context.inner.get_resource()
            && let Some(res_m) = crate::core::util::get_part_world_matrix(&res.animations[context.inner.get_animation_idx()? as usize].cached_parts, part_index, frame_no)
        {
            return Some(Box::new(res_m.m));
        }
        None
    }

    pub fn transform_particles(matrix_arr: &[f32], particles_ptr: *const u8, count: i32) -> Result<Box<[f32]>, JsValue> {
        if matrix_arr.len() < 16 {
            return Err(JsValue::from_str("Invalid matrix size"));
        }
        let mut m_arr = [0.0f32; 16];
        m_arr.copy_from_slice(&matrix_arr[0..16]);
        let matrix = crate::core::SsMatrix4x4::from_array(m_arr);

        let len = (count as usize) * 28;
        let particles_raw = unsafe { core::slice::from_raw_parts(particles_ptr, len) };

        let transformed = crate::core::util::transform_particles(&matrix, particles_raw);
        let mut result = alloc::vec::Vec::with_capacity(transformed.len() * 2);
        for (x, y) in transformed {
            result.push(x);
            result.push(y);
        }
        Ok(result.into_boxed_slice())
    }
}
