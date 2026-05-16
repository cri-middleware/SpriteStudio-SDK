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
        if let Some(fb_animations) = ssab.animations() {
            for anim in fb_animations {
                let mut cached_parts = Vec::new();
                if let Some(parts) = anim.parts_animation_data() {
                    let fb_parts = ssab.parts();
                    for (i, p) in parts.iter().enumerate() {
                        let part_meta = fb_parts.map(|meta| meta.get(i));
                        let parent_index = part_meta.map(|pm| pm.parent_index()).unwrap_or(-1);
                        let blend_type = part_meta.map(|pm| pm.blend_type()).unwrap_or(ssab::ssab::ss::format::BlendType::Mix);
                        let part_type_disc = part_meta.map(|pm| pm.part_type_type()).unwrap_or(ssab::ssab::ss::format::PartType::NONE);
                        let mask_influence = part_meta.map(|pm| pm.mask_influence()).unwrap_or(false);
                        let is_mask_contributor = if part_type_disc == ssab::ssab::ss::format::PartType::PartTypeMask {
                            true
                        } else if part_type_disc == ssab::ssab::ss::format::PartType::PartTypeShape {
                            part_meta.and_then(|pm| pm.part_type_as_part_type_shape()).map(|s| s.shape_mask()).unwrap_or(false)
                        } else if part_type_disc == ssab::ssab::ss::format::PartType::PartTypeText {
                            part_meta.and_then(|pm| pm.part_type_as_part_type_text()).map(|t| t.text_mask()).unwrap_or(false)
                        } else if part_type_disc == ssab::ssab::ss::format::PartType::PartTypeNines {
                            part_meta.and_then(|pm| pm.part_type_as_part_type_nines()).map(|n| n.nines_mask()).unwrap_or(false)
                        } else {
                            false
                        };
                        let resolved_cells = crate::core::framedata::resolve_cell_array_meta(&ssab, p.cell_array());
                        let shape_kind = part_meta.and_then(|pm| pm.part_type_as_part_type_shape()).and_then(|s| match s.shape_type() {
                            ssab::ssab::ss::format::ShapeType::Rectangle => Some(crate::core::geometry::ShapeType::Rectangle),
                            ssab::ssab::ss::format::ShapeType::Triangle => Some(crate::core::geometry::ShapeType::Triangle),
                            ssab::ssab::ss::format::ShapeType::Star => Some(crate::core::geometry::ShapeType::Star),
                            ssab::ssab::ss::format::ShapeType::Arrow => Some(crate::core::geometry::ShapeType::Arrow),
                            _ => None,
                        });
                        let mesh_binding = crate::core::framedata::build_mesh_binding_cache(part_meta.and_then(|pm| pm.mesh_binding()));
                        let mesh_cell = if part_type_disc == ssab::ssab::ss::format::PartType::PartTypeMesh {
                            crate::core::framedata::build_mesh_cell_data(&ssab, p.cell_array())
                        } else {
                            None
                        };
                        cached_parts.push(crate::core::framedata::CachedPartData::new(p, i as i16, parent_index, resolved_cells, shape_kind, mesh_binding, mesh_cell, blend_type, part_type_disc, mask_influence, is_mask_contributor));
                    }
                }
                let name_hash = anim.name_hash();
                let (canvas_width, canvas_height) = if let Some(canvas) = anim.canvas() { (canvas.x2() - canvas.x1(), canvas.y2() - canvas.y1()) } else { (0.0, 0.0) };
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
    vertice_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    deform_vec: Vec<flatbuffers::WIPOffset<crate::runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
    cursors: Vec<usize>,
    cell_meta_per_part: Vec<f32>,
    cell_texture_hashes: Vec<u32>,
    local_vertices: Vec<f32>,
    /// Per-part mesh deform retained from Phase 1 for the Phase 2 skinning pass.
    mesh_deform: Vec<crate::core::framedata::PartDeformFrame>,
}

impl<'a> LayerState<'a> {
    fn new() -> Self {
        Self {
            parts_vec: Vec::new(),
            cell_vec: Vec::new(),
            parts_color_vec: Vec::new(),
            shader_vec: Vec::new(),
            vertice_vec: Vec::new(),
            deform_vec: Vec::new(),
            cursors: Vec::new(),
            cell_meta_per_part: Vec::new(),
            cell_texture_hashes: Vec::new(),
            local_vertices: Vec::new(),
            mesh_deform: Vec::new(),
        }
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

/// Per-instance playback state. Owns the playback controller, the
/// active layer's pose data, the resolved global buffers, and the
/// configuration knobs that influence frame evaluation. Designed to
/// be the "Lightweight Instance" of Phase 3 — many of these can coexist
/// against a single shared `EvaluationContext`.
struct InstanceState<'a> {
    controller: core::playback_controller::PlaybackController,
    animation_idx: i32,
    z_order: crate::core::z_order::ZOrderTable,
    /// Currently a single layer; Phase 3 will widen this into an array
    /// (or array wrapper) so blend sources can each own a pose buffer.
    layer: LayerState<'a>,
    globals: GlobalBuffers,
    collected_event_indices: Vec<i32>,
    coordinate_system: crate::core::SsCoordinateSystem,
    origin: crate::core::SsOrigin,
    unit: crate::core::SsUnit,
    render_mode: crate::core::RenderMode,
}

impl<'a> InstanceState<'a> {
    fn new() -> Self {
        Self {
            controller: core::playback_controller::PlaybackController::new(),
            animation_idx: -1,
            z_order: crate::core::z_order::ZOrderTable::new(),
            layer: LayerState::new(),
            globals: GlobalBuffers::new(),
            collected_event_indices: Vec::new(),
            coordinate_system: crate::core::SsCoordinateSystem::default(),
            origin: crate::core::SsOrigin::default(),
            unit: crate::core::SsUnit::default(),
            render_mode: crate::core::RenderMode::default(),
        }
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
        self.instance.controller.reset_setup();
        self.instance.animation_idx = -1;
        self.instance.layer.cursors.clear();
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
        self.instance.controller.reset_setup();
        self.instance.animation_idx = -1;
        self.instance.layer.cursors.clear();
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
        self.instance.controller.setup(metadata.start_frame, metadata.end_frame, metadata.fps);
        self.instance.animation_idx = index as i32;

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

        let layer = &mut self.instance.layer;
        layer.parts_vec.clear();
        layer.parts_vec.reserve(num_parts);
        layer.cell_vec.clear();
        layer.cell_vec.reserve(cell_c);
        layer.parts_color_vec.clear();
        layer.parts_color_vec.reserve(color_c);
        layer.shader_vec.clear();
        layer.shader_vec.reserve(shader_c);
        layer.vertice_vec.clear();
        layer.vertice_vec.reserve(vertex_c);
        layer.deform_vec.clear();
        layer.deform_vec.reserve(deform_c);

        layer.cursors.clear();
        layer.cursors.resize(num_parts * crate::core::framedata::NUM_PART_ATTRIBUTES, 0);
        layer.cell_meta_per_part.clear();
        layer.cell_meta_per_part.resize(num_parts * 6, 0.0);
        layer.cell_texture_hashes.clear();
        layer.cell_texture_hashes.resize(num_parts, 0);
        layer.local_vertices.clear();
        layer.local_vertices.resize(num_parts * crate::core::framedata::LOCAL_VERTICES_STRIDE, 0.0);

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

        self.instance.z_order.setup(&metadata.cached_parts);

        Ok(())
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        self.instance.controller.update(delta_time);
    }

    pub fn get_frame_data(&mut self, frame_no: f32) -> Result<&[u8], error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };

        if self.instance.animation_idx < 0 {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }

        let metadata = &resource.animations[self.instance.animation_idx as usize];
        if !((metadata.start_frame as f32) <= frame_no && frame_no <= (metadata.end_frame as f32)) {
            return Err(error::SSRuntimeError::InvalidParameter);
        }

        self.eval.fb.reset();

        // Phase 1: extract local pose from the SSAB frame data.
        let offset = core::framedata::get_frame_data(
            &mut self.eval.fb,
            &metadata.cached_parts,
            frame_no,
            &mut self.instance.z_order,
            &mut self.instance.layer.parts_vec,
            &mut self.instance.layer.cell_vec,
            &mut self.instance.layer.parts_color_vec,
            &mut self.instance.layer.shader_vec,
            &mut self.instance.layer.vertice_vec,
            &mut self.instance.layer.deform_vec,
            &mut self.instance.layer.cursors,
            &mut self.instance.layer.cell_meta_per_part,
            &mut self.instance.layer.cell_texture_hashes,
            &mut self.instance.layer.local_vertices,
            self.instance.coordinate_system,
            self.instance.origin,
            self.instance.unit,
            metadata.canvas_width,
            metadata.canvas_height,
            &mut self.eval.draw_order_workspace,
            &mut self.eval.batches_workspace,
            &mut self.eval.mask_indices_workspace,
            &mut self.eval.deform_interpolation_start_workspace,
            &mut self.eval.deform_interpolation_end_workspace,
            &mut self.eval.deform_interpolation_out_workspace,
            &mut self.eval.deform_result_coordinate_workspace,
            &mut self.eval.deform_result_index_workspace,
            &mut self.eval.inheritance_alphas_workspace,
            &mut self.instance.layer.mesh_deform,
        );

        // Phase 3 hook: blend layers via `mix_poses(...)` between phase 1 and 2
        // once a second `LayerState` is available. The pipeline shape is the
        // intentional surface for that hand-off.

        // Phase 2: resolve global state (matrices / UVs / shape vertices)
        // from the (already-composed) local pose.
        Self::resolve_global_state(&mut self.instance, metadata);

        self.eval.fb.finish_minimal(offset);
        Ok(self.eval.fb.finished_data())
    }

    /// Phase 2 of the frame pipeline. Consumes the (post-mix) local pose
    /// in `instance.layer` and writes per-instance global outputs in
    /// `instance.globals` — world / inheritance matrices, local UVs,
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

    /// Writes per-part skinned mesh world-space vertex positions to
    /// `globals.mesh_vertices_x` / `_y`, using the prebuilt
    /// [`MeshBindingCache`](crate::core::framedata::MeshBindingCache) and the
    /// current frame's world matrices.
    ///
    /// Layout: x/y are SoA. `mesh_vertex_offsets` is a CSR prefix sum of length
    /// `num_parts + 1`; non-mesh parts contribute an empty range.
    ///
    /// Deform deltas are intentionally not consumed here yet — wiring the
    /// per-part deform workspace through Phase 2 is a follow-up. Deform-aware
    /// skinning would call `compute_skinned_mesh_vertices` with the optional
    /// `SkinnedMeshDeform` tuple populated from `LayerState`.
    #[allow(clippy::needless_range_loop)]
    fn compute_mesh_vertices(instance: &mut InstanceState<'a>, cached_parts: &[crate::core::framedata::CachedPartData]) {
        let layer = &instance.layer;
        // SSAB bind coordinates are authored Y-up; flip when the instance
        // (hence the bone world matrices) runs Y-down.
        let flip_y = instance.coordinate_system == crate::core::SsCoordinateSystem::YDown;
        let globals = &mut instance.globals;
        let num_parts = layer.parts_vec.len();

        if globals.mesh_vertex_offsets.len() != num_parts + 1 {
            globals.mesh_vertex_offsets.resize(num_parts + 1, 0);
        }

        // First pass: build the CSR offsets so we can size the SoA buffers.
        // The Cell is the authoritative geometry source (it covers both
        // skeletal and Deform-only meshes); the bind cache only adds skinning.
        globals.mesh_vertex_offsets[0] = 0;
        for i in 0..num_parts {
            let part_vertex_count = cached_parts.get(i).and_then(|c| c.mesh_cell.as_ref()).map(|m| m.num_vertices()).unwrap_or(0);
            globals.mesh_vertex_offsets[i + 1] = globals.mesh_vertex_offsets[i] + part_vertex_count as u32;
        }
        let total_vertices = *globals.mesh_vertex_offsets.last().unwrap_or(&0) as usize;

        if globals.mesh_vertices_x.len() != total_vertices {
            globals.mesh_vertices_x.resize(total_vertices, 0.0);
        }
        if globals.mesh_vertices_y.len() != total_vertices {
            globals.mesh_vertices_y.resize(total_vertices, 0.0);
        }
        for slot in globals.mesh_vertices_x.iter_mut() {
            *slot = 0.0;
        }
        for slot in globals.mesh_vertices_y.iter_mut() {
            *slot = 0.0;
        }

        // Static mesh UV / triangle-index buffers — resolved once from each
        // mesh part's frame-0 Cell (the cell is fixed per the SS mesh spec).
        // Guarded by the index-offset length so the build runs a single time.
        if globals.mesh_index_offsets.len() != num_parts + 1 {
            globals.mesh_index_offsets.resize(num_parts + 1, 0);
            globals.mesh_index_offsets[0] = 0;
            for i in 0..num_parts {
                let index_count = cached_parts.get(i).and_then(|c| c.mesh_cell.as_ref()).map(|m| m.indices.len()).unwrap_or(0);
                globals.mesh_index_offsets[i + 1] = globals.mesh_index_offsets[i] + index_count as u32;
            }
            let total_indices = *globals.mesh_index_offsets.last().unwrap_or(&0) as usize;

            globals.mesh_indices.clear();
            globals.mesh_indices.reserve(total_indices);
            globals.mesh_uvs.clear();
            globals.mesh_uvs.resize(total_vertices * 2, 0.0);
            for i in 0..num_parts {
                let Some(mesh_cell) = cached_parts.get(i).and_then(|c| c.mesh_cell.as_ref()) else {
                    continue;
                };
                globals.mesh_indices.extend_from_slice(&mesh_cell.indices);
                // UVs share the position CSR; copy only when the cell vertex
                // count matches the slot (true for well-formed skeletal meshes).
                let start = globals.mesh_vertex_offsets[i] as usize;
                let end = globals.mesh_vertex_offsets[i + 1] as usize;
                if mesh_cell.uvs.len() == (end - start) * 2 {
                    globals.mesh_uvs[start * 2..end * 2].copy_from_slice(&mesh_cell.uvs);
                }
            }
        }

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
                if binding.num_vertices() != end - start {
                    continue; // malformed: bind vertex count != cell vertex count
                }
                let deform = layer.mesh_deform.get(i).filter(|d| !d.indices.is_empty()).map(|d| {
                    (d.indices.as_slice(), d.dx.as_slice(), d.dy.as_slice(), &globals.world_matrices[i])
                });
                let (out_x, out_y) = (&mut globals.mesh_vertices_x[start..end], &mut globals.mesh_vertices_y[start..end]);
                crate::core::geometry::compute_skinned_mesh_vertices(&binding.influence_offsets, &binding.bone_indices, &binding.weights, &binding.coords_x, &binding.coords_y, &globals.world_matrices, deform, flip_y, out_x, out_y);
                continue;
            }

            // Deform-only mesh: no bones — rigidly transform the cell's rest
            // mesh by the part's own world matrix.
            let Some(mesh_cell) = cached.mesh_cell.as_ref() else {
                continue;
            };
            if mesh_cell.rest.len() != (end - start) * 2 || i >= globals.world_matrices.len() {
                continue;
            }
            let (m0, m1, m4, m5, m12, m13) = {
                let mat = &globals.world_matrices[i];
                (mat.m[0], mat.m[1], mat.m[4], mat.m[5], mat.m[12], mat.m[13])
            };
            for v in 0..(end - start) {
                let rx = mesh_cell.rest[v * 2];
                let ry = if flip_y { -mesh_cell.rest[v * 2 + 1] } else { mesh_cell.rest[v * 2 + 1] };
                globals.mesh_vertices_x[start + v] = m0 * rx + m4 * ry + m12;
                globals.mesh_vertices_y[start + v] = m1 * rx + m5 * ry + m13;
            }
            // Apply deform deltas — offsets are authored in the target's
            // orientation and applied unflipped (see evaluate_part).
            if let Some(deform) = layer.mesh_deform.get(i) {
                let vert_count = end - start;
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
        let layer = &instance.layer;
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
        let layer = &instance.layer;
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
        let layer = &instance.layer;
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

            let offset = i * stride;
            for v in 0..count {
                let mut y = vy[v] + ty;
                if coordinate_system == crate::core::SsCoordinateSystem::YDown {
                    y = -y;
                }
                globals.shape_vertices[offset + 2 * v] = vx[v] + tx;
                globals.shape_vertices[offset + 2 * v + 1] = y;

                let fx = (vx[v] + hw) / size_x;
                let fy = (hh - vy[v]) / size_y;
                globals.shape_vertex_box_coords[offset + 2 * v] = fx;
                globals.shape_vertex_box_coords[offset + 2 * v + 1] = fy;
            }
            globals.shape_vertex_counts[i] = count as i32;
        }
    }

    // Frame Number getter functions

    pub fn get_frame_no(&self) -> f32 {
        self.instance.controller.get_frame_no()
    }

    pub fn is_end_frame_reached(&self) -> bool {
        self.instance.controller.is_end_frame_reached()
    }

    pub fn is_looped(&self) -> bool {
        self.instance.controller.is_looped()
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
        self.instance.layer.cell_meta_per_part.as_ptr()
    }

    pub fn get_cell_meta_len(&self) -> usize {
        self.instance.layer.cell_meta_per_part.len()
    }

    pub fn get_local_vertices_ptr(&self) -> *const f32 {
        self.instance.layer.local_vertices.as_ptr()
    }

    pub fn get_local_vertices_len(&self) -> usize {
        self.instance.layer.local_vertices.len()
    }

    // setting getter functions
    pub fn is_playing(&self) -> bool {
        self.instance.controller.is_playing()
    }

    pub fn is_pausing(&self) -> bool {
        self.instance.controller.is_pausing()
    }

    pub fn get_skip_frames(&self) -> bool {
        self.instance.controller.get_skip_frames()
    }

    pub fn get_playback_direction(&self) -> core::playback_controller::PlaybackDirection {
        self.instance.controller.get_playback_direction()
    }

    pub fn get_playback_style(&self) -> core::playback_controller::PlaybackStyle {
        self.instance.controller.get_playback_style()
    }

    pub fn get_loops(&self) -> i32 {
        self.instance.controller.get_loops()
    }

    pub fn get_remain_loops(&self) -> i32 {
        self.instance.controller.get_remain_loops()
    }

    pub fn get_start_frame(&self) -> i32 {
        self.instance.controller.get_start_frame()
    }

    pub fn get_end_frame(&self) -> i32 {
        self.instance.controller.get_end_frame()
    }

    pub fn get_fps(&self) -> i32 {
        self.instance.controller.get_fps()
    }

    // Playback setup

    pub fn set_skip_frames(&mut self, skip: bool) {
        self.instance.controller.set_skip_frames(skip);
    }

    pub fn set_loop(&mut self, loops: i32) {
        self.instance.controller.set_loops(loops);
    }

    pub fn set_frame_rate(&mut self, fps: i32) {
        self.instance.controller.set_frame_rate(fps);
    }

    pub fn set_playback_direction(&mut self, direction: core::playback_controller::PlaybackDirection, style: core::playback_controller::PlaybackStyle) {
        self.instance.controller.set_playback_direction(direction, style);
    }

    pub fn set_animation_speed(&mut self, speed_rate: f32) {
        self.instance.controller.set_animation_speed(speed_rate);
    }

    pub fn set_animation_section(&mut self, start_frame: i32, end_frame: i32) {
        self.instance.controller.set_animation_section(start_frame, end_frame);
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
        let res = self.instance.controller.update(delta);

        self.collect_events();

        res
    }

    fn collect_events(&mut self) {
        self.instance.collected_event_indices.clear();
        let Some(resource) = self.eval.resource else {
            return;
        };
        if self.instance.animation_idx < 0 {
            return;
        }
        let metadata = &resource.animations[self.instance.animation_idx as usize];
        let Some(events_vec) = metadata.events else {
            return;
        };

        for &frame_no in self.instance.controller.get_passed_frames() {
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
                        self.instance.collected_event_indices.push(mid as i32);
                        break;
                    }
                    ::core::cmp::Ordering::Less => low = mid + 1,
                    ::core::cmp::Ordering::Greater => high = mid,
                }
            }
        }
    }

    pub fn get_passed_event_indices(&self) -> &[i32] {
        &self.instance.collected_event_indices
    }

    /// Resolve the most-recent `EventInstance` whose `frame_index <= current frame`
    /// for `slot_index` on the *currently selected animation*. Returns
    /// `synthetic_default()` when no event has fired yet (SS6 default semantics).
    /// Designed to live behind the `ss_runtime_get_active_event_instance` FFI.
    pub fn get_active_event_instance(&self, slot_index: u32) -> crate::types::ss_event_instance_info {
        let mut info = crate::types::ss_event_instance_info::synthetic_default();
        let Some(resource) = self.eval.resource else { return info };
        if self.instance.animation_idx < 0 {
            return info;
        }
        let metadata = &resource.animations[self.instance.animation_idx as usize];
        let Some(events) = metadata.events else { return info };

        let frame_no = self.instance.controller.get_frame_no();
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
                let Some(attr) = ev.value() else { continue };
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
        if self.instance.animation_idx < 0 {
            return info;
        }
        let metadata = &resource.animations[self.instance.animation_idx as usize];

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

        let frame_no = self.instance.controller.get_frame_no();
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
                let Some(attr) = ev.value() else { continue };
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
        if self.instance.animation_idx < 0 {
            return fallback;
        }
        let metadata = &resource.animations[self.instance.animation_idx as usize];
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
        self.instance.controller.play();
    }

    pub fn play_with_start_frame(&mut self, frame_no: f32) {
        self.instance.controller.play_with_start_frame(frame_no);
    }

    pub fn pause(&mut self) {
        self.instance.controller.pause();
    }

    pub fn stop(&mut self) {
        self.instance.controller.stop();
    }

    pub fn set_frame_no(&mut self, frame: f32) {
        self.instance.controller.set_frame_no(frame);
    }

    /// Sets the playback position based on `diff` (frames elapsed since
    /// playback started, with per-instance speed already applied), applying
    /// the configured section / loops / style / direction. Used by parent
    /// Players to drive child Instance / Effect Players deterministically
    /// across reverse, seek, and frame-skip operations.
    pub fn set_frame_relative(&mut self, diff: f32) {
        self.instance.controller.set_frame_relative(diff);
    }

    pub fn next_frame(&mut self) {
        self.instance.controller.next_frame();
    }

    pub fn prev_frame(&mut self) {
        self.instance.controller.prev_frame();
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
    // Cached at create time — never changes for a given effect resource.
    lifetime: i32,
    is_infinite: bool,
    // Transition detection: any change in (event_frame, is_synthetic, independent)
    // triggers reset/set_loop and re-seeds `accumulated_time`.
    last_event_frame: i32,
    last_is_synthetic: bool,
    last_independent: bool,
    // Independent-mode time accumulator (frame units, includes start_time bias).
    accumulated_time: f32,
}

