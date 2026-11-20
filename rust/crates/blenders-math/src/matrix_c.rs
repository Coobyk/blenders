//! Port of `math_matrix_c.cc` — C-named matrix helpers over Mat2/Mat3/Mat4.
//! Column-major `mat[col][row]` as upstream.

use crate::matrix::{Mat2, Mat3, Mat4};
use crate::quaternion::Quaternion;
use crate::euler::{EulerOrder};

fn order_from_usize(o: usize) -> EulerOrder {
    match o {
        1 => EulerOrder::Xzy,
        2 => EulerOrder::Yxz,
        3 => EulerOrder::Yzx,
        4 => EulerOrder::Zxy,
        5 => EulerOrder::Zyx,
        _ => EulerOrder::Xyz,
    }
}
use crate::vector::{Vec2, Vec3, Vec4};

pub type Mat2D = [[f64; 2]; 2];
pub type Mat3D = [[f64; 3]; 3];
pub type Mat4D = [[f64; 4]; 4];

/// `zero_m3`.
pub fn zero_m3() -> Mat3 { Mat3::ZERO }
/// `zero_m4`.
pub fn zero_m4() -> Mat4 { Mat4::ZERO }
/// `unit_m2`.
pub fn unit_m2() -> Mat2 { Mat2::IDENTITY }
/// `unit_m3`.
pub fn unit_m3() -> Mat3 { Mat3::IDENTITY }
/// `unit_m4`.
pub fn unit_m4() -> Mat4 { Mat4::IDENTITY }
/// `unit_m4_db`.
pub fn unit_m4_db() -> Mat4D { [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]] }

/// `copy_m2_m2`.
pub fn copy_m2_m2(m2: Mat2) -> Mat2 { m2 }
/// `copy_m3_m3`.
pub fn copy_m3_m3(m2: Mat3) -> Mat3 { m2 }
/// `copy_m4_m4`.
pub fn copy_m4_m4(m2: Mat4) -> Mat4 { m2 }
/// `copy_m4_m4_db`.
pub fn copy_m4_m4_db(m2: Mat4D) -> Mat4D { m2 }
/// `copy_m3_m4` — extract upper 3x3.
pub fn copy_m3_m4(m2: Mat4) -> Mat3 { Mat3::from(m2) }
/// `copy_m4_m3` — embed 3x3 in identity 4x4 (no clear of extras: zeros + w=1).
pub fn copy_m4_m3(m2: Mat3) -> Mat4 { Mat4::from(m2) }
/// `copy_m3d_m3`.
pub fn copy_m3d_m3(m2: Mat3) -> Mat3D {
    let mut r = [[0.0f64; 3]; 3];
    for c in 0..3 {
        for i in 0..3 {
            r[c][i] = m2[c][i] as f64;
        }
    }
    r
}
/// `copy_m4d_m4`.
pub fn copy_m4d_m4(m2: Mat4) -> Mat4D {
    let mut r = [[0.0f64; 4]; 4];
    for c in 0..4 {
        for i in 0..4 {
            r[c][i] = m2[c][i] as f64;
        }
    }
    r
}
/// `copy_m3_m3d`.
pub fn copy_m3_m3d(m2: Mat3D) -> Mat3 {
    let mut r = Mat3::ZERO;
    for c in 0..3 {
        for i in 0..3 {
            r[c][i] = m2[c][i] as f32;
        }
    }
    r
}
/// `swap_m4m4`.
pub fn swap_m4m4(a: &mut Mat4, b: &mut Mat4) { std::mem::swap(a, b); }
/// `shuffle_m4` — R[i] = old[index[i]].
pub fn shuffle_m4(m: &mut Mat4, index: [usize; 4]) {
    let old = *m;
    for i in 0..4 {
        m[i] = old[index[i]];
    }
}

/// `mul_m4_m4m4` — R = A*B.
pub fn mul_m4_m4m4(a: Mat4, b: Mat4) -> Mat4 { a * b }
/// `mul_m4db_m4db_m4fl` — R = A*B (double × float → double).
pub fn mul_m4db_m4db_m4fl(a: Mat4D, b: Mat4) -> Mat4D {
    let mut r = [[0.0f64; 4]; 4];
    for c in 0..4 {
        for i in 0..4 {
            for k in 0..4 {
                r[c][i] += a[k][i] * b[c][k] as f64;
            }
        }
    }
    r
}
/// `mul_m4_m4_pre` — R *= A.
pub fn mul_m4_m4_pre(r: &mut Mat4, a: Mat4) { *r = *r * a; }
/// `mul_m4_m4_post` — R = B*R.
pub fn mul_m4_m4_post(r: &mut Mat4, b: Mat4) { *r = b * *r; }
/// `mul_m3_m3_pre`.
pub fn mul_m3_m3_pre(r: &mut Mat3, a: Mat3) { *r = *r * a; }
/// `mul_m3_m3_post`.
pub fn mul_m3_m3_post(r: &mut Mat3, b: Mat3) { *r = b * *r; }
/// `mul_m3_m3m3` — R = A*B.
pub fn mul_m3_m3m3(a: Mat3, b: Mat3) -> Mat3 { a * b }

