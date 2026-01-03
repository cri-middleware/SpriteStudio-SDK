use std::ffi::{c_char, c_float, c_int, c_uchar, c_void, CStr};

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_create(ptr: *mut c_uchar, len: usize) -> *mut c_void {
    let buf = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    let Ok(c) = crate::Context::new(buf) else {
        return std::ptr::null_mut();
    };
    Box::into_raw(Box::new(c)) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_destroy(ptr: *mut c_void) {
    unsafe {
        let _ = Box::from_raw(ptr as *mut crate::Context<'_>);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_initialize_data(ptr: *mut c_void) -> *mut c_uchar {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    let Some(ret) = context.get_initialize_data() else {
        return std::ptr::null_mut()
    };
    Box::into_raw(Box::new(ret)) as *mut c_uchar
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_setup_animation(ptr: *mut c_void, name: *mut c_char) -> *mut c_uchar {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };

    let c_buf = unsafe { CStr::from_ptr(name) };
    let a = c_buf.to_string_lossy().into_owned();
    let Some(ret) = context.setup_animation(&a) else {
        return std::ptr::null_mut()
    };
    Box::into_raw(Box::new(ret)) as *mut c_uchar
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_data(ptr: *mut c_void, frame_no: c_int) -> *mut c_uchar {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    let Some(ret) = context.get_frame_data(frame_no) else {
        return std::ptr::null_mut()
    };
    Box::into_raw(Box::new(ret)) as *mut c_uchar
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_no(ptr: *mut c_void) -> c_int {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.get_frame_no()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_frame_no_decimal(ptr: *mut c_void) -> c_float {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.get_frame_no_decimal()
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_get_end_frame_reached(ptr: *mut c_void) -> bool {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.get_end_frame_reached()
}

// Playback setup
#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_skip_frames(ptr: *mut c_void, skip: bool) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.set_skip_frames(skip);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_loop(ptr: *mut c_void, loops: c_int) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.set_loop(loops);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_rate(ptr: *mut c_void, fps: c_int) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.set_frame_rate(fps);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_playback_direction(ptr: *mut c_void, direction: c_int, style: c_int) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    let d = if direction == 0 { crate::core::track_manager::PlaybackDirection::Forward } else { crate::core::track_manager::PlaybackDirection::Backward };
    let s = if style == 0 { crate::core::track_manager::PlaybackStyle::Normal } else { crate::core::track_manager::PlaybackStyle::PingPong };
    context.set_playback_direction(d, s);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_animation_speed(ptr: *mut c_void, speed_rate: c_float) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.set_animation_speed(speed_rate);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_animation_section(ptr: *mut c_void, start_frame: c_int, end_frame: c_int) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.set_animation_section(start_frame, end_frame);
}

// Playback control

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_update(ptr: *mut c_void, delta: f32) -> i32 {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    let (frame_no, _) = context.update(delta);

    frame_no
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play(ptr: *mut c_void) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.play();
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_play_with_start_frame(ptr: *mut c_void, frame_no: i32) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.play_with_start_frame(frame_no);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_pause(ptr: *mut c_void) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.pause();
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_stop(ptr: *mut c_void) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.stop();
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_set_frame_no(ptr: *mut c_void, frame: c_int) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.set_frame_no(frame);
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_next_frame(ptr: *mut c_void) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.next_frame();
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_runtime_prev_frame(ptr: *mut c_void) {
    let context = unsafe { &mut *(ptr as *mut crate::Context<'_>) };
    context.prev_frame();
}
