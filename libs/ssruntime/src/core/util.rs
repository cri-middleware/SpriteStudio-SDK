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