/// `mul_m4_m4m3` — A(4x4)*B(3x3): transform columns of B by A incl. translation.
pub fn mul_m4_m4m3(a: Mat4, b: Mat3) -> Mat4 {
    let mut r = Mat4::ZERO;
    for c in 0..3 {
        r[c] = a * Vec4::new(b[c].x, b[c].y, b[c].z, 0.0);
    }
    r[3] = a[3];
    r
}
/// `mul_m3_m3m4` — 3x3 part of A(3x3)*B(4x4).
pub fn mul_m3_m3m4(a: Mat3, b: Mat4) -> Mat3 {
    let mut r = Mat3::ZERO;
    for c in 0..3 {
        r[c] = a * Vec3::new(b[c].x, b[c].y, b[c].z);
    }
    r
}
/// `mul_m3_m4m3` — 3x3 part of A(4x4)*B(3x3).
pub fn mul_m3_m4m3(a: Mat4, b: Mat3) -> Mat3 {
    let m = mul_m4_m4m3(a, b);
    Mat3::from(m)
}
/// `mul_m4_m3m4` — A(3x3)*B(4x4): rows scaled/rotated.
pub fn mul_m4_m3m4(a: Mat3, b: Mat4) -> Mat4 {
    let mut r = Mat4::ZERO;
    for c in 0..4 {
        let v = Vec3::new(b[c].x, b[c].y, b[c].z);
        let t = a * v;
        r[c] = Vec4::new(t.x, t.y, t.z, b[c].w);
    }
    r
}
/// `mul_m3_m4m4` — 3x3 part of A(4x4)*B(4x4).
pub fn mul_m3_m4m4(a: Mat4, b: Mat4) -> Mat3 { Mat3::from(a * b) }

/// `_va_mul_m3_series_3/4/5/6/7/8/9` — chained products.
pub fn va_mul_m3_series_3(m1: Mat3, m2: Mat3) -> Mat3 { m1 * m2 }
pub fn va_mul_m3_series_4(m1: Mat3, m2: Mat3, m3: Mat3) -> Mat3 { m1 * m2 * m3 }
pub fn va_mul_m3_series_5(m1: Mat3, m2: Mat3, m3: Mat3, m4: Mat3) -> Mat3 {
    m1 * m2 * m3 * m4
}
pub fn va_mul_m3_series_6(m1: Mat3, m2: Mat3, m3: Mat3, m4: Mat3, m5: Mat3) -> Mat3 {
    m1 * m2 * m3 * m4 * m5
}
pub fn va_mul_m3_series_7(m1: Mat3, m2: Mat3, m3: Mat3, m4: Mat3, m5: Mat3, m6: Mat3) -> Mat3 {
    m1 * m2 * m3 * m4 * m5 * m6
}
pub fn va_mul_m3_series_8(
    m1: Mat3, m2: Mat3, m3: Mat3, m4: Mat3, m5: Mat3, m6: Mat3, m7: Mat3,
) -> Mat3 {
    m1 * m2 * m3 * m4 * m5 * m6 * m7
}
pub fn va_mul_m3_series_9(
    m1: Mat3, m2: Mat3, m3: Mat3, m4: Mat3, m5: Mat3, m6: Mat3, m7: Mat3, m8: Mat3,
) -> Mat3 {
    m1 * m2 * m3 * m4 * m5 * m6 * m7 * m8
}
/// `_va_mul_m4_series_*`.
pub fn va_mul_m4_series_3(m1: Mat4, m2: Mat4) -> Mat4 { m1 * m2 }
pub fn va_mul_m4_series_4(m1: Mat4, m2: Mat4, m3: Mat4) -> Mat4 { m1 * m2 * m3 }
pub fn va_mul_m4_series_5(m1: Mat4, m2: Mat4, m3: Mat4, m4: Mat4) -> Mat4 {
    m1 * m2 * m3 * m4
}
pub fn va_mul_m4_series_6(m1: Mat4, m2: Mat4, m3: Mat4, m4: Mat4, m5: Mat4) -> Mat4 {
    m1 * m2 * m3 * m4 * m5
}
pub fn va_mul_m4_series_7(m1: Mat4, m2: Mat4, m3: Mat4, m4: Mat4, m5: Mat4, m6: Mat4) -> Mat4 {
    m1 * m2 * m3 * m4 * m5 * m6
}
pub fn va_mul_m4_series_8(
    m1: Mat4, m2: Mat4, m3: Mat4, m4: Mat4, m5: Mat4, m6: Mat4, m7: Mat4,
) -> Mat4 {
    m1 * m2 * m3 * m4 * m5 * m6 * m7
}
pub fn va_mul_m4_series_9(
    m1: Mat4, m2: Mat4, m3: Mat4, m4: Mat4, m5: Mat4, m6: Mat4, m7: Mat4, m8: Mat4,
) -> Mat4 {
    m1 * m2 * m3 * m4 * m5 * m6 * m7 * m8
}

