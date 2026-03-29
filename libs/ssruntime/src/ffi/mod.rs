use std::ffi::{c_char, c_float, c_int, c_uchar, c_void, CStr};
use std::panic::catch_unwind;

pub struct FfiContext {
    pub ctx: crate::Context<'static>,
}
impl<'a> FfiContext {
    pub fn new() -> Self {
        Self { ctx: crate::Context::new() }
    }
}

#[inline(always)]
fn get_context<'a>(ptr: *mut c_void) -> &'a mut FfiContext {
    if ptr.is_null() {
        eprintln!("SSRuntime Error: Context pointer is null.");
        panic!("Context pointer is null.");
    }
    unsafe { &mut *(ptr as *mut FfiContext) }
}

fn check_ptr<T>(ptr: *const T, name: &str) {
    if ptr.is_null() {
        eprintln!("SSRuntime Error: {} pointer is null.", name);
        panic!("{} pointer is null.", name);
    }
}



#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_create() -> *mut c_void {
    let c = FfiContext::new();
    Box::into_raw(Box::new(c)) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_destroy(context: *mut c_void) {
    unsafe {
        let context = context as *mut FfiContext;
        let _ = Box::from_raw(context);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_load_ssab_copy(context: *mut c_void, ptr: *const c_uchar, len: usize) -> bool {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        check_ptr(ptr, "ssab_data");
        let r = ffi_context.ctx.load_copy( unsafe { std::slice::from_raw_parts(ptr as *const u8, len) } );
        r.unwrap_or(false)
    });
    result.unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_load_ssab_borrow(context: *mut c_void, ptr: *const c_uchar, len: usize) -> bool {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        check_ptr(ptr, "ssab_data");
        let r = unsafe { ffi_context.ctx.load_borrow(ptr as *const u8, len) };
        r.unwrap_or(false)
    });
    result.unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_ssab(context: *mut c_void, out_data: *mut *mut c_uchar, out_len: *mut usize) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        let Some(ssab_buf) = ffi_context.ctx.get_ssab_data() else {
            return;
        };
        unsafe {
            *out_data = ssab_buf.as_ptr() as *mut c_uchar;
            *out_len = ssab_buf.len();
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_ssab_buf(context: *mut c_void) -> *const c_uchar {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        let Some(ssab_buf) = ffi_context.ctx.get_ssab_data() else {
            return std::ptr::null();
        };
        ssab_buf.as_ptr() as *const c_uchar
    });
    result.unwrap_or(std::ptr::null())
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_ssab_len(context: *mut c_void) -> usize {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        let Some(ssab_buf) = ffi_context.ctx.get_ssab_data() else {
            return 0;
        };
        ssab_buf.len()
    });
    result.unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_setup_animation(context: *mut c_void, name: *const c_char) -> bool {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        check_ptr(name, "animation_name");
        let c_buf = unsafe { CStr::from_ptr(name) };
        let a = c_buf.to_string_lossy().into_owned();
        ffi_context.ctx.setup_animation(&a)
    });
    result.unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_data(context: *mut c_void, frame_no: c_int, out_data: *mut *mut c_uchar, out_len: *mut usize) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        let Some(ret) = ffi_context.ctx.get_frame_data(frame_no) else {
            return;
        };
        unsafe {
            *out_data = ret.as_ptr() as *mut c_uchar;
            *out_len = ret.len();
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_no(context: *mut c_void) -> c_int {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.get_frame_no()
    });
    result.unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_no_decimal(context: *mut c_void) -> c_float {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.get_frame_no_decimal()
    });
    result.unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_end_frame_reached(context: *mut c_void) -> bool {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.is_end_frame_reached()
    });
    result.unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_playing(context: *mut c_void) -> bool {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.is_playing()
    });
    result.unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_pausing(context: *mut c_void) -> bool {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.is_pausing()
    });
    result.unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_skip_frames(context: *mut c_void) -> bool {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.get_skip_frames()
    });
    result.unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_playback_direction(context: *mut c_void) -> i32 {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.get_playback_direction() as i32
    });
    result.unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_playback_style(context: *mut c_void) -> i32 {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.get_playback_style() as i32
    });
    result.unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_loops(context: *mut c_void) -> i32 {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.get_loops()
    });
    result.unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_remain_loops(context: *mut c_void) -> i32 {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.get_remain_loops()
    });
    result.unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_start_frame(context: *mut c_void) -> i32 {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.controller.get_start_frame()
    });
    result.unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_end_frame(context: *mut c_void) -> i32 {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.get_end_frame()
    });
    result.unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_fps(context: *mut c_void) -> i32 {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.get_fps()
    });
    result.unwrap_or(0)
}


// Playback setup
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_skip_frames(context: *mut c_void, skip: bool) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.set_skip_frames(skip);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_loop(context: *mut c_void, loops: c_int) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.set_loop(loops);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_rate(context: *mut c_void, fps: c_int) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.set_frame_rate(fps);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_playback_direction(context: *mut c_void, direction: c_int, style: c_int) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        let d = if direction == 0 { crate::core::playback_controller::PlaybackDirection::Forward } else { crate::core::playback_controller::PlaybackDirection::Backward };
        let s = if style == 0 { crate::core::playback_controller::PlaybackStyle::Normal } else { crate::core::playback_controller::PlaybackStyle::PingPong };
        ffi_context.ctx.set_playback_direction(d, s);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_animation_speed(context: *mut c_void, speed_rate: c_float) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.set_animation_speed(speed_rate);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_animation_section(context: *mut c_void, start_frame: c_int, end_frame: c_int) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.set_animation_section(start_frame, end_frame);
    });
}

// Playback control

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_update(context: *mut c_void, delta: f32) -> i32 {
    let result = catch_unwind(|| {
        let ffi_context = get_context(context);
        let (frame_no, _) = ffi_context.ctx.update(delta);
        frame_no
    });
    result.unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play(context: *mut c_void) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.play();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play_with_start_frame(context: *mut c_void, frame_no: i32) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.play_with_start_frame(frame_no);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_pause(context: *mut c_void) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.pause();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_stop(context: *mut c_void) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.stop();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_no(context: *mut c_void, frame: c_int) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.set_frame_no(frame);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_next_frame(context: *mut c_void) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.next_frame();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_prev_frame(context: *mut c_void) {
    let _ = catch_unwind(|| {
        let ffi_context = get_context(context);
        ffi_context.ctx.prev_frame();
    });
}
