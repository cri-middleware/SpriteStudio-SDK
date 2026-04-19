#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), not(test), feature = "panic-handler"))]
#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    #[cfg(feature = "libc_alloc")]
    unsafe {
        libc::abort();
    }
    #[cfg(not(feature = "libc_alloc"))]
    loop {}
}

#[cfg(all(not(feature = "std"), not(test), unix, feature = "panic-handler"))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[cfg(all(not(feature = "std"), feature = "libc_alloc"))]
mod allocator {
    #[cfg(unix)]
    #[link(name = "c")]
    unsafe extern "C" {}

    struct LibcAlloc;

    #[global_allocator]
    static ALLOCATOR: LibcAlloc = LibcAlloc;

    unsafe impl alloc::alloc::GlobalAlloc for LibcAlloc {
        unsafe fn alloc(&self, layout: alloc::alloc::Layout) -> *mut u8 {
            let size = layout.size();
            let align = layout.align();

            // Most libc malloc implementations provide at least 8-byte alignment.
            if align <= 8 {
                return unsafe { libc::malloc(size) as *mut u8 };
            }

            // Over-allocate to ensure we can find an aligned block.
            // We need space for: requested size + alignment + space to store the original pointer.
            let extra = align + core::mem::size_of::<usize>();
            let total_size = match size.checked_add(extra) {
                Some(s) => s,
                None => return core::ptr::null_mut(),
            };

            let raw_ptr = unsafe { libc::malloc(total_size) as *mut u8 };
            if raw_ptr.is_null() {
                return core::ptr::null_mut();
            }

            // Calculate aligned pointer.
            // We must have at least size_of::<usize>() before the aligned_ptr to store raw_ptr.
            let aligned_ptr = ((raw_ptr as usize + core::mem::size_of::<usize>() + align - 1) & !(align - 1)) as *mut u8;

            // Store the original pointer right before the aligned pointer.
            unsafe {
                let original_ptr_ptr = (aligned_ptr as *mut usize).offset(-1);
                *original_ptr_ptr = raw_ptr as usize;
            }

            aligned_ptr
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::alloc::Layout) {
            if ptr.is_null() {
                return;
            }

            let align = layout.align();
            if align <= 8 {
                unsafe { libc::free(ptr as *mut libc::c_void) };
            } else {
                unsafe {
                    let original_ptr_ptr = (ptr as *mut usize).offset(-1);
                    let original_ptr = *original_ptr_ptr as *mut libc::c_void;
                    libc::free(original_ptr);
                }
            }
        }
    }
}

extern crate alloc;
extern crate libm;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

#[rustfmt::skip]
/// cbindgen:ignore
mod runtime;
pub mod core;
pub mod error;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "web")]
pub mod web;

pub enum SsabStorage {
    Owned(Vec<u8>),
    Borrowed { ptr: *const u8, len: usize },
}
impl SsabStorage {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            SsabStorage::Owned(vec) => vec.as_slice(),
            SsabStorage::Borrowed { ptr, len } => unsafe { ::core::slice::from_raw_parts(*ptr, *len) },
        }
    }
}

pub struct AnimationMetadata<'a> {
    pub name_hash: u32,
    pub start_frame: i32,
    pub end_frame: i32,
    pub fps: i32,
    pub cached_parts: Vec<crate::core::framedata::CachedPartData<'a>>,
    pub events: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::EventsPerFrame<'a>>>>,
}

pub struct SsabData {
    storage: SsabStorage,
}

// Safety: SsabData only holds a read-only buffer. For Borrowed variant,
// the caller is responsible for ensuring the memory remains valid and unmodified.
unsafe impl Send for SsabData {}
unsafe impl Sync for SsabData {}

impl SsabData {
    pub fn load_copy(ssab_data: &[u8]) -> Result<Self, error::SSRuntimeError> {
        let storage = SsabStorage::Owned(ssab_data.to_vec());
        Ok(Self { storage })
    }

