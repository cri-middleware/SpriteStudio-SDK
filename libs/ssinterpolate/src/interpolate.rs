use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use wide::{f32x4, CmpEq, CmpGe, CmpGt, CmpLe, CmpLt};

pub trait SsMask<T> {
    fn blend(self, t: T, f: T) -> T;
}

impl SsMask<f32> for bool {
    #[inline]
    fn blend(self, t: f32, f: f32) -> f32 { if self { t } else { f } }
}

impl SsMask<f64> for bool {
    #[inline]
    fn blend(self, t: f64, f: f64) -> f64 { if self { t } else { f } }
}

impl SsMask<f32x4> for f32x4 {
    #[inline]
    fn blend(self, t: f32x4, f: f32x4) -> f32x4 {
        self.blend(t, f)
    }
}

/// A trait to abstract floating-point operations for generic interpolation.
pub trait SsFloat: Sized + Copy + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> + AddAssign + SubAssign + MulAssign + DivAssign {
    type Mask: SsMask<Self>;
    fn from_f32(v: f32) -> Self;
    fn powf(self, n: Self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn pi() -> Self;

    fn cmp_lt(self, other: Self) -> Self::Mask;
    fn cmp_le(self, other: Self) -> Self::Mask;
    fn cmp_gt(self, other: Self) -> Self::Mask;
    fn cmp_ge(self, other: Self) -> Self::Mask;
    fn cmp_eq(self, other: Self) -> Self::Mask;
}

impl SsFloat for f32 {
    type Mask = bool;
    #[inline] fn from_f32(v: f32) -> Self { v }
    #[inline] fn powf(self, n: Self) -> Self { libm::powf(self, n) }
    #[inline] fn sin(self) -> Self { libm::sinf(self) }
    #[inline] fn cos(self) -> Self { libm::cosf(self) }
    #[inline] fn pi() -> Self { core::f32::consts::PI }
    #[inline] fn cmp_lt(self, other: Self) -> Self::Mask { self < other }
    #[inline] fn cmp_le(self, other: Self) -> Self::Mask { self <= other }
    #[inline] fn cmp_gt(self, other: Self) -> Self::Mask { self > other }
    #[inline] fn cmp_ge(self, other: Self) -> Self::Mask { self >= other }
    #[inline] fn cmp_eq(self, other: Self) -> Self::Mask { self == other }
}

impl SsFloat for f64 {
    type Mask = bool;
    #[inline] fn from_f32(v: f32) -> Self { v as f64 }
    #[inline] fn powf(self, n: Self) -> Self { libm::pow(self, n) }
    #[inline] fn sin(self) -> Self { libm::sin(self) }
    #[inline] fn cos(self) -> Self { libm::cos(self) }
    #[inline] fn pi() -> Self { core::f64::consts::PI }
    #[inline] fn cmp_lt(self, other: Self) -> Self::Mask { self < other }
    #[inline] fn cmp_le(self, other: Self) -> Self::Mask { self <= other }
    #[inline] fn cmp_gt(self, other: Self) -> Self::Mask { self > other }
    #[inline] fn cmp_ge(self, other: Self) -> Self::Mask { self >= other }
    #[inline] fn cmp_eq(self, other: Self) -> Self::Mask { self == other }
}

impl SsFloat for f32x4 {
    type Mask = f32x4;
    #[inline] fn from_f32(v: f32) -> Self { f32x4::splat(v) }
    #[inline] fn powf(self, n: Self) -> Self { self.pow_f32x4(n) }
    #[inline] fn sin(self) -> Self { self.sin() }
    #[inline] fn cos(self) -> Self { self.cos() }
    #[inline] fn pi() -> Self { f32x4::splat(core::f32::consts::PI) }
    #[inline] fn cmp_lt(self, other: Self) -> Self::Mask { self.simd_lt(other) }
    #[inline] fn cmp_le(self, other: Self) -> Self::Mask { self.simd_le(other) }
    #[inline] fn cmp_gt(self, other: Self) -> Self::Mask { self.simd_gt(other) }
    #[inline] fn cmp_ge(self, other: Self) -> Self::Mask { self.simd_ge(other) }
    #[inline] fn cmp_eq(self, other: Self) -> Self::Mask { self.simd_eq(other) }
}

// --- Core Generic Implementations ---

fn bounce_time<T: SsFloat>(rate: T) -> T {
    let div_2_75 = T::from_f32(1.0 / 2.75);
    let div_2_0_2_75 = T::from_f32(2.0 / 2.75);
    let div_2_5_2_75 = T::from_f32(2.5 / 2.75);

    let mul_7_5625 = T::from_f32(7.5625);

    let val1 = mul_7_5625 * rate * rate;

    let rate_modify2 = rate - T::from_f32(1.5 / 2.75);
    let val2 = mul_7_5625 * rate_modify2 * rate_modify2 + T::from_f32(0.75);

    let rate_modify3 = rate - T::from_f32(2.25 / 2.75);
    let val3 = mul_7_5625 * rate_modify3 * rate_modify3 + T::from_f32(0.9375);

    let rate_modify4 = rate - T::from_f32(2.625 / 2.75);
    let val4 = mul_7_5625 * rate_modify4 * rate_modify4 + T::from_f32(0.984375);

    let cond1 = rate.cmp_lt(div_2_75);
    let cond2 = rate.cmp_lt(div_2_0_2_75);
    let cond3 = rate.cmp_lt(div_2_5_2_75);

    cond1.blend(val1, cond2.blend(val2, cond3.blend(val3, val4)))
}

#[inline]
fn linear_batch_f32x4(start: f32x4, end: f32x4, rate: f32x4) -> f32x4 {
    (end - start).mul_add(rate, start)
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

    (((two * rate_pow3) - (three * rate_pow2) + one) * value_start) + (((three * rate_pow2) - (two * rate_pow3)) * value_end) + ((rate_pow3 - (two * rate_pow2) + rate) * (speed_start - value_start)) + ((rate_pow3 - rate_pow2) * (speed_end - value_end))
}

#[allow(clippy::too_many_arguments)]
pub fn bezier<T: SsFloat>(value_start_x: T, value_start_y: T, value_end_x: T, value_end_y: T, rate: T, vector_start_x: T, vector_start_y: T, vector_end_x: T, vector_end_y: T) -> T {
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

        let frame_temp = (base_now_pow3 * value_start_x) + (three * base_now_pow2 * area_now * (vector_start_x + value_start_x)) + (three * base_now * area_now_pow2 * (vector_end_x + value_end_x)) + (area_now * area_now_pow2 * value_end_x);

        range_now *= half;
        area_now = frame_temp.cmp_gt(frame).blend(area_now - range_now, area_now + range_now);
    }

