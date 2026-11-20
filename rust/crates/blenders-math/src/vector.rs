//! Port of `BLI_math_vector.hh` / `math_vector.cc` (Vec2/Vec3/Vec4).

use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::base;
use crate::base::FLT_EPSILON;

macro_rules! vec_ops {
    ($VecN:ident, $n:expr, $($field:ident),+) => {
        impl Add for $VecN {
            type Output = Self;
            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self { $($field: self.$field + rhs.$field),+ }
            }
        }
        impl AddAssign for $VecN {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                $(self.$field += rhs.$field;)+
            }
        }
        impl Sub for $VecN {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: Self) -> Self {
                Self { $($field: self.$field - rhs.$field),+ }
            }
        }
        impl SubAssign for $VecN {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$field -= rhs.$field;)+
            }
        }
        impl Neg for $VecN {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                Self { $($field: -self.$field),+ }
            }
        }
        /// Scalar multiply (`mul_vn_fl`).
        impl Mul<f32> for $VecN {
            type Output = Self;
            #[inline]
            fn mul(self, rhs: f32) -> Self {
                Self { $($field: self.$field * rhs),+ }
            }
        }
        impl Mul<$VecN> for f32 {
            type Output = $VecN;
            #[inline]
            fn mul(self, rhs: $VecN) -> $VecN {
                rhs * self
            }
        }
        impl MulAssign<f32> for $VecN {
            #[inline]
            fn mul_assign(&mut self, rhs: f32) {
                $(self.$field *= rhs;)+
            }
        }
        impl DivAssign<f32> for $VecN {
            #[inline]
            fn div_assign(&mut self, rhs: f32) {
                $(self.$field /= rhs;)+
            }
        }
        /// Component-wise multiply (`mul_vn_vn`).
        impl Mul<$VecN> for $VecN {
            type Output = Self;
            #[inline]
            fn mul(self, rhs: Self) -> Self {
                Self { $($field: self.$field * rhs.$field),+ }
            }
        }
        /// Scalar divide.
        impl Div<f32> for $VecN {
            type Output = Self;
            #[inline]
            fn div(self, rhs: f32) -> Self {
                self * (1.0 / rhs)
            }
        }
        /// Component-wise divide.
        impl Div<$VecN> for $VecN {
            type Output = Self;
            #[inline]
            fn div(self, rhs: Self) -> Self {
                Self { $($field: self.$field / rhs.$field),+ }
            }
        }
        impl Index<usize> for $VecN {
            type Output = f32;
            #[inline]
            fn index(&self, i: usize) -> &f32 {
                // `#[repr(C)]` struct of exactly `$n` f32 fields.
                let arr: &[f32; $n] = unsafe { &*(self as *const Self as *const [f32; $n]) };
                &arr[i]
            }
        }
        impl IndexMut<usize> for $VecN {
            #[inline]
            fn index_mut(&mut self, i: usize) -> &mut f32 {
                let arr: &mut [f32; $n] = unsafe { &mut *(self as *mut Self as *mut [f32; $n]) };
                &mut arr[i]
            }
        }
        impl $VecN {
            /// `len_squared_vn`.
            #[inline]
            pub fn length_squared(self) -> f32 {
                self.dot(self)
            }
            /// `len_vn`.
            #[inline]
            pub fn length(self) -> f32 {
                self.length_squared().sqrt()
            }
            /// `len_manhattan_vn`.
            #[inline]
            pub fn length_manhattan(self) -> f32 {
                0.0 $(+ self.$field.abs())+
            }
            /// `dot_vnvn`.
            #[inline]
            pub fn dot(self, rhs: Self) -> f32 {
                0.0 $(+ self.$field * rhs.$field)+
            }
            /// `normalize_vn` — unit vector or zero.
            #[inline]
            pub fn normalized(self) -> Self {
                let len = self.length();
                if len != 0.0 { self / len } else { self }
            }
            /// `normalize_vn` in place.
            #[inline]
            pub fn normalize(&mut self) {
                *self = self.normalized();
            }
            /// `normalize_and_get_length`.
            #[inline]
            pub fn normalize_and_get_length(&mut self) -> f32 {
                let len = self.length();
                if len != 0.0 {
                    *self = *self / len;
                }
                len
            }
            /// `dist_vnvn`.
            #[inline]
            pub fn distance(self, rhs: Self) -> f32 {
                (self - rhs).length()
            }
            /// `dist_squared_vnvn`.
            #[inline]
            pub fn distance_squared(self, rhs: Self) -> f32 {
                (self - rhs).length_squared()
            }
            /// `mid_vn_vnvn`.
            #[inline]
            pub fn mid(self, rhs: Self) -> Self {
                (self + rhs) * 0.5
            }
            /// `interp_vn_vnvn`.
            #[inline]
            pub fn lerp(self, rhs: Self, t: f32) -> Self {
                self + (rhs - self) * t
            }
            /// `min_vn_vnvn`.
            #[inline]
            pub fn min(self, rhs: Self) -> Self {
                Self { $($field: self.$field.min(rhs.$field)),+ }
            }
            /// `max_vn_vnvn`.
            #[inline]
            pub fn max(self, rhs: Self) -> Self {
                Self { $($field: self.$field.max(rhs.$field)),+ }
            }
            /// `abs_vn`.
            #[inline]
            pub fn abs(self) -> Self {
                Self { $($field: self.$field.abs()),+ }
            }
            /// `is_zero_vn`.
            #[inline]
            pub fn is_zero(self) -> bool {
                self.length_squared() == 0.0
            }
            /// `is_finite_vn`.
            #[inline]
            pub fn is_finite(self) -> bool {
                true $(&& self.$field.is_finite())+
            }
            /// Component-wise floor.
            #[inline]
            pub fn floor(self) -> Self {
                Self { $($field: self.$field.floor()),+ }
            }
            /// `wrap_vn` in `[min, max)` per component.
            #[inline]
            pub fn wrap(self, min: f32, max: f32) -> Self {
                Self { $($field: base::wrap(self.$field, min, max)),+ }
            }
        }
    };
}