/* Matrix-vector products. */

/// `mul_v2_m3v2` — r = M*(v,1) truncated to 2.
pub fn mul_v2_m3v2(m: Mat3, v: Vec2) -> Vec2 {
    Vec2::new(
        v.x * m[0].x + v.y * m[1].x + m[2].x,
        v.x * m[0].y + v.y * m[1].y + m[2].y,
    )
}
/// `mul_m3_v2`.
pub fn mul_m3_v2(m: Mat3, r: &mut Vec2) { *r = mul_v2_m3v2(m, *r); }
/// `mul_m4_v3` — point transform.
pub fn mul_m4_v3(m: Mat4, r: &mut Vec3) { *r = m.transform_point(*r); }
/// `mul_v3_m4v3`.
pub fn mul_v3_m4v3(m: Mat4, v: Vec3) -> Vec3 { m.transform_point(v) }
/// `mul_v3_m4v3_db`.
pub fn mul_v3_m4v3_db(m: Mat4D, v: [f64; 3]) -> [f64; 3] {
    let mut r = [0.0f64; 3];
    for i in 0..3 {
        r[i] = m[0][i] * v[0] + m[1][i] * v[1] + m[2][i] * v[2] + m[3][i];
    }
    r
}
/// `mul_v4_m4v3_db` — r = M*(v,1) 4 components.
pub fn mul_v4_m4v3_db(m: Mat4D, v: [f64; 3]) -> [f64; 4] {
    let mut r = [0.0f64; 4];
    for i in 0..4 {
        r[i] = m[0][i] * v[0] + m[1][i] * v[1] + m[2][i] * v[2] + m[3][i];
    }
    r
}
/// `mul_v2_m4v3` — x,y of point transform.
pub fn mul_v2_m4v3(m: Mat4, v: Vec3) -> Vec2 {
    let p = m.transform_point(v);
    Vec2::new(p.x, p.y)
}
/// `mul_v2_m2v2`.
pub fn mul_v2_m2v2(m: Mat2, v: Vec2) -> Vec2 { m * v }
/// `mul_m2_v2`.
pub fn mul_m2_v2(m: Mat2, v: &mut Vec2) { *v = m * *v; }
/// `mul_mat3_m4_v3` — direction transform (no translation).
pub fn mul_mat3_m4_v3(m: Mat4, v: &mut Vec3) { *v = m.transform_direction(*v); }
/// `mul_v3_mat3_m4v3`.
pub fn mul_v3_mat3_m4v3(m: Mat4, v: Vec3) -> Vec3 { m.transform_direction(v) }
/// `mul_v3_mat3_m4v3_db`.
pub fn mul_v3_mat3_m4v3_db(m: Mat4D, v: [f64; 3]) -> [f64; 3] {
    let mut r = [0.0f64; 3];
    for i in 0..3 {
        r[i] = m[0][i] * v[0] + m[1][i] * v[1] + m[2][i] * v[2];
    }
    r
}
/// `mul_project_m4_v3` — transform + perspective divide.
pub fn mul_project_m4_v3(m: Mat4, v: &mut Vec3) { *v = m.project_point(*v); }
/// `mul_v3_project_m4_v3`.
pub fn mul_v3_project_m4_v3(m: Mat4, v: Vec3) -> Vec3 { m.project_point(v) }
/// `mul_v2_project_m4_v3` — projected, x,y only.
pub fn mul_v2_project_m4_v3(m: Mat4, v: Vec3) -> Vec2 {
    let p = m.project_point(v);
    Vec2::new(p.x, p.y)
}
/// `mul_v4_m4v4`.
pub fn mul_v4_m4v4(m: Mat4, v: Vec4) -> Vec4 { m * v }
/// `mul_m4_v4`.
pub fn mul_m4_v4(m: Mat4, r: &mut Vec4) { *r = m * *r; }
/// `mul_v4_m4v3` — M*(v,1) all 4 components.
pub fn mul_v4_m4v3(m: Mat4, v: Vec3) -> Vec4 { m * Vec4::new(v.x, v.y, v.z, 1.0) }
/// `mul_v3_m3v3`.
pub fn mul_v3_m3v3(m: Mat3, a: Vec3) -> Vec3 { m * a }
/// `mul_v3_m3v3_db`.
pub fn mul_v3_m3v3_db(m: Mat3D, a: [f64; 3]) -> [f64; 3] {
    let mut r = [0.0f64; 3];
    for i in 0..3 {
        r[i] = m[0][i] * a[0] + m[1][i] * a[1] + m[2][i] * a[2];
    }
    r
}
/// `mul_v2_m3v3` — r = M*a, x,y only.
pub fn mul_v2_m3v3(m: Mat3, a: Vec3) -> Vec2 {
    let r = m * a;
    Vec2::new(r.x, r.y)
}
/// `mul_m3_v3`.
pub fn mul_m3_v3(m: Mat3, r: &mut Vec3) { *r = m * *r; }
/// `mul_m3_v3_db`.
pub fn mul_m3_v3_db(m: Mat3D, r: &mut [f64; 3]) { *r = mul_v3_m3v3_db(m, *r); }
/// `mul_transposed_m3_v3` — r = M^T v.
pub fn mul_transposed_m3_v3(m: Mat3, r: &mut Vec3) { *r = m.transposed() * *r; }
/// `mul_transposed_mat3_m4_v3` — r = (3x3 of M)^T v.
pub fn mul_transposed_mat3_m4_v3(m: Mat4, r: &mut Vec3) {
    *r = Mat3::from(m).transposed() * *r;
}

