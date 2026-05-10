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
/// Generate ready-to-render quad geometry for a `ParticleState` array.
///
/// For each particle, builds 4 vertices (TL, TR, BL, BR ordering) by applying
/// per-particle rotation + scale + translation in slot-local space. Corners
/// are written directly to `out_verts` — no intermediate allocation.
///
/// Output buffer sizes:
/// - `out_verts`:  `count * 8`  floats — 4 corners × (x, y)
/// - `out_uvs`:    `count * 8`  floats — 4 corners × (u, v); same UVs every particle
/// - `out_colors`: `count * 16` floats — 4 corners × (r, g, b, a) normalized 0..1
///
/// `particles_raw` follows `ParticleState` (stride = 28). `rotation` is read in
/// **degrees**, `direction` in **radians** — the effect-state schema's mixed-units
/// convention is absorbed here so callers do not need to redo the conversion.
///
/// `y_flip = true` mirrors the particle Y axis (Godot Y-down rendering of
/// Y-up effect content): the per-particle translation Y and the rotation sign
/// are negated so the slot-local output composes correctly with a Y-up→Y-down
/// downstream transform applied via canvas-parent / view matrix.
pub fn emit_particle_quads(
    particles_raw: &[u8],
    quad_origin_x: f32, quad_origin_y: f32,
    quad_w: f32, quad_h: f32,
    u0: f32, v0: f32, u1: f32, v1: f32,
    y_flip: bool,
    out_verts: &mut [f32],
    out_uvs: &mut [f32],
    out_colors: &mut [f32],
) {
    const STRIDE: usize = 28;
    const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;
    let count = particles_raw.len() / STRIDE;

    let lx = [quad_origin_x, quad_origin_x + quad_w, quad_origin_x, quad_origin_x + quad_w];
    let ly = [quad_origin_y, quad_origin_y, quad_origin_y + quad_h, quad_origin_y + quad_h];
    let uvs_corner = [u0, v0, u1, v0, u0, v1, u1, v1];

    let y_sign = if y_flip { -1.0 } else { 1.0 };

    for i in 0..count {
        let base = i * STRIDE;
        let read = |off: usize| f32::from_le_bytes(particles_raw[base + off..base + off + 4].try_into().unwrap());
        let x = read(0);
        let y = read(4);
        let sx = read(8);
        let sy = read(12);
        let rotation_deg = read(16);
        let direction_rad = read(20);
        let a = particles_raw[base + 24] as f32 / 255.0;
        let r = particles_raw[base + 25] as f32 / 255.0;
        let g = particles_raw[base + 26] as f32 / 255.0;
        let b = particles_raw[base + 27] as f32 / 255.0;

        let angle = y_sign * (rotation_deg * DEG_TO_RAD + direction_rad);
        let c = libm::cosf(angle);
        let s = libm::sinf(angle);
        let py = y_sign * y;

        let v_base = i * 8;
        for v in 0..4 {
            let xx = lx[v] * sx;
            let yy = ly[v] * sy;
            out_verts[v_base + v * 2]     = c * xx - s * yy + x;
            out_verts[v_base + v * 2 + 1] = s * xx + c * yy + py;
        }

        out_uvs[v_base..v_base + 8].copy_from_slice(&uvs_corner);

        let c_base = i * 16;
        for v in 0..4 {
            out_colors[c_base + v * 4] = r;
            out_colors[c_base + v * 4 + 1] = g;
            out_colors[c_base + v * 4 + 2] = b;
            out_colors[c_base + v * 4 + 3] = a;
        }
    }
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

    fn write_particle(buf: &mut [u8], idx: usize, x: f32, y: f32, sx: f32, sy: f32, rot_deg: f32, dir_rad: f32, argb: [u8; 4]) {
        let base = idx * 28;
        buf[base..base + 4].copy_from_slice(&x.to_le_bytes());
        buf[base + 4..base + 8].copy_from_slice(&y.to_le_bytes());
        buf[base + 8..base + 12].copy_from_slice(&sx.to_le_bytes());
        buf[base + 12..base + 16].copy_from_slice(&sy.to_le_bytes());
        buf[base + 16..base + 20].copy_from_slice(&rot_deg.to_le_bytes());
        buf[base + 20..base + 24].copy_from_slice(&dir_rad.to_le_bytes());
        buf[base + 24..base + 28].copy_from_slice(&argb);
    }

    #[test]
    fn test_emit_particle_quads_identity() {
        // 1 particle at origin, no rotation, scale=1; quad = 10x10 centered at (-5, -5).
        let mut particles = vec![0u8; 28];
        write_particle(&mut particles, 0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, [255, 255, 0, 0]);
        let mut verts = vec![0.0f32; 8];
        let mut uvs = vec![0.0f32; 8];
        let mut colors = vec![0.0f32; 16];
        emit_particle_quads(&particles, -5.0, -5.0, 10.0, 10.0, 0.0, 0.0, 1.0, 1.0, false, &mut verts, &mut uvs, &mut colors);
        assert_eq!(verts, vec![-5.0, -5.0, 5.0, -5.0, -5.0, 5.0, 5.0, 5.0]);
        assert_eq!(uvs, vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        for v in 0..4 {
            assert!((colors[v * 4] - 1.0).abs() < 1e-5);
            assert!((colors[v * 4 + 1] - 0.0).abs() < 1e-5);
            assert!((colors[v * 4 + 2] - 0.0).abs() < 1e-5);
            assert!((colors[v * 4 + 3] - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_emit_particle_quads_y_flip() {
        // Particle at (10, 20). y_flip should mirror translation Y to (-20).
        let mut particles = vec![0u8; 28];
        write_particle(&mut particles, 0, 10.0, 20.0, 1.0, 1.0, 0.0, 0.0, [255, 255, 255, 255]);
        let mut verts = vec![0.0f32; 8];
        let mut uvs = vec![0.0f32; 8];
        let mut colors = vec![0.0f32; 16];
        emit_particle_quads(&particles, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, true, &mut verts, &mut uvs, &mut colors);
        // Zero-size quad at TL=(0,0); all 4 verts collapse to particle position with Y flipped.
        for v in 0..4 {
            assert!((verts[v * 2] - 10.0).abs() < 1e-5, "vert {} x: {}", v, verts[v * 2]);
            assert!((verts[v * 2 + 1] - (-20.0)).abs() < 1e-5, "vert {} y: {}", v, verts[v * 2 + 1]);
        }
    }

    #[test]
    fn test_emit_particle_quads_rotation_90deg() {
        // Particle at origin, rotation = 90 deg, scale=1, quad TL=(0,0) size=(10,0).
        let mut particles = vec![0u8; 28];
        write_particle(&mut particles, 0, 0.0, 0.0, 1.0, 1.0, 90.0, 0.0, [255, 255, 255, 255]);
        let mut verts = vec![0.0f32; 8];
        let mut uvs = vec![0.0f32; 8];
        let mut colors = vec![0.0f32; 16];
        emit_particle_quads(&particles, 0.0, 0.0, 10.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, &mut verts, &mut uvs, &mut colors);
        // Rot 90 (CCW) around origin: (10, 0) -> (0, 10). TL=(0,0) stays at (0,0).
        assert!(verts[0].abs() < 1e-5);
        assert!(verts[1].abs() < 1e-5);
        assert!(verts[2].abs() < 1e-5);
        assert!((verts[3] - 10.0).abs() < 1e-5);
    }
}
