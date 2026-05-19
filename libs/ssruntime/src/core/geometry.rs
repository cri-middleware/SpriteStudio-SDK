use crate::core::SsMatrix4x4;
use wide::{CmpGt, CmpLt, f32x4};

/// Computes the local vertices for a part, including pivot calculation and deformations.
/// Utilizes SIMD via the `wide` crate for maximum performance.
/// Returns a tuple containing the x and y coordinates of the 5 vertices.
pub fn compute_local_vertices(size: (f32, f32), pivot: (f32, f32), pivot_offset: (f32, f32), flip: (bool, bool), deform: Option<(&[f32; 4], &[f32; 4])>) -> ([f32; 5], [f32; 5]) {
    // 1. Pivot Calculation
    let cpx = if flip.0 { 1.0 - (pivot.0 + 0.5) } else { pivot.0 + 0.5 };
    let p_x = (cpx + pivot_offset.0) * size.0;

    let cpy = if flip.1 { 1.0 - (-pivot.1 + 0.5) } else { -pivot.1 + 0.5 };
    let p_y = (cpy - pivot_offset.1) * size.1;

    let sx = -p_x;
    let ex = sx + size.0;
    let sy = p_y;
    let ey = sy - size.1;

    // 2. Base 4 coordinates (Z order: LU, RU, LD, RD)
    let mut vx = f32x4::from([sx, ex, sx, ex]);
    let mut vy = f32x4::from([sy, sy, ey, ey]);

    // 3. Add Deform offsets
    if let Some((dx_arr, dy_arr)) = deform {
        vx += f32x4::from(*dx_arr);
        vy += f32x4::from(*dy_arr);
    }

    let vx_arr: [f32; 4] = vx.to_array();
    let vy_arr: [f32; 4] = vy.to_array();

    // 4. Compute 5th center vertex (Mathematical Optimization: Varignon's theorem)
    let cx = vx.reduce_add() * 0.25;
    let cy = vy.reduce_add() * 0.25;

    ([vx_arr[0], vx_arr[1], vx_arr[2], vx_arr[3], cx], [vy_arr[0], vy_arr[1], vy_arr[2], vy_arr[3], cy])
}

/// Computes the local UV coordinates for a part, including UV animation (translate, rotate, scale) and flipping.
/// Returns a tuple containing the u and v coordinates for the 5 vertices.
pub fn compute_local_uvs(
    uv_rect: (f32, f32, f32, f32),      // (left, top, right, bottom)
    uv_anim: (f32, f32, f32, f32, f32), // (trans_u, trans_v, rot_deg, scale_u, scale_v)
    part_flip: (bool, bool),            // (h_flip, v_flip)
    image_flip: (bool, bool),           // (img_flip_h, img_flip_v)
    rotated: bool,                      // is the cell rotated?
) -> ([f32; 5], [f32; 5]) {
    let (left, top, right, bottom) = uv_rect;
    let (tu, tv, rot_deg, su, sv) = uv_anim;

    let hw = (right - left) * 0.5;
    let hh = (bottom - top) * 0.5;

    let center_u = left + hw;
    let center_v = top + hh;

    let su_scaled = su * if image_flip.0 { -1.0 } else { 1.0 } * hw;
    let sv_scaled = sv * if image_flip.1 { -1.0 } else { 1.0 } * hh;

    let fx = if part_flip.0 { -1.0 } else { 1.0 };
    let fy = if part_flip.1 { -1.0 } else { 1.0 };

    let rot_rad = rot_deg * core::f32::consts::PI / 180.0;
    let (sin_r, cos_r) = crate::core::math::sincosf(rot_rad);

    let ou = center_u + tu;
    let ov = center_v + tv;

    // 2D affine entries laid out as
    //   [ m00  m01 ] [ x ]   [ u - ou ]
    //   [ m10  m11 ] [ y ] = [ v - ov ]
    let (m00, m10, m01, m11) = if !rotated { (cos_r * su_scaled * fx, sin_r * su_scaled * fx, -sin_r * sv_scaled * fy, cos_r * sv_scaled * fy) } else { (sin_r * sv_scaled * fx, -cos_r * sv_scaled * fx, cos_r * su_scaled * fy, sin_r * su_scaled * fy) };

    let x = f32x4::from([-1.0, 1.0, -1.0, 1.0]);
    let y = f32x4::from([-1.0, -1.0, 1.0, 1.0]);

    let u_simd = f32x4::splat(m00).mul_add(x, f32x4::splat(m01).mul_add(y, f32x4::splat(ou)));
    let v_simd = f32x4::splat(m10).mul_add(x, f32x4::splat(m11).mul_add(y, f32x4::splat(ov)));

    let u_arr: [f32; 4] = u_simd.to_array();
    let v_arr: [f32; 4] = v_simd.to_array();

    ([u_arr[0], u_arr[1], u_arr[2], u_arr[3], ou], [v_arr[0], v_arr[1], v_arr[2], v_arr[3], ov])
}

/// Performs a hit test for a point against a rectangle in local space.
/// The point is expected to be already transformed into local space (using inverse matrix).
pub fn is_point_in_rect(local_p: (f32, f32), size: (f32, f32)) -> bool {
    let hw = size.0 * 0.5;
    let hh = size.1 * 0.5;
    local_p.0 >= -hw && local_p.0 <= hw && local_p.1 >= -hh && local_p.1 <= hh
}

