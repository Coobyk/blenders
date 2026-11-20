//! Port of `math_vector_inline.cc` — C-named scalar/array helpers.
//! Operates on plain arrays to mirror the C signatures (`v` = f32,
//! `_db` = f64, `_int` = i32, `_short` = i16, `_uchar` = u8).

pub type V2 = [f32; 2];
pub type V3 = [f32; 3];
pub type V4 = [f32; 4];

/* Zero / copy / fill. */
pub fn zero_v2(r: &mut V2) { *r = [0.0; 2]; }
pub fn zero_v3(r: &mut V3) { *r = [0.0; 3]; }
pub fn zero_v4(r: &mut V4) { *r = [0.0; 4]; }
pub fn zero_v2_int(r: &mut [i32; 2]) { *r = [0; 2]; }
pub fn zero_v3_int(r: &mut [i32; 3]) { *r = [0; 3]; }
pub fn copy_v2_v2(r: &mut V2, a: V2) { *r = a; }
pub fn copy_v3_v3(r: &mut V3, a: V3) { *r = a; }
pub fn copy_v4_v4(r: &mut V4, a: V4) { *r = a; }
pub fn copy_v2_v2_db(r: &mut [f64; 2], a: [f64; 2]) { *r = a; }
pub fn copy_v3_v3_db(r: &mut [f64; 3], a: [f64; 3]) { *r = a; }
pub fn copy_v4_v4_db(r: &mut [f64; 4], a: [f64; 4]) { *r = a; }
pub fn copy_v2_v2_int(r: &mut [i32; 2], a: [i32; 2]) { *r = a; }
pub fn copy_v3_v3_int(r: &mut [i32; 3], a: [i32; 3]) { *r = a; }
pub fn copy_v4_v4_int(r: &mut [i32; 4], a: [i32; 4]) { *r = a; }
pub fn copy_v3_v3_short(r: &mut [i16; 3], a: [i16; 3]) { *r = a; }
pub fn copy_v3_v3_uchar(r: &mut [u8; 3], a: [u8; 3]) { *r = a; }
pub fn copy_v4_v4_uchar(r: &mut [u8; 4], a: [u8; 4]) { *r = a; }
pub fn copy_v2_fl(r: &mut V2, a: f32) { *r = [a; 2]; }
pub fn copy_v2_fl2(r: &mut V2, x: f32, y: f32) { *r = [x, y]; }
pub fn copy_v3_fl(r: &mut V3, a: f32) { *r = [a; 3]; }
pub fn copy_v3_fl3(r: &mut V3, x: f32, y: f32, z: f32) { *r = [x, y, z]; }
pub fn copy_v4_fl(r: &mut V4, a: f32) { *r = [a; 4]; }
pub fn copy_v4_fl4(r: &mut V4, x: f32, y: f32, z: f32, w: f32) { *r = [x, y, z, w]; }
pub fn copy_v2db_v2fl(r: &mut V2, a: [f64; 2]) { *r = [a[0] as f32, a[1] as f32]; }
pub fn copy_v2fl_v2i(r: &mut V2, a: [i32; 2]) { *r = [a[0] as f32, a[1] as f32]; }
pub fn copy_v3db_v3fl(r: &mut V3, a: [f64; 3]) { *r = [a[0] as f32, a[1] as f32, a[2] as f32]; }
pub fn copy_v3fl_v3db(r: &mut [f64; 3], a: V3) { *r = [a[0] as f64, a[1] as f64, a[2] as f64]; }
pub fn copy_v4fl_v4db(r: &mut [f64; 4], a: V4) { *r = [a[0] as f64, a[1] as f64, a[2] as f64, a[3] as f64]; }
pub fn swap_v2_v2(a: &mut V2, b: &mut V2) { std::mem::swap(a, b); }
pub fn swap_v3_v3(a: &mut V3, b: &mut V3) { std::mem::swap(a, b); }
pub fn swap_v4_v4(a: &mut V4, b: &mut V4) { std::mem::swap(a, b); }

