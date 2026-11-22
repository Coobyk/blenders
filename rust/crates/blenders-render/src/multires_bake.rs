//! Port of `intern/multires_bake.cc` — subdiv-grid
//! sampling + face-center attr calc + TBN matrices.

/// `Grid<T>` bilinear sampler (verbatim — uv×(side-1)
/// clamp + bilinear quad).
pub fn grid_sample_f32(data: &[f32], side_size: i32, uv: [f32; 2]) -> f32 {
    let xy = [uv[0] * (side_size - 1) as f32, uv[1] * (side_size - 1) as f32];
    let x0 = xy[0] as i32;
    let x1 = if x0 >= side_size - 1 { side_size - 1 } else { x0 + 1 };
    let y0 = xy[1] as i32;
    let y1 = if y0 >= side_size - 1 { side_size - 1 } else { y0 + 1 };
    let u = xy[0] - x0 as f32;
    let v = xy[1] - y0 as f32;
    let get = |x: i32, y: i32| data[(y as i64 * side_size as i64 + x as i64) as usize];
    let a = get(x0, y0);
    let b = get(x1, y0);
    let c = get(x1, y1);
    let d = get(x0, y1);
    a + (b - a) * u + (d - a) * v + (a - b - d + c) * u * v
}
/// `Grid<float3>` variant.
pub fn grid_sample_f3(data: &[[f32; 3]], side_size: i32, uv: [f32; 2]) -> [f32; 3] {
    let xy = [uv[0] * (side_size - 1) as f32, uv[1] * (side_size - 1) as f32];
    let x0 = xy[0] as i32;
    let x1 = if x0 >= side_size - 1 { side_size - 1 } else { x0 + 1 };
    let y0 = xy[1] as i32;
    let y1 = if y0 >= side_size - 1 { side_size - 1 } else { y0 + 1 };
    let u = xy[0] - x0 as f32;
    let v = xy[1] - y0 as f32;
    let get = |x: i32, y: i32| data[(y as i64 * side_size as i64 + x as i64) as usize];
    let a = get(x0, y0);
    let b = get(x1, y0);
    let c = get(x1, y1);
    let d = get(x0, y1);
    [
        a[0] + (b[0] - a[0]) * u + (d[0] - a[0]) * v + (a[0] - b[0] - d[0] + c[0]) * u * v,
        a[1] + (b[1] - a[1]) * u + (d[1] - a[1]) * v + (a[1] - b[1] - d[1] + c[1]) * u * v,
        a[2] + (b[2] - a[2]) * u + (d[2] - a[2]) * v + (a[2] - b[2] - d[2] + c[2]) * u * v,
    ]
}

/// `face_center_tex_uv_calc`/`face_center_smooth_normal_calc`
////`face_center_uv_tangent_calc` (verbatim — mean over
/// face corners).
pub fn face_center_mean_f2(attr: &[[f32; 2]], face_corners: &[usize]) -> [f32; 2] {
    let mut acc = [0.0f32; 2];
    for &c in face_corners {
        acc[0] += attr[c][0];
        acc[1] += attr[c][1];
    }
    [acc[0] / face_corners.len() as f32, acc[1] / face_corners.len() as f32]
}
pub fn face_center_mean_f3(attr: &[[f32; 3]], face_corners: &[usize]) -> [f32; 3] {
    let mut acc = [0.0f32; 3];
    for &c in face_corners {
        acc[0] += attr[c][0];
        acc[1] += attr[c][1];
        acc[2] += attr[c][2];
    }
    let n = face_corners.len() as f32;
    [acc[0] / n, acc[1] / n, acc[2] / n]
}
pub fn face_center_mean_f4(attr: &[[f32; 4]], face_corners: &[usize]) -> [f32; 4] {
    let mut acc = [0.0f32; 4];
    for &c in face_corners {
        for i in 0..4 {
            acc[i] += attr[c][i];
        }
    }
    let n = face_corners.len() as f32;
    [acc[0] / n, acc[1] / n, acc[2] / n, acc[3] / n]
}

/// `get_from_tangent_matrix_tbn` sign (verbatim —
/// weighted tangent w sign).
pub fn from_tangent_sign(tang_ws: [f32; 3], bary_uv: [f32; 2]) -> f32 {
    let u = bary_uv[0];
    let v = bary_uv[1];
    let w = 1.0 - u - v;
    if (tang_ws[0] * u + tang_ws[1] * v + tang_ws[2] * w) < 0.0 {
        -1.0
    } else {
        1.0
    }
}
