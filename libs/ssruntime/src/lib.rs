use crate::core::track_manager::{PlaybackDirection, PlaybackStyle};

mod runtime;
pub mod core;

#[cfg(not(target_arch = "wasm32"))]
pub mod ffi;

// #[cfg(target_arch = "wasm32")]
// pub mod wasm;

pub struct Context<'a> {
    ssab: ssab::ssab::SsAnimeBinary<'a>,
    track: core::track_manager::TrackManager,
    fb: flatbuffers::FlatBufferBuilder<'a>,
    animation: Option<ssab::ssab::AnimationData<'a>>,
}

impl<'a> Context<'a> {
    pub fn new(buf: &[u8]) -> Result<Context<'_>, std::io::Error> {
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
            track: core::track_manager::TrackManager::new(),
            ssab,
            fb: flatbuffers::FlatBufferBuilder::new(),
            animation: None,
        })
    }

    pub fn get_initialize_data(&mut self) -> Option<Vec<u8>> {
        self.fb.reset();
        let offset = core::initialize::get_initialize_data(&mut self.fb, &self.ssab);
        self.fb.finish_minimal(offset);
        Some(self.fb.finished_data().to_vec())
    }

    pub fn get_animation_names(&mut self) -> Option<Vec<String>> {
        self.ssab.animations().map(|animations| {
            animations.iter().map(|animation| animation.name().to_string()).collect::<Vec<String>>()
        })
    }

    pub fn setup_animation(&mut self, name: &str) -> Option<Vec<u8>> {
        let animation = self.ssab.animations()?.lookup_by_key(name, |en, key| en.key_compare_with_value(key))?;
        self.track.setup(animation.start_frame() as i32, animation.end_frame() as i32, animation.fps() as i32);
        self.animation = Some(animation);

        self.fb.reset();
        let offset = core::preparation::get_animation_preparation_data(&mut self.fb, &animation);
        self.fb.finish_minimal(offset);
        Some(self.fb.finished_data().to_vec())
    }

    pub fn get_frame_data(&mut self, frame_no: i32) -> Option<Vec<u8>> {
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
        Some(self.fb.finished_data().to_vec())
    }

    // Frame Number getter functions

    pub fn get_frame_no(&self) -> i32 {
        self.track.get_frame_no()
    }

    pub fn get_frame_no_decimal(&self) -> f32 {
        self.track.get_frame_no_decimal()
    }

    pub fn is_end_frame_reached(&self) -> bool {
        self.track.is_end_frame_reached()
    }

    // setting getter functions
    pub fn is_playing(&self) -> bool {
        self.track.is_playing()
    }

    pub fn get_skip_frames(&mut self) -> bool {
        self.track.get_skip_frames()
    }

    pub fn get_playback_direction(&self) -> PlaybackDirection {
        self.track.get_playback_direction()
    }

    pub fn get_playback_style(&self) -> PlaybackStyle {
        self.track.get_playback_style()
    }

    pub fn get_loops(&mut self) -> i32 {
        self.track.get_loops()
    }

    pub fn get_remain_loops(&mut self) -> i32 {
        self.track.get_remain_loops()
    }

    pub fn get_start_frame(&self) -> i32 {
        self.track.get_start_frame()
    }

    pub fn get_end_frame(&self) -> i32 {
        self.track.get_end_frame()
    }

    pub fn get_fps(&self) -> i32 {
        self.track.get_fps()
    }

    // Playback setup

    pub fn set_skip_frames(&mut self, skip: bool) {
        self.track.set_skip_frames(skip);
    }

    pub fn set_loop(&mut self, loops: i32) {
        self.track.set_loops(loops);
    }

    pub fn set_frame_rate(&mut self, fps: i32) {
        self.track.set_frame_rate(fps);
    }

    pub fn set_playback_direction(&mut self, direction: core::track_manager::PlaybackDirection, style: core::track_manager::PlaybackStyle) {
        self.track.set_playback_direction(direction, style);
    }

    pub fn set_animation_speed(&mut self, speed_rate: f32) {
        self.track.set_animation_speed(speed_rate);
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.track.set_animation_section(start_frame, end_frame);
    }

    // Playback control

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

    pub fn set_frame_no(&mut self, frame: i32) {
        self.track.set_frame_no(frame);
    }

    pub fn next_frame(&mut self) {
        self.track.next_frame();
    }

    pub fn prev_frame(&mut self) {
        self.track.prev_frame();
    }

}

