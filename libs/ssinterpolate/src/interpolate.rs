// Numeric-literals in this source-code are not written as "X as T". (Because often appears  in mathematical-expressions.)
const EASE_BACK_OVERRUN: f32 = 1.70158_f32;
const EASE_BACK_INOUT_OVERRUN: f32 = EASE_BACK_OVERRUN * 1.525_f32;

fn bounce_time(rate:f32) -> f32 {
    if rate < (1.0_f32 / 2.75_f32) {
        7.5625_f32 * rate * rate
    } else if rate < (2.0_f32 / 2.75_f32) {
        let rate_modify = rate - (1.5_f32 / 2.72_f32);
        7.5625_f32 * rate_modify * rate_modify + 0.75_f32
    } else if rate < (2.5_f32 / 2.75_f32) {
        let rate_modify = rate - (2.25_f32 / 2.75_f32);
        7.5625f32 * rate_modify * rate_modify + 0.9375_f32
    } else {
        let rate_modify = rate - (2.625_f32 / 2.75_f32);
        7.5625_f32 * rate_modify * rate_modify + 0.984375_f32
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_linear(value_now:f32, value_start:f32, value_end:f32) -> f32 {
    (value_now - value_start) / (value_end - value_start)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_linear(value_start:f32, value_end:f32, rate:f32) -> f32 {
    (value_end - value_start) * rate + value_start
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_accelerate(rate:f32) -> f32 {
    rate * rate
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_accelerate(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ((value_end - value_start) * (rate * rate)) + value_start
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_decelerate(rate:f32) -> f32 {
    let rate_inverse = 1.0_f32 - rate;
    1.0_f32 - (rate_inverse * rate_inverse)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_decelerate(value_start:f32, value_end:f32, rate:f32) -> f32 {
    let rate_inverse = 1.0_f32 - rate;
    ((value_end - value_start) * (1.0_f32 - (rate_inverse * rate_inverse))) + value_start
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_hermite(rate:f32, speed_start:f32, speed_end:f32) -> f32 {
    ss_interpolate_hermite(0.0_f32, 1.0_f32, rate, speed_start, speed_end)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_hermite_accuracy(rate:f64, speed_start:f64, speed_end:f64) -> f64 {
    // MEMO: for Converter
    ss_interpolate_hermite_accuracy(0.0_f64, 1.0_f64, rate, speed_start, speed_end)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_hermite(value_start:f32, value_end:f32, rate:f32, speed_start:f32, speed_end:f32) -> f32 {
    let rate_pow2 = rate * rate;
    let rate_pow3 = rate_pow2 * rate;
    (((2.0_f32 * rate_pow3) - (3.0_f32 * rate_pow2) + 1.0_f32) * value_start)
            + (((3.0_f32 * rate_pow2) - (2.0_f32 * rate_pow3)) * value_end)
            + ((rate_pow3 - (2.0_f32 * rate_pow2) + rate) * (speed_start - value_start))
            + ((rate_pow3 - rate_pow2) * (speed_end - value_end))
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_hermite_accuracy(value_start:f64, value_end:f64, rate:f64, speed_start:f64, speed_end:f64) -> f64 {
    // MEMO: for Converter
    let rate_pow2 = rate * rate;
    let rate_pow3 = rate_pow2 * rate;
    (((2.0_f64 * rate_pow3) - (3.0_f64 * rate_pow2) + 1.0_f64) * value_start)
        + (((3.0_f64 * rate_pow2) - (2.0_f64 * rate_pow3)) * value_end)
        + ((rate_pow3 - (2.0_f64 * rate_pow2) + rate) * (speed_start - value_start))
        + ((rate_pow3 - rate_pow2) * (speed_end - value_end))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_bezier(value_start_x:f32, value_end_x:f32, rate:f32, vector_start_x:f32, vector_start_y:f32, vector_end_x:f32, vector_end_y:f32) -> f32 {
    ss_interpolate_bezier(value_start_x, 0.0_f32, value_end_x, 1.0_f32, rate, vector_start_x, vector_start_y, vector_end_x, vector_end_y)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_bezier_accuracy(value_start_x:f64, value_end_x:f64, rate:f64, vector_start_x:f64, vector_start_y:f64, vector_end_x:f64, vector_end_y:f64) -> f64 {
    // MEMO: for Converter
    ss_interpolate_bezier_accuracy(value_start_x, 0.0_f64, value_end_x, 1.0_f64, rate, vector_start_x, vector_start_y, vector_end_x, vector_end_y)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_bezier(value_start_x:f32, value_start_y:f32, value_end_x:f32, value_end_y:f32, rate:f32, vector_start_x:f32, vector_start_y:f32, vector_end_x:f32, vector_end_y:f32) -> f32 {
    let frame = (ss_interpolate_linear(value_start_x, value_end_x, rate) as i32) as f32;

    let mut area_now = 0.5_f32;
    let mut range_now = 0.5_f32;
    let mut base_now;
    let mut base_now_pow2;
    let mut base_now_pow3;
    let mut area_now_pow2;

    for _ in 0..8 {
        base_now = 1.0_f32 - area_now;
        base_now_pow2 = base_now * base_now;
        base_now_pow3 = base_now_pow2 * base_now;
        area_now_pow2 = area_now * area_now;
        let frame_temp = (base_now_pow3 * value_start_x)
                            + (3.0_f32 * base_now_pow2 * area_now * (vector_start_x + value_start_x))
                            + (3.0_f32 * base_now * area_now_pow2 * (vector_end_x + value_end_x))
                            + (area_now * area_now_pow2 * value_end_x);
        range_now *= 0.5_f32;
        if frame_temp > frame {
            area_now -= range_now;
        } else {
            area_now += range_now;
        }
    }

    area_now_pow2 = area_now * area_now;
    base_now = 1.0_f32 - area_now;
    base_now_pow2 = base_now * base_now;
    base_now_pow3 = base_now_pow2 * base_now;
    (base_now_pow3 * value_start_y)
        + (3.0_f32 * base_now_pow2 * area_now * (vector_start_y + value_start_y))
        + (3.0_f32 * base_now * area_now_pow2 * (vector_end_y + value_end_y))
        + (area_now * area_now_pow2 * value_end_y)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_bezier_accuracy(value_start_x:f64, value_start_y:f64, value_end_x:f64, value_end_y:f64, rate:f64, vector_start_x:f64, vector_start_y:f64, vector_end_x:f64, vector_end_y:f64) -> f64 {
    // MEMO: for Converter
    let frame = (ss_interpolate_linear(value_start_x as f32, value_end_x as f32, rate as f32) as i32) as f64;

    let mut area_now = 0.5_f64;
    let mut range_now = 0.5_f64;
    let mut base_now;
    let mut base_now_pow2;
    let mut base_now_pow3;
    let mut area_now_pow2;

    for _ in 0..8 {
        base_now = 1.0_f64 - area_now;
        base_now_pow2 = base_now * base_now;
        base_now_pow3 = base_now_pow2 * base_now;
        area_now_pow2 = area_now * area_now;
        let frame_temp = (base_now_pow3 * value_start_x)
            + (3.0_f64 * base_now_pow2 * area_now * (vector_start_x + value_start_x))
            + (3.0_f64 * base_now * area_now_pow2 * (vector_end_x + value_end_x))
            + (area_now * area_now_pow2 * value_end_x);
        range_now *= 0.5_f64;
        if frame_temp > frame {
            area_now -= range_now;
        } else {
            area_now += range_now;
        }
    }

    area_now_pow2 = area_now * area_now;
    base_now = 1.0_f64 - area_now;
    base_now_pow2 = base_now * base_now;
    base_now_pow3 = base_now_pow2 * base_now;
    (base_now_pow3 * value_start_y)
        + (3.0_f64 * base_now_pow2 * area_now * (vector_start_y + value_start_y))
        + (3.0_f64 * base_now * area_now_pow2 * (vector_end_y + value_end_y))
        + (area_now * area_now_pow2 * value_end_y)
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_in(rate:f32, power_easing:f32) -> f32 {
    rate.powf(power_easing)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_in(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_in(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_out(rate:f32, power_easing:f32) -> f32 {
    rate.powf(1.0_f32 / power_easing)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_out(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_out(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_inout(rate:f32, power_easing:f32) -> f32 {
    if (rate * 2.0_f32) < 1.0_f32 {
        0.5_f32 * rate.powf(power_easing)
    } else {
        1.0_f32 - (0.5_f32 * (2.0_f32 - rate).powf(power_easing))
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_inout(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_inout(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_exponential_in(rate:f32) -> f32 {
    if rate.abs() <= f32::EPSILON {
        0.0_f32
    } else {
        2.0_f32.powf((10.0_f32 * (rate - 1.0_f32)) - (1.0_f32 * 0.001_f32))
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_exponential_in(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_exponential_in(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_exponential_out(rate:f32) -> f32 {
    if rate >= 0.0_f32 {
        1.0_f32
    } else {
        -1.0_f32 * 2.0_f32.powf((-10.0_f32 * rate) + 1.0_f32)
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_exponential_out(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_exponential_out(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_exponential_inout(rate:f32) -> f32 {
    if (rate * 2.0_f32) < 1.0_f32 {
        0.5_f32 * 2.0_f32.powf(10.0_f32 * (rate - 1.0_f32))
    } else {
        0.5_f32 * (-1.0_f32 * 2.0_f32.powf(-10.0_f32 * (rate - 1.0_f32)) + 2.0_f32)
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_exponential_inout(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_exponential_inout(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_sine_in(rate:f32) -> f32 {
    -1.0_f32 * (rate * (std::f32::consts::PI * 0.5_f32)).cos() + 1.0_f32
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_sine_in(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_sine_in(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_sine_out(rate:f32) -> f32 {
    (rate * (std::f32::consts::PI * 0.5_f32)).cos()
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_sine_out(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_sine_out(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_sine_inout(rate:f32) -> f32 {
    -0.5_f32 * (std::f32::consts::PI * rate).cos() - 1.0_f32
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_sine_inout(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_sine_inout(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_elastic_in(rate:f32, power_easing:f32) -> f32 {
    if rate <= 0.0_f32 || rate >= 1.0 {
        rate
    } else {
        let s = power_easing * 0.25_f32;
        let rate_slide = rate - 1.0_f32;
        -1.0_f32 * 2.0_f32.powf(10.0_f32 * rate_slide) * ((rate_slide - s) * (std::f32::consts::PI * 2.0_f32) / power_easing).sin()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_elastic_in(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_elastic_in(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_elastic_out(rate:f32, power_easing:f32) -> f32 {
    if rate <= 0.0_f32 || rate >= 1.0 {
        rate
    } else {
        let s = power_easing * 0.25_f32;
        2.0_f32.powf(-10.0_f32 * rate) * ((rate - s) * (std::f32::consts::PI * 2.0_f32) / power_easing).sin() + 1.0_f32
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_elastic_out(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_elastic_out(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_elastic_inout(rate:f32, power_easing:f32) -> f32 {
    if rate <= 0.0_f32 || rate >= 1.0_f32 {
        rate
    } else {
        let mut rate_mul_2 = rate * 2.0_f32;
        let mut power_easing_modify = power_easing;
        if power_easing_modify == 0.0_f32 {
            power_easing_modify = 0.3_f32 * 1.5_f32;
        }

        let s = power_easing_modify * 0.25_f32;
        rate_mul_2 -= 1.0_f32;
        if rate_mul_2 < 0.0_f32 {
            -0.5_f32 * 2.0_f32.powf(10.0_f32 * rate_mul_2) * ((rate - s) * (std::f32::consts::PI * 2.0_f32) / power_easing_modify).sin()
        } else {
            2.0_f32.powf(-10.0_f32 * rate_mul_2) * ((rate_mul_2 - s) * (std::f32::consts::PI * 2.0_f32) / power_easing_modify).sin() * 0.5_f32 + 1.0_f32
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_elastic_inout(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_elastic_inout(rate, power_easing))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_bounce_in(rate:f32) -> f32 {
    1.0_f32 - bounce_time(1.0_f32 - rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_bounce_in(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_bounce_in(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_bounce_out(rate:f32) -> f32 {
    bounce_time(rate)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_bounce_out(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_bounce_out(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_bounce_inout(rate:f32) -> f32 {
    if rate < 0.5_f32 {
        1.0_f32 - bounce_time(1.0_f32 - (2.0_f32 * rate)) * 0.5_f32
    } else {
        bounce_time(rate * 2.0_f32 - 1.0_f32) * 0.5_f32 + 0.5_f32
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_bounce_inout(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_bounce_inout(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_back_in(rate:f32) -> f32 {
    rate * rate * ((EASE_BACK_OVERRUN + 1.0_f32) * rate - EASE_BACK_OVERRUN)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_back_in(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_back_in(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_back_out(rate:f32) -> f32 {
    let rate_slide = rate - 1.0_f32;
    rate_slide * rate_slide * ((EASE_BACK_OVERRUN + 1.0_f32) * rate_slide + EASE_BACK_OVERRUN) + 1.0_f32
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_back_out(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_back_out(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_back_inout(rate:f32) -> f32 {
    let mut rate_mul_2 = rate * 2.0_f32;
    if rate_mul_2 < 1.0_f32 {
        (rate_mul_2 * rate_mul_2 * ((EASE_BACK_INOUT_OVERRUN + 1.0_f32) * rate_mul_2 - EASE_BACK_INOUT_OVERRUN)) * 0.5_f32
    } else {
        rate_mul_2 -= 2.0_f32;
        (rate_mul_2 * rate_mul_2 * ((EASE_BACK_INOUT_OVERRUN + 1.0_f32) * rate_mul_2 + EASE_BACK_INOUT_OVERRUN)) * 0.5_f32 + 1.0_f32
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_back_inout(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_back_inout(rate))
}