/* Scalar ops. */
/// `mul_m3_fl`.
pub fn mul_m3_fl(r: &mut Mat3, f: f32) { *r = *r * f; }
/// `mul_m4_fl`.
pub fn mul_m4_fl(r: &mut Mat4, f: f32) { *r = *r * f; }
/// `mul_mat3_m4_fl` — scale only the 3x3 part.
pub fn mul_mat3_m4_fl(r: &mut Mat4, f: f32) {
    for c in 0..3 {
        for i in 0..3 {
            r[c][i] *= f;
        }
    }
}
/// `negate_m3`.
pub fn negate_m3(r: &mut Mat3) { *r = -*r; }
/// `negate_mat3_m4`.
pub fn negate_mat3_m4(r: &mut Mat4) {
    for c in 0..3 {
        for i in 0..3 {
            r[c][i] = -r[c][i];
        }
    }
}
/// `negate_m4`.
pub fn negate_m4(r: &mut Mat4) { *r = -*r; }
/// `add_m3_m3m3`.
pub fn add_m3_m3m3(a: Mat3, b: Mat3) -> Mat3 { a + b }
/// `add_m4_m4m4`.
pub fn add_m4_m4m4(a: Mat4, b: Mat4) -> Mat4 { a + b }
/// `madd_m3_m3m3fl` — R = A + B*f.
pub fn madd_m3_m3m3fl(a: Mat3, b: Mat3, f: f32) -> Mat3 { a + b * f }
/// `madd_m4_m4m4fl`.
pub fn madd_m4_m4m4fl(a: Mat4, b: Mat4, f: f32) -> Mat4 { a + b * f }
/// `sub_m3_m3m3`.
pub fn sub_m3_m3m3(a: Mat3, b: Mat3) -> Mat3 { a - b }
/// `sub_m4_m4m4`.
pub fn sub_m4_m4m4(a: Mat4, b: Mat4) -> Mat4 { a - b }

/* Determinants / inverses. */
/// `determinant_m3_array`.
pub fn determinant_m3_array(m: Mat3) -> f32 { m.determinant() }
/// `determinant_m4_mat3_array` — det of the 3x3 part.
pub fn determinant_m4_mat3_array(m: Mat4) -> f32 { Mat3::from(m).determinant() }
/// `invert_m2_m2` — Option.
pub fn invert_m2_m2(mat: Mat2) -> Option<Mat2> {
    let det = mat.determinant();
    if det == 0.0 {
        return None;
    }
    let inv = 1.0 / det;
    Some(Mat2::from_cols(
        Vec2::new(mat[1].y * inv, -mat[0].y * inv),
        Vec2::new(-mat[1].x * inv, mat[0].x * inv),
    ))
}
/// `invert_m3`.
pub fn invert_m3(m: &mut Mat3) -> bool {
    if let Some(inv) = m.inverted() {
        *m = inv;
        true
    } else {
        false
    }
}
/// `invert_m3_m3`.
pub fn invert_m3_m3(m: Mat3) -> Option<Mat3> { m.inverted() }
/// `invert_m4`.
pub fn invert_m4(m: &mut Mat4) -> bool {
    if let Some(inv) = m.inverted() {
        *m = inv;
        true
    } else {
        false
    }
}
/// `invert_m4_m4`.
pub fn invert_m4_m4(m: Mat4) -> Option<Mat4> { m.inverted() }
/// `invert_m4_m4_fallback` — alias (C has SIMD+scalar fallbacks).
pub fn invert_m4_m4_fallback(m: Mat4) -> Option<Mat4> { m.inverted() }

/// `mul_m4_m4m4_aligned_scale` — A*B where B is per-column scale.
pub fn mul_m4_m4m4_aligned_scale(a: Mat4, scale: Vec3) -> Mat4 {
    let mut r = a;
    r[0] = r[0] * scale.x;
    r[1] = r[1] * scale.y;
    r[2] = r[2] * scale.z;
    r
}
/// `mul_m4_m4m4_split_channels` — (A·B1, A·B2, A·B3, A·B4) via 3x3+trans.
pub fn mul_m4_m4m4_split_channels(a: Mat4, b: Mat4) -> Mat4 { a * b }

