//! Port of `BLI_math_matrix.hh` / `math_matrix.cc`.
//!
//! Matrices are stored column-major: `mat[col]` is a column vector and
//! `mat[col][row]` a scalar — identical memory layout and semantics to
//! Blender's `MatBase` and the C `float m[4][4]` API (translation at
//! `mat[3]` of a `Mat4`).

use std::ops::{Add, Index, IndexMut, Mul, Neg, Sub};

use crate::vector::{Vec2, Vec3, Vec4};

/// `float2x2` — 2 columns of `Vec2`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat2 {
    pub cols: [Vec2; 2],
}

/// `float3x3` — 3 columns of `Vec3`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat3 {
    pub cols: [Vec3; 3],
}

/// `float4x4` — 4 columns of `Vec4`; `cols[3]` is the translation.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat4 {
    pub cols: [Vec4; 4],
}

macro_rules! mat_index {
    ($MatN:ident, $VecN:ident) => {
        /// `mat[col]` — column access like `MatBase::operator[]`.
        impl Index<usize> for $MatN {
            type Output = $VecN;
            #[inline]
            fn index(&self, i: usize) -> &$VecN {
                &self.cols[i]
            }
        }
        impl IndexMut<usize> for $MatN {
            #[inline]
            fn index_mut(&mut self, i: usize) -> &mut $VecN {
                &mut self.cols[i]
            }
        }
    };
}

mat_index!(Mat2, Vec2);
mat_index!(Mat3, Vec3);
mat_index!(Mat4, Vec4);

impl Mat2 {
    pub const IDENTITY: Self = Self {
        cols: [Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)],
    };

    #[inline]
    pub const fn from_cols(c0: Vec2, c1: Vec2) -> Self {
        Self { cols: [c0, c1] }
    }

    /// Rotation matrix from angle (`from_rotation<Mat2>`).
    #[inline]
    pub fn rotation(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self::from_cols(Vec2::new(c, s), Vec2::new(-s, c))
    }

    /// `determinant_m2`.
    #[inline]
    pub fn determinant(self) -> f32 {
        self[0][0] * self[1][1] - self[0][1] * self[1][0]
    }

    /// `transpose_m2`.
    #[inline]
    pub fn transposed(self) -> Self {
        Self::from_cols(
            Vec2::new(self[0][0], self[1][0]),
            Vec2::new(self[0][1], self[1][1]),
        )
    }
}

impl Mul<Vec2> for Mat2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, v: Vec2) -> Vec2 {
        self[0] * v.x + self[1] * v.y
    }
}

impl Mul<Mat2> for Mat2 {
    type Output = Mat2;
    #[inline]
    fn mul(self, rhs: Mat2) -> Mat2 {
        Mat2::from_cols(self * rhs[0], self * rhs[1])
    }
}

impl Mat3 {
    pub const IDENTITY: Self = Self {
        cols: [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ],
    };
    pub const ZERO: Self = Self {
        cols: [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
    };

    #[inline]
    pub const fn from_cols(c0: Vec3, c1: Vec3, c2: Vec3) -> Self {
        Self { cols: [c0, c1, c2] }
    }

    /// Basis vectors (`x_axis()`, `y_axis()`, `z_axis()`).
    #[inline]
    pub fn x_axis(&self) -> Vec3 {
        self[0]
    }
    #[inline]
    pub fn y_axis(&self) -> Vec3 {
        self[1]
    }
    #[inline]
    pub fn z_axis(&self) -> Vec3 {
        self[2]
    }

    /// `from_scale<Mat3>` — diagonal scale matrix.
    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self::from_cols(
            Vec3::new(scale.x, 0.0, 0.0),
            Vec3::new(0.0, scale.y, 0.0),
            Vec3::new(0.0, 0.0, scale.z),
        )
    }

    /// `transpose_m3`.
    #[inline]
    pub fn transposed(self) -> Self {
        Self::from_cols(
            Vec3::new(self[0][0], self[1][0], self[2][0]),
            Vec3::new(self[0][1], self[1][1], self[2][1]),
            Vec3::new(self[0][2], self[1][2], self[2][2]),
        )
    }

    /// `determinant_m3`.
    #[inline]
    pub fn determinant(self) -> f32 {
        self[0][0] * (self[1][1] * self[2][2] - self[1][2] * self[2][1])
            - self[0][1] * (self[1][0] * self[2][2] - self[1][2] * self[2][0])
            + self[0][2] * (self[1][0] * self[2][1] - self[1][1] * self[2][0])
    }

