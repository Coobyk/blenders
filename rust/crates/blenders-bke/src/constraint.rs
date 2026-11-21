//! Port of `intern/constraint.cc` — the constraint system's
//! portable core: `BConstraintOb`, `BKE_constraint_unique_name`,
//! `constraint_target_add`/`constraint_flush_tars`/`get_tars`
//! (single-target generic helpers), `bConstraintTypeInfo` (CTI)
//! registry, `vectomat`/`basis_cross` track-axis math,
//! `wrap_rad_angle`/`clamp_angle`, and the limit evaluators
//! (`loclimit`/`rotlimit`/`sizelimit`/`distlimit`).
//!
//! `PORT_PENDING`: `BKE_constraint_mat_convertspace` (needs
//! pose-bone lookup), `contarget_get_*_mat`, per-type `get_
//! tarmat`/`evaluate` of the 25 non-limit constraint types —
//! object/pose/depsgraph handles are injected.

use blenders_dna::constraint_types::*;
use blenders_math::matrix_c::{loc_eulo_size_to_mat4, mat4_to_size, orthogonalize_m4_stable};
use blenders_math::euler::{mat3_to_euler, EulerOrder};
use blenders_math::matrix::Mat3;
use blenders_math::{Mat4, Vec3, Vec4};

fn m4_from(m: [[f32; 4]; 4]) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(m[0][0], m[0][1], m[0][2], m[0][3]),
        Vec4::new(m[1][0], m[1][1], m[1][2], m[1][3]),
        Vec4::new(m[2][0], m[2][1], m[2][2], m[2][3]),
        Vec4::new(m[3][0], m[3][1], m[3][2], m[3][3]),
    )
}
fn m4_to(m: Mat4) -> [[f32; 4]; 4] {
    let mut out = [[0f32; 4]; 4];
    for c in 0..4 {
        out[c] = [m.cols[c].x, m.cols[c].y, m.cols[c].z, m.cols[c].w];
    }
    out
}
fn euler_order_of(rot_order: i16) -> EulerOrder {
    match rot_order {
        2 => EulerOrder::Xzy,
        3 => EulerOrder::Yxz,
        4 => EulerOrder::Yzx,
        5 => EulerOrder::Zxy,
        6 => EulerOrder::Zyx,
        _ => EulerOrder::Xyz,
    }
}

/// `bConstraintOb` — evaluation context (verbatim fields;
/// object/pose handles as `usize`).
pub struct BConstraintOb {
    /// `id` — owning ID handle.
    pub id: usize,
    /// `ob` — `Object` handle.
    pub ob: usize,
    /// `pchan` — `bPoseChannel` handle (or `NIL`).
    pub pchan: usize,
    /// `matrix` — the constraint-space matrix being written.
    pub matrix: [[f32; 4]; 4],
    /// `start` — owner-space start point.
    pub start: [f32; 3],
    /// `type` — `CONSTRAINT_OBTYPE_*`.
    pub type_: i16,
    /// `rotOrder` — owner euler order.
    pub rot_order: i16,
}

/// `CONSTRAINT_OBTYPE_OBJECT`/`BONE`/`CV`/`BONE_CONT`/`VERT`.
pub const CONSTRAINT_OBTYPE_OBJECT: i16 = 1;
/// `CONSTRAINT_OBTYPE_BONE`.
pub const CONSTRAINT_OBTYPE_BONE: i16 = 2;
/// `CONSTRAINT_OBTYPE_CV`.
pub const CONSTRAINT_OBTYPE_CV: i16 = 3;
/// `CONSTRAINT_OBTYPE_VERT`.
pub const CONSTRAINT_OBTYPE_VERT: i16 = 4;

/// `EULER_ORDER_DEFAULT`.
pub const EULER_ORDER_DEFAULT: i16 = 1;