    area_now_pow2 = area_now * area_now;
    base_now = one - area_now;
    base_now_pow2 = base_now * base_now;
    base_now_pow3 = base_now_pow2 * base_now;

    (base_now_pow3 * value_start_y) + (three * base_now_pow2 * area_now * (vector_start_y + value_start_y)) + (three * base_now * area_now_pow2 * (vector_end_y + value_end_y)) + (area_now * area_now_pow2 * value_end_y)
}

pub fn rate_ease_in<T: SsFloat>(rate: T, power_easing: T) -> T {
    rate.powf(power_easing)
}

pub fn rate_ease_out<T: SsFloat>(rate: T, power_easing: T) -> T {
    rate.powf(T::from_f32(1.0) / power_easing)
}

pub fn rate_ease_inout<T: SsFloat>(rate: T, power_easing: T) -> T {
    let rate_mul_2 = rate * T::from_f32(2.0);
    let val1 = T::from_f32(0.5) * rate_mul_2.powf(power_easing);
    let val2 = T::from_f32(1.0) - (T::from_f32(0.5) * (T::from_f32(2.0) - rate_mul_2).powf(power_easing));
    rate_mul_2.cmp_lt(T::from_f32(1.0)).blend(val1, val2)
}

pub fn rate_ease_exponential_in<T: SsFloat>(rate: T) -> T {
    let val1 = T::from_f32(0.0);
    let val2 = T::from_f32(2.0).powf(T::from_f32(10.0) * (rate - T::from_f32(1.0)));
    rate.cmp_le(T::from_f32(0.0)).blend(val1, val2)
}

pub fn rate_ease_exponential_out<T: SsFloat>(rate: T) -> T {
    let val1 = T::from_f32(1.0);
    let val2 = T::from_f32(1.0) - T::from_f32(2.0).powf(T::from_f32(-10.0) * rate);
    rate.cmp_ge(T::from_f32(1.0)).blend(val1, val2)
}

