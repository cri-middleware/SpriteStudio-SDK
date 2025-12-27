use ssab::ssab::*;
use crate::track_manager::track::{PlaybackDirection, PlaybackStyle};

pub mod runtime;

pub mod track_manager;

pub struct Context<'a> {
    pub ssab: SsAnimeBinary<'a>,
    pub track: track_manager::track::Track,
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
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid ssab version"))
        }

        Ok(Context {
            track: track_manager::track::Track::new(),
            ssab: ssab_buf,
        })
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