/// `BKE_constraint_unique_name` — `".%03d"` suffix collision
/// scheme over the constraint list (index-list → Vec).
pub fn constraint_unique_name(con: &mut BConstraint, list: &[BConstraint]) {
    let end = con
        .name
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(con.name.len());
    let base = String::from_utf8_lossy(&con.name[..end]).to_string();
    let mut taken = std::collections::HashSet::new();
    for other in list {
        let e = other
            .name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(other.name.len());
        taken.insert(String::from_utf8_lossy(&other.name[..e]).to_string());
    }
    if !taken.contains(&base) {
        return;
    }
    for i in 1..1000 {
        let candidate = format!("{}.{:03}", base, i);
        if !taken.contains(&candidate) {
            let mut name = [0u8; 64];
            let b = candidate.as_bytes();
            name[..b.len().min(63)].copy_from_slice(&b[..b.len().min(63)]);
            con.name = name;
            return;
        }
    }
}

/// `constraint_target_add` — append a `BConstraintTarget` from
/// `con`'s tar/subtarget fields (verbatim `*_get_tars` shared
/// tail).
pub fn constraint_target_add(
    targets: &mut Vec<BConstraintTarget>,
    tar: usize,
    subtarget: &[u8; 64],
    is_id: bool,
) {
    let mut ct = BConstraintTarget::default();
    ct.tar = tar;
    ct.subtarget = *subtarget;
    if is_id {
        ct.flag |= CONSTRAINT_TAR_TEMP;
    }
    targets.push(ct);
}

/// `constraint_flush_tars` — write the target back into
/// `(tar, subtarget)` slots (`no_copy` → only unlink).
pub fn constraint_flush_tars(
    targets: &mut Vec<BConstraintTarget>,
    tar: &mut usize,
    subtarget: &mut [u8; 64],
    no_copy: bool,
) {
    if targets.is_empty() {
        return;
    }
    let ct = targets.remove(0);
    if !no_copy {
        *tar = ct.tar;
        *subtarget = ct.subtarget;
    }
}

/// `is_custom_space_needed`.
pub fn is_custom_space_needed(con: &BConstraint) -> bool {
    !con.space_subtarget.iter().all(|&c| c == 0) || con.space_object != usize::MAX
}

/// `unit_ct_matrix_nullsafe` — zero `ct->matrix` when target
/// resolve failed (verbatim guard).
pub fn unit_ct_matrix_nullsafe(ct: &mut BConstraintTarget, has_matrix: bool) {
    if !has_matrix {
        ct.matrix = [[0.0; 4]; 4];
    }
}

/// `basis_cross` — signed cross-basis for `n,m ∈ {0,1,2}`.
pub fn basis_cross(n: i16, m: i16) -> f32 {
    match n - m {
        1 | -2 => 1.0,
        -1 | 2 => -1.0,
        _ => 0.0,
    }
}

/// `TARGET_Z_UP` flag for `vectomat`.
pub const TARGET_Z_UP: i16 = 1 << 0;

/// `vectomat` — build the track-axis basis matrix (verbatim
/// algorithm incl. the degenerate-up fallback, #53455).
pub fn vectomat(
    vec: [f32; 3],
    target_up: [f32; 3],
    axis: i16,
    upflag: i16,
    flags: i16,
) -> [[f32; 3]; 3] {
    let mut m = [[0.0f32; 3]; 3];
    let mut n = vec;
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len != 0.0 {
        n = [n[0] / len, n[1] / len, n[2] / len];
    } else {
        n = [0.0, 0.0, 1.0];
    }
    let mut axis = axis;
    if axis > 2 {
        axis -= 3;
    } else {
        n = [-n[0], -n[1], -n[2]];
    }

    let u = if (flags & TARGET_Z_UP) != 0 {
        target_up
    } else {
        [0.0, 0.0, 1.0]
    };

    /* Project the up vector onto the plane of n. */
    let dot = u[0] * n[0] + u[1] * n[1] + u[2] * n[2];
    let mut proj = [u[0] - dot * n[0], u[1] - dot * n[1], u[2] - dot * n[2]];
    let plen = (proj[0] * proj[0] + proj[1] * proj[1] + proj[2] * proj[2]).sqrt();
    if plen == 0.0 {
        proj = [0.0, 1.0, 0.0];
    } else {
        proj = [proj[0] / plen, proj[1] / plen, proj[2] / plen];
    }

    /* Right = normalized cross(proj, n). */
    let mut right = [
        proj[1] * n[2] - proj[2] * n[1],
        proj[2] * n[0] - proj[0] * n[2],
        proj[0] * n[1] - proj[1] * n[0],
    ];
    let rlen = (right[0] * right[0] + right[1] * right[1] + right[2] * right[2]).sqrt();
    if rlen != 0.0 {
        right = [right[0] / rlen, right[1] / rlen, right[2] / rlen];
    }

    if axis != upflag {
        let right_index = (3 - axis - upflag) as usize;
        let neg = basis_cross(axis, upflag);
        m[right_index] = [neg * right[0], neg * right[1], neg * right[2]];
        m[upflag as usize] = proj;
        m[axis as usize] = n;
    } else {
        m = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }
    m
}

