use alloc::boxed::Box;
use core::ffi::{CStr, c_char, c_float, c_int, c_uchar, c_void};

// Placeholder for eprintln! in no_std
#[allow(unused_macros)]
macro_rules! eprintln {
    ($($arg:tt)*) => {};
}

pub struct FfiResource {
    pub data: crate::SsabData,
    pub resource: Option<crate::AnimationResource<'static>>,
}

pub struct FfiContext {
    pub ctx: crate::Context<'static>,
    // We hold the resource here for backward compatibility with ss_runtime_load_ssab_*
    pub bound_resource: Option<Box<FfiResource>>,
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

#[inline(always)]
fn get_resource<'a>(ptr: *mut c_void) -> Option<&'a mut FfiResource> {
    if ptr.is_null() {
        return None;
    }
    unsafe { Some(&mut *(ptr as *mut FfiResource)) }
}

fn create_ffi_resource(data: crate::SsabData) -> *mut c_void {
    let mut r = Box::new(FfiResource { data, resource: None });
    let data_ptr = r.data.as_slice() as *const [u8];
    match crate::AnimationResource::new(unsafe { &*data_ptr }) {
        Ok(resource) => {
            r.resource = Some(resource);
            Box::into_raw(r) as *mut c_void
        }
        Err(_) => core::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
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
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure the pointer is valid and points to a valid memory block.
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
    if let Some(r) = get_resource(resource) {
        unsafe {
            let _ = Box::from_raw(r as *mut FfiResource);
        }
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

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_bind_resource(context: *mut c_void, resource: *mut c_void) -> bool {
    let Some(ffi_context) = get_context(context) else {
        return false;
    };
    let Some(ffi_resource) = get_resource(resource) else {
        return false;
    };

    // Safety: In FFI, we use 'static lifetime for the reference, which means the caller
    // MUST ensure the resource outlives the context if they manage it externally.
    // However, if we want to be safe, we'd need Arc.
    // Since we are in no_std/alloc, we can't easily use Arc without more setup.
    // For now, we assume the user manages the lifetime.
    if let Some(res) = &ffi_resource.resource {
        ffi_context.ctx.bind_resource(unsafe { &*(res as *const _) });
        true
    } else {
        false
    }
}

#[unsafe(no_mangle)]
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
            let mut ffi_res = Box::new(FfiResource { data: ssab_data, resource: None });
            let data_ptr = ffi_res.data.as_slice() as *const [u8];
            match crate::AnimationResource::new(unsafe { &*data_ptr }) {
                Ok(resource) => {
                    ffi_res.resource = Some(resource);
                    // Use a raw pointer to bind to bypass borrow checker
                    let res_ptr = ffi_res.resource.as_ref().unwrap() as *const _;
                    ffi_context.ctx.bind_resource(unsafe { &*res_ptr });
                    ffi_context.bound_resource = Some(ffi_res);
                    true
                }
                Err(e) => {
                    ffi_context.last_error = e.as_code();
                    false
                }
            }
        }
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
            let mut ffi_res = Box::new(FfiResource { data: ssab_data, resource: None });
            let data_ptr = ffi_res.data.as_slice() as *const [u8];
            match crate::AnimationResource::new(unsafe { &*data_ptr }) {
                Ok(resource) => {
                    ffi_res.resource = Some(resource);
                    let res_ptr = ffi_res.resource.as_ref().unwrap() as *const _;
                    ffi_context.ctx.bind_resource(unsafe { &*res_ptr });
                    ffi_context.bound_resource = Some(ffi_res);
                    true
                }
                Err(e) => {
                    ffi_context.last_error = e.as_code();
                    false
                }
            }
        }
        Err(e) => {
            ffi_context.last_error = e.as_code();
            false
        }
    }
}

#[unsafe(no_mangle)]
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
/// # Safety
///
/// This function is unsafe because it dereferences the `name` raw pointer.
/// The caller must ensure that `name` points to a valid, null-terminated C string.
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
    if let Some(ffi_context) = get_context(context) {
        return ffi_context.ctx.get_playback_direction() as i32;
    }
    0
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
