//! The C ABI — how every non-Rust Player talks to the runtime.
//!
//! A flat set of `extern "C"` functions mirroring the Rust API, plus the opaque handles
//! that carry state across the boundary. `cbindgen` turns this module into `ssruntime.h`
//! at build time; that header, not this page, is what a C or C++ integrator reads.
//!
//! # Handles
//!
//! Two opaque pointers, both `*mut c_void` to C:
//!
//! - **Resource** (`ss_resource_create_copy` / `_borrow` → `ss_resource_destroy`) — parsed
//!   `.ssab` data. Internally reference-counted, so it is safe to destroy a resource that
//!   contexts are still bound to; the buffer stays alive until the last one lets go.
//!   The `_borrow` variant does *not* copy, so the caller's buffer must outlive it.
//! - **Context** ([`ss_runtime_create`] → [`ss_runtime_destroy`]) — the playback state of
//!   one animation. Bind a resource with [`ss_runtime_bind_resource`]; several contexts
//!   may share one.
//!
//! Effect and instance parts get their own handles ([`ss_effect_slot_create`],
//! [`ss_instance_slot_create`]), stepped once per tick by the Player.
//!
//! # Calling convention
//!
//! Fallible calls return `bool` (or a null pointer, or [`SS_RUNTIME_INVALID_HANDLE`]) and
//! record the reason on the context; read it back with `ss_runtime_get_last_error` and
//! compare against [`crate::error::SSRuntimeError::as_code`]. Getters that return a float
//! use `NaN` for "no such thing", which is distinguishable from a legitimate `0.0`.
//!
//! Buffers handed out through `out_data` / `out_len` pairs point into runtime-owned
//! memory and stay valid only until the next call that rewrites them. Copy anything you
//! need to keep.
//!
//! # Validation
//!
//! Loading does not validate the `.ssab` — see the crate docs. Every entry point that
//! takes a data pointer trusts it.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_lazy_continuation)]

pub use crate::types::*;

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::ffi::{CStr, c_char, c_float, c_int, c_uchar, c_void};
use core::sync::atomic::{AtomicPtr, Ordering};

// Placeholder for eprintln! in no_std
#[allow(unused_macros)]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        $crate::ffi::ss_log!(3, $($arg)*)
    };
}

pub type SsLogCallback = extern "C" fn(level: core::ffi::c_int, message: *const core::ffi::c_char);
#[doc(hidden)]
pub static LOG_CALLBACK: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_log_callback(callback: SsLogCallback) {
    LOG_CALLBACK.store(callback as *mut c_void, Ordering::Relaxed);
}

#[macro_export]
macro_rules! ss_log {
    ($level:expr, $($arg:tt)*) => {
        unsafe {
            let cb_ptr = $crate::ffi::LOG_CALLBACK.load(::core::sync::atomic::Ordering::Relaxed);
            if !cb_ptr.is_null() {
                let cb: $crate::ffi::SsLogCallback = ::core::mem::transmute(cb_ptr);
                if let Ok(s) = ::alloc::ffi::CString::new(::alloc::format!($($arg)*)) {
                    cb($level, s.as_ptr());
                }
            }
        }
    };
}

use crate::SsabResource;

pub struct FfiContext {
    pub ctx: crate::Context<'static>,
    // We hold the resource here to keep it alive
    pub bound_resource: Option<Arc<SsabResource>>,
    pub last_error: i32,
}
impl Default for FfiContext {
    fn default() -> Self {
        Self::new()
    }
}

impl FfiContext {
    pub fn new() -> Self {
        Self { ctx: crate::Context::new(), bound_resource: None, last_error: 0 }
    }
}

#[inline(always)]
fn get_context<'a>(ptr: *mut c_void) -> Option<&'a mut FfiContext> {
    if ptr.is_null() {
        return None;
    }
    unsafe { Some(&mut *(ptr as *mut FfiContext)) }
}

