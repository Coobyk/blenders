//! `mpq_class` (GMP `mpq_t`) stand-in for the `WITH_GMP`-gated exact code:
//! `num_rational::BigRational` (pure-Rust arbitrary-precision rationals).
//! `mpq2`/`mpq3`/`mpq4` map to `Mpq2`/`Mpq3`/`Mpq4` — `VecBase<mpq_class, N>`
//! equivalents with field access, arithmetic, `dot`, and `get_d` approx.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use std::ops::{Add, Index, IndexMut, Mul, Neg, Sub};

pub type Mpq = BigRational;

pub fn mpq(x: f64) -> Mpq {
    BigRational::from_float(x).unwrap_or_else(BigRational::zero)
}
pub fn mpq_i(x: i64) -> Mpq {
    BigRational::from_integer(BigInt::from(x))
}
pub fn mpq_dot<const N: usize>(a: &[Mpq; N], b: &[Mpq; N]) -> Mpq {
    let mut s = BigRational::zero();
    for i in 0..N {
        s += &a[i] * &b[i];
    }
    s
}
pub fn mpq_to_f64(v: &Mpq) -> f64 {
    v.to_f64().unwrap_or(f64::NAN)
}
pub fn mpq_abs(v: &Mpq) -> Mpq {
    v.abs()
}
pub fn mpq_sgn(v: &Mpq) -> i32 {
    if v > &BigRational::zero() {
        1
    } else if v < &BigRational::zero() {
        -1
    } else {
        0
    }
}

macro_rules! mpq_vec {
    ($name:ident, $n:expr, $($field:ident : $idx:expr),+) => {
        #[derive(Clone, PartialEq)]
        pub struct $name {
            $(pub $field: Mpq),+
        }
        impl $name {
            pub fn zero() -> Self {
                $name { $($field: BigRational::zero()),+ }
            }
            pub fn new($($field: Mpq),+) -> Self {
                $name { $($field),+ }
            }
            pub fn from_f64s(v: [f64; $n]) -> Self {
                Self::new($(mpq(v[$idx])),+)
            }
            /// `v.x.get_d()` per component -> approx double array.
            pub fn get_d(&self) -> [f64; $n] {
                [$(mpq_to_f64(&self.$field)),+]
            }
            pub fn dot(&self, o: &Self) -> Mpq {
                let mut s = BigRational::zero();
                $(s += &self.$field * &o.$field;)+
                s
            }
            pub fn cross(a: &Self, b: &Self) -> Self
            where
                Self: MpqCross,
            {
                MpqCross::cross(a, b)
            }
            pub fn norm2(&self) -> Mpq {
                self.dot(self)
            }
            pub fn to_array(&self) -> [Mpq; $n] {
                [$(self.$field.clone()),+]
            }
        }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "(")?;
                $(write!(f, "{}", self.$field.to_f64().unwrap_or(f64::NAN))?; write!(f, ", ")?;)+
                write!(f, ")")
            }
        }
        impl Index<usize> for $name {
            type Output = Mpq;
            fn index(&self, i: usize) -> &Mpq {
                match i { $($idx => &self.$field,)+ _ => panic!("index") }
            }
        }
        impl IndexMut<usize> for $name {
            fn index_mut(&mut self, i: usize) -> &mut Mpq {
                match i { $($idx => &mut self.$field,)+ _ => panic!("index") }
            }
        }
        impl<'b> Sub<&'b $name> for &'b $name {
            type Output = $name;
            fn sub(self, o: &'b $name) -> $name {
                $name::new($(&self.$field - &o.$field),+)
            }
        }
        impl<'b> Add<&'b $name> for &'b $name {
            type Output = $name;
            fn add(self, o: &'b $name) -> $name {
                $name::new($(&self.$field + &o.$field),+)
            }
        }
        impl<'b> Mul<&'b Mpq> for &'b $name {
            type Output = $name;
            fn mul(self, s: &'b Mpq) -> $name {
                $name::new($(&self.$field * s),+)
            }
        }
        impl<'b> Mul<&'b $name> for &'b Mpq {
            type Output = $name;
            fn mul(self, v: &'b $name) -> $name {
                $name::new($(&v.$field * self),+)
            }
        }
        impl Neg for &$name {
            type Output = $name;
            fn neg(self) -> $name {
                $name::new($(-(&self.$field)),+)
            }
        }
    };
}

mpq_vec!(Mpq2, 2, x: 0, y: 1);
mpq_vec!(Mpq3, 3, x: 0, y: 1, z: 2);
mpq_vec!(Mpq4, 4, x: 0, y: 1, z: 2, w: 3);

pub trait MpqCross {
    fn cross(a: &Self, b: &Self) -> Self;
}
impl MpqCross for Mpq3 {
    /// `cross_v3_v3v3`.
    fn cross(a: &Self, b: &Self) -> Self {
        Mpq3::new(
            &a.y * &b.z - &a.z * &b.y,
            &a.z * &b.x - &a.x * &b.z,
            &a.x * &b.y - &a.y * &b.x,
        )
    }
}
impl MpqCross for Mpq2 {
    fn cross(_a: &Self, _b: &Self) -> Self {
        unimplemented!("mpq2 has no 3d cross")
    }
}
impl MpqCross for Mpq4 {
    fn cross(_a: &Self, _b: &Self) -> Self {
        unimplemented!("mpq4 has no 3d cross")
    }
}

impl Mpq2 {
    /// `math::cross` 2D z-component.
    pub fn cross_z(&self, o: &Self) -> Mpq {
        &self.x * &o.y - &self.y * &o.x
    }
}

impl Default for Mpq2 {
    fn default() -> Self {
        Self::zero()
    }
}
impl Default for Mpq3 {
    fn default() -> Self {
        Self::zero()
    }
}
impl Default for Mpq4 {
    fn default() -> Self {
        Self::zero()
    }
}
impl Eq for Mpq2 {}
impl Eq for Mpq3 {}
impl Eq for Mpq4 {}
