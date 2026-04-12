use crate::interpolate::*;

// --- FFI Wrappers (C API) ---

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_linear(value_now: f32, value_start: f32, value_end: f32) -> f32 {
    rate_linear(value_now, value_start, value_end)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_linear(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_accelerate(rate: f32) -> f32 {
    rate_accelerate(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_accelerate(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_accelerate(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_decelerate(rate: f32) -> f32 {
    rate_decelerate(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_decelerate(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_decelerate(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_hermite(rate: f32, speed_start: f32, speed_end: f32) -> f32 {
    hermite(0.0_f32, 1.0_f32, rate, speed_start, speed_end)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_hermite_accuracy(rate: f64, speed_start: f64, speed_end: f64) -> f64 {
    hermite(0.0_f64, 1.0_f64, rate, speed_start, speed_end)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_hermite(value_start: f32, value_end: f32, rate: f32, speed_start: f32, speed_end: f32) -> f32 {
    hermite(value_start, value_end, rate, speed_start, speed_end)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_hermite_accuracy(value_start: f64, value_end: f64, rate: f64, speed_start: f64, speed_end: f64) -> f64 {
    hermite(value_start, value_end, rate, speed_start, speed_end)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_bezier(value_start_x: f32, value_end_x: f32, rate: f32, vector_start_x: f32, vector_start_y: f32, vector_end_x: f32, vector_end_y: f32) -> f32 {
    ss_interpolate_bezier(value_start_x, 0.0_f32, value_end_x, 1.0_f32, rate, vector_start_x, vector_start_y, vector_end_x, vector_end_y)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_bezier_accuracy(value_start_x: f64, value_end_x: f64, rate: f64, vector_start_x: f64, vector_start_y: f64, vector_end_x: f64, vector_end_y: f64) -> f64 {
    ss_interpolate_bezier_accuracy(value_start_x, 0.0_f64, value_end_x, 1.0_f64, rate, vector_start_x, vector_start_y, vector_end_x, vector_end_y)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_bezier(
    value_start_x: f32,
    value_start_y: f32,
    value_end_x: f32,
    value_end_y: f32,
    rate: f32,
    vector_start_x: f32,
    vector_start_y: f32,
    vector_end_x: f32,
    vector_end_y: f32,
) -> f32 {
    bezier(value_start_x, value_start_y, value_end_x, value_end_y, rate, vector_start_x, vector_start_y, vector_end_x, vector_end_y)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_bezier_accuracy(
    value_start_x: f64,
    value_start_y: f64,
    value_end_x: f64,
    value_end_y: f64,
    rate: f64,
    vector_start_x: f64,
    vector_start_y: f64,
    vector_end_x: f64,
    vector_end_y: f64,
) -> f64 {
    bezier(value_start_x, value_start_y, value_end_x, value_end_y, rate, vector_start_x, vector_start_y, vector_end_x, vector_end_y)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_in(rate: f32, power_easing: f32) -> f32 {
    rate_ease_in(rate, power_easing)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_in(value_start: f32, value_end: f32, rate: f32, power_easing: f32) -> f32 {
    linear(value_start, value_end, rate_ease_in(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_out(rate: f32, power_easing: f32) -> f32 {
    rate_ease_out(rate, power_easing)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_out(value_start: f32, value_end: f32, rate: f32, power_easing: f32) -> f32 {
    linear(value_start, value_end, rate_ease_out(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_inout(rate: f32, power_easing: f32) -> f32 {
    rate_ease_inout(rate, power_easing)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_inout(value_start: f32, value_end: f32, rate: f32, power_easing: f32) -> f32 {
    linear(value_start, value_end, rate_ease_inout(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_exponential_in(rate: f32) -> f32 {
    rate_ease_exponential_in(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_exponential_in(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_exponential_in(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_exponential_out(rate: f32) -> f32 {
    rate_ease_exponential_out(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_exponential_out(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_exponential_out(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_exponential_inout(rate: f32) -> f32 {
    rate_ease_exponential_inout(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_exponential_inout(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_exponential_inout(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_sine_in(rate: f32) -> f32 {
    rate_ease_sine_in(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_sine_in(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_sine_in(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_sine_out(rate: f32) -> f32 {
    rate_ease_sine_out(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_sine_out(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_sine_out(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_sine_inout(rate: f32) -> f32 {
    rate_ease_sine_inout(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_sine_inout(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_sine_inout(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_elastic_in(rate: f32, power_easing: f32) -> f32 {
    rate_ease_elastic_in(rate, power_easing)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_elastic_in(value_start: f32, value_end: f32, rate: f32, power_easing: f32) -> f32 {
    linear(value_start, value_end, rate_ease_elastic_in(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_elastic_out(rate: f32, power_easing: f32) -> f32 {
    rate_ease_elastic_out(rate, power_easing)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_elastic_out(value_start: f32, value_end: f32, rate: f32, power_easing: f32) -> f32 {
    linear(value_start, value_end, rate_ease_elastic_out(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_elastic_inout(rate: f32, power_easing: f32) -> f32 {
    rate_ease_elastic_inout(rate, power_easing)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_elastic_inout(value_start: f32, value_end: f32, rate: f32, power_easing: f32) -> f32 {
    linear(value_start, value_end, rate_ease_elastic_inout(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_bounce_in(rate: f32) -> f32 {
    rate_ease_bounce_in(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_bounce_in(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_bounce_in(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_bounce_out(rate: f32) -> f32 {
    rate_ease_bounce_out(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_bounce_out(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_bounce_out(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_bounce_inout(rate: f32) -> f32 {
    rate_ease_bounce_inout(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_bounce_inout(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_bounce_inout(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_back_in(rate: f32) -> f32 {
    rate_ease_back_in(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_back_in(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_back_in(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_back_out(rate: f32) -> f32 {
    rate_ease_back_out(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_back_out(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_back_out(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_back_inout(rate: f32) -> f32 {
    rate_ease_back_inout(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_back_inout(value_start: f32, value_end: f32, rate: f32) -> f32 {
    linear(value_start, value_end, rate_ease_back_inout(rate))
}
