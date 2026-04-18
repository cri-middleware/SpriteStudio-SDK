use core::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};
use wide::f32x4;

#[cfg(not(feature = "std"))]
use libm::{cosf, fabsf, sinf};

#[cfg(feature = "std")]
#[inline]
fn cosf(f: f32) -> f32 {
    f.cos()
}
#[cfg(feature = "std")]
#[inline]
fn sinf(f: f32) -> f32 {
    f.sin()
}
#[cfg(feature = "std")]
#[inline]
fn fabsf(f: f32) -> f32 {
    f.abs()
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SsVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl SsVector3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 1.0, 1.0);

    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    #[must_use]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline]
    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }
}

// --- Vector Operators ---
impl Add for SsVector3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl AddAssign for SsVector3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
impl Sub for SsVector3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl SubAssign for SsVector3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}
impl Mul<f32> for SsVector3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}
impl MulAssign<f32> for SsVector3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}
impl Div<f32> for SsVector3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        let inv = 1.0 / rhs;
        self * inv
    }
}
impl DivAssign<f32> for SsVector3 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        let inv = 1.0 / rhs;
        *self *= inv;
    }
}

impl Index<usize> for SsVector3 {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds for SsVector3"),
        }
    }
}
impl IndexMut<usize> for SsVector3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of bounds for SsVector3"),
        }
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SsMatrix4x4 {
    pub m: [f32; 16],
}