    /// # Safety
    ///
    /// This function is unsafe because it creates a slice from a raw pointer and length.
    /// The caller must ensure that `ptr` points to a valid memory block of at least `len` bytes
    /// and that the memory remains valid for the lifetime of the returned `SsabData`.
    pub unsafe fn load_borrow(ptr: *const u8, len: usize) -> Result<Self, error::SSRuntimeError> {
        let storage = SsabStorage::Borrowed { ptr, len };
        Ok(Self { storage })
    }

    pub fn as_slice(&self) -> &[u8] {
        self.storage.as_slice()
    }
}

pub struct AnimationResource<'a> {
    data: &'a [u8],
    animations: Vec<AnimationMetadata<'a>>,
}

/// A thread-safe, shared resource containing SSAB data and its parsed metadata.
pub struct SsabResource {
    pub data: SsabData,
    pub resource: Option<AnimationResource<'static>>,
}

impl SsabResource {
    pub fn new(data: SsabData) -> alloc::sync::Arc<Self> {
        let res = Self { data, resource: None };
        let mut arc = alloc::sync::Arc::new(res);
        if let Some(mut_res) = alloc::sync::Arc::get_mut(&mut arc) {
            let data_ptr = mut_res.data.as_slice() as *const [u8];
            if let Ok(resource) = AnimationResource::new(unsafe { &*data_ptr }) {
                mut_res.resource = Some(resource);
            }
        }
        arc
    }
}

impl<'a> AnimationResource<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, error::SSRuntimeError> {
        // Validate minimally to prevent process panic on unchecked read
        if !ssab::ssab::ss::format::ss_anime_binary_buffer_has_identifier(data) {
            return Err(error::SSRuntimeError::InvalidData);
        }

        // Use unchecked for performance, but we still check the version.
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(data) };
        if ssab.version() != ssab::ssab::ss::format::SsAnimeBinaryArgs::default().version {
            return Err(error::SSRuntimeError::InvalidVersion { expected: ssab::ssab::ss::format::SsAnimeBinaryArgs::default().version, got: ssab.version() });
        }

        let mut animations = Vec::new();
        if let Some(fb_animations) = ssab.animations() {
            for anim in fb_animations {
                let mut cached_parts = Vec::new();
                if let Some(parts) = anim.parts_animation_data() {
                    for (i, p) in parts.iter().enumerate() {
                        cached_parts.push(crate::core::framedata::CachedPartData::new(p, i as i16));
                    }
                }
                animations.push(AnimationMetadata { name_hash: anim.name_hash(), start_frame: anim.start_frame() as i32, end_frame: anim.end_frame() as i32, fps: anim.fps() as i32, cached_parts, events: anim.events() });
            }
        }

        Ok(Self { data, animations })
    }

    pub fn get_ssab_data(&self) -> &'a [u8] {
        self.data
    }

    pub fn get_animation_names(&self) -> Vec<alloc::string::String> {
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(self.data) };
        let mut names = Vec::new();
        if let Some(animations) = ssab.animations() {
            for anim in animations {
                names.push(alloc::string::String::from(anim.name()));
            }
        }
        names
    }

    pub fn find_animation_index(&self, name: &str) -> Option<usize> {
        let hash = fnv1a_hash_str(name);
        self.find_animation_index_by_hash(hash)
    }

    pub fn find_animation_index_by_hash(&self, hash: u32) -> Option<usize> {
        self.animations.iter().position(|anim| anim.name_hash == hash)
    }
}

pub fn fnv1a_hash_str(text: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    let prime: u32 = 16777619;

    for b in text.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(prime);
    }
    hash
}