    /// `invert_m3_m3` — adjugate inverse; returns `None` when singular.
    pub fn inverted(self) -> Option<Self> {
        let det = self.determinant();
        if det == 0.0 {
            return None;
        }
        let inv_det = 1.0 / det;
        // Adjugate is the transpose of the cofactor matrix.
        let m = self;
        let adj = Mat3::from_cols(
            Vec3::new(
                m[1][1] * m[2][2] - m[1][2] * m[2][1],
                m[0][2] * m[2][1] - m[0][1] * m[2][2],
                m[0][1] * m[1][2] - m[0][2] * m[1][1],
            ),
            Vec3::new(
                m[1][2] * m[2][0] - m[1][0] * m[2][2],
                m[0][0] * m[2][2] - m[0][2] * m[2][0],
                m[0][2] * m[1][0] - m[0][0] * m[1][2],
            ),
            Vec3::new(
                m[1][0] * m[2][1] - m[1][1] * m[2][0],
                m[0][1] * m[2][0] - m[0][0] * m[2][1],
                m[0][0] * m[1][1] - m[0][1] * m[1][0],
            ),
        );
        Some(adj * inv_det)
    }

    /// `normalize_m3` — orthonormalize each basis column.
    pub fn normalized(self) -> Self {
        Self::from_cols(
            self[0].normalized(),
            self[1].normalized(),
            self[2].normalized(),
        )
    }

    /// `normalize_and_get_size`.
    pub fn normalize_and_get_size(&mut self) -> Vec3 {
        let mut size = Vec3::ZERO;
        for i in 0..3 {
            size[i] = self[i].normalize_and_get_length();
        }
        size
    }

    /// `is_negative` — determinant < 0.
    #[inline]
    pub fn is_negative(self) -> bool {
        self.determinant() < 0.0
    }

    /// `to_scale` — column lengths (positive unless negative det, then all negated).
    pub fn to_scale(self) -> Vec3 {
        let scale = Vec3::new(self[0].length(), self[1].length(), self[2].length());
        if self.is_negative() { -scale } else { scale }
    }

    /// `orthogonalize` — Gram-Schmidt vs axis 0 (port of `math::orthogonalize`).
    pub fn orthogonalized(self) -> Self {
        let x = self[0].normalized();
        let mut y = self[1] - x * self[1].dot(x);
        let mut z = self[2] - x * self[2].dot(x);
        let ly = y.normalize_and_get_length();
        if ly != 0.0 {
            z = (z - y * z.dot(y)).normalized();
        } else {
            y = x.orthogonal();
            z = x.cross(y);
        }
        Self::from_cols(x, y, z)
    }

    /// `transform_direction` — 3x3 transform of a vector.
    #[inline]
    pub fn transform_direction(self, v: Vec3) -> Vec3 {
        self * v
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;
    /// `mul_m3_v3` — `M * v` (v as column vector).
    #[inline]
    fn mul(self, v: Vec3) -> Vec3 {
        self[0] * v.x + self[1] * v.y + self[2] * v.z
    }
}

impl Mul<Mat3> for Mat3 {
    type Output = Mat3;
    /// `mul_m3_m3m3`.
    #[inline]
    fn mul(self, rhs: Mat3) -> Mat3 {
        Mat3::from_cols(self * rhs[0], self * rhs[1], self * rhs[2])
    }
}

impl Mul<f32> for Mat3 {
    type Output = Mat3;
    #[inline]
    fn mul(self, s: f32) -> Mat3 {
        Mat3::from_cols(self[0] * s, self[1] * s, self[2] * s)
    }
}

impl Add for Mat3 {
    type Output = Mat3;
    #[inline]
    fn add(self, rhs: Mat3) -> Mat3 {
        Mat3::from_cols(self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2])
    }
}

impl Sub for Mat3 {
    type Output = Mat3;
    #[inline]
    fn sub(self, rhs: Mat3) -> Mat3 {
        Mat3::from_cols(self[0] - rhs[0], self[1] - rhs[1], self[2] - rhs[2])
    }
}

impl Neg for Mat3 {
    type Output = Mat3;
    #[inline]
    fn neg(self) -> Mat3 {
        Mat3::from_cols(-self[0], -self[1], -self[2])
    }
}

impl Neg for Mat4 {
    type Output = Mat4;
    #[inline]
    fn neg(self) -> Mat4 {
        Mat4::from_cols(-self[0], -self[1], -self[2], -self[3])
    }
}