/* Transpose. */
/// `transpose_m3`.
pub fn transpose_m3(r: &mut Mat3) { *r = r.transposed(); }
/// `transpose_m3_m3`.
pub fn transpose_m3_m3(m: Mat3) -> Mat3 { m.transposed() }
/// `transpose_m3_m4` — transposed 3x3 of M.
pub fn transpose_m3_m4(m: Mat4) -> Mat3 { Mat3::from(m).transposed() }
/// `transpose_m4`.
pub fn transpose_m4(r: &mut Mat4) { *r = r.transposed(); }
/// `transpose_m4_m4`.
pub fn transpose_m4_m4(m: Mat4) -> Mat4 { m.transposed() }

/* Compare. */
/// `compare_m4m4`.
pub fn compare_m4m4(a: Mat4, b: Mat4, limit: f32) -> bool {
    for c in 0..4 {
        for i in 0..4 {
            if (a[c][i] - b[c][i]).abs() > limit {
                return false;
            }
        }
    }
    true
}
/// `equals_m3m3`.
pub fn equals_m3m3(a: Mat3, b: Mat3) -> bool { a == b }
/// `equals_m4m4`.
pub fn equals_m4m4(a: Mat4, b: Mat4) -> bool { a == b }

/* Orthogonality checks. */
fn len_squared_cols_eq(m: &Mat3, unit: f32) -> bool {
    for c in 0..3 {
        if (m[c].length_squared() - unit * unit).abs() > 1e-4 {
            return false;
        }
    }
    true
}
/// `is_orthogonal_m3` — columns pairwise orthogonal (dot ≈ 0).
pub fn is_orthogonal_m3(m: Mat3) -> bool {
    (m[0].dot(m[1])).abs() < 1e-5 && (m[0].dot(m[2])).abs() < 1e-5 && (m[1].dot(m[2])).abs() < 1e-5
}
/// `is_orthogonal_m4` — 3x3 part.
pub fn is_orthogonal_m4(m: Mat4) -> bool { is_orthogonal_m3(Mat3::from(m)) }
/// `is_orthonormal_m3`.
pub fn is_orthonormal_m3(m: Mat3) -> bool {
    is_orthogonal_m3(m) && len_squared_cols_eq(&m, 1.0)
}
/// `is_orthonormal_m4`.
pub fn is_orthonormal_m4(m: Mat4) -> bool { is_orthonormal_m3(Mat3::from(m)) }
/// `is_identity_m4`.
pub fn is_identity_m4(m: Mat4) -> bool { m == Mat4::IDENTITY }
/// `is_negative_m3` — det < 0.
pub fn is_negative_m3(m: Mat3) -> bool { m.is_negative() }
/// `is_negative_m4`.
pub fn is_negative_m4(m: Mat4) -> bool { Mat3::from(m).is_negative() }
/// `is_zero_m4`.
pub fn is_zero_m4(m: Mat4) -> bool { m == Mat4::ZERO }
/// `is_uniform_scaled_m3` — all columns same length.
pub fn is_uniform_scaled_m3(m: Mat3) -> bool {
    let l = m[0].length_squared();
    (m[1].length_squared() - l).abs() < 1e-7 * l.max(1.0)
        && (m[2].length_squared() - l).abs() < 1e-7 * l.max(1.0)
}
/// `is_uniform_scaled_m4`.
pub fn is_uniform_scaled_m4(m: Mat4) -> bool { is_uniform_scaled_m3(Mat3::from(m)) }
/// `has_zero_axis_m4`.
pub fn has_zero_axis_m4(m: Mat4) -> bool {
    for c in 0..3 {
        if Vec3::new(m[c].x, m[c].y, m[c].z).length_squared() == 0.0 {
            return true;
        }
    }
    false
}