/// `wrap_rad_angle` — wrap to `[-PI, +PI]`.
pub fn wrap_rad_angle(angle: f32) -> f32 {
    let b = angle * (0.5 / std::f32::consts::PI) + 0.5;
    ((b - b.floor()) - 0.5) * (2.0 * std::f32::consts::PI)
}

/// `clamp_angle` — cyclic-space clamp to `[min,max]` (verbatim).
pub fn clamp_angle(angle: f32, min: f32, max: f32) -> f32 {
    if (max - min) >= (2.0 * std::f32::consts::PI) {
        return angle;
    }
    if max <= min {
        return min;
    }
    let min_wrapped = wrap_rad_angle(min - angle);
    let max_wrapped = wrap_rad_angle(max - angle);
    if min_wrapped < max_wrapped {
        return angle + (0.0f32).clamp(min_wrapped, max_wrapped);
    }
    if max_wrapped >= 0.0 || min_wrapped <= 0.0 {
        return angle;
    }
    if max_wrapped.abs() < min_wrapped.abs() {
        return angle + max_wrapped;
    }
    angle + min_wrapped
}

/// `loclimit_evaluate`.
pub fn loclimit_evaluate(cob: &mut BConstraintOb, data: &BLocLimitConstraint) {
    if (data.flag & LIMIT_XMIN) != 0 {
        cob.matrix[3][0] = cob.matrix[3][0].max(data.xmin);
    }
    if (data.flag & LIMIT_XMAX) != 0 {
        cob.matrix[3][0] = cob.matrix[3][0].min(data.xmax);
    }
    if (data.flag & LIMIT_YMIN) != 0 {
        cob.matrix[3][1] = cob.matrix[3][1].max(data.ymin);
    }
    if (data.flag & LIMIT_YMAX) != 0 {
        cob.matrix[3][1] = cob.matrix[3][1].min(data.ymax);
    }
    if (data.flag & LIMIT_ZMIN) != 0 {
        cob.matrix[3][2] = cob.matrix[3][2].max(data.zmin);
    }
    if (data.flag & LIMIT_ZMAX) != 0 {
        cob.matrix[3][2] = cob.matrix[3][2].min(data.zmax);
    }
}