impl From<Mat3> for Mat4 {
    /// `MatBase<T,4,4>(mat3)` — rotation block embedded, zero translation.
    #[inline]
    fn from(m: Mat3) -> Self {
        Self::from_cols(
            Vec4::new(m[0].x, m[0].y, m[0].z, 0.0),
            Vec4::new(m[1].x, m[1].y, m[1].z, 0.0),
            Vec4::new(m[2].x, m[2].y, m[2].z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }
}

impl From<Mat4> for Mat3 {
    /// `view<3,3>` — top-left rotation block.
    #[inline]
    fn from(m: Mat4) -> Self {
        Self::from_cols(
            Vec3::new(m[0][0], m[0][1], m[0][2]),
            Vec3::new(m[1][0], m[1][1], m[1][2]),
            Vec3::new(m[2][0], m[2][1], m[2][2]),
        )
    }
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        cols: [
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        ],
    };

    #[inline]
    pub const fn from_cols(c0: Vec4, c1: Vec4, c2: Vec4, c3: Vec4) -> Self {
        Self {
            cols: [c0, c1, c2, c3],
        }
    }

    /// `location()` — translation column xyz.
    #[inline]
    pub fn location(&self) -> Vec3 {
        Vec3::new(self[3][0], self[3][1], self[3][2])
    }

    /// Set translation (`loc`).
    #[inline]
    pub fn set_location(&mut self, loc: Vec3) {
        self.cols[3] = Vec4::new(loc.x, loc.y, loc.z, self[3][3]);
    }

    /// `from_location<Mat4>` — pure translation matrix.
    #[inline]
    pub fn from_location(loc: Vec3) -> Self {
        let mut m = Self::IDENTITY;
        m.set_location(loc);
        m
    }

    /// `from_scale<Mat4>`.
    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self::from(Mat3::from_scale(scale))
    }

    /// `from_loc_rot_scale` — translation + rotation (Mat3) + scale.
    pub fn from_loc_rot_scale(loc: Vec3, rot: Mat3, scale: Vec3) -> Self {
        let m = rot * Mat3::from_scale(scale);
        let mut out = Self::from(m);
        out.set_location(loc);
        out
    }

    /// `transpose_m4`.
    #[inline]
    pub fn transposed(self) -> Self {
        Self::from_cols(
            Vec4::new(self[0][0], self[1][0], self[2][0], self[3][0]),
            Vec4::new(self[0][1], self[1][1], self[2][1], self[3][1]),
            Vec4::new(self[0][2], self[1][2], self[2][2], self[3][2]),
            Vec4::new(self[0][3], self[1][3], self[2][3], self[3][3]),
        )
    }

    /// `determinant_m4`.
    pub fn determinant(self) -> f32 {
        let m = self;
        // Laplace expansion along the last row.
        let mut det = 0.0;
        for col in 0..4 {
            // minor: 3x3 determinant of rows 0..2 excluding `col`.
            let mut cols = [Vec3::ZERO; 3];
            let mut n = 0;
            for c in 0..4 {
                if c != col {
                    cols[n] = Vec3::new(m[c][0], m[c][1], m[c][2]);
                    n += 1;
                }
            }
            let minor = Mat3::from_cols(cols[0], cols[1], cols[2]).determinant();
            let sign = if (col + 3) % 2 == 0 { 1.0 } else { -1.0 };
            det += sign * m[col][3] * minor;
        }
        det
    }

    /// `invert_m4_m4` — general inverse; `None` when singular.
    pub fn inverted(self) -> Option<Self> {
        // Gauss-Jordan on [A | I].
        let mut a = [[0.0f32; 4]; 4];
        let mut inv = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        for c in 0..4 {
            for r in 0..4 {
                a[c][r] = self[c][r];
            }
        }
        for col in 0..4 {
            // Pivot on largest absolute value in this column across remaining rows.
            let mut pivot_row = col;
            let mut pivot_abs = a[col][col].abs();
            for row in (col + 1)..4 {
                if a[col][row].abs() > pivot_abs {
                    pivot_abs = a[col][row].abs();
                    pivot_row = row;
                }
            }
            if pivot_abs < f32::EPSILON {
                return None;
            }
            for c in 0..4 {
                a[c].swap(col, pivot_row);
                inv[c].swap(col, pivot_row);
            }
            let inv_pivot = 1.0 / a[col][col];
            for c in 0..4 {
                a[c][col] *= inv_pivot;
                inv[c][col] *= inv_pivot;
            }
            for row in 0..4 {
                if row == col {
                    continue;
                }
                let f = a[col][row];
                for c in 0..4 {
                    a[c][row] -= f * a[c][col];
                    inv[c][row] -= f * inv[c][col];
                }
            }
        }
        Some(Self::from_cols(
            Vec4::new(inv[0][0], inv[0][1], inv[0][2], inv[0][3]),
            Vec4::new(inv[1][0], inv[1][1], inv[1][2], inv[1][3]),
            Vec4::new(inv[2][0], inv[2][1], inv[2][2], inv[2][3]),
            Vec4::new(inv[3][0], inv[3][1], inv[3][2], inv[3][3]),
        ))
    }