impl SsMatrix4x4 {
    pub const IDENTITY: Self = Self { m: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] };

    #[inline]
    #[must_use]
    pub const fn identity() -> Self {
        Self::IDENTITY
    }

    #[inline]
    #[must_use]
    pub const fn from_array(m: [f32; 16]) -> Self {
        Self { m }
    }

    #[inline]
    #[must_use]
    pub fn create_scale(x: f32, y: f32, z: f32) -> Self {
        let mut res = Self::IDENTITY;
        res.m[0] = x;
        res.m[5] = y;
        res.m[10] = z;
        res
    }

    #[inline]
    #[must_use]
    pub fn create_translation(x: f32, y: f32, z: f32) -> Self {
        let mut res = Self::IDENTITY;
        res.m[12] = x;
        res.m[13] = y;
        res.m[14] = z;
        res
    }

    #[inline]
    #[must_use]
    pub fn create_rotation_x(radians: f32) -> Self {
        let mut res = Self::IDENTITY;
        let s = sinf(radians);
        let c = cosf(radians);
        res.m[5] = c;
        res.m[6] = s;
        res.m[9] = -s;
        res.m[10] = c;
        res
    }

    #[inline]
    #[must_use]
    pub fn create_rotation_y(radians: f32) -> Self {
        let mut res = Self::IDENTITY;
        let s = sinf(radians);
        let c = cosf(radians);
        res.m[0] = c;
        res.m[2] = -s;
        res.m[8] = s;
        res.m[10] = c;
        res
    }

    #[inline]
    #[must_use]
    pub fn create_rotation_z(radians: f32) -> Self {
        let mut res = Self::IDENTITY;
        let s = sinf(radians);
        let c = cosf(radians);
        res.m[0] = c;
        res.m[1] = s;
        res.m[4] = -s;
        res.m[5] = c;
        res
    }

    #[inline]
    #[must_use]
    pub fn multiply(&self, rhs: &Self) -> Self {
        let mut res = [0.0f32; 16];

        // Load LHS columns as SIMD vectors safely
        let a_col0 = f32x4::from([self.m[0], self.m[1], self.m[2], self.m[3]]);
        let a_col1 = f32x4::from([self.m[4], self.m[5], self.m[6], self.m[7]]);
        let a_col2 = f32x4::from([self.m[8], self.m[9], self.m[10], self.m[11]]);
        let a_col3 = f32x4::from([self.m[12], self.m[13], self.m[14], self.m[15]]);

        for j in 0..4 {
            let offset = j * 4;
            let b0j = f32x4::splat(rhs.m[offset]);
            let b1j = f32x4::splat(rhs.m[offset + 1]);
            let b2j = f32x4::splat(rhs.m[offset + 2]);
            let b3j = f32x4::splat(rhs.m[offset + 3]);

            let res_col_j = (a_col0 * b0j) + (a_col1 * b1j) + (a_col2 * b2j) + (a_col3 * b3j);
            let res_arr: [f32; 4] = res_col_j.into();
            res[offset..offset + 4].copy_from_slice(&res_arr);
        }
        Self { m: res }
    }

    #[inline]
    #[must_use]
    pub fn transform(&self, v: &SsVector3) -> SsVector3 {
        let a_col0 = f32x4::from([self.m[0], self.m[1], self.m[2], self.m[3]]);
        let a_col1 = f32x4::from([self.m[4], self.m[5], self.m[6], self.m[7]]);
        let a_col2 = f32x4::from([self.m[8], self.m[9], self.m[10], self.m[11]]);
        let a_col3 = f32x4::from([self.m[12], self.m[13], self.m[14], self.m[15]]);

        let vx = f32x4::splat(v.x);
        let vy = f32x4::splat(v.y);
        let vz = f32x4::splat(v.z);
        let vw = f32x4::splat(1.0);

        let res_vec = (a_col0 * vx) + (a_col1 * vy) + (a_col2 * vz) + (a_col3 * vw);
        let res_arr: [f32; 4] = res_vec.into();

        SsVector3 { x: res_arr[0], y: res_arr[1], z: res_arr[2] }
    }

    #[must_use]
    pub fn inverse(&self) -> Self {
        let m = &self.m;

        let s0 = m[0] * m[5] - m[1] * m[4];
        let s1 = m[0] * m[6] - m[2] * m[4];
        let s2 = m[0] * m[7] - m[3] * m[4];
        let s3 = m[1] * m[6] - m[2] * m[5];
        let s4 = m[1] * m[7] - m[3] * m[5];
        let s5 = m[2] * m[7] - m[3] * m[6];

        let c5 = m[10] * m[15] - m[11] * m[14];
        let c4 = m[9] * m[15] - m[11] * m[13];
        let c3 = m[9] * m[14] - m[10] * m[13];
        let c2 = m[8] * m[15] - m[11] * m[12];
        let c1 = m[8] * m[14] - m[10] * m[12];
        let c0 = m[8] * m[13] - m[9] * m[12];

        let det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;

        if fabsf(det) < 1e-12 {
            return Self::IDENTITY;
        }

        let inv_det = 1.0 / det;
        let mut res = [0.0f32; 16];

        res[0] = (m[5] * c5 - m[6] * c4 + m[7] * c3) * inv_det;
        res[1] = (-m[1] * c5 + m[2] * c4 - m[3] * c3) * inv_det;
        res[2] = (m[13] * s5 - m[14] * s4 + m[15] * s3) * inv_det;
        res[3] = (-m[9] * s5 + m[10] * s4 - m[11] * s3) * inv_det;

        res[4] = (-m[4] * c5 + m[6] * c2 - m[7] * c1) * inv_det;
        res[5] = (m[0] * c5 - m[2] * c2 + m[3] * c1) * inv_det;
        res[6] = (-m[12] * s5 + m[14] * s2 - m[15] * s1) * inv_det;
        res[7] = (m[8] * s5 - m[10] * s2 + m[11] * s1) * inv_det;

        res[8] = (m[4] * c4 - m[5] * c2 + m[7] * c0) * inv_det;
        res[9] = (-m[0] * c4 + m[1] * c2 - m[3] * c0) * inv_det;
        res[10] = (m[12] * s4 - m[13] * s2 + m[15] * s0) * inv_det;
        res[11] = (-m[8] * s4 + m[9] * s2 - m[11] * s0) * inv_det;

        res[12] = (-m[4] * c3 + m[5] * c1 - m[6] * c0) * inv_det;
        res[13] = (m[0] * c3 - m[1] * c1 + m[2] * c0) * inv_det;
        res[14] = (-m[12] * s3 + m[13] * s1 - m[14] * s0) * inv_det;
        res[15] = (m[8] * s3 - m[9] * s1 + m[10] * s0) * inv_det;

        Self { m: res }
    }
}

// --- Matrix Traits ---
impl Default for SsMatrix4x4 {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for SsMatrix4x4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        self.multiply(&rhs)
    }
}
impl Mul<&SsMatrix4x4> for &SsMatrix4x4 {
    type Output = SsMatrix4x4;
    #[inline]
    fn mul(self, rhs: &SsMatrix4x4) -> SsMatrix4x4 {
        self.multiply(rhs)
    }
}
impl MulAssign for SsMatrix4x4 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Mul<SsVector3> for SsMatrix4x4 {
    type Output = SsVector3;
    #[inline]
    fn mul(self, rhs: SsVector3) -> SsVector3 {
        self.transform(&rhs)
    }
}

impl Mul<&SsVector3> for &SsMatrix4x4 {
    type Output = SsVector3;
    #[inline]
    fn mul(self, rhs: &SsVector3) -> SsVector3 {
        self.transform(rhs)
    }
}

impl Index<usize> for SsMatrix4x4 {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.m[index]
    }
}
impl IndexMut<usize> for SsMatrix4x4 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.m[index]
    }
}

