#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

// Only one `#[global_allocator]` may exist, so these no_std allocator features are
// mutually exclusive; feature unification can enable two unintentionally, so emit a
// readable error instead of the compiler's cryptic "multiple global allocators".
#[cfg(all(not(feature = "std"), feature = "custom_alloc", feature = "libc_alloc"))]
compile_error!("features `custom_alloc` and `libc_alloc` are mutually exclusive: enable only one #[global_allocator]");
#[cfg(all(not(feature = "std"), feature = "custom_alloc", feature = "emscripten_alloc"))]
compile_error!("features `custom_alloc` and `emscripten_alloc` are mutually exclusive: enable only one #[global_allocator]");
#[cfg(all(not(feature = "std"), target_arch = "wasm32", feature = "custom_alloc", feature = "wasm_alloc"))]
compile_error!("features `custom_alloc` and `wasm_alloc` are mutually exclusive: enable only one #[global_allocator]");

#[cfg(all(not(feature = "std"), not(test), not(feature = "web"), feature = "panic-handler"))]
#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    #[cfg(target_arch = "wasm32")]
    ::core::arch::wasm32::unreachable();

    #[cfg(all(not(target_arch = "wasm32"), feature = "libc_alloc"))]
    unsafe {
        libc::abort();
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "custom_alloc"))]
    unsafe {
        unsafe extern "C" {
            fn ss_custom_abort() -> !;
        }
        ss_custom_abort();
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "libc_alloc"), not(feature = "custom_alloc")))]
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

#[cfg(all(not(feature = "std"), feature = "emscripten_alloc"))]
mod emscripten_allocator {
    unsafe extern "C" {
        #[link_name = "malloc"]
        fn em_malloc(size: usize) -> *mut u8;
        #[link_name = "free"]
        fn em_free(ptr: *mut core::ffi::c_void);
    }

    struct EmscriptenAlloc;

    #[global_allocator]
    static ALLOCATOR: EmscriptenAlloc = EmscriptenAlloc;

    unsafe impl alloc::alloc::GlobalAlloc for EmscriptenAlloc {
        unsafe fn alloc(&self, layout: alloc::alloc::Layout) -> *mut u8 {
            let size = layout.size();
            let align = layout.align();

            if align <= 8 {
                return unsafe { em_malloc(size) };
            }

            let extra = align + core::mem::size_of::<usize>();
            let total_size = match size.checked_add(extra) {
                Some(s) => s,
                None => return core::ptr::null_mut(),
            };

            let raw_ptr = unsafe { em_malloc(total_size) };
            if raw_ptr.is_null() {
                return core::ptr::null_mut();
            }

            let aligned_ptr = ((raw_ptr as usize + core::mem::size_of::<usize>() + align - 1) & !(align - 1)) as *mut u8;

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
                unsafe { em_free(ptr as *mut core::ffi::c_void) };
            } else {
                unsafe {
                    let original_ptr_ptr = (ptr as *mut usize).offset(-1);
                    let original_ptr = *original_ptr_ptr as *mut core::ffi::c_void;
                    em_free(original_ptr);
                }
            }
        }
    }
}

/// Custom global allocator that forwards every allocation to three `extern "C"`
/// symbols the linking application must provide (e.g. a game engine allocator):
///
/// - `ss_custom_alloc(size, align)`: return `size` bytes aligned to `align`
///   (always a power of two) or null on failure. Unlike the `libc_alloc`
///   allocator, the runtime does **not** fix up alignment itself, so the host
///   must honor `align` (use `posix_memalign` / `_aligned_malloc` / the engine's
///   aligned allocator — plain `malloc` only guarantees `max_align_t`).
/// - `ss_custom_free(ptr, size, align)`: free a block returned by
///   `ss_custom_alloc`, given the same `size` and `align`. Allocators that track
///   the size internally (`malloc`/`free`, `FMemory::Free`, ...) may ignore them.
/// - `ss_custom_abort()`: see the `#[panic_handler]` above; must not return.
#[cfg(all(not(feature = "std"), feature = "custom_alloc"))]
mod custom_allocator {
    unsafe extern "C" {
        fn ss_custom_alloc(size: usize, align: usize) -> *mut u8;
        fn ss_custom_free(ptr: *mut core::ffi::c_void, size: usize, align: usize);
    }

    struct CustomAlloc;

    #[global_allocator]
    static ALLOCATOR: CustomAlloc = CustomAlloc;

    unsafe impl alloc::alloc::GlobalAlloc for CustomAlloc {
        unsafe fn alloc(&self, layout: alloc::alloc::Layout) -> *mut u8 {
            unsafe { ss_custom_alloc(layout.size(), layout.align()) }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::alloc::Layout) {
            unsafe { ss_custom_free(ptr as *mut core::ffi::c_void, layout.size(), layout.align()) }
        }
    }
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

pub mod types;

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
    pub initial_events: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::InitialEvents<'a>>>>,
    pub labels: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::Label<'a>>>>,
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
        let fb_animations = ssab.animations();

        for anim in fb_animations {
            let mut cached_parts = Vec::new();
            let parts = anim.parts_animation_data();

            let fb_parts = ssab.parts();
            for (i, p) in parts.iter().enumerate() {
                let part_meta = fb_parts.get(i);
                let parent_index = part_meta.parent_index();
                let blend_type = part_meta.blend_type();
                let part_type_disc = part_meta.part_type_type();
                let mask_influence = part_meta.mask_influence();
                // PartTypeMask is always a pure mask; drawing parts use the unified draw_as_mask flag.
                let is_mask_contributor = part_type_disc == ssab::ssab::ss::format::PartType::PartTypeMask || part_meta.draw_as_mask();
                let resolved_cells = crate::core::framedata::resolve_cell_array_meta(&ssab, p.cell_array());
                let shape_kind = part_meta.part_type_as_part_type_shape().and_then(|s| match s.shape_type() {
                    ssab::ssab::ss::format::ShapeType::Rectangle => Some(crate::core::geometry::ShapeType::Rectangle),
                    ssab::ssab::ss::format::ShapeType::Triangle => Some(crate::core::geometry::ShapeType::Triangle),
                    ssab::ssab::ss::format::ShapeType::Star => Some(crate::core::geometry::ShapeType::Star),
                    ssab::ssab::ss::format::ShapeType::Arrow => Some(crate::core::geometry::ShapeType::Arrow),
                    _ => None,
                });
                let mesh_binding = crate::core::framedata::build_mesh_binding_cache(part_meta.mesh_binding());
                let mesh_cell = if part_type_disc == ssab::ssab::ss::format::PartType::PartTypeMesh { crate::core::framedata::build_mesh_cell_data(&ssab, p.cell_array()) } else { None };
                cached_parts.push(crate::core::framedata::CachedPartData::new(p, i as i16, parent_index, resolved_cells, shape_kind, mesh_binding, mesh_cell, blend_type, part_type_disc, mask_influence, is_mask_contributor));
            }

            let name_hash = anim.name_hash();
            let canvas = anim.canvas();
            let (canvas_width, canvas_height) = (canvas.x2() - canvas.x1(), canvas.y2() - canvas.y1());
            animations.push(AnimationMetadata {
                name_hash,
                start_frame: anim.start_frame() as i32,
                end_frame: anim.end_frame() as i32,
                fps: anim.fps() as i32,
                canvas_width,
                canvas_height,
                cached_parts,
                events: anim.events(),
                initial_events: anim.initial_events(),
                labels: anim.labels(),
            });
        }

        Ok(Self { data, animations })
    }

    pub fn get_ssab_data(&self) -> &'a [u8] {
        self.data
    }

    pub fn get_animation_names(&self) -> Vec<alloc::string::String> {
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(self.data) };
        let mut names = Vec::new();
        let animations = ssab.animations();

        for anim in animations {
            names.push(alloc::string::String::from(anim.name()));
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

/// Per-layer pose buffers — the immediate output of `extract_local_pose`,
/// before any cross-layer composition. SRT / Color / Cell ID / Vertex /
/// Deform / Cursor live here, as do the pose-derived per-part buffers
/// (`cell_meta_per_part`, `cell_texture_hashes`, `local_vertices`) that
/// are written in lockstep with the pose evaluation.
///
/// Phase 3 will introduce a second `LayerState` per instance and a
/// `mix_poses()` stage between Phase 1 and Phase 2.
struct LayerState<'a> {
    parts_vec: Vec<crate::runtime::framedata::ss::runtime::PartState>,
    cell_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
    parts_color_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    shader_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    vertex_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    deform_vec: Vec<flatbuffers::WIPOffset<crate::runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
    cursors: Vec<usize>,
    cell_meta_per_part: Vec<f32>,
    cell_texture_hashes: Vec<u32>,
    local_vertices: Vec<f32>,
    /// Per-part mesh deform retained from Phase 1 for the Phase 2 skinning pass.
    mesh_deform: Vec<crate::core::framedata::PartDeformFrame>,
    raw_vertex_deform: Vec<crate::core::framedata::RawVertexDeform>,
}

impl<'a> LayerState<'a> {
    fn new() -> Self {
        Self {
            parts_vec: Vec::new(),
            cell_vec: Vec::new(),
            parts_color_vec: Vec::new(),
            shader_vec: Vec::new(),
            vertex_vec: Vec::new(),
            deform_vec: Vec::new(),
            cursors: Vec::new(),
            cell_meta_per_part: Vec::new(),
            cell_texture_hashes: Vec::new(),
            local_vertices: Vec::new(),
            mesh_deform: Vec::new(),
            raw_vertex_deform: Vec::new(),
        }
    }

    fn resize_for_parts(&mut self, num_parts: usize, cell_c: usize, color_c: usize, shader_c: usize, vertex_c: usize, deform_c: usize) {
        self.parts_vec.clear();
        self.parts_vec.reserve(num_parts);
        self.cell_vec.clear();
        self.cell_vec.reserve(cell_c);
        self.parts_color_vec.clear();
        self.parts_color_vec.reserve(color_c);
        self.shader_vec.clear();
        self.shader_vec.reserve(shader_c);
        self.vertex_vec.clear();
        self.vertex_vec.reserve(vertex_c);
        self.deform_vec.clear();
        self.deform_vec.reserve(deform_c);

        self.cursors.clear();
        self.cursors.resize(num_parts * crate::core::framedata::NUM_PART_ATTRIBUTES, 0);
        self.cell_meta_per_part.clear();
        self.cell_meta_per_part.resize(num_parts * 6, 0.0);
        self.cell_texture_hashes.clear();
        self.cell_texture_hashes.resize(num_parts, 0);
        self.local_vertices.clear();
        self.local_vertices.resize(num_parts * crate::core::framedata::LOCAL_VERTICES_STRIDE, 0.0);
        self.raw_vertex_deform.clear();
        self.raw_vertex_deform.resize(num_parts, crate::core::framedata::RawVertexDeform::default());
    }
}

/// Per-instance global buffers — the output of `resolve_global_state`.
/// World matrices, UVs, and shape vertices are composited from the
/// (possibly Phase 3-blended) `LayerState` pose; they are not per-layer.
struct GlobalBuffers {
    world_matrices: Vec<crate::core::SsMatrix4x4>,
    inheritance_matrices: Vec<crate::core::SsMatrix4x4>,
    local_uvs: Vec<f32>,
    shape_vertices: Vec<f32>,
    shape_vertex_box_coords: Vec<f32>,
    shape_vertex_counts: Vec<i32>,
    /// Skinned mesh world-space vertex positions, concatenated across all parts.
    /// Use `mesh_vertex_offsets` to slice per-part ranges:
    ///   part `p` occupies `mesh_vertex_offsets[p]..mesh_vertex_offsets[p + 1]`.
    /// Non-mesh parts contribute an empty range.
    mesh_vertices_x: Vec<f32>,
    mesh_vertices_y: Vec<f32>,
    /// CSR-style prefix sum, length = `num_parts + 1`.
    mesh_vertex_offsets: Vec<u32>,
    /// Static mesh UV (interleaved `[u, v, ...]`, texture-pixel space) and
    /// triangle indices, resolved once from each mesh part's frame-0 Cell.
    /// `mesh_uvs` is sliced per-part by `mesh_vertex_offsets`; `mesh_indices`
    /// by `mesh_index_offsets` (CSR, length `num_parts + 1`).
    mesh_uvs: Vec<f32>,
    mesh_indices: Vec<i32>,
    mesh_index_offsets: Vec<u32>,
}

impl GlobalBuffers {
    fn new() -> Self {
        Self {
            world_matrices: Vec::new(),
            inheritance_matrices: Vec::new(),
            local_uvs: Vec::new(),
            shape_vertices: Vec::new(),
            shape_vertex_box_coords: Vec::new(),
            shape_vertex_counts: Vec::new(),
            mesh_vertices_x: Vec::new(),
            mesh_vertices_y: Vec::new(),
            mesh_vertex_offsets: Vec::new(),
            mesh_uvs: Vec::new(),
            mesh_indices: Vec::new(),
            mesh_index_offsets: Vec::new(),
        }
    }
}

/// Static / shareable evaluation context. The resource borrow and the
/// FlatBufferBuilder workspace live here because they are candidates for
/// inter-instance sharing in Phase 3 (group instancing: one resource +
/// one fb workspace driving N lightweight instances serially).
struct EvaluationContext<'a> {
    resource: Option<&'a AnimationResource<'a>>,
    fb: flatbuffers::FlatBufferBuilder<'a>,

    // Workspace buffers to avoid per-frame allocations
    draw_order_workspace: Vec<u16>,
    batches_workspace: Vec<crate::runtime::framedata::ss::runtime::DrawBatch>,
    mask_indices_workspace: Vec<u16>,
    deform_interpolation_start_workspace: Vec<f32>,
    deform_interpolation_end_workspace: Vec<f32>,
    deform_interpolation_out_workspace: Vec<f32>,
    deform_result_coordinate_workspace: Vec<crate::runtime::framedata::ss::runtime::Vec2>,
    deform_result_index_workspace: Vec<i16>,
    inheritance_alphas_workspace: Vec<f32>,
}

impl<'a> EvaluationContext<'a> {
    fn new() -> Self {
        Self {
            resource: None,
            fb: flatbuffers::FlatBufferBuilder::new(),
            draw_order_workspace: Vec::new(),
            batches_workspace: Vec::new(),
            mask_indices_workspace: Vec::new(),
            deform_interpolation_start_workspace: Vec::new(),
            deform_interpolation_end_workspace: Vec::new(),
            deform_interpolation_out_workspace: Vec::new(),
            deform_result_coordinate_workspace: Vec::new(),
            deform_result_index_workspace: Vec::new(),
            inheritance_alphas_workspace: Vec::new(),
        }
    }
}

/// Sentinel `animation_idx` for "empty" sources that contribute a setup-pose
/// (zero translations / unit scales / unit alpha) snapshot to the mix. Treated
/// as active when `weight > 0.0`; carries no controller, animation, or events.
const EMPTY_ANIMATION_IDX: i32 = -2;

struct SourceState {
    animation_idx: i32,
    controller: core::playback_controller::PlaybackController,
    weight: f32,
    /// Minimum weight at which this source's events should fire (only meaningful
    /// for non-primary sources). Default `1.0` preserves the legacy "Primary
    /// fires events; secondaries never do" semantics — secondaries fire when
    /// the caller opts-in by lowering the threshold to <= their current weight.
    event_threshold: f32,
}

impl SourceState {
    fn new() -> Self {
        Self { animation_idx: -1, controller: core::playback_controller::PlaybackController::new(), weight: 1.0, event_threshold: 1.0 }
    }

    fn is_active(&self) -> bool {
        self.animation_idx != -1 && self.weight > 0.0
    }

    /// True for slots that carry a real animation (i.e. not inactive `-1` and
    /// not the empty-source sentinel `-2`). Use this whenever a method needs
    /// a valid `metadata`/`controller`-backed source.
    fn has_real_animation(&self) -> bool {
        self.animation_idx >= 0
    }
}

/// Auto-crossfade timer state. When set, [`Context::update`] advances the
/// timer using the same `delta` value that drives the playback controllers,
/// and rebalances `from`/`to` source weights linearly from (1.0, 0.0) to
/// (0.0, 1.0). On completion `to` becomes the primary, `from` is deactivated,
/// and the state is cleared.
#[derive(Debug, Clone, Copy)]
struct CrossfadeState {
    from: u32,
    to: u32,
    elapsed: f32,
    duration: f32,
}

fn count_attribute_capacities(metadata: &AnimationMetadata<'_>) -> (usize, usize, usize, usize, usize) {
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
    (cell_c, color_c, shader_c, vertex_c, deform_c)
}

/// Identity-pose `PartState` template — built once per call site and copied
/// into every slot of an empty source's layer. Setting it up lazily (rather
/// than as a `const`) keeps us out of the flatbuffers byte-layout business:
/// the 8 setters write through the same accessors that `extract_pose` uses,
/// so any future schema change to the byte offsets stays automatically in
/// sync.
#[inline]
fn identity_part_state() -> runtime::framedata::ss::runtime::PartState {
    let mut state = runtime::framedata::ss::runtime::PartState::default();
    state.set_scale_x(1.0);
    state.set_scale_y(1.0);
    state.set_local_scale_x(1.0);
    state.set_local_scale_y(1.0);
    state.set_alpha(1.0);
    state.set_local_alpha(1.0);
    state.set_uv_scale_x(1.0);
    state.set_uv_scale_y(1.0);
    state
}

/// Populate `layer.parts_vec` with `num_parts` identity-pose `PartState`
/// entries — `scale`, `local_scale`, `alpha`, `local_alpha`, `uv_scale` set
/// to `1.0`, every other continuous attribute left at `0.0`, `update_flag`
/// empty — and clear `raw_vertex_deform`. Used as the per-frame Stage A
/// output for empty blend sources so the mix step fades the resolved pose
/// toward setup-pose values without an actual animation to interpolate.
///
/// `size` deliberately stays at `0.0`: SS7 has no explicit cell-independent
/// setup size, and 0 is the canonical "no override" value at the schema
/// level. Mixes that need to preserve primary's size during fade-out should
/// use a real secondary source rather than an empty one.
///
/// Uses `Vec::resize` with a precomputed identity `PartState` so the inner
/// fill collapses to a `memcpy`-style loop (the field setters run once at
/// the template, not `N × 8` times per call). LLVM's loop vectorizer turns
/// the resulting copies into SIMD `memcpy` on x86/ARM.
fn fill_layer_with_identity_pose(layer: &mut LayerState<'_>, num_parts: usize) {
    layer.parts_vec.clear();
    layer.parts_vec.resize(num_parts, identity_part_state());
    layer.raw_vertex_deform.clear();
    layer.raw_vertex_deform.resize(num_parts, crate::core::framedata::RawVertexDeform::default());
}

