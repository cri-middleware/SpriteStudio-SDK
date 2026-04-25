use wide::f32x4;

/// Computes the local vertices for a part, including pivot calculation and deformations.
/// Utilizes SIMD via the `wide` crate for maximum performance.
/// Returns a tuple containing the number of vertices (4 or 5) and the x and y coordinates.
pub fn compute_local_vertices(size: (f32, f32), pivot: (f32, f32), pivot_offset: (f32, f32), flip: (bool, bool), deform: Option<(&[f32; 4], &[f32; 4])>, use_5_vertices: bool) -> (usize, [f32; 5], [f32; 5]) {
    let mut out_x = [0.0; 5];
    let mut out_y = [0.0; 5];

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

    out_x[0..4].copy_from_slice(&vx_arr);
    out_y[0..4].copy_from_slice(&vy_arr);

    // 4. Compute 5th center vertex if requested (Mathematical Optimization: Varignon's theorem)
    let vertex_count = if use_5_vertices {
        out_x[4] = vx_arr.iter().sum::<f32>() * 0.25;
        out_y[4] = vy_arr.iter().sum::<f32>() * 0.25;
        5
    } else {
        4
    };

    (vertex_count, out_x, out_y)
}

#[cfg(test)]
mod tests {
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
        let use_5_vertices = false;

        let (count, x, y) = compute_local_vertices(size, pivot, pivot_offset, flip, deform, use_5_vertices);

        assert_eq!(count, 4);
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

        let (count, x, y) = compute_local_vertices(size, pivot, pivot_offset, flip, None, false);

        assert_eq!(count, 4);
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
        let (_, x1, y1) = compute_local_vertices(size, pivot, pivot_offset, (false, false), None, false);
        assert_arr_approx_eq(&x1, &[0.0, 100.0, 0.0, 100.0], 4);
        assert_arr_approx_eq(&y1, &[0.0, 0.0, -100.0, -100.0], 4);

        // Flip H: pivot effectively moves to top-right
        let (_, x2, y2) = compute_local_vertices(size, pivot, pivot_offset, (true, false), None, false);
        assert_arr_approx_eq(&x2, &[-100.0, 0.0, -100.0, 0.0], 4);
        assert_arr_approx_eq(&y2, &[0.0, 0.0, -100.0, -100.0], 4);

        // Flip V: pivot effectively moves to bottom-left
        let (_, x3, y3) = compute_local_vertices(size, pivot, pivot_offset, (false, true), None, false);
        assert_arr_approx_eq(&x3, &[0.0, 100.0, 0.0, 100.0], 4);
        assert_arr_approx_eq(&y3, &[100.0, 100.0, 0.0, 0.0], 4);
    }

    #[test]
    fn test_deform_addition() {
        let size = (100.0, 100.0);
        let pivot = (0.0, 0.0);
        let deform_x = [10.0, 20.0, 30.0, 40.0];
        let deform_y = [1.0, 2.0, 3.0, 4.0];

        let (count, x, y) = compute_local_vertices(size, pivot, (0.0, 0.0), (false, false), Some((&deform_x, &deform_y)), false);

        assert_eq!(count, 4);
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

        let (count, x, y) = compute_local_vertices(size, pivot, (0.0, 0.0), (false, false), Some((&deform_x, &deform_y)), true);

        assert_eq!(count, 5);

        // Sum of X: (-50+10) + (50-10) + (-50+20) + (50-20) = -40 + 40 - 30 + 30 = 0
        // Average X = 0
        assert_approx_eq(x[4], 0.0);

        // Sum of Y: (50+5) + (50+15) + (-50-5) + (-50-15) = 55 + 65 - 55 - 65 = 0
        // Average Y = 0
        assert_approx_eq(y[4], 0.0);

        // Let's try an asymmetrical deform
        let deform_x2 = [100.0, 0.0, 0.0, 0.0];
        let deform_y2 = [0.0, 100.0, 0.0, 0.0];

        let (_, x2, y2) = compute_local_vertices(size, pivot, (0.0, 0.0), (false, false), Some((&deform_x2, &deform_y2)), true);

        // Sum of X: (-50+100) + 50 - 50 + 50 = 50 + 50 - 50 + 50 = 100. Average = 25.0
        assert_approx_eq(x2[4], 25.0);
        // Sum of Y: 50 + (50+100) - 50 - 50 = 50 + 150 - 50 - 50 = 100. Average = 25.0
        assert_approx_eq(y2[4], 25.0);
    }
}