/// Performs a hit test for a point against a 4-point quadrilateral in world space.
/// Uses the even-odd rule (ray casting) algorithm and SIMD via the `wide` crate to support both convex and concave quadrilaterals efficiently.
pub fn is_point_in_quad(target_p: (f32, f32), v_x: &[f32; 4], v_y: &[f32; 4]) -> bool {
    let tp_x = f32x4::splat(target_p.0);
    let tp_y = f32x4::splat(target_p.1);

    // Indices for LU(0) -> RU(1) -> RD(3) -> LD(2) loop
    let x0 = f32x4::from([v_x[0], v_x[1], v_x[3], v_x[2]]);
    let y0 = f32x4::from([v_y[0], v_y[1], v_y[3], v_y[2]]);
    let x1 = f32x4::from([v_x[1], v_x[3], v_x[2], v_x[0]]);
    let y1 = f32x4::from([v_y[1], v_y[3], v_y[2], v_y[0]]);

    // Ray casting to the right (+x direction)
    // Condition 1: (y0 > target_p.1) != (y1 > target_p.1)
    let y0_gt = y0.simd_gt(tp_y);
    let y1_gt = y1.simd_gt(tp_y);
    let y_cond = y0_gt ^ y1_gt;

    // Condition 2: target_p.0 < intersect_x
    // intersect_x = x0 + (target_p.1 - y0) * (x1 - x0) / (y1 - y0)
    let dx = x1 - x0;
    let dy = y1 - y0;
    let intersect_x = x0 + (tp_y - y0) * dx / dy;
    let x_cond = tp_x.simd_lt(intersect_x);

    // Combine conditions: both must be true (bitwise AND on mask)
    let inside_mask = y_cond & x_cond;

    // We count how many true values exist in the mask.
    // wide crate stores masks as all 1s (0xFFFFFFFF) for true and 0 for false in f32 format.
    // Converting the bit pattern of each lane into an array allows counting.
    let arr = inside_mask.to_array();
    let mut intersections = 0;
    for item in &arr {
        if item.to_bits() != 0 {
            intersections += 1;
        }
    }

    (intersections % 2) != 0
}

/// Represents a 4x4 grid of vertices and UVs for a Nine-Slice part.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NineSliceGrid {
    pub vertices_x: [f32; 16],
    pub vertices_y: [f32; 16],
    pub uvs_u: [f32; 16],
    pub uvs_v: [f32; 16],
}

/// Computes the 16 grid points for a Nine-Slice part.
/// `size`: (width, height) of the part.
/// `uv_rect`: (left, top, right, bottom) in normalized UV space or pixel space.
/// `margins`: (left, top, right, bottom) margins.
/// `source_size`: (width, height) of the source image/rect for margin validation.
pub fn compute_nineslice_geometry(size: (f32, f32), uv_rect: (f32, f32, f32, f32), margins: (f32, f32, f32, f32), source_size: (f32, f32)) -> NineSliceGrid {
    let (w, h) = size;
    let (l, t, r, b) = uv_rect;
    let (ml, mt, mr, mb) = margins;
    let (sw, sh) = source_size;

    // 1. Calculate Safe Margins (same as SS6-SDK)
    let mut sml = ml;
    let mut smr = mr;
    let diff_x = ml + mr - sw;
    if diff_x > 0.0 {
        let half = diff_x * 0.5;
        sml -= half;
        smr -= diff_x - half;
    }

    let mut smt = mt;
    let mut smb = mb;
    let diff_y = mt + mb - sh;
    if diff_y > 0.0 {
        let half = diff_y * 0.5;
        smt -= half;
        smb -= diff_y - half;
    }

    // 2. Calculate Grid Coordinates
    let hx2 = w * 0.5;
    let hy2 = h * 0.5;

    let vx = [-hx2, -hx2 + sml, hx2 - smr, hx2];
    let vy = [hy2, hy2 - smt, -hy2 + smb, -hy2]; // Y down in SS7-SDK default? No, let's stick to SS6-SDK logic (Y up internally)

    let uv_u = [l, l + sml / sw * (r - l), r - smr / sw * (r - l), r];
    let uv_v = [t, t + smt / sh * (b - t), b - smb / sh * (b - t), b];

    NineSliceGrid {
        vertices_x: unsafe { core::mem::transmute::<[f32x4; 4], [f32; 16]>([f32x4::from(vx); 4]) },
        vertices_y: unsafe { core::mem::transmute::<[f32x4; 4], [f32; 16]>([f32x4::splat(vy[0]), f32x4::splat(vy[1]), f32x4::splat(vy[2]), f32x4::splat(vy[3])]) },
        uvs_u: unsafe { core::mem::transmute::<[f32x4; 4], [f32; 16]>([f32x4::from(uv_u); 4]) },
        uvs_v: unsafe { core::mem::transmute::<[f32x4; 4], [f32; 16]>([f32x4::splat(uv_v[0]), f32x4::splat(uv_v[1]), f32x4::splat(uv_v[2]), f32x4::splat(uv_v[3])]) },
    }
}

/// Primitive shape types supported by Shape Parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ShapeType {
    Rectangle = 0,
    Triangle = 1,
    Star = 2,
    Arrow = 3,
}

