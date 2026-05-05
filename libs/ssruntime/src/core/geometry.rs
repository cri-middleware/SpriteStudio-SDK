use crate::core::matrix::SsMatrix4x4;
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
    let (sin_v, cos_v) = f32x4::splat(rot_rad).sin_cos();
    let sin_r = sin_v.to_array()[0];
    let cos_r = cos_v.to_array()[0];

    let ou = center_u + tu;
    let ov = center_v + tv;

    let mut mat = SsMatrix4x4::IDENTITY;
    if !rotated {
        mat.m[0] = cos_r * su_scaled * fx;
        mat.m[1] = sin_r * su_scaled * fx;
        mat.m[4] = -sin_r * sv_scaled * fy;
        mat.m[5] = cos_r * sv_scaled * fy;
    } else {
        mat.m[0] = sin_r * sv_scaled * fx;
        mat.m[1] = -cos_r * sv_scaled * fx;
        mat.m[4] = cos_r * su_scaled * fy;
        mat.m[5] = sin_r * su_scaled * fy;
    }
    mat.m[12] = ou;
    mat.m[13] = ov;

    let x = f32x4::from([-1.0, 1.0, -1.0, 1.0]);
    let y = f32x4::from([-1.0, -1.0, 1.0, 1.0]);

    let (u_simd, v_simd) = mat.transform_v4_2d(x, y);

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
}
