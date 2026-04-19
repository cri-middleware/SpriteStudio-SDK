use crate::{Context, SsabData, SsabResource};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
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

    pub fn get_animation_names(&self) -> Vec<String> {
        self.inner.resource.as_ref().map(|r| r.get_animation_names()).unwrap_or_default()
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

    pub fn bind_resource(&mut self, resource: &SSRuntimeResource) {
        if let Some(res) = &resource.inner.resource {
            // Safety: We hold the Arc in bound_resource, so the reference stays valid.
            let res_ptr = res as *const crate::AnimationResource<'static>;
            self.inner.bind_resource(unsafe { &*res_ptr });
            self.bound_resource = Some(resource.inner.clone());
        }
    }

    pub fn get_animation_names(&self) -> Result<Vec<String>, JsValue> {
        self.inner.resource.map(|r| r.get_animation_names()).ok_or_else(|| JsValue::from_str("Resource not bound"))
    }

    pub fn setup_animation(&mut self, name: &str) -> Result<(), JsValue> {
        self.inner.setup_animation(name).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    // Conventional getter (Allocates / Copies)
    pub fn get_frame_data(&mut self, frame_no: i32) -> Result<Vec<u8>, JsValue> {
        self.inner.get_frame_data(frame_no).map(|s| s.to_vec()).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    // Zero-copy getters
    pub fn get_frame_data_ptr(&mut self, frame_no: i32) -> Result<*const u8, JsValue> {
        self.inner.get_frame_data(frame_no).map(|s| s.as_ptr()).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn get_frame_data_len(&mut self, frame_no: i32) -> Result<usize, JsValue> {
        self.inner.get_frame_data(frame_no).map(|s| s.len()).map_err(|e| JsValue::from_str(&alloc::format!("{:?}", e)))
    }

    pub fn get_ssab_data(&self) -> Result<Vec<u8>, JsValue> {
        self.inner.get_ssab_data().map(|s| s.to_vec()).ok_or_else(|| JsValue::from_str("SSAB data not found"))
    }

    pub fn get_ssab_data_ptr(&self) -> Result<*const u8, JsValue> {
        self.inner.get_ssab_data().map(|s| s.as_ptr()).ok_or_else(|| JsValue::from_str("SSAB data not found"))
    }

    pub fn get_ssab_data_len(&self) -> Result<usize, JsValue> {
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
}