/// Computes local vertices for a primitive shape.
/// Returns the number of vertices and the coordinates.
pub fn compute_shape_vertices(shape_type: ShapeType, size: (f32, f32)) -> (usize, [f32; 16], [f32; 16]) {
    let (w, h) = size;
    let hw = w * 0.5;
    let hh = h * 0.5;

    let mut out_x = [0.0; 16];
    let mut out_y = [0.0; 16];
    let count;

    match shape_type {
        ShapeType::Rectangle => {
            out_x[0..4].copy_from_slice(&[-hw, hw, -hw, hw]);
            out_y[0..4].copy_from_slice(&[hh, hh, -hh, -hh]);
            count = 4;
        }
        ShapeType::Triangle => {
            // Center + 3 points
            out_x[0] = 0.0;
            out_y[0] = 0.0;

            // Angles: 0, 120, 240 degrees in radians
            const DEG120_RAD: f32 = 120.0 * core::f32::consts::PI / 180.0;
            let rads = f32x4::from([0.0, DEG120_RAD, DEG120_RAD * 2.0, 0.0]);
            let (s, c) = rads.sin_cos();

            let vx = s * hw;
            let vy = c * hh;

            out_x[1..5].copy_from_slice(&vx.to_array());
            out_y[1..5].copy_from_slice(&vy.to_array());

            out_x[4] = out_x[1]; // Close the fan
            out_y[4] = out_y[1];
            count = 5;
        }
        ShapeType::Star => {
            // Center + 10 outer points + closing vertex (so a TRIANGLE_FAN consumer
            // gets all 10 wedges).
            out_x[0] = 0.0;
            out_y[0] = 0.0;

            let hw_simd = f32x4::splat(hw);
            let hh_simd = f32x4::splat(hh);
            let r_scale = f32x4::from([1.0, 0.382, 1.0, 0.382]);

            for i in (0..10).step_by(4) {
                let idx = f32x4::from([i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32]);
                let rad = idx * (36.0 * core::f32::consts::PI / 180.0);
                let (s, c) = rad.sin_cos();

                let vx = s * hw_simd * r_scale;
                let vy = c * hh_simd * r_scale;

                let end = (i + 1 + 4).min(11);
                let len = end - (i + 1);
                let vx_arr = vx.to_array();
                let vy_arr = vy.to_array();
                out_x[i + 1..end].copy_from_slice(&vx_arr[..len]);
                out_y[i + 1..end].copy_from_slice(&vy_arr[..len]);
            }
            out_x[11] = out_x[1];
            out_y[11] = out_y[1];
            count = 12;
        }
        ShapeType::Arrow => {
            let sx = hw * 0.5;
            let sy = 0.0;
            // 7 points
            out_x[0..7].copy_from_slice(&[-sx / 2.0, sx / 2.0, sx / 2.0, hw, 0.0, -hw, -sx / 2.0]);
            out_y[0..7].copy_from_slice(&[-hh, -hh, sy, sy, hh, sy, sy]);
            count = 7;
        }
    }

    (count, out_x, out_y)
}

/// Optional per-frame deform delta passed to [`compute_skinned_mesh_vertices`].
///
/// Tuple layout: `(changed_vertex_indices, delta_x, delta_y, part_world_matrix)`.
/// `changed_vertex_indices[k]` selects which output vertex receives `(delta_x[k], delta_y[k])`.
pub type SkinnedMeshDeform<'a> = (&'a [i16], &'a [f32], &'a [f32], &'a SsMatrix4x4);