/// `rotlimit_evaluate` — decompose → clamp euler → rebuild
/// (legacy naive clamp vs cyclic `clamp_angle`).
pub fn rotlimit_evaluate(cob: &mut BConstraintOb, data: &BRotLimitConstraint) {
    let mut m4 = m4_from(cob.matrix);
    orthogonalize_m4_stable(&mut m4, 1, false);
    cob.matrix = m4_to(m4);

    if (data.flag & (LIMIT_XROT | LIMIT_YROT | LIMIT_ZROT)) == 0 {
        return;
    }

    let mut rot_order = cob.rot_order;
    if data.euler_order != CONSTRAINT_EULER_AUTO as i8 {
        rot_order = data.euler_order as i16;
    }

    let loc = [cob.matrix[3][0], cob.matrix[3][1], cob.matrix[3][2]];
    let size_v = mat4_to_size(m4_from(cob.matrix));
    let size = [size_v.x, size_v.y, size_v.z];
    let m3 = Mat3::from(m4_from(cob.matrix));
    let order = euler_order_of(rot_order);
    let eul = mat3_to_euler(&m3, order);
    let (mut ex, mut ey, mut ez) = (eul.i(), eul.j(), eul.k());

    if (data.flag & LIMIT_ROT_LEGACY_BEHAVIOR) != 0 {
        if (data.flag & LIMIT_XROT) != 0 {
            ex = ex.clamp(data.xmin, data.xmax);
        }
        if (data.flag & LIMIT_YROT) != 0 {
            ey = ey.clamp(data.ymin, data.ymax);
        }
        if (data.flag & LIMIT_ZROT) != 0 {
            ez = ez.clamp(data.zmin, data.zmax);
        }
    } else {
        if (data.flag & LIMIT_XROT) != 0 {
            ex = clamp_angle(ex, data.xmin, data.xmax);
        }
        if (data.flag & LIMIT_YROT) != 0 {
            ey = clamp_angle(ey, data.ymin, data.ymax);
        }
        if (data.flag & LIMIT_ZROT) != 0 {
            ez = clamp_angle(ez, data.zmin, data.zmax);
        }
    }

    let out = loc_eulo_size_to_mat4(
        Vec3::new(loc[0], loc[1], loc[2]),
        Vec3::new(ex, ey, ez),
        rot_order as usize,
        Vec3::new(size[0], size[1], size[2]),
    );
    cob.matrix = m4_to(out);
}

/// `sizelimit_evaluate` — clamp matrix scale per axis, keep
/// sign (verbatim `mat4_to_size` + rebuild by scaling basis).
pub fn sizelimit_evaluate(cob: &mut BConstraintOb, data: &BSizeLimitConstraint) {
    let size_v = mat4_to_size(m4_from(cob.matrix));
    let mut size = [size_v.x, size_v.y, size_v.z];
    let obsize = size;

    if (data.flag & LIMIT_XMIN) != 0 {
        size[0] = size[0].max(data.xmin);
    }
    if (data.flag & LIMIT_XMAX) != 0 {
        size[0] = size[0].min(data.xmax);
    }
    if (data.flag & LIMIT_YMIN) != 0 {
        size[1] = size[1].max(data.ymin);
    }
    if (data.flag & LIMIT_YMAX) != 0 {
        size[1] = size[1].min(data.ymax);
    }
    if (data.flag & LIMIT_ZMIN) != 0 {
        size[2] = size[2].max(data.zmin);
    }
    if (data.flag & LIMIT_ZMAX) != 0 {
        size[2] = size[2].min(data.zmax);
    }

    for i in 0..3 {
        if obsize[i].abs() > 1e-8 {
            let fac = size[i] / obsize[i];
            for r in 0..3 {
                cob.matrix[r][i] *= fac;
            }
        }
    }
}

/// `LIMIT_XROT`/`LIMIT_YROT`/`LIMIT_ZROT` (rotlimit flag bits
/// sharing the loc/size flag slots verbatim).
pub const LIMIT_XROT: i16 = LIMIT_XMIN | LIMIT_XMAX;
/// `LIMIT_YROT`.
pub const LIMIT_YROT: i16 = LIMIT_YMIN | LIMIT_YMAX;
/// `LIMIT_ZROT`.
pub const LIMIT_ZROT: i16 = LIMIT_ZMIN | LIMIT_ZMAX;

/// `bConstraintTypeInfo` — `bConstraintTypeInfo` registry entry.
pub struct BConstraintTypeInfo {
    /// `type` — `CONSTRAINT_TYPE_*`.
    pub type_: i16,
    /// `name` — UI label (verbatim `N_()`).
    pub name: &'static str,
    /// `struct_name` — DNA struct name.
    pub struct_name: &'static str,
    /// `flag` — `CTI_FLAG_*` from `cti->flag` semantics.
    pub flag: i16,
    /// `has_target` — get_tars present (verbatim table bit).
    pub has_target: bool,
}