pub fn rate_ease_exponential_inout<T: SsFloat>(rate: T) -> T {
    let zero = T::from_f32(0.0);
    let one = T::from_f32(1.0);
    let rate_mul_2 = rate * T::from_f32(2.0);

    let cond_le_0 = rate.cmp_le(zero);
    let cond_ge_1 = rate.cmp_ge(one);
    let cond_lt_1_mul = rate_mul_2.cmp_lt(one);

    let val_lt_1 = T::from_f32(0.5) * T::from_f32(2.0).powf(T::from_f32(10.0) * (rate_mul_2 - one));
    let val_ge_1 = T::from_f32(0.5) * (T::from_f32(2.0) - T::from_f32(2.0).powf(T::from_f32(-10.0) * (rate_mul_2 - one)));

    let inner_blend = cond_lt_1_mul.blend(val_lt_1, val_ge_1);

    cond_le_0.blend(zero, cond_ge_1.blend(one, inner_blend))
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

    let s = power_easing * T::from_f32(0.25);
    let rate_slide = rate - one;
    let val = T::from_f32(-1.0) * T::from_f32(2.0).powf(T::from_f32(10.0) * rate_slide) * ((rate_slide - s) * (T::pi() * T::from_f32(2.0)) / power_easing).sin();

    let cond1 = rate.cmp_le(zero);
    let cond2 = rate.cmp_ge(one);
    let cond_out = cond1.blend(T::from_f32(1.0), cond2.blend(T::from_f32(1.0), T::from_f32(0.0))).cmp_eq(T::from_f32(1.0));

    cond_out.blend(rate, val)
}

pub fn rate_ease_elastic_out<T: SsFloat>(rate: T, power_easing: T) -> T {
    let zero = T::from_f32(0.0);
    let one = T::from_f32(1.0);

    let s = power_easing * T::from_f32(0.25);
    let val = T::from_f32(2.0).powf(T::from_f32(-10.0) * rate) * ((rate - s) * (T::pi() * T::from_f32(2.0)) / power_easing).sin() + one;

    let cond1 = rate.cmp_le(zero);
    let cond2 = rate.cmp_ge(one);
    let cond_out = cond1.blend(T::from_f32(1.0), cond2.blend(T::from_f32(1.0), T::from_f32(0.0))).cmp_eq(T::from_f32(1.0));

    cond_out.blend(rate, val)
}

pub fn rate_ease_elastic_inout<T: SsFloat>(rate: T, power_easing: T) -> T {
    let zero = T::from_f32(0.0);
    let one = T::from_f32(1.0);

    let rate_mul_2 = rate * T::from_f32(2.0) - one;
    let power_easing_modify = power_easing.cmp_eq(zero).blend(T::from_f32(0.3 * 1.5), power_easing);

    let s = power_easing_modify * T::from_f32(0.25);

    let val1 = T::from_f32(-0.5) * T::from_f32(2.0).powf(T::from_f32(10.0) * rate_mul_2) * ((rate - s) * (T::pi() * T::from_f32(2.0)) / power_easing_modify).sin();
    let val2 = T::from_f32(2.0).powf(T::from_f32(-10.0) * rate_mul_2) * ((rate_mul_2 - s) * (T::pi() * T::from_f32(2.0)) / power_easing_modify).sin() * T::from_f32(0.5) + one;

    let val = rate_mul_2.cmp_lt(zero).blend(val1, val2);

    let cond1 = rate.cmp_le(zero);
    let cond2 = rate.cmp_ge(one);
    let cond_out = cond1.blend(T::from_f32(1.0), cond2.blend(T::from_f32(1.0), T::from_f32(0.0))).cmp_eq(T::from_f32(1.0));

    cond_out.blend(rate, val)
}

pub fn rate_ease_bounce_in<T: SsFloat>(rate: T) -> T {
    T::from_f32(1.0) - bounce_time(T::from_f32(1.0) - rate)
}

pub fn rate_ease_bounce_out<T: SsFloat>(rate: T) -> T {
    bounce_time(rate)
}