    /// `transform_point` — apply to a point (implicit w = 1).
    #[inline]
    pub fn transform_point(self, p: Vec3) -> Vec3 {
        Vec3::new(
            self[0][0] * p.x + self[1][0] * p.y + self[2][0] * p.z + self[3][0],
            self[0][1] * p.x + self[1][1] * p.y + self[2][1] * p.z + self[3][1],
            self[0][2] * p.x + self[1][2] * p.y + self[2][2] * p.z + self[3][2],
        )
    }

    /// `transform_direction` — apply rotation part only (w = 0).
    #[inline]
    pub fn transform_direction(self, d: Vec3) -> Vec3 {
        Mat3::from(self) * d
    }

    /// `project_point` — transform and divide by w.
    pub fn project_point(self, p: Vec3) -> Vec3 {
        let x = self[0][0] * p.x + self[1][0] * p.y + self[2][0] * p.z + self[3][0];
        let y = self[0][1] * p.x + self[1][1] * p.y + self[2][1] * p.z + self[3][1];
        let z = self[0][2] * p.x + self[1][2] * p.y + self[2][2] * p.z + self[3][2];
        let w = self[0][3] * p.x + self[1][3] * p.y + self[2][3] * p.z + self[3][3];
        if w != 0.0 {
            Vec3::new(x / w, y / w, z / w)
        } else {
            Vec3::new(x, y, z)
        }
    }

    /// `translate_m4` — post-multiply translation: `self * T(d)`.
    #[inline]
    pub fn translated(self, d: Vec3) -> Self {
        let mut m = self;
        for r in 0..3 {
            m.cols[3][r] += self[0][r] * d.x + self[1][r] * d.y + self[2][r] * d.z;
        }
        m
    }

    /// `is_perspective` — `mat[2][3] == -1`.
    #[inline]
    pub fn is_perspective(self) -> bool {
        self[2][3] == -1.0
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;
    /// `mul_m4_v4` — `M * v`.
    #[inline]
    fn mul(self, v: Vec4) -> Vec4 {
        self[0] * v.x + self[1] * v.y + self[2] * v.z + self[3] * v.w
    }
}

impl Mul<Vec3> for Mat4 {
    type Output = Vec3;
    /// `mul_m4_v3` — transform point (implicit w = 1).
    #[inline]
    fn mul(self, v: Vec3) -> Vec3 {
        self.transform_point(v)
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Mat4;
    /// `mul_m4_m4m4` — `A * B` (B applied first).
    #[inline]
    fn mul(self, rhs: Mat4) -> Mat4 {
        Mat4::from_cols(self * rhs[0], self * rhs[1], self * rhs[2], self * rhs[3])
    }
}

impl Mul<f32> for Mat4 {
    type Output = Mat4;
    #[inline]
    fn mul(self, s: f32) -> Mat4 {
        Mat4::from_cols(self[0] * s, self[1] * s, self[2] * s, self[3] * s)
    }
}

/// `math::projection` — port of the projection matrix constructors.
pub mod projection {
    use super::*;

    /// `projection::orthographic`.
    pub fn orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near_clip: f32,
        far_clip: f32,
    ) -> Mat4 {
        let x_delta = right - left;
        let y_delta = top - bottom;
        let z_delta = far_clip - near_clip;
        let mut mat = Mat4::IDENTITY;
        if x_delta != 0.0 && y_delta != 0.0 && z_delta != 0.0 {
            mat[0][0] = 2.0 / x_delta;
            mat[3][0] = -(right + left) / x_delta;
            mat[1][1] = 2.0 / y_delta;
            mat[3][1] = -(top + bottom) / y_delta;
            mat[2][2] = -2.0 / z_delta; // negate Z
            mat[3][2] = -(far_clip + near_clip) / z_delta;
        }
        mat
    }

    /// `projection::perspective`.
    pub fn perspective(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near_clip: f32,
        far_clip: f32,
    ) -> Mat4 {
        let x_delta = right - left;
        let y_delta = top - bottom;
        let z_delta = far_clip - near_clip;
        let mut mat = Mat4::IDENTITY;
        if x_delta != 0.0 && y_delta != 0.0 && z_delta != 0.0 {
            mat[0][0] = near_clip * 2.0 / x_delta;
            mat[1][1] = near_clip * 2.0 / y_delta;
            mat[2][0] = (right + left) / x_delta;
            mat[2][1] = (top + bottom) / y_delta;
            mat[2][2] = -(far_clip + near_clip) / z_delta;
            mat[2][3] = -1.0;
            mat[3][2] = (-2.0 * near_clip * far_clip) / z_delta;
            mat[3][3] = 0.0;
        }
        mat
    }

