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
    if arc.resource.is_none() {
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

    if let Some(res) = &arc_clone.resource {
        // Safe because arc_clone will be owned by ffi_context.bound_resource
        ffi_context.ctx.bind_resource(unsafe { &*(res as *const _) });
        ffi_context.bound_resource = Some(arc_clone);
        true
    } else {
        false
    }
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
pub unsafe extern "C" fn ss_runtime_get_frame_data(context: *mut c_void, frame_no: c_int, out_data: *mut *mut c_uchar, out_len: *mut usize) {
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
pub extern "C" fn ss_runtime_get_frame_no(context: *mut c_void) -> c_int {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_frame_no();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_no_decimal(context: *mut c_void) -> c_float {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_frame_no_decimal();
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
pub extern "C" fn ss_runtime_update(context: *mut c_void, delta: f32) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        let (frame_no, _) = ffi_context.ctx.update(delta);
        return frame_no;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_update_animation(context: *mut c_void, delta: f32) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.update_animation(delta);
        return ffi_context.ctx.get_frame_no();
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play(context: *mut c_void) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.ctx.play();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play_with_start_frame(context: *mut c_void, frame_no: i32) {
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
pub extern "C" fn ss_runtime_set_frame_no(context: *mut c_void, frame: c_int) {
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

#[unsafe(no_mangle)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for output.
/// The caller must ensure `out_data` and `out_len` are valid if they are not null.
pub unsafe extern "C" fn ss_runtime_get_effect_state(context: *mut c_void, part_index: u32, out_data: *mut *mut c_uchar, out_len: *mut usize) {
    if let Some(ffi_context) = get_context(context) {
        ffi_context.last_error = 0;
        match ffi_context.ctx.get_effect_state(part_index) {
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

// --- Matrix V2 FFI (Primitive-based) ---

#[unsafe(no_mangle)]
/// out_m must be a pointer to a float array of size 16.
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe extern "C" fn ss_matrix_identity(out_m: *mut f32) {
    if !out_m.is_null() {
        let identity = crate::core::SsMatrix4x4::identity();
        unsafe {
            core::ptr::copy_nonoverlapping(identity.m.as_ptr(), out_m, 16);
        }
    }
}

#[unsafe(no_mangle)]
/// out_m, lhs, rhs must be pointers to float arrays of size 16.
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
pub unsafe extern "C" fn ss_matrix_multiply(out_m: *mut f32, lhs: *const f32, rhs: *const f32) {
    if !out_m.is_null() && !lhs.is_null() && !rhs.is_null() {
        let mut lhs_arr = [0.0f32; 16];
        let mut rhs_arr = [0.0f32; 16];
        unsafe {
            core::ptr::copy_nonoverlapping(lhs, lhs_arr.as_mut_ptr(), 16);
            core::ptr::copy_nonoverlapping(rhs, rhs_arr.as_mut_ptr(), 16);
        }
        let lhs_m = crate::core::SsMatrix4x4::from_array(lhs_arr);
        let rhs_m = crate::core::SsMatrix4x4::from_array(rhs_arr);
        let res = lhs_m.multiply(&rhs_m);
        unsafe {
            core::ptr::copy_nonoverlapping(res.m.as_ptr(), out_m, 16);
        }
    }
}

#[unsafe(no_mangle)]
/// out_m, in_m must be pointers to float arrays of size 16.
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
pub unsafe extern "C" fn ss_matrix_inverse(out_m: *mut f32, in_m: *const f32) {
    if !out_m.is_null() && !in_m.is_null() {
        let mut in_arr = [0.0f32; 16];
        unsafe {
            core::ptr::copy_nonoverlapping(in_m, in_arr.as_mut_ptr(), 16);
        }
        let mat = crate::core::SsMatrix4x4::from_array(in_arr);
        let res = mat.inverse();
        unsafe {
            core::ptr::copy_nonoverlapping(res.m.as_ptr(), out_m, 16);
        }
    }
}

#[unsafe(no_mangle)]
/// out_x, out_y, out_z: output pointers for transformed vector.
/// mat: pointer to a float array of size 16.
/// x, y, z: input vector components.
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
pub unsafe extern "C" fn ss_matrix_transform_vector3(out_x: *mut f32, out_y: *mut f32, out_z: *mut f32, mat: *const f32, x: f32, y: f32, z: f32) {
    if !out_x.is_null() && !out_y.is_null() && !out_z.is_null() && !mat.is_null() {
        let mut mat_arr = [0.0f32; 16];
        unsafe {
            core::ptr::copy_nonoverlapping(mat, mat_arr.as_mut_ptr(), 16);
        }
        let mat_m = crate::core::SsMatrix4x4::from_array(mat_arr);
        let v = crate::core::SsVector3::new(x, y, z);
        let res = mat_m.transform(&v);
        unsafe {
            *out_x = res.x;
            *out_y = res.y;
            *out_z = res.z;
        }
    }
}

#[unsafe(no_mangle)]
/// out_m: pointer to a float array of size 16.
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe extern "C" fn ss_matrix_create_scale(out_m: *mut f32, x: f32, y: f32, z: f32) {
    if !out_m.is_null() {
        let res = crate::core::SsMatrix4x4::create_scale(x, y, z);
        unsafe {
            core::ptr::copy_nonoverlapping(res.m.as_ptr(), out_m, 16);
        }
    }
}

#[unsafe(no_mangle)]
/// out_m: pointer to a float array of size 16.
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe extern "C" fn ss_matrix_create_translation(out_m: *mut f32, x: f32, y: f32, z: f32) {
    if !out_m.is_null() {
        let res = crate::core::SsMatrix4x4::create_translation(x, y, z);
        unsafe {
            core::ptr::copy_nonoverlapping(res.m.as_ptr(), out_m, 16);
        }
    }
}

#[unsafe(no_mangle)]
/// out_m: pointer to a float array of size 16.
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe extern "C" fn ss_matrix_create_rotation_x(out_m: *mut f32, radians: f32) {
    if !out_m.is_null() {
        let res = crate::core::SsMatrix4x4::create_rotation_x(radians);
        unsafe {
            core::ptr::copy_nonoverlapping(res.m.as_ptr(), out_m, 16);
        }
    }
}

#[unsafe(no_mangle)]
/// out_m: pointer to a float array of size 16.
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe extern "C" fn ss_matrix_create_rotation_y(out_m: *mut f32, radians: f32) {
    if !out_m.is_null() {
        let res = crate::core::SsMatrix4x4::create_rotation_y(radians);
        unsafe {
            core::ptr::copy_nonoverlapping(res.m.as_ptr(), out_m, 16);
        }
    }
}

#[unsafe(no_mangle)]
/// out_m: pointer to a float array of size 16.
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe extern "C" fn ss_matrix_create_rotation_z(out_m: *mut f32, radians: f32) {
    if !out_m.is_null() {
        let res = crate::core::SsMatrix4x4::create_rotation_z(radians);
        unsafe {
            core::ptr::copy_nonoverlapping(res.m.as_ptr(), out_m, 16);
        }
    }
}

#[unsafe(no_mangle)]
/// out_inheritance_m, out_drawing_m: pointers to float arrays of size 16.
/// state: pointer to PartState
/// parent_matrix: pointer to a float array of size 16, or null.
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
pub unsafe extern "C" fn ss_matrix_compute_world(
    out_inheritance_m: *mut f32,
    out_drawing_m: *mut f32,
    state: *const crate::runtime::framedata::ss::runtime::PartState,
    parent_matrix: *const f32,
    has_parent_size: bool,
    parent_size_x: f32,
    parent_size_y: f32,
) {
    if !out_inheritance_m.is_null() && !out_drawing_m.is_null() && !state.is_null() {
        let p_state = unsafe { &*state };

        let p_mat_opt = if parent_matrix.is_null() {
            None
        } else {
            let mut arr = [0.0f32; 16];
            unsafe {
                core::ptr::copy_nonoverlapping(parent_matrix, arr.as_mut_ptr(), 16);
            }
            Some(crate::core::SsMatrix4x4::from_array(arr))
        };

        let p_size_opt = if has_parent_size { Some((parent_size_x, parent_size_y)) } else { None };

        let (inh, draw) = crate::core::SsMatrix4x4::compute_world_matrix(p_state, p_mat_opt.as_ref(), p_size_opt);

        unsafe {
            core::ptr::copy_nonoverlapping(inh.m.as_ptr(), out_inheritance_m, 16);
            core::ptr::copy_nonoverlapping(draw.m.as_ptr(), out_drawing_m, 16);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_passed_event_count(context: *mut c_void) -> i32 {
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_passed_events().len() as i32;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_passed_event_type(context: *mut c_void, index: i32) -> i32 {
    if let Some(ffi_context) = get_context(context)
        && let Some(event) = ffi_context.ctx.get_passed_events().get(index as usize)
    {
        return match event.data {
            crate::PassedEventData::UserData(_) => 0,
            crate::PassedEventData::Signal(_) => 1,
            crate::PassedEventData::Audio(_) => 2,
            crate::PassedEventData::Instance(_) => 3,
            crate::PassedEventData::Effect(_) => 4,
        };
    }
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_passed_event_frame_no(context: *mut c_void, index: i32) -> i32 {
    if let Some(ffi_context) = get_context(context)
        && let Some(event) = ffi_context.ctx.get_passed_events().get(index as usize)
    {
        return event.frame_no;
    }
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_passed_event_part_index(context: *mut c_void, index: i32) -> i32 {
    if let Some(ffi_context) = get_context(context)
        && let Some(event) = ffi_context.ctx.get_passed_events().get(index as usize)
    {
        return event.part_index as i32;
    }
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_passed_event_index(context: *mut c_void, index: i32) -> i32 {
    if let Some(ffi_context) = get_context(context)
        && let Some(event) = ffi_context.ctx.get_passed_events().get(index as usize)
    {
        return event.event_index;
    }
    -1
}
