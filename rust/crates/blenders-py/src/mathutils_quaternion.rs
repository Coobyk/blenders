/// Port of `mathutils_quaternion.cc` —
/// `mathutils.Quaternion` (wxyz, axis-angle,
/// slerp, rotate, to_matrix/euler).
/// `Quaternion` (verbatim — wxyz + frozen).
#[derive(Clone, Copy)]
pub struct MathutilsQuaternion {
    /// `quat` [w,x,y,z].
    pub quat: [f32; 4],
    /// `is_frozen`.
    pub is_frozen: bool,
}
impl MathutilsQuaternion {
    /// `Quaternion(axis, angle)`.
    pub fn from_axis_angle(axis: [f32; 3], angle: f32) -> Self {
        let l = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if l == 0.0 {
            return Self { quat: [1.0, 0.0, 0.0, 0.0], is_frozen: false };
        }
        let s = (angle * 0.5).sin() / l;
        Self {
            quat: [(angle * 0.5).cos(), axis[0] * s, axis[1] * s, axis[2] * s],
            is_frozen: false,
        }
    }
    /// `normalized`.
    pub fn normalized(&self) -> Self {
        let l = (self.quat[0].powi(2)
            + self.quat[1].powi(2)
            + self.quat[2].powi(2)
            + self.quat[3].powi(2))
        .sqrt();
        if l == 0.0 {
            return *self;
        }
        Self {
            quat: [
                self.quat[0] / l,
                self.quat[1] / l,
                self.quat[2] / l,
                self.quat[3] / l,
            ],
            is_frozen: self.is_frozen,
        }
    }
    /// `slerp`.
    pub fn slerp(&self, other: &Self, t: f32) -> Self {
        let mut q1 = self.normalized().quat;
        let q2 = other.normalized().quat;
        let mut dot = q1.iter().zip(&q2).map(|(a, b)| a * b).sum::<f32>();
        let mut q2 = q2;
        if dot < 0.0 {
            dot = -dot;
            for v in &mut q2 {
                *v = -*v;
            }
        }
        let (w, angle) = if dot > 0.9995 {
            (t, 0.0)
        } else {
            ((dot.clamp(-1.0, 1.0).acos() * t).sin() / (1.0 - dot * dot).sqrt(), 1.0)
        };
        let _ = angle;
        for i in 0..4 {
            q1[i] = q1[i] * (1.0 - t) + q2[i] * w;
        }
        Self { quat: q1, is_frozen: self.is_frozen }.normalized()
    }
}
