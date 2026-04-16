use crate::Context;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SSRuntimeResource {
    #[allow(dead_code)]
    data: Box<crate::SsabData>,
    pub(crate) resource: crate::AnimationResource<'static>,
}

#[wasm_bindgen]
impl SSRuntimeResource {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<SSRuntimeResource, JsValue> {
        let ssab_data = Box::new(crate::SsabData::load_copy(data).map_err(|e| JsValue::from_str(&e.to_string()))?);
        let data_ptr = ssab_data.as_slice() as *const [u8];
        let resource = crate::AnimationResource::new(unsafe { &*data_ptr }).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(SSRuntimeResource { data: ssab_data, resource })
    }

    pub fn get_animation_names(&self) -> Vec<String> {
        self.resource.get_animation_names()
    }
}

#[wasm_bindgen]
pub struct SSRuntimeContext {
    inner: Context<'static>,
    // For backward compatibility, we can hold a resource if created via old constructor
    #[allow(dead_code)]
    bound_resource: Option<SSRuntimeResource>,
}

#[wasm_bindgen]
impl SSRuntimeContext {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Option<Box<[u8]>>) -> Result<SSRuntimeContext, JsValue> {
        let mut ctx = SSRuntimeContext { inner: Context::new(), bound_resource: None };
        if let Some(d) = data {
            let res = SSRuntimeResource::new(&d)?;
            ctx.bind_resource(&res);
            ctx.bound_resource = Some(res);
        }
        Ok(ctx)
    }

    pub fn bind_resource(&mut self, resource: &SSRuntimeResource) {
        let res_ptr = &resource.resource as *const crate::AnimationResource<'static>;
        self.inner.bind_resource(unsafe { &*res_ptr });
    }

    pub fn get_animation_names(&self) -> Result<Vec<String>, JsValue> {
        self.inner.resource.map(|r| r.get_animation_names()).ok_or_else(|| JsValue::from_str("Resource not bound"))
    }

    pub fn setup_animation(&mut self, name: &str) -> bool {
        self.inner.setup_animation(name).is_ok()
    }

    pub fn get_frame_data(&mut self, frame_no: i32) -> Option<Vec<u8>> {
        self.inner.get_frame_data(frame_no).ok().map(|s| s.to_vec())
    }

    pub fn get_ssab_data(&self) -> Option<Vec<u8>> {
        self.inner.get_ssab_data().map(|s| s.to_vec())
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

    pub fn get_playback_direction(&self) -> i32 {
        if self.inner.get_playback_direction() == crate::core::playback_controller::PlaybackDirection::Forward { 0 } else { 1 }
    }

    pub fn get_playback_style(&self) -> i32 {
        self.inner.get_playback_style() as i32
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

    pub fn set_playback_direction(&mut self, direction: i32, style: i32) {
        let d = if direction == 0 { crate::core::playback_controller::PlaybackDirection::Forward } else { crate::core::playback_controller::PlaybackDirection::Backward };
        let s = if style == 0 { crate::core::playback_controller::PlaybackStyle::Normal } else { crate::core::playback_controller::PlaybackStyle::PingPong };
        self.inner.set_playback_direction(d, s);
    }

    pub fn set_animation_speed(&mut self, speed_rate: f32) {
        self.inner.set_animation_speed(speed_rate);
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.inner.set_animation_section(start_frame, end_frame);
    }

    pub fn update(&mut self, delta: f32) -> i32 {
        let (frame, _decimal) = self.inner.update(delta);
        frame
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
