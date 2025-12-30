use ssab::ssab::*;
use crate::track_manager::track::{PlaybackDirection, PlaybackStyle};

pub mod runtime;

pub mod track_manager;
mod core;

pub struct Context<'a> {
    pub ssab: SsAnimeBinary<'a>,
    pub track: track_manager::track::Track,
    pub fb: flatbuffers::FlatBufferBuilder<'a>,
    pub animation: Option<AnimationData<'a>>,
}

impl Context<'_> {
    pub fn new(buf: &[u8]) -> Result<Context<'_>, std::io::Error> {
        let r = ssab::ssab::root_as_ss_anime_binary(buf);
        let ssab_buf = match r {
            Ok(ssab_buf) => ssab_buf,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
        };

        // validate version
        if ssab_buf.version() != SsAnimeBinaryArgs::default().version {
            // invalid ssab version
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid ssab version expected: {}, got: {}", SsAnimeBinaryArgs::default().version, ssab_buf.version())))
        }

        Ok(Context {
            track: track_manager::track::Track::new(),
            ssab: ssab_buf,
            fb: flatbuffers::FlatBufferBuilder::new(),
            animation: None,
        })
    }

    pub fn get_initialize_data(&mut self) -> Option<Vec<u8>> {
        self.fb.reset();
        let offset = core::initialize::get_initialize_data(&mut self.fb, &self.ssab);
        self.fb.finish(offset, None);
        Some(self.fb.finished_data().to_vec())
    }

    pub fn get_animation_names(&mut self) -> Option<Vec<String>> {
        self.ssab.animations().map(|animations| {
            animations.iter().map(|animation| animation.name().to_string()).collect::<Vec<String>>()
        })
    }

    pub fn setup_animation(&mut self, name: &str) -> Option<Vec<u8>> {
        self.animation = None;
        let Some(animations) = self.ssab.animations() else {
            return None;
        };
        let Some(animation) = animations.lookup_by_key(name, |en, key| en.key_compare_with_value(key)) else {
            return None;
        };
        self.track.setup(animation.start_frame() as i32, animation.end_frame() as i32, animation.fps() as i32);
        self.animation = Some(animation);

        self.fb.reset();
        let offset = core::preparation::get_animation_preparation_data(&mut self.fb, &animation);
        self.fb.finish(offset, None);
        Some(self.fb.finished_data().to_vec())
    }


    // Track control

    pub fn get_frame_no(&self) -> i32 {
        self.track.get_frame_no()
    }

    pub fn get_frame_no_decimal(&self) -> f32 {
        self.track.get_frame_no_decimal()
    }

    pub fn get_end_frame_reached(&self) -> bool {
        self.track.get_end_frame_reached()
    }

    pub fn track_setup(&mut self, start_frame: i32, end_frame: i32, fps: i32) {
        self.track.setup(start_frame, end_frame, fps);
    }

    pub fn set_skip_frames(&mut self, skip: bool) {
        self.track.set_skip_frames(skip);
    }

    pub fn set_loop(&mut self, loops: i32) {
        self.track.set_loop(loops);
    }

    pub fn set_frame_rate(&mut self, fps: i32) {
        self.track.set_frame_rate(fps);
    }

    pub fn set_playback_direction(&mut self, direction: PlaybackDirection, style: PlaybackStyle) {
        self.track.set_playback_direction(direction, style);
    }

    pub fn set_animation_speed(&mut self, speed_rate: f32) {
        self.track.set_animation_speed(speed_rate);
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32, loops: i32) {
        self.track.set_animation_section(start_frame, end_frame, loops);
    }

    pub fn update(&mut self, delta: f32) -> (i32, f32) {
        self.track.update(delta)
    }

    pub fn play(&mut self) {
        self.track.play();
    }

    pub fn play_with_start_frame(&mut self, frame_no: i32) {
        self.track.play_with_start_frame(frame_no);
    }

    pub fn pause(&mut self) {
        self.track.pause();
    }

    pub fn stop(&mut self) {
        self.track.stop();
    }

    pub fn set_frame(&mut self, frame: i32) {
        self.track.set_frame(frame);
    }

    pub fn next_frame(&mut self) {
        self.track.next_frame();
    }

    pub fn prev_frame(&mut self) {
        self.track.prev_frame();
    }

}
