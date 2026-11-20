//! Port of Blender's `BLI_math` library (source/blender/blenlib/).
//!
//! Conventions match Blender exactly:
//! - Matrices are indexed `mat[col][row]`; `mat[3]` of a `Mat4` is the
//!   translation column (same as `MatBase::location()`).
//! - Quaternions store `(w, x, y, z)` like `QuaternionBase`.
//! - Angles are radians, right-handed coordinate system.

pub mod base;
pub mod blend;
pub mod color;
pub mod euler;
pub mod filter;
pub mod geom;
pub mod half;
pub mod interp;
pub mod matrix;
pub mod matrix_c;
pub mod quaternion;
pub mod solvers;
pub mod statistics;
pub mod vector;
pub mod vector_inline;

pub use euler::{Euler3, EulerOrder, EulerXYZ};
pub use matrix::{Mat2, Mat3, Mat4};
pub use quaternion::{AxisAngle, Quaternion};
pub use vector::{Vec2, Vec3, Vec4};
