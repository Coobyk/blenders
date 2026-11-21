//! Port of `intern/object.cc` pure parts —
//! `BKE_object_is_mode_compat`, `object_visibility`,
//! `obdata_to_type`, `rot_to_quat`/`quat_to_rot`,
//! `tfm_protected_backup`/`restore`/`tfm_copy`,
//! `object_to_mat3`/`to_mat4`.
//!
//! `PORT_PENDING`: modifier/hook/shaderfx management,
//! `BKE_object_eval_assign_data`/free_*_caches (runtime
//! caches), editmode checks (`*_is_in_editmode*` — runtime
//! flags), `BKE_object_link_modifiers`/`modifier_stack_copy`,
//! `where_is_calc*`/`calc_parent` (depsgraph transform
//! eval), `boundbox_get`/`dimensions_get`/`obdata_size_init`
//! (per-type bound boxes), pose arrays, exists_check
//! (Main lookup).

use blenders_dna::action_types::{ROT_MODE_AXISANGLE, ROT_MODE_QUAT};
use blenders_dna::id::{gs, ID};
use blenders_dna::object_types::*;
use blenders_math::euler::{quat_to_euler, Euler3, EulerOrder};
use blenders_math::quaternion::{axis_angle_to_quat, quat_to_axis_angle};
use blenders_math::{Quaternion, Vec3};

/// `eObjectType` verbatim (`DNA_object_types.h`).
pub const OB_MESH: i32 = 1;
/// `OB_CURVES_LEGACY` (`Curve` data — `cu->ob_type`
/// varies; use `cu_ob_type`).
pub const OB_SURF: i32 = 3;
/// `OB_FONT`.
pub const OB_FONT: i32 = 4;
/// `OB_MBALL`.
pub const OB_MBALL: i32 = 5;
/// `OB_LAMP`.
pub const OB_LAMP: i32 = 10;
/// `OB_CAMERA`.
pub const OB_CAMERA: i32 = 11;
/// `OB_SPEAKER`.
pub const OB_SPEAKER: i32 = 12;
/// `OB_LIGHTPROBE`.
pub const OB_LIGHTPROBE: i32 = 13;
/// `OB_LATTICE`.
pub const OB_LATTICE: i32 = 22;
/// `OB_ARMATURE`.
pub const OB_ARMATURE: i32 = 25;
/// `OB_CURVES`.
pub const OB_CURVES: i32 = 27;
/// `OB_POINTCLOUD`.
pub const OB_POINTCLOUD: i32 = 28;
/// `OB_VOLUME`.
pub const OB_VOLUME: i32 = 29;
/// `OB_GREASE_PENCIL`.
pub const OB_GREASE_PENCIL: i32 = 30;

/// `OB_VISIBLE_*` verbatim (`DNA_object_types.h`).
pub const OB_VISIBLE_SELF: i32 = 1 << 0;
/// `OB_VISIBLE_PARTICLES`.
pub const OB_VISIBLE_PARTICLES: i32 = 1 << 1;
/// `OB_VISIBLE_INSTANCES`.
pub const OB_VISIBLE_INSTANCES: i32 = 1 << 2;

/// `DAG_EVAL_*` verbatim (`BKE_object.hh`).
pub const DAG_EVAL_VIEWPORT: i32 = 0;
/// `DAG_EVAL_RENDER`.
pub const DAG_EVAL_RENDER: i32 = 1;

/// `BKE_object_is_mode_compat` (verbatim).
pub fn object_is_mode_compat(ob_mode: i32, object_mode: i32) -> bool {
    ob_mode == object_mode || (ob_mode & object_mode) != 0
}

/// `BKE_object_visibility` (verbatim — `has_particles`/
/// `has_geometry_set_instances` injected).
pub fn object_visibility(
    ob: &Object,
    dag_eval_mode: i32,
    has_particles: bool,
    has_geometry_set_instances: bool,
) -> i32 {
    if (ob.base_flag & crate::layer::BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT) == 0 {
        return 0;
    }
    let mut visibility = OB_VISIBLE_SELF;
    if has_particles {
        visibility |= OB_VISIBLE_INSTANCES | OB_VISIBLE_PARTICLES;
    } else if ob.transflag & OB_DUPLI != 0 {
        visibility |= OB_VISIBLE_INSTANCES;
    }
    if has_geometry_set_instances {
        visibility |= OB_VISIBLE_INSTANCES;
    }
    if visibility & (OB_VISIBLE_PARTICLES | OB_VISIBLE_INSTANCES) != 0 {
        match dag_eval_mode {
            DAG_EVAL_VIEWPORT => {
                if (ob.duplicator_visibility_flag & OB_DUPLI_FLAG_VIEWPORT) == 0 {
                    visibility &= !OB_VISIBLE_SELF;
                }
            }
            DAG_EVAL_RENDER => {
                if (ob.duplicator_visibility_flag & OB_DUPLI_FLAG_RENDER) == 0 {
                    visibility &= !OB_VISIBLE_SELF;
                }
            }
            _ => {}
        }
    }
    visibility
}