impl AsRef<[f32; 16]> for SsMatrix4x4 {
    #[inline]
    fn as_ref(&self) -> &[f32; 16] {
        &self.m
    }
}
impl AsMut<[f32; 16]> for SsMatrix4x4 {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; 16] {
        &mut self.m
    }
}

// --- Conversion Traits ---
impl From<[f32; 16]> for SsMatrix4x4 {
    #[inline]
    fn from(m: [f32; 16]) -> Self {
        Self { m }
    }
}
impl From<SsMatrix4x4> for [f32; 16] {
    #[inline]
    fn from(mat: SsMatrix4x4) -> [f32; 16] {
        mat.m
    }
}

impl From<[f32; 3]> for SsVector3 {
    #[inline]
    fn from(v: [f32; 3]) -> Self {
        Self::new(v[0], v[1], v[2])
    }
}
impl From<SsVector3> for [f32; 3] {
    #[inline]
    fn from(v: SsVector3) -> [f32; 3] {
        [v.x, v.y, v.z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn assert_f32_eq(a: f32, b: f32) {
        assert!((a - b).abs() < EPSILON, "{} != {}", a, b);
    }

    fn assert_vec3_eq(a: SsVector3, b: SsVector3) {
        assert_f32_eq(a.x, b.x);
        assert_f32_eq(a.y, b.y);
        assert_f32_eq(a.z, b.z);
    }

    fn assert_mat4_eq(a: &SsMatrix4x4, b: &SsMatrix4x4) {
        for i in 0..16 {
            assert_f32_eq(a.m[i], b.m[i]);
        }
    }

    // --- SsVector3 Tests ---

    #[test]
    fn test_vector_creation_and_constants() {
        let v = SsVector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);

        assert_eq!(SsVector3::ZERO.x, 0.0);
        assert_eq!(SsVector3::ZERO.y, 0.0);
        assert_eq!(SsVector3::ZERO.z, 0.0);

        assert_eq!(SsVector3::ONE.x, 1.0);
        assert_eq!(SsVector3::ONE.y, 1.0);
        assert_eq!(SsVector3::ONE.z, 1.0);
    }

    #[test]
    fn test_vector_math() {
        let v1 = SsVector3::new(1.0, 2.0, 3.0);
        let v2 = SsVector3::new(4.0, 5.0, 6.0);

        // Dot product: 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert_f32_eq(v1.dot(v2), 32.0);

        // Length squared: 1^2 + 2^2 + 3^2 = 14
        assert_f32_eq(v1.length_squared(), 14.0);
    }

    #[test]
    fn test_vector_operators() {
        let v1 = SsVector3::new(1.0, 2.0, 3.0);
        let v2 = SsVector3::new(4.0, 5.0, 6.0);

        assert_vec3_eq(v1 + v2, SsVector3::new(5.0, 7.0, 9.0));
        assert_vec3_eq(v1 - v2, SsVector3::new(-3.0, -3.0, -3.0));
        assert_vec3_eq(v1 * 2.0, SsVector3::new(2.0, 4.0, 6.0));
        assert_vec3_eq(v1 / 2.0, SsVector3::new(0.5, 1.0, 1.5));

        let mut v3 = v1;
        v3 += v2;
        assert_vec3_eq(v3, SsVector3::new(5.0, 7.0, 9.0));

        let mut v4 = v1;
        v4 -= v2;
        assert_vec3_eq(v4, SsVector3::new(-3.0, -3.0, -3.0));

        let mut v5 = v1;
        v5 *= 3.0;
        assert_vec3_eq(v5, SsVector3::new(3.0, 6.0, 9.0));

        let mut v6 = v1;
        v6 /= 2.0;
        assert_vec3_eq(v6, SsVector3::new(0.5, 1.0, 1.5));
    }

    #[test]
    fn test_vector_indexing() {
        let mut v = SsVector3::new(1.0, 2.0, 3.0);
        assert_eq!(v[0], 1.0);
        assert_eq!(v[1], 2.0);
        assert_eq!(v[2], 3.0);

        v[1] = 5.0;
        assert_eq!(v.y, 5.0);
    }

    // --- SsMatrix4x4 Tests ---

    #[test]
    fn test_matrix_identity() {
        let id = SsMatrix4x4::identity();
        assert_f32_eq(id.m[0], 1.0);
        assert_f32_eq(id.m[5], 1.0);
        assert_f32_eq(id.m[10], 1.0);
        assert_f32_eq(id.m[15], 1.0);

        assert_f32_eq(id.m[1], 0.0); // Random off-diagonal element
    }

    #[test]
    fn test_matrix_create_scale() {
        let s = SsMatrix4x4::create_scale(2.0, 3.0, 4.0);
        assert_f32_eq(s.m[0], 2.0);
        assert_f32_eq(s.m[5], 3.0);
        assert_f32_eq(s.m[10], 4.0);
        assert_f32_eq(s.m[15], 1.0);
    }

    #[test]
    fn test_matrix_create_translation() {
        let t = SsMatrix4x4::create_translation(5.0, 6.0, 7.0);
        assert_f32_eq(t.m[12], 5.0);
        assert_f32_eq(t.m[13], 6.0);
        assert_f32_eq(t.m[14], 7.0);
        assert_f32_eq(t.m[15], 1.0);
    }

    #[test]
    fn test_matrix_create_rotation_x() {
        let r = SsMatrix4x4::create_rotation_x(core::f32::consts::FRAC_PI_2); // 90 degrees
        // cos(90) = 0, sin(90) = 1
        assert_f32_eq(r.m[5], 0.0);
        assert_f32_eq(r.m[6], 1.0);
        assert_f32_eq(r.m[9], -1.0);
        assert_f32_eq(r.m[10], 0.0);
    }

    #[test]
    fn test_matrix_create_rotation_y() {
        let r = SsMatrix4x4::create_rotation_y(core::f32::consts::FRAC_PI_2); // 90 degrees
        // cos(90) = 0, sin(90) = 1
        assert_f32_eq(r.m[0], 0.0);
        assert_f32_eq(r.m[2], -1.0);
        assert_f32_eq(r.m[8], 1.0);
        assert_f32_eq(r.m[10], 0.0);
    }

    #[test]
    fn test_matrix_create_rotation_z() {
        let r = SsMatrix4x4::create_rotation_z(core::f32::consts::FRAC_PI_2); // 90 degrees
        // cos(90) = 0, sin(90) = 1
        assert_f32_eq(r.m[0], 0.0);
        assert_f32_eq(r.m[1], 1.0);
        assert_f32_eq(r.m[4], -1.0);
        assert_f32_eq(r.m[5], 0.0);
    }

    #[test]
    fn test_matrix_multiplication() {
        let t = SsMatrix4x4::create_translation(1.0, 2.0, 3.0);
        let s = SsMatrix4x4::create_scale(2.0, 2.0, 2.0);

        // Scale then Translate (T * S)
        // Vector v will be scaled by 2, then translated by (1, 2, 3)
        let ts = t * s;

        let v = SsVector3::new(1.0, 1.0, 1.0);
        let res = ts * v;

        // (1*2)+1 = 3, (1*2)+2 = 4, (1*2)+3 = 5
        assert_vec3_eq(res, SsVector3::new(3.0, 4.0, 5.0));
    }

    #[test]
    fn test_matrix_transform_vector() {
        let t = SsMatrix4x4::create_translation(5.0, -2.0, 1.0);
        let v = SsVector3::new(1.0, 1.0, 1.0);

        let transformed = t.transform(&v);
        assert_vec3_eq(transformed, SsVector3::new(6.0, -1.0, 2.0));

        // Test operator overloading
        let transformed2 = &t * &v;
        assert_vec3_eq(transformed2, SsVector3::new(6.0, -1.0, 2.0));
    }

    #[test]
    fn test_matrix_inverse() {
        // Identity inverse is Identity
        let id = SsMatrix4x4::identity();
        let id_inv = id.inverse();
        assert_mat4_eq(&id, &id_inv);

        // Translation inverse
        let t = SsMatrix4x4::create_translation(2.0, 3.0, 4.0);
        let t_inv = t.inverse();
        let expected_t_inv = SsMatrix4x4::create_translation(-2.0, -3.0, -4.0);
        assert_mat4_eq(&t_inv, &expected_t_inv);

        // Scale inverse
        let s = SsMatrix4x4::create_scale(2.0, 4.0, 5.0);
        let s_inv = s.inverse();
        let expected_s_inv = SsMatrix4x4::create_scale(0.5, 0.25, 0.2);
        assert_mat4_eq(&s_inv, &expected_s_inv);

        // T * S inverse
        let ts = t * s;
        let ts_inv = ts.inverse();

        // A * A^-1 = I
        let identity_check = ts * ts_inv;
        assert_mat4_eq(&identity_check, &SsMatrix4x4::identity());
    }

    #[test]
    fn test_matrix_indexing_and_conversions() {
        let mut m = SsMatrix4x4::identity();
        m[0] = 5.0;
        assert_f32_eq(m.m[0], 5.0);
        assert_f32_eq(m[0], 5.0);

        let arr: [f32; 16] = m.into();
        assert_f32_eq(arr[0], 5.0);

        let m2: SsMatrix4x4 = arr.into();
        assert_f32_eq(m2[0], 5.0);
    }
}
