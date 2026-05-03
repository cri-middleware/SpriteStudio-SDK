#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

#[cfg(all(not(feature = "std"), not(test), not(feature = "web"), feature = "panic-handler"))]
#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    #[cfg(target_arch = "wasm32")]
    ::core::arch::wasm32::unreachable();

    #[cfg(all(not(target_arch = "wasm32"), feature = "libc_alloc"))]
    unsafe {
        libc::abort();
    }

    #[cfg(not(target_arch = "wasm32"))]
    loop {}
}

#[cfg(all(not(feature = "std"), not(test), unix, feature = "panic-handler"))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[cfg(all(not(feature = "std"), not(feature = "web"), target_arch = "wasm32", feature = "wasm_alloc"))]
mod wasm_allocator {
    #[global_allocator]
    static ALLOCATOR: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;
}

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

use alloc::vec::Vec;

#[rustfmt::skip]
/// cbindgen:ignore
mod runtime;
pub mod core;
pub mod error;

pub use crate::core::SsCoordinateSystem;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "web")]
pub mod web;

/// A read-only borrow of an external SSAB byte buffer.
///
/// The pointer/length pair is never observed publicly; it is only constructed
/// inside [`SsabData::load_borrow`] (whose `# Safety` clause owns the validity
/// invariant) and read internally via [`SsabStorage::as_slice`].
pub(crate) struct BorrowedBuffer {
    ptr: ::core::ptr::NonNull<u8>,
    len: usize,
}

// SAFETY: `BorrowedBuffer` represents a borrow of read-only memory whose
// validity is upheld by the caller of `SsabData::load_borrow`. Nothing in this
// crate mutates the pointed-to bytes, so transferring (`Send`) or sharing
// (`Sync`) the handle does not introduce data races as long as the caller's
// contract — that the memory remains valid and unmodified for the duration of
// the borrow, including across threads — is honored.
unsafe impl Send for BorrowedBuffer {}
unsafe impl Sync for BorrowedBuffer {}

pub(crate) enum SsabStorage {
    Owned(Vec<u8>),
    Borrowed(BorrowedBuffer),
}
impl SsabStorage {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            SsabStorage::Owned(vec) => vec.as_slice(),
            // SAFETY: see the safety contract on `BorrowedBuffer` and
            // `SsabData::load_borrow`.
            SsabStorage::Borrowed(b) => unsafe { ::core::slice::from_raw_parts(b.ptr.as_ptr(), b.len) },
        }
    }
}

pub struct AnimationMetadata<'a> {
    pub name_hash: u32,
    pub start_frame: i32,
    pub end_frame: i32,
    pub fps: i32,
    pub canvas_width: f32,
    pub canvas_height: f32,
    pub cached_parts: Vec<crate::core::framedata::CachedPartData<'a>>,
    pub events: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::EventsPerFrame<'a>>>>,
}

pub struct SsabData {
    storage: SsabStorage,
}

// `SsabData` derives `Send + Sync` automatically: `Vec<u8>` is `Send + Sync`
// and `BorrowedBuffer` carries its own `unsafe impl` of those traits, scoped
// to the smallest type that actually needs it.

impl SsabData {
    pub fn load_copy(ssab_data: &[u8]) -> Result<Self, error::SSRuntimeError> {
        let storage = SsabStorage::Owned(ssab_data.to_vec());
        Ok(Self { storage })
    }