    /// `projection::perspective_fov`.
    pub fn perspective_fov(
        angle_left: f32,
        angle_right: f32,
        angle_bottom: f32,
        angle_top: f32,
        near_clip: f32,
        far_clip: f32,
    ) -> Mat4 {
        let mut mat = perspective(
            angle_left.tan(),
            angle_right.tan(),
            angle_bottom.tan(),
            angle_top.tan(),
            near_clip,
            far_clip,
        );
        mat[0][0] /= near_clip;
        mat[1][1] /= near_clip;
        mat
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn mat4_transform_and_inverse() {
        let m = Mat4::from_loc_rot_scale(
            Vec3::new(1.0, 2.0, 3.0),
            Mat3::IDENTITY,
            Vec3::new(2.0, 2.0, 2.0),
        );
        let p = m.transform_point(Vec3::new(1.0, 1.0, 1.0));
        assert!((p - Vec3::new(3.0, 4.0, 5.0)).length() < 1e-6);
        let inv = m.inverted().unwrap();
        let back = inv.transform_point(p);
        assert!((back - Vec3::new(1.0, 1.0, 1.0)).length() < 1e-5);
    }

    #[test]
    fn mat3_mul_transpose() {
        let m = Mat3::from_cols(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0), Vec3::new(7.0, 8.0, 9.0));
        let t = m.transposed();
        assert_eq!(t[0][0], m[0][0]);
        assert_eq!(t[0][1], m[1][0]);
    }

    #[test]
    fn perspective_maps() {
        let m = projection::perspective(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0);
        assert!(m.is_perspective());
        // Near-plane point projects to -1 z in NDC.
        let p = m.project_point(Vec3::new(0.0, 0.0, -0.1));
        assert!(approx(p.z, -1.0));
    }

