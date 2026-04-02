use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

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
    #[inline] fn powf(self, n: Self) -> Self { libm::powf(self, n) }
    #[inline] fn sin(self) -> Self { libm::sinf(self) }
    #[inline] fn cos(self) -> Self { libm::cosf(self) }
    #[inline] fn pi() -> Self { core::f32::consts::PI }
}

impl SsFloat for f64 {
    #[inline] fn from_f32(v: f32) -> Self { v as f64 }
    #[inline] fn to_i32(self) -> i32 { self as i32 }
    #[inline] fn powf(self, n: Self) -> Self { libm::pow(self, n) }
    #[inline] fn sin(self) -> Self { libm::sin(self) }
    #[inline] fn cos(self) -> Self { libm::cos(self) }
    #[inline] fn pi() -> Self { core::f64::consts::PI }
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
        let rate_modify = rate - T::from_f32(1.5 / 2.75);
        mul_7_5625 * rate_modify * rate_modify + T::from_f32(0.75)
    } else if rate < div_2_5_2_75 {
        let rate_modify = rate - T::from_f32(2.25 / 2.75);
        mul_7_5625 * rate_modify * rate_modify + T::from_f32(0.9375)
    } else {
        let rate_modify = rate - T::from_f32(2.625 / 2.75);
        mul_7_5625 * rate_modify * rate_modify + T::from_f32(0.984375)
    }
}

use wide::*;

#[inline]
pub fn linear_batch_f32x4(start: f32x4, end: f32x4, rate: f32x4) -> f32x4 {
    (end - start).mul_add(rate, start)
}

#[inline]
pub fn rate_linear_batch_f32x4(value_now: f32x4, value_start: f32x4, value_end: f32x4) -> f32x4 {
    (value_now - value_start) / (value_end - value_start)
}

#[inline]
pub fn bezier_batch_f32x4(
    value_start_x: f32x4, value_start_y: f32x4,
    value_end_x: f32x4, value_end_y: f32x4,
    rate: f32x4,
    vector_start_x: f32x4, vector_start_y: f32x4,
    vector_end_x: f32x4, vector_end_y: f32x4
) -> f32x4 {
    let frame = linear_batch_f32x4(value_start_x, value_end_x, rate);

    let mut area_now = f32x4::splat(0.5);
    let mut range_now = f32x4::splat(0.5);
    
    let one = f32x4::splat(1.0);
    let three = f32x4::splat(3.0);
    let half = f32x4::splat(0.5);

    for _ in 0..8 {
        let base_now = one - area_now;
        let base_now_pow2 = base_now * base_now;
        let base_now_pow3 = base_now_pow2 * base_now;
        let area_now_pow2 = area_now * area_now;
        
        let frame_temp = (base_now_pow3 * value_start_x)
            + (three * base_now_pow2 * area_now * (vector_start_x + value_start_x))
            + (three * base_now * area_now_pow2 * (vector_end_x + value_end_x))
            + (area_now * area_now_pow2 * value_end_x);
            
        range_now = range_now * half;
        
        let m_lt = frame.simd_lt(frame_temp);
        
        let area_sub = area_now - range_now;
        let area_add = area_now + range_now;
        
        area_now = m_lt.blend(area_sub, area_add);
    }

    let area_now_pow2 = area_now * area_now;
    let base_now = one - area_now;
    let base_now_pow2 = base_now * base_now;
    let base_now_pow3 = base_now_pow2 * base_now;
    
    (base_now_pow3 * value_start_y)
        + (three * base_now_pow2 * area_now * (vector_start_y + value_start_y))
        + (three * base_now * area_now_pow2 * (vector_end_y + value_end_y))
        + (area_now * area_now_pow2 * value_end_y)
}

#[inline]
pub fn hermite_batch_f32x4(
    v_start: f32x4, v_end: f32x4,
    rate: f32x4,
    s_start: f32x4, s_end: f32x4
) -> f32x4 {
    let rate_pow2 = rate * rate;
    let rate_pow3 = rate_pow2 * rate;
    
    let two = f32x4::splat(2.0);
    let three = f32x4::splat(3.0);
    let one = f32x4::splat(1.0);

    let c1 = two.mul_add(rate_pow3, one) - (three * rate_pow2);
    let c2 = (three * rate_pow2) - (two * rate_pow3);
    let c3 = rate_pow3 - (two * rate_pow2) + rate;
    let c4 = rate_pow3 - rate_pow2;

    c1.mul_add(v_start, c2.mul_add(v_end, c3.mul_add(s_start - v_start, c4 * (s_end - v_end))))
}

/// Batch linear interpolation for arrays (e.g. Deform attributes).
pub fn interpolate_array_linear(start: &[f32], end: &[f32], rate: f32, out: &mut [f32]) {
    let rate_v = f32x4::splat(rate);
    
    let mut start_chunks = start.chunks_exact(4);
    let mut end_chunks = end.chunks_exact(4);
    let mut out_chunks = out.chunks_exact_mut(4);

    for ((s, e), o) in start_chunks.by_ref().zip(end_chunks.by_ref()).zip(out_chunks.by_ref()) {
        let s_v = f32x4::from([s[0], s[1], s[2], s[3]]);
        let e_v = f32x4::from([e[0], e[1], e[2], e[3]]);
        let res = linear_batch_f32x4(s_v, e_v, rate_v);
        o.copy_from_slice(&res.to_array());
    }
    
    let start_rem = start_chunks.remainder();
    let end_rem = end_chunks.remainder();
    let out_rem = out_chunks.into_remainder();

    for i in 0..start_rem.len() {
        out_rem[i] = linear(start_rem[i], end_rem[i], rate);
    }
}