/// Computes skinned (bone-weighted) mesh vertex positions in world space.
///
/// For each vertex `v`:
/// 1. **Skeletal** — accumulate `Σ bone_world_matrices[bone_indices[i]] · (coords_x[i], coords_y[i]) · weights[i]`
///    over the CSR influence slice `influence_offsets[v]..influence_offsets[v + 1]`.
///    Translation IS applied (MultiplyPoint3x4 — m[12], m[13] terms).
/// 2. **Deform delta** (optional) — for each entry in `deform.0`, add
///    `part_world_mat · (delta_x, delta_y)` (rotation/scale only — MultiplyVector).
///
/// Bone indices are expected to be pre-resolved to **part indices** by ssconverter;
/// no further `TableIDPartsBone` lookup is performed at runtime.
///
/// Layout: CSR for `influence_offsets`, SoA for `weights`/`coords_x`/`coords_y`.
/// The common case (≤4 influences per vertex) is computed with `f32x4`: lanes are
/// gathered into stack-local arrays, multiplied as SIMD, then reduced. The rare
/// >4-influence path falls back to a scalar accumulator.
///
/// `flip_y` negates the bind `coords_y`. SSAB-side bind coordinates are authored
/// Y-up; set `flip_y` when the bone world matrices are Y-down so the bind-pose
/// invariant (each influence resolves to the same rest position) holds.
///
/// Skins exactly `out_x.len()` vertices from the front of the binding. The SS7
/// meshList appends 4 unbound placeholder vertices (the cell-quad corners) after
/// the rendered mesh, so `influence_offsets` may be longer than `out_x.len() + 1`;
/// the trailing entries are simply never read.
pub fn compute_skinned_mesh_vertices(
    influence_offsets: &[u32],
    bone_indices: &[u16],
    weights: &[f32],
    coords_x: &[f32],
    coords_y: &[f32],
    bone_world_matrices: &[SsMatrix4x4],
    deform: Option<SkinnedMeshDeform<'_>>,
    flip_y: bool,
    out_x: &mut [f32],
    out_y: &mut [f32],
) {
    let num_vertices = out_x.len();
    debug_assert_eq!(out_y.len(), num_vertices);
    debug_assert!(influence_offsets.len() > num_vertices);

    // Bind coords are authored Y-up; negate for a Y-down target. Resolved once
    // and applied as a sign multiply (SIMD in the fast path) — no per-influence
    // branch.
    let y_sign = if flip_y { -1.0f32 } else { 1.0f32 };
    let y_sign_v = f32x4::from([y_sign; 4]);

    // 1. Skeletal: per-vertex weighted bone transform sum (MultiplyPoint3x4 — translation included).
    for v in 0..num_vertices {
        let start = influence_offsets[v] as usize;
        let end = influence_offsets[v + 1] as usize;
        let count = end - start;

        if count <= 4 {
            // Fast path: gather up to 4 influences into f32x4 lanes (zero-padded).
            let mut w_lanes = [0.0f32; 4];
            let mut cx_lanes = [0.0f32; 4];
            let mut cy_lanes = [0.0f32; 4];
            let mut m0_lanes = [0.0f32; 4];
            let mut m4_lanes = [0.0f32; 4];
            let mut m12_lanes = [0.0f32; 4];
            let mut m1_lanes = [0.0f32; 4];
            let mut m5_lanes = [0.0f32; 4];
            let mut m13_lanes = [0.0f32; 4];

            for k in 0..count {
                let i = start + k;
                w_lanes[k] = weights[i];
                cx_lanes[k] = coords_x[i];
                cy_lanes[k] = coords_y[i];
                let mat = &bone_world_matrices[bone_indices[i] as usize];
                m0_lanes[k] = mat.m[0];
                m4_lanes[k] = mat.m[4];
                m12_lanes[k] = mat.m[12];
                m1_lanes[k] = mat.m[1];
                m5_lanes[k] = mat.m[5];
                m13_lanes[k] = mat.m[13];
            }

            let w = f32x4::from(w_lanes);
            let cx = f32x4::from(cx_lanes);
            let cy = f32x4::from(cy_lanes) * y_sign_v;

            let px = f32x4::from(m0_lanes) * cx + f32x4::from(m4_lanes) * cy + f32x4::from(m12_lanes);
            let py = f32x4::from(m1_lanes) * cx + f32x4::from(m5_lanes) * cy + f32x4::from(m13_lanes);

            out_x[v] = (px * w).reduce_add();
            out_y[v] = (py * w).reduce_add();
        } else {
            // Slow path: >4 influences (rare in SS data). Scalar accumulator.
            let mut acc_x = 0.0f32;
            let mut acc_y = 0.0f32;
            for i in start..end {
                let w = weights[i];
                let cx = coords_x[i];
                let cy = coords_y[i] * y_sign;
                let mat = &bone_world_matrices[bone_indices[i] as usize];
                let px = mat.m[0] * cx + mat.m[4] * cy + mat.m[12];
                let py = mat.m[1] * cx + mat.m[5] * cy + mat.m[13];
                acc_x += px * w;
                acc_y += py * w;
            }
            out_x[v] = acc_x;
            out_y[v] = acc_y;
        }
    }

    // 2. Deform delta: MultiplyVector — rotation/scale only, NO translation.
    //    Kept scalar: writes are randomly-indexed (scatter), so SIMD wouldn't help.
    if let Some((indices, dx, dy, part_mat)) = deform {
        debug_assert_eq!(dx.len(), indices.len());
        debug_assert_eq!(dy.len(), indices.len());

        for k in 0..indices.len() {
            let v = indices[k] as usize;
            let tx = part_mat.m[0] * dx[k] + part_mat.m[4] * dy[k];
            let ty = part_mat.m[1] * dx[k] + part_mat.m[5] * dy[k];
            out_x[v] += tx;
            out_y[v] += ty;
        }
    }
}