    /// # Safety
    ///
    /// This function is unsafe because it creates a slice from a raw pointer and length.
    /// The caller must ensure:
    /// - `ptr` is non-null and points to a valid memory block of at least `len` bytes.
    /// - The memory remains valid and unmodified for the lifetime of the returned
    ///   `SsabData` (and any `SsabResource` derived from it).
    /// - If the resulting `SsabData` is sent to or shared with another thread,
    ///   no other party may mutate the memory for the duration of the borrow.
    ///
    /// Returns `SSRuntimeError::InvalidParameter` if `ptr` is null.
    pub unsafe fn load_borrow(ptr: *const u8, len: usize) -> Result<Self, error::SSRuntimeError> {
        let Some(ptr) = ::core::ptr::NonNull::new(ptr as *mut u8) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        Ok(Self { storage: SsabStorage::Borrowed(BorrowedBuffer { ptr, len }) })
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
///
/// This is a self-referential structure: `resource` holds borrows into `data`'s
/// buffer. The `'static` lifetime stored internally is fictitious — the actual
/// lifetime is bound to `&self`. The `resource()` accessor restores the correct
/// lifetime, so external code never observes the unsound `'static` form.
///
/// Invariants (must hold for soundness):
/// - Fields are private so `data` cannot be mutated or replaced after construction.
/// - Field declaration order ensures `resource` is dropped before `data`.
/// - `SsabData::Owned` keeps its `Vec<u8>` buffer at a stable heap address;
///   `SsabData::Borrowed` requires the caller to keep the memory valid (documented
///   on `SsabData::load_borrow`).
pub struct SsabResource {
    // NOTE: declaration order matters — `resource` borrows from `data`,
    // so it must drop first.
    resource: Option<AnimationResource<'static>>,
    data: SsabData,
}

impl SsabResource {
    pub fn new(data: SsabData) -> alloc::sync::Arc<Self> {
        let res = Self { resource: None, data };
        let mut arc = alloc::sync::Arc::new(res);
        if let Some(mut_res) = alloc::sync::Arc::get_mut(&mut arc) {
            // SAFETY: `mut_res.data` lives as long as `mut_res` (i.e. as long as the
            // Arc allocation). The `AnimationResource` we build holds borrows into
            // that buffer, but we store it as `'static` because Rust cannot express
            // a self-borrow directly. The `resource()` accessor downgrades the lie
            // back to a self-bound lifetime before exposing it.
            let data_slice: &'static [u8] = unsafe { &*(mut_res.data.as_slice() as *const [u8]) };
            if let Ok(resource) = AnimationResource::new(data_slice) {
                mut_res.resource = Some(resource);
            }
        }
        arc
    }

    /// Returns the parsed animation resource, with its lifetime tied to `&self`.
    #[inline]
    pub fn resource(&self) -> Option<&AnimationResource<'_>> {
        self.resource.as_ref()
    }

    /// Returns the underlying SSAB data.
    #[inline]
    pub fn data(&self) -> &SsabData {
        &self.data
    }

    /// Returns a raw pointer to the parsed resource (or null if parsing failed),
    /// detached from `self`'s borrow scope.
    ///
    /// This is intended for FFI/WASM bindings that need to extend the resource
    /// reference into a `'static` context while transferring ownership of the
    /// surrounding `Arc<SsabResource>` to a long-lived holder. Using a raw
    /// pointer breaks the borrow chain so the `Arc` can be moved into its
    /// holder before the pointer is dereferenced.
    ///
    /// Obtaining the pointer is safe; dereferencing it is `unsafe` and the
    /// caller must uphold the obligations below.
    ///
    /// # Caller obligations when dereferencing
    /// - Keep some `Arc<SsabResource>` pointing at this allocation alive for the
    ///   entire duration the pointer (or any borrow derived from it) is in use.
    /// - Do not dereference after the last `Arc` referencing this allocation
    ///   is dropped — doing so is undefined behaviour.
    #[inline]
    pub fn resource_static_ptr(&self) -> *const AnimationResource<'static> {
        match &self.resource {
            Some(r) => r as *const _,
            None => ::core::ptr::null(),
        }
    }
}

impl<'a> AnimationResource<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, error::SSRuntimeError> {
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
                    let fb_parts = ssab.parts();
                    for (i, p) in parts.iter().enumerate() {
                        let parent_index = if let Some(parts_meta) = fb_parts { parts_meta.get(i).parent_index() } else { -1 };
                        cached_parts.push(crate::core::framedata::CachedPartData::new(p, i as i16, parent_index));
                    }
                }
                let name_hash = anim.name_hash();
                let (canvas_width, canvas_height) = if let Some(canvas) = anim.canvas() { (canvas.x2() - canvas.x1(), canvas.y2() - canvas.y1()) } else { (0.0, 0.0) };
                animations.push(AnimationMetadata { name_hash, start_frame: anim.start_frame() as i32, end_frame: anim.end_frame() as i32, fps: anim.fps() as i32, canvas_width, canvas_height, cached_parts, events: anim.events() });
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

struct ContextBuffers<'a> {
    parts_vec: Vec<crate::runtime::framedata::ss::runtime::PartState>,
    cell_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
    parts_color_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    shader_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    vertice_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    deform_vec: Vec<flatbuffers::WIPOffset<crate::runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
    cursors: Vec<usize>,
    collected_event_indices: Vec<i32>,
    world_matrices: Vec<crate::core::SsMatrix4x4>,
    inheritance_matrices: Vec<crate::core::SsMatrix4x4>,
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
            collected_event_indices: Vec::new(),
            world_matrices: Vec::new(),
            inheritance_matrices: Vec::new(),
        }
    }
}

