use wasm_bindgen::prelude::*;
use crate::Context;

#[wasm_bindgen]
pub struct SSRuntimeContext {
    _source_data: Vec<u8>,
    inner: Context,
}

#[wasm_bindgen]
impl SSRuntimeContext {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<SSRuntimeContext, JsValue> {
        let source_data = data.to_vec();

        let slice_ptr = source_data.as_slice();
        let static_slice: &'static [u8] = unsafe { std::mem::transmute(slice_ptr) };

        match Context::new(static_slice) {
            Ok(ctx) => Ok(SSRuntimeContext {
                _source_data: source_data,
                inner: ctx,
            }),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    pub fn get_initialize_data(&mut self) -> Vec<u8> {
        self.inner.get_initialize_data().to_vec()
    }

    pub fn get_animation_names(&mut self) -> Vec<String> {
        self.inner
            .get_animation_names()
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn setup_animation(&mut self, name: &str) -> Option<Vec<u8>> {
        self.inner.setup_animation(name).map(|s| s.to_vec())
    }

    pub fn get_frame_data(&mut self, frame_no: i32) -> Option<Vec<u8>> {
        self.inner.get_frame_data(frame_no).map(|s| s.to_vec())
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

    pub fn get_skip_frames(&mut self) -> bool {
        self.inner.get_skip_frames()
    }

    pub fn get_loops(&mut self) -> i32 {
        self.inner.get_loops()
    }

    pub fn get_remain_loops(&mut self) -> i32 {
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

    pub fn set_animation_speed(&mut self, speed_rate: f32) {
        self.inner.set_animation_speed(speed_rate);
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.inner.set_animation_section(start_frame, end_frame);
    }

    pub fn update(&mut self, delta: f32) -> i32 {
        let (frame, decimal) = self.inner.update(delta);
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