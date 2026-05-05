#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_lazy_continuation)]

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
            let cb_ptr = $crate::ffi::LOG_CALLBACK.load(core::sync::atomic::Ordering::Relaxed);
            if !cb_ptr.is_null() {
                let cb: $crate::ffi::SsLogCallback = core::mem::transmute(cb_ptr);
                if let Ok(s) = alloc::ffi::CString::new(alloc::format!($($arg)*)) {
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
pub unsafe extern "C" fn ss_runtime_get_ssab(context: *mut c_void, out_data: *mut *mut c_uchar, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        if let Some(ssab_buf) = ffi_context.ctx.get_ssab_data() {
            unsafe {
                if !out_data.is_null() {
                    *out_data = ssab_buf.as_ptr() as *mut c_uchar;
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
    let a = alloc::string::String::from_utf8_lossy(c_buf.to_bytes()).into_owned();
    match ffi_context.ctx.setup_animation(&a) {
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
/// Per-part FNV-1a hash of the cellmap name owning the current cell, written
/// by `ss_runtime_get_frame_data`. Indexed by `part_index`. Matches
/// `cellmaps[map_id].name_hash` in the SSAB binary, so a Player can resolve
/// its texture table directly without walking `FrameData::cells` and
/// `SsAnimeBinary::cellmaps`. Parts without a current cell carry zero.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_cell_texture_hashes(context: *mut c_void, out_data: *mut *const u32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_cell_texture_hashes_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_cell_texture_hashes_len();
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
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_z_order(context: *mut c_void, out_data: *mut *const i32, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        unsafe {
            if !out_data.is_null() {
                *out_data = ffi_context.ctx.get_z_order_ptr();
            }
            if !out_len.is_null() {
                *out_len = ffi_context.ctx.get_z_order_len();
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

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_end_frame_reached(context: *mut c_void) -> bool {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.is_end_frame_reached();
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
        return ffi_context.ctx.controller.get_start_frame();
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
pub extern "C" fn ss_runtime_update_animation(context: *mut c_void, delta: f32) -> c_float {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.update_animation(delta);
        return ffi_context.ctx.get_frame_no();
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

/// Sets the playback position based on `diff` (frames elapsed since playback
/// started, with per-instance speed already applied), applying the configured
/// `set_animation_section`, `set_loops`, and `set_playback_direction` values.
///
/// This is the entry point a parent Player uses to drive a child Instance or
/// Effect Player deterministically: it works correctly under reverse, seek,
/// and frame-skip because the same `diff` always produces the same frame.
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_relative(context: *mut c_void, diff: c_float) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.set_frame_relative(diff);
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

#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_create(resource: *const c_void, effect_name_hash: u32, seed: u32) -> *mut c_void {
    let res = unsafe { &*(resource as *const crate::SsabResource) };
    if let Some(r) = res.resource()
        && let Ok(effect_ctx) = crate::EffectContext::new(r, effect_name_hash, seed)
    {
        return Box::into_raw(Box::new(effect_ctx)) as *mut c_void;
    }
    core::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_update(effect_ctx: *mut c_void, frame: f32) {
    if effect_ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *(effect_ctx as *mut crate::EffectContext) };
    ctx.update(frame);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_reset(effect_ctx: *mut c_void) {
    if effect_ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *(effect_ctx as *mut crate::EffectContext) };
    ctx.reset();
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_get_lifetime(effect_ctx: *mut c_void) -> i32 {
    if effect_ctx.is_null() {
        return 0;
    }
    let ctx = unsafe { &*(effect_ctx as *const crate::EffectContext) };
    ctx.get_lifetime()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_is_infinite(effect_ctx: *mut c_void) -> bool {
    if effect_ctx.is_null() {
        return false;
    }
    let ctx = unsafe { &*(effect_ctx as *const crate::EffectContext) };
    ctx.is_infinite()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_effect_get_state(effect_ctx: *mut c_void, out_data: *mut *mut c_uchar, out_len: *mut usize) {
    if effect_ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *(effect_ctx as *mut crate::EffectContext) };
    if let Ok(ret) = ctx.get_state() {
        unsafe {
            if !out_data.is_null() {
                *out_data = ret.as_ptr() as *mut c_uchar;
            }
            if !out_len.is_null() {
                *out_len = ret.len();
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_effect_destroy(effect_ctx: *mut c_void) {
    if effect_ctx.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(effect_ctx as *mut crate::EffectContext);
    }
}

// --- Utilities ---

#[unsafe(no_mangle)]
/// Calculate the target frame for a child instance animation.
pub extern "C" fn ss_util_calculate_instance_frame(parent_frame: f32, key_frame: i32, start_offset: i32, speed: f32, independent: bool, accumulated_time: f32) -> f32 {
    crate::core::util::calculate_instance_frame(parent_frame, key_frame, start_offset, speed, independent, accumulated_time)
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
/// Transform multiple particle positions by a matrix.
/// matrix_ptr: pointer to float array of size 16.
/// particles_ptr: pointer to ParticleState array from ss_effect_get_state.
/// count: number of particles.
/// out_pos_ptr: pointer to float array (x, y, x, y, ...) to store transformed 2D coordinates.
pub unsafe extern "C" fn ss_effect_transform_particles(matrix_ptr: *const f32, particles_ptr: *const c_void, count: i32, out_pos_ptr: *mut f32) {
    if matrix_ptr.is_null() || particles_ptr.is_null() || out_pos_ptr.is_null() {
        return;
    }

    let mut m_arr = [0.0f32; 16];
    unsafe {
        core::ptr::copy_nonoverlapping(matrix_ptr, m_arr.as_mut_ptr(), 16);
    }
    let matrix = crate::core::SsMatrix4x4::from_array(m_arr);

    // particles_ptr is a pointer to [ParticleState; count]
    // Layout from effect_state.fbs: stride = 28
    let len = (count as usize) * 28;
    let particles_raw = unsafe { core::slice::from_raw_parts(particles_ptr as *const u8, len) };

    let transformed = crate::core::util::transform_particles(&matrix, particles_raw);

    for (i, (tx, ty)) in transformed.into_iter().enumerate() {
        unsafe {
            *(out_pos_ptr.add(i * 2)) = tx;
            *(out_pos_ptr.add(i * 2 + 1)) = ty;
        }
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

/// Computes the local vertex coordinates for a part.
///
/// # Safety
/// - `input_data`: Must point to an array of at least 6 floats if `has_deform` (flag 4) is false,
///   or 14 floats if `has_deform` is true.
///   Layout: [size_x, size_y, pivot_x, pivot_y, pivot_offset_x, pivot_offset_y, (optional) dx0..3, dy0..3]
/// - `out_x`, `out_y`: Must point to buffers with at least 5 floats.
#[unsafe(no_mangle)]
/// Computes the local vertices for a part using primitive parameters.
/// deform_x_ptr, deform_y_ptr: optional pointers to float arrays of size 4.
/// out_x, out_y: pointers to float arrays of size 5 to store results.
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
pub unsafe extern "C" fn ss_vertex_compute_local(
    size_w: f32,
    size_h: f32,
    pivot_x: f32,
    pivot_y: f32,
    pivot_offset_x: f32,
    pivot_offset_y: f32,
    h_flip: bool,
    v_flip: bool,
    deform_x_ptr: *const f32,
    deform_y_ptr: *const f32,
    out_x: *mut f32,
    out_y: *mut f32,
) -> bool {
    if out_x.is_null() || out_y.is_null() {
        return false;
    }

    let deform = if !deform_x_ptr.is_null() && !deform_y_ptr.is_null() {
        let dx = unsafe { &*deform_x_ptr.cast::<[f32; 4]>() };
        let dy = unsafe { &*deform_y_ptr.cast::<[f32; 4]>() };
        Some((dx, dy))
    } else {
        None
    };

    let (vx, vy) = crate::core::geometry::compute_local_vertices((size_w, size_h), (pivot_x, pivot_y), (pivot_offset_x, pivot_offset_y), (h_flip, v_flip), deform);

    unsafe {
        *out_x.cast::<[f32; 5]>() = vx;
        *out_y.cast::<[f32; 5]>() = vy;
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_geometry_is_point_in_rect(px: f32, py: f32, size_w: f32, size_h: f32) -> bool {
    crate::core::geometry::is_point_in_rect((px, py), (size_w, size_h))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_geometry_is_point_in_quad(
    tx: f32,
    ty: f32,
    vx: *const f32, // Array of 4 floats
    vy: *const f32, // Array of 4 floats
) -> bool {
    if vx.is_null() || vy.is_null() {
        return false;
    }
    let vxa = unsafe { &*(vx as *const [f32; 4]) };
    let vya = unsafe { &*(vy as *const [f32; 4]) };
    crate::core::geometry::is_point_in_quad((tx, ty), vxa, vya)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_nineslice_compute_local(
    size_w: f32,
    size_h: f32,
    u_left: f32,
    v_top: f32,
    u_right: f32,
    v_bottom: f32,
    margin_l: f32,
    margin_t: f32,
    margin_r: f32,
    margin_b: f32,
    source_w: f32,
    source_h: f32,
    out_vx: *mut f32, // Array of 16 floats
    out_vy: *mut f32, // Array of 16 floats
    out_uv: *mut f32, // Array of 16 floats
    out_vv: *mut f32, // Array of 16 floats
) -> bool {
    if out_vx.is_null() || out_vy.is_null() || out_uv.is_null() || out_vv.is_null() {
        return false;
    }

    let grid = crate::core::geometry::compute_nineslice_geometry((size_w, size_h), (u_left, v_top, u_right, v_bottom), (margin_l, margin_t, margin_r, margin_b), (source_w, source_h));

    unsafe {
        *out_vx.cast::<[f32; 16]>() = grid.vertices_x;
        *out_vy.cast::<[f32; 16]>() = grid.vertices_y;
        *out_uv.cast::<[f32; 16]>() = grid.uvs_u;
        *out_vv.cast::<[f32; 16]>() = grid.uvs_v;
    }
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ss_shape_compute_local(
    shape_type: i32,
    size_w: f32,
    size_h: f32,
    out_vx: *mut f32, // Array of 16 floats
    out_vy: *mut f32, // Array of 16 floats
    out_count: *mut i32,
) -> bool {
    if out_vx.is_null() || out_vy.is_null() || out_count.is_null() {
        return false;
    }

    let shape = match shape_type {
        0 => crate::core::geometry::ShapeType::Rectangle,
        1 => crate::core::geometry::ShapeType::Triangle,
        2 => crate::core::geometry::ShapeType::Star,
        3 => crate::core::geometry::ShapeType::Arrow,
        _ => return false,
    };

    let (count, vx, vy) = crate::core::geometry::compute_shape_vertices(shape, (size_w, size_h));

    unsafe {
        *out_vx.cast::<[f32; 16]>() = vx;
        *out_vy.cast::<[f32; 16]>() = vy;
        *out_count = count as i32;
    }
    true
}