/* Add. */
pub fn add_v2_fl(r: &mut V2, a: f32) { r[0] += a; r[1] += a; }
pub fn add_v3_fl(r: &mut V3, a: f32) { r[0] += a; r[1] += a; r[2] += a; }
pub fn add_v2_v2(r: &mut V2, a: V2) { r[0] += a[0]; r[1] += a[1]; }
pub fn add_v2_v2v2(r: &mut V2, a: V2, b: V2) { *r = [a[0] + b[0], a[1] + b[1]]; }
pub fn add_v3_v3(r: &mut V3, a: V3) { r[0] += a[0]; r[1] += a[1]; r[2] += a[2]; }
pub fn add_v3_v3v3(r: &mut V3, a: V3, b: V3) { *r = [a[0] + b[0], a[1] + b[1], a[2] + b[2]]; }
pub fn add_v3_v3_db(r: &mut [f64; 3], a: [f64; 3]) { r[0] += a[0]; r[1] += a[1]; r[2] += a[2]; }
pub fn add_v3_uchar_clamped(r: &mut V3, a: u8) {
    r[0] += a as f32;
    r[1] += a as f32;
    r[2] += a as f32;
}
pub fn add_v4_v4(r: &mut V4, a: V4) { r[0] += a[0]; r[1] += a[1]; r[2] += a[2]; r[3] += a[3]; }
pub fn add_v3fl_v3fl_v3i(r: &mut V3, a: V3, b: [i32; 3]) {
    r[0] += a[0] + b[0] as f32;
    r[1] += a[1] + b[1] as f32;
    r[2] += a[2] + b[2] as f32;
}
pub fn add_newell_cross_v3_v3v3(n: &mut V3, v_prev: V3, v_curr: V3) {
    n[0] += (v_prev[1] - v_curr[1]) * (v_prev[2] + v_curr[2]);
    n[1] += (v_prev[2] - v_curr[2]) * (v_prev[0] + v_curr[0]);
    n[2] += (v_prev[0] - v_curr[0]) * (v_prev[1] + v_curr[1]);
}

/* Subtract. */
pub fn sub_v2_v2(r: &mut V2, a: V2) { r[0] -= a[0]; r[1] -= a[1]; }
pub fn sub_v2_v2v2(r: &mut V2, a: V2, b: V2) { *r = [a[0] - b[0], a[1] - b[1]]; }
pub fn sub_v2_v2v2_db(r: &mut [f64; 2], a: [f64; 2], b: [f64; 2]) {
    *r = [a[0] - b[0], a[1] - b[1]];
}
pub fn sub_v2_v2v2_int(r: &mut [i32; 2], a: [i32; 2], b: [i32; 2]) {
    *r = [a[0] - b[0], a[1] - b[1]];
}
pub fn sub_v2db_v2fl_v2fl(r: &mut [f64; 2], a: V2, b: V2) {
    *r = [a[0] as f64 - b[0] as f64, a[1] as f64 - b[1] as f64];
}
pub fn sub_v3_v3(r: &mut V3, a: V3) { r[0] -= a[0]; r[1] -= a[1]; r[2] -= a[2]; }
pub fn sub_v3_v3v3(r: &mut V3, a: V3, b: V3) { *r = [a[0] - b[0], a[1] - b[1], a[2] - b[2]]; }
pub fn sub_v3_v3v3_db(r: &mut [f64; 3], a: [f64; 3], b: [f64; 3]) {
    *r = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}
pub fn sub_v3_v3v3_int(r: &mut [i32; 3], a: [i32; 3], b: [i32; 3]) {
    *r = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}
pub fn sub_v3db_v3fl_v3fl(r: &mut [f64; 3], a: V3, b: V3) {
    *r = [a[0] as f64 - b[0] as f64, a[1] as f64 - b[1] as f64, a[2] as f64 - b[2] as f64];
}
pub fn sub_v4_v4v4(r: &mut V4, a: V4, b: V4) {
    *r = [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]];
}

