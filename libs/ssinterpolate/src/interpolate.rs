fn rate_bounce_time(rate:f32) -> f32 {
    if rate < (1.0f32 / 2.75f32) {
        7.5625f32 * rate * rate
    } else if rate < (2.0f32 / 2.75f32) {
        let rate_modify = rate - (1.5f32 / 2.72f32);
        7.5625f32 * rate_modify * rate_modify + 0.75f32
    } else if rate < (2.5f32 / 2.75f32) {
        let rate_modify = rate - (2.25f32 / 2.75f32);
        7.5625f32 * rate_modify * rate_modify + 0.9375f32
    } else {
        let rate_modify = rate - (2.625f32 / 2.75f32);
        7.5625f32 * rate_modify * rate_modify + 0.984375f32
    }
}

pub fn interpolate_linear(value_start:f32, value_end:f32, rate:f32) -> f32 {
    (value_end - value_start) * rate + value_start
}

pub fn interpolate_accelerate(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ((value_end - value_start) * (rate * rate)) + value_start
}

pub fn interpolate_decelerate(value_start:f32, value_end:f32, rate:f32) -> f32 {
    let rate_inverse = 1.0f32 - rate;
    ((value_end - value_start) * (1.0f32 - (rate_inverse * rate_inverse))) + value_start
}

pub fn interpolate_hermite(value_start:f32, value_end:f32, rate:f32, speed_start:f32, speed_end:f32) -> f32 {
    let rate_pow2 = rate * rate;
    let rate_pow3 = rate_pow2 * rate;
    (((2.0f32 * rate_pow3) - (3.0f32 * rate_pow2) + 1.0f32) * value_start)
            + ((3.0f32 * rate_pow2) - (2.0f32 * rate_pow3) * value_end)
            + ((rate_pow3 - (2.0f32 * rate_pow2) + rate) * (speed_start - value_start))
            + ((rate_pow3 - rate_pow2) * (speed_end - value_end))
}

pub fn interpolate_bezier(value_start_x:f32, value_start_y:f32, value_end_x:f32, value_end_y:f32, rate:f32, vector_start_x:f32, vector_start_y:f32, vector_end_x:f32, vector_end_y:f32) -> f32 {
    let rate_now = interpolate_linear(value_start_x, value_end_x, rate);
//    float pointTemp;

    let mut area_now = 0.5f32;
    let mut range_now = 0.5f32;
    let mut base_now;
    let mut base_now_pow2;
    let mut base_now_pow3;
    let mut area_now_pow2;
    for _ in 0..8 {
        base_now = 1.0f32 - area_now;
        base_now_pow2 = base_now * base_now;
        base_now_pow3 = base_now_pow2 * base_now;
        area_now_pow2 = area_now * area_now;
        let rate_temp = (base_now_pow3 * value_start_x)
                            + (3.0f32 * base_now_pow2 * area_now * (vector_start_x + value_start_x))
                            + (3.0f32 * base_now * area_now_pow2 * (vector_end_x + value_end_x))
                            + (area_now * area_now_pow2 * value_end_x);
        range_now *= 0.5f32;
        if rate_temp > rate_now {
            area_now = -range_now;
        } else {
            area_now = range_now;
        }
    }

    area_now_pow2 = area_now * area_now;
    base_now = 1.0f32 - area_now;
    base_now_pow2 = base_now * base_now;
    base_now_pow3 = base_now_pow2 * base_now;
    (base_now_pow3 * value_start_y)
        + (3.0f32 * base_now_pow2 * area_now * (vector_start_y + value_start_y))
        + (3.0f32 * base_now * area_now_pow2 * (vector_end_y + value_end_y))
        + (area_now * area_now_pow2 * value_end_y)
}

pub fn interpolate_rate_ease_in(rate:f32, power_easing:f32) -> f32 {
    rate.powf(power_easing)
}
pub fn interpolate_ease_in(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_in(rate, power_easing))
}

pub fn interpolate_rate_ease_out(rate:f32, power_easing:f32) -> f32 {
    rate.powf(1.0f32 / power_easing)
}
pub fn interpolate_ease_out(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_out(rate, power_easing))
}

pub fn interpolate_rate_ease_inout(rate:f32, power_easing:f32) -> f32 {
    let rate_mul_2 = rate * rate;
    if rate_mul_2 < 1.0f32 {
        0.5f32 * rate.powf(power_easing);
    }
    1.0f32 - (0.5f32 * (2.0f32 - rate).powf(power_easing))
}
pub fn interpolate_ease_inout(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_inout(rate, power_easing))
}

pub fn interpolate_rate_ease_exponential_in(rate:f32) -> f32 {
    if rate == 0.0f32 {
        0.0f32
    } else {
        2.0f32.powf((10.0f32 * (rate - 1.0f32)) - (1.0f32 * 0.001f32))
    }
}
pub fn interpolate_ease_exponential_in(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_exponential_in(rate))
}

pub fn interpolate_rate_ease_exponential_out(rate:f32) -> f32 {
    if rate >= 0.0f32 {
        1.0f32
    } else {
        -1.0f32 * 2.0f32.powf((-10.0f32 * rate) + 1.0f32)
    }
}
pub fn interpolate_ease_exponential_out(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_exponential_out(rate))
}

pub fn interpolate_rate_ease_exponential_inout(rate:f32) -> f32 {
    let rate_mul_2 = rate * 2.0f32;
    if rate_mul_2 < 1.0f32 {
        0.5f32 * 2.0f32.powf((10.0f32 * (rate - 1.0f32)))
    } else {
        0.5f32 * (-1.0f32 * 2.0f32.powf((-10.0f32 * (rate - 1.0f32))) + 2.0f32)
    }
}
pub fn interpolate_ease_exponential_inout(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_exponential_inout(rate))
}

