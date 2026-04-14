use wide::{f32x4, u32x4};

pub struct XorShift32x4 {
    state: u32x4,
}

impl XorShift32x4 {
    pub fn new(seeds: [u32; 4]) -> Self {
        let mut safe_seeds = seeds;
        for s in safe_seeds.iter_mut() {
            if *s == 0 {
                // XorShift will be stuck at 0 if the seed is 0.
                *s = 1;
            }
        }
        Self { state: u32x4::new(safe_seeds) }
    }

    pub fn gen_float(&mut self) -> f32x4 {
        let mut x = self.state;
        x = x ^ (x << 13);
        x = x ^ (x >> 17);
        x = x ^ (x << 5);
        self.state = x;

        // Generate float in range [1.0, 2.0)
        let mask = u32x4::splat(0x007F_FFFF);
        let exp = u32x4::splat(0x3F80_0000);
        let bits = (x & mask) | exp;

        // SAFETY: Both u32x4 and f32x4 have the same size and alignment (16 bytes).
        // Transmuting from an array of bits representing IEEE-754 floats to SIMD floats is safe.
        let f: f32x4 = unsafe { core::mem::transmute(bits) };

        // Return float in range [0.0, 1.0)
        f - f32x4::splat(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xorshift32x4_zero_seed() {
        let mut rng = XorShift32x4::new([0, 0, 0, 0]);
        let val1 = rng.gen_float();
        let val2 = rng.gen_float();

        // Ensure values are not identical over time (stuck at 0 for example)
        let vals1 = val1.to_array();
        let vals2 = val2.to_array();

        assert_ne!(vals1, vals2);
    }

    #[test]
    fn test_xorshift32x4_range() {
        let mut rng = XorShift32x4::new([12345, 67890, 13579, 24680]);
        for _ in 0..100 {
            let val = rng.gen_float();
            let arr = val.to_array();
            for &f in arr.iter() {
                assert!(f >= 0.0 && f < 1.0, "Value out of range: {}", f);
            }
        }
    }

    #[test]
    fn test_xorshift32x4_distribution() {
        let mut rng = XorShift32x4::new([12345, 67890, 13579, 24680]);
        let val1 = rng.gen_float().to_array();
        let val2 = rng.gen_float().to_array();

        // Elements should not be identically correlated
        assert_ne!(val1[0], val1[1]);
        assert_ne!(val1[2], val1[3]);

        // Sequential calls should differ
        assert_ne!(val1, val2);
    }
}