/// Mix per-source poses in `layers` into `resolved_layer` using each
/// source's `weight`. Continuous PartState attributes (TRSA, geometry, UV,
/// skew, mask, bounding_radius) and `raw_vertex_deform` are weighted-summed
/// across active sources after auto-normalization (sum of weights = 1).
/// Discrete attributes (cell/part_color/shader/vertex/deform indices, the
/// per-attribute vecs, flip flags, hide, priority, texture, cell_meta,
/// mesh_deform) are taken from `layers[primary_idx]`. `update_flag` is
/// OR-merged across active sources so Stage B sees every attribute that
/// contributed.
///
/// A source is "active" when `animation_idx >= 0 && weight > 0.0`. When no
/// source is active, the resolved pose equals `layers[primary_idx]`.
fn mix_poses<'b>(sources: &[SourceState], layers: &[LayerState<'b>], primary_idx: usize, resolved_layer: &mut LayerState<'b>) {
    let primary = &layers[primary_idx];
    let num_parts = primary.parts_vec.len();

    // Discrete structure: copied from primary verbatim.
    resolved_layer.cell_vec.clone_from(&primary.cell_vec);
    resolved_layer.parts_color_vec.clone_from(&primary.parts_color_vec);
    resolved_layer.shader_vec.clone_from(&primary.shader_vec);
    resolved_layer.deform_vec.clone_from(&primary.deform_vec);
    resolved_layer.cell_meta_per_part.clone_from(&primary.cell_meta_per_part);
    resolved_layer.cell_texture_hashes.clone_from(&primary.cell_texture_hashes);

    // mesh_deform holds variable-size per-part deltas; deep-copy from primary.
    if resolved_layer.mesh_deform.len() != num_parts {
        resolved_layer.mesh_deform.resize(num_parts, crate::core::framedata::PartDeformFrame::default());
    }
    for (dst, src) in resolved_layer.mesh_deform.iter_mut().zip(primary.mesh_deform.iter()) {
        dst.indices.clear();
        dst.indices.extend_from_slice(&src.indices);
        dst.dx.clear();
        dst.dx.extend_from_slice(&src.dx);
        dst.dy.clear();
        dst.dy.extend_from_slice(&src.dy);
    }

    // Stage B fills these; clear them so stale post-mix data is never read.
    resolved_layer.vertex_vec.clear();
    for v in resolved_layer.local_vertices.iter_mut() {
        *v = 0.0;
    }

    // Sum of active source weights for auto-normalization.
    let total_weight: f32 = sources.iter().filter(|s| s.is_active()).map(|s| s.weight).sum();

    resolved_layer.parts_vec.clear();
    resolved_layer.parts_vec.reserve(num_parts);
    if resolved_layer.raw_vertex_deform.len() != num_parts {
        resolved_layer.raw_vertex_deform.clear();
        resolved_layer.raw_vertex_deform.resize(num_parts, crate::core::framedata::RawVertexDeform::default());
    } else {
        for slot in resolved_layer.raw_vertex_deform.iter_mut() {
            *slot = crate::core::framedata::RawVertexDeform::default();
        }
    }

    for i in 0..num_parts {
        let primary_state = primary.parts_vec[i];
        let mut mixed = primary_state;

        if total_weight > 0.0 {
            let inv_total = 1.0 / total_weight;
            let mut pos_x = 0.0_f32;
            let mut pos_y = 0.0_f32;
            let mut pos_z = 0.0_f32;
            let mut rot_x = 0.0_f32;
            let mut rot_y = 0.0_f32;
            let mut rot_z = 0.0_f32;
            let mut scl_x = 0.0_f32;
            let mut scl_y = 0.0_f32;
            let mut loscl_x = 0.0_f32;
            let mut loscl_y = 0.0_f32;
            let mut alpha = 0.0_f32;
            let mut local_alpha = 0.0_f32;
            let mut pivot_x = 0.0_f32;
            let mut pivot_y = 0.0_f32;
            let mut anchor_x = 0.0_f32;
            let mut anchor_y = 0.0_f32;
            let mut size_x = 0.0_f32;
            let mut size_y = 0.0_f32;
            let mut uv_tx = 0.0_f32;
            let mut uv_ty = 0.0_f32;
            let mut uv_rz = 0.0_f32;
            let mut uv_sx = 0.0_f32;
            let mut uv_sy = 0.0_f32;
            let mut bound_r = 0.0_f32;
            let mut mask = 0.0_f32;
            let mut skew_x = 0.0_f32;
            let mut skew_y = 0.0_f32;
            let mut dx_acc = wide::f32x4::splat(0.0);
            let mut dy_acc = wide::f32x4::splat(0.0);
            let mut deform_present = false;
            let mut flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();

            for (j, src) in sources.iter().enumerate() {
                if !src.is_active() {
                    continue;
                }
                let w = src.weight * inv_total;
                let s = &layers[j].parts_vec[i];
                pos_x += w * s.position_x();
                pos_y += w * s.position_y();
                pos_z += w * s.position_z();
                rot_x += w * s.rotation_x();
                rot_y += w * s.rotation_y();
                rot_z += w * s.rotation_z();
                scl_x += w * s.scale_x();
                scl_y += w * s.scale_y();
                loscl_x += w * s.local_scale_x();
                loscl_y += w * s.local_scale_y();
                alpha += w * s.alpha();
                local_alpha += w * s.local_alpha();
                pivot_x += w * s.pivot_x();
                pivot_y += w * s.pivot_y();
                anchor_x += w * s.anchor_x();
                anchor_y += w * s.anchor_y();
                size_x += w * s.size_x();
                size_y += w * s.size_y();
                uv_tx += w * s.uv_translation_x();
                uv_ty += w * s.uv_translation_y();
                uv_rz += w * s.uv_rotation_z();
                uv_sx += w * s.uv_scale_x();
                uv_sy += w * s.uv_scale_y();
                bound_r += w * s.bounding_radius();
                mask += w * s.mask();
                skew_x += w * s.skew_x();
                skew_y += w * s.skew_y();
                flag |= s.update_flag();

                let rvd = &layers[j].raw_vertex_deform[i];
                if rvd.present {
                    deform_present = true;
                    let w_v = wide::f32x4::splat(w);
                    dx_acc = w_v.mul_add(wide::f32x4::from(rvd.dx), dx_acc);
                    dy_acc = w_v.mul_add(wide::f32x4::from(rvd.dy), dy_acc);
                }
            }

            mixed.set_position_x(pos_x);
            mixed.set_position_y(pos_y);
            mixed.set_position_z(pos_z);
            mixed.set_rotation_x(rot_x);
            mixed.set_rotation_y(rot_y);
            mixed.set_rotation_z(rot_z);
            mixed.set_scale_x(scl_x);
            mixed.set_scale_y(scl_y);
            mixed.set_local_scale_x(loscl_x);
            mixed.set_local_scale_y(loscl_y);
            mixed.set_alpha(alpha);
            mixed.set_local_alpha(local_alpha);
            mixed.set_pivot_x(pivot_x);
            mixed.set_pivot_y(pivot_y);
            mixed.set_anchor_x(anchor_x);
            mixed.set_anchor_y(anchor_y);
            mixed.set_size_x(size_x);
            mixed.set_size_y(size_y);
            mixed.set_uv_translation_x(uv_tx);
            mixed.set_uv_translation_y(uv_ty);
            mixed.set_uv_rotation_z(uv_rz);
            mixed.set_uv_scale_x(uv_sx);
            mixed.set_uv_scale_y(uv_sy);
            mixed.set_bounding_radius(bound_r);
            mixed.set_mask(mask);
            mixed.set_skew_x(skew_x);
            mixed.set_skew_y(skew_y);
            mixed.set_update_flag(flag);

            if deform_present {
                resolved_layer.raw_vertex_deform[i] = crate::core::framedata::RawVertexDeform { present: true, dx: dx_acc.to_array(), dy: dy_acc.to_array() };
            }
        }

        resolved_layer.parts_vec.push(mixed);
    }
}

/// Per-instance playback state. Owns one playback source per blend layer,
/// the per-source pose buffers (`layers`), the post-mix pose
/// (`resolved_layer`) that feeds global resolution, the resolved global
/// buffers, and the configuration knobs that influence frame evaluation.
/// Designed to be the "Lightweight Instance" of Phase 3 — many of these
/// can coexist against a single shared `EvaluationContext`.
///
/// `sources` and `layers` are kept length-parallel and always contain at
/// least one slot (the primary). The `primary` field designates the
/// authoritative source for events and non-blendable discrete attributes.
struct InstanceState<'a> {
    sources: Vec<SourceState>,
    layers: Vec<LayerState<'a>>,
    resolved_layer: LayerState<'a>,
    primary: u32,
    z_order: crate::core::z_order::ZOrderTable,
    globals: GlobalBuffers,
    collected_event_indices: Vec<i32>,
    /// Parallel to `collected_event_indices`: source handle that fired each
    /// collected event. With `event_threshold == 1.0` (default), every entry
    /// equals `primary` — legacy "Primary fires events" semantics. When
    /// secondaries opt-in (lower threshold), entries record the firing source.
    collected_event_source_handles: Vec<u32>,
    coordinate_system: crate::core::SsCoordinateSystem,
    origin: crate::core::SsOrigin,
    unit: crate::core::SsUnit,
    render_mode: crate::core::RenderMode,
    crossfade: Option<CrossfadeState>,
}

impl<'a> InstanceState<'a> {
    fn new() -> Self {
        Self {
            sources: alloc::vec![SourceState::new()],
            layers: alloc::vec![LayerState::new()],
            resolved_layer: LayerState::new(),
            primary: 0,
            z_order: crate::core::z_order::ZOrderTable::new(),
            globals: GlobalBuffers::new(),
            collected_event_indices: Vec::new(),
            collected_event_source_handles: Vec::new(),
            coordinate_system: crate::core::SsCoordinateSystem::default(),
            origin: crate::core::SsOrigin::default(),
            unit: crate::core::SsUnit::default(),
            render_mode: crate::core::RenderMode::default(),
            crossfade: None,
        }
    }

    fn primary_idx(&self) -> usize {
        self.primary as usize
    }

    fn collapse_to_primary(&mut self) {
        self.sources.truncate(1);
        self.layers.truncate(1);
        self.sources[0] = SourceState::new();
        self.primary = 0;
        self.crossfade = None;
    }

    fn primary_source(&self) -> &SourceState {
        &self.sources[self.primary_idx()]
    }

    fn primary_source_mut(&mut self) -> &mut SourceState {
        let idx = self.primary_idx();
        &mut self.sources[idx]
    }

    /// `true` when the primary is the only active source. The mix step can
    /// short-circuit to a `mem::swap` in this case — the weighted sum would
    /// yield primary's pose verbatim with `inv_total * primary.weight == 1.0`,
    /// so the swap is byte-equivalent to running `mix_poses` while skipping
    /// the per-part vec clone + 27 per-part PartState setter calls.
    fn primary_is_only_active(&self) -> bool {
        let primary_idx = self.primary_idx();
        if !self.sources[primary_idx].is_active() {
            return false;
        }
        !self.sources.iter().enumerate().any(|(i, s)| i != primary_idx && s.is_active())
    }
}