/// `float2` — 2-component vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

/// `float3` — 3-component vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// `float4` — 4-component vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

vec_ops!(Vec2, 2, x, y);
vec_ops!(Vec3, 3, x, y, z);
vec_ops!(Vec4, 4, x, y, z, w);

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }
    /// Perpendicular vector `(-y, x)`.
    #[inline]
    pub fn orthogonal(self) -> Self {
        Self::new(-self.y, self.x)
    }
    /// `cross_v2v2` — 2D perp-dot `ax*by - ay*bx`.
    #[inline]
    pub fn cross(self, rhs: Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }
}

impl Vec3 {
    /// `axis_dominant_v3` — indices of the two non-dominant axes `(a, b)`.
    #[inline]
    pub fn axis_dominant(self) -> (usize, usize) {
        let (xn, yn, zn) = (self.x.abs(), self.y.abs(), self.z.abs());
        if zn >= xn && zn >= yn {
            (0, 1)
        } else if yn >= xn && yn >= zn {
            (0, 2)
        } else {
            (1, 2)
        }
    }
    /// `axis_dominant_v3_max` — same axes plus the dominant magnitude.
    #[inline]
    pub fn axis_dominant_max(self) -> (usize, usize, f32) {
        let (xn, yn, zn) = (self.x.abs(), self.y.abs(), self.z.abs());
        if zn >= xn && zn >= yn {
            (0, 1, zn)
        } else if yn >= xn && yn >= zn {
            (0, 2, yn)
        } else {
            (1, 2, xn)
        }
    }
    /// `axis_dominant_v3_single` — index of the largest |component|.
    #[inline]
    pub fn abs_max_axis(self) -> usize {
        let (x, y, z) = (self.x.abs(), self.y.abs(), self.z.abs());
        if x > y {
            if x > z {
                0
            } else {
                2
            }
        } else if y > z {
            1
        } else {
            2
        }
    }
    /// `axis_dominant_v3_ortho_single` — index of the smallest |component|.
    #[inline]
    pub fn abs_min_axis(self) -> usize {
        let (x, y, z) = (self.x.abs(), self.y.abs(), self.z.abs());
        if x < y {
            if x < z {
                0
            } else {
                2
            }
        } else if y < z {
            1
        } else {
            2
        }
    }
    /// `max_axis_v3`.
    #[inline]
    pub fn max_axis(self) -> usize {
        if self.x > self.y {
            if self.x > self.z {
                0
            } else {
                2
            }
        } else if self.y > self.z {
            1
        } else {
            2
        }
    }
    /// `min_axis_v3`.
    #[inline]
    pub fn min_axis(self) -> usize {
        if self.x < self.y {
            if self.x < self.z {
                0
            } else {
                2
            }
        } else if self.y < self.z {
            1
        } else {
            2
        }
    }