    #[test]
    fn det_and_inverse_mat3() {
        let r = Mat3::from_cols(
            Vec3::new(FRAC_PI_2.cos(), FRAC_PI_2.sin(), 0.0),
            Vec3::new(-FRAC_PI_2.sin(), FRAC_PI_2.cos(), 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        );
        assert!(approx(r.determinant(), 1.0));
        let v = r * Vec3::X;
        assert!((v - Vec3::Y).length() < 1e-5);
        assert!(approx((r.inverted().unwrap() * r)[0].x, 1.0));
    }
}

/* ---- math_matrix.cc (Eigen-backed parts, ported natively) ---- */

/// Jacobi eigen-decomposition of a symmetric 3x3. Returns (eigenvectors-as-cols, eigenvalues).
fn jacobi_eigen3(a: Mat3) -> (Mat3, [f32; 3]) {
    // a is indexed [col][row]; symmetric so m[i][j] == m[j][i].
    let mut m = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = a[j][i];
        }
    }
    let mut v = [[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]; // v[col][row]
    for _ in 0..50 {
        // largest off-diagonal
        let (mut p, mut q, mut max) = (0usize, 1usize, m[0][1].abs());
        for (i, j) in [(0, 2), (1, 2)] {
            if m[i][j].abs() > max {
                max = m[i][j].abs();
                p = i;
                q = j;
            }
        }
        if max < 1e-10 {
            break;
        }
        // Jacobi rotation angle
        let app = m[p][p];
        let aqq = m[q][q];
        let apq = m[p][q];
        let theta = 0.5 * (aqq - app).atan2(2.0 * apq);
        let c = theta.cos();
        let s = theta.sin();
        for i in 0..3 {
            let mip = m[i][p];
            let miq = m[i][q];
            m[i][p] = c * mip - s * miq;
            m[i][q] = s * mip + c * miq;
        }
        for j in 0..3 {
            let mpj = m[p][j];
            let mqj = m[q][j];
            m[p][j] = c * mpj - s * mqj;
            m[q][j] = s * mpj + c * mqj;
        }
        for i in 0..3 {
            let vip = v[i][p];
            let viq = v[i][q];
            v[i][p] = c * vip - s * viq;
            v[i][q] = s * vip + c * viq;
        }
    }
    let evals = [m[0][0], m[1][1], m[2][2]];
    let vecs = Mat3::from_cols(
        Vec3::new(v[0][0], v[0][1], v[0][2]),
        Vec3::new(v[1][0], v[1][1], v[1][2]),
        Vec3::new(v[2][0], v[2][1], v[2][2]),
    );
    (vecs, evals)
}

/// One-sided 3x3 SVD: `m = U * diag(s) * V^T`. `s` sorted descending.
fn svd3(m: Mat3) -> (Mat3, Vec3, Mat3) {
    let ata = m.transposed() * m;
    let (mut v, evals) = jacobi_eigen3(ata);
    let mut s = Vec3::new(
        evals[0].max(0.0).sqrt(),
        evals[1].max(0.0).sqrt(),
        evals[2].max(0.0).sqrt(),
    );
    // sort descending via axis index order
    let mut order = [0usize, 1, 2];
    order.sort_by(|&i, &j| s[j].partial_cmp(&s[i]).unwrap_or(std::cmp::Ordering::Equal));
    let sorted_cols = [v[order[0]], v[order[1]], v[order[2]]];
    v = Mat3::from_cols(sorted_cols[0], sorted_cols[1], sorted_cols[2]);
    s = Vec3::new(s[order[0]], s[order[1]], s[order[2]]);

    // U = M*V*S^-1 with degenerate columns repaired via cross products.
    let mut cols = [Vec3::ZERO; 3];
    for i in 0..3 {
        if s[i] > 1e-12 {
            cols[i] = m * v[i] / s[i];
        }
    }
    for i in 0..3 {
        if s[i] <= 1e-12 {
            let a = cols[(i + 1) % 3];
            let b = cols[(i + 2) % 3];
            let mut c = a.cross(b);
            if c.length_squared() < 1e-20 {
                // fully degenerate: pick an arbitrary orthogonal unit vector
                c = if a.length_squared() > 1e-20 {
                    a.orthogonal()
                } else {
                    Vec3::X
                };
            }
            cols[i] = c.normalized();
        }
    }
    let mut u = Mat3::from_cols(cols[0], cols[1], cols[2]);
    // keep U right-handed
    if u.determinant() < 0.0 {
        u = Mat3::from_cols(u[0], u[1], -u[2]);
        s.z = -s.z;
    }
    (u, s, v)
}

impl Mat3 {
    /// `adjoint` — adjugate (transpose of cofactor matrix).
    pub fn adjoint(self) -> Self {
        let m = self;
        Mat3::from_cols(
            Vec3::new(
                m[1][1] * m[2][2] - m[1][2] * m[2][1],
                -(m[1][0] * m[2][2] - m[1][2] * m[2][0]),
                m[1][0] * m[2][1] - m[1][1] * m[2][0],
            ),
            Vec3::new(
                -(m[0][1] * m[2][2] - m[0][2] * m[2][1]),
                m[0][0] * m[2][2] - m[0][2] * m[2][0],
                -(m[0][0] * m[2][1] - m[0][1] * m[2][0]),
            ),
            Vec3::new(
                m[0][1] * m[1][2] - m[0][2] * m[1][1],
                -(m[0][0] * m[1][2] - m[0][2] * m[1][0]),
                m[0][0] * m[1][1] - m[0][1] * m[1][0],
            ),
        )
    }

    /// `pseudo_invert` — normal inverse, else Moore-Penrose via Jacobi SVD.
    pub fn pseudo_invert(self, epsilon: f32) -> Self {
        if let Some(inv) = self.inverted() {
            return inv;
        }
        let (u, s, v) = svd3(self);
        let w = Mat3::from_scale(Vec3::new(
            if s.x < epsilon { 0.0 } else { 1.0 / s.x },
            if s.y < epsilon { 0.0 } else { 1.0 / s.y },
            if s.z < epsilon { 0.0 } else { 1.0 / s.z },
        ));
        (v * w) * u.transposed()
    }

    /// `polar_decompose` — M = U*P; U orthonormal (rotation-like), P symmetric.
    pub fn polar_decompose(self) -> (Mat3, Mat3) {
        let (w, s, v) = svd3(self);
        let s_mat = Mat3::from_scale(s);
        let vt = v.transposed();
        (w * vt, (v * s_mat) * vt)
    }

    /// `is_similarity_transform` — no shear, uniform scale.
    pub fn is_similarity_transform(self, epsilon: f32) -> bool {
        if self[0].dot(self[1]).abs() > epsilon
            || self[0].dot(self[2]).abs() > epsilon
            || self[1].dot(self[2]).abs() > epsilon
        {
            return false;
        }
        let l0 = self[0].length_squared();
        let l1 = self[1].length_squared();
        let l2 = self[2].length_squared();
        (l0 - l1).abs() <= epsilon && (l0 - l2).abs() <= epsilon && (l1 - l2).abs() <= epsilon
    }