fn create_ffi_resource(data: crate::SsabData) -> *mut c_void {
    let arc = SsabResource::new(data);
    if arc.resource().is_none() {
        return core::ptr::null_mut();
    }
    Arc::into_raw(arc) as *mut c_void
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Creates an AnimationResource by copying the provided data.
///
/// # Security and Validation
/// To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
/// Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
/// The caller MUST ensure the data is trusted or validated before passing it to this function.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid memory block.
pub unsafe extern "C" fn ss_resource_create_copy(ptr: *const c_uchar, len: usize) -> *mut c_void {
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    let data = unsafe { ::core::slice::from_raw_parts(ptr, len) };
    match crate::SsabData::load_copy(data) {
        Ok(ssab_data) => create_ffi_resource(ssab_data),
        Err(_) => core::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Creates an AnimationResource by borrowing the provided data.
///
/// # Security and Validation
/// To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
/// Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
/// The caller MUST ensure the data is trusted or validated before passing it to this function.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid memory block of at least `len` bytes,
/// AND the memory must remain valid and unmodified until this resource is destroyed.
pub unsafe extern "C" fn ss_resource_create_borrow(ptr: *const c_uchar, len: usize) -> *mut c_void {
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    match unsafe { crate::SsabData::load_borrow(ptr, len) } {
        Ok(ssab_data) => create_ffi_resource(ssab_data),
        Err(_) => core::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_resource_destroy(resource: *mut c_void) {
    if resource.is_null() {
        return;
    }
    unsafe {
        let _ = Arc::from_raw(resource as *mut SsabResource);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_create() -> *mut c_void {
    let c = FfiContext::new();
    Box::into_raw(Box::new(c)) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_destroy(context: *mut c_void) {
    if let Some(context) = get_context(context) {
        unsafe {
            let _ = Box::from_raw(context as *mut FfiContext);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_reset(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.reset();
        ffi_context.bound_resource = None;
        ffi_context.last_error = 0;
    }
}

/// Binds an AnimationResource to a Context.
///
/// # Safety and Lifetimes
/// - To save memory, a single Resource can be bound to multiple Contexts.
/// - **Lifetime Safety:** This implementation uses `Arc` for internal reference counting.
///   It is safe to destroy the `resource` (via `ss_resource_destroy`) even if it is still bound
///   to contexts; the memory will be kept alive until all bound contexts are also destroyed or reset.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_bind_resource(context: *mut c_void, resource: *mut c_void) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    if resource.is_null() {
        return false;
    }

    // Safety: Increment refcount by cloning from the raw pointer.
    let arc = unsafe { Arc::from_raw(resource as *mut SsabResource) };
    let arc_clone = arc.clone();
    let _ = Arc::into_raw(arc); // Keep original pointer valid

    // Use a raw pointer to break the borrow on `arc_clone` so it can be moved
    // into `bound_resource` before the pointer is dereferenced. The pointer
    // stays valid because `bound_resource` owns the surrounding Arc.
    let res_ptr = arc_clone.resource_static_ptr();
    if res_ptr.is_null() {
        return false;
    }
    ffi_context.bound_resource = Some(arc_clone);
    ffi_context.ctx.bind_resource(unsafe { &*res_ptr });
    true
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid memory block.
pub unsafe extern "C" fn ss_runtime_load_ssab_copy(context: *mut c_void, ptr: *const c_uchar, len: usize) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    if ptr.is_null() {
        ffi_context.last_error = crate::error::SSRuntimeError::InvalidParameter.as_code();
        return false;
    }

    let data = unsafe { ::core::slice::from_raw_parts(ptr, len) };
    match crate::SsabData::load_copy(data) {
        Ok(ssab_data) => {
            let res_ptr = create_ffi_resource(ssab_data);
            if res_ptr.is_null() {
                ffi_context.last_error = crate::error::SSRuntimeError::MemoryError.as_code();
                return false;
            }
            let success = ss_runtime_bind_resource(context, res_ptr);
            ss_resource_destroy(res_ptr);
            success
        }
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid memory block.
pub unsafe extern "C" fn ss_runtime_load_ssab_borrow(context: *mut c_void, ptr: *const c_uchar, len: usize) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    if ptr.is_null() {
        ffi_context.last_error = crate::error::SSRuntimeError::InvalidParameter.as_code();
        return false;
    }

    match unsafe { crate::SsabData::load_borrow(ptr, len) } {
        Ok(ssab_data) => {
            let res_ptr = create_ffi_resource(ssab_data);
            if res_ptr.is_null() {
                ffi_context.last_error = crate::error::SSRuntimeError::MemoryError.as_code();
                return false;
            }
            let success = ss_runtime_bind_resource(context, res_ptr);
            ss_resource_destroy(res_ptr);
            success
        }
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_ssab(context: *mut c_void, out_data: *mut *const c_uchar, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        if let Some(ssab_buf) = ffi_context.ctx.get_ssab_data() {
            unsafe {
                if !out_data.is_null() {
                    *out_data = ssab_buf.as_ptr() as *const c_uchar;
                }
                if !out_len.is_null() {
                    *out_len = ssab_buf.len();
                }
            }
        } else {
            ffi_context.last_error = crate::error::SSRuntimeError::NotLoaded.as_code();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_ssab_buf(context: *mut c_void) -> *const c_uchar {
    if let Some(ffi_context) = get_context(context) {
        if let Some(ssab_buf) = ffi_context.ctx.get_ssab_data() {
            return ssab_buf.as_ptr() as *const c_uchar;
        } else {
            ffi_context.last_error = crate::error::SSRuntimeError::NotLoaded.as_code();
        }
    }
    core::ptr::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_ssab_len(context: *mut c_void) -> usize {
    if let Some(ffi_context) = get_context(context) {
        if let Some(ssab_buf) = ffi_context.ctx.get_ssab_data() {
            return ssab_buf.len();
        } else {
            ffi_context.last_error = crate::error::SSRuntimeError::NotLoaded.as_code();
        }
    }
    0
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid null-terminated string.
pub unsafe extern "C" fn ss_runtime_setup_animation(context: *mut c_void, name: *const c_char) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    if name.is_null() {
        ffi_context.last_error = crate::error::SSRuntimeError::InvalidParameter.as_code();
        return false;
    }
    let c_buf = unsafe { CStr::from_ptr(name) };
    let a = c_buf.to_str().unwrap_or_default();
    match ffi_context.ctx.setup_animation(a) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe extern "C" fn ss_runtime_setup_animation_by_hash(context: *mut c_void, name_hash: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    match ffi_context.ctx.setup_animation_by_hash(name_hash) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_frame_data(context: *mut c_void, frame_no: c_float, out_data: *mut *mut c_uchar, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        match ffi_context.ctx.get_frame_data(frame_no) {
            Ok(ret) => unsafe {
                if !out_data.is_null() {
                    *out_data = ret.as_ptr() as *mut c_uchar;
                }
                if !out_len.is_null() {
                    *out_len = ret.len();
                }
            },
            Err(e) => {
                ffi_context.last_error = e.as_code();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_world_matrices(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_world_matrices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_world_matrices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_local_uvs(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_local_uvs_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_local_uvs_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Per-part resolved cell metadata for the current frame, written by
/// `ss_runtime_get_frame_data`. Layout:
/// `[pivot_x, pivot_y, size_w, size_h, rect_left, rect_top]` per part,
/// indexed by `part_index * 6`. `(rect_left, rect_top)` is the cell
/// rectangle origin in the cellmap's texture pixel space, letting a
/// Player build a complete source rect (origin + size) without its own
/// cellmap lookup. Parts without a current cell carry zeros.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_cell_meta(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_cell_meta_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_cell_meta_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Per-part local-space vertex positions for the current frame, written by
/// `ss_runtime_get_frame_data`. Layout per part:
/// `[lt.x, lt.y, rt.x, rt.y, lb.x, lb.y, rb.x, rb.y, center.x, center.y]`
/// = 10 floats, indexed by `part_index * 10`. Coordinates are in part-local
/// space with `coordinate_system` (Y-Up/Y-Down) flip already applied, so a
/// Player just multiplies them by the world matrix. Includes pivot, image
/// flip, and per-frame vertex deformation. Parts without a current cell
/// carry zeros. Replaces `FrameData::vertices` for renderers that consume
/// pre-resolved SoA buffers.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_local_vertices(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_local_vertices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_local_vertices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Per-part shape vertex positions for the current frame, written by
/// `ss_runtime_get_frame_data`. Layout: 12 vertices × `(x, y)` = 24 floats
/// per part, indexed by `part_index * 24`. Vertices are in local space,
/// post-pivot, post-coord-system, so a Player just multiplies them by the
/// world matrix. Only the first
/// `ss_runtime_get_shape_vertex_counts(...)[part_index]` vertices are
/// valid; remaining slots are zero-initialized. Non-shape parts carry zero
/// count and zero-filled vertex slots.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_shape_vertices(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_shape_vertices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_shape_vertices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Per-vertex normalized bounding-box coordinates paired with
/// `ss_runtime_get_shape_vertices`. Layout: 12 vertices × `(fx, fy)` = 24
/// floats per part, indexed by `part_index * 24`. `(0, 0)` corresponds to
/// the LT corner of the shape's pre-pivot bounding box, `(1, 1)` to the RB
/// corner, regardless of pivot offset or coordinate system. Players use
/// this to interpolate the four part-color corners without re-deriving the
/// box layout.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_shape_vertex_box_coords(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_shape_vertex_box_coords_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_shape_vertex_box_coords_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Per-part valid vertex count for `ss_runtime_get_shape_vertices`. Indexed
/// by `part_index`. Zero for parts that are not shape parts (or shape
/// parts whose size collapsed to zero).
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_shape_vertex_counts(context: *mut c_void, out_data: *mut *const i32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_shape_vertex_counts_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_shape_vertex_counts_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Skinned mesh per-vertex X coordinates in world space, concatenated across all
/// parts. Pair with `ss_runtime_get_mesh_vertex_offsets` to slice per-part
/// ranges: part `p`'s vertices live at indices `offsets[p]..offsets[p + 1]`.
/// Non-mesh parts contribute an empty range. Indirect render mode leaves the
/// buffer empty (length 0).
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_vertices_x(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_vertices_x_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_vertices_x_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Skinned mesh per-vertex Y coordinates. SoA companion to
/// `ss_runtime_get_mesh_vertices_x`; sliced identically via
/// `ss_runtime_get_mesh_vertex_offsets`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_vertices_y(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_vertices_y_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_vertices_y_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// CSR-style prefix sum (length = `num_parts + 1`) describing where each
/// part's skinned mesh vertices live inside `ss_runtime_get_mesh_vertices_x`
/// / `_y`. Part `p` occupies indices `[offsets[p], offsets[p + 1])`; non-mesh
/// parts have `offsets[p] == offsets[p + 1]`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_vertex_offsets(context: *mut c_void, out_data: *mut *const u32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_vertex_offsets_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_vertex_offsets_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Mesh per-vertex UV coordinates in cellmap texture-pixel space, interleaved
/// `[u, v, ...]`, concatenated across all parts. Static — resolved once from
/// each mesh part's frame-0 Cell. Sliced per-part by
/// `ss_runtime_get_mesh_vertex_offsets`, identically to the vertex buffers.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_uvs(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_uvs_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_uvs_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Mesh triangle indices (part-local, 0-based), concatenated across all parts.
/// Static. Slice part `p`'s range via `ss_runtime_get_mesh_index_offsets`:
/// `[offsets[p], offsets[p + 1])`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_indices(context: *mut c_void, out_data: *mut *const i32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_indices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_indices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// CSR-style prefix sum (length = `num_parts + 1`) for the triangle indices in
/// `ss_runtime_get_mesh_indices`. Part `p` occupies `[offsets[p], offsets[p + 1])`;
/// non-mesh parts have an empty range.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_mesh_index_offsets(context: *mut c_void, out_data: *mut *const u32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_mesh_index_offsets_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_mesh_index_offsets_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Nine-Slice per-vertex positions for the current frame, interleaved
/// `[x, y, ...]` and concatenated across all parts, written by
/// `ss_runtime_get_frame_data`. Slice per part with
/// `ss_runtime_get_nines_vertex_offsets`; parts drawn by another
/// `DrawBatchKind` occupy an empty range.
///
/// Coordinates are part-local with pivot, image flip and the
/// `coordinate_system` (Y-Up/Y-Down) flip already applied — exactly like
/// `ss_runtime_get_local_vertices` — so a Player just multiplies them by the
/// part's world matrix. The grid is a 4x4 sheet for `nines_fill_mode == 0`
/// (stretch) and independent quads per repeat for `== 1` (tile).
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_vertices(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_vertices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_vertices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Nine-Slice per-vertex UVs in cellmap texture-pixel space, interleaved
/// `[u, v, ...]`. Shares the vertex CSR of
/// `ss_runtime_get_nines_vertex_offsets`. Divide by the texture size to
/// normalize, the same as `ss_runtime_get_local_uvs`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_uvs(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_uvs_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_uvs_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Nine-Slice per-vertex box coordinates, interleaved `[fx, fy, ...]` with
/// `0` at the part's left / top edge and `1` at its right / bottom edge.
/// Shares the vertex CSR of `ss_runtime_get_nines_vertex_offsets`.
///
/// Use them to bilinearly interpolate the four `part_color` corners across the
/// grid — the same weighting `ss_runtime_get_shape_vertex_box_coords` provides
/// for Shape parts. A part flip is already folded in, so `fx` / `fy` always
/// describe the drawn box.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_box_coords(context: *mut c_void, out_data: *mut *const c_float, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_box_coords_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_box_coords_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// Nine-Slice triangle indices, concatenated across all parts and sliced by
/// `ss_runtime_get_nines_index_offsets`. Values are **part-local**
/// (`0..vertex_count`), so a Player passes them alongside the part's vertex
/// slice without rebasing. Each quad is wound `LT, RT, LB / LB, RT, RB`.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_indices(context: *mut c_void, out_data: *mut *const i32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_indices_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_indices_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// CSR-style prefix sum (length = `num_parts + 1`) over the vertex buffers
/// `ss_runtime_get_nines_vertices` / `_uvs` / `_box_coords`. Part `p` occupies
/// vertices `[offsets[p], offsets[p + 1])` — multiply by 2 to index the
/// interleaved floats. Non-nines parts have an empty range.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_vertex_offsets(context: *mut c_void, out_data: *mut *const u32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_vertex_offsets_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_vertex_offsets_len();
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// CSR-style prefix sum (length = `num_parts + 1`) for the triangle indices in
/// `ss_runtime_get_nines_indices`. Part `p` occupies
/// `[offsets[p], offsets[p + 1])`; non-nines parts have an empty range.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_nines_index_offsets(context: *mut c_void, out_data: *mut *const u32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_nines_index_offsets_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_nines_index_offsets_len();
            }
        }
    }
}

// Frame Number getter functions
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_no(context: *mut c_void) -> c_float {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_frame_no();
    }
    0.0
}

/// Returns `true` once the configured loops have completed, staying `true`
/// until the next `ss_runtime_play` / `ss_runtime_setup_animation`. Always
/// `false` under an infinite loop count, and never raised by
/// `ss_runtime_stop`.
///
/// This is a *state*, not a pulse: to notify once, take its rising edge.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_finished(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.is_finished();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_looped(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.is_looped();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_playing(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.is_playing();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_playing_forward(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.is_playing_forward();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_pausing(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.is_pausing();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_skip_frames(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_skip_frames();
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_playback_direction(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) { if ffi_context.ctx.get_playback_direction() == crate::core::playback_controller::PlaybackDirection::Forward { 0 } else { 1 } } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_playback_style(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_playback_style() as i32;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_loops(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_loops();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_remain_loops(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_remain_loops();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_start_frame(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_start_frame();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_end_frame(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_end_frame();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_fps(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_fps();
    }
    0
}

// Playback setup
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_skip_frames(context: *mut c_void, skip: bool) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_skip_frames(skip);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_loop(context: *mut c_void, loops: c_int) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_loop(loops);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_rate(context: *mut c_void, fps: c_int) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_frame_rate(fps);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_playback_direction(context: *mut c_void, direction: c_int, style: c_int) {
    if let Some(ffi_context) = get_context(context) {
        let d = if direction == 0 { crate::core::playback_controller::PlaybackDirection::Forward } else { crate::core::playback_controller::PlaybackDirection::Backward };
        let s = if style == 0 { crate::core::playback_controller::PlaybackStyle::Normal } else { crate::core::playback_controller::PlaybackStyle::PingPong };
        ffi_context.ctx.set_playback_direction(d, s);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_animation_speed(context: *mut c_void, speed_rate: c_float) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_animation_speed(speed_rate);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_animation_section(context: *mut c_void, start_frame: c_int, end_frame: c_int) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_animation_section(start_frame, end_frame);
    }
}

// Playback control

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_update(context: *mut c_void, delta: f32) -> c_float {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.update(delta);
    }
    0.0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.play();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play_with_start_frame(context: *mut c_void, frame_no: f32) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.play_with_start_frame(frame_no);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_pause(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.pause();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_stop(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.stop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_no(context: *mut c_void, frame: c_float) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_frame_no(frame);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_next_frame(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.next_frame();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_prev_frame(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.prev_frame();
    }
}

/// Run a fallible `Context` call, recording the error code on failure. Every
/// bool-returning track entry point funnels through this so the `last_error`
/// contract is stated once.
fn report<F>(ffi_context: &mut FfiContext, f: F) -> bool
where
    F: FnOnce(&mut crate::Context<'static>) -> Result<(), crate::error::SSRuntimeError>,
{
    match f(&mut ffi_context.ctx) {
        Ok(()) => true,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

/// # Safety
/// `p` must be null or a valid null-terminated string.
unsafe fn cstr_to_string(p: *const c_char) -> Option<alloc::string::String> {
    if p.is_null() {
        return None;
    }
    let c_buf = unsafe { CStr::from_ptr(p) };
    Some(alloc::string::String::from_utf8_lossy(c_buf.to_bytes()).into_owned())
}

/// Sentinel returned by the handle-returning track functions on failure.
pub const SS_RUNTIME_INVALID_HANDLE: u32 = u32::MAX;

/// Track blend operators, mirroring [`crate::TrackBlend`].
pub const SS_RUNTIME_TRACK_BLEND_MIX: u32 = 0;
pub const SS_RUNTIME_TRACK_BLEND_ADDITIVE: u32 = 1;

// Animation mixing — the track stack. Tracks fold onto the setup pose in
// ascending order: `alpha` is a track's strength, `weight` only decides
// proportions among several sources sharing one track. Track 0 carries the
// structure (discrete attributes and events) and is what `setup_animation`
// establishes. See `libs/ssruntime/docs` for the model.

/// # Safety
/// `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn ss_runtime_set_track_animation(context: *mut c_void, track: u32, name: *const c_char) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    let Some(a) = (unsafe { cstr_to_string(name) }) else {
        ffi_context.last_error = crate::error::SSRuntimeError::InvalidParameter.as_code();
        return false;
    };
    report(ffi_context, |ctx| ctx.set_track_animation(track, &a))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_track_animation_by_hash(context: *mut c_void, track: u32, hash: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.set_track_animation_by_hash(track, hash))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_track_alpha(context: *mut c_void, track: u32, alpha: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.set_track_alpha(track, alpha))
}

/// Returns `NaN` for a track that does not exist, so callers can tell it apart
/// from a valid `alpha` of 0.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_track_alpha(context: *mut c_void, track: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.track_alpha(track)).unwrap_or(f32::NAN)
}

/// `blend` is one of `SS_RUNTIME_TRACK_BLEND_*`.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_track_blend(context: *mut c_void, track: u32, blend: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    let blend = match blend {
        SS_RUNTIME_TRACK_BLEND_MIX => crate::TrackBlend::Mix,
        SS_RUNTIME_TRACK_BLEND_ADDITIVE => crate::TrackBlend::Additive,
        _ => {
            ffi_context.last_error = crate::error::SSRuntimeError::InvalidParameter.as_code();
            return false;
        }
    };
    report(ffi_context, |ctx| ctx.set_track_blend(track, blend))
}

/// Returns `SS_RUNTIME_INVALID_HANDLE` for a track that does not exist.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_track_blend(context: *mut c_void, track: u32) -> u32 {
    match get_context(context).and_then(|c| c.ctx.track_blend(track)) {
        Some(crate::TrackBlend::Mix) => SS_RUNTIME_TRACK_BLEND_MIX,
        Some(crate::TrackBlend::Additive) => SS_RUNTIME_TRACK_BLEND_ADDITIVE,
        None => SS_RUNTIME_INVALID_HANDLE,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_track_count(context: *mut c_void) -> u32 {
    get_context(context).map_or(0, |c| c.ctx.track_count())
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_clear_track(context: *mut c_void, track: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.clear_track(track))
}

/// Fade `track` to `name` over `duration_ms` — the same time unit as
/// `ss_runtime_update`'s delta.
///
/// # Safety
/// `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn ss_runtime_crossfade(context: *mut c_void, track: u32, name: *const c_char, duration_ms: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    let Some(a) = (unsafe { cstr_to_string(name) }) else {
        ffi_context.last_error = crate::error::SSRuntimeError::InvalidParameter.as_code();
        return false;
    };
    report(ffi_context, |ctx| ctx.crossfade(track, &a, duration_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_crossfade_by_hash(context: *mut c_void, track: u32, hash: u32, duration_ms: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.crossfade_by_hash(track, hash, duration_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_fade_track(context: *mut c_void, track: u32, target_alpha: c_float, duration_ms: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.fade_track(track, target_alpha, duration_ms))
}

/// Progress in `[0, 1]` of the fade on `track`; `NaN` when it is not fading.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_track_fade_progress(context: *mut c_void, track: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.track_fade_progress(track)).unwrap_or(f32::NAN)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_cancel_track_fade(context: *mut c_void, track: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.cancel_track_fade(track))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_track_frame(context: *mut c_void, track: u32, frame: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.set_track_frame(track, frame))
}

/// Returns `NaN` when the track has no animation on it.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_track_frame(context: *mut c_void, track: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.track_frame(track)).unwrap_or(f32::NAN)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_track_speed(context: *mut c_void, track: u32, speed: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.set_track_speed(track, speed))
}

/// Handle of the source carrying the track's animation, so it can be given a
/// weight once the track holds several. `SS_RUNTIME_INVALID_HANDLE` when the
/// track has no animation.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_track_main_source(context: *mut c_void, track: u32) -> u32 {
    get_context(context).and_then(|c| c.ctx.track_main_source(track)).unwrap_or(SS_RUNTIME_INVALID_HANDLE)
}

/// Add another weighted source to `track` (locomotion-style blending). Returns
/// `SS_RUNTIME_INVALID_HANDLE` on failure.
///
/// # Safety
/// `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn ss_runtime_track_add_source(context: *mut c_void, track: u32, name: *const c_char, weight: c_float) -> u32 {
    let Some(ffi_context) = get_context(context) else {
        return SS_RUNTIME_INVALID_HANDLE;
    };
    ffi_context.last_error = 0;
    let Some(a) = (unsafe { cstr_to_string(name) }) else {
        ffi_context.last_error = crate::error::SSRuntimeError::InvalidParameter.as_code();
        return SS_RUNTIME_INVALID_HANDLE;
    };
    match ffi_context.ctx.track_add_source(track, &a, weight) {
        Ok(h) => h,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            SS_RUNTIME_INVALID_HANDLE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_track_add_source_by_hash(context: *mut c_void, track: u32, hash: u32, weight: c_float) -> u32 {
    let Some(ffi_context) = get_context(context) else {
        return SS_RUNTIME_INVALID_HANDLE;
    };
    ffi_context.last_error = 0;
    match ffi_context.ctx.track_add_source_by_hash(track, hash, weight) {
        Ok(h) => h,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            SS_RUNTIME_INVALID_HANDLE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_source_weight(context: *mut c_void, handle: u32, weight: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.set_source_weight(handle, weight))
}

/// Returns `NaN` for an invalid or released handle, so `weight == 0` stays
/// distinguishable from "no such source".
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_source_weight(context: *mut c_void, handle: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.source_weight(handle)).unwrap_or(f32::NAN)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_source_frame(context: *mut c_void, handle: u32, frame: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.set_source_frame(handle, frame))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_source_frame(context: *mut c_void, handle: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.source_frame(handle)).unwrap_or(f32::NAN)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_source_event_threshold(context: *mut c_void, handle: u32, threshold: c_float) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    report(ffi_context, |ctx| ctx.set_source_event_threshold(handle, threshold))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_source_event_threshold(context: *mut c_void, handle: u32) -> c_float {
    get_context(context).and_then(|c| c.ctx.source_event_threshold(handle)).unwrap_or(f32::NAN)
}

/// Create a new EffectSlot bound to `resource`. Returns null on failure
/// (resource not loaded, effect_name_hash not found, malformed effect data).
#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_create(resource: *const c_void, effect_name_hash: u32, seed: u32) -> *mut c_void {
    if resource.is_null() {
        return core::ptr::null_mut();
    }
    let res = unsafe { &*(resource as *const crate::SsabResource) };
    let Some(r) = res.resource() else { return core::ptr::null_mut() };
    // SAFETY: Lifetime widening matches the existing `EffectContext` FFI pattern.
    // The caller owns `resource` and must keep it alive for the slot's lifetime
    // (the underlying `Arc<SsabResource>` ensures the SSAB buffer outlives the slot).
    let r_static: &'static crate::AnimationResource<'static> = unsafe { core::mem::transmute(r) };
    match crate::EffectSlot::new(r_static, effect_name_hash, seed) {
        Ok(slot) => Box::into_raw(Box::new(slot)) as *mut c_void,
        Err(_) => core::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_destroy(slot: *mut c_void) {
    if slot.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(slot as *mut crate::EffectSlot);
    }
}

/// Reset the slot's playback / "last started key" memory to its just-created
/// state without destroying it. Call on an explicit head-out (`ss_runtime_play`
/// / replay) so a re-played animation re-fires its effect from scratch instead
/// of debouncing the first key as the same one the slot last started. The
/// resource binding and emitter caches are preserved.
#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_reset(slot: *mut c_void) {
    if slot.is_null() {
        return;
    }
    let slot = unsafe { &mut *(slot as *mut crate::EffectSlot) };
    slot.reset();
}

/// Advance the slot one frame. The caller resolves the active effect event
/// (via `ss_runtime_get_active_effect_event`) and feeds it here together with
/// timing inputs. `parent_looped` re-arms transition detection across the
/// parent's loop edge.
///
/// `y_flip = true` mirrors particle Y position and rotation sign, for Y-down
/// rendering of Y-up authored content (Godot, pixi.js). Pass `false` for
/// Y-up engines (Unity 2D).
///
/// `vert_stride` selects the vertex layout of the returned `EffectDrawPlan`:
/// `2` packs (x, y) pairs (Godot, pixi.js, wgpu); `3` packs (x, y, 0) triples
/// (Unity Mesh, Unreal procedural mesh). Other values fall back to `2`.
///
/// Returns `EffectDrawPlan` FlatBuffer pointer + length when visible. Buffer
/// is valid only until the next call on the same slot.
#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_step(slot: *mut c_void, ev: ss_effect_event_info, parent_frame: f32, delta_seconds: f32, fps: f32, parent_looped: bool, y_flip: bool, vert_stride: u8) -> ss_effect_step_result {
    if slot.is_null() {
        return ss_effect_step_result::not_visible();
    }
    let slot = unsafe { &mut *(slot as *mut crate::EffectSlot) };
    slot.step(ev, parent_frame, delta_seconds, fps, parent_looped, y_flip, vert_stride)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_get_seed(slot: *mut c_void) -> u32 {
    if slot.is_null() {
        return 0;
    }
    let slot = unsafe { &*(slot as *const crate::EffectSlot) };
    slot.get_seed()
}

/// Whether the slot is currently running independently of the parent's frame.
/// Reflects the most recently seen `EventEffect.independent`. Callers use
/// this to decide whether to schedule continuous tick updates regardless of
/// parent frame change.
#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_slot_is_independent(slot: *mut c_void) -> bool {
    if slot.is_null() {
        return false;
    }
    let slot = unsafe { &*(slot as *const crate::EffectSlot) };
    slot.is_independent()
}

/// Create a fresh InstanceSlot. Slots are stateless at construction (no
/// resource binding) — `step` operates against a child runtime context
/// supplied per call.
#[unsafe(no_mangle)]
pub extern "C" fn ss_instance_slot_create() -> *mut c_void {
    Box::into_raw(Box::new(crate::InstanceSlot::new())) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_instance_slot_destroy(slot: *mut c_void) {
    if slot.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(slot as *mut crate::InstanceSlot);
    }
}

/// Reset the slot's "last started key" memory to its just-created state without
/// destroying it. Call on an explicit head-out (`ss_runtime_play` / replay) so a
/// re-played animation re-fires the instance child from scratch instead of
/// debouncing the first key as the same one the slot last started.
#[unsafe(no_mangle)]
pub extern "C" fn ss_instance_slot_reset(slot: *mut c_void) {
    if slot.is_null() {
        return;
    }
    let slot = unsafe { &mut *(slot as *mut crate::InstanceSlot) };
    slot.reset();
}

/// Drive the instance slot for one tick. The caller resolves the active
/// EventInstance (via `ss_runtime_get_active_event_instance` against the
/// *parent* runtime context) and passes the *child* runtime context here so
/// that label resolution and frame stepping happen against the right Context.
/// Returns `transitioned == true` on EventInstance edges; the caller is
/// responsible for any side effects tied to `play()` (event-sink notifications)
/// since those are host-side concerns.
#[unsafe(no_mangle)]
pub extern "C" fn ss_instance_slot_step(slot: *mut c_void, ev: ss_event_instance_info, child_ctx: *mut c_void, parent_frame: f32, delta_seconds: f32, parent_looped: bool) -> ss_instance_step_result {
    if slot.is_null() {
        return ss_instance_step_result::not_visible();
    }
    let Some(child_ffi) = get_context(child_ctx) else {
        return ss_instance_step_result::not_visible();
    };
    let slot = unsafe { &mut *(slot as *mut crate::InstanceSlot) };
    slot.step(ev, &mut child_ffi.ctx, parent_frame, delta_seconds, parent_looped)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_instance_slot_is_independent(slot: *mut c_void) -> bool {
    if slot.is_null() {
        return false;
    }
    let slot = unsafe { &*(slot as *const crate::InstanceSlot) };
    slot.is_independent()
}

/// Set the render mode. `mode` is interpreted as:
///   0 = Direct       (pre-bake world matrices; default)
///   1 = SceneGraph   (skip world matrix pre-computation)
/// Unknown values fall back to Direct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_runtime_set_render_mode(context: *mut c_void, mode: u8) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_render_mode(crate::core::RenderMode::from_u8(mode));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_runtime_get_render_mode(context: *mut c_void) -> u8 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.render_mode() as u8;
    }
    crate::core::RenderMode::Direct as u8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_context_set_coordinate_system(context: *mut c_void, coordinate_system: i32) {
    if let Some(ffi_context) = get_context(context) {
        let cs = match coordinate_system {
            1 => crate::core::SsCoordinateSystem::YDown,
            _ => crate::core::SsCoordinateSystem::YUp,
        };
        ffi_context.ctx.set_coordinate_system(cs);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_context_get_coordinate_system(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        match ffi_context.ctx.get_coordinate_system() {
            crate::core::SsCoordinateSystem::YDown => 1,
            crate::core::SsCoordinateSystem::YUp => 0,
        }
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_context_set_origin(context: *mut c_void, origin: i32) {
    if let Some(ffi_context) = get_context(context) {
        let o = match origin {
            1 => crate::core::SsOrigin::TopLeft,
            _ => crate::core::SsOrigin::Center,
        };
        ffi_context.ctx.set_origin(o);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_context_get_origin(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        match ffi_context.ctx.get_origin() {
            crate::core::SsOrigin::TopLeft => 1,
            crate::core::SsOrigin::Center => 0,
        }
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_context_set_unit(context: *mut c_void, unit: i32) {
    if let Some(ffi_context) = get_context(context) {
        let u = match unit {
            1 => crate::core::SsUnit::Normalized,
            _ => crate::core::SsUnit::Pixel,
        };
        ffi_context.ctx.set_unit(u);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_context_get_unit(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        match ffi_context.ctx.get_unit() {
            crate::core::SsUnit::Normalized => 1,
            crate::core::SsUnit::Pixel => 0,
        }
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_passed_event_count(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_passed_event_indices().len() as i32;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_passed_event_index(context: *mut c_void, index: i32) -> i32 {
    if let Some(ffi_context) = get_context(context)
        && let Some(&event_idx) = ffi_context.ctx.get_passed_event_indices().get(index as usize)
    {
        return event_idx;
    }
    -1
}

/// Resolve the active `EventInstance` for `slot_index` on the parent's
/// currently selected animation, at the current frame. Returns SS6
/// default semantics (`is_synthetic_default = true`, `loop_num = 1`,
/// `speed = 1.0`) when no event is active, so callers don't need to
/// branch on the no-event case.
///
/// `start_label_hash` / `end_label_hash` are returned raw. They are
/// already resolved internally by `ss_instance_slot_step` for the
/// child runtime — most Players should consume the slot helper rather
/// than resolving labels themselves.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_active_event_instance(context: *mut c_void, slot_index: u32) -> ss_event_instance_info {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_active_event_instance(slot_index);
    }
    ss_event_instance_info::synthetic_default()
}

/// Resolve the active `EventEffect` for `slot_index` on the parent's
/// currently selected animation, at the current frame. When no real event
/// has fired yet, falls back to `InitialEvents.effect` for the slot (with
/// `is_synthetic_default = true`); if neither exists, returns full SS6
/// defaults (`start_time = 0`, `speed = 1.0`, `independent = false`).
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_active_effect_event(context: *mut c_void, slot_index: u32) -> ss_effect_event_info {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_active_effect_event(slot_index);
    }
    ss_effect_event_info::synthetic_default()
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid null-terminated string.
pub unsafe extern "C" fn ss_runtime_hash_string(s: *const c_char) -> u32 {
    if s.is_null() {
        return 0;
    }
    let c_str = unsafe { core::ffi::CStr::from_ptr(s) };
    if let Ok(rust_str) = c_str.to_str() { crate::fnv1a_hash_str(rust_str) } else { 0 }
}

// --- Override Layer API (Phase 2) ---

/// Resolve a part name to its `part_index` in the current primary animation.
/// Returns `-1` on null context / null or invalid name / not found.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ss_runtime_find_part_index(context: *mut c_void, part_name: *const c_char) -> i32 {
    let Some(ffi_context) = get_context(context) else {
        return -1;
    };
    if part_name.is_null() {
        return -1;
    }
    let c_str = unsafe { CStr::from_ptr(part_name) };
    let Ok(name) = c_str.to_str() else {
        return -1;
    };
    ffi_context.ctx.find_part_index(name)
}

/// Set or clear a per-part visibility override.
/// `force`: `0` = clear/revert, `1` = force-hide, `2` = force-visible (reserved).
/// `cascade != 0` propagates a force-hide to descendants.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_part_visibility_override(context: *mut c_void, part: u32, force: c_uchar, cascade: c_uchar) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    match ffi_context.ctx.set_part_visibility_override(part, force, cascade != 0) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

/// Clear a part's visibility override (revert to animation).
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_clear_part_visibility_override(context: *mut c_void, part: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    match ffi_context.ctx.clear_part_visibility_override(part) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

/// Set a per-part color override from a single packed RGBA (`0xRRGGBBAA`, fills
/// all four corners). `blend_op`: `0=Mix 1=Mul 2=Add 3=Sub`. `priority_mode`:
/// `0=OverwriteOnNextKeyframe 1=HoldUntilNextAnimation 2=Permanent`. Normal
/// parts only.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_part_color_override(context: *mut c_void, part: u32, blend_op: c_uchar, rgba: u32, priority_mode: c_uchar) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    match ffi_context.ctx.set_part_color_override(part, blend_op, rgba, priority_mode) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

/// Set a per-part color override from four packed RGBA corners in ssruntime
/// order `lt, rt, lb, rb` (`rgba_ltrtlbrb` points to 4 `uint32` = `0xRRGGBBAA`).
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ss_runtime_set_part_color_override_corners(context: *mut c_void, part: u32, blend_op: c_uchar, rgba_ltrtlbrb: *const u32, priority_mode: c_uchar) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    if rgba_ltrtlbrb.is_null() {
        ffi_context.last_error = crate::error::SSRuntimeError::InvalidParameter.as_code();
        return false;
    }
    let packed = unsafe { core::slice::from_raw_parts(rgba_ltrtlbrb, 4) };
    let corners = [packed[0].to_be_bytes(), packed[1].to_be_bytes(), packed[2].to_be_bytes(), packed[3].to_be_bytes()];
    match ffi_context.ctx.set_part_color_override_corners(part, blend_op, corners, priority_mode) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

/// Clear a part's color override.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_clear_part_color_override(context: *mut c_void, part: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    match ffi_context.ctx.clear_part_color_override(part) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

/// Set a per-part cell override, identified by FNV-1a hashes of the cellmap name
/// (without `.ssce`) and the cell name. `(0, 0)` reverts to the animation.
/// `priority_mode`: `0=OverwriteOnNextKeyframe 1=HoldUntilNextAnimation 2=
/// Permanent`. Normal + Mask parts only. Fails if the cell cannot be resolved.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_part_cell_override(context: *mut c_void, part: u32, cellmap_name_hash: u32, cell_name_hash: u32, priority_mode: c_uchar) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    match ffi_context.ctx.set_part_cell_override(part, cellmap_name_hash, cell_name_hash, priority_mode) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

/// Clear a part's cell override.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_clear_part_cell_override(context: *mut c_void, part: u32) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    match ffi_context.ctx.clear_part_cell_override(part) {
        Ok(_) => true,
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

/// Clear every per-part override on the context.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_clear_all_part_overrides(context: *mut c_void) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    ffi_context.last_error = 0;
    ffi_context.ctx.clear_all_part_overrides();
    true
}