pub fn rate_ease_bounce_inout<T: SsFloat>(rate: T) -> T {
    let val1 = (T::from_f32(1.0) - bounce_time(T::from_f32(1.0) - (T::from_f32(2.0) * rate))) * T::from_f32(0.5);
    let val2 = bounce_time(rate * T::from_f32(2.0) - T::from_f32(1.0)) * T::from_f32(0.5) + T::from_f32(0.5);
    rate.cmp_lt(T::from_f32(0.5)).blend(val1, val2)
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
    let rate_mul_2 = rate * T::from_f32(2.0);

    let cond = rate_mul_2.cmp_lt(T::from_f32(1.0));
    let val1 = (rate_mul_2 * rate_mul_2 * ((overshoot + T::from_f32(1.0)) * rate_mul_2 - overshoot)) * T::from_f32(0.5);

    let rate_mul_2_sub = rate_mul_2 - T::from_f32(2.0);
    let val2 = (rate_mul_2_sub * rate_mul_2_sub * ((overshoot + T::from_f32(1.0)) * rate_mul_2_sub + overshoot)) * T::from_f32(0.5) + T::from_f32(1.0);

    cond.blend(val1, val2)
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
        for val in out {
            assert_near(val, 0.5);
        }
    }
}

pub fn interpolate_array_bezier(
    start: &[f32],
    end: &[f32],
    frame_no: f32,
    prev_frame: f32,
    next_frame: f32,
    c_start_time: f32,
    c_start_value: f32,
    c_end_time: f32,
    c_end_value: f32,
    out: &mut [f32],
) {
    let rate = rate_linear(frame_no, prev_frame, next_frame);
    let rate_v = f32x4::splat(rate);
    let prev_f_v = f32x4::splat(prev_frame);
    let next_f_v = f32x4::splat(next_frame);
    let c_st_v = f32x4::splat(c_start_time);
    let c_sv_v = f32x4::splat(c_start_value);
    let c_et_v = f32x4::splat(c_end_time);
    let c_ev_v = f32x4::splat(c_end_value);

    let mut start_chunks = start.chunks_exact(4);
    let mut end_chunks = end.chunks_exact(4);
    let mut out_chunks = out.chunks_exact_mut(4);

    for ((s, e), o) in start_chunks.by_ref().zip(end_chunks.by_ref()).zip(out_chunks.by_ref()) {
        let s_v = f32x4::from([s[0], s[1], s[2], s[3]]);
        let e_v = f32x4::from([e[0], e[1], e[2], e[3]]);
        let res = bezier(prev_f_v, s_v, next_f_v, e_v, rate_v, c_st_v, c_sv_v, c_et_v, c_ev_v);
        o.copy_from_slice(&res.to_array());
    }

    let start_rem = start_chunks.remainder();
    let end_rem = end_chunks.remainder();
    let out_rem = out_chunks.into_remainder();

    for i in 0..start_rem.len() {
        out_rem[i] = bezier(prev_frame, start_rem[i], next_frame, end_rem[i], rate, c_start_time, c_start_value, c_end_time, c_end_value);
    }
}

pub fn interpolate_array_hermite(
    start: &[f32],
    end: &[f32],
    frame_no: f32,
    prev_frame: f32,
    next_frame: f32,
    c_start_value: f32,
    c_end_value: f32,
    out: &mut [f32],
) {
    let rate = rate_linear(frame_no, prev_frame, next_frame);
    let rate_v = f32x4::splat(rate);
    let c_sv_v = f32x4::splat(c_start_value);
    let c_ev_v = f32x4::splat(c_end_value);

    let mut start_chunks = start.chunks_exact(4);
    let mut end_chunks = end.chunks_exact(4);
    let mut out_chunks = out.chunks_exact_mut(4);

    for ((s, e), o) in start_chunks.by_ref().zip(end_chunks.by_ref()).zip(out_chunks.by_ref()) {
        let s_v = f32x4::from([s[0], s[1], s[2], s[3]]);
        let e_v = f32x4::from([e[0], e[1], e[2], e[3]]);
        let res = hermite(s_v, e_v, rate_v, c_sv_v, c_ev_v);
        o.copy_from_slice(&res.to_array());
    }

    let start_rem = start_chunks.remainder();
    let end_rem = end_chunks.remainder();
    let out_rem = out_chunks.into_remainder();

    for i in 0..start_rem.len() {
        out_rem[i] = hermite(start_rem[i], end_rem[i], rate, c_start_value, c_end_value);
    }
}
