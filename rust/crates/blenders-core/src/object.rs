//! Port of `Object` (DNA_object_types.h / BKE_object).
//!
//! Currently the ID header plus the transform triple every object has —
//! loc/rot/scale and the derived `obmat` world matrix, matching
//! `BKE_object_mat4_set`/`BKE_object_to_mat4`.

use crate::id::{Id, IdKind};
use blenders_math::{Euler3, EulerOrder, Mat4, Vec3};

/// `eObjectType` (DNA_object_types.h).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ObjectType {
    Empty = 0,
    Mesh = 1,
    Curve = 2,
    Surface = 3,
    Text = 4,
    MetaBall = 5,
    Lattice = 7,
    Light = 10,
    Camera = 11,
    Speaker = 12,
    LightProbe = 13,
    Armature = 14,
    GreasePencil = 15,
    PointCloud = 16,
    Volume = 17,
}

/// `eRotationModes` (DNA_action_types.h) — object's rotation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RotationMode {
    QuatWXYZ = 0,
    EulerXYZ = 1,
    EulerXZY = 2,
    EulerYXZ = 3,
    EulerYZX = 4,
    EulerZXY = 5,
    EulerZYX = 6,
    AxisAngle = 8,
}

/// Port of `struct Object` — the transform-relevant slice.
#[derive(Debug)]
pub struct Object {
    /// `Object.id` header.
    pub id: Id,
    /// `Object.type`.
    pub object_type: ObjectType,
    /// `Object.loc[3]`.
    pub loc: Vec3,
    /// `Object.dloc[3]` — delta location.
    pub dloc: Vec3,
    /// `Object.scale[3]`.
    pub scale: Vec3,
    /// `Object.dscale[3]`.
    pub dscale: Vec3,
    /// `Object.rot[3]` — euler rotation (radians).
    pub rot: Vec3,
    /// `Object.drot[3]` — delta rotation.
    pub drot: Vec3,
    /// `Object.quat[4]` — w,x,y,z.
    pub quat: blenders_math::Quaternion,
    /// `Object.rotAxis[3]` + `Object.rotAngle` — axis-angle rotation.
    pub rot_axis: Vec3,
    pub rot_angle: f32,
    /// `Object.rotmode`.
    pub rotmode: RotationMode,
    /// `Object.parent` — raw pointer in Blender; index into a Main list later.
    pub parent: Option<usize>,
    /// `Object.data` — pointer to geometry datablock (mesh, curve, …).
    pub data: Option<usize>,
    /// `Object.object_to_world` — evaluated world matrix (`obmat`).
    pub object_to_world: Mat4,
    /// `Object.world_to_object` — inverse (`imat`).
    pub world_to_object: Mat4,
}

impl Object {
    /// `BKE_object_new` — create an object datablock.
    pub fn new(object_type: ObjectType, name: &str) -> Self {
        Self {
            id: Id::new(IdKind::Object, name),
            object_type,
            loc: Vec3::ZERO,
            dloc: Vec3::ZERO,
            scale: Vec3::splat(1.0),
            dscale: Vec3::splat(1.0),
            rot: Vec3::ZERO,
            drot: Vec3::ZERO,
            quat: blenders_math::Quaternion::IDENTITY,
            rot_axis: Vec3::Y,
            rot_angle: 0.0,
            rotmode: RotationMode::QuatWXYZ,
            parent: None,
            data: None,
            object_to_world: Mat4::IDENTITY,
            world_to_object: Mat4::IDENTITY,
        }
    }

    /// `BKE_object_to_mat4` — compute the loc/rot/scale matrix from the
    /// current transform fields incl. deltas (`loc + dloc` etc.).
    pub fn to_mat4(&self) -> Mat4 {
        let loc = self.loc + self.dloc;
        let scale = self.scale * self.dscale;
        let rot = self.rotation_mat();
        Mat4::from_loc_rot_scale(loc, rot, scale)
    }

    /// `BKE_object_rot_to_mat3` — rotation from the active `rotmode`.
    fn rotation_mat(&self) -> blenders_math::Mat3 {
        match self.rotmode {
            RotationMode::QuatWXYZ => self.quat.to_mat3(),
            RotationMode::AxisAngle => {
                blenders_math::quaternion::mat3_from_axis_angle(
                    blenders_math::AxisAngle {
                        axis: self.rot_axis.normalized(),
                        angle: self.rot_angle + self.drot.y,
                    },
                )
            }
            mode => {
                let order = match mode {
                    RotationMode::EulerXYZ => EulerOrder::Xyz,
                    RotationMode::EulerXZY => EulerOrder::Xzy,
                    RotationMode::EulerYXZ => EulerOrder::Yxz,
                    RotationMode::EulerYZX => EulerOrder::Yzx,
                    RotationMode::EulerZXY => EulerOrder::Zxy,
                    RotationMode::EulerZYX => EulerOrder::Zyx,
                    _ => EulerOrder::Xyz,
                };
                let angles = self.rot + self.drot;
                Euler3::new(angles, order).to_mat3()
            }
        }
    }

    /// `BKE_object_mat4_set` — recompute `object_to_world` and its inverse.
    pub fn update_matrices(&mut self) {
        self.object_to_world = self.to_mat4();
        self.world_to_object = self
            .object_to_world
            .inverted()
            .unwrap_or(Mat4::IDENTITY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_matrix() {
        let mut ob = Object::new(ObjectType::Mesh, "Cube");
        ob.loc = Vec3::new(1.0, 2.0, 3.0);
        ob.scale = Vec3::splat(2.0);
        ob.update_matrices();
        let p = ob.object_to_world.transform_point(Vec3::X);
        // (1,0,0) scaled x2 then translated → (3,2,3).
        assert!((p - Vec3::new(3.0, 2.0, 3.0)).length() < 1e-5);
    }
}
