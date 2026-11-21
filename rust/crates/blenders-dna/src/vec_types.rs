//! Port of `DNA_vec_types.h` — plain vec/mat/rect PODs + `DualQuat`.
//! These alias the math crate's `Vec2/Vec3/Vec4/Mat4` where possible.

pub type Vec2f = blenders_math::vector::Vec2;
pub type Vec2i = [i32; 2];
pub type Vec2d = [f64; 2];
pub type Vec3i = [i32; 3];
pub type Vec3f = blenders_math::vector::Vec3;
pub type Vec3d = [f64; 3];
pub type Vec4i = [i32; 4];
pub type Vec4f = blenders_math::vector::Vec4;
pub type Vec4d = [f64; 4];
/// `mat4x4f` — row-major 4x4 (DNA `value[i][j]`).
pub type Mat4x4f = [[f32; 4]; 4];

/// `rcti` — int rect.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Rcti {
    pub xmin: i32,
    pub xmax: i32,
    pub ymin: i32,
    pub ymax: i32,
}

/// `rctf` — float rect.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Rctf {
    pub xmin: f32,
    pub xmax: f32,
    pub ymin: f32,
    pub ymax: f32,
}

/// `DualQuat`.
#[derive(Clone, Copy, Default)]
pub struct DualQuat {
    pub quat: [f32; 4],
    pub trans: [f32; 4],
    pub scale: [[f32; 4]; 4],
    pub scale_weight: f32,
}
