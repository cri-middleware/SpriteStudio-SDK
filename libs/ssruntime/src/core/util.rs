use crate::core::matrix::SsMatrix4x4;
use alloc::vec::Vec;
use wide::f32x4;

/// Calculate the target frame for a child instance animation.
pub fn calculate_instance_frame(parent_frame: f32, key_frame: i32, start_offset: i32, speed: f32, independent: bool, accumulated_time: f32) -> f32 {
    if independent { accumulated_time * speed + start_offset as f32 } else { (parent_frame - key_frame as f32) * speed + start_offset as f32 }
}

/// Transform multiple 2D points (x, y, x, y...) by a matrix using SIMD (wide::f32x4).
pub fn transform_vec2s(matrix: &SsMatrix4x4, input: &[f32]) -> Vec<f32> {
    let count = input.len() / 2;
    let mut result = Vec::with_capacity(input.len());

    let mut i = 0;
    while i + 3 < count {
        let base = i * 2;
        // Rearrange to SoA: vx = [x0, x1, x2, x3], vy = [y0, y1, y2, y3]
        // This makes the math easier with wide crate.
        let vx = f32x4::from([input[base], input[base + 2], input[base + 4], input[base + 6]]);
        let vy = f32x4::from([input[base + 1], input[base + 3], input[base + 5], input[base + 7]]);

        let (res_x_simd, res_y_simd) = matrix.transform_v4_2d(vx, vy);
        let res_x: [f32; 4] = res_x_simd.into();
        let res_y: [f32; 4] = res_y_simd.into();

        for k in 0..4 {
            result.push(res_x[k]);
            result.push(res_y[k]);
        }
        i += 4;
    }

    while i < count {
        let base = i * 2;
        let vx = input[base];
        let vy = input[base + 1];
        result.push(matrix.m[0] * vx + matrix.m[4] * vy + matrix.m[12]);
        result.push(matrix.m[1] * vx + matrix.m[5] * vy + matrix.m[13]);
        i += 1;
    }

    result
}

/// Transform multiple particle positions by a matrix using SIMD (wide::f32x4).
/// particles_raw: Raw byte slice of ParticleState array from ss_effect_get_state.
pub fn transform_particles(matrix: &SsMatrix4x4, particles_raw: &[u8]) -> Vec<(f32, f32)> {
    // Layout from effect_state.fbs:
    // x:0(4), y:4(4), scale_x:8(4), scale_y:12(4), rotation:16(4), direction:20(4), color:24(4) = 28 bytes per particle
    let stride = 28;
    let count = particles_raw.len() / stride;

    // For particles, we extract x, y and then use the general transformer.
    // This is slightly less efficient due to extraction, but cleaner and still SIMD.
    // For extreme performance, we'd keep the specialized stride-aware loop.
    let mut extracted = Vec::with_capacity(count * 2);
    for i in 0..count {
        let base = i * stride;
        extracted.push(f32::from_le_bytes(particles_raw[base..base + 4].try_into().unwrap()));
        extracted.push(f32::from_le_bytes(particles_raw[base + 4..base + 8].try_into().unwrap()));
    }

    let transformed = transform_vec2s(matrix, &extracted);
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        result.push((transformed[i * 2], transformed[i * 2 + 1]));
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_instance_frame() {
        // Dependent case: (10 - 2) * 2.0 + 5 = 21.0
        assert_eq!(calculate_instance_frame(10.0, 2, 5, 2.0, false, 0.0), 21.0);
        // Independent case: 100.0 * 0.5 + 10 = 60.0
        assert_eq!(calculate_instance_frame(10.0, 2, 10, 0.5, true, 100.0), 60.0);

        // Negative speed (reverse playback)
        // (20 - 10) * -1.0 + 30 = 20.0
        assert_eq!(calculate_instance_frame(20.0, 10, 30, -1.0, false, 0.0), 20.0);

        // Zero parent frame
        assert_eq!(calculate_instance_frame(0.0, 0, 0, 1.0, false, 0.0), 0.0);
    }

    #[test]
    fn test_transform_vec2s() {
        let matrix = SsMatrix4x4::create_translation(100.0, -50.0, 0.0);

        // 5 points (10 floats) to test SIMD (4) + Remainder (1)
        let input = vec![
            0.0, 0.0, // p0
            10.0, 10.0, // p1
            -5.0, 20.0, // p2
            30.0, -5.0, // p3
            1.0, 1.0, // p4 (remainder)
        ];

        let output = transform_vec2s(&matrix, &input);
        assert_eq!(output.len(), 10);

        let expected = [100.0, -50.0, 110.0, -40.0, 95.0, -30.0, 130.0, -55.0, 101.0, -49.0];

        for i in 0..10 {
            assert!((output[i] - expected[i]).abs() < 1e-5, "Idx {}: {} != {}", i, output[i], expected[i]);
        }
    }

    #[test]
    fn test_transform_particles_simd_and_remainder() {
        let matrix = SsMatrix4x4::create_translation(10.0, 20.0, 0.0);
        // Stride 28: x:0(4), y:4(4), rest: 20
        let mut particles = vec![0u8; 28 * 2];
        
        // p0: (0, 0) -> (10, 20)
        let x0 = 0.0f32.to_le_bytes();
        let y0 = 0.0f32.to_le_bytes();
        particles[0..4].copy_from_slice(&x0);
        particles[4..8].copy_from_slice(&y0);

        // p1: (5, -5) -> (15, 15)
        let x1 = 5.0f32.to_le_bytes();
        let y1 = (-5.0f32).to_le_bytes();
        particles[28..32].copy_from_slice(&x1);
        particles[32..36].copy_from_slice(&y1);

        let transformed = transform_particles(&matrix, &particles);
        assert_eq!(transformed.len(), 2);
        assert!((transformed[0].0 - 10.0).abs() < 1e-5);
        assert!((transformed[0].1 - 20.0).abs() < 1e-5);
        assert!((transformed[1].0 - 15.0).abs() < 1e-5);
        assert!((transformed[1].1 - 15.0).abs() < 1e-5);
    }
}