impl<'a> EffectSlot<'a> {
    pub fn new(resource: &'a AnimationResource<'a>, effect_name_hash: u32, seed: u32) -> Result<Self, error::SSRuntimeError> {
        let ctx = EffectContext::new(resource, effect_name_hash, seed)?;
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(resource.data) };
        let lifetime = ctx.get_lifetime();
        let is_infinite = ctx.is_infinite();
        Ok(Self { ctx, ssab, fb: flatbuffers::FlatBufferBuilder::new(), emitter_resource_cache: alloc::vec::Vec::new(), lifetime, is_infinite, last_event_frame: -1, last_is_synthetic: false, last_independent: false, accumulated_time: 0.0 })
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

            let (src_x, src_y, sw, sh) = if let Some(rect) = cell.rectangle() { (rect.x1(), rect.y1(), rect.x2(), rect.y2()) } else { (0.0, 0.0, 0.0, 0.0) };
            // Pivot Y is negated to match Godot's prior emit_effect_slot behavior;
            // the Y-flip logic on the per-particle pass handles the rest of the
            // axis convention difference.
            let (pivot_x, pivot_y) = if let Some(p) = cell.pivot() { (p.v1(), -p.v2()) } else { (0.0, 0.0) };
            entry.sw = sw;
            entry.sh = sh;
            entry.quad_origin_x = -sw * 0.5 + pivot_x * sw;
            entry.quad_origin_y = -sh * 0.5 + pivot_y * sh;