pub struct Context<'a> {
    eval: EvaluationContext<'a>,
    instance: InstanceState<'a>,
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context { eval: EvaluationContext::new(), instance: InstanceState::new() }
    }

    pub fn reset(&mut self) {
        self.eval.resource = None;
        self.eval.fb.reset();
        self.instance.collapse_to_primary();
        self.instance.layers[0].cursors.clear();
        self.instance.resolved_layer.cursors.clear();
        self.instance.coordinate_system = crate::core::SsCoordinateSystem::default();
        self.instance.origin = crate::core::SsOrigin::default();
        self.instance.unit = crate::core::SsUnit::default();
        self.instance.render_mode = crate::core::RenderMode::default();
        self.instance.z_order = crate::core::z_order::ZOrderTable::new();
    }

    pub fn set_render_mode(&mut self, mode: crate::core::RenderMode) {
        self.instance.render_mode = mode;
    }

    pub fn render_mode(&self) -> crate::core::RenderMode {
        self.instance.render_mode
    }

    pub fn bind_resource(&mut self, resource: &'a AnimationResource<'a>) {
        self.eval.resource = Some(resource);
        self.eval.fb.reset();
        self.instance.collapse_to_primary();
        self.instance.layers[0].cursors.clear();
        self.instance.resolved_layer.cursors.clear();
        // Configuration (coordinate_system, origin, unit) is preserved.
    }

    pub fn get_ssab_data(&self) -> Option<&[u8]> {
        self.eval.resource.map(|r| r.get_ssab_data())
    }

    pub fn setup_animation(&mut self, name: &str) -> Result<(), error::SSRuntimeError> {
        let hash = fnv1a_hash_str(name);
        self.setup_animation_by_hash(hash)
    }

    pub fn setup_animation_by_hash(&mut self, hash: u32) -> Result<(), error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };

        let Some(index) = resource.find_animation_index_by_hash(hash) else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };

        let metadata = &resource.animations[index];
        self.instance.collapse_to_primary();
        let primary = self.instance.primary_source_mut();
        primary.controller.setup(metadata.start_frame, metadata.end_frame, metadata.fps);
        primary.animation_idx = index as i32;
        primary.weight = 1.0;

        let num_parts = metadata.cached_parts.len();
        let (cell_c, color_c, shader_c, vertex_c, deform_c) = count_attribute_capacities(metadata);

        for layer in self.instance.layers.iter_mut() {
            layer.resize_for_parts(num_parts, cell_c, color_c, shader_c, vertex_c, deform_c);
        }
        self.instance.resolved_layer.resize_for_parts(num_parts, cell_c, color_c, shader_c, vertex_c, deform_c);

        let globals = &mut self.instance.globals;
        globals.world_matrices.clear();
        globals.world_matrices.resize(num_parts, crate::core::SsMatrix4x4::IDENTITY);
        globals.inheritance_matrices.clear();
        globals.inheritance_matrices.resize(num_parts, crate::core::SsMatrix4x4::IDENTITY);
        globals.local_uvs.clear();
        globals.local_uvs.resize(num_parts * 10, 0.0);
        globals.shape_vertices.clear();
        globals.shape_vertices.resize(num_parts * crate::core::framedata::SHAPE_VERTICES_STRIDE, 0.0);
        globals.shape_vertex_box_coords.clear();
        globals.shape_vertex_box_coords.resize(num_parts * crate::core::framedata::SHAPE_VERTICES_STRIDE, 0.0);
        globals.shape_vertex_counts.clear();
        globals.shape_vertex_counts.resize(num_parts, 0);
        // Invalidate the static mesh buffers so compute_mesh_vertices rebuilds
        // them for the newly selected animation's Cells.
        globals.mesh_index_offsets.clear();

        self.instance.z_order.setup(&metadata.cached_parts);

        Ok(())
    }

    /// Adds a secondary blend source. Returns a stable `u32` handle (slot
    /// index) that can be used with `set_source_weight`, `set_primary`,
    /// `set_source_frame`, `set_source_speed`, and `remove_blend_source`.
    ///
    /// The animation must live in the currently bound `AnimationResource`
    /// (same-SSAB constraint). Call `setup_animation` first to establish
    /// the primary; this method errors with `AnimationNotFound` until then.
    pub fn add_blend_source(&mut self, name: &str, weight: f32) -> Result<u32, error::SSRuntimeError> {
        let hash = fnv1a_hash_str(name);
        self.add_blend_source_by_hash(hash, weight)
    }

    pub fn add_blend_source_by_hash(&mut self, hash: u32, weight: f32) -> Result<u32, error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };
        if !self.instance.primary_source().has_real_animation() {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }
        let Some(index) = resource.find_animation_index_by_hash(hash) else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };

        let metadata = &resource.animations[index];
        let num_parts = metadata.cached_parts.len();
        let (cell_c, color_c, shader_c, vertex_c, deform_c) = count_attribute_capacities(metadata);

        let mut source = SourceState::new();
        source.controller.setup(metadata.start_frame, metadata.end_frame, metadata.fps);
        source.animation_idx = index as i32;
        source.weight = weight;
        self.instance.sources.push(source);

        let mut layer = LayerState::new();
        layer.resize_for_parts(num_parts, cell_c, color_c, shader_c, vertex_c, deform_c);
        self.instance.layers.push(layer);

        Ok((self.instance.sources.len() - 1) as u32)
    }

    /// Marks the given slot as inactive. Stable handles: the slot remains
    /// in `sources`/`layers` (so other handles do not shift), but mix
    /// iteration skips it. Cannot remove the primary slot; call
    /// `set_primary` to relocate the primary first. Cancels any in-flight
    /// crossfade that referenced this slot.
    pub fn remove_blend_source(&mut self, handle: u32) -> Result<(), error::SSRuntimeError> {
        if handle == self.instance.primary {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let idx = handle as usize;
        let Some(source) = self.instance.sources.get_mut(idx) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        source.controller.reset_setup();
        source.animation_idx = -1;
        source.weight = 0.0;
        source.event_threshold = 1.0;
        if let Some(cf) = self.instance.crossfade
            && (cf.from == handle || cf.to == handle)
        {
            self.instance.crossfade = None;
        }
        Ok(())
    }

    /// Deactivates every non-primary slot. The primary stays untouched.
    /// Cancels any in-flight crossfade.
    pub fn clear_blend_sources(&mut self) {
        let primary_idx = self.instance.primary_idx();
        for (i, source) in self.instance.sources.iter_mut().enumerate() {
            if i == primary_idx {
                continue;
            }
            source.controller.reset_setup();
            source.animation_idx = -1;
            source.weight = 0.0;
            source.event_threshold = 1.0;
        }
        self.instance.crossfade = None;
    }

    pub fn blend_source_count(&self) -> u32 {
        self.instance.sources.iter().filter(|s| s.animation_idx != -1).count() as u32
    }

    pub fn set_source_weight(&mut self, handle: u32, weight: f32) -> Result<(), error::SSRuntimeError> {
        let idx = handle as usize;
        let Some(source) = self.instance.sources.get_mut(idx) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        if source.animation_idx == -1 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        source.weight = weight;
        Ok(())
    }

    pub fn source_weight(&self, handle: u32) -> Option<f32> {
        let source = self.instance.sources.get(handle as usize)?;
        (source.animation_idx != -1).then_some(source.weight)
    }

    pub fn set_primary(&mut self, handle: u32) -> Result<(), error::SSRuntimeError> {
        let Some(source) = self.instance.sources.get(handle as usize) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        if source.animation_idx < 0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        self.instance.primary = handle;
        Ok(())
    }

    pub fn primary_handle(&self) -> u32 {
        self.instance.primary
    }

    pub fn set_source_frame(&mut self, handle: u32, frame: f32) -> Result<(), error::SSRuntimeError> {
        let idx = handle as usize;
        let Some(source) = self.instance.sources.get_mut(idx) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        if source.animation_idx < 0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        source.controller.set_frame_no(frame);
        Ok(())
    }

    pub fn source_frame(&self, handle: u32) -> Option<f32> {
        let source = self.instance.sources.get(handle as usize)?;
        (source.animation_idx >= 0).then(|| source.controller.get_frame_no())
    }

    pub fn set_source_speed(&mut self, handle: u32, speed: f32) -> Result<(), error::SSRuntimeError> {
        let idx = handle as usize;
        let Some(source) = self.instance.sources.get_mut(idx) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        if source.animation_idx < 0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        source.controller.set_animation_speed(speed);
        Ok(())
    }

    /// Adds an "empty" blend source that contributes identity values (scale /
    /// alpha / uv_scale = 1, all other continuous attributes = 0) to the mix
    /// — i.e. the SS analog of Spine's `setEmptyAnimation`. Returns a stable
    /// slot handle.
    ///
    /// Empty sources do not own an animation, controller, or events, so
    /// `set_source_frame`, `set_source_speed`, `set_primary`, and
    /// `set_source_event_threshold` reject them with `InvalidParameter`.
    /// `set_source_weight`, `source_weight`, `remove_blend_source`, and
    /// `clear_blend_sources` operate on them like any other slot.
    ///
    /// Primary must already be set up (real animation); use this to fade the
    /// existing primary toward a setup/identity pose.
    pub fn add_empty_blend_source(&mut self, weight: f32) -> Result<u32, error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };
        let primary_anim_idx = self.instance.primary_source().animation_idx;
        if primary_anim_idx < 0 {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }
        let num_parts = resource.animations[primary_anim_idx as usize].cached_parts.len();

        let mut source = SourceState::new();
        source.animation_idx = EMPTY_ANIMATION_IDX;
        source.weight = weight;
        self.instance.sources.push(source);

        let mut layer = LayerState::new();
        layer.resize_for_parts(num_parts, 0, 0, 0, 0, 0);
        self.instance.layers.push(layer);

        Ok((self.instance.sources.len() - 1) as u32)
    }

    /// Per-source event firing threshold. Default `1.0` for every slot —
    /// secondaries with the default threshold never fire events (legacy
    /// "Primary-only" behavior). Lowering the threshold below the source's
    /// current `weight` opts it into event collection; events from multiple
    /// sources can be distinguished via [`get_passed_event_source_handles`].
    ///
    /// Primary always fires its events regardless of this setting; rejects
    /// empty sources (`InvalidParameter`).
    pub fn set_source_event_threshold(&mut self, handle: u32, threshold: f32) -> Result<(), error::SSRuntimeError> {
        let idx = handle as usize;
        let Some(source) = self.instance.sources.get_mut(idx) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        if !source.has_real_animation() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        source.event_threshold = threshold;
        Ok(())
    }

    pub fn source_event_threshold(&self, handle: u32) -> Option<f32> {
        let source = self.instance.sources.get(handle as usize)?;
        source.has_real_animation().then_some(source.event_threshold)
    }

    /// Starts a linear weight crossfade from `from` to `to` over `duration_ms`
    /// (same time unit as [`Context::update`]'s `delta`). Each subsequent
    /// `update(delta)` advances the timer, rebalances the two source weights
    /// from `(1.0, 0.0)` to `(0.0, 1.0)`, and on completion promotes `to` to
    /// primary and deactivates `from`. Cancels any in-flight crossfade and
    /// overwrites the current weights of `from`/`to` immediately on this call
    /// to `(1.0, 0.0)` so the fade always starts from a defined state.
    ///
    /// Errors: `InvalidParameter` when handles are invalid / inactive /
    /// identical, or `duration_ms <= 0.0`. Empty sources are accepted as
    /// `to` (useful for fade-out to setup pose); `from` must carry a real
    /// animation so the post-fade primary is well-defined.
    pub fn crossfade(&mut self, from: u32, to: u32, duration_ms: f32) -> Result<(), error::SSRuntimeError> {
        if from == to || duration_ms.is_nan() || duration_ms <= 0.0 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let from_idx = from as usize;
        let to_idx = to as usize;
        let from_source = self.instance.sources.get(from_idx).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if !from_source.has_real_animation() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let to_source = self.instance.sources.get(to_idx).ok_or(error::SSRuntimeError::InvalidParameter)?;
        if to_source.animation_idx == -1 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        self.instance.sources[from_idx].weight = 1.0;
        self.instance.sources[to_idx].weight = 0.0;
        self.instance.crossfade = Some(CrossfadeState { from, to, elapsed: 0.0, duration: duration_ms });
        Ok(())
    }

    /// Returns the in-flight crossfade's progress in `[0.0, 1.0]`, or `None`
    /// when no crossfade is active.
    pub fn crossfade_progress(&self) -> Option<f32> {
        self.instance.crossfade.map(|cf| (cf.elapsed / cf.duration).clamp(0.0, 1.0))
    }

    /// Cancels the in-flight crossfade. Source weights are left at their
    /// current value — call [`set_source_weight`] to normalize them.
    pub fn cancel_crossfade(&mut self) {
        self.instance.crossfade = None;
    }

    pub fn get_frame_data(&mut self, frame_no: f32) -> Result<&[u8], error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };

        let primary_anim_idx = self.instance.primary_source().animation_idx;
        if primary_anim_idx < 0 {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }

        let primary_metadata = &resource.animations[primary_anim_idx as usize];
        if !((primary_metadata.start_frame as f32) <= frame_no && frame_no < (primary_metadata.end_frame as f32 + 1.0)) {
            return Err(error::SSRuntimeError::InvalidParameter);
        }

        self.eval.fb.reset();

        let coordinate_system = self.instance.coordinate_system;
        let origin = self.instance.origin;
        let unit = self.instance.unit;

        // Stage A: per-source pose extraction. Each source uses its own
        // controller-driven frame; primary uses the explicit `frame_no`
        // argument to preserve single-animation backward compatibility.
        // Primary is always extracted regardless of its weight — `mix_poses`
        // uses it as the discrete-attribute / structure source even when its
        // continuous-value contribution is zero. Empty sources (sentinel
        // `EMPTY_ANIMATION_IDX`) skip `extract_pose` and fill their layer
        // with an identity pose so the weighted sum fades the resolved pose
        // toward setup-pose values.
        let primary_idx = self.instance.primary_idx();
        let primary_num_parts = primary_metadata.cached_parts.len();
        let num_sources = self.instance.sources.len();
        for i in 0..num_sources {
            let source_anim_idx = self.instance.sources[i].animation_idx;
            if source_anim_idx == -1 {
                continue;
            }
            if i != primary_idx && self.instance.sources[i].weight <= 0.0 {
                continue;
            }
            if source_anim_idx == EMPTY_ANIMATION_IDX {
                fill_layer_with_identity_pose(&mut self.instance.layers[i], primary_num_parts);
                continue;
            }
            let frame_for_source = if i == primary_idx { frame_no } else { self.instance.sources[i].controller.get_frame_no() };
            let metadata = &resource.animations[source_anim_idx as usize];
            let layer = &mut self.instance.layers[i];
            core::framedata::extract_pose(
                &mut self.eval.fb,
                &metadata.cached_parts,
                frame_for_source,
                &mut layer.parts_vec,
                &mut layer.cell_vec,
                &mut layer.parts_color_vec,
                &mut layer.shader_vec,
                &mut layer.deform_vec,
                &mut layer.raw_vertex_deform,
                &mut layer.cursors,
                &mut layer.cell_meta_per_part,
                &mut layer.cell_texture_hashes,
                &mut layer.mesh_deform,
                coordinate_system,
                origin,
                unit,
                metadata.canvas_width,
                metadata.canvas_height,
                &mut self.eval.deform_interpolation_start_workspace,
                &mut self.eval.deform_interpolation_end_workspace,
                &mut self.eval.deform_interpolation_out_workspace,
                &mut self.eval.deform_result_coordinate_workspace,
                &mut self.eval.deform_result_index_workspace,
                &mut self.eval.inheritance_alphas_workspace,
            );
        }

        if self.instance.primary_is_only_active() {
            // Fast path: weighted sum with one active source = identity. Swap
            // is O(1) vs `mix_poses`'s per-part vec clone + 27 setters.
            ::core::mem::swap(&mut self.instance.layers[primary_idx], &mut self.instance.resolved_layer);
        } else {
            mix_poses(&self.instance.sources, &self.instance.layers, primary_idx, &mut self.instance.resolved_layer);
        }

        // Stage B + C + draw plan on the resolved pose.
        let resolved = &mut self.instance.resolved_layer;
        let offset = core::framedata::compose_frame_data(
            &mut self.eval.fb,
            &primary_metadata.cached_parts,
            &mut self.instance.z_order,
            &mut resolved.parts_vec,
            &resolved.cell_vec,
            &resolved.parts_color_vec,
            &resolved.shader_vec,
            &mut resolved.vertex_vec,
            &resolved.deform_vec,
            &resolved.raw_vertex_deform,
            &resolved.cell_meta_per_part,
            &resolved.cell_texture_hashes,
            &mut resolved.local_vertices,
            coordinate_system,
            self.instance.render_mode,
            &mut self.eval.draw_order_workspace,
            &mut self.eval.batches_workspace,
            &mut self.eval.mask_indices_workspace,
        );

        // Phase 2: resolve global state (matrices / UVs / shape vertices)
        // from the (post-mix) local pose.
        Self::resolve_global_state(&mut self.instance, primary_metadata);

        self.eval.fb.finish_minimal(offset);
        Ok(self.eval.fb.finished_data())
    }

    /// Phase 2 of the frame pipeline. Consumes the (post-mix) local pose
    /// in `instance.resolved_layer` and writes per-instance global outputs
    /// in `instance.globals` — world / inheritance matrices, local UVs,
    /// and shape vertex buffers.
    fn resolve_global_state(instance: &mut InstanceState<'a>, metadata: &AnimationMetadata<'a>) {
        if instance.render_mode == crate::core::RenderMode::Direct {
            Self::compute_matrices(instance, &metadata.cached_parts);
            // Skinned mesh skinning depends on world matrices, so it lives in the
            // Direct path. Mesh parts in Indirect mode get empty per-part ranges.
            Self::compute_mesh_vertices(instance, &metadata.cached_parts);
        } else {
            Self::clear_mesh_vertices(instance, metadata.cached_parts.len());
        }
        Self::compute_local_uvs(instance, &metadata.cached_parts);
        Self::compute_shape_vertices(instance, &metadata.cached_parts);
    }

    /// Writes per-part mesh world-space vertex positions to
    /// `globals.mesh_vertices_x` / `_y` for the current frame — skeletal
    /// (bone-weighted) and Deform-only meshes alike, with the frame's deform
    /// deltas applied.
    ///
    /// Layout: x/y are SoA, sliced per part by `mesh_vertex_offsets` (a CSR
    /// prefix sum of length `num_parts + 1`; non-mesh parts get an empty
    /// range). The CSR offsets and the static UV / index buffers are built
    /// once by `setup_static_mesh_buffers`; only the vertex positions are
    /// recomputed each frame.
    #[allow(clippy::needless_range_loop)]
    fn compute_mesh_vertices(instance: &mut InstanceState<'a>, cached_parts: &[crate::core::framedata::CachedPartData]) {
        let num_parts = instance.resolved_layer.parts_vec.len();

        // The CSR offsets and the static UV / index buffers depend only on the
        // chosen animation's per-part Cells (fixed by the SS mesh spec). Build
        // them — and size the SoA position buffers — once; `play()` clears
        // `mesh_index_offsets` so a new animation forces the rebuild.
        if instance.globals.mesh_index_offsets.len() != num_parts + 1 {
            Self::setup_static_mesh_buffers(instance, cached_parts);
        }

        let layer = &instance.resolved_layer;
        // SSAB bind coordinates are authored Y-up; flip when the instance
        // (hence the bone world matrices) runs Y-down.
        let flip_y = instance.coordinate_system == crate::core::SsCoordinateSystem::YDown;
        // The Deform-only mesh rest comes from the cell's `table_coordinate`,
        // which ssconverter copies verbatim from the `.ssce` meshPointList —
        // texture-pixel space, i.e. already Y-down. So it must be flipped for a
        // Y-up target, the OPPOSITE of the Y-up-native bind coordinates above.
        let flip_rest = instance.coordinate_system == crate::core::SsCoordinateSystem::YUp;
        let globals = &mut instance.globals;

        let total_vertices = globals.mesh_vertices_x.len();
        if total_vertices == 0 {
            return;
        }

        for i in 0..num_parts {
            let Some(cached) = cached_parts.get(i) else {
                continue;
            };
            let start = globals.mesh_vertex_offsets[i] as usize;
            let end = globals.mesh_vertex_offsets[i + 1] as usize;
            if end <= start {
                continue; // not a mesh part
            }

            // Skeletal mesh: a bone-weighted binding drives the vertices.
            let skeletal = cached.mesh_binding.as_ref().map(|b| !b.bone_indices.is_empty()).unwrap_or(false);
            if skeletal {
                let binding = cached.mesh_binding.as_ref().unwrap();
                // The SS7 `.ssae` meshList carries the N rendered mesh vertices
                // followed by 4 trailing placeholder entries (the cell-quad
                // corners, bound to no bone), so a well-formed binding has
                // `cell_count + 4` vertices. `compute_skinned_mesh_vertices`
                // skins exactly `out_x.len()` (= cell count) vertices from the
                // front of the binding, so a longer binding is fine — the
                // trailing extras are simply never read. Only a binding with
                // *fewer* vertices than the cell is unusable.
                if binding.num_vertices() < end - start {
                    continue; // bind has fewer vertices than the cell — cannot skin
                }
                let deform = layer.mesh_deform.get(i).filter(|d| !d.indices.is_empty()).map(|d| (d.indices.as_slice(), d.dx.as_slice(), d.dy.as_slice(), &globals.world_matrices[i]));
                let (out_x, out_y) = (&mut globals.mesh_vertices_x[start..end], &mut globals.mesh_vertices_y[start..end]);
                crate::core::geometry::compute_skinned_mesh_vertices(&binding.influence_offsets, &binding.bone_indices, &binding.weights, &binding.coords_x, &binding.coords_y, &globals.world_matrices, deform, flip_y, out_x, out_y);
                continue;
            }

            // Deform-only mesh: no bones — rigidly transform the cell's rest
            // mesh by the part's own world matrix.
            let Some(mesh_cell) = cached.mesh_cell.as_ref() else {
                continue;
            };
            if mesh_cell.rest_x.len() != (end - start) || i >= globals.world_matrices.len() {
                continue;
            }
            // Rest transform (W · rest) — SIMD over 4 vertices at a time.
            {
                let (out_x, out_y) = (&mut globals.mesh_vertices_x[start..end], &mut globals.mesh_vertices_y[start..end]);
                crate::core::geometry::compute_rigid_mesh_vertices(&mesh_cell.rest_x, &mesh_cell.rest_y, &globals.world_matrices[i], flip_rest, out_x, out_y);
            }
            // Apply deform deltas as-is — evaluate_part has already applied any
            // coordinate-system Y conversion. Sparse scatter, kept scalar.
            if let Some(deform) = layer.mesh_deform.get(i) {
                let vert_count = end - start;
                let mat = &globals.world_matrices[i];
                let (m0, m1, m4, m5) = (mat.m[0], mat.m[1], mat.m[4], mat.m[5]);
                for k in 0..deform.indices.len() {
                    let v = deform.indices[k] as usize;
                    if v < vert_count {
                        globals.mesh_vertices_x[start + v] += m0 * deform.dx[k] + m4 * deform.dy[k];
                        globals.mesh_vertices_y[start + v] += m1 * deform.dx[k] + m5 * deform.dy[k];
                    }
                }
            }
        }
    }

    /// Builds the per-instance static mesh buffers — CSR vertex / index
    /// offsets, texture-space UVs, triangle indices — and sizes and zeroes the
    /// SoA position buffers. These depend only on the chosen animation's
    /// per-part Cells (fixed by the SS mesh spec), so this runs once per
    /// `play()` rather than every frame. A statically malformed mesh part
    /// keeps the zeros written here, so the per-frame pass needs no clear.
    fn setup_static_mesh_buffers(instance: &mut InstanceState<'a>, cached_parts: &[crate::core::framedata::CachedPartData]) {
        let num_parts = instance.resolved_layer.parts_vec.len();
        let globals = &mut instance.globals;

        // CSR vertex offsets — non-mesh parts contribute an empty range. The
        // Cell is the authoritative geometry source for both skeletal and
        // Deform-only meshes; the bind cache only adds skinning.
        globals.mesh_vertex_offsets.clear();
        globals.mesh_vertex_offsets.resize(num_parts + 1, 0);
        for i in 0..num_parts {
            let vertex_count = cached_parts.get(i).and_then(|c| c.mesh_cell.as_ref()).map(|m| m.num_vertices()).unwrap_or(0);
            globals.mesh_vertex_offsets[i + 1] = globals.mesh_vertex_offsets[i] + vertex_count as u32;
        }
        let total_vertices = *globals.mesh_vertex_offsets.last().unwrap_or(&0) as usize;

        // CSR triangle-index offsets.
        globals.mesh_index_offsets.clear();
        globals.mesh_index_offsets.resize(num_parts + 1, 0);
        for i in 0..num_parts {
            let index_count = cached_parts.get(i).and_then(|c| c.mesh_cell.as_ref()).map(|m| m.indices.len()).unwrap_or(0);
            globals.mesh_index_offsets[i + 1] = globals.mesh_index_offsets[i] + index_count as u32;
        }
        let total_indices = *globals.mesh_index_offsets.last().unwrap_or(&0) as usize;

        // Static UVs (texture-pixel space) and triangle indices, per part.
        globals.mesh_indices.clear();
        globals.mesh_indices.reserve(total_indices);
        globals.mesh_uvs.clear();
        globals.mesh_uvs.resize(total_vertices * 2, 0.0);
        for i in 0..num_parts {
            let Some(mesh_cell) = cached_parts.get(i).and_then(|c| c.mesh_cell.as_ref()) else {
                continue;
            };
            globals.mesh_indices.extend_from_slice(&mesh_cell.indices);
            // UVs share the position CSR; copy only when the cell vertex count
            // matches the slot (true for well-formed meshes).
            let start = globals.mesh_vertex_offsets[i] as usize;
            let end = globals.mesh_vertex_offsets[i + 1] as usize;
            if mesh_cell.uvs.len() == (end - start) * 2 {
                globals.mesh_uvs[start * 2..end * 2].copy_from_slice(&mesh_cell.uvs);
            }
        }

        // SoA position buffers — sized and zeroed once. Each valid mesh part
        // overwrites its whole range every frame.
        globals.mesh_vertices_x.clear();
        globals.mesh_vertices_x.resize(total_vertices, 0.0);
        globals.mesh_vertices_y.clear();
        globals.mesh_vertices_y.resize(total_vertices, 0.0);
    }

    fn clear_mesh_vertices(instance: &mut InstanceState<'a>, num_parts: usize) {
        let globals = &mut instance.globals;
        if globals.mesh_vertex_offsets.len() != num_parts + 1 {
            globals.mesh_vertex_offsets.resize(num_parts + 1, 0);
        }
        for slot in globals.mesh_vertex_offsets.iter_mut() {
            *slot = 0;
        }
        globals.mesh_vertices_x.clear();
        globals.mesh_vertices_y.clear();
        globals.mesh_uvs.clear();
        globals.mesh_indices.clear();
        globals.mesh_index_offsets.clear();
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_matrices(instance: &mut InstanceState<'a>, cached_parts: &[crate::core::framedata::CachedPartData]) {
        let layer = &instance.resolved_layer;
        let globals = &mut instance.globals;
        for i in 0..layer.parts_vec.len() {
            let parent_idx = cached_parts[i].parent_index;
            let valid_parent_idx = (parent_idx >= 0 && (parent_idx as usize) < i).then_some(parent_idx as usize);

            let parent_world = valid_parent_idx.map(|p_idx| &globals.inheritance_matrices[p_idx]);
            let parent_size = valid_parent_idx.map(|p_idx| {
                let p_state = &layer.parts_vec[p_idx];
                (p_state.size_x(), p_state.size_y())
            });

            let part_state = &layer.parts_vec[i];
            let (inh_m, draw_m) = crate::core::SsMatrix4x4::compute_world_matrix(part_state, parent_world, parent_size);
            globals.inheritance_matrices[i] = inh_m;
            globals.world_matrices[i] = draw_m;
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_local_uvs(instance: &mut InstanceState<'a>, cached_parts: &[crate::core::framedata::CachedPartData]) {
        let layer = &instance.resolved_layer;
        let globals = &mut instance.globals;
        let num_parts = layer.parts_vec.len();
        if globals.local_uvs.len() != num_parts * 10 {
            globals.local_uvs.resize(num_parts * 10, 0.0);
        }
        for slot in globals.local_uvs.iter_mut() {
            *slot = 0.0;
        }

        for i in 0..num_parts {
            let state = &layer.parts_vec[i];
            if state.cell() < 0 {
                continue;
            }
            let cursor = layer.cursors[i * core::framedata::NUM_PART_ATTRIBUTES + core::framedata::IDX_ATTRIBUTECELL];
            let Some(meta) = cached_parts[i].resolved_cells.get(cursor) else { continue };
            if meta.size_w == 0.0 && meta.size_h == 0.0 {
                continue;
            }

            let rect_right = meta.rect_left + meta.size_w;
            let rect_bottom = meta.rect_top + meta.size_h;

            let (tu, tv, rot_deg, su, sv, fh, fv, ifh, ifv) =
                (state.uv_translation_x() * meta.tex_w, state.uv_translation_y() * meta.tex_h, state.uv_rotation_z(), state.uv_scale_x(), state.uv_scale_y(), state.flip_h(), state.flip_v(), state.img_flip_h(), state.img_flip_v());

            let (us, vs) = crate::core::geometry::compute_local_uvs((meta.rect_left, meta.rect_top, rect_right, rect_bottom), (tu, tv, rot_deg, su, sv), (fh, fv), (ifh, ifv), meta.rotated);

            let offset = i * 10;
            for k in 0..5 {
                globals.local_uvs[offset + k * 2] = us[k];
                globals.local_uvs[offset + k * 2 + 1] = vs[k];
            }
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_shape_vertices(instance: &mut InstanceState<'a>, cached_parts: &[crate::core::framedata::CachedPartData]) {
        let coordinate_system = instance.coordinate_system;
        let layer = &instance.resolved_layer;
        let globals = &mut instance.globals;
        let num_parts = layer.parts_vec.len();
        let stride = crate::core::framedata::SHAPE_VERTICES_STRIDE;
        if globals.shape_vertices.len() != num_parts * stride {
            globals.shape_vertices.resize(num_parts * stride, 0.0);
        }
        if globals.shape_vertex_box_coords.len() != num_parts * stride {
            globals.shape_vertex_box_coords.resize(num_parts * stride, 0.0);
        }
        if globals.shape_vertex_counts.len() != num_parts {
            globals.shape_vertex_counts.resize(num_parts, 0);
        }
        for slot in globals.shape_vertices.iter_mut() {
            *slot = 0.0;
        }
        for slot in globals.shape_vertex_box_coords.iter_mut() {
            *slot = 0.0;
        }
        for slot in globals.shape_vertex_counts.iter_mut() {
            *slot = 0;
        }

        for i in 0..num_parts {
            let Some(shape_kind) = cached_parts[i].shape_kind else { continue };
            let state = &layer.parts_vec[i];
            let size_x = state.size_x();
            let size_y = state.size_y();
            if size_x <= 0.0 || size_y <= 0.0 {
                continue;
            }

            let (count, vx, vy) = crate::core::geometry::compute_shape_vertices(shape_kind, (size_x, size_y));
            if count < 3 {
                continue;
            }

            let mut effective_pivot_y = state.pivot_y();
            if coordinate_system == crate::core::SsCoordinateSystem::YDown {
                effective_pivot_y = -effective_pivot_y;
            }
            let tx = -state.pivot_x() * size_x;
            let ty = -effective_pivot_y * size_y;
            let hw = size_x * 0.5;
            let hh = size_y * 0.5;

            // Vertex deform: the 4 box-corner offsets ([LT, RT, LB, RB], in the
            // part's Y-up local space) are spread across every shape vertex by
            // bilinear interpolation over its box coordinates (fx, fy) — the same
            // weighting the renderer uses for per-corner PartColor. Offsets are
            // added in Y-up space, before the YDown negation, so the dy sign
            // matches the 4-corner deform path in `compute_local_vertices`.
            let deform = layer.raw_vertex_deform.get(i).filter(|d| d.present);

            let offset = i * stride;
            for v in 0..count {
                let fx = (vx[v] + hw) / size_x;
                let fy = (hh - vy[v]) / size_y;

                let (mut dvx, mut dvy) = (vx[v] + tx, vy[v] + ty);
                if let Some(d) = deform {
                    let (ddx, ddy) = crate::core::geometry::bilinear_box_deform(&d.dx, &d.dy, fx, fy);
                    dvx += ddx;
                    dvy += ddy;
                }

                let mut y = dvy;
                if coordinate_system == crate::core::SsCoordinateSystem::YDown {
                    y = -y;
                }
                globals.shape_vertices[offset + 2 * v] = dvx;
                globals.shape_vertices[offset + 2 * v + 1] = y;

                globals.shape_vertex_box_coords[offset + 2 * v] = fx;
                globals.shape_vertex_box_coords[offset + 2 * v + 1] = fy;
            }
            globals.shape_vertex_counts[i] = count as i32;
        }
    }

    // Frame Number getter functions

    pub fn get_frame_no(&self) -> f32 {
        self.instance.primary_source().controller.get_frame_no()
    }

    pub fn is_end_frame_reached(&self) -> bool {
        self.instance.primary_source().controller.is_end_frame_reached()
    }

    pub fn is_looped(&self) -> bool {
        self.instance.primary_source().controller.is_looped()
    }

    pub fn get_world_matrices_ptr(&self) -> *const f32 {
        self.instance.globals.world_matrices.as_ptr() as *const f32
    }

    pub fn get_world_matrices_len(&self) -> usize {
        self.instance.globals.world_matrices.len() * 16
    }

    pub fn get_local_uvs_ptr(&self) -> *const f32 {
        self.instance.globals.local_uvs.as_ptr()
    }

    pub fn get_local_uvs_len(&self) -> usize {
        self.instance.globals.local_uvs.len()
    }

    pub fn get_shape_vertices_ptr(&self) -> *const f32 {
        self.instance.globals.shape_vertices.as_ptr()
    }

    pub fn get_shape_vertices_len(&self) -> usize {
        self.instance.globals.shape_vertices.len()
    }

    pub fn get_shape_vertex_box_coords_ptr(&self) -> *const f32 {
        self.instance.globals.shape_vertex_box_coords.as_ptr()
    }

    pub fn get_shape_vertex_box_coords_len(&self) -> usize {
        self.instance.globals.shape_vertex_box_coords.len()
    }

    pub fn get_shape_vertex_counts_ptr(&self) -> *const i32 {
        self.instance.globals.shape_vertex_counts.as_ptr()
    }

    pub fn get_mesh_vertices_x_ptr(&self) -> *const f32 {
        self.instance.globals.mesh_vertices_x.as_ptr()
    }

    pub fn get_mesh_vertices_x_len(&self) -> usize {
        self.instance.globals.mesh_vertices_x.len()
    }

    pub fn get_mesh_vertices_y_ptr(&self) -> *const f32 {
        self.instance.globals.mesh_vertices_y.as_ptr()
    }

    pub fn get_mesh_vertices_y_len(&self) -> usize {
        self.instance.globals.mesh_vertices_y.len()
    }

    pub fn get_mesh_vertex_offsets_ptr(&self) -> *const u32 {
        self.instance.globals.mesh_vertex_offsets.as_ptr()
    }

    pub fn get_mesh_vertex_offsets_len(&self) -> usize {
        self.instance.globals.mesh_vertex_offsets.len()
    }

    pub fn get_mesh_uvs_ptr(&self) -> *const f32 {
        self.instance.globals.mesh_uvs.as_ptr()
    }

    pub fn get_mesh_uvs_len(&self) -> usize {
        self.instance.globals.mesh_uvs.len()
    }

    pub fn get_mesh_indices_ptr(&self) -> *const i32 {
        self.instance.globals.mesh_indices.as_ptr()
    }

    pub fn get_mesh_indices_len(&self) -> usize {
        self.instance.globals.mesh_indices.len()
    }

    pub fn get_mesh_index_offsets_ptr(&self) -> *const u32 {
        self.instance.globals.mesh_index_offsets.as_ptr()
    }

    pub fn get_mesh_index_offsets_len(&self) -> usize {
        self.instance.globals.mesh_index_offsets.len()
    }

    pub fn get_shape_vertex_counts_len(&self) -> usize {
        self.instance.globals.shape_vertex_counts.len()
    }

    pub fn get_cell_meta_ptr(&self) -> *const f32 {
        self.instance.resolved_layer.cell_meta_per_part.as_ptr()
    }

    pub fn get_cell_meta_len(&self) -> usize {
        self.instance.resolved_layer.cell_meta_per_part.len()
    }

    pub fn get_local_vertices_ptr(&self) -> *const f32 {
        self.instance.resolved_layer.local_vertices.as_ptr()
    }

    pub fn get_local_vertices_len(&self) -> usize {
        self.instance.resolved_layer.local_vertices.len()
    }

    // setting getter functions
    pub fn is_playing(&self) -> bool {
        self.instance.primary_source().controller.is_playing()
    }

    pub fn is_pausing(&self) -> bool {
        self.instance.primary_source().controller.is_pausing()
    }

    pub fn get_skip_frames(&self) -> bool {
        self.instance.primary_source().controller.get_skip_frames()
    }

    pub fn get_playback_direction(&self) -> core::playback_controller::PlaybackDirection {
        self.instance.primary_source().controller.get_playback_direction()
    }

    pub fn get_playback_style(&self) -> core::playback_controller::PlaybackStyle {
        self.instance.primary_source().controller.get_playback_style()
    }

    pub fn get_loops(&self) -> i32 {
        self.instance.primary_source().controller.get_loops()
    }

    pub fn get_remain_loops(&self) -> i32 {
        self.instance.primary_source().controller.get_remain_loops()
    }

    pub fn get_start_frame(&self) -> i32 {
        self.instance.primary_source().controller.get_start_frame()
    }

    pub fn get_end_frame(&self) -> i32 {
        self.instance.primary_source().controller.get_end_frame()
    }

    pub fn get_fps(&self) -> i32 {
        self.instance.primary_source().controller.get_fps()
    }

    // Playback setup

    pub fn set_skip_frames(&mut self, skip: bool) {
        self.instance.primary_source_mut().controller.set_skip_frames(skip);
    }

    pub fn set_loop(&mut self, loops: i32) {
        self.instance.primary_source_mut().controller.set_loops(loops);
    }

    pub fn set_frame_rate(&mut self, fps: i32) {
        self.instance.primary_source_mut().controller.set_frame_rate(fps);
    }

    pub fn set_playback_direction(&mut self, direction: core::playback_controller::PlaybackDirection, style: core::playback_controller::PlaybackStyle) {
        self.instance.primary_source_mut().controller.set_playback_direction(direction, style);
    }

    pub fn set_animation_speed(&mut self, speed_rate: f32) {
        self.instance.primary_source_mut().controller.set_animation_speed(speed_rate);
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.instance.primary_source_mut().controller.set_animation_section(start_frame, end_frame);
    }

    pub fn set_coordinate_system(&mut self, coordinate_system: crate::core::SsCoordinateSystem) {
        self.instance.coordinate_system = coordinate_system;
    }

    pub fn get_coordinate_system(&self) -> crate::core::SsCoordinateSystem {
        self.instance.coordinate_system
    }

    pub fn set_origin(&mut self, origin: crate::core::SsOrigin) {
        self.instance.origin = origin;
    }

    pub fn get_origin(&self) -> crate::core::SsOrigin {
        self.instance.origin
    }

    pub fn set_unit(&mut self, unit: crate::core::SsUnit) {
        self.instance.unit = unit;
    }

    pub fn get_unit(&self) -> crate::core::SsUnit {
        self.instance.unit
    }

    // Playback control

    pub fn update(&mut self, delta: f32) -> f32 {
        let primary_idx = self.instance.primary_idx();
        let mut primary_frame = 0.0;
        for (i, source) in self.instance.sources.iter_mut().enumerate() {
            if !source.has_real_animation() {
                continue;
            }
            let frame = source.controller.update(delta);
            if i == primary_idx {
                primary_frame = frame;
            }
        }

        self.advance_crossfade(delta);
        self.collect_events();

        primary_frame
    }

    /// Advance an in-flight crossfade by `delta`. Linearly rebalances the
    /// `from`/`to` source weights, and on completion promotes `to` to primary
    /// and deactivates `from`. No-op when no crossfade is active. Safe against
    /// `from`/`to` having been removed externally — the slot's `-1` sentinel
    /// causes the crossfade to be cancelled.
    fn advance_crossfade(&mut self, delta: f32) {
        let Some(mut cf) = self.instance.crossfade else { return };
        let from_idx = cf.from as usize;
        let to_idx = cf.to as usize;
        if self.instance.sources.get(from_idx).is_none_or(|s| !s.has_real_animation()) || self.instance.sources.get(to_idx).is_none_or(|s| s.animation_idx == -1) {
            self.instance.crossfade = None;
            return;
        }
        cf.elapsed += delta;
        let t = (cf.elapsed / cf.duration).clamp(0.0, 1.0);
        self.instance.sources[from_idx].weight = 1.0 - t;
        self.instance.sources[to_idx].weight = t;
        if cf.elapsed >= cf.duration {
            // Completion: only swap primary if `to` is a real animation. An
            // empty `to` (fade-out target) keeps the original primary so the
            // discrete-attribute source stays valid; the caller can call
            // `set_primary` or `remove_blend_source` next to finalize the
            // fade-out.
            if self.instance.sources[to_idx].has_real_animation() {
                self.instance.primary = cf.to;
                self.instance.sources[from_idx].animation_idx = -1;
                self.instance.sources[from_idx].weight = 0.0;
                self.instance.sources[from_idx].controller.reset_setup();
            }
            self.instance.crossfade = None;
        } else {
            self.instance.crossfade = Some(cf);
        }
    }

    fn collect_events(&mut self) {
        self.instance.collected_event_indices.clear();
        self.instance.collected_event_source_handles.clear();
        let Some(resource) = self.eval.resource else {
            return;
        };
        let primary_idx = self.instance.primary_idx();
        let num_sources = self.instance.sources.len();

        for i in 0..num_sources {
            let source = &self.instance.sources[i];
            if !source.has_real_animation() {
                continue;
            }
            let is_primary = i == primary_idx;
            if !is_primary && source.weight < source.event_threshold {
                continue;
            }
            let metadata = &resource.animations[source.animation_idx as usize];
            let Some(events_vec) = metadata.events else { continue };
            let source_handle = i as u32;

            // Disjoint-field borrow: `collected_event_*` and `sources[i].controller`
            // are independent fields of `self.instance`, so we can hold an
            // immutable view of the controller while pushing to the collected
            // vecs. Re-acquire the controller ref each frame iteration to
            // keep the borrow lifetime tight.
            let passed_frames_len = self.instance.sources[i].controller.get_passed_frames().len();
            for f_idx in 0..passed_frames_len {
                let frame_no = self.instance.sources[i].controller.get_passed_frames()[f_idx];
                if frame_no < 0 || frame_no > u16::MAX as i32 {
                    continue;
                }
                let target_frame = frame_no as u16;
                let mut low = 0;
                let mut high = events_vec.len();
                while low < high {
                    let mid = low + (high - low) / 2;
                    match events_vec.get(mid).frame_index().cmp(&target_frame) {
                        ::core::cmp::Ordering::Equal => {
                            self.instance.collected_event_indices.push(mid as i32);
                            self.instance.collected_event_source_handles.push(source_handle);
                            break;
                        }
                        ::core::cmp::Ordering::Less => low = mid + 1,
                        ::core::cmp::Ordering::Greater => high = mid,
                    }
                }
            }
        }
    }

    pub fn get_passed_event_indices(&self) -> &[i32] {
        &self.instance.collected_event_indices
    }

    /// Parallel to [`get_passed_event_indices`]: source handle that fired
    /// each collected event. With every source at the default event threshold
    /// (`1.0`), entries are uniformly `primary_handle()`. Diverges once
    /// secondaries opt-in to event firing via [`set_source_event_threshold`].
    pub fn get_passed_event_source_handles(&self) -> &[u32] {
        &self.instance.collected_event_source_handles
    }

    /// Resolve the most-recent `EventInstance` whose `frame_index <= current frame`
    /// for `slot_index` on the *currently selected animation*. Returns
    /// `synthetic_default()` when no event has fired yet (SS6 default semantics).
    /// Designed to live behind the `ss_runtime_get_active_event_instance` FFI.
    pub fn get_active_event_instance(&self, slot_index: u32) -> crate::types::ss_event_instance_info {
        let mut info = crate::types::ss_event_instance_info::synthetic_default();
        let Some(resource) = self.eval.resource else { return info };
        if self.instance.primary_source().animation_idx < 0 {
            return info;
        }
        let metadata = &resource.animations[self.instance.primary_source().animation_idx as usize];
        let Some(events) = metadata.events else { return info };

        let frame_no = self.instance.primary_source().controller.get_frame_no();
        let frame_int = if frame_no < 0.0 {
            -1
        } else if frame_no >= u16::MAX as f32 {
            u16::MAX as i32
        } else {
            frame_no as i32
        };

        // Forward scan: events are sorted by frame_index. Later matches override
        // earlier ones, so the loop naturally lands on the most-recent
        // EventInstance for `slot_index`.
        for i in 0..events.len() {
            let epf = events.get(i);
            if (epf.frame_index() as i32) > frame_int {
                break;
            }
            let Some(instances) = epf.instances() else { continue };
            for j in 0..instances.len() {
                let ev = instances.get(j);
                if ev.part_index() as u32 != slot_index {
                    continue;
                }
                let attr = ev.value();
                info = crate::types::ss_event_instance_info {
                    event_frame: epf.frame_index() as i32,
                    start_label_hash: attr.start_label_hash(),
                    start_offset: attr.start_offset(),
                    end_label_hash: attr.end_label_hash(),
                    end_offset: attr.end_offset(),
                    loop_num: attr.loop_num(),
                    speed: attr.speed(),
                    independent: attr.independent(),
                    reverse: attr.reverse(),
                    pingpong: attr.pingpong(),
                    is_synthetic_default: false,
                };
            }
        }
        info
    }

    /// Resolve the most-recent `EventEffect` whose `frame_index <= current frame`
    /// for `slot_index` on the *currently selected animation*. When no event has
    /// fired yet, falls back to `InitialEvents.effect` for that slot (still flagged
    /// `is_synthetic_default = true`); if neither exists returns full SS6 defaults.
    /// Designed to live behind the `ss_runtime_get_active_effect_event` FFI.
    pub fn get_active_effect_event(&self, slot_index: u32) -> crate::types::ss_effect_event_info {
        let mut info = crate::types::ss_effect_event_info::synthetic_default();
        let Some(resource) = self.eval.resource else { return info };
        if self.instance.primary_source().animation_idx < 0 {
            return info;
        }
        let metadata = &resource.animations[self.instance.primary_source().animation_idx as usize];

        // Fallback first: InitialEvents.effect for this slot. Real EventEffect
        // matches override below; the synthetic-default flag stays `true` for
        // initial-only state so callers don't treat it as an event edge.
        if let Some(inits) = metadata.initial_events
            && (slot_index as usize) < inits.len()
        {
            let entry = inits.get(slot_index as usize);
            if let Some(eff) = entry.effect() {
                info.event_frame = 0;
                info.start_time = eff.start_time();
                let s = eff.speed();
                info.speed = if s == 0.0 { 1.0 } else { s };
                info.independent = eff.independent();
                info.is_synthetic_default = true;
            }
        }

        let Some(events) = metadata.events else { return info };

        let frame_no = self.instance.primary_source().controller.get_frame_no();
        let frame_int = if frame_no < 0.0 {
            -1
        } else if frame_no >= u16::MAX as f32 {
            u16::MAX as i32
        } else {
            frame_no as i32
        };

        for i in 0..events.len() {
            let epf = events.get(i);
            if (epf.frame_index() as i32) > frame_int {
                break;
            }
            let Some(effects) = epf.effects() else { continue };
            for j in 0..effects.len() {
                let ev = effects.get(j);
                if ev.part_index() as u32 != slot_index {
                    continue;
                }
                let attr = ev.value();
                let s = attr.speed();
                info = crate::types::ss_effect_event_info { event_frame: epf.frame_index() as i32, start_time: attr.start_time(), speed: if s == 0.0 { 1.0 } else { s }, independent: attr.independent(), is_synthetic_default: false };
            }
        }
        info
    }

    /// Look up `label_hash` in the currently selected animation's `labels`
    /// table and return its `time`. Returns `fallback` when `label_hash == 0`
    /// (caller convention for "unset"), no animation is bound, or the hash
    /// is not found. Linear scan — labels per animation are typically <50.
    pub fn resolve_label_time(&self, label_hash: u32, fallback: i32) -> i32 {
        if label_hash == 0 {
            return fallback;
        }
        let Some(resource) = self.eval.resource else { return fallback };
        if self.instance.primary_source().animation_idx < 0 {
            return fallback;
        }
        let metadata = &resource.animations[self.instance.primary_source().animation_idx as usize];
        let Some(labels) = metadata.labels else { return fallback };
        for i in 0..labels.len() {
            let label = labels.get(i);
            if label.name_hash() == label_hash {
                return label.time();
            }
        }
        fallback
    }

    pub fn play(&mut self) {
        self.instance.primary_source_mut().controller.play();
    }

    pub fn play_with_start_frame(&mut self, frame_no: f32) {
        self.instance.primary_source_mut().controller.play_with_start_frame(frame_no);
    }

    pub fn pause(&mut self) {
        self.instance.primary_source_mut().controller.pause();
    }

    pub fn stop(&mut self) {
        self.instance.primary_source_mut().controller.stop();
    }

    pub fn set_frame_no(&mut self, frame: f32) {
        self.instance.primary_source_mut().controller.set_frame_no(frame);
    }

    /// Sets the playback position based on `diff` (frames elapsed since
    /// playback started, with per-instance speed already applied), applying
    /// the configured section / loops / style / direction. Used by parent
    /// Players to drive child Instance / Effect Players deterministically
    /// across reverse, seek, and frame-skip operations.
    pub fn set_frame_relative(&mut self, diff: f32) {
        self.instance.primary_source_mut().controller.set_frame_relative(diff);
    }

    pub fn next_frame(&mut self) {
        self.instance.primary_source_mut().controller.next_frame();
    }

    pub fn prev_frame(&mut self) {
        self.instance.primary_source_mut().controller.prev_frame();
    }
}

pub struct EffectContext<'a> {
    pub eval: crate::core::effect::instance::EffectEvaluationContext<'a>,
    pub instance: crate::core::effect::instance::EffectInstanceState,
    seed: u32,
}

impl<'a> EffectContext<'a> {
    pub fn new(resource: &'a AnimationResource<'a>, effect_name_hash: u32, seed: u32) -> Result<Self, error::SSRuntimeError> {
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(resource.data) };
        let Some(effects) = ssab.effects() else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };
        let Some(effect) = crate::core::ssab_lookup::find_effect(&effects, effect_name_hash) else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };
        // The effect was located by name_hash, but its data is malformed
        // (e.g. missing required `effect_node` table).
        let Some((eval, instance)) = crate::core::effect::instance::EffectEvaluationContext::from_ssab(&effect, seed) else {
            return Err(error::SSRuntimeError::MalformedEffect);
        };
        Ok(Self { eval, instance, seed })
    }

    pub fn get_seed(&self) -> u32 {
        self.seed
    }

    pub fn reset(&mut self) {
        self.instance.reset();
    }

    pub fn set_loop(&mut self, loop_flag: bool) {
        self.instance.set_loop(loop_flag);
    }

    pub fn get_lifetime(&self) -> i32 {
        self.eval.effect_time_length
    }

    pub fn is_infinite(&self) -> bool {
        self.eval.is_infinite
    }

    pub fn update(&mut self, frame: f32) {
        self.instance.update(&self.eval, frame);
    }

    pub fn get_draw_commands(&mut self) -> (&[crate::core::effect::instance::EmitterDrawCommand], &[crate::core::effect::types::ParticleDrawState]) {
        self.instance.get_draw_commands(&self.eval)
    }
}

