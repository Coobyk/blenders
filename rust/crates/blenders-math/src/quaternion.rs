//! Port of `BLI_math_quaternion.hh` / `BLI_math_axis_angle.hh` / `math_rotation.cc`.
//!
//! `Quaternion` stores `(w, x, y, z)` exactly like `QuaternionBase`.

use std::ops::{Index, IndexMut, Mul, Neg};

use crate::base;
use crate::matrix::Mat3;
use crate::vector::{Vec3, Vec4};

/// `QuaternionBase<float>` — (w, x, y, z).
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// `AxisAngleBase<float, AngleRadian>` — normalized axis + angle (radians).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisAngle {
    pub axis: Vec3,
    pub angle: f32,
}

impl Index<usize> for Quaternion {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.w,
            1 => &self.x,
            2 => &self.y,
            3 => &self.z,
            _ => panic!("quaternion index {i} out of range"),
        }
    }
}

impl IndexMut<usize> for Quaternion {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.w,
            1 => &mut self.x,
            2 => &mut self.y,
            3 => &mut self.z,
            _ => panic!("quaternion index {i} out of range"),
        }
    }
}

impl Quaternion {
    /// `identity()`.
    pub const IDENTITY: Self = Self {
        w: 1.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    #[inline]
    pub const fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    /// `dot_qtqt`.
    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.w * rhs.w + self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// `len_qt`.
    #[inline]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// `is_unit_scale`.
    #[inline]
    pub fn is_unit(self) -> bool {
        (self.length() - 1.0).abs() < 0.0002
    }

    /// `normalize_qt` — returns identity when degenerate.
    #[inline]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len != 0.0 {
            let inv = 1.0 / len;
            Self::new(self.w * inv, self.x * inv, self.y * inv, self.z * inv)
        } else {
            Self::IDENTITY
        }
    }

    /// `conjugate_qt`.
    #[inline]
    pub fn conjugated(self) -> Self {
        Self::new(self.w, -self.x, -self.y, -self.z)
    }

    /// `invert_qt` — inverse (conjugate / |q|²).
    #[inline]
    pub fn inverted(self) -> Self {
        let n2 = self.dot(self);
        let c = self.conjugated();
        if n2 != 0.0 {
            Self::new(c.w / n2, c.x / n2, c.y / n2, c.z / n2)
        } else {
            Self::IDENTITY
        }
    }

    /// `to_quaternion(AxisAngle)` — `axis_angle_normalized_to_quat` semantics.
    /// Axis is assumed normalized (assert in Blender).
    pub fn from_axis_angle(aa: AxisAngle) -> Self {
        let half = aa.angle * 0.5;
        let (hs, hc) = half.sin_cos();
        Self::new(hc, aa.axis.x * hs, aa.axis.y * hs, aa.axis.z * hs)
    }

    /// `to_axis_angle(Quaternion)` — faithful port: axis from xyz, angle via
    /// `AngleT(cos, sin) * 2`; degenerate sin→identity axis.
    pub fn to_axis_angle(self) -> AxisAngle {
        let mut axis = Vec3::new(self.x, self.y, self.z);
        let cos_half = self.w;
        let sin_half = axis.length();
        if sin_half < 0.0005 {
            // Degenerate: identity axis scaled by sign of w.
            return AxisAngle {
                axis: Vec3::new(base::sign(cos_half), 0.0, 0.0),
                angle: 0.0,
            };
        }
        axis = axis / sin_half;
        AxisAngle {
            axis,
            angle: 2.0 * sin_half.atan2(cos_half),
        }
    }

    /// `from_rotation<Mat3>(quat)` → port of `quat_to_mat3`.
    /// Uses the M_SQRT2 formulation from `math_rotation.cc`.
    pub fn to_mat3(self) -> Mat3 {
        const SQRT2: f64 = std::f64::consts::SQRT_2;
        let q0 = SQRT2 * self.w as f64;
        let q1 = SQRT2 * self.x as f64;
        let q2 = SQRT2 * self.y as f64;
        let q3 = SQRT2 * self.z as f64;

        let qda = q0 * q1;
        let qdb = q0 * q2;
        let qdc = q0 * q3;
        let qaa = q1 * q1;
        let qab = q1 * q2;
        let qac = q1 * q3;
        let qbb = q2 * q2;
        let qbc = q2 * q3;
        let qcc = q3 * q3;

        // Columns of the rotation matrix.
        Mat3::from_cols(
            Vec3::new(
                (1.0 - (qbb + qcc)) as f32,
                (qdc + qab) as f32,
                (-qdb + qac) as f32,
            ),
            Vec3::new(
                (-qdc + qab) as f32,
                (1.0 - (qaa + qcc)) as f32,
                (qda + qbc) as f32,
            ),
            Vec3::new(
                (qdb + qac) as f32,
                (-qda + qbc) as f32,
                (1.0 - (qaa + qbb)) as f32,
            ),
        )
    }

    /// `to_mat4` — embed `to_mat3` in a 4x4.
    #[inline]
    pub fn to_mat4(self) -> crate::matrix::Mat4 {
        self.to_mat3().into()
    }

    /// `mul_qt_v3` — rotate a vector: `v' = q * v * q⁻¹` (unit quat fast path).
    pub fn rotate_vec3(self, v: Vec3) -> Vec3 {
        // Standard fast path equivalent to Blender's quat_mul_vec3.
        let qv = Vec3::new(self.x, self.y, self.z);
        let t = qv.cross(v) * 2.0;
        v + t * self.w + qv.cross(t)
    }

    /// `slerp_qt_qt` — spherical interpolation between two unit quaternions.
    pub fn slerp(self, other: Self, t: f32) -> Self {
        let mut q2 = other;
        let mut cosom = self.dot(q2);
        // Take the shorter arc.
        if cosom < 0.0 {
            cosom = -cosom;
            q2 = -q2;
        }
        let (scale0, scale1);
        if (1.0 - cosom) > 1e-6 {
            let omega = cosom.acos();
            let sinom = omega.sin();
            scale0 = ((1.0 - t) * omega).sin() / sinom;
            scale1 = (t * omega).sin() / sinom;
        } else {
            // Linear fallback for nearly identical quats.
            scale0 = 1.0 - t;
            scale1 = t;
        }
        Self::new(
            scale0 * self.w + scale1 * q2.w,
            scale0 * self.x + scale1 * q2.x,
            scale0 * self.y + scale1 * q2.y,
            scale0 * self.z + scale1 * q2.z,
        )
    }

    /// `expmap` — rotation vector (axis * angle).
    pub fn expmap(self) -> Vec3 {
        let aa = self.to_axis_angle();
        aa.axis * aa.angle
    }

    /// `QuaternionBase::expmap` — from rotation vector.
    pub fn from_expmap(expmap: Vec3) -> Self {
        let mut v = expmap;
        let angle = v.normalize_and_get_length();
        if angle != 0.0 {
            Self::from_axis_angle(AxisAngle {
                axis: v,
                angle: base::wrap_angle(angle),
            })
        } else {
            Self::IDENTITY
        }
    }
}