/// Rigidly transforms a Deform-only mesh's rest vertices by the part's world
/// matrix — `out = W · rest` for every vertex. SoA layout in and out,
/// computed four vertices at a time with `f32x4`; the contiguous SoA loads
/// keep the input gather vectorizable. The per-frame deform delta is a sparse
/// scatter and is applied separately (scalar) by the caller.
///
/// `flip_rest_y` negates the rest Y. The rest comes from the cell's
/// `table_coordinate` (Y-down native), so it flips for a Y-up target — the
/// opposite of the Y-up-native skinned bind coordinates.
pub fn compute_rigid_mesh_vertices(rest_x: &[f32], rest_y: &[f32], world: &SsMatrix4x4, flip_rest_y: bool, out_x: &mut [f32], out_y: &mut [f32]) {
    let count = out_x.len();
    debug_assert_eq!(out_y.len(), count);
    debug_assert_eq!(rest_x.len(), count);
    debug_assert_eq!(rest_y.len(), count);

    let m = &world.m;
    let m0 = f32x4::from([m[0]; 4]);
    let m1 = f32x4::from([m[1]; 4]);
    let m4 = f32x4::from([m[4]; 4]);
    let m5 = f32x4::from([m[5]; 4]);
    let m12 = f32x4::from([m[12]; 4]);
    let m13 = f32x4::from([m[13]; 4]);
    // Resolved once; applied as a sign multiply (f32x4 in the SIMD body, scalar in the tail).
    let y_sign = if flip_rest_y { -1.0f32 } else { 1.0f32 };
    let y_sign_v = f32x4::from([y_sign; 4]);

    // SIMD body: 4 vertices per iteration. `rest_*` are contiguous SoA, so
    // each lane load is a plain 128-bit read; the output buffers are SoA too.
    let chunks = count / 4;
    for c in 0..chunks {
        let b = c * 4;
        let rx = f32x4::from([rest_x[b], rest_x[b + 1], rest_x[b + 2], rest_x[b + 3]]);
        let ry = f32x4::from([rest_y[b], rest_y[b + 1], rest_y[b + 2], rest_y[b + 3]]) * y_sign_v;
        let ox = m0 * rx + m4 * ry + m12;
        let oy = m1 * rx + m5 * ry + m13;
        out_x[b..b + 4].copy_from_slice(&ox.to_array());
        out_y[b..b + 4].copy_from_slice(&oy.to_array());
    }

    // Scalar tail: the final < 4 vertices.
    for v in (chunks * 4)..count {
        let rx = rest_x[v];
        let ry = rest_y[v] * y_sign;
        out_x[v] = m[0] * rx + m[4] * ry + m[12];
        out_y[v] = m[1] * rx + m[5] * ry + m[13];
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_is_point_in_rect() {
        assert!(is_point_in_rect((0.0, 0.0), (100.0, 100.0)));
        assert!(is_point_in_rect((49.0, -49.0), (100.0, 100.0)));
        assert!(!is_point_in_rect((51.0, 0.0), (100.0, 100.0)));
    }

    #[test]
    fn test_is_point_in_quad() {
        // Simple 100x100 square vertices in Z-order: LU, RU, LD, RD
        let vx = [-50.0, 50.0, -50.0, 50.0];
        let vy = [50.0, 50.0, -50.0, -50.0];

        assert!(is_point_in_quad((0.0, 0.0), &vx, &vy));
        assert!(is_point_in_quad((45.0, 45.0), &vx, &vy));
        assert!(!is_point_in_quad((55.0, 0.0), &vx, &vy));

        // Deformed trapezoid
        let vx2 = [-20.0, 20.0, -50.0, 50.0];
        let vy2 = [50.0, 50.0, -50.0, -50.0];
        assert!(is_point_in_quad((0.0, 0.0), &vx2, &vy2));
        assert!(is_point_in_quad((15.0, 45.0), &vx2, &vy2));
        assert!(!is_point_in_quad((25.0, 45.0), &vx2, &vy2));
    }

    #[test]
    fn test_compute_local_uvs_basic() {
        let rect = (0.0, 0.0, 1.0, 1.0);
        let anim = (0.0, 0.0, 0.0, 1.0, 1.0);
        let (u, v) = compute_local_uvs(rect, anim, (false, false), (false, false), false);
        assert_arr_approx_eq(&u, &[0.0, 1.0, 0.0, 1.0], 4);
        assert_arr_approx_eq(&v, &[0.0, 0.0, 1.0, 1.0], 4);
    }

    #[test]
    fn test_compute_local_uvs_part_flip() {
        let rect = (0.0, 0.0, 1.0, 1.0);
        let anim = (0.0, 0.0, 0.0, 1.0, 1.0);
        let (u, v) = compute_local_uvs(rect, anim, (true, false), (false, false), false);
        assert_arr_approx_eq(&u, &[1.0, 0.0, 1.0, 0.0], 4);
        assert_arr_approx_eq(&v, &[0.0, 0.0, 1.0, 1.0], 4);
    }

    use super::*;

    // Float comparison helper
    fn assert_approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "{} != {}", a, b);
    }

    fn assert_arr_approx_eq(a: &[f32], b: &[f32], count: usize) {
        for i in 0..count {
            assert_approx_eq(a[i], b[i]);
        }
    }

    #[test]
    fn test_basic_4_vertices() {
        // Simple 100x100 square, center pivot
        let size = (100.0, 100.0);
        let pivot = (0.0, 0.0);
        let pivot_offset = (0.0, 0.0);
        let flip = (false, false);
        let deform = None;

        let (x, y) = compute_local_vertices(size, pivot, pivot_offset, flip, deform);
        // Expecting LU: (-50, 50), RU: (50, 50), LD: (-50, -50), RD: (50, -50)
        assert_arr_approx_eq(&x, &[-50.0, 50.0, -50.0, 50.0], 4);
        assert_arr_approx_eq(&y, &[50.0, 50.0, -50.0, -50.0], 4);
    }

    #[test]
    fn test_pivot_offset() {
        // 100x100 square, center pivot, offset by 0.5 (moves pivot to right/bottom)
        let size = (100.0, 100.0);
        let pivot = (0.0, 0.0);
        let pivot_offset = (0.5, -0.5); // Move pivot right by 50, down by 50
        let flip = (false, false);

        let (x, y) = compute_local_vertices(size, pivot, pivot_offset, flip, None);
        // Expecting LU: (-100, 100), RU: (0, 100), LD: (-100, 0), RD: (0, 0)
        assert_arr_approx_eq(&x, &[-100.0, 0.0, -100.0, 0.0], 4);
        assert_arr_approx_eq(&y, &[100.0, 100.0, 0.0, 0.0], 4);
    }

    #[test]
    fn test_flip_h_v() {
        // 100x100 square, pivot at top-left (-0.5, 0.5)
        let size = (100.0, 100.0);
        let pivot = (-0.5, 0.5);
        let pivot_offset = (0.0, 0.0);

        // No flip: LU should be (0, 0), RU: (100, 0), LD: (0, -100), RD: (100, -100)
        let (x1, y1) = compute_local_vertices(size, pivot, pivot_offset, (false, false), None);
        assert_arr_approx_eq(&x1, &[0.0, 100.0, 0.0, 100.0], 4);
        assert_arr_approx_eq(&y1, &[0.0, 0.0, -100.0, -100.0], 4);

        // Flip H: pivot effectively moves to top-right
        let (x2, y2) = compute_local_vertices(size, pivot, pivot_offset, (true, false), None);
        assert_arr_approx_eq(&x2, &[-100.0, 0.0, -100.0, 0.0], 4);
        assert_arr_approx_eq(&y2, &[0.0, 0.0, -100.0, -100.0], 4);

        // Flip V: pivot effectively moves to bottom-left
        let (x3, y3) = compute_local_vertices(size, pivot, pivot_offset, (false, true), None);
        assert_arr_approx_eq(&x3, &[0.0, 100.0, 0.0, 100.0], 4);
        assert_arr_approx_eq(&y3, &[100.0, 100.0, 0.0, 0.0], 4);
    }

    #[test]
    fn test_deform_addition() {
        let size = (100.0, 100.0);
        let pivot = (0.0, 0.0);
        let deform_x = [10.0, 20.0, 30.0, 40.0];
        let deform_y = [1.0, 2.0, 3.0, 4.0];

        let (x, y) = compute_local_vertices(size, pivot, (0.0, 0.0), (false, false), Some((&deform_x, &deform_y)));
        // Original X: [-50, 50, -50, 50] -> [+10, +20, +30, +40]
        assert_arr_approx_eq(&x, &[-40.0, 70.0, -20.0, 90.0], 4);
        // Original Y: [50, 50, -50, -50] -> [+1, +2, +3, +4]
        assert_arr_approx_eq(&y, &[51.0, 52.0, -47.0, -46.0], 4);
    }

    #[test]
    fn test_5_vertices_center_calculation() {
        let size = (100.0, 100.0);
        let pivot = (0.0, 0.0);

        // A complex deform that makes it an irregular quadrilateral
        let deform_x = [10.0, -10.0, 20.0, -20.0];
        let deform_y = [5.0, 15.0, -5.0, -15.0];

        let (x, y) = compute_local_vertices(size, pivot, (0.0, 0.0), (false, false), Some((&deform_x, &deform_y)));

        // Sum of X: (-50+10) + (50-10) + (-50+20) + (50-20) = -40 + 40 - 30 + 30 = 0
        // Average X = 0
        assert_approx_eq(x[4], 0.0);

        // Sum of Y: (50+5) + (50+15) + (-50-5) + (-50-15) = 55 + 65 - 55 - 65 = 0
        // Average Y = 0
        assert_approx_eq(y[4], 0.0);

        // Let's try an asymmetrical deform
        let deform_x2 = [100.0, 0.0, 0.0, 0.0];
        let deform_y2 = [0.0, 100.0, 0.0, 0.0];

        let (x2, y2) = compute_local_vertices(size, pivot, (0.0, 0.0), (false, false), Some((&deform_x2, &deform_y2)));

        // Sum of X: (-50+100) + 50 - 50 + 50 = 50 + 50 - 50 + 50 = 100. Average = 25.0
        assert_approx_eq(x2[4], 25.0);
        // Sum of Y: 50 + (50+100) - 50 - 50 = 50 + 150 - 50 - 50 = 100. Average = 25.0
        assert_approx_eq(y2[4], 25.0);
    }

    #[test]
    fn test_vertices_complex_flip_offset() {
        // 100x100 square, pivot at bottom-right (0.5, -0.5), offset by (0.5, 0.5)
        // effectively pivot is at (1.0, 0.0) relative to center, which is right edge center.
        let size = (100.0, 100.0);
        let pivot = (0.5, -0.5);
        let pivot_offset = (0.5, 0.5);
        let flip = (true, true); // Double flip

        let (x, y) = compute_local_vertices(size, pivot, pivot_offset, flip, None);

        // With H-Flip: pivot_x becomes 1.0 - (0.5 + 0.5) = 0.0
        // With V-Flip: pivot_y becomes 1.0 - (0.5 + 0.5) = 0.0
        // Resulting p_x = (0.0 + 0.5) * 100 = 50
        // Resulting p_y = (0.0 - 0.5) * 100 = -50
        // sx = -50, ex = 50, sy = -50, ey = -150
        assert_approx_eq(x[0], -50.0); // LU
        assert_approx_eq(y[0], -50.0);
    }

    #[test]
    fn test_uvs_full_transform() {
        let rect = (0.0, 0.0, 1.0, 1.0);
        // Translate 0.1, Rotate 90deg, Scale 2.0
        let anim = (0.1, 0.1, 90.0, 2.0, 2.0);
        let (u, v) = compute_local_uvs(rect, anim, (false, false), (false, false), false);

        // Center is (0.5, 0.5)
        // LU (0, 0) relative to center is (-0.5, -0.5)
        // Scale 2.0 -> (-1.0, -1.0)
        // Rotate 90deg -> (1.0, -1.0)  [x' = -y, y' = x]
        // Back to origin (0.5, 0.5) -> (1.5, -0.5)
        // Translate 0.1 -> (1.6, -0.4)
        assert_approx_eq(u[0], 1.6);
        assert_approx_eq(v[0], -0.4);
    }

    #[test]
    fn test_uvs_rotated_cell() {
        let rect = (0.1, 0.1, 0.5, 0.4); // 0.4x0.3 rect
        let anim = (0.0, 0.0, 0.0, 1.0, 1.0);
        let (u, v) = compute_local_uvs(rect, anim, (false, false), (false, false), true);

        // In rotated mode, LU corresponds to (left, bottom) of the rect
        assert_approx_eq(u[0], 0.1);
        assert_approx_eq(v[0], 0.4);
        // RU corresponds to (left, top)
        assert_approx_eq(u[1], 0.1);
        assert_approx_eq(v[1], 0.1);
    }

    #[test]
    fn test_hit_test_edge_cases() {
        let vx = [0.0, 10.0, 0.0, 10.0];
        let vy = [10.0, 10.0, 0.0, 0.0];

        // Inside
        assert!(is_point_in_quad((5.0, 5.0), &vx, &vy));
        // Outside
        assert!(!is_point_in_quad((11.0, 5.0), &vx, &vy));
        // Exactly on edge (should be true with our epsilon)
        assert!(is_point_in_quad((0.0, 5.0), &vx, &vy));
        // Exactly on vertex
        assert!(is_point_in_quad((0.0, 0.0), &vx, &vy));
    }

    #[test]
    fn test_hit_test_concave_quad() {
        // Vertices forming a true concave shape (boomerang style)
        // LU: (0, 10), RU: (2, 2), LD: (0, 0), RD: (10, 0)
        // This makes RU folded inwards.
        let vx = [0.0, 2.0, 0.0, 10.0];
        let vy = [10.0, 2.0, 0.0, 0.0];

        // Inside the top arm
        assert!(is_point_in_quad((1.0, 5.0), &vx, &vy));
        // Inside the bottom arm
        assert!(is_point_in_quad((5.0, 1.0), &vx, &vy));

        // Outside, in the concave notch (right side of the fold)
        assert!(!is_point_in_quad((5.0, 5.0), &vx, &vy));
        // Outside, entirely to the right
        assert!(!is_point_in_quad((11.0, 5.0), &vx, &vy));
    }

    #[test]
    fn test_compute_nineslice_geometry() {
        let size = (100.0, 100.0);
        let uv_rect = (0.0, 0.0, 1.0, 1.0);
        let margins = (10.0, 10.0, 10.0, 10.0);
        let source_size = (100.0, 100.0);

        let grid = compute_nineslice_geometry(size, uv_rect, margins, source_size);

        // Check top-left corner vertex
        assert_approx_eq(grid.vertices_x[0], -50.0);
        assert_approx_eq(grid.vertices_y[0], 50.0);
        // Check grid vertex (1,1)
        assert_approx_eq(grid.vertices_x[5], -40.0);
        assert_approx_eq(grid.vertices_y[5], 40.0);

        // Check UVs
        assert_approx_eq(grid.uvs_u[0], 0.0);
        assert_approx_eq(grid.uvs_v[0], 0.0);
        assert_approx_eq(grid.uvs_u[5], 0.1);
        assert_approx_eq(grid.uvs_v[5], 0.1);
    }

    #[test]
    fn test_compute_shape_vertices_rectangle() {
        let (count, x, y) = compute_shape_vertices(ShapeType::Rectangle, (100.0, 200.0));
        assert_eq!(count, 4);
        assert_approx_eq(x[0], -50.0);
        assert_approx_eq(y[0], 100.0);
    }

    #[test]
    fn test_compute_shape_vertices_star_closes_fan() {
        let (count, x, y) = compute_shape_vertices(ShapeType::Star, (100.0, 200.0));
        assert_eq!(count, 12);
        assert_approx_eq(x[0], 0.0);
        assert_approx_eq(y[0], 0.0);
        assert_approx_eq(x[11], x[1]);
        assert_approx_eq(y[11], y[1]);
    }

    #[test]
    fn test_skinned_mesh_identity_single_influence() {
        // Two vertices, each with one influence to bone 0 (identity), weight 1.0 →
        // output should equal the bind-pose coordinates.
        let offsets = [0u32, 1, 2];
        let bone_indices = [0u16, 0];
        let weights = [1.0f32, 1.0];
        let coords_x = [3.0f32, -4.0];
        let coords_y = [5.0f32, 7.0];
        let bones = [SsMatrix4x4::IDENTITY];

        let mut out_x = [0.0f32; 2];
        let mut out_y = [0.0f32; 2];

        compute_skinned_mesh_vertices(&offsets, &bone_indices, &weights, &coords_x, &coords_y, &bones, None, false, &mut out_x, &mut out_y);

        assert_arr_approx_eq(&out_x, &[3.0, -4.0], 2);
        assert_arr_approx_eq(&out_y, &[5.0, 7.0], 2);
    }

    #[test]
    fn test_skinned_mesh_translation_applied() {
        // MultiplyPoint3x4 must include translation: bone translates (10, 20),
        // bind coord (3, 5) → output (13, 25).
        let offsets = [0u32, 1];
        let bone_indices = [0u16];
        let weights = [1.0f32];
        let coords_x = [3.0f32];
        let coords_y = [5.0f32];
        let bones = [SsMatrix4x4::create_translation(10.0, 20.0, 0.0)];

        let mut out_x = [0.0f32];
        let mut out_y = [0.0f32];

        compute_skinned_mesh_vertices(&offsets, &bone_indices, &weights, &coords_x, &coords_y, &bones, None, false, &mut out_x, &mut out_y);

        assert_arr_approx_eq(&out_x, &[13.0], 1);
        assert_arr_approx_eq(&out_y, &[25.0], 1);
    }

    #[test]
    fn test_skinned_mesh_two_bone_blend() {
        // One vertex weighted 50/50 between identity-bone and translated-bone,
        // both viewing bind coord (0, 0) → output (50, 0).
        let offsets = [0u32, 2];
        let bone_indices = [0u16, 1];
        let weights = [0.5f32, 0.5];
        let coords_x = [0.0f32, 0.0];
        let coords_y = [0.0f32, 0.0];
        let bones = [SsMatrix4x4::IDENTITY, SsMatrix4x4::create_translation(100.0, 0.0, 0.0)];

        let mut out_x = [0.0f32];
        let mut out_y = [0.0f32];

        compute_skinned_mesh_vertices(&offsets, &bone_indices, &weights, &coords_x, &coords_y, &bones, None, false, &mut out_x, &mut out_y);

        assert_arr_approx_eq(&out_x, &[50.0], 1);
        assert_arr_approx_eq(&out_y, &[0.0], 1);
    }

    #[test]
    fn test_skinned_mesh_rotation() {
        // 90deg Z rotation applied to bone, bind coord (1, 0) → output (0, 1).
        let offsets = [0u32, 1];
        let bone_indices = [0u16];
        let weights = [1.0f32];
        let coords_x = [1.0f32];
        let coords_y = [0.0f32];
        let bones = [SsMatrix4x4::create_rotation_z(core::f32::consts::FRAC_PI_2)];

        let mut out_x = [0.0f32];
        let mut out_y = [0.0f32];

        compute_skinned_mesh_vertices(&offsets, &bone_indices, &weights, &coords_x, &coords_y, &bones, None, false, &mut out_x, &mut out_y);

        assert_arr_approx_eq(&out_x, &[0.0], 1);
        assert_arr_approx_eq(&out_y, &[1.0], 1);
    }

    #[test]
    fn test_skinned_mesh_deform_delta_ignores_translation() {
        // Verifies deform uses MultiplyVector (no translation).
        // part_mat = T(1000,1000) * R(90°), delta = (1, 0) →
        // skeletal yields (0, 0) (identity bone, zero bind), deform adds R·(1,0) = (0, 1).
        let offsets = [0u32, 1];
        let bone_indices = [0u16];
        let weights = [1.0f32];
        let coords_x = [0.0f32];
        let coords_y = [0.0f32];
        let bones = [SsMatrix4x4::IDENTITY];

        let rot = SsMatrix4x4::create_rotation_z(core::f32::consts::FRAC_PI_2);
        let trans = SsMatrix4x4::create_translation(1000.0, 1000.0, 0.0);
        let part_mat = trans * rot;

        let indices = [0i16];
        let dx = [1.0f32];
        let dy = [0.0f32];

        let mut out_x = [0.0f32];
        let mut out_y = [0.0f32];

        compute_skinned_mesh_vertices(&offsets, &bone_indices, &weights, &coords_x, &coords_y, &bones, Some((&indices, &dx, &dy, &part_mat)), false, &mut out_x, &mut out_y);

        assert_arr_approx_eq(&out_x, &[0.0], 1);
        assert_arr_approx_eq(&out_y, &[1.0], 1);
    }

    #[test]
    fn test_skinned_mesh_partial_deform() {
        // Three vertices, only vertex 1 receives a deform delta;
        // vertices 0 and 2 must remain at their skeletal-only positions.
        let offsets = [0u32, 1, 2, 3];
        let bone_indices = [0u16; 3];
        let weights = [1.0f32; 3];
        let coords_x = [1.0f32, 2.0, 3.0];
        let coords_y = [10.0f32, 20.0, 30.0];
        let bones = [SsMatrix4x4::IDENTITY];

        let indices = [1i16];
        let dx = [100.0f32];
        let dy = [200.0f32];
        let part_mat = SsMatrix4x4::IDENTITY;

        let mut out_x = [0.0f32; 3];
        let mut out_y = [0.0f32; 3];

        compute_skinned_mesh_vertices(&offsets, &bone_indices, &weights, &coords_x, &coords_y, &bones, Some((&indices, &dx, &dy, &part_mat)), false, &mut out_x, &mut out_y);

        assert_arr_approx_eq(&out_x, &[1.0, 102.0, 3.0], 3);
        assert_arr_approx_eq(&out_y, &[10.0, 220.0, 30.0], 3);
    }

    #[test]
    fn test_rigid_mesh_vertices_simd_chunk_and_tail() {
        // 6 vertices = one 4-wide SIMD chunk + a 2-vertex scalar tail.
        let rest_x = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let rest_y = [10.0f32, 20.0, 30.0, 40.0, 50.0, 60.0];
        // 90° rotation: W · (x, y) = (-y, x).
        let world = SsMatrix4x4::create_rotation_z(core::f32::consts::FRAC_PI_2);
        let mut out_x = [0.0f32; 6];
        let mut out_y = [0.0f32; 6];

        compute_rigid_mesh_vertices(&rest_x, &rest_y, &world, false, &mut out_x, &mut out_y);
        assert_arr_approx_eq(&out_x, &[-10.0, -20.0, -30.0, -40.0, -50.0, -60.0], 6);
        assert_arr_approx_eq(&out_y, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 6);

        // flip_rest_y negates rest Y before the transform: out_x = -(-y) = y.
        compute_rigid_mesh_vertices(&rest_x, &rest_y, &world, true, &mut out_x, &mut out_y);
        assert_arr_approx_eq(&out_x, &[10.0, 20.0, 30.0, 40.0, 50.0, 60.0], 6);
        assert_arr_approx_eq(&out_y, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 6);
    }

    #[test]
    fn test_skinned_mesh_five_influences_scalar_fallback() {
        // Exercises the >4-influence scalar path. Five identity bones, equal weights,
        // bind coord (10, 20) seen by every influence → output equals (10, 20).
        let offsets = [0u32, 5];
        let bone_indices = [0u16; 5];
        let weights = [0.2f32; 5];
        let coords_x = [10.0f32; 5];
        let coords_y = [20.0f32; 5];
        let bones = [SsMatrix4x4::IDENTITY];

        let mut out_x = [0.0f32];
        let mut out_y = [0.0f32];

        compute_skinned_mesh_vertices(&offsets, &bone_indices, &weights, &coords_x, &coords_y, &bones, None, false, &mut out_x, &mut out_y);

        assert_arr_approx_eq(&out_x, &[10.0], 1);
        assert_arr_approx_eq(&out_y, &[20.0], 1);
    }
}