/// High-level effect slot: wraps `EffectContext` with the per-slot lifecycle
/// state that previously lived in each Player's effect-slot table. Callers
/// resolve the active effect event externally (via `Context::get_active_effect_event`)
/// and feed it to `step()` along with frame timing — `step` owns transition
/// detection, accumulator, dead-effect skip, simulator update, and state
/// serialization in one call.
///
/// The cross-language contract is identical in Rust / C / Web: 1) construct
/// once per effect part, 2) call `step()` per frame with the resolved event
/// and timing, 3) read `state_buf` only when `visible == true` and only
/// before the next `step()`.
/// Resolved emitter render resources, cached on first encounter and reused
/// across frames. Identified by (cellmap_hash, cell_hash, blend_in) — the
/// effect simulator may reorder emitters when siblings go silent, so the
/// cache is keyed by content not slot index.
struct EmitterResourceEntry {
    cellmap_hash: u32,
    cell_hash: u32,
    blend_in: i32,
    resolved_ok: bool,
    blend_byte: u8,
    quad_origin_x: f32,
    quad_origin_y: f32,
    sw: f32,
    sh: f32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
}

// Lifted out of `build_draw_plan` so the row buffer can be a cached field.
#[derive(Default, Clone, Copy)]
struct ResolvedEmitter {
    ok: bool,
    blend_byte: u8,
    quad_origin_x: f32,
    quad_origin_y: f32,
    sw: f32,
    sh: f32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    cellmap_hash: u32,
}

