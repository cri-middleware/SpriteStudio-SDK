#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), not(test)))]
#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate libm;
extern crate alloc;

use alloc::vec::Vec;

/// cbindgen:ignore
mod runtime;
pub mod core;
pub mod error;

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
         match self {
             SsabStorage::Owned(vec) => vec.as_slice(),
             SsabStorage::Borrowed { ptr, len } => unsafe { ::core::slice::from_raw_parts(*ptr, *len) },
         }
    }
}

struct ContextBuffers<'a> {
    parts_vec: Vec<crate::runtime::framedata::ss::runtime::PartState>,
    cell_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
    parts_color_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    shader_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    vertice_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    deform_vec: Vec<flatbuffers::WIPOffset<crate::runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
    cursors: Vec<usize>,
    cached_parts: Vec<crate::core::framedata::CachedPartData<'a>>,
}

impl<'a> ContextBuffers<'a> {
    fn new() -> Self {
        Self {
            parts_vec: Vec::new(),
            cell_vec: Vec::new(),
            parts_color_vec: Vec::new(),
            shader_vec: Vec::new(),
            vertice_vec: Vec::new(),
            deform_vec: Vec::new(),
            cursors: Vec::new(),
            cached_parts: Vec::new(),
        }
    }
}

pub struct Context<'a> {
    ssab_data: Option<SsabStorage>,
    fb: flatbuffers::FlatBufferBuilder<'a>,
    controller: core::playback_controller::PlaybackController,
    animation_idx: i32,
    buffers: ContextBuffers<'a>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context {
            ssab_data: None,
            fb: flatbuffers::FlatBufferBuilder::new(),
            controller: core::playback_controller::PlaybackController::new(),
            animation_idx: -1,
            buffers: ContextBuffers::new(),
        }
    }

    pub fn reset(&mut self) {
        self.ssab_data = None;
        self.controller.reset_setup();
        self.fb.reset();
        self.animation_idx = -1;
        self.buffers.cached_parts.clear();
        self.buffers.cursors.clear();
    }

    fn validate_ssab_data(&self, buf: &[u8]) -> Result<(), error::SSRuntimeError> {
        // validate ssab version
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(buf) };
        if ssab.version() != ssab::ssab::ss::format::SsAnimeBinaryArgs::default().version {
            // invalid ssab version
            return Err(error::SSRuntimeError::InvalidVersion {
                expected: ssab::ssab::ss::format::SsAnimeBinaryArgs::default().version,
                got: ssab.version(),
            });
        }
        Ok(())
    }

    pub fn load_copy(&mut self, ssab_data: &[u8]) -> Result<(), error::SSRuntimeError> {
        self.reset();
        self.ssab_data = Some(SsabStorage::Owned(ssab_data.to_vec()));
        let Some(data) = self.ssab_data.as_ref().map(|d| d.as_slice()) else {
            return Err(error::SSRuntimeError::EmptyData);
        };
        self.validate_ssab_data(data)
    }

    pub unsafe fn load_borrow(&mut self, ptr: *const u8, len: usize) -> Result<(), error::SSRuntimeError> {
        self.reset();
        self.ssab_data = Some(SsabStorage::Borrowed { ptr, len });
        let Some(data) = self.ssab_data.as_ref().map(|d| d.as_slice()) else {
            return Err(error::SSRuntimeError::EmptyData);
        };
        self.validate_ssab_data(data)
    }

    pub fn get_ssab_data(&self) -> Option<&[u8]> {
        self.ssab_data.as_ref().map(|d| d.as_slice())
    }

    pub fn setup_animation(&mut self, name: &str) -> Result<(), error::SSRuntimeError> {
        let Some(ssab_buf) = self.ssab_data.as_ref().map(|ssab_data| ssab_data.as_slice()) else {
            return Err(error::SSRuntimeError::NotLoaded);
        };
        // Extend the lifetime of ssab_buf to 'a so we can cache the flatbuffer objects
        // This is safe because ssab_data is owned by Context and we clear the cache on load/reset.
        let ssab_buf: &'a [u8] = unsafe { ::core::mem::transmute(ssab_buf) };
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
        let Some((index, animation)) = ssab.animations().and_then(|animations| {
            animations.iter().enumerate().find(|(_i, en)| { en.key_compare_with_value(name) == ::core::cmp::Ordering::Equal })
        }) else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };
        self.controller.setup(animation.start_frame() as i32, animation.end_frame() as i32, animation.fps() as i32);
        self.animation_idx = index as i32;

        let num_parts = animation.parts_animation_data().map(|p| p.len()).unwrap_or(0);

        let mut cell_c = 0;
        let mut color_c = 0;
        let mut shader_c = 0;
        let mut vertex_c = 0;
        let mut deform_c = 0;

        self.buffers.cached_parts.clear();
        self.buffers.cached_parts.reserve(num_parts);

        if let Some(parts) = animation.parts_animation_data() {
            for (i, p) in parts.iter().enumerate() {
                if p.cell_array().is_some() { cell_c += 1; }
                if p.part_color_array().is_some() { color_c += 1; }
                if p.shader_array().is_some() { shader_c += 1; }
                if p.vertex_array().is_some() { vertex_c += 1; }
                if p.deform_array().is_some() { deform_c += 1; }
                
                self.buffers.cached_parts.push(crate::core::framedata::CachedPartData::new(p, i as i16));
            }
        }

        self.buffers.parts_vec.reserve(num_parts);
        self.buffers.cell_vec.reserve(cell_c);
        self.buffers.parts_color_vec.reserve(color_c);
        self.buffers.shader_vec.reserve(shader_c);
        self.buffers.vertice_vec.reserve(vertex_c);
        self.buffers.deform_vec.reserve(deform_c);

        self.buffers.cursors.clear();
        self.buffers.cursors.resize(num_parts * crate::core::framedata::NUM_PART_ATTRIBUTES, 0);

        Ok(())
    }

    pub fn get_frame_data(&mut self, frame_no: i32) -> Result<&[u8], error::SSRuntimeError> {
        let Some(ssab_buf) = self.ssab_data.as_ref().map(|ssab_data| ssab_data.as_slice()) else {
            return Err(error::SSRuntimeError::NotLoaded);
        };
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
        
        if self.animation_idx < 0 {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }

        let Some(animation) =  ssab.animations().map(|animations| animations.get(self.animation_idx as usize)) else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };
        if !(animation.start_frame() as i32 <= frame_no && frame_no <= animation.end_frame() as i32) {
            // invalid frame_no
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        self.fb.reset();
        let offset = core::framedata::get_frame_data(
            &mut self.fb,
            &self.buffers.cached_parts,
            frame_no,
            &mut self.buffers.parts_vec,
            &mut self.buffers.cell_vec,
            &mut self.buffers.parts_color_vec,
            &mut self.buffers.shader_vec,
            &mut self.buffers.vertice_vec,
            &mut self.buffers.deform_vec,
            &mut self.buffers.cursors,
        );
        self.fb.finish_minimal(offset);
        Ok(self.fb.finished_data())
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

    pub fn get_skip_frames(&self) -> bool {
        self.controller.get_skip_frames()
    }

    pub fn get_playback_direction(&self) -> core::playback_controller::PlaybackDirection {
        self.controller.get_playback_direction()
    }

    pub fn get_playback_style(&self) -> core::playback_controller::PlaybackStyle {
        self.controller.get_playback_style()
    }

    pub fn get_loops(&self) -> i32 {
        self.controller.get_loops()
    }

    pub fn get_remain_loops(&self) -> i32 {
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
                let _ = ssconverter::convert_sspj_to_ssab_files(sspj.as_path(), &config, &|_msg: &str| { });
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
                context.load_copy(&input_buf).unwrap();

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

    #[test]
    fn evaluate_animation_at_frame() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");
        
        let mut context = Context::new();
        context.load_copy(&input_buf).expect("Failed to load ssab");
        context.setup_animation("anime_1").expect("Failed to setup animation");

        // Find part index for "POS_XYZ"
        let pos_xyz_idx;
        {
            let ssab_buf = context.get_ssab_data().unwrap();
            let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
            let parts_meta = ssab.parts().expect("No parts in ssab");
            pos_xyz_idx = parts_meta.iter().position(|p| p.name() == Some("POS_XYZ")).expect("Part POS_XYZ not found") as i16;
        }

        // Test frame 0
        {
            let frame_data_buf = context.get_frame_data(0).expect("Failed to get frame data at frame 0");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts().expect("No parts in frame data");
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 0.0);
            assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }

        // Test frame 5 (interpolating)
        {
            let frame_data_buf = context.get_frame_data(5).expect("Failed to get frame data at frame 5");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts().expect("No parts in frame data");
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 16.0);
            assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }

        // Test frame 10
        {
            let frame_data_buf = context.get_frame_data(10).expect("Failed to get frame data at frame 10");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts().expect("No parts in frame data");
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 32.0);
            assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }

        // Test frame 11 (not interpolating, no key)
        {
            let frame_data_buf = context.get_frame_data(11).expect("Failed to get frame data at frame 11");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts().expect("No parts in frame data");
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 32.0);
            // Should NOT contain AttributePosX update flag because it's not a keyframe and no interpolation
            assert!(!part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }
    }

    #[test]
    fn test_part_hierarchy_and_visibility() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");
        
        let mut context = Context::new();
        context.load_copy(&input_buf).expect("Failed to load ssab");
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let ssab_buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
        let parts_meta = ssab.parts().expect("No parts in ssab");
        
        // Hierarchy check
        let scl_xy_idx = parts_meta.iter().position(|p| p.name() == Some("SCL_XY")).expect("Part SCL_XY not found");
        let scl_xy_part = parts_meta.get(scl_xy_idx);
        let parent_idx = scl_xy_part.parent_index();
        assert!(parent_idx >= 0);
        let parent_part = parts_meta.get(parent_idx as usize);
        assert_eq!(parent_part.name(), Some("LSCL_XY"));

        // Visibility (Hide flag) check
        let hide_part_idx = parts_meta.iter().position(|p| p.name() == Some("HIDE")).expect("Part HIDE not found") as i16;
        
        // Frame 0: Not hidden
        let frame_0_buf = context.get_frame_data(0).unwrap();
        let frame_0 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_0_buf) };
        let hide_state_0 = frame_0.parts().unwrap().iter().find(|p| p.part_index() == hide_part_idx).unwrap();
        assert_eq!(hide_state_0.hide(), false);

        // Frame 2: Hidden
        let frame_2_buf = context.get_frame_data(2).unwrap();
        let frame_2 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_2_buf) };
        let hide_state_2 = frame_2.parts().unwrap().iter().find(|p| p.part_index() == hide_part_idx).unwrap();
        assert_eq!(hide_state_2.hide(), true);
    }

    #[test]
    fn test_deform_animation() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/allAttributeV7/allAttributeV7_ssab/deform.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read deform.ssab");
        
        let mut context = Context::new();
        context.load_copy(&input_buf).expect("Failed to load ssab");
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let ssab_buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
        
        // Find a part that has deform animation
        let mut deform_part_idx = -1;
        if let Some(animation) = ssab.animations().and_then(|animes| animes.iter().find(|a| a.key_compare_with_value("anime_1") == ::core::cmp::Ordering::Equal)) {
            if let Some(parts_anime) = animation.parts_animation_data() {
                for (idx, p_anime) in parts_anime.iter().enumerate() {
                    if p_anime.deform_array().is_some() {
                        deform_part_idx = idx as i32;
                        break;
                    }
                }
            }
        }
        assert!(deform_part_idx >= 0, "No part with deform animation found in anime_1");

        // Test frame 0
        {
            let frame_0_buf = context.get_frame_data(0).unwrap();
            let frame_0 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_0_buf) };
            let parts = frame_0.parts().unwrap();
            let deform_state = parts.iter().find(|p| p.part_index() == deform_part_idx as i16).expect("Deform part not found in frame 0");
            
            if deform_state.deform() >= 0 {
                let deforms = frame_0.deforms().unwrap();
                let deform_attr = deforms.get(deform_state.deform() as usize);
                if let Some(coords) = deform_attr.coordinate() {
                    println!("Frame 0 Deform found with {} coordinates", coords.len());
                }
            }
        }

        // Test frame 10: Interpolated deform
        {
            let frame_10_buf = context.get_frame_data(10).unwrap();
            let frame_10 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_10_buf) };
            let parts = frame_10.parts().unwrap();
            let deform_state = parts.iter().find(|p| p.part_index() == deform_part_idx as i16).expect("Deform part not found in frame 10");
            
            println!("Frame 10 Deform index: {}", deform_state.deform());
            if deform_state.deform() >= 0 {
                let deforms = frame_10.deforms().unwrap();
                let deform_attr = deforms.get(deform_state.deform() as usize);
                println!("Frame 10 Coordinates: {:?}", deform_attr.coordinate().map(|c| c.len()));
                println!("Frame 10 Vertex Indices: {:?}", deform_attr.index().map(|i| i.len()));
            }
        }
    }
}