/// `BKE_object_obdata_to_type` (verbatim — `ob_type`
/// injected for `ID_CU_LEGACY`).
pub fn object_obdata_to_type(id_code: u16, cu_ob_type: i32) -> i32 {
    match id_code {
        c if c == gs(b"ME") => OB_MESH,
        c if c == gs(b"CU") => cu_ob_type,
        c if c == gs(b"MB") => OB_MBALL,
        c if c == gs(b"LA") => OB_LAMP,
        c if c == gs(b"SK") => OB_SPEAKER,
        c if c == gs(b"CA") => OB_CAMERA,
        c if c == gs(b"LT") => OB_LATTICE,
        c if c == gs(b"AR") => OB_ARMATURE,
        c if c == gs(b"LP") => OB_LIGHTPROBE,
        c if c == gs(b"CV") => OB_CURVES,
        c if c == gs(b"PT") => OB_POINTCLOUD,
        c if c == gs(b"VO") => OB_VOLUME,
        c if c == gs(b"GP") => OB_GREASE_PENCIL,
        _ => -1,
    }
}

/// `BKE_object_rot_to_quat` (verbatim).
pub fn object_rot_to_quat(ob: &Object) -> Quaternion {
    if ob.rotmode > 0 {
        let order = match ob.rotmode {
            1 => EulerOrder::Xyz,
            2 => EulerOrder::Xzy,
            3 => EulerOrder::Yxz,
            4 => EulerOrder::Yzx,
            5 => EulerOrder::Zxy,
            _ => EulerOrder::Zyx,
        };
        Euler3 {
            xyz: Vec3::new(ob.rot[0], ob.rot[1], ob.rot[2]),
            order,
        }
        .to_quaternion()
    } else if ob.rotmode == ROT_MODE_AXISANGLE {
        axis_angle_to_quat(
            Vec3::new(ob.rot_axis[0], ob.rot_axis[1], ob.rot_axis[2]),
            ob.rot_angle,
        )
    } else {
        Quaternion::new(ob.quat[0], ob.quat[1], ob.quat[2], ob.quat[3]).normalized()
    }
}

/// `BKE_object_quat_to_rot` (verbatim).
pub fn object_quat_to_rot(ob: &mut Object, quat: Quaternion) {
    match ob.rotmode {
        ROT_MODE_QUAT => {
            let q = quat.normalized();
            ob.quat = [q.w, q.x, q.y, q.z];
        }
        ROT_MODE_AXISANGLE => {
            let (axis, angle) = quat_to_axis_angle(quat);
            ob.rot_axis = [axis.x, axis.y, axis.z];
            ob.rot_angle = angle;
        }
        _ => {
            let order = match ob.rotmode {
                1 => EulerOrder::Xyz,
                2 => EulerOrder::Xzy,
                3 => EulerOrder::Yxz,
                4 => EulerOrder::Yzx,
                5 => EulerOrder::Zxy,
                _ => EulerOrder::Zyx,
            };
            let e = quat_to_euler(quat, order);
            ob.rot = [e.xyz.x, e.xyz.y, e.xyz.z];
        }
    }
}

/// `ObjectTfmProtectedChannels` verbatim.
#[derive(Clone, Copy, Default)]
pub struct ObjectTfmProtectedChannels {
    /// `loc`,`dloc`,`scale`,`dscale`,`rot`,`drot`.
    pub loc: [f32; 3],
    /// `dloc`.
    pub dloc: [f32; 3],
    /// `scale`.
    pub scale: [f32; 3],
    /// `dscale`.
    pub dscale: [f32; 3],
    /// `rot`.
    pub rot: [f32; 3],
    /// `drot`.
    pub drot: [f32; 3],
    /// `quat`.
    pub quat: [f32; 4],
    /// `dquat`.
    pub dquat: [f32; 4],
    /// `rotAxis`.
    pub rot_axis: [f32; 3],
    /// `drotAxis`.
    pub drot_axis: [f32; 3],
    /// `rotAngle`.
    pub rot_angle: f32,
    /// `drotAngle`.
    pub drot_angle: f32,
}

