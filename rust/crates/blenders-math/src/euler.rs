//! Port of `BLI_math_euler_types.hh` + euler conversion paths from
//! `BLI_math_matrix.hh` (`normalized_to_eul2`, `to_euler`, `to_quaternion`).

use std::ops::{Index, IndexMut, Neg};

use crate::matrix::Mat3;
use crate::quaternion::Quaternion;
use crate::vector::Vec3;

/// `EulerOrder` — must match `eRotationModes` in `DNA_action_types.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EulerOrder {
    Xyz = 1,
    Xzy,
    Yxz,
    Yzx,
    Zxy,
    Zyx,
}

impl Default for EulerOrder {
    fn default() -> Self {
        EulerOrder::Xyz
    }
}

impl EulerOrder {
    /// `parity()` — true when the axis sequence is "shuffled"
    /// (XZY, YXZ, ZYX), false for cyclic orders.
    pub fn parity(self) -> bool {
        matches!(self, EulerOrder::Xzy | EulerOrder::Yxz | EulerOrder::Zyx)
    }

    /// `i_index()` — axis index (0=x,1=y,2=z) of the first rotation.
    pub fn i_index(self) -> usize {
        match self {
            EulerOrder::Xyz | EulerOrder::Xzy => 0,
            EulerOrder::Yxz | EulerOrder::Yzx => 1,
            EulerOrder::Zxy | EulerOrder::Zyx => 2,
        }
    }

    /// `j_index()` — axis index of the second rotation.
    pub fn j_index(self) -> usize {
        match self {
            EulerOrder::Yxz | EulerOrder::Zxy => 0,
            EulerOrder::Xyz | EulerOrder::Zyx => 1,
            EulerOrder::Xzy | EulerOrder::Yzx => 2,
        }
    }

    /// `k_index()` — axis index of the third rotation.
    pub fn k_index(self) -> usize {
        match self {
            EulerOrder::Yzx | EulerOrder::Zyx => 0,
            EulerOrder::Xzy | EulerOrder::Zxy => 1,
            EulerOrder::Xyz | EulerOrder::Yxz => 2,
        }
    }
}

/// `EulerXYZBase<float>` — fixed XYZ-order Euler triple (radians).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct EulerXYZ {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// `Euler3Base<float>` — Euler triple with arbitrary order.
/// Angles stored as XYZ triple like `Euler3Base::xyz_`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Euler3 {
    pub xyz: Vec3,
    pub order: EulerOrder,
}

impl EulerXYZ {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// `to_quaternion(EulerXYZ)` — port of `eul_to_quat` (XYZ order).
    pub fn to_quaternion(self) -> Quaternion {
        let (sin_i, cos_i) = (self.x * 0.5).sin_cos();
        let (sin_j, cos_j) = (self.y * 0.5).sin_cos();
        let (sin_k, cos_k) = (self.z * 0.5).sin_cos();
        let cos_cos = cos_i * cos_k;
        let cos_sin = cos_i * sin_k;
        let sin_cos = sin_i * cos_k;
        let sin_sin = sin_i * sin_k;

        Quaternion::new(
            cos_j * cos_cos + sin_j * sin_sin,
            cos_j * sin_cos - sin_j * cos_sin,
            cos_j * sin_sin + sin_j * cos_cos,
            cos_j * cos_sin - sin_j * sin_cos,
        )
    }

    /// `to_mat3` via quaternion (`eul_to_mat3` for XYZ order).
    pub fn to_mat3(self) -> Mat3 {
        self.to_quaternion().to_mat3()
    }
}

impl Euler3 {
    pub const ORDER: EulerOrder = EulerOrder::Xyz;

    #[inline]
    pub const fn new(xyz: Vec3, order: EulerOrder) -> Self {
        Self { xyz, order }
    }

    /// First-axis rotation value (`eul.i()`).
    #[inline]
    pub fn i(&self) -> f32 {
        self.xyz[self.order.i_index()]
    }
    /// Second-axis rotation value (`eul.j()`).
    #[inline]
    pub fn j(&self) -> f32 {
        self.xyz[self.order.j_index()]
    }
    /// Third-axis rotation value (`eul.k()`).
    #[inline]
    pub fn k(&self) -> f32 {
        self.xyz[self.order.k_index()]
    }