pub struct Context<'a> {
    resource: Option<&'a AnimationResource<'a>>,
    fb: flatbuffers::FlatBufferBuilder<'a>,
    controller: core::playback_controller::PlaybackController,
    animation_idx: i32,
    buffers: ContextBuffers<'a>,
    coordinate_system: crate::core::SsCoordinateSystem,
    origin: crate::core::SsOrigin,
    unit: crate::core::SsUnit,
    world_matrix_calculation_enabled: bool,
    z_order: crate::core::z_order::ZOrderTable,
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context {
            resource: None,
            fb: flatbuffers::FlatBufferBuilder::new(),
            controller: core::playback_controller::PlaybackController::new(),
            animation_idx: -1,
            buffers: ContextBuffers::new(),
            coordinate_system: crate::core::SsCoordinateSystem::default(),
            origin: crate::core::SsOrigin::default(),
            unit: crate::core::SsUnit::default(),
            world_matrix_calculation_enabled: true,
            z_order: crate::core::z_order::ZOrderTable::new(),
        }
    }

    pub fn reset(&mut self) {
        self.resource = None;
        self.controller.reset_setup();
        self.fb.reset();
        self.animation_idx = -1;
        self.buffers.cursors.clear();
        self.coordinate_system = crate::core::SsCoordinateSystem::default();
        self.origin = crate::core::SsOrigin::default();
        self.unit = crate::core::SsUnit::default();
        self.world_matrix_calculation_enabled = true;
        self.z_order = crate::core::z_order::ZOrderTable::new();
    }

    pub fn set_world_matrix_calculation_enabled(&mut self, enabled: bool) {
        self.world_matrix_calculation_enabled = enabled;
    }

    pub fn is_world_matrix_calculation_enabled(&self) -> bool {
        self.world_matrix_calculation_enabled
    }

    pub fn bind_resource(&mut self, resource: &'a AnimationResource<'a>) {
        self.resource = Some(resource);
        self.controller.reset_setup();
        self.fb.reset();
        self.animation_idx = -1;
        self.buffers.cursors.clear();
        // Configuration (coordinate_system, origin, unit) is preserved.
    }

    pub fn get_ssab_data(&self) -> Option<&[u8]> {
        self.resource.map(|r| r.get_ssab_data())
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

        self.buffers.world_matrices.clear();
        self.buffers.world_matrices.resize(num_parts, crate::core::SsMatrix4x4::IDENTITY);
        self.buffers.inheritance_matrices.clear();
        self.buffers.inheritance_matrices.resize(num_parts, crate::core::SsMatrix4x4::IDENTITY);

        self.z_order.setup(&metadata.cached_parts);

        Ok(())
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        self.controller.update(delta_time);
    }

    pub fn get_frame_data(&mut self, frame_no: f32) -> Result<&[u8], error::SSRuntimeError> {
        let Some(resource) = self.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };

        if self.animation_idx < 0 {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }

        let metadata = &resource.animations[self.animation_idx as usize];
        if !((metadata.start_frame as f32) <= frame_no && frame_no <= (metadata.end_frame as f32)) {
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
            &mut self.buffers.cursors,
            self.coordinate_system,
            self.origin,
            self.unit,
            metadata.canvas_width,
            metadata.canvas_height,
        );

        if self.world_matrix_calculation_enabled {
            self.compute_matrices(&metadata.cached_parts);
        }

        self.z_order.update(&self.buffers.parts_vec);

        self.fb.finish_minimal(offset);
        Ok(self.fb.finished_data())
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_matrices(&mut self, cached_parts: &[crate::core::framedata::CachedPartData]) {
        for i in 0..self.buffers.parts_vec.len() {
            let parent_idx = cached_parts[i].parent_index;
            let valid_parent_idx = (parent_idx >= 0 && (parent_idx as usize) < i).then_some(parent_idx as usize);

            let parent_world = valid_parent_idx.map(|p_idx| &self.buffers.inheritance_matrices[p_idx]);
            let parent_size = valid_parent_idx.map(|p_idx| {
                let p_state = &self.buffers.parts_vec[p_idx];
                (p_state.size_x(), p_state.size_y())
            });

            let part_state = &self.buffers.parts_vec[i];
            let (inh_m, draw_m) = crate::core::SsMatrix4x4::compute_world_matrix(part_state, parent_world, parent_size);
            self.buffers.inheritance_matrices[i] = inh_m;
            self.buffers.world_matrices[i] = draw_m;
        }
    }

    // Frame Number getter functions

    pub fn get_frame_no(&self) -> f32 {
        self.controller.get_frame_no()
    }

    pub fn is_end_frame_reached(&self) -> bool {
        self.controller.is_end_frame_reached()
    }

    pub fn get_world_matrices_ptr(&self) -> *const f32 {
        self.buffers.world_matrices.as_ptr() as *const f32
    }

    pub fn get_world_matrices_len(&self) -> usize {
        self.buffers.world_matrices.len() * 16
    }

    pub fn get_z_order_ptr(&self) -> *const i32 {
        self.z_order.as_ptr()
    }

    pub fn get_z_order_len(&self) -> usize {
        self.z_order.len()
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

    pub fn set_coordinate_system(&mut self, coordinate_system: crate::core::SsCoordinateSystem) {
        self.coordinate_system = coordinate_system;
    }

    pub fn get_coordinate_system(&self) -> crate::core::SsCoordinateSystem {
        self.coordinate_system
    }

    pub fn set_origin(&mut self, origin: crate::core::SsOrigin) {
        self.origin = origin;
    }

    pub fn get_origin(&self) -> crate::core::SsOrigin {
        self.origin
    }

    pub fn set_unit(&mut self, unit: crate::core::SsUnit) {
        self.unit = unit;
    }

    pub fn get_unit(&self) -> crate::core::SsUnit {
        self.unit
    }

    // Playback control

    pub fn update(&mut self, delta: f32) -> f32 {
        let res = self.controller.update(delta);

        self.collect_events();

        res
    }

    fn collect_events(&mut self) {
        self.buffers.collected_event_indices.clear();
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
            if frame_no < 0 || frame_no > u16::MAX as i32 {
                continue;
            }
            let target_frame = frame_no as u16;

            // Find events for this frame using binary search
            let mut low = 0;
            let mut high = events_vec.len();

            while low < high {
                let mid = low + (high - low) / 2;
                match events_vec.get(mid).frame_index().cmp(&target_frame) {
                    ::core::cmp::Ordering::Equal => {
                        self.buffers.collected_event_indices.push(mid as i32);
                        break;
                    }
                    ::core::cmp::Ordering::Less => low = mid + 1,
                    ::core::cmp::Ordering::Greater => high = mid,
                }
            }
        }
    }

    pub fn get_passed_event_indices(&self) -> &[i32] {
        &self.buffers.collected_event_indices
    }

    pub fn play(&mut self) {
        self.controller.play();
    }

    pub fn play_with_start_frame(&mut self, frame_no: f32) {
        self.controller.play_with_start_frame(frame_no);
    }

    pub fn pause(&mut self) {
        self.controller.pause();
    }

    pub fn stop(&mut self) {
        self.controller.stop();
    }

    pub fn set_frame_no(&mut self, frame: f32) {
        self.controller.set_frame_no(frame);
    }

    pub fn next_frame(&mut self) {
        self.controller.next_frame();
    }

    pub fn prev_frame(&mut self) {
        self.controller.prev_frame();
    }
}

