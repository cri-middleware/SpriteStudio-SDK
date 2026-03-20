use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A trait to abstract floating-point operations for generic interpolation.
pub trait SsFloat:
    Sized
    + Copy
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
{
    fn from_f32(v: f32) -> Self;
    fn to_i32(self) -> i32;
    fn powf(self, n: Self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn pi() -> Self;
}

impl SsFloat for f32 {
    #[inline] fn from_f32(v: f32) -> Self { v }
    #[inline] fn to_i32(self) -> i32 { self as i32 }
    #[inline] fn powf(self, n: Self) -> Self { f32::powf(self, n) }
    #[inline] fn sin(self) -> Self { f32::sin(self) }
    #[inline] fn cos(self) -> Self { f32::cos(self) }
    #[inline] fn pi() -> Self { std::f32::consts::PI }
}

impl SsFloat for f64 {
    #[inline] fn from_f32(v: f32) -> Self { v as f64 }
    #[inline] fn to_i32(self) -> i32 { self as i32 }
    #[inline] fn powf(self, n: Self) -> Self { f64::powf(self, n) }
    #[inline] fn sin(self) -> Self { f64::sin(self) }
    #[inline] fn cos(self) -> Self { f64::cos(self) }
    #[inline] fn pi() -> Self { std::f64::consts::PI }
}

// --- Core Generic Implementations ---

fn bounce_time<T: SsFloat>(rate: T) -> T {
    let div_2_75 = T::from_f32(1.0 / 2.75);
    let div_2_0_2_75 = T::from_f32(2.0 / 2.75);
    let div_2_5_2_75 = T::from_f32(2.5 / 2.75);
    
    let mul_7_5625 = T::from_f32(7.5625);

    if rate < div_2_75 {
        mul_7_5625 * rate * rate
    } else if rate < div_2_0_2_75 {
        let rate_modify = rate - T::from_f32(1.5 / 2.72); // Note: original code has 2.72 here, keeping it to match behavior
        mul_7_5625 * rate_modify * rate_modify + T::from_f32(0.75)
    } else if rate < div_2_5_2_75 {
        let rate_modify = rate - T::from_f32(2.25 / 2.75);
        mul_7_5625 * rate_modify * rate_modify + T::from_f32(0.9375)
    } else {
        let rate_modify = rate - T::from_f32(2.625 / 2.75);
        mul_7_5625 * rate_modify * rate_modify + T::from_f32(0.984375)
    }
}

pub fn rate_linear<T: SsFloat>(value_now: T, value_start: T, value_end: T) -> T {
    (value_now - value_start) / (value_end - value_start)
}

pub fn linear<T: SsFloat>(value_start: T, value_end: T, rate: T) -> T {
    (value_end - value_start) * rate + value_start
}

pub fn rate_accelerate<T: SsFloat>(rate: T) -> T {
    rate * rate
}

pub fn rate_decelerate<T: SsFloat>(rate: T) -> T {
    let rate_inverse = T::from_f32(1.0) - rate;
    T::from_f32(1.0) - (rate_inverse * rate_inverse)
}

pub fn hermite<T: SsFloat>(value_start: T, value_end: T, rate: T, speed_start: T, speed_end: T) -> T {
    let rate_pow2 = rate * rate;
    let rate_pow3 = rate_pow2 * rate;
    
    let two = T::from_f32(2.0);
    let three = T::from_f32(3.0);
    let one = T::from_f32(1.0);

    (((two * rate_pow3) - (three * rate_pow2) + one) * value_start)
        + (((three * rate_pow2) - (two * rate_pow3)) * value_end)
        + ((rate_pow3 - (two * rate_pow2) + rate) * (speed_start - value_start))
        + ((rate_pow3 - rate_pow2) * (speed_end - value_end))
}

pub fn bezier<T: SsFloat>(
    value_start_x: T, value_start_y: T,
    value_end_x: T, value_end_y: T,
    rate: T,
    vector_start_x: T, vector_start_y: T,
    vector_end_x: T, vector_end_y: T
) -> T {
    let frame_i32 = linear(value_start_x, value_end_x, rate).to_i32();
    let frame = T::from_f32(frame_i32 as f32);

    let mut area_now = T::from_f32(0.5);
    let mut range_now = T::from_f32(0.5);
    let mut base_now;
    let mut base_now_pow2;
    let mut base_now_pow3;
    let mut area_now_pow2;
    
    let one = T::from_f32(1.0);
    let three = T::from_f32(3.0);
    let half = T::from_f32(0.5);

    for _ in 0..8 {
        base_now = one - area_now;
        base_now_pow2 = base_now * base_now;
        base_now_pow3 = base_now_pow2 * base_now;
        area_now_pow2 = area_now * area_now;
        
        let frame_temp = (base_now_pow3 * value_start_x)
            + (three * base_now_pow2 * area_now * (vector_start_x + value_start_x))
            + (three * base_now * area_now_pow2 * (vector_end_x + value_end_x))
            + (area_now * area_now_pow2 * value_end_x);
            
        range_now *= half;
        if frame_temp > frame {
            area_now -= range_now;
        } else {
            area_now += range_now;
        }
    }

    area_now_pow2 = area_now * area_now;
    base_now = one - area_now;
    base_now_pow2 = base_now * base_now;
    base_now_pow3 = base_now_pow2 * base_now;
    
    (base_now_pow3 * value_start_y)
        + (three * base_now_pow2 * area_now * (vector_start_y + value_start_y))
        + (three * base_now * area_now_pow2 * (vector_end_y + value_end_y))
        + (area_now * area_now_pow2 * value_end_y)
}

pub fn rate_ease_in<T: SsFloat>(rate: T, power_easing: T) -> T {
    rate.powf(power_easing)
}

pub fn rate_ease_out<T: SsFloat>(rate: T, power_easing: T) -> T {
    rate.powf(T::from_f32(1.0) / power_easing)
}

<<<<<<< HEAD
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
||||||| parent of 8303652 (declare public rust methods in ssinterpolate)
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
    let rate_mul_2 = rate * rate;
    if rate_mul_2 < 1.0_f32 {
        0.5_f32 * rate.powf(power_easing)
=======
pub fn rate_ease_inout<T: SsFloat>(rate: T, power_easing: T) -> T {
    let rate_mul_2 = rate * T::from_f32(2.0);
    if rate_mul_2 < T::from_f32(1.0) {
        T::from_f32(0.5) * rate.powf(power_easing)
>>>>>>> 8303652 (declare public rust methods in ssinterpolate)
    } else {
        T::from_f32(1.0) - (T::from_f32(0.5) * (T::from_f32(2.0) - rate).powf(power_easing))
    }
}

<<<<<<< HEAD
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_exponential_in(rate:f32) -> f32 {
    if rate.abs() <= f32::EPSILON {
        0.0_f32
||||||| parent of 8303652 (declare public rust methods in ssinterpolate)
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_exponential_in(rate:f32) -> f32 {
    if rate == 0.0_f32 {
        0.0_f32
=======
pub fn rate_ease_exponential_in<T: SsFloat>(rate: T) -> T {
    if rate == T::from_f32(0.0) {
        T::from_f32(0.0)
>>>>>>> 8303652 (declare public rust methods in ssinterpolate)
    } else {
        T::from_f32(2.0).powf((T::from_f32(10.0) * (rate - T::from_f32(1.0))) - T::from_f32(0.001))
    }
}

pub fn rate_ease_exponential_out<T: SsFloat>(rate: T) -> T {
    if rate >= T::from_f32(0.0) {
        T::from_f32(1.0)
    } else {
        T::from_f32(-1.0) * T::from_f32(2.0).powf((T::from_f32(-10.0) * rate) + T::from_f32(1.0))
    }
}

<<<<<<< HEAD
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_exponential_inout(rate:f32) -> f32 {
    if (rate * 2.0_f32) < 1.0_f32 {
        0.5_f32 * 2.0_f32.powf(10.0_f32 * (rate - 1.0_f32))
||||||| parent of 8303652 (declare public rust methods in ssinterpolate)
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_exponential_inout(rate:f32) -> f32 {
    let rate_mul_2 = rate * 2.0_f32;
    if rate_mul_2 < 1.0_f32 {
        0.5_f32 * 2.0_f32.powf(10.0_f32 * (rate - 1.0_f32))
=======
pub fn rate_ease_exponential_inout<T: SsFloat>(rate: T) -> T {
    let rate_mul_2 = rate * T::from_f32(2.0);
    if rate_mul_2 < T::from_f32(1.0) {
        T::from_f32(0.5) * T::from_f32(2.0).powf(T::from_f32(10.0) * (rate - T::from_f32(1.0)))
>>>>>>> 8303652 (declare public rust methods in ssinterpolate)
    } else {
        T::from_f32(0.5) * (T::from_f32(-1.0) * T::from_f32(2.0).powf(T::from_f32(-10.0) * (rate - T::from_f32(1.0))) + T::from_f32(2.0))
    }
}

pub fn rate_ease_sine_in<T: SsFloat>(rate: T) -> T {
    T::from_f32(-1.0) * (rate * (T::pi() * T::from_f32(0.5))).cos() + T::from_f32(1.0)
}

pub fn rate_ease_sine_out<T: SsFloat>(rate: T) -> T {
    (rate * (T::pi() * T::from_f32(0.5))).cos()
}

pub fn rate_ease_sine_inout<T: SsFloat>(rate: T) -> T {
    T::from_f32(-0.5) * (T::pi() * rate).cos() - T::from_f32(1.0)
}

pub fn rate_ease_elastic_in<T: SsFloat>(rate: T, power_easing: T) -> T {
    let zero = T::from_f32(0.0);
    let one = T::from_f32(1.0);
    if rate <= zero || rate >= one {
        rate
    } else {
        let s = power_easing * T::from_f32(0.25);
        let rate_slide = rate - one;
        T::from_f32(-1.0) * T::from_f32(2.0).powf(T::from_f32(10.0) * rate_slide) * ((rate_slide - s) * (T::pi() * T::from_f32(2.0)) / power_easing).sin()
    }
}

pub fn rate_ease_elastic_out<T: SsFloat>(rate: T, power_easing: T) -> T {
    let zero = T::from_f32(0.0);
    let one = T::from_f32(1.0);
    if rate <= zero || rate >= one {
        rate
    } else {
        let s = power_easing * T::from_f32(0.25);
        T::from_f32(2.0).powf(T::from_f32(-10.0) * rate) * ((rate - s) * (T::pi() * T::from_f32(2.0)) / power_easing).sin() + one
    }
}

pub fn rate_ease_elastic_inout<T: SsFloat>(rate: T, power_easing: T) -> T {
    let zero = T::from_f32(0.0);
    let one = T::from_f32(1.0);
    if rate <= zero || rate >= one {
        rate
    } else {
        let mut rate_mul_2 = rate * T::from_f32(2.0);
        let mut power_easing_modify = power_easing;
        if power_easing_modify == zero {
            power_easing_modify = T::from_f32(0.3 * 1.5);
        }

        let s = power_easing_modify * T::from_f32(0.25);
        rate_mul_2 -= one;
        if rate_mul_2 < zero {
            T::from_f32(-0.5) * T::from_f32(2.0).powf(T::from_f32(10.0) * rate_mul_2) * ((rate - s) * (T::pi() * T::from_f32(2.0)) / power_easing_modify).sin()
        } else {
            T::from_f32(2.0).powf(T::from_f32(-10.0) * rate_mul_2) * ((rate_mul_2 - s) * (T::pi() * T::from_f32(2.0)) / power_easing_modify).sin() * T::from_f32(0.5) + one
        }
    }
}

pub fn rate_ease_bounce_in<T: SsFloat>(rate: T) -> T {
    T::from_f32(1.0) - bounce_time(T::from_f32(1.0) - rate)
}

pub fn rate_ease_bounce_out<T: SsFloat>(rate: T) -> T {
    bounce_time(rate)
}

pub fn rate_ease_bounce_inout<T: SsFloat>(rate: T) -> T {
    if rate < T::from_f32(0.5) {
        T::from_f32(1.0) - bounce_time(T::from_f32(1.0) - (T::from_f32(2.0) * rate)) * T::from_f32(0.5)
    } else {
        bounce_time(rate * T::from_f32(2.0) - T::from_f32(1.0)) * T::from_f32(0.5) + T::from_f32(0.5)
    }
}

pub fn rate_ease_back_in<T: SsFloat>(rate: T) -> T {
    let overshoot = T::from_f32(1.70158);
    rate * rate * ((overshoot + T::from_f32(1.0)) * rate - overshoot)
}

<<<<<<< HEAD
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_back_in(rate:f32) -> f32 {
    rate * rate * ((EASE_BACK_OVERRUN + 1.0_f32) * rate - EASE_BACK_OVERRUN)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_back_in(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_back_in(rate))
||||||| parent of 8303652 (declare public rust methods in ssinterpolate)
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_back_in(rate:f32) -> f32 {
    let overshoot = 1.70158_f32;
    rate * rate * ((overshoot + 1.0_f32) * rate - overshoot)
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_back_in(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_back_in(rate))
=======
pub fn rate_ease_back_out<T: SsFloat>(rate: T) -> T {
    let overshoot = T::from_f32(1.70158);
    let rate_slide = rate - T::from_f32(1.0);
    rate_slide * rate_slide * ((overshoot + T::from_f32(1.0)) * rate_slide + overshoot) + T::from_f32(1.0)
>>>>>>> 8303652 (declare public rust methods in ssinterpolate)
}

<<<<<<< HEAD
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
||||||| parent of 8303652 (declare public rust methods in ssinterpolate)
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_back_out(rate:f32) -> f32 {
    let overshoot = 1.70158_f32;
    let rate_slide = rate - 1.0_f32;
    rate_slide * rate_slide * ((overshoot + 1.0_f32) * rate_slide + overshoot) + 1.0_f32
}
#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_ease_back_out(value_start:f32, value_end:f32, rate:f32) -> f32 {
    ss_interpolate_linear(value_start, value_end, ss_interpolate_rate_ease_back_out(rate))
}

#[unsafe(no_mangle)]
pub extern "C" fn ss_interpolate_rate_ease_back_inout(rate:f32) -> f32 {
    let overshoot = 1.70158_f32 * 1.525_f32;
    let mut rate_mul_2 = rate * 2.0_f32;
    if rate_mul_2 < 1.0_f32 {
        (rate_mul_2 * rate_mul_2 * ((overshoot + 1.0_f32) * rate_mul_2 - overshoot)) * 0.5_f32
=======
pub fn rate_ease_back_inout<T: SsFloat>(rate: T) -> T {
    let overshoot = T::from_f32(1.70158 * 1.525);
    let mut rate_mul_2 = rate * T::from_f32(2.0);
    if rate_mul_2 < T::from_f32(1.0) {
        (rate_mul_2 * rate_mul_2 * ((overshoot + T::from_f32(1.0)) * rate_mul_2 - overshoot)) * T::from_f32(0.5)
>>>>>>> 8303652 (declare public rust methods in ssinterpolate)
    } else {
<<<<<<< HEAD
        rate_mul_2 -= 2.0_f32;
        (rate_mul_2 * rate_mul_2 * ((EASE_BACK_INOUT_OVERRUN + 1.0_f32) * rate_mul_2 + EASE_BACK_INOUT_OVERRUN)) * 0.5_f32 + 1.0_f32
||||||| parent of 8303652 (declare public rust methods in ssinterpolate)
        rate_mul_2 -= 2.0_f32;
        (rate_mul_2 * rate_mul_2 * ((overshoot + 1.0_f32) * rate_mul_2 + overshoot)) * 0.5_f32 + 1.0_f32
=======
        rate_mul_2 -= T::from_f32(2.0);
        (rate_mul_2 * rate_mul_2 * ((overshoot + T::from_f32(1.0)) * rate_mul_2 + overshoot)) * T::from_f32(0.5) + T::from_f32(1.0)
>>>>>>> 8303652 (declare public rust methods in ssinterpolate)
    }
}