    /// `to_quaternion(Euler3)` — port of `BLI_math_euler.hh`: swizzle the
    /// angles to an XYZ triple (with parity flip on Y), convert, then
    /// swizzle the quaternion's xyz components back to real axes.
    pub fn to_quaternion(self) -> Quaternion {
        // eul.ijk() — the (i, j, k) components as an XYZ triple.
        let mut eul_xyz = EulerXYZ::new(self.i(), self.j(), self.k());
        if self.order.parity() {
            eul_xyz.y = -eul_xyz.y;
        }
        let quat = eul_xyz.to_quaternion();

        let mut quat_xyz = [0.0f32; 3];
        quat_xyz[self.order.i_index()] = quat.x;
        quat_xyz[self.order.j_index()] = if self.order.parity() { -quat.y } else { quat.y };
        quat_xyz[self.order.k_index()] = quat.z;

        Quaternion::new(quat.w, quat_xyz[0], quat_xyz[1], quat_xyz[2])
    }

    /// `to_mat3` via quaternion.
    pub fn to_mat3(self) -> Mat3 {
        self.to_quaternion().to_mat3()
    }
}

impl Neg for Euler3 {
    type Output = Euler3;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.xyz, self.order)
    }
}

impl Index<usize> for EulerXYZ {
    type Output = f32;
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("euler index {i} out of range"),
        }
    }
}

impl IndexMut<usize> for EulerXYZ {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("euler index {i} out of range"),
        }
    }
}

/// `detail::normalized_to_eul2` for XYZ order — two candidate solutions.
/// `mat` must be a normalized rotation matrix (assert in Blender).
fn normalized_to_eul2_xyz(mat: &Mat3) -> (EulerXYZ, EulerXYZ) {
    let mut eul1 = EulerXYZ::ZERO;
    let mut eul2 = EulerXYZ::ZERO;

    let cy = mat[0][0].hypot(mat[0][1]);
    if cy > 16.0 * f32::EPSILON {
        eul1.x = mat[1][2].atan2(mat[2][2]);
        eul1.y = (-mat[0][2]).atan2(cy);
        eul1.z = mat[0][1].atan2(mat[0][0]);

        eul2.x = (-mat[1][2]).atan2(-mat[2][2]);
        eul2.y = (-mat[0][2]).atan2(-cy);
        eul2.z = (-mat[0][1]).atan2(-mat[0][0]);
    } else {
        eul1.x = (-mat[2][1]).atan2(mat[1][1]);
        eul1.y = (-mat[0][2]).atan2(cy);
        eul1.z = 0.0;

        eul2 = eul1;
    }
    (eul1, eul2)
}

/// `detail::normalized_to_eul2` for arbitrary `EulerOrder`.
fn normalized_to_eul2_order(mat: &Mat3, order: EulerOrder) -> (Euler3, Euler3) {
    let i = order.i_index();
    let j = order.j_index();
    let k = order.k_index();

    let mut a1 = [0.0f32; 3];
    let mut a2 = [0.0f32; 3];

    let cy = mat[i][i].hypot(mat[i][j]);
    if cy > 16.0 * f32::EPSILON {
        a1[i] = mat[j][k].atan2(mat[k][k]);
        a1[j] = (-mat[i][k]).atan2(cy);
        a1[k] = mat[i][j].atan2(mat[i][i]);

        a2[i] = (-mat[j][k]).atan2(-mat[k][k]);
        a2[j] = (-mat[i][k]).atan2(-cy);
        a2[k] = (-mat[i][j]).atan2(-mat[i][i]);
    } else {
        a1[i] = (-mat[k][j]).atan2(mat[j][j]);
        a1[j] = (-mat[i][k]).atan2(cy);
        a1[k] = 0.0;
        a2 = a1;
    }

    let mut eul1 = Euler3::new(Vec3::new(a1[0], a1[1], a1[2]), order);
    let mut eul2 = Euler3::new(Vec3::new(a2[0], a2[1], a2[2]), order);
    if order.parity() {
        eul1 = -eul1;
        eul2 = -eul2;
    }
    (eul1, eul2)
}