#[cfg(test)]
mod tests {
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

    #[test]
    fn test_initialize_data() {
        prepare();
        TARGET_SSPAB_DIRS.iter().for_each(|&dir| {
            let ssab_dir = get_test_base_path().join(dir);
            ssab_dir.read_dir().unwrap().for_each(|entry| {
                let f = entry.unwrap().path();
                let buf =  std::fs::read(f).unwrap();
                let mut context = Context::new(&buf).unwrap();
                let Some(init_buf) = context.get_initialize_data() else {
                    // TODO: error handling
                    return;
                };

                // show .ss*e(.ssce, .ssee, .ssse, .charmap and external instance(animation)) names
                let init = unsafe { runtime::initialize::ss::runtime::root_as_init_data_unchecked(&init_buf) };
                println!("name: {}", init.name().unwrap());
                println!("  cellmaps:");
                init.cellmaps().map(|cellmaps| cellmaps.iter().for_each(|c| println!("    cellmap name: {}", c.name().unwrap())));
                println!("  effects:");
                init.effects().map(|effects| effects.iter().for_each(|effect| println!("    effect name: {}", effect.name().unwrap())));
                println!("  sound_lists:");
                init.sound_lists().map(|sound_lists| sound_lists.iter().for_each(|sound_list| println!("    sound_list name: {}", sound_list.name().unwrap())));
                println!("  font_bitmaps:");
                init.font_bitmaps().map(|font_bitmaps| font_bitmaps.iter().for_each(|font_bitmap| { println!("    font_bitmap name: {}", font_bitmap.name().unwrap()); }));
                println!("  external_instances:");
                init.external_instances().map(|external_instances| external_instances.iter().for_each(|external_instance| { println!("    external_instance name: {}", external_instance); }));
                println!("  parts:");
                init.parts().map(|parts| parts.iter().for_each(|part| { println!("    part name: {}", part.name().unwrap()); }));
                println!();
            })
        })
    }

    #[test]
    fn test_animation_names() {
        prepare();
        TARGET_SSPAB_DIRS.iter().for_each(|&dir| {
            let ssab_dir = get_test_base_path().join(dir);
            ssab_dir.read_dir().unwrap().for_each(|entry| {
                let f = entry.unwrap().path();
                let buf =  std::fs::read(f).unwrap();
                let mut context = Context::new(&buf).unwrap();
                let Some(init_buf) = context.get_initialize_data() else {
                    // TODO: error handling
                    return;
                };
                let init = unsafe { runtime::initialize::ss::runtime::root_as_init_data_unchecked(&init_buf) };
                println!("animations name: {}", init.name().unwrap());
                context.get_animation_names().map(|names| names.iter().for_each(|name| {
                    println!("  animation name: {}", name);
                    let Some(prepare_buf) = context.setup_animation(name) else {
                        // TODO: error handling
                        return;
                    };
                    let prepare = unsafe { runtime::preparation::ss::runtime::root_as_anim_prep_data(&prepare_buf) }.unwrap();
                    println!("    start_frame: {}", prepare.start_frame());
                    println!("    end_frame: {}", prepare.end_frame());
                    println!("    total_frame: {}", prepare.total_frame());
                    println!("    fps: {}", prepare.fps());
                    prepare.canvas().map(|canvas| println!("    canvas x1: {}, y1: {}, x2: {}, y2: {}", canvas.x1(), canvas.y1(), canvas.x2(), canvas.y2()));

                    println!();
                }));
                println!();
            })
        })
    }

}