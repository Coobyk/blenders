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
