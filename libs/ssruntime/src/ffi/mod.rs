use std::ffi::{c_char, c_float, c_int, c_uchar, CStr};

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_create(ptr: *mut c_uchar, len: usize) -> *mut crate::Context {
    let buf = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    let Ok(c) = crate::Context::new(buf) else {
        return std::ptr::null_mut();
    };
    Box::into_raw(Box::new(c))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_destroy(context: *mut crate::Context) {
    unsafe {
        let _ = Box::from_raw(context);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_ssab_buf(context: *mut crate::Context) -> *mut c_uchar {
    let context = unsafe { &mut *context };
    context.get_ssab_buf_ptr() as *mut c_uchar
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_ssab_buf_len(context: *mut crate::Context) -> usize {
    let context = unsafe { &mut *context };
    context.get_ssab_buf_len()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_setup_animation(context: *mut crate::Context, name: *mut c_char) -> bool {
    let context = unsafe { &mut *context };

    let c_buf = unsafe { CStr::from_ptr(name) };
    let a = c_buf.to_string_lossy().into_owned();
    context.setup_animation(&a)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_data(context: *mut crate::Context, frame_no: c_int, out_data: *mut *mut c_uchar, out_len: *mut usize) {
    let context = unsafe { &mut *context };
    let Some(ret) = context.get_frame_data(frame_no) else {
        return;
    };
    unsafe {
        *out_data = ret.as_ptr() as *mut c_uchar;
        *out_len = ret.len();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_no(context: *mut crate::Context) -> c_int {
    let context = unsafe { &mut *context };
    context.get_frame_no()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_no_decimal(context: *mut crate::Context) -> c_float {
    let context = unsafe { &mut *context };
    context.get_frame_no_decimal()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_end_frame_reached(context: *mut crate::Context) -> bool {
    let context = unsafe { &mut *context };
    context.is_end_frame_reached()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_is_playing(context: *mut crate::Context) -> bool {
    let context = unsafe { &mut *context };
    context.is_playing()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_skip_frames(context: *mut crate::Context) -> bool {
    let context = unsafe { &mut *context };
    context.get_skip_frames()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_playback_direction(context: *mut crate::Context) -> i32 {
    let context = unsafe { &mut *context };
    context.get_playback_direction() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_playback_style(context: *mut crate::Context) -> i32 {
    let context = unsafe { &mut *context };
    context.get_playback_style() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_loops(context: *mut crate::Context) -> i32 {
    let context = unsafe { &mut *context };
    context.get_loops()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_remain_loops(context: *mut crate::Context) -> i32 {
    let context = unsafe { &mut *context };
    context.get_remain_loops()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_start_frame(context: *mut crate::Context) -> i32 {
    let context = unsafe { &mut *context };
    context.controller.get_start_frame()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_end_frame(context: *mut crate::Context) -> i32 {
    let context = unsafe { &mut *context };
    context.get_end_frame()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_fps(context: *mut crate::Context) -> i32 {
    let context = unsafe { &mut *context };
    context.get_fps()
}


// Playback setup
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_skip_frames(context: *mut crate::Context, skip: bool) {
    let context = unsafe { &mut *context };
    context.set_skip_frames(skip);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_loop(context: *mut crate::Context, loops: c_int) {
    let context = unsafe { &mut *context };
    context.set_loop(loops);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_rate(context: *mut crate::Context, fps: c_int) {
    let context = unsafe { &mut *context };
    context.set_frame_rate(fps);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_playback_direction(context: *mut crate::Context, direction: c_int, style: c_int) {
    let context = unsafe { &mut *context };
    let d = if direction == 0 { crate::core::playback_controller::PlaybackDirection::Forward } else { crate::core::playback_controller::PlaybackDirection::Backward };
    let s = if style == 0 { crate::core::playback_controller::PlaybackStyle::Normal } else { crate::core::playback_controller::PlaybackStyle::PingPong };
    context.set_playback_direction(d, s);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_animation_speed(context: *mut crate::Context, speed_rate: c_float) {
    let context = unsafe { &mut *context };
    context.set_animation_speed(speed_rate);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_animation_section(context: *mut crate::Context, start_frame: c_int, end_frame: c_int) {
    let context = unsafe { &mut *context };
    context.set_animation_section(start_frame, end_frame);
}

// Playback control

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_update(context: *mut crate::Context, delta: f32) -> i32 {
    let context = unsafe { &mut *context };
    let (frame_no, _) = context.update(delta);

    frame_no
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play(context: *mut crate::Context) {
    let context = unsafe { &mut *context };
    context.play();
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play_with_start_frame(context: *mut crate::Context, frame_no: i32) {
    let context = unsafe { &mut *context };
    context.play_with_start_frame(frame_no);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_pause(context: *mut crate::Context) {
    let context = unsafe { &mut *context };
    context.pause();
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_stop(context: *mut crate::Context) {
    let context = unsafe { &mut *context };
    context.stop();
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_no(context: *mut crate::Context, frame: c_int) {
    let context = unsafe { &mut *context };
    context.set_frame_no(frame);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_next_frame(context: *mut crate::Context) {
    let context = unsafe { &mut *context };
    context.next_frame();
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_prev_frame(context: *mut crate::Context) {
    let context = unsafe { &mut *context };
    context.prev_frame();
}