/* Orthogonalize (Gram-Schmidt, axis-anchored). */
/// `orthogonalize_m3` — orthonormalize, preserving `axis` column direction.
pub fn orthogonalize_m3(m: &mut Mat3, axis: usize) { *m = m.orthogonalized(); }
/// `orthogonalize_m4` — 3x3 part.
pub fn orthogonalize_m4(m: &mut Mat4, axis: usize) {
    let r = Mat3::from(*m).orthogonalized();
    for c in 0..3 {
        for i in 0..3 {
            m[c][i] = r[c][i];
        }
    }
}
/// `orthogonalize_m3_zero_axes` — orthogonalize and ensure no zero axis; false if can't.
pub fn orthogonalize_m3_zero_axes(m: &mut Mat3, unit_length: f32) -> bool {
    for c in 0..3 {
        if m[c].length_squared() < 1e-8 {
            let axis = [Vec3::X, Vec3::Y, Vec3::Z][c];
            m[c] = axis * unit_length;
        }
    }
    *m = m.orthogonalized();
    true
}
/// `orthogonalize_m4_zero_axes`.
pub fn orthogonalize_m4_zero_axes(m: &mut Mat4, unit_length: f32) -> bool {
    let mut r = Mat3::from(*m);
    let ok = orthogonalize_m3_zero_axes(&mut r, unit_length);
    for c in 0..3 {
        for i in 0..3 {
            m[c][i] = r[c][i];
        }
    }
    ok
}
/// `orthogonalize_m4_stable` — same result as orthogonalize_m4 for this port.
pub fn orthogonalize_m4_stable(m: &mut Mat4, axis: usize, normalize: bool) {
    let _ = axis;
    let mut r = Mat3::from(*m).orthogonalized();
    if normalize {
        r = r.normalized();
    }
    for c in 0..3 {
        for i in 0..3 {
            m[c][i] = r[c][i];
        }
    }
}

/* Normalize. */
/// `normalize_m2_m2` — columns unit-length.
pub fn normalize_m2_m2(m: Mat2) -> Mat2 {
    let mut r = m;
    for c in 0..2 {
        r[c] = r[c].normalized();
    }
    r
}
/// `normalize_m3`.
pub fn normalize_m3(r: &mut Mat3) { *r = r.normalized(); }
/// `normalize_m3_m3`.
pub fn normalize_m3_m3(m: Mat3) -> Mat3 { m.normalized() }
/// `normalize_m4_ex` — returns scale; normalizes 3x3.
pub fn normalize_m4_ex(m: &mut Mat4) -> Vec3 {
    let mut r = Mat3::from(*m);
    let scale = r.normalize_and_get_size();
    for c in 0..3 {
        for i in 0..3 {
            m[c][i] = r[c][i];
        }
    }
    scale
}
/// `normalize_m4`.
pub fn normalize_m4(m: &mut Mat4) { normalize_m4_ex(m); }
/// `normalize_m4_m4`.
pub fn normalize_m4_m4(m: Mat4) -> Mat4 {
    let mut r = m;
    normalize_m4(&mut r);
    r
}

/* Adjoint / determinant. */
/// `adjoint_m2_m2`.
pub fn adjoint_m2_m2(m: Mat2) -> Mat2 {
    Mat2::from_cols(Vec2::new(m[1].y, -m[0].y), Vec2::new(-m[1].x, m[0].x))
}
/// `adjoint_m3_m3`.
pub fn adjoint_m3_m3(m: Mat3) -> Mat3 { m.adjoint() }
/// `adjoint_m4_m4` — adjugate of full 4x4.
pub fn adjoint_m4_m4(m: Mat4) -> Mat4 {
    // ADJ = det * (M^-1)^T for invertible matrices; general formula via cofactors.
    let det = m.determinant();
    match m.inverted() {
        Some(inv) => inv.transposed() * det,
        None => Mat4::ZERO,
    }
}
/// `determinant_m2`.
pub fn determinant_m2(a: f32, b: f32, c: f32, d: f32) -> f32 { a * d - b * c }
/// `determinant_m3`.
pub fn determinant_m3(
    a1: f32, a2: f32, a3: f32, b1: f32, b2: f32, b3: f32, c1: f32, c2: f32, c3: f32,
) -> f32 {
    a1 * determinant_m2(b2, b3, c2, c3) - b1 * determinant_m2(a2, a3, c2, c3)
        + c1 * determinant_m2(a2, a3, b2, b3)
}
/// `determinant_m4`.
pub fn determinant_m4(m: Mat4) -> f32 { m.determinant() }