    /// First two components (`float2(v3)`).
    #[inline]
    pub fn xy(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z: Self = Self { x: 0.0, y: 0.0, z: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }
    /// `cross_v3_v3v3`.
    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
    /// `dot_v3v3` is `self.dot(rhs)`; triple product `cross_v3_v3v3(...)·v` → `triple_v3`.
    #[inline]
    pub fn triple(self, b: Self, c: Self) -> f32 {
        self.cross(b).dot(c)
    }
    /// `project_v3_v3v3` — projection of `self` onto `target` (assumed non-zero).
    #[inline]
    pub fn project_onto(self, target: Self) -> Self {
        target * (self.dot(target) / target.dot(target))
    }
    /// `project_v3_v3v3_normalized` — target must already be unit length.
    #[inline]
    pub fn project_onto_normalized(self, target: Self) -> Self {
        target * self.dot(target)
    }
    /// `reflect_v3_v3v3` — reflect `self` around `normal` (unit length).
    #[inline]
    pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }
    /// `madd_v3_v3v3fl` — `a + b * t`.
    #[inline]
    pub fn madd(self, b: Self, t: f32) -> Self {
        self + b * t
    }
    /// `orthogonal_v3` — any vector perpendicular to `self`.
    pub fn orthogonal(self) -> Self {
        let n = self.normalized();
        // Pick the axis least aligned to avoid degenerate cross.
        let other = if n.x.abs() < n.y.abs() {
            Vec3::X
        } else if n.y.abs() < n.z.abs() {
            Vec3::Y
        } else {
            Vec3::Z
        };
        n.cross(other).normalized()
    }
    /// `angle_normalized_v3v3` — angle between two unit vectors.
    #[inline]
    pub fn angle_normalized(self, other: Self) -> f32 {
        base::saacos(self.dot(other))
    }
    /// `angle_v3v3`.
    #[inline]
    pub fn angle(self, other: Self) -> f32 {
        self.normalized().angle_normalized(other.normalized())
    }
}