/// Batch Hermite interpolation for arrays (e.g. Deform attributes).
pub fn interpolate_array_hermite(
    v_start: &[f32], v_end: &[f32],
    rate: f32,
    s_start: &[f32], s_end: &[f32],
    out: &mut [f32]
) {
    let rate_v = f32x4::splat(rate);
    
    let mut vs_chunks = v_start.chunks_exact(4);
    let mut ve_chunks = v_end.chunks_exact(4);
    let mut ss_chunks = s_start.chunks_exact(4);
    let mut se_chunks = s_end.chunks_exact(4);
    let mut out_chunks = out.chunks_exact_mut(4);

    let zip1 = vs_chunks.by_ref().zip(ve_chunks.by_ref());
    let zip2 = ss_chunks.by_ref().zip(se_chunks.by_ref());
    let iter = zip1.zip(zip2).zip(out_chunks.by_ref());

    for (((vs, ve), (ss, se)), o) in iter {
        let vs_v = f32x4::from([vs[0], vs[1], vs[2], vs[3]]);
        let ve_v = f32x4::from([ve[0], ve[1], ve[2], ve[3]]);
        let ss_v = f32x4::from([ss[0], ss[1], ss[2], ss[3]]);
        let se_v = f32x4::from([se[0], se[1], se[2], se[3]]);
        
        let res = hermite_batch_f32x4(vs_v, ve_v, rate_v, ss_v, se_v);
        o.copy_from_slice(&res.to_array());
    }
    
    let vs_rem = vs_chunks.remainder();
    let ve_rem = ve_chunks.remainder();
    let ss_rem = ss_chunks.remainder();
    let se_rem = se_chunks.remainder();
    let out_rem = out_chunks.into_remainder();

    for i in 0..vs_rem.len() {
        out_rem[i] = hermite(vs_rem[i], ve_rem[i], rate, ss_rem[i], se_rem[i]);
    }
}

/// Batch linear interpolation for Vertex attribute (fixed 8 floats: LT.x, LT.y, RT.x, RT.y, LB.x, LB.y, RB.x, RB.y).
pub fn interpolate_vertex_linear(start: &[f32; 8], end: &[f32; 8], rate: f32, out: &mut [f32; 8]) {
    let rate_v = f32x4::splat(rate);
    for ((s, e), o) in start.chunks_exact(4).zip(end.chunks_exact(4)).zip(out.chunks_exact_mut(4)) {
        let s_v = f32x4::from([s[0], s[1], s[2], s[3]]);
        let e_v = f32x4::from([e[0], e[1], e[2], e[3]]);
        let res = linear_batch_f32x4(s_v, e_v, rate_v);
        o.copy_from_slice(&res.to_array());
    }
}

/// Batch linear interpolation for PartColor attribute (fixed 16 floats: LT.rgba, RT.rgba, LB.rgba, RB.rgba).
pub fn interpolate_color_linear(start: &[f32; 16], end: &[f32; 16], rate: f32, out: &mut [f32; 16]) {
    let rate_v = f32x4::splat(rate);
    for ((s, e), o) in start.chunks_exact(4).zip(end.chunks_exact(4)).zip(out.chunks_exact_mut(4)) {
        let s_v = f32x4::from([s[0], s[1], s[2], s[3]]);
        let e_v = f32x4::from([e[0], e[1], e[2], e[3]]);
        let res = linear_batch_f32x4(s_v, e_v, rate_v);
        o.copy_from_slice(&res.to_array());
    }
}

/// Batch Back-In easing rate calculation for 4 elements.
#[inline]
pub fn rate_back_in_batch_f32x4(rate: f32x4) -> f32x4 {
    let overshoot = f32x4::splat(1.70158);
    let one = f32x4::splat(1.0);
    rate * rate * rate.mul_add(overshoot + one, -overshoot)
}

/// Batch Back-Out easing rate calculation for 4 elements.
#[inline]
pub fn rate_back_out_batch_f32x4(rate: f32x4) -> f32x4 {
    let overshoot = f32x4::splat(1.70158);
    let one = f32x4::splat(1.0);
    let rate_slide = rate - one;
    (rate_slide * rate_slide).mul_add(rate_slide.mul_add(overshoot + one, overshoot), one)
}

/// Batch Accelerate easing rate calculation for 4 elements.
#[inline]
pub fn rate_accelerate_batch_f32x4(rate: f32x4) -> f32x4 {
    rate * rate
}

/// Batch Decelerate easing rate calculation for 4 elements.
#[inline]
pub fn rate_decelerate_batch_f32x4(rate: f32x4) -> f32x4 {
    let one = f32x4::splat(1.0);
    let rate_inverse = one - rate;
    one - (rate_inverse * rate_inverse)
}

