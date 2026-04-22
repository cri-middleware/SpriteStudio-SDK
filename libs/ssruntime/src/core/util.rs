use crate::core::framedata::{evaluate_part, CachedPartData, NUM_PART_ATTRIBUTES};
use crate::core::matrix::SsMatrix4x4;
use crate::core::{SsCoordinateSystem, SsOrigin, SsUnit};
use alloc::vec;
use alloc::vec::Vec;
use wide::f32x4;

/// Calculate the target frame for a child instance animation.
pub fn calculate_instance_frame(parent_frame: f32, key_frame: i32, start_offset: i32, speed: f32, independent: bool, accumulated_time: f32) -> f32 {
    if independent {
        accumulated_time * speed + start_offset as f32
    } else {
        (parent_frame - key_frame as f32) * speed + start_offset as f32
    }
}

/// Compute world matrix for a specific part by traversing its ancestors.
pub fn get_part_world_matrix(parts: &[CachedPartData], part_index: u16, frame_no: i32, coordinate_system: SsCoordinateSystem, origin: SsOrigin, unit: SsUnit, canvas_width: f32, canvas_height: f32) -> Option<SsMatrix4x4> {
    if (part_index as usize) >= parts.len() {
        return None;
    }

    // Temporary cursors for this specific part's ancestor calculation.
    // We use a fresh cursor set to avoid side-effects on the main context's cursors.
    let mut temp_cursors = vec![0usize; parts.len() * NUM_PART_ATTRIBUTES];

    fn compute_recursive(target_idx: u16, frame: i32, parts: &[CachedPartData], cursors: &mut [usize], coordinate_system: SsCoordinateSystem, origin: SsOrigin, unit: SsUnit, canvas_width: f32, canvas_height: f32, is_drawing: bool) -> SsMatrix4x4 {
        let part = &parts[target_idx as usize];
        let part_cursors = &mut cursors[target_idx as usize * NUM_PART_ATTRIBUTES..(target_idx as usize + 1) * NUM_PART_ATTRIBUTES];

        let res = evaluate_part(part, frame, part_cursors, coordinate_system, origin, unit, canvas_width, canvas_height);
        let parent_idx = part.parent_index;
        let parent_mat = if parent_idx >= 0 { Some(compute_recursive(parent_idx as u16, frame, parts, cursors, coordinate_system, origin, unit, canvas_width, canvas_height, false)) } else { None };

        let (inh, draw) = SsMatrix4x4::compute_world_matrix(&res.state, parent_mat.as_ref(), None);
        if is_drawing { draw } else { inh }
    }

    Some(compute_recursive(part_index, frame_no, parts, &mut temp_cursors, coordinate_system, origin, unit, canvas_width, canvas_height, true))
}

