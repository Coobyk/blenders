//! Port of `BLI_math_vector.hh` / `math_vector.cc` (Vec2/Vec3/Vec4).

use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::base;

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