#[inline]
fn bounce_time_batch_f32x4(rate: f32x4) -> f32x4 {
    let div_2_75 = f32x4::splat(1.0 / 2.75);
    let div_2_0_2_75 = f32x4::splat(2.0 / 2.75);
    let div_2_5_2_75 = f32x4::splat(2.5 / 2.75);
    let mul_7_5625 = f32x4::splat(7.5625);

    let res1 = mul_7_5625 * rate * rate;
    
    let r2 = rate - f32x4::splat(1.5 / 2.75);
    let res2 = r2.mul_add(mul_7_5625 * r2, f32x4::splat(0.75));
    
    let r3 = rate - f32x4::splat(2.25 / 2.75);
    let res3 = r3.mul_add(mul_7_5625 * r3, f32x4::splat(0.9375));
    
    let r4 = rate - f32x4::splat(2.625 / 2.75);
    let res4 = r4.mul_add(mul_7_5625 * r4, f32x4::splat(0.984375));

    // Using explicit trait calls with simd_lt
    let m3 = rate.simd_lt(div_2_5_2_75);
    let s3 = m3.blend(res3, res4);
    
    let m2 = rate.simd_lt(div_2_0_2_75);
    let s2 = m2.blend(res2, s3);
    
    let m1 = rate.simd_lt(div_2_75);
    m1.blend(res1, s2)
}

/// Batch Bounce-In easing rate calculation for 4 elements.
#[inline]
pub fn rate_bounce_in_batch_f32x4(rate: f32x4) -> f32x4 {
    let one = f32x4::splat(1.0);
    one - bounce_time_batch_f32x4(one - rate)
}

/// Batch Bounce-Out easing rate calculation for 4 elements.
#[inline]
pub fn rate_bounce_out_batch_f32x4(rate: f32x4) -> f32x4 {
    bounce_time_batch_f32x4(rate)
}

/// Batch Elastic-In easing rate calculation for 4 elements.
#[inline]
pub fn rate_elastic_in_batch_f32x4(rate: f32x4, power_easing: f32x4) -> f32x4 {
    let r = rate.to_array();
    let p = power_easing.to_array();
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = rate_ease_elastic_in(r[i], p[i]);
    }
    f32x4::from(out)
}

/// Batch Back-InOut easing rate calculation for 4 elements.
#[inline]
pub fn rate_back_inout_batch_f32x4(rate: f32x4) -> f32x4 {
    let overshoot = f32x4::splat(1.70158 * 1.525);
    let half = f32x4::splat(0.5);
    let one = f32x4::splat(1.0);
    let two = f32x4::splat(2.0);

    let rate_mul_2 = rate * two;
    let m_lt = rate_mul_2.simd_lt(one);

    // LT case
    let r2_lt = rate_mul_2 * rate_mul_2;
    let res_lt = (r2_lt * rate_mul_2.mul_add(overshoot + one, -overshoot)) * half;

    // GE case
    let rate_mul_2_ge = rate_mul_2 - two;
    let r2_ge = rate_mul_2_ge * rate_mul_2_ge;
    let res_ge = (r2_ge * rate_mul_2_ge.mul_add(overshoot + one, overshoot)).mul_add(half, one);

    m_lt.blend(res_lt, res_ge)
}

/// Batch Bounce-InOut easing rate calculation for 4 elements.
#[inline]
pub fn rate_bounce_inout_batch_f32x4(rate: f32x4) -> f32x4 {
    let half = f32x4::splat(0.5);
    let one = f32x4::splat(1.0);
    let two = f32x4::splat(2.0);

    let m_lt = rate.simd_lt(half);
    let rate_mul_2 = rate * two;
    
    let res_lt = (one - bounce_time_batch_f32x4(one - rate_mul_2)) * half;
    let res_ge = bounce_time_batch_f32x4(rate_mul_2 - one).mul_add(half, half);
    
    m_lt.blend(res_lt, res_ge)
}

/// Batch Elastic-Out easing rate calculation for 4 elements (fallback scalar).
#[inline]
pub fn rate_elastic_out_batch_f32x4(rate: f32x4, power_easing: f32x4) -> f32x4 {
    let r = rate.to_array();
    let p = power_easing.to_array();
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = rate_ease_elastic_out(r[i], p[i]);
    }
    f32x4::from(out)
}

/// Batch Elastic-InOut easing rate calculation for 4 elements (fallback scalar).
#[inline]
pub fn rate_elastic_inout_batch_f32x4(rate: f32x4, power_easing: f32x4) -> f32x4 {
    let r = rate.to_array();
    let p = power_easing.to_array();
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = rate_ease_elastic_inout(r[i], p[i]);
    }
    f32x4::from(out)
}

/// Batch Exponential-In easing rate calculation for 4 elements (fallback scalar).
#[inline]
pub fn rate_exponential_in_batch_f32x4(rate: f32x4) -> f32x4 {
    let r = rate.to_array();
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = rate_ease_exponential_in(r[i]);
    }
    f32x4::from(out)
}