/* Multiply. */
pub fn mul_v2_fl(r: &mut V2, a: f32) { r[0] *= a; r[1] *= a; }
pub fn mul_v2_v2(r: &mut V2, a: V2) { r[0] *= a[0]; r[1] *= a[1]; }
pub fn mul_v2_v2fl(r: &mut V2, a: V2, f: f32) { *r = [a[0] * f, a[1] * f]; }
pub fn mul_v2_v2v2(r: &mut V2, a: V2, b: V2) { *r = [a[0] * b[0], a[1] * b[1]]; }
pub fn mul_v3_fl(r: &mut V3, a: f32) { r[0] *= a; r[1] *= a; r[2] *= a; }
pub fn mul_v3_v3(r: &mut V3, a: V3) { r[0] *= a[0]; r[1] *= a[1]; r[2] *= a[2]; }
pub fn mul_v3_v3fl(r: &mut V3, a: V3, f: f32) { *r = [a[0] * f, a[1] * f, a[2] * f]; }
pub fn mul_v3_v3v3(r: &mut V3, a: V3, b: V3) { *r = [a[0] * b[0], a[1] * b[1], a[2] * b[2]]; }
pub fn mul_v3db_db(r: &mut [f64; 3], a: f64) { r[0] *= a; r[1] *= a; r[2] *= a; }
pub fn mul_v4_fl(r: &mut V4, a: f32) { r[0] *= a; r[1] *= a; r[2] *= a; r[3] *= a; }
pub fn mul_v4_v4fl(r: &mut V4, a: V4, f: f32) { *r = [a[0] * f, a[1] * f, a[2] * f, a[3] * f]; }
pub fn madd_v2_v2fl(r: &mut V2, a: V2, f: f32) { r[0] += a[0] * f; r[1] += a[1] * f; }
pub fn madd_v2_v2v2fl(r: &mut V2, a: V2, b: V2, f: f32) {
    *r = [a[0] + b[0] * f, a[1] + b[1] * f];
}
pub fn madd_v3_v3fl(r: &mut V3, a: V3, f: f32) {
    r[0] += a[0] * f;
    r[1] += a[1] * f;
    r[2] += a[2] * f;
}
pub fn madd_v3_v3v3v3(r: &mut V3, a: V3, b: V3, c: V3) {
    *r = [a[0] + b[0] * c[0], a[1] + b[1] * c[1], a[2] + b[2] * c[2]];
}
pub fn madd_v3fl_v3fl_v3fl_v3i(r: &mut V3, a: V3, b: V3, c: [i32; 3]) {
    r[0] += a[0] + b[0] * c[0] as f32;
    r[1] += a[1] + b[1] * c[1] as f32;
    r[2] += a[2] + b[2] * c[2] as f32;
}
pub fn madd_v4_v4fl(r: &mut V4, a: V4, f: f32) {
    r[0] += a[0] * f;
    r[1] += a[1] * f;
    r[2] += a[2] * f;
    r[3] += a[3] * f;
}
/// `mul_project_m4_v3_zfac` — z-denominator of a projected point (mat col-major).
pub fn mul_project_m4_v3_zfac(mat: &crate::matrix::Mat4, co: V3) -> f32 {
    mat[0].w * co[0] + mat[1].w * co[1] + mat[2].w * co[2] + mat[3].w
}

/* Negate / invert. */
pub fn negate_v2(r: &mut V2) { r[0] = -r[0]; r[1] = -r[1]; }
pub fn negate_v2_v2(r: &mut V2, a: V2) { *r = [-a[0], -a[1]]; }
pub fn negate_v3(r: &mut V3) { r[0] = -r[0]; r[1] = -r[1]; r[2] = -r[2]; }
pub fn negate_v3_db(r: &mut [f64; 3]) { r[0] = -r[0]; r[1] = -r[1]; r[2] = -r[2]; }
pub fn negate_v3_v3(r: &mut V3, a: V3) { *r = [-a[0], -a[1], -a[2]]; }
pub fn negate_v4(r: &mut V4) { r[0] = -r[0]; r[1] = -r[1]; r[2] = -r[2]; r[3] = -r[3]; }
pub fn negate_v4_v4(r: &mut V4, a: V4) { *r = [-a[0], -a[1], -a[2], -a[3]]; }
pub fn invert_v2(r: &mut V2) { r[0] = 1.0 / r[0]; r[1] = 1.0 / r[1]; }
pub fn invert_v3(r: &mut V3) { r[0] = 1.0 / r[0]; r[1] = 1.0 / r[1]; r[2] = 1.0 / r[2]; }
/// `invert_v3_safe` — 0.0 component stays 0.0.
pub fn invert_v3_safe(r: &mut V3) {
    for c in r.iter_mut() {
        if *c != 0.0 {
            *c = 1.0 / *c;
        }
    }
}

