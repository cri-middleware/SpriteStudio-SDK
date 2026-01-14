
mod runtime;
pub mod core;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "web")]
pub mod web;

pub struct Context {
    ssab: ssab::ssab::SsAnimeBinary<'static>,
    controller: core::playback_controller::PlaybackController,
    fb: flatbuffers::FlatBufferBuilder<'static>,
    animation: Option<ssab::ssab::AnimationData<'static>>,
}

impl Context {
    pub fn new(buf: &[u8]) -> Result<Context, std::io::Error> {
        let buf: &'static [u8] = unsafe { std::mem::transmute(buf) };

        let ssab = unsafe { ssab::ssab::root_as_ss_anime_binary_unchecked(buf) };

        // validate version
        if ssab.version() != ssab::ssab::SsAnimeBinaryArgs::default().version {
            // invalid ssab version
            let msg = "invalid ssab version expected: ".to_string()
                + &ssab::ssab::SsAnimeBinaryArgs::default().version.to_string()
                + ", got: "
                + &ssab.version().to_string();
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, msg))
        }

        Ok(Context {
            controller: core::playback_controller::PlaybackController::new(),
            ssab,
            fb: flatbuffers::FlatBufferBuilder::new(),
            animation: None,
        })
    }

    pub fn setup_animation(&mut self, name: &str) -> bool {
        let Some(animation) = self.ssab.animations().map(|animations| {
            animations.lookup_by_key(name, |en, key| en.key_compare_with_value(key))
        }).flatten() else { return false; };
        self.controller.setup(animation.start_frame() as i32, animation.end_frame() as i32, animation.fps() as i32);
        self.animation = Some(animation);
        true
    }

    pub fn get_frame_data(&mut self, frame_no: i32) -> Option<&[u8]> {
        let Some(animation) = self.animation else {
            return None;
        };
        if !(animation.start_frame() as i32 <= frame_no && frame_no <= animation.end_frame() as i32) {
            // invalid frame_no
            return None;
        }
        self.fb.reset();
        let offset = core::framedata::get_frame_data(&mut self.fb, &animation, frame_no);
        self.fb.finish_minimal(offset);
        Some(self.fb.finished_data())
    }

    // Frame Number getter functions

    pub fn get_frame_no(&self) -> i32 {
        self.controller.get_frame_no()
    }

    pub fn get_frame_no_decimal(&self) -> f32 {
        self.controller.get_frame_no_decimal()
    }

    pub fn is_end_frame_reached(&self) -> bool {
        self.controller.is_end_frame_reached()
    }

    // setting getter functions
    pub fn is_playing(&self) -> bool {
        self.controller.is_playing()
    }

    pub fn get_skip_frames(&mut self) -> bool {
        self.controller.get_skip_frames()
    }

    pub fn get_playback_direction(&self) -> core::playback_controller::PlaybackDirection {
        self.controller.get_playback_direction()
    }

    pub fn get_playback_style(&self) -> core::playback_controller::PlaybackStyle {
        self.controller.get_playback_style()
    }

    pub fn get_loops(&mut self) -> i32 {
        self.controller.get_loops()
    }

    pub fn get_remain_loops(&mut self) -> i32 {
        self.controller.get_remain_loops()
    }

    pub fn get_start_frame(&self) -> i32 {
        self.controller.get_start_frame()
    }

    pub fn get_end_frame(&self) -> i32 {
        self.controller.get_end_frame()
    }

    pub fn get_fps(&self) -> i32 {
        self.controller.get_fps()
    }

    // Playback setup

    pub fn set_skip_frames(&mut self, skip: bool) {
        self.controller.set_skip_frames(skip);
    }

    pub fn set_loop(&mut self, loops: i32) {
        self.controller.set_loops(loops);
    }

    pub fn set_frame_rate(&mut self, fps: i32) {
        self.controller.set_frame_rate(fps);
    }

    pub fn set_playback_direction(&mut self, direction: core::playback_controller::PlaybackDirection, style: core::playback_controller::PlaybackStyle) {
        self.controller.set_playback_direction(direction, style);
    }

    pub fn set_animation_speed(&mut self, speed_rate: f32) {
        self.controller.set_animation_speed(speed_rate);
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.controller.set_animation_section(start_frame, end_frame);
    }

    // Playback control

    pub fn update(&mut self, delta: f32) -> (i32, f32) {
        self.controller.update(delta)
    }

    pub fn play(&mut self) {
        self.controller.play();
    }

    pub fn play_with_start_frame(&mut self, frame_no: i32) {
        self.controller.play_with_start_frame(frame_no);
    }

    pub fn pause(&mut self) {
        self.controller.pause();
    }

    pub fn stop(&mut self) {
        self.controller.stop();
    }

    pub fn set_frame_no(&mut self, frame: i32) {
        self.controller.set_frame_no(frame);
    }

    pub fn next_frame(&mut self) {
        self.controller.next_frame();
    }

    pub fn prev_frame(&mut self) {
        self.controller.prev_frame();
    }

}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::path::PathBuf;
    use ssconverter3;
    use super::*;
    use super::runtime;

    static TARGET_SSPJ_FILES: [&str; 4] = [
        "tests/overall/overall.sspj",
        "tests/ParticleEffect/ParticleEffect.sspj",
        "tests/allAttributeV7/allAttributeV7.sspj",
        "tests/allPartsV7/allPartsV7.sspj",
    ];

    static TARGET_SSPAB_DIRS: [&str; 4] = [
        "tests/overall/overall_ssab",
        "tests/ParticleEffect/ParticleEffect_ssab",
        "tests/allAttributeV7/allAttributeV7_ssab",
        "tests/allPartsV7/allPartsV7_ssab",
    ];


    fn get_test_base_path() -> PathBuf {
        std::env::current_dir()
            .expect("Failed to get the current directory")
            .ancestors()
            .nth(2)
            .expect("Failed to get the base directory")
            .to_path_buf()
    }

    fn prepare() {
        TARGET_SSPAB_DIRS.iter().enumerate().for_each(|(i, &dir)| {
            let ssab_dir = get_test_base_path().join(dir);
            if !ssab_dir.exists() {
                let sspj = get_test_base_path().join(TARGET_SSPJ_FILES[i]);
                let config = ssconverter3::ConverterConfig::default();
                let _ = ssconverter3::convert_sspj_to_ssab_files(sspj.as_path(), &config);
            }
        });
    }
}