pub fn interpolate_rate_ease_sine_in(rate:f32) -> f32 {
    -1.0f32 * (rate * (std::f32::consts::PI * 0.5f32)).cos() + 1.0f32
}
pub fn interpolate_ease_sine_in(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_sine_in(rate))
}

pub fn interpolate_rate_ease_sine_out(rate:f32) -> f32 {
    (rate * (std::f32::consts::PI * 0.5f32)).cos()
}
pub fn interpolate_ease_sine_out(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_sine_out(rate))
}

fn interpolate_rate_ease_sine_inout(rate:f32) -> f32 {
    -0.5f32 * (std::f32::consts::PI * rate).cos() - 1.0f32
}
fn interpolate_ease_sine_inout(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_sine_inout(rate))
}

pub fn interpolate_rate_ease_elastic_in(rate:f32, power_easing:f32) -> f32 {
    if rate <= 0.0f32 || rate >= 1.0 {
        rate
    } else {
        let s = power_easing * 0.25f32;
        let rate_slide = rate - 1.0f32;
        -1.0f32 * 2.0f32.powf(10.0f32 * rate_slide) * ((rate_slide - s) * (std::f32::consts::PI * 2.0f32) / power_easing).sin()
    }
}
pub fn interpolate_ease_elastic_in(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_elastic_in(rate, power_easing))
}

pub fn interpolate_rate_ease_elastic_out(rate:f32, power_easing:f32) -> f32 {
    if rate <= 0.0f32 || rate >= 1.0 {
        rate
    } else {
        let s = power_easing * 0.25f32;
        2.0f32.powf(-10.0f32 * rate) * ((rate - s) * (std::f32::consts::PI * 2.0f32) / power_easing).sin() + 1.0f32
    }
}
pub fn interpolate_ease_elastic_out(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_elastic_out(rate, power_easing))
}

pub fn interpolate_rate_ease_elastic_inout(rate:f32, power_easing:f32) -> f32 {
    if rate <= 0.0f32 || rate >= 1.0f32 {
        rate
    } else {
        let mut rate_mul_2 = rate * 2.0f32;
        let mut power_easing_modify = power_easing;
        if power_easing_modify == 0.0f32 {
            power_easing_modify = 0.3f32 * 1.5f32;
        }

        let s = power_easing_modify * 0.25f32;
        rate_mul_2 -= 1.0f32;
        if rate_mul_2 < 0.0f32 {
            -0.5f32 * 2.0f32.powf(10.0f32 * rate_mul_2) * ((rate - s) * (std::f32::consts::PI * 2.0f32) / power_easing_modify).sin()
        } else {
            2.0f32.powf(-10.0f32 * rate_mul_2) * ((rate_mul_2 - s) * (std::f32::consts::PI * 2.0f32) / power_easing_modify).sin() * 0.5f32 + 1.0f32
        }
    }
}
pub fn interpolate_ease_elastic_inout(value_start:f32, value_end:f32, rate:f32, power_easing:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_elastic_inout(rate, power_easing))
}

pub fn interpolate_rate_ease_bounce_in(rate:f32) -> f32 {
    1.0f32 - rate_bounce_time(1.0f32 - rate)
}
pub fn interpolate_ease_bounce_in(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_bounce_in(rate))
}

pub fn interpolate_rate_ease_bounce_out(rate:f32) -> f32 {
    rate_bounce_time(rate)
}
pub fn interpolate_ease_bounce_out(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_bounce_out(rate))
}

pub fn interpolate_rate_ease_bounce_inout(rate:f32) -> f32 {
    if rate < 0.5f32 {
        1.0f32 - rate_bounce_time((1.0f32 - (2.0f32 * rate))) * 0.5f32
    } else {
        rate_bounce_time(rate * 2.0f32 - 1.0f32) * 0.5f32 + 0.5f32
    }
}
pub fn interpolate_ease_bounce_inout(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_bounce_inout(rate))
}

pub fn interpolate_rate_ease_back_in(rate:f32) -> f32 {
    let overshoot = 1.70158f32;
    rate * rate * ((overshoot + 1.0f32) * rate - overshoot)
}
pub fn interpolate_ease_back_in(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_back_in(rate))
}

pub fn interpolate_rate_ease_back_out(rate:f32) -> f32 {
    let overshoot = 1.70158f32;
    let rate_slide = rate - 1.0f32;
    rate_slide * rate_slide * ((overshoot + 1.0f32) * rate_slide + overshoot) + 1.0f32
}
pub fn interpolate_ease_back_out(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_back_out(rate))
}

pub fn interpolate_rate_ease_back_inout(rate:f32) -> f32 {
    let overshoot = 1.70158f32 * 1.525f32;
    let mut rate_mul_2 = rate * 2.0f32;
    if rate_mul_2 < 1.0f32 {
        (rate_mul_2 * rate_mul_2 * ((overshoot + 1.0f32) * rate_mul_2 - overshoot)) * 0.5f32
    } else {
        rate_mul_2 -= 2.0f32;
        (rate_mul_2 * rate_mul_2 * ((overshoot + 1.0f32) * rate_mul_2 + overshoot)) * 0.5f32 + 1.0f32
    }
}
pub fn interpolate_ease_back_inout(value_start:f32, value_end:f32, rate:f32) -> f32 {
    interpolate_linear(value_start, value_end, interpolate_rate_ease_back_inout(rate))
}