/* Dot / cross. */
pub fn dot_v2v2(a: V2, b: V2) -> f32 { a[0] * b[0] + a[1] * b[1] }
pub fn dot_v2v2_db(a: [f64; 2], b: [f64; 2]) -> f64 { a[0] * b[0] + a[1] * b[1] }
pub fn dot_v4v4(a: V4, b: V4) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3] }
pub fn cross_v2v2(a: V2, b: V2) -> f32 { a[0] * b[1] - a[1] * b[0] }
pub fn cross_v2v2_db(a: [f64; 2], b: [f64; 2]) -> f64 { a[0] * b[1] - a[1] * b[0] }
pub fn cross_v3_v3v3(r: &mut V3, a: V3, b: V3) {
    *r = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
}
pub fn cross_v3_v3v3_db(r: &mut [f64; 3], a: [f64; 3], b: [f64; 3]) {
    *r = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
}
/// `dot_m3_v3_row_x/y/z` — row i of M (col-major) dotted with a.
pub fn dot_m3_v3_row_x(m: &crate::matrix::Mat3, a: V3) -> f32 {
    m[0].x * a[0] + m[1].x * a[1] + m[2].x * a[2]
}
pub fn dot_m3_v3_row_y(m: &crate::matrix::Mat3, a: V3) -> f32 {
    m[0].y * a[0] + m[1].y * a[1] + m[2].y * a[2]
}
pub fn dot_m3_v3_row_z(m: &crate::matrix::Mat3, a: V3) -> f32 {
    m[0].z * a[0] + m[1].z * a[1] + m[2].z * a[2]
}

/* Lengths / normalization. */
pub fn len_squared_v2v2(a: V2, b: V2) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1]];
    dot_v2v2(d, d)
}
pub fn len_squared_v4v4(a: V4, b: V4) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]];
    dot_v4v4(d, d)
}
pub fn len_v2(v: V2) -> f32 { dot_v2v2(v, v).sqrt() }
pub fn len_v2_db(v: [f64; 2]) -> f64 { dot_v2v2_db(v, v).sqrt() }
pub fn len_v2v2(a: V2, b: V2) -> f32 { len_squared_v2v2(a, b).sqrt() }
pub fn len_v2v2_db(a: [f64; 2], b: [f64; 2]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1]];
    dot_v2v2_db(d, d).sqrt()
}
pub fn len_v2v2_int(a: [i32; 2], b: [i32; 2]) -> f32 {
    let d = [(a[0] - b[0]) as f32, (a[1] - b[1]) as f32];
    dot_v2v2(d, d).sqrt()
}
pub fn len_v3v3(a: V3, b: V3) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}
pub fn len_v4(v: V4) -> f32 { dot_v4v4(v, v).sqrt() }
pub fn len_v4v4(a: V4, b: V4) -> f32 { len_squared_v4v4(a, b).sqrt() }
pub fn len_manhattan_v2(v: V2) -> f32 { v[0].abs() + v[1].abs() }
pub fn len_manhattan_v2_int(v: [i32; 2]) -> i32 { v[0].abs() + v[1].abs() }
pub fn len_manhattan_v3(v: V3) -> f32 { v[0].abs() + v[1].abs() + v[2].abs() }
pub fn len_manhattan_v2v2(a: V2, b: V2) -> f32 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs()
}
pub fn len_manhattan_v2v2_int(a: [i32; 2], b: [i32; 2]) -> i32 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs()
}
pub fn normalize_v2(r: &mut V2) -> f32 {
    let l = len_v2(*r);
    if l != 0.0 {
        r[0] /= l;
        r[1] /= l;
    }
    l
}
pub fn normalize_v2_length(r: &mut V2, len: f32) -> f32 {
    let l = normalize_v2(r);
    r[0] *= len;
    r[1] *= len;
    l
}
pub fn normalize_v2_v2(r: &mut V2, a: V2) -> f32 {
    *r = a;
    normalize_v2(r)
}
pub fn normalize_v2_v2_length(r: &mut V2, a: V2, len: f32) -> f32 {
    *r = a;
    normalize_v2_length(r, len)
}
pub fn normalize_v3(r: &mut V3) -> f32 {
    let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    if l != 0.0 {
        r[0] /= l;
        r[1] /= l;
        r[2] /= l;
    }
    l
}
pub fn normalize_v3_db(r: &mut [f64; 3]) -> f64 {
    let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    if l != 0.0 {
        r[0] /= l;
        r[1] /= l;
        r[2] /= l;
    }
    l
}
pub fn normalize_v3_length(r: &mut V3, len: f32) -> f32 {
    let l = normalize_v3(r);
    r[0] *= len;
    r[1] *= len;
    r[2] *= len;
    l
}
pub fn normalize_v3_length_db(r: &mut [f64; 3], len: f64) -> f64 {
    let l = normalize_v3_db(r);
    r[0] *= len;
    r[1] *= len;
    r[2] *= len;
    l
}
pub fn normalize_v3_v3(r: &mut V3, a: V3) -> f32 {
    *r = a;
    normalize_v3(r)
}
pub fn normalize_v3_v3_length(r: &mut V3, a: V3, len: f32) -> f32 {
    *r = a;
    normalize_v3_length(r, len)
}