#[derive(Clone)]
pub enum PassedEventData<'a> {
    UserData(ssab::ssab::ss::format::PartAttributeUserData<'a>),
    Signal(ssab::ssab::ss::format::PartAttributeSignal<'a>),
    Audio(ssab::ssab::ss::format::PartAttributeAudio<'a>),
    Instance(ssab::ssab::ss::format::PartAttributeInstance<'a>),
    Effect(ssab::ssab::ss::format::PartAttributeEffect<'a>),
}

#[derive(Clone)]
pub struct PassedEvent<'a> {
    pub frame_no: i32,
    pub part_index: i16,
    pub data: PassedEventData<'a>,
    pub event_index: i32,
}

struct ContextBuffers<'a> {
    parts_vec: Vec<crate::runtime::framedata::ss::runtime::PartState>,
    cell_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
    parts_color_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    shader_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    vertice_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    deform_vec: Vec<flatbuffers::WIPOffset<crate::runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
    instance_states: BTreeMap<u16, core::instance::InstancePartState>,
    cursors: Vec<usize>,
    collected_events: Vec<PassedEvent<'a>>,
}

impl<'a> ContextBuffers<'a> {
    fn new() -> Self {
        Self { parts_vec: Vec::new(), cell_vec: Vec::new(), parts_color_vec: Vec::new(), shader_vec: Vec::new(), vertice_vec: Vec::new(), deform_vec: Vec::new(), instance_states: BTreeMap::new(), cursors: Vec::new(), collected_events: Vec::new() }
    }
}

pub struct Context<'a> {
    effect_instances: BTreeMap<u32, crate::core::effect::instance::EffectInstance>,
    next_effect_seed: u32,
    resource: Option<&'a AnimationResource<'a>>,
    fb: flatbuffers::FlatBufferBuilder<'a>,
    controller: core::playback_controller::PlaybackController,
    animation_idx: i32,
    buffers: ContextBuffers<'a>,
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context { resource: None, effect_instances: BTreeMap::new(), next_effect_seed: 123456, fb: flatbuffers::FlatBufferBuilder::new(), controller: core::playback_controller::PlaybackController::new(), animation_idx: -1, buffers: ContextBuffers::new() }
    }

    pub fn reset(&mut self) {
        self.effect_instances.clear();
        self.next_effect_seed = 123456;
        self.resource = None;
        self.controller.reset_setup();
        self.fb.reset();
        self.animation_idx = -1;
        self.buffers.instance_states.clear();
        self.buffers.cursors.clear();
    }

    pub fn bind_resource(&mut self, resource: &'a AnimationResource<'a>) {
        self.reset();
        self.resource = Some(resource);
    }

    pub fn get_ssab_data(&self) -> Option<&[u8]> {
        self.resource.map(|r| r.get_ssab_data())
    }

    fn update_instance(&mut self, frame_no: i32, delta_time: f32) {
        let Some(resource) = self.resource else {
            return;
        };
        if self.animation_idx < 0 {
            return;
        }

        let metadata = &resource.animations[self.animation_idx as usize];

        // We need to check which parts are instances and update their accumulated time if they are independent.
        // For simplicity in this iteration, we iterate all cached parts.
        // In a more optimized version, we could have a list of instance part indices.
        for (i, cached_part) in metadata.cached_parts.iter().enumerate() {
            if let Some(instance_array) = cached_part.instance_array {
                let part_index = cached_part.part_index as u16;
                // Use the cursor for this part's instance attribute
                let mut cursor = self.buffers.cursors[i * crate::core::framedata::NUM_PART_ATTRIBUTES + crate::core::framedata::IDX_ATTRIBUTEINSTANCE];
                if let Some(attr) = crate::core::interpolation::evaluate_instance(instance_array, frame_no, &mut cursor) {
                    if attr.independent() {
                        let state = self.buffers.instance_states.entry(part_index).or_default();
                        if state.last_update_frame != frame_no {
                            state.accumulated_time += delta_time;
                            state.last_update_frame = frame_no;
                        }
                    } else {
                        // If not independent, we don't necessarily need to keep state here,
                        // but we might want to clear it if it was previously independent.
                        self.buffers.instance_states.remove(&part_index);
                    }
                }
            }
        }
    }

    pub fn setup_animation(&mut self, name: &str) -> Result<(), error::SSRuntimeError> {
        let hash = fnv1a_hash_str(name);
        self.setup_animation_by_hash(hash)
    }

    pub fn setup_animation_by_hash(&mut self, hash: u32) -> Result<(), error::SSRuntimeError> {
        let Some(resource) = self.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };

        let Some(index) = resource.find_animation_index_by_hash(hash) else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };

        let metadata = &resource.animations[index];
        self.controller.setup(metadata.start_frame, metadata.end_frame, metadata.fps);
        self.animation_idx = index as i32;

        let num_parts = metadata.cached_parts.len();

        let mut cell_c = 0;
        let mut color_c = 0;
        let mut shader_c = 0;
        let mut vertex_c = 0;
        let mut deform_c = 0;

        for p in &metadata.cached_parts {
            if p.cell_array.is_some() {
                cell_c += 1;
            }
            if p.part_color_array.is_some() {
                color_c += 1;
            }
            if p.shader_array.is_some() {
                shader_c += 1;
            }
            if p.vertex_array.is_some() {
                vertex_c += 1;
            }
            if p.deform_array.is_some() {
                deform_c += 1;
            }
        }

        self.buffers.parts_vec.clear();
        self.buffers.parts_vec.reserve(num_parts);
        self.buffers.cell_vec.clear();
        self.buffers.cell_vec.reserve(cell_c);
        self.buffers.parts_color_vec.clear();
        self.buffers.parts_color_vec.reserve(color_c);
        self.buffers.shader_vec.clear();
        self.buffers.shader_vec.reserve(shader_c);
        self.buffers.vertice_vec.clear();
        self.buffers.vertice_vec.reserve(vertex_c);
        self.buffers.deform_vec.clear();
        self.buffers.deform_vec.reserve(deform_c);

        self.buffers.cursors.clear();
        self.buffers.cursors.resize(num_parts * crate::core::framedata::NUM_PART_ATTRIBUTES, 0);

        // Automatic effect setup for effect parts
        self.effect_instances.clear();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(resource.data) };
        if let Some(parts) = ssab.parts() {
            for (i, p) in parts.iter().enumerate() {
                if p.part_type_type() == ssab::ssab::ss::format::PartType::PartTypeEffect
                    && let Some(effect_part) = p.part_type_as_part_type_effect()
                {
                    let hash = effect_part.ref_effect_name_hash();
                    if let Some(effects) = ssab.effects()
                        && let Some(effect) = effects.iter().find(|e| e.name_hash() == hash)
                    {
                        // Diversify seeds for each effect instance
                        let seed = self.next_effect_seed;
                        self.next_effect_seed = self.next_effect_seed.wrapping_add(1);

                        self.effect_instances.insert(i as u32, crate::core::effect::instance::EffectInstance::from_ssab(&effect, seed));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        self.controller.update(delta_time);
        let frame_no = self.controller.get_frame_no();
        self.update_instance(frame_no, delta_time);
    }

    pub fn get_frame_data(&mut self, frame_no: i32) -> Result<&[u8], error::SSRuntimeError> {
        let Some(resource) = self.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };

        if self.animation_idx < 0 {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }

        let metadata = &resource.animations[self.animation_idx as usize];
        if !(metadata.start_frame <= frame_no && frame_no <= metadata.end_frame) {
            return Err(error::SSRuntimeError::InvalidParameter);
        }

        self.fb.reset();
        let offset = core::framedata::get_frame_data(
            &mut self.fb,
            &metadata.cached_parts,
            frame_no,
            &mut self.buffers.parts_vec,
            &mut self.buffers.cell_vec,
            &mut self.buffers.parts_color_vec,
            &mut self.buffers.shader_vec,
            &mut self.buffers.vertice_vec,
            &mut self.buffers.deform_vec,
            &self.buffers.instance_states,
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
        let res = self.controller.update(delta);
        let f = self.get_frame_no() as f32 + self.get_frame_no_decimal();
        for effect in self.effect_instances.values_mut() {
            effect.update(f);
        }

        self.collect_events();

        res
    }

    fn collect_events(&mut self) {
        self.buffers.collected_events.clear();
        let Some(resource) = self.resource else {
            return;
        };
        if self.animation_idx < 0 {
            return;
        }
        let metadata = &resource.animations[self.animation_idx as usize];
        let Some(events_vec) = metadata.events else {
            return;
        };

        for &frame_no in self.controller.get_passed_frames() {
            // Find events for this frame using binary search
            let mut low = 0;
            let mut high = events_vec.len();
            let mut found_idx = None;

            while low < high {
                let mid = low + (high - low) / 2;
                let mid_frame = events_vec.get(mid).frame_index() as i32;
                if mid_frame == frame_no {
                    found_idx = Some(mid);
                    break;
                } else if mid_frame < frame_no {
                    low = mid + 1;
                } else {
                    high = mid;
                }
            }

            if let Some(idx) = found_idx {
                let events_per_frame = events_vec.get(idx);

                if let Some(users) = events_per_frame.users() {
                    for (i, user) in users.iter().enumerate() {
                        let val = user.value().unwrap();
                        self.buffers.collected_events.push(PassedEvent { frame_no, part_index: user.part_index() as i16, data: PassedEventData::UserData(val), event_index: i as i32 });
                    }
                }
                if let Some(signals) = events_per_frame.signals() {
                    for (i, signal) in signals.iter().enumerate() {
                        let val = signal.value().unwrap();
                        self.buffers.collected_events.push(PassedEvent { frame_no, part_index: signal.part_index() as i16, data: PassedEventData::Signal(val), event_index: i as i32 });
                    }
                }
                if let Some(audios) = events_per_frame.audios() {
                    for (i, audio) in audios.iter().enumerate() {
                        let val = audio.value().unwrap();
                        self.buffers.collected_events.push(PassedEvent { frame_no, part_index: audio.part_index() as i16, data: PassedEventData::Audio(val), event_index: i as i32 });
                    }
                }
                if let Some(instances) = events_per_frame.instances() {
                    for (i, instance) in instances.iter().enumerate() {
                        let val = instance.value().unwrap();
                        self.buffers.collected_events.push(PassedEvent { frame_no, part_index: instance.part_index() as i16, data: PassedEventData::Instance(val), event_index: i as i32 });
                    }
                }
                if let Some(effects) = events_per_frame.effects() {
                    for (i, effect) in effects.iter().enumerate() {
                        let val = effect.value().unwrap();
                        self.buffers.collected_events.push(PassedEvent { frame_no, part_index: effect.part_index() as i16, data: PassedEventData::Effect(val), event_index: i as i32 });
                    }
                }
            }
        }
    }

    pub fn get_passed_events(&self) -> &[PassedEvent<'a>] {
        &self.buffers.collected_events
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

impl<'a> Context<'a> {
    pub fn get_effect_state(&mut self, part_index: u32) -> Result<&[u8], error::SSRuntimeError> {
        let (draw_commands, draw_particles) = match self.effect_instances.get_mut(&part_index) {
            Some(instance) => instance.get_draw_commands(),
            None => return Err(error::SSRuntimeError::NotLoaded),
        };

        self.fb.reset();

        let mut fbs_emitters = Vec::with_capacity(draw_commands.len());
        for cmd in draw_commands {
            let mut fbs_particles = Vec::with_capacity(cmd.particle_end - cmd.particle_start);
            for p in &draw_particles[cmd.particle_start..cmd.particle_end] {
                fbs_particles.push(crate::runtime::effect_state::ss::runtime::ParticleState::new(p.x, p.y, p.scale_x, p.scale_y, p.rotation, p.direction, p.color.a, p.color.r, p.color.g, p.color.b));
            }
            let particles_offset = self.fb.create_vector(&fbs_particles);

            let mut builder = crate::runtime::effect_state::ss::runtime::EmitterStateBuilder::new(&mut self.fb);
            builder.add_cell_index(cmd.cell_index);
            builder.add_blend_type(cmd.blend_type);
            builder.add_particles(particles_offset);
            fbs_emitters.push(builder.finish());
        }

        let emitters_offset = self.fb.create_vector(&fbs_emitters);

        let mut builder = crate::runtime::effect_state::ss::runtime::EffectStateBuilder::new(&mut self.fb);
        builder.add_emitters(emitters_offset);
        let root = builder.finish();

        self.fb.finish_minimal(root);
        Ok(self.fb.finished_data())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use std::path::PathBuf;

    use super::*;

    static TARGET_SSPJ_FILES: [&str; 4] = ["tests/overall/overall.sspj", "tests/ParticleEffect/ParticleEffect.sspj", "tests/allAttributeV7/allAttributeV7.sspj", "tests/allPartsV7/allPartsV7.sspj"];

    static TARGET_SSPAB_DIRS: [&str; 4] = ["tests/overall/overall_ssab", "tests/ParticleEffect/ParticleEffect_ssab", "tests/allAttributeV7/allAttributeV7_ssab", "tests/allPartsV7/allPartsV7_ssab"];

    fn get_test_base_path() -> PathBuf {
        std::env::current_dir().expect("Failed to get the current directory").ancestors().nth(2).expect("Failed to get the base directory").to_path_buf()
    }

    fn prepare() {
        TARGET_SSPAB_DIRS.iter().enumerate().for_each(|(i, &dir)| {
            let ssab_dir = get_test_base_path().join(dir);
            if !ssab_dir.exists() {
                let sspj = get_test_base_path().join(TARGET_SSPJ_FILES[i]);
                let config = ssconverter::ConverterConfig::default();
                let _ = ssconverter::convert_sspj_to_ssab_files(sspj.as_path(), &config, &|_msg: &str| {});
            }
        });
    }

    #[test]
    fn load_parts_of_ssab_files() {
        prepare();
        TARGET_SSPAB_DIRS.iter().for_each(|&dir| {
            let ssab_dir = get_test_base_path().join(dir);
            println!("{}: ", ssab_dir.file_name().unwrap().to_str().unwrap());
            for entry in ssab_dir.read_dir().unwrap() {
                let f = entry.unwrap().path();
                if !f.is_file() || f.extension().unwrap() != "ssab" {
                    continue;
                }

                let input_buf = std::fs::read(&f).unwrap();
                let ssab_data = SsabData::load_copy(&input_buf).unwrap();
                let resource = AnimationResource::new(ssab_data.as_slice()).unwrap();
                let mut context = Context::new();
                context.bind_resource(&resource);

                let ssab_buf = context.get_ssab_data().unwrap();
                let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
                println!("  ssab: {}", f.file_name().unwrap().to_str().unwrap());
                println!("  parts");
                if let Some(parts) = ssab.parts() {
                    parts.iter().enumerate().for_each(|(idx, part)| {
                        println!("    id: {}, parent_id: {}, type: {}, name: {}", idx, part.parent_index(), part.part_type_type().variant_name().unwrap(), part.name().unwrap());
                    })
                }

                println!();
            }
        });
    }

    #[test]
    fn evaluate_animation_at_frame() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
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

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
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
        assert!(!hide_state_0.hide());

        // Frame 2: Hidden
        let frame_2_buf = context.get_frame_data(2).unwrap();
        let frame_2 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_2_buf) };
        let hide_state_2 = frame_2.parts().unwrap().iter().find(|p| p.part_index() == hide_part_idx).unwrap();
        assert!(hide_state_2.hide());
    }

    #[test]
    fn test_deform_animation() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/allAttributeV7/allAttributeV7_ssab/deform.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read deform.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let ssab_buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };

        // Find a part that has deform animation
        let mut deform_part_idx = -1;
        if let Some(animation) = ssab.animations().and_then(|animes| animes.iter().find(|a| a.key_compare_with_value("anime_1") == ::core::cmp::Ordering::Equal))
            && let Some(parts_anime) = animation.parts_animation_data()
        {
            for (idx, p_anime) in parts_anime.iter().enumerate() {
                if p_anime.deform_array().is_some() {
                    deform_part_idx = idx as i32;
                    break;
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

    #[test]
    fn test_ssab_data_and_resource_separation() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

        // 1. Test SsabData load_copy
        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        assert_eq!(ssab_data.as_slice().len(), input_buf.len());

        // 2. Test AnimationResource creation from SsabData
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to create AnimationResource");

        // 3. Validate context binding
        let mut context = Context::new();
        context.bind_resource(&resource);

        // Test names
        let names = resource.get_animation_names();
        assert!(!names.is_empty());
        assert!(names.contains(&alloc::string::String::from("anime_1")));

        // Setup animation
        context.setup_animation("anime_1").expect("Failed to setup animation");
        assert_eq!(context.get_frame_no(), 0);
    }

    #[test]
    fn test_invalid_ssab_data() {
        // Prepare dummy invalid data
        let mut invalid_buf = vec![0u8; 128];
        invalid_buf[0] = 0xFF; // Just some garbage

        let ssab_data = SsabData::load_copy(&invalid_buf).expect("Failed to load SsabData");

        // Now we use safe verification, so it should return Err(InvalidData) instead of panicking
        let result = AnimationResource::new(ssab_data.as_slice());
        assert_eq!(result.err(), Some(error::SSRuntimeError::InvalidData));
    }

    #[test]
    fn test_automatic_effect_setup() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Effect.ssab");
        let data = std::fs::read(ssab_path).unwrap();
        let resource = AnimationResource::new(&data).unwrap();
        let mut context = Context::new();
        context.bind_resource(&resource);

        // Setup an animation that likely contains effect parts
        let anim_name = resource.get_animation_names().first().cloned().unwrap();
        context.setup_animation(&anim_name).unwrap();

        // Verify that effect instances were created
        assert!(!context.effect_instances.is_empty(), "Effect instances should be automatically created for effect parts");

        // Check the number of effect parts in ssab to match
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(&data) };
        let mut expected_count = 0;
        if let Some(parts) = ssab.parts() {
            for p in parts {
                if p.part_type_type() == ssab::ssab::ss::format::PartType::PartTypeEffect {
                    expected_count += 1;
                }
            }
        }
        assert_eq!(context.effect_instances.len(), expected_count, "Number of effect instances should match PartType::Effect count");

        // Verify next_effect_seed was incremented
        assert!(context.next_effect_seed > 123456, "next_effect_seed should have been incremented");

        // Ensure we can get state for the first effect part
        let first_part_idx = *context.effect_instances.keys().next().unwrap();
        let state_res = context.get_effect_state(first_part_idx);
        assert!(state_res.is_ok(), "Should be able to get effect state for the automatically created instance");
    }

    #[test]
    fn test_event_collection_integration() {
        prepare();
        let mut base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        base.pop();
        base.pop();
        let path = base.join("tests/overall/overall_ssab/UserData.ssab");
        let data = std::fs::read(path).expect("Failed to read test ssab file");

        let ssab_data = SsabData::load_copy(&data).unwrap();
        let resource = AnimationResource::new(ssab_data.as_slice()).unwrap();

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);

        let anim_name = resource.get_animation_names().first().cloned().expect("No animations found in SSAB");
        ctx.setup_animation(&anim_name).unwrap();
        ctx.play();

        // Advance by 15 frames (0 -> 15)
        let interval = 1000.0 / 30.0;
        ctx.update(interval * 15.0);

        let events = ctx.get_passed_events();
        // println!("Collected events at frames: {:?}", events.iter().map(|e| e.frame_no).collect::<Vec<_>>());
        assert!(!events.is_empty(), "Should have collected at least one event");

        // Advance further to cover the whole animation
        ctx.update(interval * 100.0);
        let events = ctx.get_passed_events();
        // println!("All collected events at frames: {:?}", events.iter().map(|e| e.frame_no).collect::<Vec<_>>());

        // Let's check for any event that is not at frame 0 (setup)
        assert!(events.iter().any(|e| e.frame_no > 0), "Should have collected at least one event at a frame > 0");
    }
}