            // UV uses cellmap.size_original — the texture dimensions at SSAB build time.
            // Atlas / scaled-texture pipelines must remap UVs engine-side.
            let (tex_w, tex_h) = if let Some(s) = cellmap.size_original() { (s.v1(), s.v2()) } else { (1.0, 1.0) };
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
        // Parent loop edge re-arms transition detection: the same EventEffect
        // identity (event_frame + is_synthetic) re-fires on the next playthrough,
        // and without this reset the equality check below would treat the loop
        // edge as a no-op (effect frozen at end of previous cycle).
        if parent_looped {
            self.last_event_frame = -1;
            self.last_is_synthetic = false;
        }

        let event_frame = ev.event_frame;
        let start_time = ev.start_time;
        let speed = ev.speed;
        let independent = ev.independent;
        let is_synthetic = ev.is_synthetic_default;

        let transitioned = self.last_event_frame != event_frame || self.last_is_synthetic != is_synthetic || self.last_independent != independent;

        if transitioned {
            self.ctx.reset();
            // Set-loop semantics carried over from the SS6 player: independent
            // effects loop within their lifetime, dependent effects play once
            // and the C++ caller previously checked elapsed-vs-lifetime. The
            // dead-effect early-return below covers the dependent finite case.
            self.ctx.set_loop(independent);
            self.last_event_frame = event_frame;
            self.last_is_synthetic = is_synthetic;
            self.last_independent = independent;
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

        // Dead-effect skip: dependent finite effect past its last particle's death.
        // Independent effects loop (set_loop above) so they never drift past lifetime.
        if !self.is_infinite && !independent && self.lifetime > 0 && relative_frame > self.lifetime as f32 {
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

        // Drive the simulator and copy command keys out before we touch other
        // `self` fields — the slices returned by `get_draw_commands` borrow from
        // `self.ctx.instance`, so we narrow the borrow to a tight scope.
        let cmd_keys: alloc::vec::Vec<(u32, u32, i32, usize, usize)> = {
            let (commands, _particles) = self.ctx.get_draw_commands();
            commands.iter().map(|c| (c.cell_map_name_hash, c.cell_name_hash, c.blend_type, c.particle_start, c.particle_end)).collect()
        };

        // Phase 1: resolve resources via split-borrow on disjoint self fields.
        let entries: alloc::vec::Vec<(usize, usize, usize)> = {
            let cache = &mut self.emitter_resource_cache;
            let ssab = &self.ssab;
            cmd_keys.into_iter().map(|(cmh, ch, bi, ps, pe)| (Self::ensure_emitter_resource_in(cache, ssab, cmh, ch, bi), ps, pe)).collect()
        };

        let total_quads: usize = entries.iter().map(|(_, s, e)| e - s).sum();
        if total_quads == 0 {
            return &[];
        }

        let mut verts: alloc::vec::Vec<f32> = alloc::vec::Vec::with_capacity(total_quads * 4 * stride);
        let mut uvs: alloc::vec::Vec<f32> = alloc::vec::Vec::with_capacity(total_quads * 4 * 2);
        let mut colors: alloc::vec::Vec<f32> = alloc::vec::Vec::with_capacity(total_quads * 4 * 4);
        let mut indices: alloc::vec::Vec<i32> = alloc::vec::Vec::with_capacity(total_quads * 6);

        // Snapshot resolved entries to avoid re-borrowing self.emitter_resource_cache
        // while writing into self.fb.
        struct Resolved {
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
        let resolved: alloc::vec::Vec<Resolved> = entries
            .iter()
            .map(|(i, _, _)| {
                let e = &self.emitter_resource_cache[*i];
                Resolved { ok: e.resolved_ok, blend_byte: e.blend_byte, quad_origin_x: e.quad_origin_x, quad_origin_y: e.quad_origin_y, sw: e.sw, sh: e.sh, u0: e.u0, v0: e.v0, u1: e.u1, v1: e.v1, cellmap_hash: e.cellmap_hash }
            })
            .collect();

        let mut command_meta: alloc::vec::Vec<(u32, u8, u32, u32)> = alloc::vec::Vec::with_capacity(entries.len());

        // Particles live on `self.ctx.instance.draw_particles`; borrow inline.
        let particles = &self.ctx.instance.draw_particles;

        for (cmd_idx, (_, p_start, p_end)) in entries.iter().enumerate() {
            let r = &resolved[cmd_idx];
            let particle_offset = (verts.len() / (4 * stride)) as u32;
            let particle_count = (p_end - p_start) as u32;

            if !r.ok {
                command_meta.push((r.cellmap_hash, r.blend_byte, 0, particle_offset));
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
                    verts.push(wx);
                    verts.push(wy);
                    if stride == 3 {
                        verts.push(0.0);
                    }
                }
                uvs.extend_from_slice(&uvs_corner);
                let r_f = p.color.r as f32 / 255.0;
                let g_f = p.color.g as f32 / 255.0;
                let b_f = p.color.b as f32 / 255.0;
                let a_f = p.color.a as f32 / 255.0;
                for _ in 0..4 {
                    colors.extend_from_slice(&[r_f, g_f, b_f, a_f]);
                }
                // Quad-local indices: command consumers slice [off*6 .. (off+qc)*6]
                // and pair it with the verts slice [off*4*stride .. (off+qc)*4*stride],
                // so values must be 0..qc*4 within the command, not global.
                let v = (i * 4) as i32;
                indices.extend_from_slice(&[v, v + 1, v + 2, v + 2, v + 1, v + 3]);
            }

            command_meta.push((r.cellmap_hash, r.blend_byte, particle_count, particle_offset));
        }

        // Build the FlatBuffer. Commands are inlined fixed-size structs, so
        // we collect them as POD then create the vector in one shot — no
        // per-element vtable/offset overhead.
        self.fb.reset();
        let cmds: alloc::vec::Vec<crate::runtime::effect_draw_plan::ss::runtime::EffectDrawCommand> =
            command_meta.iter().map(|(hash, blend, qc, off)| crate::runtime::effect_draw_plan::ss::runtime::EffectDrawCommand::new(*hash, *qc, *off, *blend)).collect();
        let commands_off = self.fb.create_vector(&cmds);
        let verts_off = self.fb.create_vector(&verts);
        let uvs_off = self.fb.create_vector(&uvs);
        let colors_off = self.fb.create_vector(&colors);
        let indices_off = self.fb.create_vector(&indices);

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
}

impl Default for InstanceSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceSlot {
    pub const fn new() -> Self {
        Self { last_event_frame: -1, last_is_synthetic: false, last_independent: false }
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
        if parent_looped {
            self.last_event_frame = -1;
            self.last_is_synthetic = false;
        }

        let transitioned = self.last_event_frame == -1 || self.last_event_frame != ev.event_frame || self.last_is_synthetic != ev.is_synthetic_default;

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
        }

        // Branch on `independent`. Synced children are seeked deterministically
        // from the parent's frame each tick — same `parent_frame` always yields
        // the same `child_frame`. Independent children own their own controller
        // time: forward `delta_seconds` to `update`, except on the transition
        // tick itself where `play()` will snap to start_frame (any further step
        // here would visibly skip the first frame).
        let child_frame_no = if ev.independent {
            if delta_seconds > 0.0 && !transitioned {
                let d_ms = delta_seconds * 1000.0 * ev.speed;
                child_ctx.update(d_ms)
            } else {
                child_ctx.get_frame_no()
            }
        } else {
            let diff = (parent_frame - ev.event_frame as f32) * ev.speed;
            child_ctx.set_frame_relative(diff);
            child_ctx.get_frame_no()
        };

        let child_looped = ev.independent && delta_seconds > 0.0 && child_frame_no < previous_child_frame && !transitioned;

        self.last_independent = ev.independent;

        crate::types::ss_instance_step_result { visible: true, transitioned, independent: ev.independent, child_frame_no, child_looped }
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
        let num_parts = context.instance.layer.parts_vec.len();
        assert_eq!(world_matrices_len, num_parts * 16);

        let world_matrices = unsafe { ::core::slice::from_raw_parts(world_matrices_ptr, world_matrices_len) };

        // For simplicity, just test that matrices are non-zero/valid
        assert!(world_matrices[0] != 0.0 || world_matrices[1] != 0.0);
    }

    #[test]
    fn test_local_uvs() {
        prepare();
        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let _ = context.get_frame_data(0.0).expect("Failed to get frame data");

        let uvs_ptr = context.get_local_uvs_ptr();
        let uvs_len = context.get_local_uvs_len();
        let num_parts = context.instance.layer.parts_vec.len();
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
        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let _ = context.get_frame_data(0.0).expect("Failed to get frame data");

        let meta_ptr = context.get_cell_meta_ptr();
        let meta_len = context.get_cell_meta_len();
        let num_parts = context.instance.layer.parts_vec.len();
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
        if let Some(animation) = ssab.animations().and_then(|animes| animes.iter().find(|a| a.name() == "anime_1"))
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
        let ssab_path = get_test_base_path().join("tests/allPartsV7/allPartsV7_ssab/shape.ssab");
        let input_buf = std::fs::read(&ssab_path).expect("Failed to read shape.ssab");

        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = AnimationResource::new(ssab_data.as_slice()).expect("Failed to load resource");
        let mut context = Context::new();
        context.bind_resource(&resource);
        context.setup_animation("anime_1").expect("Failed to setup animation");

        let ssab_buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
        let parts_meta = ssab.parts().expect("No parts in ssab");
        let shape_idx = parts_meta.iter().position(|p| p.name() == Some("shape_1")).expect("shape_1 not found") as i16;

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
