
/// cbindgen:ignore
mod runtime;
pub mod core;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "web")]
pub mod web;

enum SsabStorage {
    Owned(Vec<u8>),
    Borrowed { ptr: *const u8, len: usize},
}
impl SsabStorage {
     fn as_slice(&self) -> &[u8] {
        unsafe {
            match self {
                SsabStorage::Owned(vec) => vec.as_slice(),
                SsabStorage::Borrowed { ptr, len } => std::slice::from_raw_parts(*ptr, *len),
            }
        }
    }
}

pub struct Context<'a> {
    ssab_data: Option<SsabStorage>,
    fb: flatbuffers::FlatBufferBuilder<'a>,
    controller: core::playback_controller::PlaybackController,
    animation_idx: i32,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context {
            ssab_data: None,
            fb: flatbuffers::FlatBufferBuilder::new(),
            controller: core::playback_controller::PlaybackController::new(),
            animation_idx: -1,
        }
    }

    pub fn reset(&mut self) {
        self.ssab_data = None;
        self.controller.reset_setup();
        self.fb.reset();
        self.animation_idx = -1;
    }

    fn validate_ssab_data(&self, buf: &[u8]) -> Result<bool, std::io::Error> {
        // validate ssab version
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(buf) };
        if ssab.version() != ssab::ssab::ss::format::SsAnimeBinaryArgs::default().version {
            // invalid ssab version
            let msg = "invalid ssab version expected: ".to_string()
                + &ssab::ssab::ss::format::SsAnimeBinaryArgs::default().version.to_string()
                + ", got: "
                + &ssab.version().to_string();
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, msg))
        }
        Ok(true)
    }

    pub fn load_copy(&mut self, ssab_data: &[u8]) -> Result<bool, std::io::Error> {
        self.ssab_data = Some(SsabStorage::Owned(ssab_data.to_vec()));
        let Some(data) = self.ssab_data.as_ref().map(|d| d.as_slice()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "ssab data is empty"));
        };
        self.validate_ssab_data(data)
    }

    pub unsafe fn load_borrow(&mut self, ptr: *const u8, len: usize) -> Result<bool, std::io::Error> {
        self.ssab_data = Some(SsabStorage::Borrowed { ptr, len });
        let Some(data) = self.ssab_data.as_ref().map(|d| d.as_slice()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "ssab data is empty"));
        };
        self.validate_ssab_data(data)
    }

    pub fn get_ssab_data(&self) -> Option<&[u8]> {
        self.ssab_data.as_ref().map(|d| d.as_slice())
    }

    pub fn setup_animation(&mut self, name: &str) -> bool {
        let Some(ssab_buf) = self.ssab_data.as_ref().map(|ssab_data| ssab_data.as_slice()) else {
            return false;
        };
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
        let Some((index, animation)) = ssab.animations().and_then(|animations| {
            animations.iter().enumerate().find(|(_i, en)| { en.key_compare_with_value(name) == std::cmp::Ordering::Equal })
        }) else {
            return false;
        };
        self.controller.setup(animation.start_frame() as i32, animation.end_frame() as i32, animation.fps() as i32);
        self.animation_idx = index as i32;
        true
    }

    pub fn get_frame_data(&mut self, frame_no: i32) -> Option<&[u8]> {
        let Some(ssab_buf) = self.ssab_data.as_ref().map(|ssab_data| ssab_data.as_slice()) else {
            return None;
        };
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
        let Some(animation) =  ssab.animations().map(|animations| animations.get(self.animation_idx as usize)) else {
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

    pub fn is_pausing(&self) -> bool {
        self.controller.is_pausing()
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
    use std::path::PathBuf;
    use ssconverter;
    use super::*;

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
                let config = ssconverter::ConverterConfig::default();
                let _ = ssconverter::convert_sspj_to_ssab_files(sspj.as_path(), &config);
            }
        });
    }

    #[test]
    fn load_parts_of_ssab_files() {
        prepare();
        TARGET_SSPAB_DIRS.iter().for_each(|&dir| {
            let ssab_dir = get_test_base_path().join(dir);
            println!("{}: ", ssab_dir.file_name().unwrap().to_str().unwrap().to_string());
            for entry in ssab_dir.read_dir().unwrap() {
                let f = entry.unwrap().path();
                if !f.is_file() || f.extension().unwrap() != "ssab" { continue; }

                let input_buf =  std::fs::read(&f).unwrap();
                let mut context = Context::new();
                context.load_copy(&input_buf);

                let ssab_buf = context.get_ssab_data().unwrap();
                let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
                println!("  ssab: {}", f.file_name().unwrap().to_str().unwrap());
                println!("  parts");
                ssab.parts().map(|parts| parts.iter().enumerate().for_each(|(idx, part)| {
                    println!("    id: {}, parent_id: {}, type: {}, name: {}", idx, part.parent_index(), part.part_type_type().variant_name().unwrap(), part.name().unwrap());
                }));

                println!();
            }
        });
    }
}