/// `mul_qt_qtqt(a, b)` — Hamilton product; `a ⊗ b` applies `b` first.
impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn mul(self, b: Quaternion) -> Quaternion {
        let a = self;
        Quaternion::new(
            a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
            a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
            a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
            a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        )
    }
}

impl Mul<Vec3> for Quaternion {
    type Output = Vec3;
    #[inline]
    fn mul(self, v: Vec3) -> Vec3 {
        self.rotate_vec3(v)
    }
}

impl Mul<f32> for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn mul(self, s: f32) -> Quaternion {
        Quaternion::new(self.w * s, self.x * s, self.y * s, self.z * s)
    }
}

impl Neg for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn neg(self) -> Quaternion {
        Quaternion::new(-self.w, -self.x, -self.y, -self.z)
    }
}

/// `mat3_normalized_to_quat` — port of `normalized_to_quat_with_checks`
/// + `normalized_to_quat_fast` from `BLI_math_matrix.hh`.
impl From<Mat3> for Quaternion {
    fn from(mat: Mat3) -> Self {
        let det = mat.determinant();
        if !det.is_finite() {
            return Self::IDENTITY;
        }
        let m = if det < 0.0 { -mat } else { mat };

        let mut q = Vec4::ZERO;

        if m[2][2] < 0.0 {
            if m[0][0] > m[1][1] {
                let trace = 1.0 + m[0][0] - m[1][1] - m[2][2];
                let mut s = 2.0 * trace.sqrt();
                if m[1][2] < m[2][1] {
                    s = -s; // ensure W non-negative
                }
                q.x = 0.25 * s;
                let s_inv = 1.0 / s;
                q.w = (m[1][2] - m[2][1]) * s_inv;
                q.y = (m[0][1] + m[1][0]) * s_inv;
                q.z = (m[2][0] + m[0][2]) * s_inv;
                if trace == 1.0 && q.w == 0.0 && q.y == 0.0 && q.z == 0.0 {
                    q.x = 1.0;
                }
            } else {
                let trace = 1.0 - m[0][0] + m[1][1] - m[2][2];
                let mut s = 2.0 * trace.sqrt();
                if m[2][0] < m[0][2] {
                    s = -s;
                }
                q.y = 0.25 * s;
                let s_inv = 1.0 / s;
                q.w = (m[2][0] - m[0][2]) * s_inv;
                q.x = (m[0][1] + m[1][0]) * s_inv;
                q.z = (m[1][2] + m[2][1]) * s_inv;
                if trace == 1.0 && q.w == 0.0 && q.x == 0.0 && q.z == 0.0 {
                    q.y = 1.0;
                }
            }
        } else {
            if m[0][0] < -m[1][1] {
                let trace = 1.0 - m[0][0] - m[1][1] + m[2][2];
                let mut s = 2.0 * trace.sqrt();
                if m[0][1] < m[1][0] {
                    s = -s;
                }
                q.z = 0.25 * s;
                let s_inv = 1.0 / s;
                q.w = (m[0][1] - m[1][0]) * s_inv;
                q.x = (m[2][0] + m[0][2]) * s_inv;
                q.y = (m[1][2] + m[2][1]) * s_inv;
                if trace == 1.0 && q.w == 0.0 && q.x == 0.0 && q.y == 0.0 {
                    q.z = 1.0;
                }
            } else {
                // Zero matrix falls through here (see #101848).
                let trace = 1.0 + m[0][0] + m[1][1] + m[2][2];
                let s = 2.0 * trace.sqrt();
                q.w = 0.25 * s;
                let s_inv = 1.0 / s;
                q.x = (m[1][2] - m[2][1]) * s_inv;
                q.y = (m[2][0] - m[0][2]) * s_inv;
                q.z = (m[0][1] - m[1][0]) * s_inv;
                if trace == 1.0 && q.x == 0.0 && q.y == 0.0 && q.z == 0.0 {
                    q.w = 1.0;
                }
            }
        }

        // Renormalize if far from unit (round-off in the branches above).
        let q_len_sq = q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w;
        const THRESHOLD: f32 = 0.0002; // BLI_ASSERT_UNIT_EPSILON
        if (q_len_sq - 1.0).abs() >= THRESHOLD {
            let inv = 1.0 / q_len_sq.sqrt();
            q = q * inv;
        }
        Quaternion::new(q.w, q.x, q.y, q.z)
    }
}