pub struct EffectContext<'a> {
    pub instance: crate::core::effect::instance::EffectInstance,
    fb: flatbuffers::FlatBufferBuilder<'a>,
}

impl<'a> EffectContext<'a> {
    pub fn new(resource: &'a AnimationResource<'a>, effect_name_hash: u32, seed: u32) -> Result<Self, error::SSRuntimeError> {
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(resource.data) };
        let Some(effects) = ssab.effects() else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };
        let Some(effect) = effects.iter().find(|e| e.name_hash() == effect_name_hash) else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };
        // The effect was located by name_hash, but its data is malformed
        // (e.g. missing required `effect_node` table).
        let Some(instance) = crate::core::effect::instance::EffectInstance::from_ssab(&effect, seed) else {
            return Err(error::SSRuntimeError::MalformedEffect);
        };
        Ok(Self { instance, fb: flatbuffers::FlatBufferBuilder::new() })
    }

    pub fn reset(&mut self) {
        self.instance.reset();
    }

    pub fn get_lifetime(&self) -> i32 {
        self.instance.effect_time_length
    }

    pub fn is_infinite(&self) -> bool {
        self.instance.is_infinite
    }

    pub fn update(&mut self, frame: f32) {
        self.instance.update(frame);
    }

    pub fn get_state(&mut self) -> Result<&[u8], error::SSRuntimeError> {
        let (draw_commands, draw_particles) = self.instance.get_draw_commands();

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
            let frame_data_buf = context.get_frame_data(0.0).expect("Failed to get frame data at frame 0");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts().expect("No parts in frame data");
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 0.0);
            assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }

        // Test frame 5 (interpolating)
        {
            let frame_data_buf = context.get_frame_data(5.0).expect("Failed to get frame data at frame 5");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts().expect("No parts in frame data");
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 16.0);
            assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }

        // Test frame 10
        {
            let frame_data_buf = context.get_frame_data(10.0).expect("Failed to get frame data at frame 10");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts().expect("No parts in frame data");
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 32.0);
            assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }

        // Test frame 11 (not interpolating, no key)
        {
            let frame_data_buf = context.get_frame_data(11.0).expect("Failed to get frame data at frame 11");
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
        let frame_0_buf = context.get_frame_data(0.0).unwrap();
        let frame_0 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_0_buf) };
        let hide_state_0 = frame_0.parts().unwrap().iter().find(|p| p.part_index() == hide_part_idx).unwrap();
        assert!(!hide_state_0.hide());

        // Frame 2: Hidden
        let frame_2_buf = context.get_frame_data(2.0).unwrap();
        let frame_2 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_2_buf) };
        let hide_state_2 = frame_2.parts().unwrap().iter().find(|p| p.part_index() == hide_part_idx).unwrap();
        assert!(hide_state_2.hide());
    }

    #[test]
    fn test_world_matrices() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        // Verify world matrices are computed correctly
        let frame_no = 0.0;
        let _ = context.get_frame_data(frame_no).expect("Failed to get frame data");

        let world_matrices_ptr = context.get_world_matrices_ptr();
        let world_matrices_len = context.get_world_matrices_len();

        // Length should be 16 floats per part
        let num_parts = context.buffers.parts_vec.len();
        assert_eq!(world_matrices_len, num_parts * 16);

        let world_matrices = unsafe { ::core::slice::from_raw_parts(world_matrices_ptr, world_matrices_len) };

        // For simplicity, just test that matrices are non-zero/valid
        assert!(world_matrices[0] != 0.0 || world_matrices[1] != 0.0);
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
            let frame_0_buf = context.get_frame_data(0.0).unwrap();
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
            let frame_10_buf = context.get_frame_data(10.0).unwrap();
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
        assert_eq!(context.get_frame_no(), 0.0);
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

        let events = ctx.get_passed_event_indices();
        assert!(!events.is_empty(), "Should have collected at least one event");

        // Advance further to cover the whole animation
        ctx.update(interval * 100.0);
        let events = ctx.get_passed_event_indices();
        assert!(!events.is_empty(), "Should have collected events");
        assert!(events.iter().any(|&e| e > 0), "Should have collected at least one event with index > 0");
    }

    #[test]
    fn test_ssab_resource_accessors() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let res = SsabResource::new(ssab_data);

        // resource() returns Some for a valid SSAB.
        assert!(res.resource().is_some());

        // data() returns the underlying SSAB bytes.
        assert_eq!(res.data().as_slice().len(), input_buf.len());

        // resource_static_ptr() is non-null and points at the same allocation as resource().
        let ptr = res.resource_static_ptr();
        assert!(!ptr.is_null());
        let safe_ref_addr = res.resource().unwrap() as *const _ as usize;
        assert_eq!(ptr as usize, safe_ref_addr);
    }

    #[test]
    fn test_load_borrow_rejects_null_pointer() {
        // `load_borrow` must not construct an SsabData wrapping a null pointer,
        // because `as_slice` would then call `from_raw_parts(null, len)` (UB for len > 0).
        let result = unsafe { SsabData::load_borrow(::core::ptr::null(), 16) };
        match result {
            Err(error::SSRuntimeError::InvalidParameter) => {}
            _ => panic!("Expected InvalidParameter for null pointer"),
        }
    }

    #[test]
    fn test_ssab_resource_arc_sharing_keeps_buffer_alive() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let res = SsabResource::new(ssab_data);

        // Take a raw pointer, then drop the original Arc handle while a clone keeps
        // the allocation alive. The pointer must remain valid through the clone.
        let ptr = res.resource_static_ptr();
        assert!(!ptr.is_null());
        let clone = res.clone();
        drop(res);

        // Safety: `clone` still owns the allocation.
        let names = unsafe { (*ptr).get_animation_names() };
        assert!(names.contains(&alloc::string::String::from("anime_1")));
        drop(clone);
    }
}