/* Clamp. */
pub fn clamp_v2(v: &mut V2, min: V2, max: V2) {
    for i in 0..2 {
        v[i] = v[i].clamp(min[i], max[i]);
    }
}
pub fn clamp_v3(v: &mut V3, min: V3, max: V3) {
    for i in 0..3 {
        v[i] = v[i].clamp(min[i], max[i]);
    }
}
pub fn clamp_v4(v: &mut V4, min: V4, max: V4) {
    for i in 0..4 {
        v[i] = v[i].clamp(min[i], max[i]);
    }
}

/* Comparison. */
pub fn compare_v2v2(a: V2, b: V2, limit: f32) -> bool {
    (a[0] - b[0]).abs() < limit && (a[1] - b[1]).abs() < limit
}
pub fn compare_v3v3(a: V3, b: V3, limit: f32) -> bool {
    (a[0] - b[0]).abs() < limit && (a[1] - b[1]).abs() < limit && (a[2] - b[2]).abs() < limit
}
pub fn compare_v4v4(a: V4, b: V4, limit: f32) -> bool {
    (a[0] - b[0]).abs() < limit
        && (a[1] - b[1]).abs() < limit
        && (a[2] - b[2]).abs() < limit
        && (a[3] - b[3]).abs() < limit
}
pub fn compare_len_v3v3(a: V3, b: V3, limit: f32) -> bool {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz < limit * limit
}
pub fn compare_size_v3v3(a: V3, b: V3, limit: f32) -> bool {
    let l1 = a[0] * a[0] + a[1] * a[1] + a[2] * a[2];
    let l2 = b[0] * b[0] + b[1] * b[1] + b[2] * b[2];
    (l1 - l2).abs() < limit * limit
}
pub fn equals_v2v2(a: V2, b: V2) -> bool { a == b }
pub fn equals_v3v3(a: V3, b: V3) -> bool { a == b }
pub fn equals_v4v4(a: V4, b: V4) -> bool { a == b }
pub fn equals_v4v4_int(a: [i32; 4], b: [i32; 4]) -> bool { a == b }
pub fn is_zero_v2(v: V2) -> bool { v[0] == 0.0 && v[1] == 0.0 }
pub fn is_zero_v4(v: V4) -> bool { v[0] == 0.0 && v[1] == 0.0 && v[2] == 0.0 && v[3] == 0.0 }
pub fn is_one_v3(v: V3) -> bool { v[0] == 1.0 && v[1] == 1.0 && v[2] == 1.0 }
pub fn is_finite_v2(v: V2) -> bool { v[0].is_finite() && v[1].is_finite() }
pub fn is_finite_v4(v: V4) -> bool {
    v[0].is_finite() && v[1].is_finite() && v[2].is_finite() && v[3].is_finite()
}

/* Normals as packed shorts. */
pub fn normal_short_to_float_v3(out: &mut V3, input: [i16; 3]) {
    *out = [
        input[0] as f32 * (1.0 / 32767.0),
        input[1] as f32 * (1.0 / 32767.0),
        input[2] as f32 * (1.0 / 32767.0),
    ];
}
pub fn normal_float_to_short_v3(out: &mut [i16; 3], input: V3) {
    *out = [
        (input[0] * 32767.0) as i16,
        (input[1] * 32767.0) as i16,
        (input[2] * 32767.0) as i16,
    ];
}

/// `line_point_side_v2`.
pub fn line_point_side_v2(l1: V2, l2: V2, pt: V2) -> f32 {
    ((l1[0] - pt[0]) * (l2[1] - pt[1])) - ((l2[0] - pt[0]) * (l1[1] - pt[1]))
}