/// `rotation_between_vecs_to_quat` — shortest rotation from `a` to `b`.
pub fn rotation_between_vecs_to_quat(from: Vec3, to: Vec3) -> Quaternion {
    let a = from.normalized();
    let b = to.normalized();
    let d = a.dot(b);
    if d > 1.0 - 1e-6 {
        return Quaternion::IDENTITY;
    }
    if d < -1.0 + 1e-6 {
        // Opposed: rotate 180° about any orthogonal axis.
        return Quaternion::from_axis_angle(AxisAngle {
            axis: a.orthogonal(),
            angle: std::f32::consts::PI,
        });
    }
    let axis = a.cross(b);
    // Quaternion from half-angle via axis*sin(θ/2).
    // (1+cos θ) = w·2, sin θ/2 components proportional to axis·sin(θ/2).
    let w = (1.0 + d).sqrt();
    let inv_2w = 1.0 / (2.0 * w);
    Quaternion::new(w * 0.5, axis.x * inv_2w, axis.y * inv_2w, axis.z * inv_2w).normalized()
}

/// `axis_angle_normalized_to_mat3` — Rodrigues' formula, axis assumed unit.
pub fn mat3_from_axis_angle(aa: AxisAngle) -> Mat3 {
    let axis = aa.axis;
    let angle = aa.angle;
    // AxisAngleCartesian caches sin/cos in Blender; compute here.
    let (sin_a, cos_a) = angle.sin_cos();
    let nsi = 1.0 - cos_a;
    let (x, y, z) = (axis.x, axis.y, axis.z);

    let ic0 = x * x * nsi;
    let ic1 = x * y * nsi;
    let ic2 = x * z * nsi;
    let ic3 = y * y * nsi;
    let ic4 = y * z * nsi;
    let ic5 = z * z * nsi;

    let ic6 = x * sin_a;
    let ic7 = y * sin_a;
    let ic8 = z * sin_a;

    Mat3::from_cols(
        Vec3::new(ic0 + cos_a, ic1 + ic8, ic2 - ic7),
        Vec3::new(ic1 - ic8, ic3 + cos_a, ic4 + ic6),
        Vec3::new(ic2 + ic7, ic4 - ic6, ic5 + cos_a),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::Mat3;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn axis_angle_roundtrip() {
        let aa = AxisAngle {
            axis: Vec3::Z,
            angle: FRAC_PI_2,
        };
        let q = Quaternion::from_axis_angle(aa);
        let rotated = q.rotate_vec3(Vec3::X);
        assert!((rotated - Vec3::Y).length() < 1e-6, "{rotated:?}");
        let aa2 = q.to_axis_angle();
        assert!(approx(aa2.angle, FRAC_PI_2));
        assert!((aa2.axis - Vec3::Z).length() < 1e-5);
    }

    #[test]
    fn quat_mat_roundtrip() {
        let q = Quaternion::from_axis_angle(AxisAngle {
            axis: Vec3::new(1.0, 1.0, 1.0).normalized(),
            angle: 0.7,
        });
        let m = q.to_mat3();
        let q2 = Quaternion::from(m);
        // Quats are sign-ambiguous; compare rotation effect.
        let v = Vec3::new(0.3, -0.8, 1.2);
        assert!((q.rotate_vec3(v) - q2.rotate_vec3(v)).length() < 1e-5);
    }

    #[test]
    fn between_vecs() {
        let q = rotation_between_vecs_to_quat(Vec3::X, Vec3::Y);
        assert!((q.rotate_vec3(Vec3::X) - Vec3::Y).length() < 1e-5);
    }

    #[test]
    fn identity_mat() {
        let q = Quaternion::from(Mat3::IDENTITY);
        assert!(approx(q.w, 1.0));
    }

    #[test]
    fn slerp_midpoint() {
        // 0° → 90° about Z: midpoint rotates X by 45°.
        let a = Quaternion::IDENTITY;
        let b = Quaternion::from_axis_angle(AxisAngle { axis: Vec3::Z, angle: FRAC_PI_2 });
        let mid = a.slerp(b, 0.5);
        let v = mid.rotate_vec3(Vec3::X);
        let expected = Vec3::new(FRAC_PI_4.cos(), FRAC_PI_4.sin(), 0.0);
        assert!((v - expected).length() < 1e-5, "{v:?} vs {expected:?}");
    }
}

impl Quaternion {
    /// `mat3_to_quat` / `normalized_to_quat_with_checks` — rotation matrix to
    /// unit quaternion (trace-based branch selection, result normalized).
    pub fn from_mat3(mat: crate::matrix::Mat3) -> Self {
        let m = |c: usize, r: usize| mat[c][r];
        let tr = 0.25 * (1.0 + m(0, 0) + m(1, 1) + m(2, 2));
        let mut q = if tr > f32::EPSILON {
            let s = tr.sqrt();
            let inv = 1.0 / (4.0 * s);
            Quaternion {
                w: s,
                x: (m(1, 2) - m(2, 1)) * inv,
                y: (m(2, 0) - m(0, 2)) * inv,
                z: (m(0, 1) - m(1, 0)) * inv,
            }
        } else {
            let (mut i, mut j, mut k) = (0usize, 1usize, 2usize);
            if m(1, 1) > m(0, 0) {
                i = 1;
                j = 2;
                k = 0;
            }
            if m(2, 2) > m(i, i) {
                i = 2;
                j = 0;
                k = 1;
            }
            let tr = ((m(i, i) - (m(j, j) + m(k, k))) + 1.0).max(0.0).sqrt();
            let inv = 0.5 / tr;
            let mut qv = [0.0f32; 3];
            qv[i] = 0.5 * tr;
            qv[j] = (m(i, j) + m(j, i)) * inv;
            qv[k] = (m(i, k) + m(k, i)) * inv;
            Quaternion {
                w: (m(j, k) - m(k, j)) * inv,
                x: qv[0],
                y: qv[1],
                z: qv[2],
            }
        };
        q = q.normalized();
        q
    }
}