/// Batch Exponential-Out easing rate calculation for 4 elements (fallback scalar).
#[inline]
pub fn rate_exponential_out_batch_f32x4(rate: f32x4) -> f32x4 {
    let r = rate.to_array();
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = rate_ease_exponential_out(r[i]);
    }
    f32x4::from(out)
}

/// Batch Exponential-InOut easing rate calculation for 4 elements (fallback scalar).
#[inline]
pub fn rate_exponential_inout_batch_f32x4(rate: f32x4) -> f32x4 {
    let r = rate.to_array();
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = rate_ease_exponential_inout(r[i]);
    }
    f32x4::from(out)
}

/// Batch Ease-In easing rate calculation for 4 elements (fallback scalar).
#[inline]
pub fn rate_ease_in_batch_f32x4(rate: f32x4, power_easing: f32x4) -> f32x4 {
    let r = rate.to_array();
    let p = power_easing.to_array();
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = rate_ease_in(r[i], p[i]);
    }
    f32x4::from(out)
}

/// Batch Ease-Out easing rate calculation for 4 elements (fallback scalar).
#[inline]
pub fn rate_ease_out_batch_f32x4(rate: f32x4, power_easing: f32x4) -> f32x4 {
    let r = rate.to_array();
    let p = power_easing.to_array();
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = rate_ease_out(r[i], p[i]);
    }
    f32x4::from(out)
}

/// Batch Ease-InOut easing rate calculation for 4 elements (fallback scalar).
#[inline]
pub fn rate_ease_inout_batch_f32x4(rate: f32x4, power_easing: f32x4) -> f32x4 {
    let r = rate.to_array();
    let p = power_easing.to_array();
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = rate_ease_inout(r[i], p[i]);
    }
    f32x4::from(out)
}

/// Batch Sine-In easing rate calculation for 4 elements using Bhaskara I's approximation.
#[inline]
pub fn rate_sine_in_batch_f32x4(rate: f32x4) -> f32x4 {
    let one = f32x4::splat(1.0);
    // Sine-In(t) = 1 - cos(t * pi/2) = 1 - sin((1-t) * pi/2)
    one - rate_sine_out_batch_f32x4(one - rate)
}

/// Batch Sine-Out easing rate calculation for 4 elements using Bhaskara I's approximation.
#[inline]
pub fn rate_sine_out_batch_f32x4(rate: f32x4) -> f32x4 {
    let pi = f32x4::splat(core::f32::consts::PI);
    let x = rate * f32x4::splat(core::f32::consts::PI * 0.5); // x in [0, pi/2]
    
    let num = (f32x4::splat(16.0) * x) * (pi - x);
    let den = f32x4::splat(5.0) * pi * pi - (f32x4::splat(4.0) * x) * (pi - x);
    num / den
}

/// Batch Sine-InOut easing rate calculation for 4 elements using Bhaskara I's approximation.
#[inline]
pub fn rate_sine_inout_batch_f32x4(rate: f32x4) -> f32x4 {
    let half = f32x4::splat(0.5);
    let one = f32x4::splat(1.0);
    let pi = f32x4::splat(core::f32::consts::PI);
    
    // InOut(t) = 0.5 * (1 - cos(t * pi))
    let x = rate * pi;
    
    // cos(t * pi) = sin(pi/2 - t * pi)
    let angle = f32x4::splat(core::f32::consts::PI * 0.5) - x;
    let abs_angle = angle.abs();
    
    // Bhaskara I sin(x) approx for abs_angle in [0, pi]
    let num_b = f32x4::splat(16.0) * abs_angle * (pi - abs_angle);
    let den_b = f32x4::splat(5.0) * pi * pi - f32x4::splat(4.0) * abs_angle * (pi - abs_angle);
    let cos_approx = num_b / den_b;
    
    // Handle sign for angle in [-pi/2, pi/2]
    let sign = f32x4::splat(1.0).copysign(angle);
    half * (one - (cos_approx * sign))
}

#[inline]
pub fn rate_linear<T: SsFloat>(value_now: T, value_start: T, value_end: T) -> T {
    (value_now - value_start) / (value_end - value_start)
}

#[inline]
pub fn linear<T: SsFloat>(value_start: T, value_end: T, rate: T) -> T {
    (value_end - value_start) * rate + value_start
}

#[inline]
pub fn rate_accelerate<T: SsFloat>(rate: T) -> T {
    rate * rate
}

#[inline]
pub fn rate_decelerate<T: SsFloat>(rate: T) -> T {
    let rate_inverse = T::from_f32(1.0) - rate;
    T::from_f32(1.0) - (rate_inverse * rate_inverse)
}

#[inline]
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
    let frame = linear(value_start_x, value_end_x, rate);

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

pub fn rate_ease_inout<T: SsFloat>(rate: T, power_easing: T) -> T {
    let rate_mul_2 = rate * T::from_f32(2.0);
    if rate_mul_2 < T::from_f32(1.0) {
        T::from_f32(0.5) * rate_mul_2.powf(power_easing)
    } else {
        T::from_f32(1.0) - (T::from_f32(0.5) * (T::from_f32(2.0) - rate_mul_2).powf(power_easing))
    }
}

