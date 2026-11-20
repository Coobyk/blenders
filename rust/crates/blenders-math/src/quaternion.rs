//! Port of `BLI_math_quaternion.hh` / `BLI_math_axis_angle.hh` / `math_rotation.cc`.
//!
//! `Quaternion` stores `(w, x, y, z)` exactly like `QuaternionBase`.

use std::ops::{Add, Index, IndexMut, Mul, Neg};
use crate::base::FLT_EPSILON;

use crate::base;
use crate::matrix::{Mat3, Mat4};
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
impl Add for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn add(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            w: self.w + rhs.w,
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

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

/* ---- math_rotation_c.cc: quaternion angle/track/interp functions ---- */

/// `rotation_between_quats_to_quat` — q = q1^-1 * q2.
pub fn rotation_between_quats_to_quat(q1: Quaternion, q2: Quaternion) -> Quaternion {
    let mut tquat = q1.conjugated();
    tquat = tquat * (1.0 / tquat.dot(tquat));
    tquat * q2
}

/// `quat_split_swing_and_twist` — returns (twist_angle, Option<swing>, Option<twist>).
pub fn quat_split_swing_and_twist(
    q_in: Quaternion,
    axis: usize,
    want_swing: bool,
    want_twist: bool,
) -> (f32, Option<Quaternion>, Option<Quaternion>) {
    debug_assert!(axis <= 2);
    let q = if q_in.w < 0.0 { -q_in } else { q_in };
    let t = q_in_axis(q, axis).atan2(q.w);

    let mut swing = None;
    let mut twist = None;
    if want_swing || want_twist {
        let (sin_t, cos_t) = t.sin_cos();
        if want_swing {
            let mut twist_inv = Quaternion {
                w: cos_t,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            twist_inv[axis + 1] = -sin_t;
            swing = Some(q * twist_inv);
        }
        if want_twist {
            let mut tq = Quaternion {
                w: cos_t,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            tq[axis + 1] = sin_t;
            twist = Some(tq);
        }
    }
    (2.0 * t, swing, twist)
}

fn q_in_axis(q: Quaternion, axis: usize) -> f32 {
    q[axis + 1]
}

/// `angle_normalized_qt` — twice acos of w (unit quaternion).
pub fn angle_normalized(self_q: Quaternion) -> f32 {
    2.0 * self_q.w.clamp(-1.0, 1.0).acos()
}

/// `angle_qt`.
pub fn angle(q: Quaternion) -> f32 {
    angle_normalized(q.normalized())
}

/// `angle_normalized_qtqt` — angle between two unit quaternions.
pub fn angle_normalized_qtqt(q1: Quaternion, q2: Quaternion) -> f32 {
    angle_normalized(rotation_between_quats_to_quat(q1, q2))
}

/// `angle_qtqt`.
pub fn angle_qtqt(q1: Quaternion, q2: Quaternion) -> f32 {
    angle_normalized_qtqt(q1.normalized(), q2.normalized())
}

/// `angle_signed_normalized_qt`.
pub fn angle_signed_normalized(q: Quaternion) -> f32 {
    if q.w >= 0.0 {
        2.0 * q.w.clamp(-1.0, 1.0).acos()
    } else {
        -2.0 * (-q.w).clamp(-1.0, 1.0).acos()
    }
}

/// `angle_signed_normalized_qtqt`.
pub fn angle_signed_normalized_qtqt(q1: Quaternion, q2: Quaternion) -> f32 {
    if q1.dot(q2) >= 0.0 {
        angle_normalized_qtqt(q1, q2)
    } else {
        -angle_normalized_qtqt(q1, -q2)
    }
}

/// `angle_signed_qt`.
pub fn angle_signed_qt(q: Quaternion) -> f32 {
    angle_signed_normalized(q.normalized())
}

/// `angle_signed_qtqt`.
pub fn angle_signed_qtqt(q1: Quaternion, q2: Quaternion) -> f32 {
    if q1.dot(q2) >= 0.0 {
        angle_qtqt(q1, q2)
    } else {
        -angle_qtqt(q1, -q2)
    }
}

/// `vec_to_quat` — quaternion aligning `axis` of `vec` to point along +axis
/// with `upflag` orientation. `axis` 0-2 = +X/Y/Z, 3-5 = -X/Y/Z.
pub fn vec_to_quat(vec: Vec3, mut axis: u8, upflag: u8) -> Quaternion {
    const EPS: f32 = 1e-4;
    let len = vec.length();
    if len == 0.0 {
        return Quaternion::IDENTITY;
    }
    let tvec = if axis > 2 {
        axis -= 3;
        vec
    } else {
        -vec
    };
    let (mut nor, co_raw) = match axis {
        0 => {
            let n = Vec3::new(0.0, -tvec.z, tvec.y);
            (
                if tvec.y.abs() + tvec.z.abs() < EPS {
                    Vec3::new(n.x, 1.0, n.z)
                } else {
                    n
                },
                tvec.x,
            )
        }
        1 => {
            let n = Vec3::new(tvec.z, 0.0, -tvec.x);
            (
                if tvec.x.abs() + tvec.z.abs() < EPS {
                    Vec3::new(n.x, n.y, 1.0)
                } else {
                    n
                },
                tvec.y,
            )
        }
        _ => {
            let n = Vec3::new(-tvec.y, tvec.x, 0.0);
            (
                if tvec.x.abs() + tvec.y.abs() < EPS {
                    Vec3::new(1.0, n.y, n.z)
                } else {
                    n
                },
                tvec.z,
            )
        }
    };
    let co = co_raw / len;
    nor.normalize();
    let mut q = Quaternion::from_axis_angle(AxisAngle {
        axis: nor,
        angle: co.clamp(-1.0, 1.0).acos(),
    });
    if axis != upflag {
        let mat = q.to_mat3();
        let fp = mat[2];
        let angle = match (axis, upflag) {
            (0, 1) => 0.5 * fp.z.atan2(fp.y),
            (0, _) => -0.5 * fp.y.atan2(fp.z),
            (1, 0) => -0.5 * fp.z.atan2(fp.x),
            (1, _) => 0.5 * fp.x.atan2(fp.z),
            (2, 0) => 0.5 * (-fp.y).atan2(-fp.x),
            _ => -0.5 * (-fp.x).atan2(-fp.y),
        };
        let si = angle.sin() / len;
        let q2 = Quaternion {
            w: angle.cos(),
            x: tvec.x * si,
            y: tvec.y * si,
            z: tvec.z * si,
        };
        q = q2 * q;
    }
    q
}

/// `interp_qt_qtqt` — slerp with shortest-arc correction.
pub fn interp_qt(a: Quaternion, b: Quaternion, t: f32) -> Quaternion {
    let mut cosom = a.dot(b);
    let quat = if cosom < 0.0 {
        cosom = -cosom;
        -a
    } else {
        a
    };
    let (w0, w1) = crate::vector::interp_dot_slerp(t, cosom);
    quat * w0 + b * w1
}

/// `add_qt_qtqt` — `a + t*b` componentwise.
pub fn add_qt_scaled(a: Quaternion, b: Quaternion, t: f32) -> Quaternion {
    a + b * t
}

/// `tri_to_quat_ex` — quaternion from triangle + precomputed normal.
pub fn tri_to_quat_ex(v1: Vec3, v2: Vec3, no_orig: Vec3) -> Quaternion {
    let vec = no_orig;
    let mut n = Vec3::new(vec.y, -vec.x, 0.0);
    n.normalize();
    if n.x == 0.0 && n.y == 0.0 {
        n.x = 1.0;
    }
    let angle = -0.5 * vec.z.clamp(-1.0, 1.0).acos();
    let (si, co) = angle.sin_cos();
    let q1 = Quaternion {
        w: co,
        x: n.x * si,
        y: n.y * si,
        z: 0.0,
    };
    let mat = q1.to_mat3();
    let imat = mat.inverted().unwrap_or(mat);
    let mut vec2 = imat * (v2 - v1);
    vec2.z = 0.0;
    vec2.normalize();
    let angle2 = 0.5 * vec2.y.atan2(vec2.x);
    let (si2, co2) = angle2.sin_cos();
    let q2 = Quaternion {
        w: co2,
        x: 0.0,
        y: 0.0,
        z: si2,
    };
    q1 * q2
}

/// `tri_to_quat` — returns (quat, face_normal_length).
pub fn tri_to_quat(a: Vec3, b: Vec3, c: Vec3) -> (Quaternion, f32) {
    let (n, len) = crate::geom::normal_tri(a, b, c);
    (tri_to_quat_ex(a, b, n), len)
}

/// `sin_cos_from_fraction` — exact circle symmetry for integer fractions.
pub fn sin_cos_from_fraction(numerator: i32, denominator: i32) -> (f32, f32) {
    let mut numerator = numerator * 8;
    let denominator = denominator * 8;
    let octant = numerator / denominator;
    let mut cos_sign = 1.0f32;
    let mut swap = false;
    match octant {
        0 => {}
        1 | 2 => {
            numerator = denominator / 4 - numerator;
            swap = true;
        }
        3 | 4 => {
            numerator = denominator / 2 - numerator;
            cos_sign = -1.0;
        }
        5 | 6 => {
            numerator -= denominator * 3 / 4;
            swap = true;
            cos_sign = -1.0;
        }
        _ => {
            numerator -= denominator;
        }
    }
    let angle = 2.0 * std::f32::consts::PI * (numerator as f32 / denominator as f32);
    let (s, c) = (angle.sin(), angle.cos() * cos_sign);
    if swap {
        (c, s)
    } else {
        (s, c)
    }
}

/// `print_qt` — debug print.
pub fn print_qt(s: &str, q: Quaternion) {
    println!("{}: {:.3} {:.3} {:.3} {:.3}", s, q.w, q.x, q.y, q.z);
}

/// `angle_to_mat2` — 2D rotation matrix.
pub fn angle_to_mat2(angle: f32) -> crate::matrix::Mat2 {
    crate::matrix::Mat2::rotation(angle)
}

/// `rotation_between_vecs_to_mat3` — matrix rotating unit `v1` to unit `v2`.
pub fn rotation_between_vecs_to_mat3(v1: Vec3, v2: Vec3) -> Mat3 {
    let mut axis = v1.cross(v2);
    let angle_sin = axis.normalize_and_get_length();
    let angle_cos = v1.dot(v2);
    let (axis, s, c) = if angle_sin > FLT_EPSILON {
        (axis, angle_sin, angle_cos)
    } else if angle_cos > 0.0 {
        return Mat3::IDENTITY;
    } else {
        let mut ax = crate::vector::ortho_v3(v1);
        ax.normalize();
        (ax, 0.0, -1.0)
    };
    axis_angle_normalized_to_mat3_ex(axis, s, c)
}

/// `axis_angle_normalized_to_mat3_ex` — rotation matrix from unit axis with
/// precomputed sin/cos.
pub fn axis_angle_normalized_to_mat3_ex(axis: Vec3, angle_sin: f32, angle_cos: f32) -> Mat3 {
    let nsi = 1.0 - angle_cos;
    let (x, y, z) = (axis.x, axis.y, axis.z);
    Mat3::from_cols(
        Vec3::new(
            nsi * x * x + angle_cos,
            nsi * x * y + angle_sin * z,
            nsi * x * z - angle_sin * y,
        ),
        Vec3::new(
            nsi * x * y - angle_sin * z,
            nsi * y * y + angle_cos,
            nsi * y * z + angle_sin * x,
        ),
        Vec3::new(
            nsi * x * z + angle_sin * y,
            nsi * y * z - angle_sin * x,
            nsi * z * z + angle_cos,
        ),
    )
}

/// `quat_apply_track` — premultiply `quat` by the hard-coded axis track rotation.
pub fn quat_apply_track(quat: Quaternion, mut axis: u8, upflag: u8) -> Quaternion {
    const SQ: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let track = [
        Quaternion { w: SQ, x: 0.0, y: -SQ, z: 0.0 },
        Quaternion { w: 0.5, x: 0.5, y: 0.5, z: 0.5 },
        Quaternion { w: SQ, x: 0.0, y: 0.0, z: SQ },
        Quaternion { w: SQ, x: 0.0, y: SQ, z: 0.0 },
        Quaternion { w: 0.5, x: -0.5, y: -0.5, z: 0.5 },
        Quaternion { w: 0.0, x: SQ, y: SQ, z: 0.0 },
    ];
    let mut quat = quat * track[axis as usize];
    if axis > 2 {
        axis -= 3;
    }
    if upflag != ((2 - axis) >> 1) {
        let mut q = Quaternion { w: SQ, x: 0.0, y: 0.0, z: 0.0 };
        q[axis as usize + 1] = if axis == 1 { SQ } else { -SQ };
        quat = quat * q;
    }
    quat
}

/// `vec_apply_track` — permute `vec` into track-axis space.
pub fn vec_apply_track(vec: Vec3, axis: u8) -> Vec3 {
    match axis {
        0 | 3 => Vec3::new(vec.x, vec.z, -vec.y),
        1 => vec,
        2 => vec,
        4 => Vec3::new(-vec.z, vec.y, vec.x),
        _ => Vec3::new(-vec.x, -vec.y, vec.z),
    }
}

/// `focallength_to_fov`.
pub fn focallength_to_fov(focal_length: f32, sensor: f32) -> f32 {
    2.0 * ((sensor / 2.0) / focal_length).atan()
}

/// `fov_to_focallength`.
pub fn fov_to_focallength(hfov: f32, sensor: f32) -> f32 {
    (sensor / 2.0) / (hfov * 0.5).tan()
}

/// `mod_inline` — wraps `a` into [0, b).
fn mod_inline(a: f32, b: f32) -> f32 {
    a - (b * (a / b).floor())
}

/// `angle_wrap_rad` — wrap to [-pi, pi).
pub fn angle_wrap_rad(angle: f32) -> f32 {
    mod_inline(angle + std::f32::consts::PI, std::f32::consts::PI * 2.0) - std::f32::consts::PI
}

/// `angle_wrap_deg` — wrap to [-180, 180).
pub fn angle_wrap_deg(angle: f32) -> f32 {
    mod_inline(angle + 180.0, 360.0) - 180.0
}

/// `angle_compat_rad` — `angle` wrapped near `angle_compat`.
pub fn angle_compat_rad(angle: f32, angle_compat: f32) -> f32 {
    angle_compat + angle_wrap_rad(angle - angle_compat)
}

const AXIS_CONVERT_MATRIX: [[[f32; 3]; 3]; 23] = [
    [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]],
    [[-1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, -1.0, 0.0]],
    [[-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],
    [[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]],
    [[0.0, -1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, -1.0]],
    [[0.0, 0.0, 1.0], [-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]],
    [[0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
    [[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
    [[0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [-1.0, 0.0, 0.0]],
    [[0.0, 0.0, -1.0], [0.0, -1.0, 0.0], [-1.0, 0.0, 0.0]],
    [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]],
    [[0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [-1.0, 0.0, 0.0]],
    [[0.0, -1.0, 0.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0]],
    [[0.0, 0.0, 1.0], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0]],
    [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
    [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
    [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
    [[0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]],
    [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
    [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]],
    [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, -1.0]],
    [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, -1.0, 0.0]],
    [[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]],
];

const AXIS_CONVERT_LUT: [[u32; 24]; 23] = [
    [0x8C8, 0x4D0, 0x2E0, 0xAE8, 0x701, 0x511, 0x119, 0xB29, 0x682, 0x88A, 0x09A, 0x2A2,
     0x80B, 0x413, 0x223, 0xA2B, 0x644, 0x454, 0x05C, 0xA6C, 0x745, 0x94D, 0x15D, 0x365],
    [0xAC8, 0x8D0, 0x4E0, 0x2E8, 0x741, 0x951, 0x159, 0x369, 0x702, 0xB0A, 0x11A, 0x522,
     0xA0B, 0x813, 0x423, 0x22B, 0x684, 0x894, 0x09C, 0x2AC, 0x645, 0xA4D, 0x05D, 0x465],
    [0x4C8, 0x2D0, 0xAE0, 0x8E8, 0x681, 0x291, 0x099, 0x8A9, 0x642, 0x44A, 0x05A, 0xA62,
     0x40B, 0x213, 0xA23, 0x82B, 0x744, 0x354, 0x15C, 0x96C, 0x705, 0x50D, 0x11D, 0xB25],
    [0x2C8, 0xAD0, 0x8E0, 0x4E8, 0x641, 0xA51, 0x059, 0x469, 0x742, 0x34A, 0x15A, 0x962,
     0x20B, 0xA13, 0x823, 0x42B, 0x704, 0xB14, 0x11C, 0x52C, 0x685, 0x28D, 0x09D, 0x8A5],
    [0x708, 0xB10, 0x120, 0x528, 0x8C1, 0xAD1, 0x2D9, 0x4E9, 0x942, 0x74A, 0x35A, 0x162,
     0x64B, 0xA53, 0x063, 0x46B, 0x804, 0xA14, 0x21C, 0x42C, 0x885, 0x68D, 0x29D, 0x0A5],
    [0xB08, 0x110, 0x520, 0x728, 0x941, 0x151, 0x359, 0x769, 0x802, 0xA0A, 0x21A, 0x422,
     0xA4B, 0x053, 0x463, 0x66B, 0x884, 0x094, 0x29C, 0x6AC, 0x8C5, 0xACD, 0x2DD, 0x4E5],
    [0x508, 0x710, 0xB20, 0x128, 0x881, 0x691, 0x299, 0x0A9, 0x8C2, 0x4CA, 0x2DA, 0xAE2,
     0x44B, 0x653, 0xA63, 0x06B, 0x944, 0x754, 0x35C, 0x16C, 0x805, 0x40D, 0x21D, 0xA25],
    [0x108, 0x510, 0x720, 0xB28, 0x801, 0x411, 0x219, 0xA29, 0x882, 0x08A, 0x29A, 0x6A2,
     0x04B, 0x453, 0x663, 0xA6B, 0x8C4, 0x4D4, 0x2DC, 0xAEC, 0x945, 0x14D, 0x35D, 0x765],
    [0x748, 0x350, 0x160, 0x968, 0xAC1, 0x2D1, 0x4D9, 0x8E9, 0xA42, 0x64A, 0x45A, 0x062,
     0x68B, 0x293, 0x0A3, 0x8AB, 0xA04, 0x214, 0x41C, 0x82C, 0xB05, 0x70D, 0x51D, 0x125],
    [0x948, 0x750, 0x360, 0x168, 0xB01, 0x711, 0x519, 0x129, 0xAC2, 0x8CA, 0x4DA, 0x2E2,
     0x88B, 0x693, 0x2A3, 0x0AB, 0xA44, 0x654, 0x45C, 0x06C, 0xA05, 0x80D, 0x41D, 0x225],
    [0x348, 0x150, 0x960, 0x768, 0xA41, 0x051, 0x459, 0x669, 0xA02, 0x20A, 0x41A, 0x822,
     0x28B, 0x093, 0x8A3, 0x6AB, 0xB04, 0x114, 0x51C, 0x72C, 0xAC5, 0x2CD, 0x4DD, 0x8E5],
    [0x148, 0x950, 0x760, 0x368, 0xA01, 0x811, 0x419, 0x229, 0xB02, 0x10A, 0x51A, 0x722,
     0x08B, 0x893, 0x6A3, 0x2AB, 0xAC4, 0x8D4, 0x4DC, 0x2EC, 0xA45, 0x04D, 0x45D, 0x665],
    [0x688, 0x890, 0x0A0, 0x2A8, 0x4C1, 0x8D1, 0xAD9, 0x2E9, 0x502, 0x70A, 0xB1A, 0x122,
     0x74B, 0x953, 0x163, 0x36B, 0x404, 0x814, 0xA1C, 0x22C, 0x445, 0x64D, 0xA5D, 0x065],
    [0x888, 0x090, 0x2A0, 0x6A8, 0x501, 0x111, 0xB19, 0x729, 0x402, 0x80A, 0xA1A, 0x222,
     0x94B, 0x153, 0x363, 0x76B, 0x444, 0x054, 0xA5C, 0x66C, 0x4C5, 0x8CD, 0xADD, 0x2E5],
    [0x288, 0x690, 0x8A0, 0x0A8, 0x441, 0x651, 0xA59, 0x069, 0x4C2, 0x2CA, 0xADA, 0x8E2,
     0x34B, 0x753, 0x963, 0x16B, 0x504, 0x714, 0xB1C, 0x12C, 0x405, 0x20D, 0xA1D, 0x825],
    [0x088, 0x290, 0x6A0, 0x8A8, 0x401, 0x211, 0xA19, 0x829, 0x442, 0x04A, 0xA5A, 0x662,
     0x14B, 0x353, 0x763, 0x96B, 0x4C4, 0x2D4, 0xADC, 0x8EC, 0x505, 0x10D, 0xB1D, 0x725],
    [0x648, 0x450, 0x060, 0xA68, 0x2C1, 0x4D1, 0x8D9, 0xAE9, 0x282, 0x68A, 0x89A, 0x0A2,
     0x70B, 0x513, 0x123, 0xB2B, 0x204, 0x414, 0x81C, 0xA2C, 0x345, 0x74D, 0x95D, 0x165],
    [0xA48, 0x650, 0x460, 0x068, 0x341, 0x751, 0x959, 0x169, 0x2C2, 0xACA, 0x8DA, 0x4E2,
     0xB0B, 0x713, 0x523, 0x12B, 0x284, 0x694, 0x89C, 0x0AC, 0x205, 0xA0D, 0x81D, 0x425],
    [0x448, 0x050, 0xA60, 0x668, 0x281, 0x091, 0x899, 0x6A9, 0x202, 0x40A, 0x81A, 0xA22,
     0x50B, 0x113, 0xB23, 0x72B, 0x344, 0x154, 0x95C, 0x76C, 0x2C5, 0x4CD, 0x8DD, 0xAE5],
    [0x048, 0xA50, 0x660, 0x468, 0x201, 0xA11, 0x819, 0x429, 0x342, 0x14A, 0x95A, 0x762,
     0x10B, 0xB13, 0x723, 0x52B, 0x2C4, 0xAD4, 0x8DC, 0x4EC, 0x285, 0x08D, 0x89D, 0x6A5],
    [0x808, 0xA10, 0x220, 0x428, 0x101, 0xB11, 0x719, 0x529, 0x142, 0x94A, 0x75A, 0x362,
     0x8CB, 0xAD3, 0x2E3, 0x4EB, 0x044, 0xA54, 0x65C, 0x46C, 0x085, 0x88D, 0x69D, 0x2A5],
    [0xA08, 0x210, 0x420, 0x828, 0x141, 0x351, 0x759, 0x969, 0x042, 0xA4A, 0x65A, 0x462,
     0xACB, 0x2D3, 0x4E3, 0x8EB, 0x084, 0x294, 0x69C, 0x8AC, 0x105, 0xB0D, 0x71D, 0x525],
    [0x408, 0x810, 0xA20, 0x228, 0x081, 0x891, 0x699, 0x2A9, 0x102, 0x50A, 0x71A, 0xB22,
     0x4CB, 0x8D3, 0xAE3, 0x2EB, 0x144, 0x954, 0x75C, 0x36C, 0x045, 0x44D, 0x65D, 0xA65],
];

fn axis_signed(axis: i32) -> i32 {
    if axis < 3 {
        axis
    } else {
        axis - 3
    }
}

/// `mat3_from_axis_conversion` — conversion matrix between forward/up axis
/// conventions; `None` when degenerate or unknown.
pub fn mat3_from_axis_conversion(
    src_forward: i32,
    src_up: i32,
    dst_forward: i32,
    dst_up: i32,
) -> Option<Mat3> {
    if src_forward == dst_forward && src_up == dst_up {
        return Some(Mat3::IDENTITY);
    }
    if axis_signed(src_forward) == axis_signed(src_up)
        || axis_signed(dst_forward) == axis_signed(dst_up)
    {
        return Some(Mat3::IDENTITY);
    }
    let value = (src_forward as u32)
        | ((src_up as u32) << 3)
        | ((dst_forward as u32) << 6)
        | ((dst_up as u32) << 9);
    for (i, row) in AXIS_CONVERT_LUT.iter().enumerate() {
        for &v in row.iter() {
            if v == value {
                let m = AXIS_CONVERT_MATRIX[i];
                // C stores row-major [row][col]; our Mat3 is [col][row].
                return Some(Mat3::from_cols(
                    Vec3::new(m[0][0], m[1][0], m[2][0]),
                    Vec3::new(m[0][1], m[1][1], m[2][1]),
                    Vec3::new(m[0][2], m[1][2], m[2][2]),
                ));
            }
        }
    }
    None
}

/// `mat3_from_axis_conversion_single`.
pub fn mat3_from_axis_conversion_single(src_axis: i32, dst_axis: i32) -> Option<Mat3> {
    if src_axis == dst_axis {
        return Some(Mat3::IDENTITY);
    }
    let src_axis_next = (src_axis + 1) % 3;
    let mut dst_axis_next = (dst_axis + 1) % 3;
    if (src_axis < 3) != (dst_axis < 3) {
        dst_axis_next += 3;
    }
    mat3_from_axis_conversion(src_axis, src_axis_next, dst_axis, dst_axis_next)
}

/* ---- math_rotation_c.cc: DualQuat ---- */

/// `DualQuat` — dual quaternion with optional scale for skinning.
#[derive(Debug, Clone, Copy)]
pub struct DualQuat {
    pub quat: Quaternion,
    pub trans: Quaternion,
    pub scale: Mat4,
    pub scale_weight: f32,
}

fn mat4_to_size(m: Mat4) -> Vec3 {
    Vec3::new(
        m[0].xyz().length(),
        m[1].xyz().length(),
        m[2].xyz().length(),
    )
}

fn is_orthonormal_m3(m: Mat3) -> bool {
    const EPS: f32 = 0.01;
    (m[0].length() - 1.0).abs() < EPS
        && (m[1].length() - 1.0).abs() < EPS
        && (m[2].length() - 1.0).abs() < EPS
        && m[0].dot(m[1]).abs() < EPS
        && m[0].dot(m[2]).abs() < EPS
        && m[1].dot(m[2]).abs() < EPS
}

/// `mat4_to_dquat` — convert transform to dual quaternion given `basemat`.
pub fn mat4_to_dquat(basemat: Mat4, mat: Mat4) -> DualQuat {
    let base_rs = mat * basemat;
    let scale = mat4_to_size(base_rs);
    let dscale = scale - Vec3::splat(1.0);

    let mat3 = Mat3::from_cols(mat[0].xyz(), mat[1].xyz(), mat[2].xyz());
    let mut scale_mat = Mat4::IDENTITY;
    let mut scale_weight = 0.0;
    let r;

    if !is_orthonormal_m3(mat3)
        || mat.determinant() < 0.0
        || dscale.length_squared() > 1e-4f32 * 1e-4
    {
        // extra orthogonalize, avoid flipping with stretched bones
        let mut tmp3 = Mat3::from_cols(base_rs[0].xyz(), base_rs[1].xyz(), base_rs[2].xyz());
        tmp3 = tmp3.orthogonalized();
        let basequat = Quaternion::from_mat3(tmp3);
        let mut base_r = basequat.to_mat4();
        base_r[3] = base_rs[3];

        let baseinv = basemat.inverted().unwrap_or(basemat);
        let r_mat = base_r * baseinv;
        let base_rinv = base_r.inverted().unwrap_or(base_r);
        let s = base_rinv * base_rs;

        scale_mat = basemat * s * baseinv;
        scale_weight = 1.0;
        r = r_mat;
    } else {
        r = mat;
    }

    let quat = Quaternion::from_mat3(Mat3::from_cols(
        r[0].xyz(),
        r[1].xyz(),
        r[2].xyz(),
    ));
    let t = r[3];
    let q = quat;
    let trans = Quaternion {
        w: -0.5 * (t.x * q.x + t.y * q.y + t.z * q.z),
        x: 0.5 * (t.x * q.w + t.y * q.z - t.z * q.y),
        y: 0.5 * (-t.x * q.z + t.y * q.w + t.z * q.x),
        z: 0.5 * (t.x * q.y - t.y * q.x + t.z * q.w),
    };
    DualQuat {
        quat,
        trans,
        scale: scale_mat,
        scale_weight,
    }
}

/// `dquat_to_mat4`.
pub fn dquat_to_mat4(dq: &DualQuat) -> Mat4 {
    let mut q0 = dq.quat;
    let mut len = q0.dot(q0).sqrt();
    if len != 0.0 {
        len = 1.0 / len;
    }
    q0 = q0 * len;
    let mut r = q0.to_mat4();
    let t = dq.trans;
    r[3].x = 2.0 * (-t.w * q0.x + t.x * q0.w - t.y * q0.z + t.z * q0.y) * len;
    r[3].y = 2.0 * (-t.w * q0.y + t.x * q0.z + t.y * q0.w - t.z * q0.x) * len;
    r[3].z = 2.0 * (-t.w * q0.z - t.x * q0.y + t.y * q0.x + t.z * q0.w) * len;
    if dq.scale_weight != 0.0 {
        r = r * dq.scale;
    }
    r
}

/// `add_weighted_dq_dq` — accumulate `dq * weight` into `dq_sum`.
pub fn add_weighted_dq_dq(dq_sum: &mut DualQuat, dq: &DualQuat, weight: f32) {
    let mut weight = weight;
    let flipped = dq.quat.dot(dq_sum.quat) < 0.0;
    if flipped {
        weight = -weight;
    }
    dq_sum.quat = dq_sum.quat + dq.quat * weight;
    dq_sum.trans = dq_sum.trans + dq.trans * weight;
    if dq.scale_weight != 0.0 {
        if flipped {
            weight = -weight;
        }
        dq_sum.scale = dq_sum.scale + dq.scale * weight;
        dq_sum.scale_weight += weight;
    }
}

/// `add_weighted_dq_dq_pivot` — accumulate with scale correction at `pivot`.
pub fn add_weighted_dq_dq_pivot(
    dq_sum: &mut DualQuat,
    dq: &DualQuat,
    pivot: Vec3,
    weight: f32,
    compute_scale_matrix: bool,
) {
    if dq.scale_weight != 0.0 {
        let mut mdq = *dq;
        // translation induced by scale at the pivot point
        let dst = dq.scale * pivot - pivot;
        let q = mdq.quat;
        mdq.trans.w -= 0.5 * (q.x * dst.x + q.y * dst.y + q.z * dst.z);
        mdq.trans.x += 0.5 * (q.w * dst.x + q.y * dst.z - q.z * dst.y);
        mdq.trans.y += 0.5 * (q.w * dst.y + q.z * dst.x - q.x * dst.z);
        mdq.trans.z += 0.5 * (q.w * dst.z + q.x * dst.y - q.y * dst.x);
        if compute_scale_matrix {
            mdq.scale[3].x -= dst.x;
            mdq.scale[3].y -= dst.y;
            mdq.scale[3].z -= dst.z;
        } else {
            mdq.scale_weight = 0.0;
        }
        add_weighted_dq_dq(dq_sum, &mdq, weight);
    } else {
        add_weighted_dq_dq(dq_sum, dq, weight);
    }
}

/// `normalize_dq` — normalize by total weight.
pub fn normalize_dq(dq: &mut DualQuat, totweight: f32) {
    let scale = 1.0 / totweight;
    dq.quat = dq.quat * scale;
    dq.trans = dq.trans * scale;
    if dq.scale_weight != 0.0 {
        let addweight = totweight - dq.scale_weight;
        if addweight != 0.0 {
            dq.scale[0][0] += addweight;
            dq.scale[1][1] += addweight;
            dq.scale[2][2] += addweight;
            dq.scale[3][3] += addweight;
        }
        dq.scale = dq.scale * scale;
        dq.scale_weight = 1.0;
    }
}

/// `mul_v3m3_dq` — transform `r` by the dual quaternion; also returns the
/// crazy-space correction Mat3.
pub fn mul_v3m3_dq(r_in: Vec3, dq: &DualQuat) -> (Vec3, Mat3) {
    let (w, x, y, z) = (dq.quat.w, dq.quat.x, dq.quat.y, dq.quat.z);
    let (t0, t1, t2, t3) = (dq.trans.w, dq.trans.x, dq.trans.y, dq.trans.z);
    let m = Mat3::from_cols(
        Vec3::new(
            w * w + x * x - y * y - z * z,
            2.0 * (x * y + w * z),
            2.0 * (x * z - w * y),
        ),
        Vec3::new(
            2.0 * (x * y - w * z),
            w * w + y * y - x * x - z * z,
            2.0 * (y * z + w * x),
        ),
        Vec3::new(
            2.0 * (x * z + w * y),
            2.0 * (y * z - w * x),
            w * w + z * z - x * x - y * y,
        ),
    );
    let mut len2 = dq.quat.dot(dq.quat);
    if len2 > 0.0 {
        len2 = 1.0 / len2;
    }
    let t = Vec3::new(
        2.0 * (-t0 * x + w * t1 - t2 * z + y * t3),
        2.0 * (-t0 * y + t1 * z - x * t3 + w * t2),
        2.0 * (-t0 * z + x * t2 + w * t3 - t1 * y),
    );
    let mut r = r_in;
    if dq.scale_weight != 0.0 {
        r = dq.scale * r;
    }
    r = m * r;
    r = (r + t) * len2;

    let r_mat = if dq.scale_weight != 0.0 {
        let scalemat = Mat3::from_cols(
            dq.scale[0].xyz(),
            dq.scale[1].xyz(),
            dq.scale[2].xyz(),
        );
        (m * scalemat) * len2
    } else {
        m * len2
    };
    (r, r_mat)
}

/// `axis_angle_to_quat` — axis may be unnormalized.
pub fn axis_angle_to_quat(axis: Vec3, angle: f32) -> Quaternion {
    let mut nor = axis;
    if nor.normalize_and_get_length() != 0.0 {
        Quaternion::from_axis_angle(AxisAngle { axis: nor, angle })
    } else {
        Quaternion::IDENTITY
    }
}

/// `quat_to_axis_angle` — returns (axis, angle).
pub fn quat_to_axis_angle(q: Quaternion) -> (Vec3, f32) {
    let ha = q.w.clamp(-1.0, 1.0).acos();
    let mut si = ha.sin();
    let angle = ha * 2.0;
    if si.abs() < 0.0005 {
        si = 1.0;
    }
    let mut axis = Vec3::new(q.x / si, q.y / si, q.z / si);
    if axis.is_zero() {
        axis.y = 1.0;
    }
    (axis, angle)
}

/// `axis_angle_to_eulO`.
pub fn axis_angle_to_eul(axis: Vec3, angle: f32, order: crate::euler::EulerOrder) -> crate::euler::Euler3 {
    let q = axis_angle_to_quat(axis, angle);
    crate::euler::quat_to_euler(q, order)
}

/// `eulO_to_axis_angle`.
pub fn eul_to_axis_angle(eul: crate::euler::Euler3) -> (Vec3, f32) {
    quat_to_axis_angle(eul.to_quaternion())
}

/// `axis_angle_normalized_to_mat3`.
pub fn axis_angle_normalized_to_mat3(axis: Vec3, angle: f32) -> Mat3 {
    let (s, c) = angle.sin_cos();
    axis_angle_normalized_to_mat3_ex(axis, s, c)
}

/// `axis_angle_to_mat3` — axis may be unnormalized.
pub fn axis_angle_to_mat3(axis: Vec3, angle: f32) -> Mat3 {
    let mut nor = axis;
    if nor.normalize_and_get_length() == 0.0 {
        return Mat3::IDENTITY;
    }
    axis_angle_normalized_to_mat3(nor, angle)
}

/// `axis_angle_to_mat4`.
pub fn axis_angle_to_mat4(axis: Vec3, angle: f32) -> Mat4 {
    let m = axis_angle_to_mat3(axis, angle);
    let mut r = Mat4::IDENTITY;
    r[0].x = m[0].x;
    r[0].y = m[0].y;
    r[0].z = m[0].z;
    r[1].x = m[1].x;
    r[1].y = m[1].y;
    r[1].z = m[1].z;
    r[2].x = m[2].x;
    r[2].y = m[2].y;
    r[2].z = m[2].z;
    r
}

/// `mat3_normalized_to_axis_angle`.
pub fn mat3_normalized_to_axis_angle(mat: Mat3) -> (Vec3, f32) {
    quat_to_axis_angle(Quaternion::from_mat3(mat))
}

/// `mat3_to_axis_angle`.
pub fn mat3_to_axis_angle(mat: Mat3) -> (Vec3, f32) {
    mat3_normalized_to_axis_angle(mat.normalized())
}

/// `mat4_to_axis_angle` (and normalized variant — same path via Mat3).
pub fn mat4_to_axis_angle(mat: Mat4) -> (Vec3, f32) {
    mat3_to_axis_angle(Mat3::from_cols(mat[0].xyz(), mat[1].xyz(), mat[2].xyz()))
}

/// `axis_angle_to_mat3_single` — single-axis rotation ('X', 'Y', 'Z' as 0,1,2).
pub fn axis_angle_to_mat3_single(axis: usize, angle: f32) -> Mat3 {
    let (angle_sin, angle_cos) = angle.sin_cos();
    match axis {
        0 => Mat3::from_cols(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, angle_cos, -angle_sin),
            Vec3::new(0.0, angle_sin, angle_cos),
        ),
        1 => Mat3::from_cols(
            Vec3::new(angle_cos, 0.0, angle_sin),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(-angle_sin, 0.0, angle_cos),
        ),
        _ => Mat3::from_cols(
            Vec3::new(angle_cos, -angle_sin, 0.0),
            Vec3::new(angle_sin, angle_cos, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ),
    }
}

/// `axis_angle_to_mat4_single`.
pub fn axis_angle_to_mat4_single(axis: usize, angle: f32) -> Mat4 {
    let m = axis_angle_to_mat3_single(axis, angle);
    let mut r = Mat4::IDENTITY;
    for c in 0..3 {
        for row in 0..3 {
            r[c][row] = m[c][row];
        }
    }
    r
}

/// `axis_angle_to_quat_single` — quaternion rotating `angle` around `axis` (0,1,2).
pub fn axis_angle_to_quat_single(axis: usize, angle: f32) -> Quaternion {
    let half = angle * 0.5;
    let mut q = Quaternion {
        w: half.cos(),
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    q[axis + 1] = half.sin();
    q
}

/// `quat_normalized_to_expmap` — axis * angle.
pub fn quat_normalized_to_expmap(q: Quaternion) -> Vec3 {
    let (axis, angle) = quat_to_axis_angle(q);
    axis * angle
}

/// `quat_to_expmap`.
pub fn quat_to_expmap(q: Quaternion) -> Vec3 {
    quat_normalized_to_expmap(q.normalized())
}

/// `expmap_to_quat`.
pub fn expmap_to_quat(expmap: Vec3) -> Quaternion {
    let mut axis = expmap;
    let angle = axis.normalize_and_get_length();
    if angle != 0.0 {
        Quaternion::from_axis_angle(AxisAngle {
            axis,
            angle: angle_wrap_rad(angle),
        })
    } else {
        Quaternion::IDENTITY
    }
}

/// `rotate_eul` — rotate XYZ euler by `angle` around `axis` (0,1,2).
pub fn rotate_eul(beul: crate::euler::EulerXYZ, axis: usize, angle: f32) -> crate::euler::EulerXYZ {
    let mut eul = crate::euler::EulerXYZ::ZERO;
    match axis {
        0 => eul.x = angle,
        1 => eul.y = angle,
        _ => eul.z = angle,
    }
    let mat1 = eul.to_mat3();
    let mat2 = beul.to_mat3();
    let totmat = mat2 * mat1;
    crate::euler::mat3_to_euler_xyz(&totmat)
}

/// `rotate_eulO`.
pub fn rotate_eul_o(beul: crate::euler::Euler3, axis: usize, angle: f32) -> crate::euler::Euler3 {
    let mut eul = Vec3::ZERO;
    eul[axis] = angle;
    let eul = crate::euler::Euler3::new(eul, beul.order);
    let totmat = beul.to_mat3() * eul.to_mat3();
    crate::euler::mat3_to_euler(&totmat, beul.order)
}

/// `eulO_to_gimbal_axis` — gimbal frame columns.
pub fn eul_to_gimbal_axis(eul: crate::euler::Euler3) -> Mat3 {
    let axes = [
        eul.order.i_index(),
        eul.order.j_index(),
        eul.order.k_index(),
    ];
    // first axis is local
    let mat = eul.to_mat3();
    let col0 = mat[axes[0]];
    // second axis is local minus first rotation
    let mut teul = eul.xyz;
    teul[axes[0]] = 0.0;
    let mat2 = crate::euler::Euler3::new(teul, eul.order).to_mat3();
    let col1 = mat2[axes[1]];
    // last axis is global
    let mut col2 = Vec3::ZERO;
    col2[axes[2]] = 1.0;
    let mut gmat = Mat3::IDENTITY;
    gmat[axes[0]] = col0;
    gmat[axes[1]] = col1;
    gmat[axes[2]] = col2;
    gmat
}

/// `add_eul_euleul` — `a` rotated by `b` (quat multiply order `b*a`).
pub fn add_eul_euleul(a: crate::euler::Euler3, b: crate::euler::Euler3) -> crate::euler::Euler3 {
    let quat = b.to_quaternion() * a.to_quaternion();
    crate::euler::quat_to_euler(quat, a.order)
}

/// `sub_eul_euleul` — `a` minus `b` (`b^-1 * a`).
pub fn sub_eul_euleul(a: crate::euler::Euler3, b: crate::euler::Euler3) -> crate::euler::Euler3 {
    let quat = b.to_quaternion().inverted() * a.to_quaternion();
    crate::euler::quat_to_euler(quat, a.order)
}