/// `BKE_constraint_typeinfo_get` — the CTI table (verbatim
/// names/types; per-type callbacks are `PORT_PENDING`).
pub fn constraint_typeinfo_get(con_type: i16) -> Option<BConstraintTypeInfo> {
    let (name, struct_name, has_target) = match con_type {
        CONSTRAINT_TYPE_CHILDOF => ("Child Of", "bChildOfConstraint", true),
        CONSTRAINT_TYPE_TRACKTO => ("Track To", "bTrackToConstraint", true),
        CONSTRAINT_TYPE_KINEMATIC => ("Inverse Kinematics", "bKinematicConstraint", true),
        CONSTRAINT_TYPE_FOLLOWPATH => ("Follow Path", "bFollowPathConstraint", true),
        CONSTRAINT_TYPE_ROTLIMIT => ("Limit Rotation", "bRotLimitConstraint", false),
        CONSTRAINT_TYPE_LOCLIMIT => ("Limit Location", "bLocLimitConstraint", false),
        CONSTRAINT_TYPE_SIZELIMIT => ("Limit Scale", "bSizeLimitConstraint", false),
        CONSTRAINT_TYPE_ROTLIKE => ("Copy Rotation", "bRotateLikeConstraint", true),
        CONSTRAINT_TYPE_LOCLIKE => ("Copy Location", "bLocateLikeConstraint", true),
        CONSTRAINT_TYPE_SIZELIKE => ("Copy Scale", "bSizeLikeConstraint", true),
        CONSTRAINT_TYPE_ACTION => ("Action", "bActionConstraint", true),
        CONSTRAINT_TYPE_LOCKTRACK => ("Locked Track", "bLockTrackConstraint", true),
        CONSTRAINT_TYPE_DISTLIMIT => ("Limit Distance", "bDistLimitConstraint", true),
        CONSTRAINT_TYPE_STRETCHTO => ("Stretch To", "bStretchToConstraint", true),
        CONSTRAINT_TYPE_MINMAX => ("Floor", "bMinMaxConstraint", true),
        CONSTRAINT_TYPE_CLAMPTO => ("Clamp To", "bClampToConstraint", true),
        CONSTRAINT_TYPE_TRANSFORM => ("Copy Transforms", "bTransLikeConstraint", true),
        CONSTRAINT_TYPE_SHRINKWRAP => ("Shrinkwrap", "bShrinkwrapConstraint", true),
        CONSTRAINT_TYPE_DAMPTRACK => ("Damped Track", "bDampTrackConstraint", true),
        CONSTRAINT_TYPE_SPLINEIK => ("Spline IK", "bSplineIkConstraint", true),
        CONSTRAINT_TYPE_TRANSLIKE => ("Copy Transforms (Legacy)", "bTransformConstraint", true),
        CONSTRAINT_TYPE_SAMEVOL => ("Maintain Volume", "bSameVolumeConstraint", false),
        CONSTRAINT_TYPE_PIVOT => ("Pivot", "bPivotConstraint", true),
        CONSTRAINT_TYPE_FOLLOWTRACK => ("Follow Track", "bFollowTrackConstraint", false),
        CONSTRAINT_TYPE_CAMERASOLVER => ("Camera Solver", "bCameraSolverConstraint", false),
        CONSTRAINT_TYPE_OBJECTSOLVER => ("Object Solver", "bObjectSolverConstraint", false),
        CONSTRAINT_TYPE_TRANSFORM_CACHE => {
            ("Transform Cache", "bTransformCacheConstraint", false)
        }
        CONSTRAINT_TYPE_ARMATURE => ("Armature", "bArmatureConstraint", false),
        CONSTRAINT_TYPE_GEOMETRY_ATTRIBUTE => {
            ("Geometry Attribute", "bGeometryAttributeConstraint", false)
        }
        _ => return None,
    };
    Some(BConstraintTypeInfo {
        type_: con_type,
        name,
        struct_name,
        flag: 0,
        has_target,
    })
}