/// `BKE_object_tfm_protected_backup` (verbatim).
pub fn object_tfm_protected_backup(ob: &Object) -> ObjectTfmProtectedChannels {
    ObjectTfmProtectedChannels {
        loc: ob.loc,
        dloc: ob.dloc,
        scale: ob.scale,
        dscale: ob.dscale,
        rot: ob.rot,
        drot: ob.drot,
        quat: ob.quat,
        dquat: ob.dquat,
        rot_axis: ob.rot_axis,
        drot_axis: ob.drot_axis,
        rot_angle: ob.rot_angle,
        drot_angle: ob.drot_angle,
    }
}

/// `BKE_object_tfm_protected_restore` (verbatim).
pub fn object_tfm_protected_restore(
    ob: &mut Object,
    obtfm: &ObjectTfmProtectedChannels,
    protectflag: i16,
) {
    for i in 0..3 {
        if protectflag & (OB_LOCK_LOCX << i) != 0 {
            ob.loc[i] = obtfm.loc[i];
            ob.dloc[i] = obtfm.dloc[i];
        }
        if protectflag & (OB_LOCK_SCALEX << i) != 0 {
            ob.scale[i] = obtfm.scale[i];
            ob.dscale[i] = obtfm.dscale[i];
        }
        if protectflag & (OB_LOCK_ROTX << i) != 0 {
            ob.rot[i] = obtfm.rot[i];
            ob.drot[i] = obtfm.drot[i];
            ob.quat[i + 1] = obtfm.quat[i + 1];
            ob.dquat[i + 1] = obtfm.dquat[i + 1];
            ob.rot_axis[i] = obtfm.rot_axis[i];
            ob.drot_axis[i] = obtfm.drot_axis[i];
        }
    }
    if (protectflag & OB_LOCK_ROT4D) != 0 && (protectflag & OB_LOCK_ROTW) != 0 {
        ob.quat[0] = obtfm.quat[0];
        ob.dquat[0] = obtfm.dquat[0];
        ob.rot_angle = obtfm.rot_angle;
        ob.drot_angle = obtfm.drot_angle;
    }
}

/// `BKE_object_tfm_copy` (verbatim).
pub fn object_tfm_copy(object_dst: &mut Object, object_src: &Object) {
    object_dst.loc = object_src.loc;
    object_dst.dloc = object_src.dloc;
    object_dst.scale = object_src.scale;
    object_dst.dscale = object_src.dscale;
    object_dst.rot = object_src.rot;
    object_dst.drot = object_src.drot;
    object_dst.quat = object_src.quat;
    object_dst.dquat = object_src.dquat;
    object_dst.rot_axis = object_src.rot_axis;
    object_dst.drot_axis = object_src.drot_axis;
    object_dst.rot_angle = object_src.rot_angle;
    object_dst.drot_angle = object_src.drot_angle;
}

/// `BKE_object_to_mat3` — `R * diag(scale)` (verbatim;
/// column-major output).
pub fn object_to_mat3(ob: &Object) -> [[f32; 3]; 3] {
    let m3 = object_rot_to_quat(ob).to_mat3();
    let mut r = [[0.0f32; 3]; 3];
    for c in 0..3 {
        let col = m3.cols[c];
        r[c] = [col.x * ob.scale[c], col.y * ob.scale[c], col.z * ob.scale[c]];
    }
    r
}

/// `BKE_object_to_mat4` — `copy_m4_m3` + `loc+dloc`
/// translation column (verbatim; column-major).
pub fn object_to_mat4(ob: &Object) -> [[f32; 4]; 4] {
    let m3 = object_to_mat3(ob);
    let mut m = [[0.0f32; 4]; 4];
    for c in 0..3 {
        for r in 0..3 {
            m[c][r] = m3[c][r];
        }
    }
    m[3][0] = ob.loc[0] + ob.dloc[0];
    m[3][1] = ob.loc[1] + ob.dloc[1];
    m[3][2] = ob.loc[2] + ob.dloc[2];
    m[3][3] = 1.0;
    m
}