pub struct EffectSlot<'a> {
    ctx: EffectContext<'a>,
    // SSAB root, kept for cellmap lookup during draw-plan build. Borrows from
    // the same resource as `ctx`; lifetime tied to the surrounding `'a`.
    ssab: ssab::ssab::ss::format::SsAnimeBinary<'a>,
    // FlatBufferBuilder owned by the slot for `EffectDrawPlan` output.
    // Reset each `step`; buffer pointer stays valid until the next call.
    fb: flatbuffers::FlatBufferBuilder<'a>,
    // Resolved emitter resources keyed by (cellmap_hash, cell_hash, blend_in).
    emitter_resource_cache: alloc::vec::Vec<EmitterResourceEntry>,
    // Per-frame scratch for `build_draw_plan`: clear()+reserve()'d each tick
    // so capacities stabilise at the peak and steady-state allocation is zero.
    scratch_cmd_keys: alloc::vec::Vec<(u32, u32, i32, usize, usize)>,
    scratch_entries: alloc::vec::Vec<(usize, usize, usize)>,
    scratch_verts: alloc::vec::Vec<f32>,
    scratch_uvs: alloc::vec::Vec<f32>,
    scratch_colors: alloc::vec::Vec<f32>,
    scratch_indices: alloc::vec::Vec<i32>,
    scratch_resolved: alloc::vec::Vec<ResolvedEmitter>,
    scratch_command_meta: alloc::vec::Vec<(u32, u8, u32, u32)>,
    scratch_cmds: alloc::vec::Vec<crate::runtime::effect_draw_plan::ss::runtime::EffectDrawCommand>,
    // Cached at create time — never changes for a given effect resource.
    lifetime: i32,
    is_infinite: bool,
    // Transition detection: any change in (event_frame, is_synthetic, independent)
    // triggers reset/set_loop and re-seeds `accumulated_time`.
    last_event_frame: i32,
    last_is_synthetic: bool,
    last_independent: bool,
    // Speed of the last real EventEffect — reused while the implicit frame-0
    // key is being ignored (independent playback across a parent loop).
    last_speed: f32,
    // Independent-mode time accumulator (frame units, includes start_time bias).
    accumulated_time: f32,
}

impl<'a> EffectSlot<'a> {
    pub fn new(resource: &'a AnimationResource<'a>, effect_name_hash: u32, seed: u32) -> Result<Self, error::SSRuntimeError> {
        let ctx = EffectContext::new(resource, effect_name_hash, seed)?;
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(resource.data) };
        let lifetime = ctx.get_lifetime();
        let is_infinite = ctx.is_infinite();
        Ok(Self {
            ctx,
            ssab,
            fb: flatbuffers::FlatBufferBuilder::new(),
            emitter_resource_cache: alloc::vec::Vec::new(),
            scratch_cmd_keys: alloc::vec::Vec::new(),
            scratch_entries: alloc::vec::Vec::new(),
            scratch_verts: alloc::vec::Vec::new(),
            scratch_uvs: alloc::vec::Vec::new(),
            scratch_colors: alloc::vec::Vec::new(),
            scratch_indices: alloc::vec::Vec::new(),
            scratch_resolved: alloc::vec::Vec::new(),
            scratch_command_meta: alloc::vec::Vec::new(),
            scratch_cmds: alloc::vec::Vec::new(),
            lifetime,
            is_infinite,
            last_event_frame: -1,
            last_is_synthetic: false,
            last_independent: false,
            last_speed: 1.0,
            accumulated_time: 0.0,
        })
    }

    pub fn get_seed(&self) -> u32 {
        self.ctx.get_seed()
    }

    /// Whether the slot is currently running independently of the parent's
    /// frame. Reflects the most recently seen `EventEffect.independent`;
    /// used by callers to decide whether to schedule continuous tick updates.
    pub fn is_independent(&self) -> bool {
        self.last_independent
    }

    /// Resolve `(cellmap_hash, cell_hash, blend_in)` to render geometry/UV,
    /// caching the result in `emitter_resource_cache`. Linear search — typical
    /// effects have 1-3 unique tuples and the cache survives across frames.
    /// Free-standing form so callers can split-borrow `cache` and `ssab`
    /// independently of other `self` fields (the simulator output borrow lives
    /// on `self.ctx.instance` simultaneously inside `build_draw_plan`).
    fn ensure_emitter_resource_in(cache: &mut alloc::vec::Vec<EmitterResourceEntry>, ssab: &ssab::ssab::ss::format::SsAnimeBinary<'a>, cellmap_hash: u32, cell_hash: u32, blend_in: i32) -> usize {
        if let Some(idx) = cache.iter().position(|c| c.cellmap_hash == cellmap_hash && c.cell_hash == cell_hash && c.blend_in == blend_in) {
            return idx;
        }
        let mut entry = EmitterResourceEntry { cellmap_hash, cell_hash, blend_in, resolved_ok: false, blend_byte: 0, quad_origin_x: 0.0, quad_origin_y: 0.0, sw: 0.0, sh: 0.0, u0: 0.0, v0: 0.0, u1: 0.0, v1: 0.0 };

        if cellmap_hash != 0
            && let Some(cellmaps) = ssab.cellmaps()
            && let Some(cellmap) = crate::core::ssab_lookup::find_cellmap(&cellmaps, cellmap_hash)
            && let Some(cells) = cellmap.cells()
            && let Some(cell) = crate::core::ssab_lookup::find_cell(&cells, cell_hash)
        {
            // SS6-style blend mapping: 0=Mix, 1=Add. Anything else falls back to Mix.
            entry.blend_byte = if blend_in == 1 { 1 } else { 0 };

            let rect = cell.rectangle();
            let (src_x, src_y, sw, sh) = (rect.x1(), rect.y1(), rect.x2(), rect.y2());
            // Pivot Y is negated to match Godot's prior emit_effect_slot behavior;
            // the Y-flip logic on the per-particle pass handles the rest of the
            // axis convention difference.
            let p = cell.pivot();
            let (pivot_x, pivot_y) = (p.v1(), -p.v2());
            entry.sw = sw;
            entry.sh = sh;
            entry.quad_origin_x = -sw * 0.5 + pivot_x * sw;
            entry.quad_origin_y = -sh * 0.5 + pivot_y * sh;

            // UV uses cellmap.size_original — the texture dimensions at SSAB build time.
            // Atlas / scaled-texture pipelines must remap UVs engine-side.
            let s = cellmap.size_original();
            let (tex_w, tex_h) = (s.v1(), s.v2());
            let tex_w = if tex_w > 0.0 { tex_w } else { 1.0 };
            let tex_h = if tex_h > 0.0 { tex_h } else { 1.0 };
            entry.u0 = src_x / tex_w;
            entry.v0 = src_y / tex_h;
            entry.u1 = (src_x + sw) / tex_w;
            entry.v1 = (src_y + sh) / tex_h;
            entry.resolved_ok = true;
        }

        let idx = cache.len();
        cache.push(entry);
        idx
    }

    pub fn step(&mut self, ev: crate::types::ss_effect_event_info, parent_frame: f32, delta_seconds: f32, fps: f32, parent_looped: bool, y_flip: bool, vert_stride: u8) -> crate::types::ss_effect_step_result {
        // While the slot is playing independently, the SS7 implicit frame-0
        // playback key (synthetic default) is ignored — the independent effect
        // keeps running across the parent's loop instead of being reset to it.
        // Only a real authored EventEffect re-transitions the slot.
        let ignore_implicit = self.last_independent && ev.is_synthetic_default;

        // Parent loop edge re-arms transition detection: the same EventEffect
        // identity (event_frame + is_synthetic) re-fires on the next playthrough,
        // and without this reset the equality check below would treat the loop
        // edge as a no-op (effect frozen at end of previous cycle). Skipped when
        // the implicit key is being ignored, so independent playback persists.
        if parent_looped && !ignore_implicit {
            self.last_event_frame = -1;
            self.last_is_synthetic = false;
        }

        // When ignoring the implicit key, keep stepping with the last real
        // event's identity / mode / speed rather than the synthetic default's.
        let event_frame = if ignore_implicit { self.last_event_frame } else { ev.event_frame };
        let start_time = ev.start_time;
        let speed = if ignore_implicit { self.last_speed } else { ev.speed };
        let independent = if ignore_implicit { true } else { ev.independent };
        let is_synthetic = ev.is_synthetic_default;

        let transitioned = !ignore_implicit && (self.last_event_frame != event_frame || self.last_is_synthetic != is_synthetic || self.last_independent != independent);

        if transitioned {
            self.ctx.reset();
            // `independent` controls the *clock* (own delta-time vs parent-synced
            // frame), not whether the effect loops. Looping is a property of the
            // effect resource itself: only genuinely infinite effects loop; finite
            // ones play once and are culled by the dead-effect early-return below
            // (which now applies regardless of `independent`). The earlier SS6
            // carry-over `set_loop(independent)` wrongly looped finite independent
            // effects forever, so they never died at their lifetime.
            self.ctx.set_loop(self.is_infinite);
            self.last_event_frame = event_frame;
            self.last_is_synthetic = is_synthetic;
            self.last_independent = independent;
            self.last_speed = speed;
            self.accumulated_time = start_time as f32;
        }

        let relative_frame: f32 = if independent {
            // Skip the first tick after transition — `play()` snaps to start_time
            // and stepping on the same tick would visibly overshoot frame 1.
            if !transitioned && delta_seconds > 0.0 {
                let delta_frames = delta_seconds * fps;
                self.accumulated_time += delta_frames * speed;
            }
            self.accumulated_time
        } else {
            let elapsed = parent_frame - event_frame as f32;
            if elapsed < 0.0 {
                return crate::types::ss_effect_step_result::not_visible();
            }
            elapsed * speed + start_time as f32
        };

        if relative_frame < 0.0 {
            return crate::types::ss_effect_step_result::not_visible();
        }

        // Dead-effect skip: a finite effect past its last particle's death is
        // gone — independent or not. Independent only changes how `relative_frame`
        // is advanced (own accumulator vs parent-synced); both clocks must still
        // honour the effect's finite lifetime. (Previously `!independent` excluded
        // independent effects here, so finite ones never disappeared.)
        if !self.is_infinite && self.lifetime > 0 && relative_frame > self.lifetime as f32 {
            return crate::types::ss_effect_step_result::not_visible();
        }

        self.ctx.update(relative_frame);
        let buf = self.build_draw_plan(y_flip, vert_stride);
        if buf.is_empty() { crate::types::ss_effect_step_result::not_visible() } else { crate::types::ss_effect_step_result { visible: true, draw_plan_buf: buf.as_ptr(), draw_plan_len: buf.len() } }
    }

    /// Build the `EffectDrawPlan` FlatBuffer from the simulator's current
    /// draw_commands + draw_particles. Resolves per-emitter resources via
    /// `ensure_emitter_resource` (cached) and writes verts/uvs/colors/indices
    /// concatenated for all emitters in one pass. Returns the FlatBuffer slice;
    /// valid until the next call on this slot.
    fn build_draw_plan(&mut self, y_flip: bool, vert_stride: u8) -> &[u8] {
        // 180/π — avoid `core::f32::consts::PI` (the path is shadowed in this
        // crate's no_std + libc_alloc build).
        const DEG_TO_RAD: f32 = 0.017_453_292;
        let stride = if vert_stride == 3 { 3usize } else { 2usize };
        let y_sign: f32 = if y_flip { -1.0 } else { 1.0 };

        // Narrow the borrow on `self.ctx.get_draw_commands` so other self
        // fields stay touchable afterwards.
        self.scratch_cmd_keys.clear();
        {
            let (commands, _particles) = self.ctx.get_draw_commands();
            self.scratch_cmd_keys.reserve(commands.len());
            for c in commands.iter() {
                self.scratch_cmd_keys.push((c.cell_map_name_hash, c.cell_name_hash, c.blend_type, c.particle_start, c.particle_end));
            }
        }

        // Phase 1: resolve resources via split-borrow on disjoint self fields.
        self.scratch_entries.clear();
        self.scratch_entries.reserve(self.scratch_cmd_keys.len());
        {
            let cache = &mut self.emitter_resource_cache;
            let ssab = &self.ssab;
            for &(cmh, ch, bi, ps, pe) in self.scratch_cmd_keys.iter() {
                self.scratch_entries.push((Self::ensure_emitter_resource_in(cache, ssab, cmh, ch, bi), ps, pe));
            }
        }

        let total_quads: usize = self.scratch_entries.iter().map(|(_, s, e)| e - s).sum();
        if total_quads == 0 {
            return &[];
        }

        self.scratch_verts.clear();
        self.scratch_uvs.clear();
        self.scratch_colors.clear();
        self.scratch_indices.clear();
        self.scratch_verts.reserve(total_quads * 4 * stride);
        self.scratch_uvs.reserve(total_quads * 4 * 2);
        self.scratch_colors.reserve(total_quads * 4 * 4);
        self.scratch_indices.reserve(total_quads * 6);

        // Snapshot resolved entries so self.emitter_resource_cache is free
        // to be re-borrowed when we later write into self.fb.
        self.scratch_resolved.clear();
        self.scratch_resolved.reserve(self.scratch_entries.len());
        for (i, _, _) in self.scratch_entries.iter() {
            let e = &self.emitter_resource_cache[*i];
            self.scratch_resolved.push(ResolvedEmitter {
                ok: e.resolved_ok,
                blend_byte: e.blend_byte,
                quad_origin_x: e.quad_origin_x,
                quad_origin_y: e.quad_origin_y,
                sw: e.sw,
                sh: e.sh,
                u0: e.u0,
                v0: e.v0,
                u1: e.u1,
                v1: e.v1,
                cellmap_hash: e.cellmap_hash,
            });
        }

        self.scratch_command_meta.clear();
        self.scratch_command_meta.reserve(self.scratch_entries.len());

        // Particles live on `self.ctx.instance.draw_particles`; borrow inline.
        let particles = &self.ctx.instance.draw_particles;

        for (cmd_idx, (_, p_start, p_end)) in self.scratch_entries.iter().enumerate() {
            let r = &self.scratch_resolved[cmd_idx];
            let particle_offset = (self.scratch_verts.len() / (4 * stride)) as u32;
            let particle_count = (p_end - p_start) as u32;

            if !r.ok {
                self.scratch_command_meta.push((r.cellmap_hash, r.blend_byte, 0, particle_offset));
                continue;
            }

            let lx = [r.quad_origin_x, r.quad_origin_x + r.sw, r.quad_origin_x, r.quad_origin_x + r.sw];
            let ly = [r.quad_origin_y, r.quad_origin_y, r.quad_origin_y + r.sh, r.quad_origin_y + r.sh];
            let uvs_corner = [r.u0, r.v0, r.u1, r.v0, r.u0, r.v1, r.u1, r.v1];

            for (i, p) in particles[*p_start..*p_end].iter().enumerate() {
                let angle = y_sign * (p.rotation * DEG_TO_RAD + p.direction);
                let c = libm::cosf(angle);
                let s = libm::sinf(angle);
                let py = y_sign * p.y;

                for v in 0..4 {
                    let xx = lx[v] * p.scale_x;
                    let yy = ly[v] * p.scale_y;
                    let wx = c * xx - s * yy + p.x;
                    let wy = s * xx + c * yy + py;
                    self.scratch_verts.push(wx);
                    self.scratch_verts.push(wy);
                    if stride == 3 {
                        self.scratch_verts.push(0.0);
                    }
                }
                self.scratch_uvs.extend_from_slice(&uvs_corner);
                let r_f = p.color.r as f32 / 255.0;
                let g_f = p.color.g as f32 / 255.0;
                let b_f = p.color.b as f32 / 255.0;
                let a_f = p.color.a as f32 / 255.0;
                for _ in 0..4 {
                    self.scratch_colors.extend_from_slice(&[r_f, g_f, b_f, a_f]);
                }
                // Quad-local indices: command consumers slice [off*6 .. (off+qc)*6]
                // and pair it with the verts slice [off*4*stride .. (off+qc)*4*stride],
                // so values must be 0..qc*4 within the command, not global.
                let v = (i * 4) as i32;
                self.scratch_indices.extend_from_slice(&[v, v + 1, v + 2, v + 2, v + 1, v + 3]);
            }

            self.scratch_command_meta.push((r.cellmap_hash, r.blend_byte, particle_count, particle_offset));
        }

        // Commands are inlined fixed-size structs; collect as POD then create
        // the vector in one shot (no per-element vtable/offset overhead).
        self.fb.reset();
        self.scratch_cmds.clear();
        self.scratch_cmds.reserve(self.scratch_command_meta.len());
        for &(hash, blend, qc, off) in self.scratch_command_meta.iter() {
            self.scratch_cmds.push(crate::runtime::effect_draw_plan::ss::runtime::EffectDrawCommand::new(hash, qc, off, blend));
        }
        let commands_off = self.fb.create_vector(&self.scratch_cmds);
        let verts_off = self.fb.create_vector(&self.scratch_verts);
        let uvs_off = self.fb.create_vector(&self.scratch_uvs);
        let colors_off = self.fb.create_vector(&self.scratch_colors);
        let indices_off = self.fb.create_vector(&self.scratch_indices);

        let mut plan_b = crate::runtime::effect_draw_plan::ss::runtime::EffectDrawPlanBuilder::new(&mut self.fb);
        plan_b.add_commands(commands_off);
        plan_b.add_vert_stride(stride as u8);
        plan_b.add_verts(verts_off);
        plan_b.add_uvs(uvs_off);
        plan_b.add_colors(colors_off);
        plan_b.add_indices(indices_off);
        let root = plan_b.finish();
        self.fb.finish_minimal(root);
        self.fb.finished_data()
    }
}