    /// `to_rot_scale<true>` — decompose into unit-quaternion + (possibly negative) scale.
    pub fn to_quat_scale(self) -> (crate::quaternion::Quaternion, Vec3) {
        let mut m = self;
        let scale = m.normalize_and_get_size();
        if m.is_negative() {
            m = -m;
            return (crate::quaternion::Quaternion::from_mat3(m), -scale);
        }
        (crate::quaternion::Quaternion::from_mat3(m), scale)
    }

    /// `interpolate` — polar-decomposed matrix interpolation (rot slerp, scale lerp).
    pub fn interpolate(a: Mat3, b: Mat3, t: f32) -> Mat3 {
        let (mut u_a, mut p_a) = a.polar_decompose();
        let (mut u_b, mut p_b) = b.polar_decompose();
        if u_a.is_negative() {
            u_a = -u_a;
            p_a = -p_a;
        }
        if u_b.is_negative() {
            u_b = -u_b;
            p_b = -p_b;
        }
        let quat_a = crate::quaternion::Quaternion::from_mat3(u_a.normalized());
        let quat_b = crate::quaternion::Quaternion::from_mat3(u_b.normalized());
        let quat = quat_a.slerp(quat_b, t);
        let u = quat.to_mat3();
        let p = p_a + (p_b - p_a) * t;
        u * p
    }

    /// `interpolate_fast` — normalize-column decompose, then slerp/lerp.
    pub fn interpolate_fast(a: Mat3, b: Mat3, t: f32) -> Mat3 {
        let (a_quat, a_scale) = a.to_quat_scale();
        let (b_quat, b_scale) = b.to_quat_scale();
        let scale = a_scale + (b_scale - a_scale) * t;
        let rotation = a_quat.slerp(b_quat, t);
        rotation.to_mat3() * Mat3::from_scale(scale)
    }
}

impl Mat4 {
    /// `interpolate` — 3x3 part via polar decompose, location lerped.
    pub fn interpolate(a: Mat4, b: Mat4, t: f32) -> Mat4 {
        let a3 = Mat3::from_cols(a[0].xyz(), a[1].xyz(), a[2].xyz());
        let b3 = Mat3::from_cols(b[0].xyz(), b[1].xyz(), b[2].xyz());
        let m3 = Mat3::interpolate(a3, b3, t);
        let loc = a.location() + (b.location() - a.location()) * t;
        let mut result = Mat4::from_cols(
            Vec4::new(m3[0].x, m3[0].y, m3[0].z, 0.0),
            Vec4::new(m3[1].x, m3[1].y, m3[1].z, 0.0),
            Vec4::new(m3[2].x, m3[2].y, m3[2].z, 0.0),
            Vec4::new(loc.x, loc.y, loc.z, 1.0),
        );
        result[3][3] = 1.0;
        result
    }

    /// `interpolate_fast` — loc/rot/scale decompose, slerp + lerp.
    pub fn interpolate_fast(a: Mat4, b: Mat4, t: f32) -> Mat4 {
        let a3 = Mat3::from_cols(a[0].xyz(), a[1].xyz(), a[2].xyz());
        let b3 = Mat3::from_cols(b[0].xyz(), b[1].xyz(), b[2].xyz());
        let (a_quat, a_scale) = a3.to_quat_scale();
        let (b_quat, b_scale) = b3.to_quat_scale();
        let loc = a.location() + (b.location() - a.location()) * t;
        let scale = a_scale + (b_scale - a_scale) * t;
        let rotation = a_quat.slerp(b_quat, t);
        Mat4::from_loc_rot_scale(loc, rotation.to_mat3(), scale)
    }