/// `to_euler(Mat3)` → `EulerXYZ` — picks the candidate with the smallest
/// Manhattan length, like Blender.
pub fn mat3_to_euler_xyz(mat: &Mat3) -> EulerXYZ {
    let (eul1, eul2) = normalized_to_eul2_xyz(mat);
    let m1 = eul1.x.abs() + eul1.y.abs() + eul1.z.abs();
    let m2 = eul2.x.abs() + eul2.y.abs() + eul2.z.abs();
    if m1 > m2 { eul2 } else { eul1 }
}

/// `to_euler(Mat3, EulerOrder)` → `Euler3`.
pub fn mat3_to_euler(mat: &Mat3, order: EulerOrder) -> Euler3 {
    let (eul1, eul2) = normalized_to_eul2_order(mat, order);
    let m1 = eul1.xyz.length_manhattan();
    let m2 = eul2.xyz.length_manhattan();
    if m1 > m2 { eul2 } else { eul1 }
}

/// `to_euler(Quaternion)` → `EulerXYZ` (via unit matrix).
pub fn quat_to_euler_xyz(q: Quaternion) -> EulerXYZ {
    mat3_to_euler_xyz(&q.to_mat3())
}

/// `to_euler(Quaternion, EulerOrder)` → `Euler3`.
pub fn quat_to_euler(q: Quaternion, order: EulerOrder) -> Euler3 {
    mat3_to_euler(&q.to_mat3(), order)
}

/// `compatible_eul` — adjust `eul` to its closest representation near
/// `reference` by ±2π offsets (`eul = eul.wrapped_around(reference)`).
pub fn compatible_eul_xyz(eul: EulerXYZ, reference: EulerXYZ) -> EulerXYZ {
    let mut out = eul;
    for i in 0..3 {
        let d = out[i] - reference[i];
        if d.abs() > std::f32::consts::PI {
            let rounds = (d / std::f32::consts::TAU).round();
            out[i] -= rounds * std::f32::consts::TAU;
        }
    }
    out
}

/// `to_nearest_euler` — like `to_euler` but wraps candidates near `reference`.
pub fn mat3_to_nearest_euler_xyz(mat: &Mat3, reference: EulerXYZ) -> EulerXYZ {
    let (eul1, eul2) = normalized_to_eul2_xyz(mat);
    let eul1 = compatible_eul_xyz(eul1, reference);
    let eul2 = compatible_eul_xyz(eul2, reference);
    let d1 = (eul1.x - reference.x).abs() + (eul1.y - reference.y).abs() + (eul1.z - reference.z).abs();
    let d2 = (eul2.x - reference.x).abs() + (eul2.y - reference.y).abs() + (eul2.z - reference.z).abs();
    if d1 > d2 { eul2 } else { eul1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn xyz_roundtrip() {
        let e = EulerXYZ::new(0.3, -0.4, 0.9);
        let m = e.to_mat3();
        let e2 = mat3_to_euler_xyz(&m);
        assert!(approx(e.x, e2.x), "{} vs {}", e.x, e2.x);
        assert!(approx(e.y, e2.y));
        assert!(approx(e.z, e2.z));
    }

    #[test]
    fn all_orders_roundtrip() {
        for order in [
            EulerOrder::Xyz,
            EulerOrder::Xzy,
            EulerOrder::Yxz,
            EulerOrder::Yzx,
            EulerOrder::Zxy,
            EulerOrder::Zyx,
        ] {
            let e = Euler3::new(Vec3::new(0.3, -0.4, 0.5), order);
            let m = e.to_mat3();
            let e2 = mat3_to_euler(&m, order);
            // Compare via matrix equivalence (angles can alias).
            let m2 = e2.to_mat3();
            for c in 0..3 {
                for r in 0..3 {
                    assert!(
                        approx(m[c][r], m2[c][r]),
                        "order {order:?} mismatch at [{c}][{r}]"
                    );
                }
            }
        }
    }

    #[test]
    fn quat_to_euler() {
        let e = EulerXYZ::new(FRAC_PI_2, 0.0, 0.0);
        let q = e.to_quaternion();
        let m = q.to_mat3();
        // Rotate X by 90°: Y → Z.
        let v = m * crate::vector::Vec3::Y;
        assert!((v - crate::vector::Vec3::Z).length() < 1e-5);
    }
}