/// High-level instance-child slot: tracks the per-slot transition state
/// previously kept in each Player's instance-children table. `step()` resolves
/// labels against the *child* runtime context, configures the child's
/// playback (animation section, loop, direction), and advances the child's
/// frame — leaving only the C++/host-side `play()` event sink and the
/// redraw call to the caller.
pub struct InstanceSlot {
    last_event_frame: i32,
    last_is_synthetic: bool,
    last_independent: bool,
    // Speed of the last real EventInstance — reused while the implicit frame-0
    // key is being ignored (independent playback across a parent loop).
    last_speed: f32,
}

impl Default for InstanceSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceSlot {
    pub const fn new() -> Self {
        Self { last_event_frame: -1, last_is_synthetic: false, last_independent: false, last_speed: 1.0 }
    }

    /// Whether the slot is currently driving its child independently of the
    /// parent's frame. Reflects the most recently seen `EventInstance.independent`.
    pub fn is_independent(&self) -> bool {
        self.last_independent
    }

    /// Drive the slot for one tick. Configures `child_ctx` on transition
    /// edges and advances its frame either by parent-relative seek or by
    /// delta-time update depending on `independent`.
    pub fn step(&mut self, ev: crate::types::ss_event_instance_info, child_ctx: &mut Context<'_>, previous_child_frame: f32, parent_frame: f32, delta_seconds: f32, parent_looped: bool) -> crate::types::ss_instance_step_result {
        // While the slot drives its child independently, the SS7 implicit
        // frame-0 playback key (synthetic default) is ignored — the independent
        // child keeps running across the parent's loop instead of being
        // re-seeked to it. Only a real authored EventInstance re-transitions it.
        let ignore_implicit = self.last_independent && ev.is_synthetic_default;

        if parent_looped && !ignore_implicit {
            self.last_event_frame = -1;
            self.last_is_synthetic = false;
        }

        let transitioned = !ignore_implicit && (self.last_event_frame == -1 || self.last_event_frame != ev.event_frame || self.last_is_synthetic != ev.is_synthetic_default);

        if transitioned {
            // `default_end` mirrors the original C++ fallback: `getTotalFrames()-1`,
            // i.e. (end - start). Labels resolve against absolute frame time, so
            // the fallback is only used when label_hash == 0 — preserving the
            // pre-refactor behavior.
            let child_total = child_ctx.get_end_frame() - child_ctx.get_start_frame() + 1;
            let default_end = if child_total > 0 { child_total - 1 } else { 0 };
            let start_label_time = child_ctx.resolve_label_time(ev.start_label_hash, 0);
            let end_label_time = child_ctx.resolve_label_time(ev.end_label_hash, default_end);
            let start_frame = start_label_time + ev.start_offset;
            let mut end_frame = end_label_time + ev.end_offset;
            if end_frame < start_frame {
                end_frame = start_frame;
            }

            let dir = if ev.reverse { crate::core::playback_controller::PlaybackDirection::Backward } else { crate::core::playback_controller::PlaybackDirection::Forward };
            let style = if ev.pingpong { crate::core::playback_controller::PlaybackStyle::PingPong } else { crate::core::playback_controller::PlaybackStyle::Normal };
            child_ctx.set_playback_direction(dir, style);
            child_ctx.set_animation_section(start_frame, end_frame);
            child_ctx.set_loop(ev.loop_num);

            self.last_event_frame = ev.event_frame;
            self.last_is_synthetic = ev.is_synthetic_default;
            self.last_speed = ev.speed;
        }

        // When ignoring the implicit key, keep the child's playback config and
        // step it with the last real event's independent mode + speed.
        let independent = if ignore_implicit { true } else { ev.independent };
        let speed = if ignore_implicit { self.last_speed } else { ev.speed };

        // Branch on `independent`. Synced children are seeked deterministically
        // from the parent's frame each tick — same `parent_frame` always yields
        // the same `child_frame`. Independent children own their own controller
        // time: forward `delta_seconds` to `update`, except on the transition
        // tick itself where `play()` will snap to start_frame (any further step
        // here would visibly skip the first frame).
        let child_frame_no = if independent {
            if delta_seconds > 0.0 && !transitioned {
                let d_ms = delta_seconds * 1000.0 * speed;
                child_ctx.update(d_ms)
            } else {
                child_ctx.get_frame_no()
            }
        } else {
            let diff = (parent_frame - ev.event_frame as f32) * speed;
            child_ctx.set_frame_relative(diff);
            child_ctx.get_frame_no()
        };

        let child_looped = independent && delta_seconds > 0.0 && child_frame_no < previous_child_frame && !transitioned;

        self.last_independent = independent;

        crate::types::ss_instance_step_result { visible: true, transitioned, independent, child_frame_no, child_looped }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use std::path::PathBuf;

    use super::*;

    static TARGET_SSPJ_FILES: [&str; 5] = ["tests/overall/overall.sspj", "tests/ParticleEffect/ParticleEffect.sspj", "tests/allAttributeV7/allAttributeV7.sspj", "tests/allPartsV7/allPartsV7.sspj", "tests/Ringo/Ringo.sspj"];

    static TARGET_SSPAB_DIRS: [&str; 5] = ["tests/overall/overall", "tests/ParticleEffect/ParticleEffect", "tests/allAttributeV7/allAttributeV7", "tests/allPartsV7/allPartsV7", "tests/Ringo/Ringo"];

    fn get_test_base_path() -> PathBuf {
        std::env::current_dir().expect("Failed to get the current directory").ancestors().nth(2).expect("Failed to get the base directory").to_path_buf()
    }

    fn prepare() {
        // Regenerate fixtures exactly once per test run, always from source, so cached `.ssab`
        // files can never go stale against the current ssab schema (a stale cache previously caused
        // flatbuffers to panic on out-of-range reads). `Once` serializes regeneration so the many
        // parallel `prepare()` callers don't race on a half-written directory.
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            TARGET_SSPAB_DIRS.iter().enumerate().for_each(|(i, &dir)| {
                let ssab_dir = get_test_base_path().join(dir);
                let _ = std::fs::remove_dir_all(&ssab_dir);
                let sspj = get_test_base_path().join(TARGET_SSPJ_FILES[i]);
                let config = ssconverter::ConverterConfig::default();
                let _ = ssconverter::convert_sspj_to_ssab_files(sspj.as_path(), &config, &|_msg: &str| {});
            });
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
                ssab.parts().iter().enumerate().for_each(|(idx, part)| {
                    println!("    id: {}, parent_id: {}, type: {}, name: {}", idx, part.parent_index(), part.part_type_type().variant_name().unwrap(), part.name());
                });

                println!();
            }
        });
    }

    #[test]
    fn evaluate_animation_at_frame() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
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
            let parts_meta = ssab.parts();
            pos_xyz_idx = parts_meta.iter().position(|p| p.name() == "POS_XYZ").expect("Part POS_XYZ not found") as i16;
        }

        // Test frame 0
        {
            let frame_data_buf = context.get_frame_data(0.0).expect("Failed to get frame data at frame 0");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts();
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 0.0);
            assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }

        // Test frame 5 (interpolating)
        {
            let frame_data_buf = context.get_frame_data(5.0).expect("Failed to get frame data at frame 5");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts();
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 16.0);
            assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }

        // Test frame 10
        {
            let frame_data_buf = context.get_frame_data(10.0).expect("Failed to get frame data at frame 10");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts();
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 32.0);
            assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }

        // Test frame 11 (past the last keyframe — value held at last key).
        // The flag is now ON: per the updated `evaluate_float` semantics, ON
        // means "value comes from animation data", which still holds when the
        // value is a held last keyframe. Render-path consumers
        // (e.g. Godot's `use_advanced`) and cell-default fallbacks both rely
        // on this signal to avoid silently dropping animated state.
        {
            let frame_data_buf = context.get_frame_data(11.0).expect("Failed to get frame data at frame 11");
            let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
            let parts = frame_data.parts();
            let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
            assert_eq!(part_state.position_x(), 32.0);
            assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        }
    }

    // Regression for the subframe-end-frame fix: `get_frame_data` accepts the
    // half-open interval `[start_frame, end_frame + 1)` so a Player doing
    // subframe interpolation can pass the fractional overshoot that briefly
    // lands in `(end_frame, end_frame + 1)` at the end / loop boundary without
    // the runtime erroring (which would blank the whole pose). Inside that
    // trailing interval the pose holds the last keyframe — byte-for-byte
    // identical to `end_frame`, i.e. no extrapolation. The upper bound is
    // exclusive, and the lower bound is unchanged.
    #[test]
    fn get_frame_data_accepts_subframe_overshoot_at_animation_end() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");
        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let start_frame = context.get_start_frame();
        let end_frame = context.get_end_frame();
        assert!(end_frame > start_frame, "fixture sanity: anime_1 should span multiple frames");

        // Pose at exactly `end_frame` (the last valid integer frame), copied out
        // because the returned slice aliases the context's reused frame buffer.
        let pose_at_end = context.get_frame_data(end_frame as f32).expect("end_frame must be valid").to_vec();
        assert!(!pose_at_end.is_empty(), "end_frame pose should not be empty");

        // Subframe overshoot into `(end_frame, end_frame + 1)` is accepted (it
        // used to return `InvalidParameter`) and holds the `end_frame` pose
        // exactly — no extrapolation past the final keyframe.
        for overshoot in [0.001_f32, 0.5, 0.999] {
            let frame_no = end_frame as f32 + overshoot;
            let pose = context.get_frame_data(frame_no).unwrap_or_else(|_| panic!("subframe overshoot {frame_no} must be accepted")).to_vec();
            assert_eq!(pose, pose_at_end, "pose at {frame_no} must hold the end_frame pose");
        }

        // Upper bound is exclusive: `end_frame + 1.0` (the start of the
        // nonexistent next frame) is still rejected, as is anything below
        // `start_frame`.
        assert!(context.get_frame_data(end_frame as f32 + 1.0).is_err(), "end_frame + 1.0 must be rejected");
        assert!(context.get_frame_data(start_frame as f32 - 0.5).is_err(), "below start_frame must be rejected");
    }

    #[test]
    fn test_part_hierarchy_and_visibility() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let ssab_buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
        let parts_meta = ssab.parts();

        // Hierarchy check
        let scl_xy_idx = parts_meta.iter().position(|p| p.name() == "SCL_XY").expect("Part SCL_XY not found");
        let scl_xy_part = parts_meta.get(scl_xy_idx);
        let parent_idx = scl_xy_part.parent_index();
        assert!(parent_idx >= 0);
        let parent_part = parts_meta.get(parent_idx as usize);
        assert_eq!(parent_part.name(), "LSCL_XY");

        // Visibility (Hide flag) check
        let hide_part_idx = parts_meta.iter().position(|p| p.name() == "HIDE").expect("Part HIDE not found") as i16;

        // Frame 0: Not hidden
        let frame_0_buf = context.get_frame_data(0.0).unwrap();
        let frame_0 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_0_buf) };
        let hide_state_0 = frame_0.parts().iter().find(|p| p.part_index() == hide_part_idx).unwrap();
        assert!(!hide_state_0.hide());

        // Frame 2: Hidden
        let frame_2_buf = context.get_frame_data(2.0).unwrap();
        let frame_2 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_2_buf) };
        let hide_state_2 = frame_2.parts().iter().find(|p| p.part_index() == hide_part_idx).unwrap();
        assert!(hide_state_2.hide());
    }

    #[test]
    fn test_world_matrices() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
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
        let num_parts = context.instance.resolved_layer.parts_vec.len();
        assert_eq!(world_matrices_len, num_parts * 16);

        let world_matrices = unsafe { ::core::slice::from_raw_parts(world_matrices_ptr, world_matrices_len) };

        // For simplicity, just test that matrices are non-zero/valid
        assert!(world_matrices[0] != 0.0 || world_matrices[1] != 0.0);
    }

    #[test]
    fn test_local_uvs() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let _ = context.get_frame_data(0.0).expect("Failed to get frame data");

        let uvs_ptr = context.get_local_uvs_ptr();
        let uvs_len = context.get_local_uvs_len();
        let num_parts = context.instance.resolved_layer.parts_vec.len();
        // 5 vertices * 2 floats per part.
        assert_eq!(uvs_len, num_parts * 10);

        let uvs = unsafe { ::core::slice::from_raw_parts(uvs_ptr, uvs_len) };

        // At least one cell-bearing part must have produced non-zero UVs.
        // (Cell-less parts and parts whose cell can't be resolved keep their
        // all-zero default — we don't make a stronger per-part assertion here
        // since the test fixture's exact part roster is incidental.)
        let any_nonzero = uvs.iter().any(|f| *f != 0.0);
        assert!(any_nonzero, "Expected at least one part to have non-zero UVs");

        // Find any part whose UV slot was populated with a non-degenerate
        // quad. There should be at least one such part in a basic animation.
        let any_valid_quad = (0..num_parts).any(|i| {
            let off = i * 10;
            let lt_u = uvs[off];
            let rt_u = uvs[off + 2];
            let lt_v = uvs[off + 1];
            let lb_v = uvs[off + 5];
            (rt_u - lt_u).abs() > f32::EPSILON && (lb_v - lt_v).abs() > f32::EPSILON
        });
        assert!(any_valid_quad, "Expected at least one part to have a non-degenerate UV quad");

        // Recomputing on a different frame should still yield a sane buffer.
        let _ = context.get_frame_data(1.0).expect("Failed to get frame data on frame 1");
        let uvs_after = unsafe { ::core::slice::from_raw_parts(context.get_local_uvs_ptr(), context.get_local_uvs_len()) };
        assert_eq!(uvs_after.len(), num_parts * 10);
        assert!(uvs_after.iter().any(|f| *f != 0.0));
    }

    #[test]
    fn test_cell_meta_per_part() {
        // Per-part cell metadata exposed via FFI lets Players drop their own
        // cellmap binary search. Layout: 6 floats per part
        // `[pivot_x, pivot_y, size_w, size_h, rect_left, rect_top]`.
        // Cell-bearing parts populate it; the rest stay zero.
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let _ = context.get_frame_data(0.0).expect("Failed to get frame data");

        let meta_ptr = context.get_cell_meta_ptr();
        let meta_len = context.get_cell_meta_len();
        let num_parts = context.instance.resolved_layer.parts_vec.len();
        assert_eq!(meta_len, num_parts * 6);

        let meta = unsafe { ::core::slice::from_raw_parts(meta_ptr, meta_len) };

        // At least one cell-bearing part must have non-zero size_w/size_h
        // (zero pivot is legal for a centered cell, so use size as the signal).
        let any_nonzero_size = (0..num_parts).any(|i| {
            let off = i * 6;
            meta[off + 2] != 0.0 || meta[off + 3] != 0.0
        });
        assert!(any_nonzero_size, "Expected at least one part to have non-zero cell size_w/size_h");

        // Recomputing should produce a sane buffer of the same shape.
        let _ = context.get_frame_data(1.0).expect("Failed to get frame data on frame 1");
        let meta_after = unsafe { ::core::slice::from_raw_parts(context.get_cell_meta_ptr(), context.get_cell_meta_len()) };
        assert_eq!(meta_after.len(), num_parts * 6);
    }

    #[test]
    fn test_deform_animation() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/allAttributeV7/allAttributeV7/deform.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read deform.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.set_render_mode(crate::core::RenderMode::SceneGraph);
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let ssab_buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };

        // Find a part that has deform animation
        let mut deform_part_idx = -1;
        if let Some(animation) = ssab.animations().iter().find(|a| a.name() == "anime_1") {
            let parts_anime = animation.parts_animation_data();
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
            let parts = frame_0.parts();
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
            let parts = frame_10.parts();
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
    fn test_shape_default_size_64x64() {
        // SS6 SsAnimeDecoder::updateState assigns a (64,64) initial size to
        // shape parts before reading animation data. The fixture's shape parts
        // (`shape_1`/`shape_2`/`shape_3` in `shape.ssab`) have no SizeX/SizeY
        // keyframes, so the runtime must surface the SS6 default instead of the
        // cell-size fallback used for cell-bearing types.
        prepare();
        let ssab_path = get_test_base_path().join("tests/allPartsV7/allPartsV7/shape.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read shape.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let ssab_buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
        let parts_meta = ssab.parts();
        let shape_idx = parts_meta.iter().position(|p| p.name() == "shape_1").expect("shape_1 not found") as i16;

        let frame_data_buf = context.get_frame_data(0.0).expect("Failed to get frame data");
        let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
        let parts = frame_data.parts();
        let state = parts.iter().find(|p| p.part_index() == shape_idx).expect("shape_1 state not found");

        assert_eq!(state.size_x(), 64.0, "Shape part default size_x must be 64.0");
        assert_eq!(state.size_y(), 64.0, "Shape part default size_y must be 64.0");
    }

    #[test]
    fn test_ssab_data_and_resource_separation() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
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
        let path = base.join("tests/overall/overall/UserData.ssab");
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
        let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
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
        let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
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

    type MixLayerPart = (f32, f32, f32, f32, f32, Option<([f32; 4], [f32; 4])>);

    fn build_mix_layer(parts: &[MixLayerPart]) -> LayerState<'static> {
        // tuple: (pos_x, pos_y, rot_z, scale, alpha, optional (dx, dy) deform).
        let mut layer = LayerState::new();
        for (i, p) in parts.iter().enumerate() {
            let mut s = runtime::framedata::ss::runtime::PartState::default();
            s.set_part_index(i as i16);
            s.set_cell(-1);
            s.set_vertex(-1);
            s.set_position_x(p.0);
            s.set_position_y(p.1);
            s.set_rotation_z(p.2);
            s.set_scale_x(p.3);
            s.set_scale_y(p.3);
            s.set_alpha(p.4);
            s.set_size_x(10.0);
            s.set_size_y(10.0);
            s.set_update_flag(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX);
            layer.parts_vec.push(s);
            let rvd = match p.5 {
                Some((dx, dy)) => crate::core::framedata::RawVertexDeform { present: true, dx, dy },
                None => crate::core::framedata::RawVertexDeform::default(),
            };
            layer.raw_vertex_deform.push(rvd);
        }
        layer
    }

    fn mix_source(weight: f32) -> SourceState {
        let mut s = SourceState::new();
        s.animation_idx = 0; // any non-negative value makes it active
        s.weight = weight;
        s
    }

    #[test]
    fn mix_poses_single_source_passthrough() {
        let layers = vec![build_mix_layer(&[(10.0, 20.0, 1.5, 2.0, 0.5, None)])];
        let sources = vec![mix_source(1.0)];
        let mut resolved = LayerState::new();

        mix_poses(&sources, &layers, 0, &mut resolved);

        assert_eq!(resolved.parts_vec.len(), 1);
        let r = &resolved.parts_vec[0];
        assert_eq!(r.position_x(), 10.0);
        assert_eq!(r.position_y(), 20.0);
        assert_eq!(r.rotation_z(), 1.5);
        assert_eq!(r.scale_x(), 2.0);
        assert_eq!(r.alpha(), 0.5);
    }

    #[test]
    fn mix_poses_50_50_blend_trsa() {
        let layer_a = build_mix_layer(&[(10.0, 20.0, 0.0, 1.0, 1.0, None)]);
        let layer_b = build_mix_layer(&[(30.0, 40.0, ::core::f32::consts::FRAC_PI_2, 2.0, 0.5, None)]);
        let sources = vec![mix_source(1.0), mix_source(1.0)];
        let layers = vec![layer_a, layer_b];
        let mut resolved = LayerState::new();

        mix_poses(&sources, &layers, 0, &mut resolved);

        let r = &resolved.parts_vec[0];
        assert!((r.position_x() - 20.0).abs() < 1e-6);
        assert!((r.position_y() - 30.0).abs() < 1e-6);
        assert!((r.rotation_z() - ::core::f32::consts::FRAC_PI_4).abs() < 1e-4);
        assert!((r.scale_x() - 1.5).abs() < 1e-6);
        assert!((r.alpha() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn mix_poses_auto_normalizes_weights() {
        let layer_a = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
        let layer_b = build_mix_layer(&[(100.0, 100.0, 0.0, 1.0, 1.0, None)]);
        let layers = vec![layer_a, layer_b];

        let mut resolved_a = LayerState::new();
        mix_poses(&[mix_source(0.7), mix_source(0.3)], &layers, 0, &mut resolved_a);

        let mut resolved_b = LayerState::new();
        mix_poses(&[mix_source(7.0), mix_source(3.0)], &layers, 0, &mut resolved_b);

        // Same ratio → same mix outcome regardless of absolute magnitudes.
        assert!((resolved_a.parts_vec[0].position_x() - resolved_b.parts_vec[0].position_x()).abs() < 1e-5);
        assert!((resolved_a.parts_vec[0].position_x() - 30.0).abs() < 1e-5);
    }

    #[test]
    fn mix_poses_inactive_animation_idx_skipped() {
        let layer_a = build_mix_layer(&[(10.0, 0.0, 0.0, 1.0, 1.0, None)]);
        let layer_b = build_mix_layer(&[(90.0, 0.0, 0.0, 1.0, 1.0, None)]);
        let layers = vec![layer_a, layer_b];

        let mut src_b = mix_source(1.0);
        src_b.animation_idx = -1; // removed slot — must be skipped
        let sources = vec![mix_source(1.0), src_b];

        let mut resolved = LayerState::new();
        mix_poses(&sources, &layers, 0, &mut resolved);

        // Only source A contributes; resolved == primary verbatim.
        assert_eq!(resolved.parts_vec[0].position_x(), 10.0);
    }

    #[test]
    fn mix_poses_zero_weight_source_skipped() {
        let layer_a = build_mix_layer(&[(10.0, 0.0, 0.0, 1.0, 1.0, None)]);
        let layer_b = build_mix_layer(&[(90.0, 0.0, 0.0, 1.0, 1.0, None)]);
        let layers = vec![layer_a, layer_b];

        // A active, B weight 0 → only A contributes. inv_total = 1/1 → x = 10.
        let sources = vec![mix_source(1.0), mix_source(0.0)];

        let mut resolved = LayerState::new();
        mix_poses(&sources, &layers, 0, &mut resolved);

        assert_eq!(resolved.parts_vec[0].position_x(), 10.0);
    }

    #[test]
    fn mix_poses_total_weight_zero_uses_primary() {
        let layer_a = build_mix_layer(&[(7.0, 0.0, 0.0, 1.0, 1.0, None)]);
        let layer_b = build_mix_layer(&[(99.0, 0.0, 0.0, 1.0, 1.0, None)]);
        let layers = vec![layer_a, layer_b];

        // All weights zero. Fall back to primary's pose.
        let sources = vec![mix_source(0.0), mix_source(0.0)];

        let mut resolved = LayerState::new();
        mix_poses(&sources, &layers, 0, &mut resolved);

        assert_eq!(resolved.parts_vec[0].position_x(), 7.0);
    }

    #[test]
    fn mix_poses_raw_vertex_deform_weighted() {
        let layer_a = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, Some(([1.0, 2.0, 3.0, 4.0], [10.0, 20.0, 30.0, 40.0])))]);
        let layer_b = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, Some(([5.0, 6.0, 7.0, 8.0], [50.0, 60.0, 70.0, 80.0])))]);
        let sources = vec![mix_source(1.0), mix_source(1.0)];
        let layers = vec![layer_a, layer_b];

        let mut resolved = LayerState::new();
        mix_poses(&sources, &layers, 0, &mut resolved);

        let d = &resolved.raw_vertex_deform[0];
        assert!(d.present);
        assert!((d.dx[0] - 3.0).abs() < 1e-6);
        assert!((d.dx[3] - 6.0).abs() < 1e-6);
        assert!((d.dy[0] - 30.0).abs() < 1e-6);
        assert!((d.dy[3] - 60.0).abs() < 1e-6);
    }

    fn load_ringo_resource() -> SsabData {
        prepare();
        let ssab_path = get_test_base_path().join("tests/Ringo/Ringo/Ringo.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
        SsabData::load_copy(&input_buf).expect("Failed to load SsabData")
    }

    fn evaluate_ringo(resource: &AnimationResource<'_>, anim: &str) -> Vec<(f32, f32, f32, f32)> {
        let mut ctx = Context::new();
        ctx.bind_resource(resource);
        ctx.setup_animation(anim).unwrap_or_else(|_| panic!("setup_animation({anim}) failed"));
        let start = ctx.instance.primary_source().controller.get_start_frame() as f32;
        let _ = ctx.get_frame_data(start).expect("get_frame_data failed");
        ctx.instance.resolved_layer.parts_vec.iter().map(|p| (p.position_x(), p.position_y(), p.rotation_z(), p.alpha())).collect()
    }

    #[test]
    fn ringo_blend_walk_run_at_50_50_is_midpoint() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        let walk = evaluate_ringo(&resource, "walk");
        let run = evaluate_ringo(&resource, "run");

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk_start = ctx.instance.primary_source().controller.get_start_frame() as f32;
        let handle = ctx.add_blend_source("run", 1.0).expect("add_blend_source(run) failed");
        let run_start = ctx.instance.sources[handle as usize].controller.get_start_frame() as f32;
        ctx.set_source_frame(handle, run_start).unwrap();
        let _ = ctx.get_frame_data(walk_start).expect("get_frame_data failed");
        let mixed: Vec<(f32, f32, f32, f32)> = ctx.instance.resolved_layer.parts_vec.iter().map(|p| (p.position_x(), p.position_y(), p.rotation_z(), p.alpha())).collect();

        assert_eq!(walk.len(), run.len());
        assert_eq!(walk.len(), mixed.len());
        for i in 0..walk.len() {
            let (wx, wy, wr, wa) = walk[i];
            let (rx, ry, rr, ra) = run[i];
            let (mx, my, mr, ma) = mixed[i];
            // Auto-normalized 1.0 / 1.0 weights → 50/50 midpoint per attribute.
            assert!((mx - 0.5 * (wx + rx)).abs() < 1e-3, "part {i} position_x: mixed={mx} expected_midpoint={}", 0.5 * (wx + rx));
            assert!((my - 0.5 * (wy + ry)).abs() < 1e-3, "part {i} position_y");
            assert!((mr - 0.5 * (wr + rr)).abs() < 1e-3, "part {i} rotation_z");
            assert!((ma - 0.5 * (wa + ra)).abs() < 1e-3, "part {i} alpha");
        }
    }

    #[test]
    fn ringo_blend_zero_weight_secondary_equals_primary() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        let walk_only = evaluate_ringo(&resource, "walk");

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk_start = ctx.instance.primary_source().controller.get_start_frame() as f32;
        let _ = ctx.add_blend_source("run", 0.0).expect("add_blend_source(run) failed");
        let _ = ctx.get_frame_data(walk_start).expect("get_frame_data failed");
        let with_inactive: Vec<(f32, f32, f32, f32)> = ctx.instance.resolved_layer.parts_vec.iter().map(|p| (p.position_x(), p.position_y(), p.rotation_z(), p.alpha())).collect();

        // Zero-weight secondary contributes nothing; resolved must equal primary verbatim.
        assert_eq!(walk_only, with_inactive);
    }

    #[test]
    fn ringo_blend_removed_source_equals_primary() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        let walk_only = evaluate_ringo(&resource, "walk");

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk_start = ctx.instance.primary_source().controller.get_start_frame() as f32;
        let handle = ctx.add_blend_source("run", 1.0).expect("add_blend_source(run) failed");
        ctx.remove_blend_source(handle).expect("remove_blend_source failed");
        let _ = ctx.get_frame_data(walk_start).expect("get_frame_data failed");
        let after_remove: Vec<(f32, f32, f32, f32)> = ctx.instance.resolved_layer.parts_vec.iter().map(|p| (p.position_x(), p.position_y(), p.rotation_z(), p.alpha())).collect();

        assert_eq!(walk_only, after_remove);
    }

    #[test]
    fn ringo_blend_api_validates_state() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        // add_blend_source before setup_animation -> AnimationNotFound.
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        assert!(matches!(ctx.add_blend_source("run", 1.0), Err(error::SSRuntimeError::AnimationNotFound)));

        ctx.setup_animation("walk").unwrap();

        // Unknown animation name -> AnimationNotFound.
        assert!(matches!(ctx.add_blend_source("not_a_real_anim", 1.0), Err(error::SSRuntimeError::AnimationNotFound)));

        // Cannot remove the primary slot.
        let primary = ctx.primary_handle();
        assert!(matches!(ctx.remove_blend_source(primary), Err(error::SSRuntimeError::InvalidParameter)));

        // Invalid handle on weight/frame/primary/speed setters.
        assert!(matches!(ctx.set_source_weight(99, 1.0), Err(error::SSRuntimeError::InvalidParameter)));
        assert!(matches!(ctx.set_source_frame(99, 0.0), Err(error::SSRuntimeError::InvalidParameter)));
        assert!(matches!(ctx.set_source_speed(99, 1.0), Err(error::SSRuntimeError::InvalidParameter)));
        assert!(matches!(ctx.set_primary(99), Err(error::SSRuntimeError::InvalidParameter)));
        assert!(ctx.source_weight(99).is_none());
        assert!(ctx.source_frame(99).is_none());
    }

    #[test]
    fn ringo_set_primary_routes_to_new_primary() {
        // After set_primary, all primary-targeted accessors must route to
        // the new primary's controller; the old primary's state must remain
        // independent.
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();

        let walk_handle = ctx.primary_handle();
        let run_handle = ctx.add_blend_source("run", 1.0).unwrap();
        assert_eq!(ctx.primary_handle(), walk_handle);

        // Set walk to frame 5 via primary-targeted setter.
        ctx.set_frame_no(5.0);
        assert_eq!(ctx.get_frame_no(), 5.0);

        // Swap primary to run; primary-targeted setter now targets run.
        ctx.set_primary(run_handle).unwrap();
        assert_eq!(ctx.primary_handle(), run_handle);
        ctx.set_frame_no(20.0);
        assert_eq!(ctx.get_frame_no(), 20.0);

        // Swap back to walk; walk's frame must be unchanged by run-period mutations.
        ctx.set_primary(walk_handle).unwrap();
        assert_eq!(ctx.primary_handle(), walk_handle);
        assert_eq!(ctx.get_frame_no(), 5.0, "walk's controller must be independent of run's");

        // And one more swap confirms run's frame is also preserved.
        ctx.set_primary(run_handle).unwrap();
        assert_eq!(ctx.get_frame_no(), 20.0);
    }

    #[test]
    fn ringo_clear_blend_sources_deactivates_secondaries() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();

        let primary_handle = ctx.primary_handle();
        let h1 = ctx.add_blend_source("run", 1.0).unwrap();
        let h2 = ctx.add_blend_source("dance", 1.0).unwrap();
        let h3 = ctx.add_blend_source("jump", 1.0).unwrap();
        assert_eq!(ctx.blend_source_count(), 4);

        ctx.clear_blend_sources();

        // Primary stays active; every secondary is deactivated.
        assert_eq!(ctx.blend_source_count(), 1);
        assert_eq!(ctx.primary_handle(), primary_handle);
        assert!(ctx.source_weight(primary_handle).is_some());
        assert!(ctx.source_weight(h1).is_none());
        assert!(ctx.source_weight(h2).is_none());
        assert!(ctx.source_weight(h3).is_none());

        // get_frame_data still produces a non-empty pose after clear.
        let walk_start = ctx.instance.primary_source().controller.get_start_frame() as f32;
        let _ = ctx.get_frame_data(walk_start).expect("get_frame_data should succeed after clear");
        assert!(!ctx.instance.resolved_layer.parts_vec.is_empty());
    }

    #[test]
    fn ringo_blend_set_source_speed() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let handle = ctx.add_blend_source("run", 1.0).unwrap();

        // Valid handle: any positive speed accepted.
        assert!(ctx.set_source_speed(handle, 2.0).is_ok());
        assert!(ctx.set_source_speed(handle, 0.5).is_ok());

        // Removed slot returns InvalidParameter.
        ctx.remove_blend_source(handle).unwrap();
        assert!(matches!(ctx.set_source_speed(handle, 1.0), Err(error::SSRuntimeError::InvalidParameter)));
    }

    #[test]
    fn ringo_blend_primary_weight_zero_keeps_structure() {
        // Regression for the bug where Stage A skipped the primary when its
        // weight was 0, leaving the resolved pose empty. Primary is the
        // discrete/structure source and must always be extracted regardless
        // of weight; its weight only controls its continuous-value share.
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        let run_only = evaluate_ringo(&resource, "run");

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk_start = ctx.instance.primary_source().controller.get_start_frame() as f32;
        let primary_handle = ctx.primary_handle();
        let handle = ctx.add_blend_source("run", 1.0).expect("add_blend_source(run) failed");
        let run_start = ctx.instance.sources[handle as usize].controller.get_start_frame() as f32;
        ctx.set_source_frame(handle, run_start).unwrap();
        ctx.set_source_weight(primary_handle, 0.0).unwrap();
        let _ = ctx.get_frame_data(walk_start).expect("get_frame_data failed");
        let resolved: Vec<(f32, f32, f32, f32)> = ctx.instance.resolved_layer.parts_vec.iter().map(|p| (p.position_x(), p.position_y(), p.rotation_z(), p.alpha())).collect();

        // Primary's structure must be preserved: parts count matches run (same SSAE).
        assert!(!resolved.is_empty(), "resolved parts_vec must not be empty when primary weight=0");
        assert_eq!(resolved.len(), run_only.len());

        // Continuous values must equal the secondary verbatim (total_weight = 1.0, w = 1.0 for run only).
        for i in 0..resolved.len() {
            let (mx, my, mr, ma) = resolved[i];
            let (rx, ry, rr, ra) = run_only[i];
            assert!((mx - rx).abs() < 1e-3, "part {i} position_x: mixed={mx} run={rx}");
            assert!((my - ry).abs() < 1e-3, "part {i} position_y");
            assert!((mr - rr).abs() < 1e-3, "part {i} rotation_z");
            assert!((ma - ra).abs() < 1e-3, "part {i} alpha");
        }
    }

    #[test]
    fn setup_animation_after_add_blend_clears_secondary() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let _run = ctx.add_blend_source("run", 0.5).unwrap();
        assert_eq!(ctx.blend_source_count(), 2);

        ctx.setup_animation("dance").unwrap();

        assert_eq!(ctx.blend_source_count(), 1, "secondary slots must be cleared by setup_animation");
        assert_eq!(ctx.primary_handle(), 0);
        assert_eq!(ctx.instance.sources.len(), 1);
        assert_eq!(ctx.instance.layers.len(), 1);
    }

    #[test]
    fn setup_animation_resets_primary_handle() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let run = ctx.add_blend_source("run", 0.5).unwrap();
        ctx.set_primary(run).unwrap();
        assert_eq!(ctx.primary_handle(), run);

        ctx.setup_animation("dance").unwrap();

        assert_eq!(ctx.primary_handle(), 0, "setup_animation must reset primary to slot 0");
        assert_eq!(ctx.instance.sources.len(), 1);
    }

    #[test]
    fn bind_resource_clears_blend_state() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let _ = ctx.add_blend_source("run", 0.5).unwrap();
        assert_eq!(ctx.blend_source_count(), 2);

        // Re-bind to the same resource (same effect as switching to a new one
        // for blend-state semantics). The blend slots must collapse.
        ctx.bind_resource(&resource);

        assert_eq!(ctx.instance.sources.len(), 1);
        assert_eq!(ctx.instance.layers.len(), 1);
        assert_eq!(ctx.primary_handle(), 0);
        // Primary is also reset: animation_idx < 0 → blend_source_count is 0.
        assert_eq!(ctx.blend_source_count(), 0);
    }

    #[test]
    fn ringo_blend_weight_round_trip() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let handle = ctx.add_blend_source("run", 0.42).unwrap();
        assert_eq!(ctx.source_weight(handle), Some(0.42));

        ctx.set_source_weight(handle, 0.9).unwrap();
        assert_eq!(ctx.source_weight(handle), Some(0.9));

        ctx.remove_blend_source(handle).unwrap();
        assert!(ctx.source_weight(handle).is_none());
    }

    #[test]
    fn mix_poses_update_flag_or_merged() {
        // Source A has AttributePosX set, source B has AttributeRotZ. The mixed
        // pose must report both so Stage B's `needs_center` and any consumer
        // logic see every attribute that contributed to the value.
        let mut layer_a = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
        let mut layer_b = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
        layer_a.parts_vec[0].set_update_flag(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX);
        layer_b.parts_vec[0].set_update_flag(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ);

        let sources = vec![mix_source(1.0), mix_source(1.0)];
        let layers = vec![layer_a, layer_b];

        let mut resolved = LayerState::new();
        mix_poses(&sources, &layers, 0, &mut resolved);

        let f = resolved.parts_vec[0].update_flag();
        assert!(f.contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
        assert!(f.contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ));
    }

    #[test]
    fn crossfade_rebalances_weights_and_completes() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();

        let walk = ctx.primary_handle();
        let run = ctx.add_blend_source("run", 0.0).unwrap();
        ctx.crossfade(walk, run, 1000.0).unwrap();

        // Initial state: weights overwritten to (1, 0); progress at 0.
        assert_eq!(ctx.source_weight(walk), Some(1.0));
        assert_eq!(ctx.source_weight(run), Some(0.0));
        assert_eq!(ctx.crossfade_progress(), Some(0.0));

        // Halfway.
        ctx.update(500.0);
        let p = ctx.crossfade_progress().unwrap();
        assert!((p - 0.5).abs() < 1e-3, "progress at midpoint = {p}");
        assert!((ctx.source_weight(walk).unwrap() - 0.5).abs() < 1e-3);
        assert!((ctx.source_weight(run).unwrap() - 0.5).abs() < 1e-3);
        // Primary unchanged mid-fade.
        assert_eq!(ctx.primary_handle(), walk);

        // Past completion: primary swaps, `from` deactivated, crossfade cleared.
        ctx.update(600.0);
        assert!(ctx.crossfade_progress().is_none());
        assert_eq!(ctx.primary_handle(), run);
        assert!(ctx.source_weight(walk).is_none(), "walk should be deactivated");
        assert_eq!(ctx.source_weight(run), Some(1.0));
    }

    #[test]
    fn crossfade_validates_arguments() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk = ctx.primary_handle();
        let run = ctx.add_blend_source("run", 1.0).unwrap();

        // Identical handles.
        assert!(matches!(ctx.crossfade(walk, walk, 100.0), Err(error::SSRuntimeError::InvalidParameter)));
        // Zero / negative duration.
        assert!(matches!(ctx.crossfade(walk, run, 0.0), Err(error::SSRuntimeError::InvalidParameter)));
        assert!(matches!(ctx.crossfade(walk, run, -10.0), Err(error::SSRuntimeError::InvalidParameter)));
        // Invalid handle.
        assert!(matches!(ctx.crossfade(walk, 99, 100.0), Err(error::SSRuntimeError::InvalidParameter)));
        // `from` must be a real animation (not empty / not removed).
        ctx.remove_blend_source(run).unwrap();
        assert!(matches!(ctx.crossfade(run, walk, 100.0), Err(error::SSRuntimeError::InvalidParameter)));
    }

    #[test]
    fn crossfade_cancelled_when_referenced_slot_removed() {
        // `remove_blend_source` on a slot that the in-flight crossfade
        // references must clear the crossfade so subsequent `update`s do not
        // touch dangling slot weights.
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk = ctx.primary_handle();
        let run = ctx.add_blend_source("run", 0.0).unwrap();
        ctx.crossfade(walk, run, 1000.0).unwrap();
        assert!(ctx.crossfade_progress().is_some());

        ctx.remove_blend_source(run).unwrap();
        assert!(ctx.crossfade_progress().is_none(), "removing `to` must cancel crossfade");
    }

    #[test]
    fn crossfade_to_empty_keeps_primary() {
        // `to` = empty source: the fade interpolates toward setup pose without
        // changing primary (empty cannot be promoted). On completion the
        // crossfade clears but primary is unchanged so discrete attributes
        // (Cell, blend_type, events) keep firing from the original primary.
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk = ctx.primary_handle();
        let empty = ctx.add_empty_blend_source(0.0).unwrap();
        ctx.crossfade(walk, empty, 500.0).unwrap();

        ctx.update(500.0);
        assert!(ctx.crossfade_progress().is_none());
        assert_eq!(ctx.primary_handle(), walk, "empty `to` keeps original primary");
        assert_eq!(ctx.source_weight(empty), Some(1.0));
        assert_eq!(ctx.source_weight(walk), Some(0.0));
    }

    #[test]
    fn empty_blend_source_fades_to_identity_pose() {
        // walk + empty at 50/50 must produce a pose midway between walk's
        // pose and identity (scale = 1, alpha = 1, translations/rotations = 0).
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        let walk = evaluate_ringo(&resource, "walk");

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk_start = ctx.instance.primary_source().controller.get_start_frame() as f32;
        ctx.set_source_weight(ctx.primary_handle(), 1.0).unwrap();
        let _ = ctx.add_empty_blend_source(1.0).unwrap();
        let _ = ctx.get_frame_data(walk_start).expect("get_frame_data failed");
        let mixed: Vec<(f32, f32, f32, f32)> = ctx.instance.resolved_layer.parts_vec.iter().map(|p| (p.position_x(), p.position_y(), p.rotation_z(), p.alpha())).collect();

        assert_eq!(walk.len(), mixed.len());
        for i in 0..walk.len() {
            let (wx, wy, wr, wa) = walk[i];
            let (mx, my, mr, ma) = mixed[i];
            // Empty contributes (0, 0, 0, 1) for (pos_x, pos_y, rot_z, alpha).
            assert!((mx - 0.5 * wx).abs() < 1e-3, "part {i} position_x: mixed={mx} expected={}", 0.5 * wx);
            assert!((my - 0.5 * wy).abs() < 1e-3, "part {i} position_y");
            assert!((mr - 0.5 * wr).abs() < 1e-3, "part {i} rotation_z");
            assert!((ma - 0.5 * (wa + 1.0)).abs() < 1e-3, "part {i} alpha: mixed={ma} expected={}", 0.5 * (wa + 1.0));
        }
    }

    #[test]
    fn empty_blend_source_rejects_unsupported_setters() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let empty = ctx.add_empty_blend_source(0.5).unwrap();

        // Allowed: weight setter/getter.
        assert!(ctx.set_source_weight(empty, 0.8).is_ok());
        assert_eq!(ctx.source_weight(empty), Some(0.8));

        // Rejected: frame, speed, event threshold, set_primary (no controller / no animation).
        assert!(matches!(ctx.set_source_frame(empty, 0.0), Err(error::SSRuntimeError::InvalidParameter)));
        assert!(matches!(ctx.set_source_speed(empty, 1.0), Err(error::SSRuntimeError::InvalidParameter)));
        assert!(matches!(ctx.set_source_event_threshold(empty, 0.5), Err(error::SSRuntimeError::InvalidParameter)));
        assert!(matches!(ctx.set_primary(empty), Err(error::SSRuntimeError::InvalidParameter)));
        assert!(ctx.source_frame(empty).is_none());
        assert!(ctx.source_event_threshold(empty).is_none());
    }

    #[test]
    fn empty_blend_source_requires_real_primary() {
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        // No setup_animation called yet -> primary is still inactive.
        assert!(matches!(ctx.add_empty_blend_source(1.0), Err(error::SSRuntimeError::AnimationNotFound)));
    }

    #[test]
    fn event_threshold_default_fires_primary_only() {
        // Default threshold is 1.0 for every source so only the primary's
        // events appear in collected output; source-handle parallel array is
        // uniformly the primary handle.
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk = ctx.primary_handle();
        let _run = ctx.add_blend_source("run", 1.0).unwrap();
        // Spin enough updates to cover walk's full frame range.
        for _ in 0..200 {
            ctx.update(16.0);
            for &h in ctx.get_passed_event_source_handles() {
                assert_eq!(h, walk, "default threshold should fire primary events only");
            }
        }
    }

    #[test]
    fn event_threshold_secondary_opt_in() {
        // Lowering a secondary's threshold below its weight enrolls its
        // events into the collected output, tagged with the secondary handle.
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        // Find an animation in the Ringo fixture that owns at least one event;
        // skip the test gracefully if none exists.
        let mut anim_with_events: Option<&str> = None;
        for name in ["walk", "run", "dance", "jump", "Wait", "wait"] {
            let mut probe = Context::new();
            probe.bind_resource(&resource);
            if probe.setup_animation(name).is_err() {
                continue;
            }
            let anim_idx = probe.instance.primary_source().animation_idx as usize;
            if resource.animations[anim_idx].events.is_some_and(|e| !e.is_empty()) {
                anim_with_events = Some(name);
                break;
            }
        }
        let Some(event_anim) = anim_with_events else {
            // Ringo fixture has no events on any tried animation: passthrough.
            return;
        };

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let secondary = ctx.add_blend_source(event_anim, 1.0).unwrap();
        ctx.set_source_event_threshold(secondary, 0.5).unwrap();
        assert_eq!(ctx.source_event_threshold(secondary), Some(0.5));

        let mut saw_secondary = false;
        for _ in 0..200 {
            ctx.update(16.0);
            for &h in ctx.get_passed_event_source_handles() {
                if h == secondary {
                    saw_secondary = true;
                }
            }
            if saw_secondary {
                break;
            }
        }
        assert!(saw_secondary, "secondary with threshold below weight should fire events");
    }

    #[test]
    fn event_threshold_below_weight_required() {
        // weight (1.0) < threshold (1.5) → secondary still does not fire.
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let primary = ctx.primary_handle();
        let secondary = ctx.add_blend_source("run", 1.0).unwrap();
        ctx.set_source_event_threshold(secondary, 1.5).unwrap();
        for _ in 0..200 {
            ctx.update(16.0);
            for &h in ctx.get_passed_event_source_handles() {
                assert_eq!(h, primary, "threshold above weight should silence secondary");
            }
        }
    }

    #[test]
    fn event_threshold_at_weight_boundary_fires() {
        // Boundary case: weight == event_threshold. Firing rule is
        // `weight >= threshold` so the secondary must fire at the exact
        // boundary, mirroring the documented `>=` semantics.
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");

        let mut anim_with_events: Option<&str> = None;
        for name in ["walk", "run", "dance", "jump", "Wait", "wait"] {
            let mut probe = Context::new();
            probe.bind_resource(&resource);
            if probe.setup_animation(name).is_err() {
                continue;
            }
            let anim_idx = probe.instance.primary_source().animation_idx as usize;
            if resource.animations[anim_idx].events.is_some_and(|e| !e.is_empty()) {
                anim_with_events = Some(name);
                break;
            }
        }
        let Some(event_anim) = anim_with_events else {
            return;
        };

        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let secondary = ctx.add_blend_source(event_anim, 0.5).unwrap();
        ctx.set_source_event_threshold(secondary, 0.5).unwrap();

        let mut saw_secondary = false;
        for _ in 0..200 {
            ctx.update(16.0);
            for &h in ctx.get_passed_event_source_handles() {
                if h == secondary {
                    saw_secondary = true;
                }
            }
            if saw_secondary {
                break;
            }
        }
        assert!(saw_secondary, "weight == threshold must fire (>= semantics)");
    }

    #[test]
    fn crossfade_cancel_preserves_weights() {
        // `cancel_crossfade` clears the in-flight state but leaves source
        // weights at their current mid-fade value (documented contract for
        // caller-driven normalization). Subsequent `update` must not rebalance
        // them further.
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk = ctx.primary_handle();
        let run = ctx.add_blend_source("run", 0.0).unwrap();

        ctx.crossfade(walk, run, 1000.0).unwrap();
        ctx.update(500.0);
        let walk_w = ctx.source_weight(walk).unwrap();
        let run_w = ctx.source_weight(run).unwrap();
        assert!((walk_w - 0.5).abs() < 1e-3);
        assert!((run_w - 0.5).abs() < 1e-3);

        ctx.cancel_crossfade();
        assert!(ctx.crossfade_progress().is_none(), "cancel clears progress");

        // Subsequent updates must not rebalance the cancelled fade.
        ctx.update(300.0);
        assert_eq!(ctx.source_weight(walk), Some(walk_w), "walk weight frozen after cancel");
        assert_eq!(ctx.source_weight(run), Some(run_w), "run weight frozen after cancel");
        // Primary unchanged.
        assert_eq!(ctx.primary_handle(), walk);
    }

    #[test]
    fn crossfade_overwrites_in_flight() {
        // Re-invoking `crossfade` mid-fade discards the in-flight one and
        // re-initializes weights of the new `from`/`to` to (1, 0). Progress
        // resets to 0.
        let ssab = load_ringo_resource();
        let resource = AnimationResource::new(ssab.as_slice()).expect("Failed to load resource");
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation("walk").unwrap();
        let walk = ctx.primary_handle();
        let run = ctx.add_blend_source("run", 0.0).unwrap();

        ctx.crossfade(walk, run, 1000.0).unwrap();
        ctx.update(500.0);
        assert!((ctx.crossfade_progress().unwrap() - 0.5).abs() < 1e-3);

        // Same handles, new duration → weights snap back to (1, 0) and progress restarts at 0.
        ctx.crossfade(walk, run, 200.0).unwrap();
        assert_eq!(ctx.crossfade_progress(), Some(0.0), "overwrite resets progress");
        assert_eq!(ctx.source_weight(walk), Some(1.0), "overwrite resets from weight to 1.0");
        assert_eq!(ctx.source_weight(run), Some(0.0), "overwrite resets to weight to 0.0");

        // New fade completes in 200ms, not the 500ms remaining of the discarded one.
        ctx.update(200.0);
        assert!(ctx.crossfade_progress().is_none(), "new fade completes per new duration");
        assert_eq!(ctx.primary_handle(), run);
    }
}