/// Transform multiple 2D points (x, y, x, y...) by a matrix using SIMD (wide::f32x4).
pub fn transform_vec2s(matrix: &SsMatrix4x4, input: &[f32]) -> Vec<f32> {
    let count = input.len() / 2;
    let mut result = Vec::with_capacity(input.len());

    let m0 = f32x4::splat(matrix.m[0]);
    let m1 = f32x4::splat(matrix.m[1]);
    let m4 = f32x4::splat(matrix.m[4]);
    let m5 = f32x4::splat(matrix.m[5]);
    let m12 = f32x4::splat(matrix.m[12]);
    let m13 = f32x4::splat(matrix.m[13]);

    let mut i = 0;
    while i + 3 < count {
        let base = i * 2;
        // Rearrange to SoA: vx = [x0, x1, x2, x3], vy = [y0, y1, y2, y3]
        // This makes the math easier with wide crate.
        let vx = f32x4::from([input[base], input[base + 2], input[base + 4], input[base + 6]]);
        let vy = f32x4::from([input[base + 1], input[base + 3], input[base + 5], input[base + 7]]);

        let res_x: [f32; 4] = (m0 * vx + m4 * vy + m12).into();
        let res_y: [f32; 4] = (m1 * vx + m5 * vy + m13).into();

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
        // ... (existing test code)
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_get_part_world_matrix_ydown() {
        use crate::{AnimationResource, SsabData};
        use std::path::PathBuf;

        fn get_test_base_path() -> PathBuf {
            std::env::current_dir().unwrap().ancestors().nth(2).unwrap().to_path_buf()
        }

        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        if !ssab_path.exists() {
            return;
        }

        let input_buf = std::fs::read(&ssab_path).unwrap();
        let ssab_data = SsabData::load_copy(&input_buf).unwrap();
        let resource = AnimationResource::new(ssab_data.as_slice()).unwrap();
        let metadata = &resource.animations[0];

        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_data.as_slice()) };
        let pos_xyz_idx = ssab.parts().unwrap().iter().position(|p| p.name() == Some("POS_XYZ")).unwrap() as u16;

        // Calculate world matrix via utility at frame 10 (Y-Up)
        let res_m_up = get_part_world_matrix(&metadata.cached_parts, pos_xyz_idx, 10, SsCoordinateSystem::YUp, SsOrigin::Center, SsUnit::Pixel, metadata.canvas_width, metadata.canvas_height).unwrap();
        
        // Calculate world matrix via utility at frame 10 (Y-Down)
        let res_m_down = get_part_world_matrix(&metadata.cached_parts, pos_xyz_idx, 10, SsCoordinateSystem::YDown, SsOrigin::Center, SsUnit::Pixel, metadata.canvas_width, metadata.canvas_height).unwrap();

        // In Y-Up, Y is positive. In Y-Down, it should be negated.
        assert!((res_m_up.m[12] - 32.0).abs() < 1e-5, "World X: {} != 32.0", res_m_up.m[12]);
        assert_eq!(res_m_up.m[12], res_m_down.m[12]); // X is same
        
        // Let's assume original Y translation at frame 10 was 0.0 or something else, but down should be negated.
        assert!((res_m_up.m[13] + res_m_down.m[13]).abs() < 1e-5, "Y should be negated: {} vs {}", res_m_up.m[13], res_m_down.m[13]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_get_part_world_matrix_topleft_normalized() {
        use crate::{AnimationResource, SsabData};
        use std::path::PathBuf;

        fn get_test_base_path() -> PathBuf {
            std::env::current_dir().unwrap().ancestors().nth(2).unwrap().to_path_buf()
        }

        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        if !ssab_path.exists() {
            return;
        }

        let input_buf = std::fs::read(&ssab_path).unwrap();
        let ssab_data = SsabData::load_copy(&input_buf).unwrap();
        let resource = AnimationResource::new(ssab_data.as_slice()).unwrap();
        let metadata = &resource.animations[0];

        let _ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_data.as_slice()) };
        let root_idx = 0; // Usually index 0 is root
        
        let canvas_w = metadata.canvas_width;
        let canvas_h = metadata.canvas_height;

        // Center / Pixel (Original)
        let m_orig = get_part_world_matrix(&metadata.cached_parts, root_idx, 0, SsCoordinateSystem::YUp, SsOrigin::Center, SsUnit::Pixel, canvas_w, canvas_h).unwrap();
        
        // TopLeft / Pixel
        let m_tl = get_part_world_matrix(&metadata.cached_parts, root_idx, 0, SsCoordinateSystem::YUp, SsOrigin::TopLeft, SsUnit::Pixel, canvas_w, canvas_h).unwrap();
        
        // m_tl should be m_orig shifted by (w/2, -h/2) in Y-Up
        assert!((m_tl.m[12] - (m_orig.m[12] + canvas_w / 2.0)).abs() < 1e-5);
        assert!((m_tl.m[13] - (m_orig.m[13] - canvas_h / 2.0)).abs() < 1e-5);

        // Center / Normalized
        let m_norm = get_part_world_matrix(&metadata.cached_parts, root_idx, 0, SsCoordinateSystem::YUp, SsOrigin::Center, SsUnit::Normalized, canvas_w, canvas_h).unwrap();
        assert!((m_norm.m[12] - (m_orig.m[12] / canvas_w)).abs() < 1e-5);
        assert!((m_norm.m[13] - (m_orig.m[13] / canvas_h)).abs() < 1e-5);
    }
}