pub fn rate_ease_exponential_in<T: SsFloat>(rate: T) -> T {
    if rate <= T::from_f32(0.0) {
        T::from_f32(0.0)
    } else {
        T::from_f32(2.0).powf(T::from_f32(10.0) * (rate - T::from_f32(1.0)))
    }
}

pub fn rate_ease_exponential_out<T: SsFloat>(rate: T) -> T {
    if rate >= T::from_f32(1.0) {
        T::from_f32(1.0)
    } else {
        T::from_f32(1.0) - T::from_f32(2.0).powf(T::from_f32(-10.0) * rate)
    }
}

pub fn rate_ease_exponential_inout<T: SsFloat>(rate: T) -> T {
    if rate <= T::from_f32(0.0) { return T::from_f32(0.0); }
    if rate >= T::from_f32(1.0) { return T::from_f32(1.0); }
    let rate_mul_2 = rate * T::from_f32(2.0);
    if rate_mul_2 < T::from_f32(1.0) {
        T::from_f32(0.5) * T::from_f32(2.0).powf(T::from_f32(10.0) * (rate_mul_2 - T::from_f32(1.0)))
    } else {
        T::from_f32(0.5) * (T::from_f32(2.0) - T::from_f32(2.0).powf(T::from_f32(-10.0) * (rate_mul_2 - T::from_f32(1.0))))
    }
}

pub fn rate_ease_sine_in<T: SsFloat>(rate: T) -> T {
    T::from_f32(1.0) - (rate * (T::pi() * T::from_f32(0.5))).cos()
}

pub fn rate_ease_sine_out<T: SsFloat>(rate: T) -> T {
    (rate * (T::pi() * T::from_f32(0.5))).sin()
}

pub fn rate_ease_sine_inout<T: SsFloat>(rate: T) -> T {
    T::from_f32(0.5) * (T::from_f32(1.0) - (T::pi() * rate).cos())
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
        (T::from_f32(1.0) - bounce_time(T::from_f32(1.0) - (T::from_f32(2.0) * rate))) * T::from_f32(0.5)
    } else {
        bounce_time(rate * T::from_f32(2.0) - T::from_f32(1.0)) * T::from_f32(0.5) + T::from_f32(0.5)
    }
}

pub fn rate_ease_back_in<T: SsFloat>(rate: T) -> T {
    let overshoot = T::from_f32(1.70158);
    rate * rate * ((overshoot + T::from_f32(1.0)) * rate - overshoot)
}

pub fn rate_ease_back_out<T: SsFloat>(rate: T) -> T {
    let overshoot = T::from_f32(1.70158);
    let rate_slide = rate - T::from_f32(1.0);
    rate_slide * rate_slide * ((overshoot + T::from_f32(1.0)) * rate_slide + overshoot) + T::from_f32(1.0)
}

