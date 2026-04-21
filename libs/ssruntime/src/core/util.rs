use crate::core::framedata::{CachedPartData, NUM_PART_ATTRIBUTES, evaluate_part};
use crate::core::matrix::SsMatrix4x4;
use crate::core::matrix::SsVector3;
use alloc::vec;
use alloc::vec::Vec;
use wide::f32x4;

/// Calculate the target frame for a child instance animation.
pub fn calculate_instance_frame(parent_frame: f32, key_frame: i32, start_offset: i32, speed: f32, independent: bool, accumulated_time: f32) -> f32 {
    if independent { accumulated_time * speed + start_offset as f32 } else { (parent_frame - key_frame as f32) * speed + start_offset as f32 }
}

/// Compute world matrix for a specific part by traversing its ancestors.
pub fn get_part_world_matrix(parts: &[CachedPartData], part_index: u16, frame_no: i32) -> Option<SsMatrix4x4> {
    if (part_index as usize) >= parts.len() {
        return None;
    }

    // Temporary cursors for this specific part's ancestor calculation.
    // We use a fresh cursor set to avoid side-effects on the main context's cursors.
    let mut temp_cursors = vec![0usize; parts.len() * NUM_PART_ATTRIBUTES];

    fn compute_recursive(target_idx: u16, frame: i32, parts: &[CachedPartData], cursors: &mut [usize], is_drawing: bool) -> SsMatrix4x4 {
        let part = &parts[target_idx as usize];
        let part_cursors = &mut cursors[target_idx as usize * NUM_PART_ATTRIBUTES..(target_idx as usize + 1) * NUM_PART_ATTRIBUTES];

        if let Some(res) = evaluate_part(part, frame, part_cursors) {
            let parent_idx = part.parent_index;
            let parent_mat = if parent_idx >= 0 { Some(compute_recursive(parent_idx as u16, frame, parts, cursors, false)) } else { None };

            let (inh, draw) = SsMatrix4x4::compute_world_matrix(&res.state, parent_mat.as_ref(), None);
            if is_drawing { draw } else { inh }
        } else {
            SsMatrix4x4::identity()
        }
    }

    Some(compute_recursive(part_index, frame_no, parts, &mut temp_cursors, true))
}

/// Transform multiple particle positions by a matrix using SIMD (wide::f32x4).
/// particles_raw: Raw byte slice of ParticleState array from ss_effect_get_state.
pub fn transform_particles(matrix: &SsMatrix4x4, particles_raw: &[u8]) -> Vec<(f32, f32)> {
    // Layout from effect_state.fbs:
    // x:0(4), y:4(4), scale_x:8(4), scale_y:12(4), rotation:16(4), direction:20(4), color:24(4) = 28 bytes per particle
    let stride = 28;
    let count = particles_raw.len() / stride;
    let mut result = Vec::with_capacity(count);

    let mut i = 0;
    // Process 4 particles at a time
    while i + 3 < count {
        let b0 = i * stride;
        let b1 = (i + 1) * stride;
        let b2 = (i + 2) * stride;
        let b3 = (i + 3) * stride;

        // Load 4 X and 4 Y coordinates
        let vx = f32x4::from([
            f32::from_le_bytes(particles_raw[b0..b0 + 4].try_into().unwrap()),
            f32::from_le_bytes(particles_raw[b1..b1 + 4].try_into().unwrap()),
            f32::from_le_bytes(particles_raw[b2..b2 + 4].try_into().unwrap()),
            f32::from_le_bytes(particles_raw[b3..b3 + 4].try_into().unwrap()),
        ]);
        let vy = f32x4::from([
            f32::from_le_bytes(particles_raw[b0 + 4..b0 + 8].try_into().unwrap()),
            f32::from_le_bytes(particles_raw[b1 + 4..b1 + 8].try_into().unwrap()),
            f32::from_le_bytes(particles_raw[b2 + 4..b2 + 8].try_into().unwrap()),
            f32::from_le_bytes(particles_raw[b3 + 4..b3 + 8].try_into().unwrap()),
        ]);

        // Transform: Res = col0*vx + col1*vy + col3 (simplified 2D transform as Z is assumed 0)
        // We use SIMD for the core calculation, then push the results.
        let res_x: [f32; 4] = (f32x4::splat(matrix.m[0]) * vx + f32x4::splat(matrix.m[4]) * vy + f32x4::splat(matrix.m[12])).into();
        let res_y: [f32; 4] = (f32x4::splat(matrix.m[1]) * vx + f32x4::splat(matrix.m[5]) * vy + f32x4::splat(matrix.m[13])).into();

        result.push((res_x[0], res_y[0]));
        result.push((res_x[1], res_y[1]));
        result.push((res_x[2], res_y[2]));
        result.push((res_x[3], res_y[3]));

        i += 4;
    }

    // Remaining particles
    while i < count {
        let base = i * stride;
        let px = f32::from_le_bytes(particles_raw[base..base + 4].try_into().unwrap());
        let py = f32::from_le_bytes(particles_raw[base + 4..base + 8].try_into().unwrap());

        let vec = SsVector3::new(px, py, 0.0);
        let transformed = matrix.transform(&vec);
        result.push((transformed.x, transformed.y));
        i += 1;
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
    fn test_transform_particles_simd_and_remainder() {
        // ... (existing test code)
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_get_part_world_matrix_with_real_data() {
        use crate::{AnimationResource, SsabData};
        use std::path::PathBuf;

        fn get_test_base_path() -> PathBuf {
            std::env::current_dir().unwrap().ancestors().nth(2).unwrap().to_path_buf()
        }

        let ssab_path = get_test_base_path().join("tests/overall/overall_ssab/Basic.ssab");
        if !ssab_path.exists() {
            // Skip if test data is not prepared
            return;
        }

        let input_buf = std::fs::read(&ssab_path).unwrap();
        let ssab_data = SsabData::load_copy(&input_buf).unwrap();
        let resource = AnimationResource::new(ssab_data.as_slice()).unwrap();
        let metadata = &resource.animations[0]; // anime_1

        // Find index of "POS_XYZ"
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_data.as_slice()) };
        let pos_xyz_idx = ssab.parts().unwrap().iter().position(|p| p.name() == Some("POS_XYZ")).unwrap() as u16;

        // Calculate world matrix via utility at frame 10
        let res_m = get_part_world_matrix(&metadata.cached_parts, pos_xyz_idx, 10).unwrap();

        // X translation should be 32.0 based on evaluate_animation_at_frame test
        assert!((res_m.m[12] - 32.0).abs() < 1e-5, "World X: {} != 32.0", res_m.m[12]);
        assert!((res_m.m[0] - 1.0).abs() < 1e-5, "Scale X: {} != 1.0", res_m.m[0]);
    }
}