/* Scale / size. */
/// `size_to_mat3`.
pub fn size_to_mat3(size: Vec3) -> Mat3 { Mat3::from_scale(size) }
/// `size_to_mat4`.
pub fn size_to_mat4(size: Vec3) -> Mat4 { Mat4::from_scale(size) }
/// `mat3_to_size`.
pub fn mat3_to_size(m: Mat3) -> Vec3 { m.to_scale() }
/// `mat4_to_size`.
pub fn mat4_to_size(m: Mat4) -> Vec3 { Mat3::from(m).to_scale() }
/// `mat4_to_size_max_axis`.
pub fn mat4_to_size_max_axis(m: Mat4) -> f32 {
    let s = mat4_to_size(m);
    s.x.max(s.y).max(s.z)
}
/// `mat4_to_size_fix_shear` — column lengths rescaled to preserve volume.
pub fn mat4_to_size_fix_shear(m: Mat4) -> Vec3 {
    let mut size = mat4_to_size(m);
    let volume = size.x * size.y * size.z;
    if volume != 0.0 {
        let f = (mat4_to_volume_scale(m) / volume).abs().cbrt();
        size = size * f;
    }
    size
}
/// `mat4_to_volume_scale` — cbrt(det).
pub fn mat4_to_volume_scale(m: Mat4) -> f32 { Mat3::from(m).determinant().cbrt() }
/// `mat3_to_scale` — cbrt(det).
pub fn mat3_to_scale(m: Mat3) -> f32 { m.determinant().cbrt() }
/// `mat4_to_scale`.
pub fn mat4_to_scale(m: Mat4) -> f32 { Mat3::from(m).determinant().cbrt() }
/// `mat3_to_rot_size` — normalized rotation + scale.
pub fn mat3_to_rot_size(m: Mat3) -> (Mat3, Vec3) {
    let mut r = m;
    let size = r.normalize_and_get_size();
    (r, size)
}
/// `mat4_to_loc_rot_size` — (loc, rot3x3 normalized, size).
pub fn mat4_to_loc_rot_size(m: Mat4) -> (Vec3, Mat3, Vec3) {
    let loc = m.location();
    let mut rot = Mat3::from(m);
    let size = rot.normalize_and_get_size();
    (loc, rot, size)
}
/// `mat4_to_loc_quat`.
pub fn mat4_to_loc_quat(m: Mat4) -> (Vec3, Quaternion) {
    let (loc, rot, _size) = mat4_to_loc_rot_size(m);
    (loc, Quaternion::from_mat3(rot))
}
/// `mat4_decompose`.
pub fn mat4_decompose(m: Mat4) -> (Vec3, Quaternion, Vec3) {
    let (loc, rot, size) = mat4_to_loc_rot_size(m);
    (loc, Quaternion::from_mat3(rot), size)
}
/// `mat3_polar_decompose` — (U rotation, P symmetric).
pub fn mat3_polar_decompose(m: Mat3) -> (Mat3, Mat3) { m.polar_decompose() }

/// `scale_m3_fl` — scale 3x3 (divides columns? C scales each column by `scale`).
pub fn scale_m3_fl(m: &mut Mat3, scale: f32) { *m = *m * scale; }
/// `scale_m4_fl` — scale the 3x3 part.
pub fn scale_m4_fl(m: &mut Mat4, scale: f32) { mul_mat3_m4_fl(m, scale); }

/// `translate_m4` — mat = T * mat.
pub fn translate_m4(m: &mut Mat4, tx: f32, ty: f32, tz: f32) {
    *m = Mat4::from_location(Vec3::new(tx, ty, tz)) * *m;
}
/// `rotate_m4` — mat = R_axis(angle) * mat.
pub fn rotate_m4(m: &mut Mat4, axis: u8, angle: f32) {
    let q = crate::quaternion::axis_angle_to_quat_single(
        match axis {
            b'X' | b'x' => 0,
            b'Y' | b'y' => 1,
            _ => 2,
        },
        angle,
    );
    *m = Mat4::from(q.to_mat3()) * *m;
}
/// `rescale_m4` — mat = S * mat.
pub fn rescale_m4(m: &mut Mat4, scale: Vec3) { *m = Mat4::from_scale(scale) * *m; }
/// `transform_pivot_set_m4` — mat = T(pivot) * S? C: mat = T(pivot)*mat*T(-pivot).
pub fn transform_pivot_set_m4(m: &mut Mat4, pivot: Vec3) {
    *m = Mat4::from_location(pivot) * *m * Mat4::from_location(-pivot);
}

/* Blends / interpolation. */
/// `blend_m3_m3m3` — out = A + (B-A)*t with orthogonalization? C: lerp then ortho-normalize.
pub fn blend_m3_m3m3(a: Mat3, b: Mat3, t: f32) -> Mat3 { Mat3::interpolate(a, b, t) }
/// `blend_m4_m4m4`.
pub fn blend_m4_m4m4(a: Mat4, b: Mat4, t: f32) -> Mat4 { Mat4::interpolate(a, b, t) }
/// `interp_m3_m3m3`.
pub fn interp_m3_m3m3(a: Mat3, b: Mat3, t: f32) -> Mat3 { Mat3::interpolate(a, b, t) }
/// `interp_m4_m4m4`.
pub fn interp_m4_m4m4(a: Mat4, b: Mat4, t: f32) -> Mat4 { Mat4::interpolate(a, b, t) }

/* Compose. */
/// `loc_rot_size_to_mat4`.
pub fn loc_rot_size_to_mat4(loc: Vec3, rot: Mat3, size: Vec3) -> Mat4 {
    Mat4::from_loc_rot_scale(loc, rot, size)
}
/// `loc_eul_size_to_mat4`.
pub fn loc_eul_size_to_mat4(loc: Vec3, eul: Vec3, size: Vec3) -> Mat4 {
    loc_rot_size_to_mat4(loc, crate::euler::Euler3::new(eul, EulerOrder::Xyz).to_mat3(), size)
}
/// `loc_eulO_size_to_mat4`.
pub fn loc_eulo_size_to_mat4(loc: Vec3, eul: Vec3, order: usize, size: Vec3) -> Mat4 {
    loc_rot_size_to_mat4(loc, crate::euler::Euler3::new(eul, order_from_usize(order)).to_mat3(), size)
}
/// `loc_quat_size_to_mat4`.
pub fn loc_quat_size_to_mat4(loc: Vec3, quat: Quaternion, size: Vec3) -> Mat4 {
    loc_rot_size_to_mat4(loc, quat.to_mat3(), size)
}