impl Vec4 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v, w: v }
    }
    /// First two components.
    #[inline]
    pub fn xy(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// `xyz()` — drop the w component.
    #[inline]
    pub fn xyz(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl From<Vec3> for Vec4 {
    #[inline]
    fn from(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z, 0.0)
    }
}

impl From<Vec2> for Vec3 {
    #[inline]
    fn from(v: Vec2) -> Self {
        Self::new(v.x, v.y, 0.0)
    }
}

impl From<Vec3> for Vec2 {
    #[inline]
    fn from(v: Vec3) -> Self {
        Self::new(v.x, v.y)
    }
}

impl From<[f32; 3]> for Vec3 {
    #[inline]
    fn from(a: [f32; 3]) -> Self {
        Self::new(a[0], a[1], a[2])
    }
}

impl From<Vec3> for [f32; 3] {
    #[inline]
    fn from(v: Vec3) -> Self {
        [v.x, v.y, v.z]
    }
}

/// Compare with absolute epsilon (`compare_v3v3`).
#[inline]
pub fn compare_v3(a: Vec3, b: Vec3, eps: f32) -> bool {
    a.distance_squared(b) < eps * eps
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn cross_and_dot() {
        let x = Vec3::X;
        let y = Vec3::Y;
        assert_eq!(x.cross(y), Vec3::Z);
        assert_eq!(x.dot(y), 0.0);
        assert_eq!(Vec3::Z.triple(Vec3::X, Vec3::Y), 1.0);
    }

    #[test]
    fn normalize_and_length() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.length(), 5.0);
        assert!((v.normalized().length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn project_reflect() {
        let v = Vec3::new(1.0, 1.0, 0.0);
        assert_eq!(v.project_onto(Vec3::X), Vec3::X);
        let r = Vec3::new(1.0, 1.0, 0.0).reflect(Vec3::Y);
        assert!((r - Vec3::new(1.0, -1.0, 0.0)).is_zero());
    }

    #[test]
    fn angle() {
        assert!((Vec3::X.angle(Vec3::Y) - FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn index() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v[0], 1.0);
        assert_eq!(v[2], 3.0);
    }
}

/* ---- math_vector.cc: interpolation ---- */

/// `interp_dot_slerp` — slerp weights for dot `cosom` at `t`.
pub fn interp_dot_slerp(t: f32, cosom: f32) -> (f32, f32) {
    const EPS: f32 = 1e-4;
    if cosom.abs() < 1.0 - EPS {
        let omega = cosom.acos();
        let sinom = omega.sin();
        (((1.0 - t) * omega).sin() / sinom, (t * omega).sin() / sinom)
    } else {
        (1.0 - t, t)
    }
}

/// `interp_v3_v3v3_slerp` — spherical interp of unit vecs; `None` on opposites.
pub fn slerp_v3(a: Vec3, b: Vec3, t: f32) -> Option<Vec3> {
    let cosom = a.dot(b);
    if cosom < -1.0 + FLT_EPSILON {
        return None;
    }
    let (w0, w1) = interp_dot_slerp(t, cosom);
    Some(a * w0 + b * w1)
}

/// `interp_v3_v3v3_slerp_safe` — falls back through an orthogonal mid-vector.
pub fn slerp_v3_safe(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    if let Some(r) = slerp_v3(a, b, t) {
        return r;
    }
    let mut ab_ortho = ortho_v3(a);
    ab_ortho.normalize();
    if t < 0.5 {
        slerp_v3(a, ab_ortho, t * 2.0).unwrap_or(a)
    } else {
        slerp_v3(ab_ortho, b, (t - 0.5) * 2.0).unwrap_or(b)
    }
}

/// `interp_v2_v2v2v2v2_cubic` — cubic Bezier through 4 control points.
pub fn interp_v2_cubic(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2, u: f32) -> Vec2 {
    let q0 = v1.lerp(v2, u);
    let q1 = v2.lerp(v3, u);
    let q2 = v3.lerp(v4, u);
    let r0 = q0.lerp(q1, u);
    let r1 = q1.lerp(q2, u);
    r0.lerp(r1, u)
}

/// `interp_v3_v3v3v3` — weighted sum of 3 vectors.
pub fn interp_v3_weighted3(v1: Vec3, v2: Vec3, v3: Vec3, w: Vec3) -> Vec3 {
    v1 * w.x + v2 * w.y + v3 * w.z
}

/// `interp_v3_v3v3v3v3`.
pub fn interp_v3_weighted4(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3, w: Vec4) -> Vec3 {
    v1 * w.x + v2 * w.y + v3 * w.z + v4 * w.w
}

/// `interp_v4_v4v4v4`.
pub fn interp_v4_weighted3(v1: Vec4, v2: Vec4, v3: Vec4, w: Vec3) -> Vec4 {
    v1 * w.x + v2 * w.y + v3 * w.z
}

/// `interp_v4_v4v4v4v4`.
pub fn interp_v4_weighted4(v1: Vec4, v2: Vec4, v3: Vec4, v4: Vec4, w: Vec4) -> Vec4 {
    v1 * w.x + v2 * w.y + v3 * w.z + v4 * w.w
}

/// `interp_v3_v3v3v3_uv` — barycentric point on a triangle from uv.
pub fn interp_v3_uv(v1: Vec3, v2: Vec3, v3: Vec3, uv: Vec2) -> Vec3 {
    v1 + (v2 - v1) * uv.x + (v3 - v1) * uv.y
}

/// `interp_v3_v3v3_uchar` — lerp 8-bit channels (floor).
pub fn interp_u8(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let s = 1.0 - t;
    [
        (s * a[0] as f32 + t * b[0] as f32).floor() as u8,
        (s * a[1] as f32 + t * b[1] as f32).floor() as u8,
        (s * a[2] as f32 + t * b[2] as f32).floor() as u8,
    ]
}

/// `mid_v3_v3v3v3` — centroid of 3 points.
pub fn mid3(v1: Vec3, v2: Vec3, v3: Vec3) -> Vec3 {
    (v1 + v2 + v3) / 3.0
}

/// `mid_v3_v3v3v3v3` — centroid of 4 points.
pub fn mid4(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> Vec3 {
    (v1 + v2 + v3 + v4) / 4.0
}

/// `mid_v3_v3_array` — mean of a slice.
pub fn mid_slice(vec_arr: &[Vec3]) -> Vec3 {
    let factor = 1.0 / vec_arr.len() as f32;
    vec_arr.iter().fold(Vec3::ZERO, |r, v| r + *v * factor)
}

/// `mid_v3_v3v3_angle_weighted` — mid of 2 unit normals scaled by 0..1 angle.
pub fn mid_angle_weighted(a: Vec3, b: Vec3) -> Vec3 {
    let mut r = a + b;
    let angle = (std::f32::consts::FRAC_2_PI * 2.0)
        * (r.normalize_and_get_length() / 2.0).acos();
    r * angle
}

/* ---- math_vector.cc: angles ---- */

/// `angle_normalized_v2v2` — angle between two unit Vec2 (accurate form).
pub fn angle_normalized_v2v2(a: Vec2, b: Vec2) -> f32 {
    let safe_asin = |x: f32| x.clamp(-1.0, 1.0).asin();
    if a.dot(b) >= 0.0 {
        2.0 * safe_asin((a - b).length() / 2.0)
    } else {
        std::f32::consts::PI - 2.0 * safe_asin((a + b).length() / 2.0)
    }
}

/// `angle_v3v3v3` — interior angle at `b`.
pub fn angle_v3v3v3(a: Vec3, b: Vec3, c: Vec3) -> f32 {
    let mut vec1 = b - a;
    let mut vec2 = b - c;
    vec1.normalize();
    vec2.normalize();
    vec1.angle_normalized(vec2)
}

/// `cos_v3v3v3`.
pub fn cos_v3v3v3(p1: Vec3, p2: Vec3, p3: Vec3) -> f32 {
    let mut vec1 = p2 - p1;
    let mut vec2 = p2 - p3;
    vec1.normalize();
    vec2.normalize();
    vec1.dot(vec2)
}

/// `angle_v2v2v2` — interior angle at `b` in 2D.
pub fn angle_v2v2v2(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    let mut vec1 = b - a;
    let mut vec2 = b - c;
    vec1.normalize();
    vec2.normalize();
    angle_normalized_v2v2(vec1, vec2)
}

/// `cos_v2v2v2`.
pub fn cos_v2v2v2(p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    let mut vec1 = p2 - p1;
    let mut vec2 = p2 - p3;
    vec1.normalize();
    vec2.normalize();
    vec1.dot(vec2)
}

/// `angle_v2v2` — angle between 2D vectors (normalizes copies).
pub fn angle_v2v2(a: Vec2, b: Vec2) -> f32 {
    angle_normalized_v2v2(a.normalized(), b.normalized())
}

/// `angle_signed_v2v2`.
pub fn angle_signed_v2v2(v1: Vec2, v2: Vec2) -> f32 {
    let perp_dot = v1.y * v2.x - v1.x * v2.y;
    perp_dot.atan2(v1.dot(v2))
}

/// `angle_on_axis_v3v3_v3` — angle between v1,v2 projected onto plane of `axis`.
pub fn angle_on_axis(v1: Vec3, v2: Vec3, axis: Vec3) -> f32 {
    let v1p = project_plane_normalized(v1, axis);
    let v2p = project_plane_normalized(v2, axis);
    v1p.normalized().angle_normalized(v2p.normalized())
}

/// `angle_signed_on_axis_v3v3_v3`.
pub fn angle_signed_on_axis(v1: Vec3, v2: Vec3, axis: Vec3) -> f32 {
    let v1p = project_plane_normalized(v1, axis);
    let v2p = project_plane_normalized(v2, axis);
    let mut angle = v1p.normalized().angle_normalized(v2p.normalized());
    let tproj = v2p.cross(v1p);
    if tproj.dot(axis) < 0.0 {
        angle = std::f32::consts::PI * 2.0 - angle;
    }
    angle
}

/// `angle_on_axis_v3v3v3_v3`.
pub fn angle_on_axis_v3v3v3(v1: Vec3, v2: Vec3, v3: Vec3, axis: Vec3) -> f32 {
    angle_on_axis(v1 - v2, v3 - v2, axis)
}

/// `angle_signed_on_axis_v3v3v3_v3`.
pub fn angle_signed_on_axis_v3v3v3(v1: Vec3, v2: Vec3, v3: Vec3, axis: Vec3) -> f32 {
    angle_signed_on_axis(v1 - v2, v3 - v2, axis)
}

/// `angle_tri_v3` — the 3 interior angles of a triangle.
pub fn angle_tri(v1: Vec3, v2: Vec3, v3: Vec3) -> [f32; 3] {
    let mut ed1 = v3 - v1;
    let mut ed2 = v1 - v2;
    let mut ed3 = v2 - v3;
    ed1.normalize();
    ed2.normalize();
    ed3.normalize();
    let a0 = std::f32::consts::PI - ed1.angle_normalized(ed2);
    let a1 = std::f32::consts::PI - ed2.angle_normalized(ed3);
    [a0, a1, std::f32::consts::PI - (a0 + a1)]
}

/// `angle_quad_v3` — the 4 interior angles of a quad.
pub fn angle_quad(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> [f32; 4] {
    let mut ed = [v4 - v1, v1 - v2, v2 - v3, v3 - v4];
    for e in ed.iter_mut() {
        e.normalize();
    }
    [
        std::f32::consts::PI - ed[0].angle_normalized(ed[1]),
        std::f32::consts::PI - ed[1].angle_normalized(ed[2]),
        std::f32::consts::PI - ed[2].angle_normalized(ed[3]),
        std::f32::consts::PI - ed[3].angle_normalized(ed[0]),
    ]
}

/// `angle_poly_v3` — interior angles of an ngon.
pub fn angle_poly(verts: &[Vec3]) -> Vec<f32> {
    let len = verts.len();
    let mut angles = Vec::with_capacity(len);
    let mut vec = [Vec3::ZERO; 3];
    vec[2] = verts[len - 1] - verts[0];
    vec[2].normalize();
    for i in 0..len {
        vec[i % 3] = verts[(i + 1) % len] - verts[i % len];
        vec[i % 3].normalize();
        angles.push(std::f32::consts::PI - vec[(i + 2) % 3].angle_normalized(vec[i % 3]));
    }
    angles
}

/* ---- math_vector.cc: geometry ---- */

/// `project_v2_v2v2` — project `p` onto `v_proj` (zero-safe).
pub fn project_v2(p: Vec2, v_proj: Vec2) -> Vec2 {
    if v_proj.is_zero() {
        return Vec2::ZERO;
    }
    v_proj * (p.dot(v_proj) / v_proj.dot(v_proj))
}

/// `project_v3_v3v3` — project `p` onto `v_proj` (zero-safe).
pub fn project_v3(p: Vec3, v_proj: Vec3) -> Vec3 {
    if v_proj.is_zero() {
        return Vec3::ZERO;
    }
    v_proj * (p.dot(v_proj) / v_proj.dot(v_proj))
}

/// `project_plane_v3_v3v3` — project `p` onto plane through origin, unnormalized normal.
pub fn project_plane_v3(p: Vec3, v_plane: Vec3) -> Vec3 {
    let mul = p.dot(v_plane) / v_plane.dot(v_plane);
    p + v_plane * (-mul)
}

/// `project_plane_normalized_v3_v3v3` — unit normal variant.
pub fn project_plane_normalized(p: Vec3, v_plane: Vec3) -> Vec3 {
    let mul = p.dot(v_plane);
    p + v_plane * (-mul)
}

/// `project_v3_plane` — project `out` onto the plane (normal, point).
pub fn project_point_to_plane(out: Vec3, plane_no: Vec3, plane_co: Vec3) -> Vec3 {
    let vector = out - plane_co;
    let mul = vector.dot(plane_no) / plane_no.length_squared();
    out + plane_no * (-mul)
}

/// `bisect_v3_v3v3v3` — bisector direction at `b` between `a` and `c`.
pub fn bisect_v3(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let mut d_12 = b - a;
    let mut d_23 = c - b;
    d_12.normalize();
    d_23.normalize();
    let mut r = d_12 + d_23;
    r.normalize();
    r
}

/// `ortho_v3_v3` — any vector perpendicular to `v` (axis-switched variant).
pub fn ortho_v3(v: Vec3) -> Vec3 {
    match v.abs_max_axis() {
        0 => Vec3::new(-v.y - v.z, v.x, v.x),
        1 => Vec3::new(v.y, -v.x - v.z, v.y),
        _ => Vec3::new(v.z, v.z, -v.x - v.y),
    }
}

/// `ortho_v2_v2` — 2D perpendicular.
pub fn ortho_v2(v: Vec2) -> Vec2 {
    Vec2::new(-v.y, v.x)
}

/// `rotate_v2_v2fl` — rotate 2D point by `angle`.
pub fn rotate_v2(p: Vec2, angle: f32) -> Vec2 {
    let (si, co) = angle.sin_cos();
    Vec2::new(co * p.x - si * p.y, si * p.x + co * p.y)
}

/* ---- math_vector.cc: other ---- */

/// `print_v2` — debug print.
pub fn print_v2(s: &str, v: Vec2) {
    println!("{}: {:.8} {:.8}", s, v.x, v.y);
}

/// `print_v3` — debug print.
pub fn print_v3(s: &str, v: Vec3) {
    println!("{}: {:.8} {:.8} {:.8}", s, v.x, v.y, v.z);
}

/// `print_v4` — debug print.
pub fn print_v4(s: &str, v: Vec4) {
    println!("{}: {:.8} {:.8} {:.8} {:.8}", s, v.x, v.y, v.z, v.w);
}

/// `minmax_v4v4_v4` — grow `(min, max)` to include `vec`.
pub fn minmax_v4(min: &mut Vec4, max: &mut Vec4, vec: Vec4) {
    *min = min.min(vec);
    *max = max.max(vec);
}

/// `minmax_v3v3_v3`.
pub fn minmax_v3(min: &mut Vec3, max: &mut Vec3, vec: Vec3) {
    *min = min.min(vec);
    *max = max.max(vec);
}

/// `minmax_v2v2_v2`.
pub fn minmax_v2(min: &mut Vec2, max: &mut Vec2, vec: Vec2) {
    *min = min.min(vec);
    *max = max.max(vec);
}

/// `dist_ensure_v3_v3fl` — move `v1` to exactly `dist` from `v2`.
pub fn dist_ensure_v3(v1: &mut Vec3, v2: Vec3, dist: f32) {
    if v2 != *v1 {
        let mut nor = *v1 - v2;
        nor.normalize();
        *v1 = v2 + nor * dist;
    }
}

/// `dist_ensure_v2_v2fl`.
pub fn dist_ensure_v2(v1: &mut Vec2, v2: Vec2, dist: f32) {
    if v2 != *v1 {
        let mut nor = *v1 - v2;
        nor.normalize();
        *v1 = v2 + nor * dist;
    }
}

/// `axis_sort_v3` — return axis indices sorted by descending value.
pub fn axis_sort_v3(axis_values: Vec3) -> [usize; 3] {
    let mut v = [axis_values.x, axis_values.y, axis_values.z];
    let mut order = [0usize, 1, 2];
    macro_rules! swap_axis {
        ($a:expr, $b:expr) => {
            v.swap($a, $b);
            order.swap($a, $b);
        };
    }
    if v[0] < v[1] {
        if v[2] < v[0] {
            swap_axis!(0, 2);
        }
    } else if v[1] < v[2] {
        swap_axis!(0, 1);
    } else {
        swap_axis!(0, 2);
    }
    if v[2] < v[1] {
        swap_axis!(1, 2);
    }
    order
}

/* ---- math_vector.cc: array (vn) functions ---- */

/// `dot_vn_vn` — f64 dot product of two slices.
pub fn dot_vn(a: &[f32], b: &[f32]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| *x as f64 * *y as f64).sum()
}

/// `len_squared_vn`.
pub fn len_squared_vn(a: &[f32]) -> f64 {
    a.iter().map(|x| (*x as f64) * (*x as f64)).sum()
}

/// `normalize_vn_vn` — normalize into `tar`, returns magnitude (0 for tiny).
pub fn normalize_vn_vn(tar: &mut [f32], src: &[f32]) -> f32 {
    let d = len_squared_vn(src);
    if d > 1.0e-35 {
        let d_sqrt = d.sqrt() as f32;
        for (t, s) in tar.iter_mut().zip(src.iter()) {
            *t = *s * (1.0 / d_sqrt);
        }
        d_sqrt
    } else {
        tar.fill(0.0);
        0.0
    }
}

/// `normalize_vn` — in-place.
pub fn normalize_vn(array: &mut [f32]) -> f32 {
    let d = len_squared_vn(array);
    if d > 1.0e-35 {
        let d_sqrt = d.sqrt() as f32;
        for t in array.iter_mut() {
            *t *= 1.0 / d_sqrt;
        }
        d_sqrt
    } else {
        array.fill(0.0);
        0.0
    }
}

/// `range_vn_fl` — `start + step*i` for each element.
pub fn range_vn_fl(tar: &mut [f32], start: f32, step: f32) {
    for (i, t) in tar.iter_mut().enumerate() {
        *t = start + step * i as f32;
    }
}

/// `negate_vn`.
pub fn negate_vn(tar: &mut [f32]) {
    for t in tar.iter_mut() {
        *t *= -1.0;
    }
}

/// `negate_vn_vn`.
pub fn negate_vn_vn(tar: &mut [f32], src: &[f32]) {
    for (t, s) in tar.iter_mut().zip(src.iter()) {
        *t = -*s;
    }
}

/// `mul_vn_vn` — `tar *= src` elementwise.
pub fn mul_vn_vn(tar: &mut [f32], src: &[f32]) {
    for (t, s) in tar.iter_mut().zip(src.iter()) {
        *t *= *s;
    }
}

/// `mul_vn_vnvn` — `tar = a * b` elementwise.
pub fn mul_vn_vnvn(tar: &mut [f32], a: &[f32], b: &[f32]) {
    for ((t, x), y) in tar.iter_mut().zip(a.iter()).zip(b.iter()) {
        *t = *x * *y;
    }
}

/// `mul_vn_fl` — `tar *= f`.
pub fn mul_vn_fl(tar: &mut [f32], f: f32) {
    for t in tar.iter_mut() {
        *t *= f;
    }
}

/// `mul_vn_vn_fl` — `tar = src * f`.
pub fn mul_vn_vn_fl(tar: &mut [f32], src: &[f32], f: f32) {
    for (t, s) in tar.iter_mut().zip(src.iter()) {
        *t = *s * f;
    }
}

/// `add_vn_vn` — `tar += src`.
pub fn add_vn_vn(tar: &mut [f32], src: &[f32]) {
    for (t, s) in tar.iter_mut().zip(src.iter()) {
        *t += *s;
    }
}

/// `add_vn_vnvn` — `tar = a + b`.
pub fn add_vn_vnvn(tar: &mut [f32], a: &[f32], b: &[f32]) {
    for ((t, x), y) in tar.iter_mut().zip(a.iter()).zip(b.iter()) {
        *t = *x + *y;
    }
}

/// `sub_vn_vn` — `tar -= src`.
pub fn sub_vn_vn(tar: &mut [f32], src: &[f32]) {
    for (t, s) in tar.iter_mut().zip(src.iter()) {
        *t -= *s;
    }
}

/// `sub_vn_vnvn` — `tar = a - b`.
pub fn sub_vn_vnvn(tar: &mut [f32], a: &[f32], b: &[f32]) {
    for ((t, x), y) in tar.iter_mut().zip(a.iter()).zip(b.iter()) {
        *t = *x - *y;
    }
}

/// `interp_vn_vn` — `tar = s*tar + t*src`.
pub fn interp_vn_vn(tar: &mut [f32], src: &[f32], t: f32) {
    let s = 1.0 - t;
    for (x, y) in tar.iter_mut().zip(src.iter()) {
        *x = s * *x + t * *y;
    }
}

/// `add_vn_vn_d` — f64 `tar += src`.
pub fn add_vn_vn_d(tar: &mut [f64], src: &[f64]) {
    for (t, s) in tar.iter_mut().zip(src.iter()) {
        *t += *s;
    }
}

/// `add_vn_vnvn_d` — f64 `tar = a + b`.
pub fn add_vn_vnvn_d(tar: &mut [f64], a: &[f64], b: &[f64]) {
    for ((t, x), y) in tar.iter_mut().zip(a.iter()).zip(b.iter()) {
        *t = *x + *y;
    }
}

/// `mul_vn_db` — f64 `tar *= f`.
pub fn mul_vn_db(tar: &mut [f64], f: f64) {
    for t in tar.iter_mut() {
        *t *= f;
    }
}

/// `interp_v3_v3v3_db` — f64 lerp of `[f64;3]`.
pub fn interp_v3_db(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let s = 1.0 - t;
    [
        s * a[0] + t * b[0],
        s * a[1] + t * b[1],
        s * a[2] + t * b[2],
    ]
}

/// `interp_v2_v2v2_db`.
pub fn interp_v2_db(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    let s = 1.0 - t;
    [s * a[0] + t * b[0], s * a[1] + t * b[1]]
}
