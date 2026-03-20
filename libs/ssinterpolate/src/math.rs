use std::ops::{Add, Sub, Mul, Div};

/// A trait to abstract floating-point operations for generic interpolation.
pub trait Float:
    Sized
    + Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
    fn from_f32(v: f32) -> Self;
    fn powf(self, n: Self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn pi() -> Self;
}

impl Float for f32 {
    #[inline] fn from_f32(v: f32) -> Self { v }
    #[inline] fn powf(self, n: Self) -> Self { self.powf(n) }
    #[inline] fn sin(self) -> Self { self.sin() }
    #[inline] fn cos(self) -> Self { self.cos() }
    #[inline] fn pi() -> Self { std::f32::consts::PI }
}

impl Float for f64 {
    #[inline] fn from_f32(v: f32) -> Self { v as f64 }
    #[inline] fn powf(self, n: Self) -> Self { self.powf(n) }
    #[inline] fn sin(self) -> Self { self.sin() }
    #[inline] fn cos(self) -> Self { self.cos() }
    #[inline] fn pi() -> Self { std::f64::consts::PI }
}

// --- Generic Implementations ---

#[inline]
pub fn rate_linear<T: Float>(value_now: T, value_start: T, value_end: T) -> T {
    (value_now - value_start) / (value_end - value_start)
}

#[inline]
pub fn linear<T: Float>(value_start: T, value_end: T, rate: T) -> T {
    (value_end - value_start) * rate + value_start
}

#[inline]
pub fn hermite<T: Float>(value_start: T, value_end: T, rate: T, speed_start: T, speed_end: T) -> T {
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