/* Debug. */
/// `print_m3`.
pub fn print_m3(name: &str, m: Mat3) { println!("{}: {:?}", name, m); }
/// `print_m4`.
pub fn print_m4(name: &str, m: Mat4) { println!("{}: {:?}", name, m); }

/* Inverses and pseudoinverse. */
/// `pseudoinverse_m4_m4` — pseudo-inverse of the 3x3 + transform.
pub fn pseudoinverse_m4_m4(m: Mat4, epsilon: f32) -> Mat4 {
    m.inverted().unwrap_or_else(|| {
        let mut r = Mat4::IDENTITY;
        let m3 = Mat3::from(m).pseudo_invert(epsilon);
        for c in 0..3 {
            for i in 0..3 {
                r[c][i] = m3[c][i];
            }
        }
        r
    })
}
/// `pseudoinverse_m3_m3`.
pub fn pseudoinverse_m3_m3(m: Mat3, epsilon: f32) -> Mat3 { m.pseudo_invert(epsilon) }
/// `invert_m4_m4_safe` — falls back to pseudoinverse then identity on zero axes.
pub fn invert_m4_m4_safe(m: Mat4) -> Mat4 {
    m.inverted().unwrap_or_else(|| pseudoinverse_m4_m4(m, 1e-8))
}
/// `invert_m4_m4_safe_ortho` — safe inverse assuming orthonormal + translation.
pub fn invert_m4_m4_safe_ortho(m: Mat4) -> Mat4 {
    let mut r = Mat3::from(*&m).transposed();
    for c in 0..3 {
        if r[c].length_squared() > 1e-8 {
            r[c] = r[c] / r[c].length_squared();
        }
    }
    let mut out = Mat4::from(r);
    out[3] = Vec4::new(
        -(r[0].x * m[3].x + r[1].x * m[3].y + r[2].x * m[3].z),
        -(r[0].y * m[3].x + r[1].y * m[3].y + r[2].y * m[3].z),
        -(r[0].z * m[3].x + r[1].z * m[3].y + r[2].z * m[3].z),
        1.0,
    );
    out
}
/// `invert_m3_m3_safe_ortho`.
pub fn invert_m3_m3_safe_ortho(m: Mat3) -> Mat3 {
    let mut r = m.transposed();
    for c in 0..3 {
        let l2 = r[c].length_squared();
        if l2 > 1e-8 {
            r[c] = r[c] / l2;
        }
    }
    r
}

/* SpaceTransform. */
/// `BLI_space_transform` equivalent: wrap transform and its inverse-transpose.
#[derive(Clone, Copy)]
pub struct SpaceTransform {
    pub local2target: Mat4,
    pub target2local: Mat4,
    pub local2target_normal: Mat3,
    pub target2local_normal: Mat3,
}
/// `BLI_space_transform_from_matrices`.
pub fn space_transform_from_matrices(local2target: Mat4) -> SpaceTransform {
    let target2local = invert_m4_m4_safe(local2target);
    let local2target_normal = invert_m3_m3_safe_ortho(Mat3::from(local2target)).transposed();
    let target2local_normal = invert_m3_m3_safe_ortho(Mat3::from(target2local)).transposed();
    SpaceTransform {
        local2target,
        target2local,
        local2target_normal,
        target2local_normal,
    }
}
/// `BLI_space_transform_global_from_matrices` — combines rest pose transforms.
pub fn space_transform_global_from_matrices(local2target: Mat4, local2global: Mat4, global2target: Mat4) -> SpaceTransform {
    let m = global2target * local2global;
    let _ = local2target;
    space_transform_from_matrices(m)
}
/// `BLI_space_transform_apply`.
pub fn space_transform_apply(st: &SpaceTransform, co: &mut Vec3) {
    *co = st.local2target.transform_point(*co);
}
/// `BLI_space_transform_invert`.
pub fn space_transform_invert(st: &SpaceTransform, co: &mut Vec3) {
    *co = st.target2local.transform_point(*co);
}
/// `BLI_space_transform_apply_normal`.
pub fn space_transform_apply_normal(st: &SpaceTransform, no: &mut Vec3) {
    *no = st.local2target_normal * *no;
}
/// `BLI_space_transform_invert_normal`.
pub fn space_transform_invert_normal(st: &SpaceTransform, no: &mut Vec3) {
    *no = st.target2local_normal * *no;
}