    /// `projection::perspective_infinite` — Lengyel "Projection Matrix Tricks".
    pub fn perspective_infinite(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near_clip: f32,
    ) -> Self {
        let x_delta = right - left;
        let y_delta = top - bottom;
        let mut mat = Mat4::IDENTITY;
        if x_delta != 0.0 && y_delta != 0.0 {
            const EPS: f32 = 2.4e-7;
            mat[0][0] = near_clip * 2.0 / x_delta;
            mat[1][1] = near_clip * 2.0 / y_delta;
            mat[2][0] = (right + left) / x_delta;
            mat[2][1] = (top + bottom) / y_delta;
            mat[2][2] = -1.0;
            mat[2][3] = EPS - 1.0;
            mat[3][2] = (EPS - 2.0) * near_clip;
            mat[3][3] = 0.0;
        }
        mat
    }
}

/// `to_quaternion_legacy` — rotate z-axis then align x-axis.
pub fn to_quaternion_legacy(mat: Mat3) -> crate::quaternion::Quaternion {
    use crate::quaternion::Quaternion;
    let n_mat = mat.normalized();
    let z_axis = n_mat.z_axis();
    let nor = Vec3::new(z_axis.y, -z_axis.x, 0.0).normalized();
    let ha = 0.5 * z_axis.z.clamp(-1.0, 1.0).acos();
    let q1 = { let v = -nor * ha.sin(); Quaternion { w: ha.cos(), x: v.x, y: v.y, z: v.z } };
    let x_axis = q1.conjugated().rotate_vec3(n_mat.x_axis());
    let ha2 = 0.5 * x_axis.y.atan2(x_axis.x);
    let q2 = Quaternion { w: ha2.cos(), x: 0.0, y: 0.0, z: ha2.sin() };
    q1 * q2
}

fn is_identity_eps(m: &Mat3, eps: f32) -> bool {
    for c in 0..3 {
        for r in 0..3 {
            let target = if c == r { 1.0 } else { 0.0 };
            if (m[c][r] - target).abs() > eps {
                return false;
            }
        }
    }
    true
}

fn is_identity4_eps(m: &Mat4, eps: f32) -> bool {
    for c in 0..4 {
        for r in 0..4 {
            let target = if c == r { 1.0 } else { 0.0 };
            if (m[c][r] - target).abs() > eps {
                return false;
            }
        }
    }
    true
}

/// `transform_normals` — apply inverse-transpose rotation to `normals` in place.
pub fn transform_normals(transform: Mat3, normals: &mut [Vec3]) {
    if is_identity_eps(&transform, 1e-6) {
        return;
    }
    let normal_transform = transform.inverted().unwrap_or(transform).transposed();
    if normal_transform.is_similarity_transform(1e-6) {
        let nt = normal_transform.normalized();
        let (x, y, z) = (nt.x_axis(), nt.y_axis(), nt.z_axis());
        for n in normals.iter_mut() {
            let n0 = *n;
            *n = n0.x * x + n0.y * y + n0.z * z;
        }
    } else {
        let (x, y, z) = (
            normal_transform.x_axis(),
            normal_transform.y_axis(),
            normal_transform.z_axis(),
        );
        for n in normals.iter_mut() {
            let n0 = *n;
            *n = (n0.x * x + n0.y * y + n0.z * z).normalized();
        }
    }
}

/// `transform_normals(src, transform, dst)` — non-in-place variant.
pub fn transform_normals_to(src: &[Vec3], transform: Mat3, dst: &mut [Vec3]) {
    if is_identity_eps(&transform, 1e-6) {
        dst.copy_from_slice(src);
        return;
    }
    let normal_transform = transform.inverted().unwrap_or(transform).transposed();
    if normal_transform.is_similarity_transform(1e-6) {
        let nt = normal_transform.normalized();
        let (x, y, z) = (nt.x_axis(), nt.y_axis(), nt.z_axis());
        for (d, n) in dst.iter_mut().zip(src.iter()) {
            *d = n.x * x + n.y * y + n.z * z;
        }
    } else {
        let (x, y, z) = (
            normal_transform.x_axis(),
            normal_transform.y_axis(),
            normal_transform.z_axis(),
        );
        for (d, n) in dst.iter_mut().zip(src.iter()) {
            *d = (n.x * x + n.y * y + n.z * z).normalized();
        }
    }
}

/// `transform_points(src, transform, dst)`.
pub fn transform_points_to(src: &[Vec3], transform: Mat4, dst: &mut [Vec3]) {
    if is_identity4_eps(&transform, 1e-6) {
        dst.copy_from_slice(src);
        return;
    }
    let (x, y, z) = (transform[0].xyz(), transform[1].xyz(), transform[2].xyz());
    let loc = transform.location();
    for (d, p) in dst.iter_mut().zip(src.iter()) {
        *d = p.x * x + p.y * y + p.z * z + loc;
    }
}

/// `transform_points(transform, points)` — in-place variant.
pub fn transform_points(transform: Mat4, points: &mut [Vec3]) {
    if is_identity4_eps(&transform, 1e-6) {
        return;
    }
    let (x, y, z) = (transform[0].xyz(), transform[1].xyz(), transform[2].xyz());
    let loc = transform.location();
    for p in points.iter_mut() {
        let p0 = *p;
        *p = p0.x * x + p0.y * y + p0.z * z + loc;
    }
}