pub fn rate_ease_back_inout<T: SsFloat>(rate: T) -> T {
    let overshoot = T::from_f32(1.70158 * 1.525);
    let mut rate_mul_2 = rate * T::from_f32(2.0);
    if rate_mul_2 < T::from_f32(1.0) {
        (rate_mul_2 * rate_mul_2 * ((overshoot + T::from_f32(1.0)) * rate_mul_2 - overshoot)) * T::from_f32(0.5)
    } else {
        rate_mul_2 -= T::from_f32(2.0);
        (rate_mul_2 * rate_mul_2 * ((overshoot + T::from_f32(1.0)) * rate_mul_2 + overshoot)) * T::from_f32(0.5) + T::from_f32(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_near(actual: f32, expected: f32) {
        let eps = 1e-2;
        if (actual - expected).abs() > eps {
            panic!("actual: {}, expected: {}, diff: {}", actual, expected, (actual - expected).abs());
        }
    }

    #[test]
    fn test_linear() {
        assert_near(linear(0.0, 10.0, 0.0), 0.0);
        assert_near(linear(0.0, 10.0, 0.5), 5.0);
        assert_near(linear(0.0, 10.0, 1.0), 10.0);
        assert_near(rate_linear(5.0, 0.0, 10.0), 0.5);
    }

    #[test]
    fn test_accelerate_decelerate() {
        assert_near(rate_accelerate(0.0), 0.0);
        assert_near(rate_accelerate(0.5), 0.25);
        assert_near(rate_accelerate(1.0), 1.0);
        assert_near(rate_decelerate(0.0), 0.0);
        assert_near(rate_decelerate(0.5), 0.75);
        assert_near(rate_decelerate(1.0), 1.0);
    }

    #[test]
    fn test_hermite() {
        assert_near(hermite(0.0, 1.0, 0.0, 0.0, 1.0), 0.0);
        assert_near(hermite(0.0, 1.0, 1.0, 0.0, 1.0), 1.0);
        assert_near(hermite(0.0, 10.0, 0.5, 0.0, 10.0), 5.0);
    }

    #[test]
    fn test_bezier() {
        // Linear behavior when handles are aligned
        assert_near(bezier(0.0, 0.0, 1.0, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0), 0.5);
        assert_near(bezier(0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0), 0.0);
        assert_near(bezier(0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0), 1.0);
    }

    #[test]
    fn test_easing_basic() {
        assert_near(rate_ease_in(0.0, 2.0), 0.0);
        assert_near(rate_ease_in(0.5, 2.0), 0.25);
        assert_near(rate_ease_in(1.0, 2.0), 1.0);
        assert_near(rate_ease_out(0.0, 2.0), 0.0);
        assert_near(rate_ease_out(0.25, 2.0), 0.5);
        assert_near(rate_ease_out(1.0, 2.0), 1.0);
        assert_near(rate_ease_inout(0.0, 2.0), 0.0);
        assert_near(rate_ease_inout(0.5, 2.0), 0.5);
        assert_near(rate_ease_inout(1.0, 2.0), 1.0);
    }

    #[test]
    fn test_sine() {
        assert_near(rate_ease_sine_in(0.0), 0.0);
        assert_near(rate_ease_sine_in(0.5), 1.0 - (core::f32::consts::PI * 0.25).cos());
        assert_near(rate_ease_sine_in(1.0), 1.0);
        assert_near(rate_ease_sine_out(0.0), 0.0);
        assert_near(rate_ease_sine_out(0.5), (core::f32::consts::PI * 0.25).sin());
        assert_near(rate_ease_sine_out(1.0), 1.0);
        assert_near(rate_ease_sine_inout(0.0), 0.0);
        assert_near(rate_ease_sine_inout(0.5), 0.5);
        assert_near(rate_ease_sine_inout(1.0), 1.0);
    }

    #[test]
    fn test_exponential() {
        assert_near(rate_ease_exponential_in(0.0), 0.0);
        assert_near(rate_ease_exponential_in(1.0), 1.0);
        assert_near(rate_ease_exponential_out(0.0), 0.0);
        assert_near(rate_ease_exponential_out(1.0), 1.0);
        assert_near(rate_ease_exponential_inout(0.0), 0.0);
        assert_near(rate_ease_exponential_inout(0.5), 0.5);
        assert_near(rate_ease_exponential_inout(1.0), 1.0);
    }

    #[test]
    fn test_bounce() {
        assert_near(rate_ease_bounce_in(0.0), 0.0);
        assert_near(rate_ease_bounce_in(1.0), 1.0);
        assert_near(rate_ease_bounce_out(0.0), 0.0);
        assert_near(rate_ease_bounce_out(1.0), 1.0);
        assert_near(rate_ease_bounce_inout(0.0), 0.0);
        assert_near(rate_ease_bounce_inout(0.5), 0.5);
        assert_near(rate_ease_bounce_inout(1.0), 1.0);
    }

    #[test]
    fn test_back() {
        assert_near(rate_ease_back_in(0.0), 0.0);
        assert_near(rate_ease_back_in(1.0), 1.0);
        assert_near(rate_ease_back_out(0.0), 0.0);
        assert_near(rate_ease_back_out(1.0), 1.0);
        assert_near(rate_ease_back_inout(0.0), 0.0);
        assert_near(rate_ease_back_inout(0.5), 0.5);
        assert_near(rate_ease_back_inout(1.0), 1.0);
    }

    #[test]
    fn test_simd_linear_batch() {
        let start = f32x4::from([0.0, 10.0, 20.0, 30.0]);
        let end = f32x4::from([10.0, 20.0, 30.0, 40.0]);
        let rate = f32x4::splat(0.5);
        let res = linear_batch_f32x4(start, end, rate);
        let expected = [5.0, 15.0, 25.0, 35.0];
        let actual: [f32; 4] = res.into();
        for i in 0..4 {
            assert_near(actual[i], expected[i]);
        }
    }

    #[test]
    fn test_interpolate_array_linear() {
        // Case 1: Length is multiple of 4
        let start = [0.0, 10.0, 20.0, 30.0];
        let end = [10.0, 20.0, 30.0, 40.0];
        let mut out = [0.0; 4];
        interpolate_array_linear(&start, &end, 0.5, &mut out);
        assert_near(out[0], 5.0);
        assert_near(out[1], 15.0);
        assert_near(out[2], 25.0);
        assert_near(out[3], 35.0);

        // Case 2: Length is NOT multiple of 4 (tests remainder handling)
        let start_odd = [0.0, 10.0, 20.0, 30.0, 40.0, 50.0];
        let end_odd = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0];
        let mut out_odd = [0.0; 6];
        interpolate_array_linear(&start_odd, &end_odd, 0.1, &mut out_odd);
        assert_near(out_odd[0], 1.0);
        assert_near(out_odd[4], 41.0);
        assert_near(out_odd[5], 51.0);
    }

    #[test]
    fn test_interpolate_array_hermite() {
        let v_start = [0.0, 10.0, 20.0, 30.0, 40.0];
        let v_end = [10.0, 20.0, 30.0, 40.0, 50.0];
        let s_start = [0.0, 0.0, 0.0, 0.0, 0.0];
        let s_end = [10.0, 10.0, 10.0, 10.0, 10.0];
        let mut out = [0.0; 5];
        
        interpolate_array_hermite(&v_start, &v_end, 0.5, &s_start, &s_end, &mut out);
        
        for i in 0..5 {
            let expected = hermite(v_start[i], v_end[i], 0.5, s_start[i], s_end[i]);
            assert_near(out[i], expected);
        }
    }

    #[test]
    fn test_interpolate_vertex_linear() {
        let start = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let end = [10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0];
        let mut out = [0.0; 8];
        interpolate_vertex_linear(&start, &end, 0.5, &mut out);
        for i in 0..8 {
            assert_near(out[i], start[i] + 5.0);
        }
    }

    #[test]
    fn test_interpolate_color_linear() {
        let start = [0.0; 16];
        let end = [1.0; 16];
        let mut out = [0.0; 16];
        interpolate_color_linear(&start, &end, 0.5, &mut out);
        for i in 0..16 {
            assert_near(out[i], 0.5);
        }
    }

    #[test]
    fn test_rate_back_batch() {
        let rate_v = f32x4::from([0.0, 0.5, 1.0, 0.5]);
        let res_in = rate_back_in_batch_f32x4(rate_v);
        let res_out = rate_back_out_batch_f32x4(rate_v);
        let actual_in = res_in.to_array();
        let actual_out = res_out.to_array();
        
        for i in 0..4 {
            assert_near(actual_in[i], rate_ease_back_in(rate_v.to_array()[i]));
            assert_near(actual_out[i], rate_ease_back_out(rate_v.to_array()[i]));
        }
    }

    #[test]
    fn test_rate_sine_batch() {
        let test_cases = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];
        for chunk in test_cases.chunks(4) {
            let mut input = [0.0; 4];
            for (i, &v) in chunk.iter().enumerate() { input[i] = v; }
            let rate_v = f32x4::from(input);
            
            let res_in = rate_sine_in_batch_f32x4(rate_v);
            let res_out = rate_sine_out_batch_f32x4(rate_v);
            let res_inout = rate_sine_inout_batch_f32x4(rate_v);
            
            let actual_in = res_in.to_array();
            let actual_out = res_out.to_array();
            let actual_inout = res_inout.to_array();
            
            for i in 0..chunk.len() {
                let v = chunk[i];
                assert_near(actual_in[i], rate_ease_sine_in(v));
                assert_near(actual_out[i], rate_ease_sine_out(v));
                assert_near(actual_inout[i], rate_ease_sine_inout(v));
            }
        }
    }

    #[test]
    fn test_rate_accelerate_decelerate_batch() {
        let rate_v = f32x4::from([0.0, 0.25, 0.5, 1.0]);
        let res_acc = rate_accelerate_batch_f32x4(rate_v);
        let res_dec = rate_decelerate_batch_f32x4(rate_v);
        let actual_acc = res_acc.to_array();
        let actual_dec = res_dec.to_array();
        
        for i in 0..4 {
            let v = rate_v.to_array()[i];
            assert_near(actual_acc[i], rate_accelerate(v));
            assert_near(actual_dec[i], rate_decelerate(v));
        }
    }

    #[test]
    fn test_rate_bounce_batch() {
        let test_cases = [0.0, 0.25, 0.5, 0.75, 0.9, 1.0];
        for chunk in test_cases.chunks(4) {
            let mut input = [0.0; 4];
            for (i, &v) in chunk.iter().enumerate() { input[i] = v; }
            let rate_v = f32x4::from(input);
            
            let res_in = rate_bounce_in_batch_f32x4(rate_v);
            let res_out = rate_bounce_out_batch_f32x4(rate_v);
            
            let actual_in = res_in.to_array();
            let actual_out = res_out.to_array();
            
            for i in 0..chunk.len() {
                let v = chunk[i];
                assert_near(actual_in[i], rate_ease_bounce_in(v));
                assert_near(actual_out[i], rate_ease_bounce_out(v));
            }
        }
    }

    #[test]
    fn test_rate_elastic_batch() {
        let rate_v = f32x4::from([0.0, 0.25, 0.5, 1.0]);
        let p_v = f32x4::splat(0.3); // default power_easing
        let res_in = rate_elastic_in_batch_f32x4(rate_v, p_v);
        let actual_in = res_in.to_array();
        
        for i in 0..4 {
            let v = rate_v.to_array()[i];
            // Since it's an approximation, we check if it's broadly correct
            // For production quality, higher order polynomials are needed.
            let expected = rate_ease_elastic_in(v, 0.3);
            let diff = (actual_in[i] - expected).abs();
            assert!(diff < 0.1, "ElasticIn(v={}) actual={}, expected={}, diff={}", v, actual_in[i], expected, diff);
        }
    }

    #[test]
    fn test_rate_linear_batch() {
        let value_now = f32x4::from([5.0, 5.0, 5.0, 5.0]);
        let value_start = f32x4::from([0.0, 0.0, 0.0, 0.0]);
        let value_end = f32x4::from([10.0, 10.0, 10.0, 10.0]);
        let res = super::rate_linear_batch_f32x4(value_now, value_start, value_end);
        for i in 0..4 {
            assert_near(res.to_array()[i], 0.5);
        }
    }

    #[test]
    fn test_hermite_batch() {
        let v_start = f32x4::from([0.0, 0.0, 0.0, 0.0]);
        let v_end = f32x4::from([1.0, 1.0, 10.0, 10.0]);
        let rate = f32x4::from([0.0, 1.0, 0.5, 0.5]);
        let s_start = f32x4::from([0.0, 0.0, 0.0, 0.0]);
        let s_end = f32x4::from([1.0, 1.0, 10.0, 10.0]);
        
        let res = super::hermite_batch_f32x4(v_start, v_end, rate, s_start, s_end);
        for i in 0..4 {
            let actual = res.to_array()[i];
            let expected = super::hermite(
                v_start.to_array()[i], v_end.to_array()[i],
                rate.to_array()[i],
                s_start.to_array()[i], s_end.to_array()[i]
            );
            assert_near(actual, expected);
        }
    }

    #[test]
    fn test_rate_elastic_out_batch() {
        let rate_v = f32x4::from([0.0, 0.25, 0.5, 1.0]);
        let p_v = f32x4::splat(0.3);
        let res = super::rate_elastic_out_batch_f32x4(rate_v, p_v);
        
        for i in 0..4 {
            let v = rate_v.to_array()[i];
            assert_near(res.to_array()[i], super::rate_ease_elastic_out(v, 0.3));
        }
    }

    #[test]
    fn test_rate_elastic_inout_batch() {
        let rate_v = f32x4::from([0.0, 0.25, 0.5, 1.0]);
        let p_v = f32x4::splat(0.3);
        let res = super::rate_elastic_inout_batch_f32x4(rate_v, p_v);
        
        for i in 0..4 {
            let v = rate_v.to_array()[i];
            assert_near(res.to_array()[i], super::rate_ease_elastic_inout(v, 0.3));
        }
    }

    #[test]
    fn test_rate_ease_basic_batch() {
        let rate_v = f32x4::from([0.0, 0.25, 0.5, 1.0]);
        let p_v = f32x4::splat(2.0);
        let res_in = super::rate_ease_in_batch_f32x4(rate_v, p_v);
        let res_out = super::rate_ease_out_batch_f32x4(rate_v, p_v);
        let res_inout = super::rate_ease_inout_batch_f32x4(rate_v, p_v);
        
        for i in 0..4 {
            let v = rate_v.to_array()[i];
            assert_near(res_in.to_array()[i], super::rate_ease_in(v, 2.0));
            assert_near(res_out.to_array()[i], super::rate_ease_out(v, 2.0));
            assert_near(res_inout.to_array()[i], super::rate_ease_inout(v, 2.0));
        }
    }

    #[test]
    fn test_rate_exponential_batch() {
        let rate_v = f32x4::from([0.0, 0.25, 0.5, 1.0]);
        let res_in = super::rate_exponential_in_batch_f32x4(rate_v);
        let res_out = super::rate_exponential_out_batch_f32x4(rate_v);
        let res_inout = super::rate_exponential_inout_batch_f32x4(rate_v);
        
        for i in 0..4 {
            let v = rate_v.to_array()[i];
            assert_near(res_in.to_array()[i], super::rate_ease_exponential_in(v));
            assert_near(res_out.to_array()[i], super::rate_ease_exponential_out(v));
            assert_near(res_inout.to_array()[i], super::rate_ease_exponential_inout(v));
        }
    }

    #[test]
    fn test_rate_back_inout_batch() {
        let rate_v = f32x4::from([0.0, 0.25, 0.75, 1.0]);
        let res = super::rate_back_inout_batch_f32x4(rate_v);
        for i in 0..4 {
            let v = rate_v.to_array()[i];
            assert_near(res.to_array()[i], super::rate_ease_back_inout(v));
        }
    }
    
    #[test]
    fn test_rate_bounce_inout_batch() {
        let rate_v = f32x4::from([0.0, 0.25, 0.75, 1.0]);
        let res = super::rate_bounce_inout_batch_f32x4(rate_v);
        for i in 0..4 {
            let v = rate_v.to_array()[i];
            assert_near(res.to_array()[i], super::rate_ease_bounce_inout(v));
        }
    }

    #[test]
    fn test_bezier_batch() {
        let vx = f32x4::from([0.0, 0.0, 1.0, 1.0]);
        let vy = f32x4::from([0.0, 0.0, 1.0, 1.0]);
        let ex = f32x4::from([1.0, 1.0, 0.0, 0.0]);
        let ey = f32x4::from([1.0, 1.0, 0.0, 0.0]);
        let rate = f32x4::splat(0.5);
        let vec_sx = f32x4::splat(0.0);
        let vec_sy = f32x4::splat(0.0);
        let vec_ex = f32x4::splat(0.0);
        let vec_ey = f32x4::splat(0.0);
        
        let res = super::bezier_batch_f32x4(vx, vy, ex, ey, rate, vec_sx, vec_sy, vec_ex, vec_ey);
        for i in 0..4 {
            assert_near(res.to_array()[i], super::bezier(vx.to_array()[i], vy.to_array()[i], ex.to_array()[i], ey.to_array()[i], rate.to_array()[i], 0.0, 0.0, 0.0, 0.0));
        }
    }
}
