//! Port of `intern/curve_deform.cc` — deform coords by a curve
//! object (used by the Curve modifier).
//!
//! `BKE_where_on_path`/`BKE_anim_path_get_length` are injected; the
//! BMesh `dvert` fetch and `BKE_defvert_find_weight` collapse into
//! the `vert_weight` callback (None when no weights are used).

use blenders_math::matrix::{Mat3, Mat4};
use blenders_math::quaternion as qt;
use blenders_math::quaternion::Quaternion;
use blenders_math::vector::Vec3;

/// `MOD_CURVE_INVERT_VGROUP`.
pub const MOD_CURVE_INVERT_VGROUP: i16 = 1 << 2;

/// `Curve.flag` bits used here.
pub const CU_STRETCH: i32 = 1 << 0;
pub const CU_PATH_RADIUS: i32 = 1 << 1;
pub const CU_DEFORM_BOUNDS_OFF: i32 = 1 << 2;

const FLT_EPSILON: f32 = 1.19209290e-7;

/// `CurveDeform` — local-space transform state.
pub struct CurveDeform {
    pub dmin: Vec3,
    pub dmax: Vec3,
    pub curvespace: Mat4,
    pub objectspace: Mat4,
    pub objectspace3: Mat3,
    pub no_rot_axis: i32,
}

/// `init_curve_deform`.
fn init_curve_deform(
    ob_curve_to_world: Mat4,
    ob_target_to_world: Mat4,
) -> CurveDeform {
    let imat = ob_target_to_world.inverted().unwrap_or(Mat4::IDENTITY);
    let objectspace = imat * ob_curve_to_world;
    let curvespace = objectspace.inverted().unwrap_or(Mat4::IDENTITY);
    CurveDeform {
        dmin: Vec3::ZERO,
        dmax: Vec3::ZERO,
        objectspace,
        curvespace,
        objectspace3: Mat3::from(objectspace),
        no_rot_axis: 0,
    }
}

/// `BKE_where_on_path` result.
pub struct WhereOnPath {
    /// `loc[4]` — curve-space point (w ignored).
    pub loc: Vec3,
    /// `dir[3]` — tangent.
    pub dir: Vec3,
    /// `new_quat[4]`.
    pub quat: Quaternion,
    /// `radius`.
    pub radius: f32,
}

/// `calc_curve_deform` — rotate & translate `co` to the curve.
///
/// `cu_flag` = `Curve.flag`; `anim_path_len` returns -1 when the
/// anim path is missing (curve cache absent / cyclic deps).
#[allow(clippy::too_many_arguments)]
fn calc_curve_deform(
    cu_flag: i32,
    co: &mut Vec3,
    axis: u8,
    cd: &CurveDeform,
    anim_path_len: &dyn Fn() -> f32,
    where_on_path: &mut dyn FnMut(f32) -> Option<WhereOnPath>,
    r_quat: Option<&mut Quaternion>,
) -> bool {
    let mut fac;
    let index: usize;
    let is_neg_axis = axis > 2;

    /* options */
    if is_neg_axis {
        index = (axis - 3) as usize;
        if (cu_flag & CU_STRETCH) != 0 {
            let divisor = cd.dmax[index] - cd.dmin[index];
            fac = if divisor > FLT_EPSILON {
                -(co[index] - cd.dmax[index]) / divisor
            } else {
                0.0
            };
        } else {
            let totdist = anim_path_len();
            fac = if totdist > FLT_EPSILON {
                -(co[index] - cd.dmax[index]) / totdist
            } else {
                0.0
            };
        }
    } else {
        index = axis as usize;
        if (cu_flag & CU_STRETCH) != 0 {
            let divisor = cd.dmax[index] - cd.dmin[index];
            fac = if divisor > FLT_EPSILON {
                (co[index] - cd.dmin[index]) / divisor
            } else {
                0.0
            };
        } else {
            let totdist = anim_path_len();
            fac = if totdist > FLT_EPSILON {
                (co[index] - cd.dmin[index]) / totdist
            } else {
                0.0
            };
        }
    }

    let _ = &mut fac;

    let Some(wop) = where_on_path(fac) else {
        return false;
    };
    let WhereOnPath {
        loc,
        mut dir,
        quat: mut new_quat,
        radius,
    } = wop;

    if cd.no_rot_axis != 0 {
        /* This is not exactly the same as 2.4x, since the axis is
         * having rotation removed rather than changing the axis
         * before calculating the tilt but serves much the same
         * purpose. */
        let mut dir_flat = dir;
        dir_flat[cd.no_rot_axis as usize - 1] = 0.0;

        dir = dir.normalized();
        dir_flat = dir_flat.normalized();

        let q = qt::rotation_between_vecs_to_quat(dir, dir_flat);
        /* Could this be done faster? */
        new_quat = q * new_quat;
    }

    /* Logic for 'cent' orientation (see C comment block). */
    let mut quat = new_quat;
    let mut cent = *co;

    /* Zero the axis which is not used. */
    quat = qt::quat_apply_track(
        quat,
        axis,
        if axis == 0 || axis == 2 { 1 } else { 0 },
    );
    cent = qt::vec_apply_track(cent, axis);
    cent[index] = 0.0;

    /* scale if enabled */
    if (cu_flag & CU_PATH_RADIUS) != 0 {
        cent = cent * radius;
    }

    /* local rotation */
    let quat_n = quat.normalized();
    cent = quat_n.rotate_vec3(cent);

    /* translation */
    *co = cent + loc;

    if let Some(r) = r_quat {
        *r = quat;
    }
    true
}

/// `curve_deform_coords_impl` — full API.
///
/// `vert_weight` = `BKE_defvert_find_weight(dvert, defgrp_index)`
/// per vertex index (None → `use_dverts` false); `anim_path_len`
/// returns negative/zero when the curve cache path is missing.
#[allow(clippy::too_many_arguments)]
pub fn curve_deform_coords_impl(
    cu_flag: i32,
    ob_curve_to_world: Mat4,
    ob_target_to_world: Mat4,
    vert_coords: &mut [Vec3],
    vert_weight: Option<&dyn Fn(usize) -> f32>,
    flag: i16,
    defaxis: u8,
    track_axis: u8,
    anim_path_len: &dyn Fn() -> f32,
    where_on_path: &mut dyn FnMut(f32) -> Option<WhereOnPath>,
) {
    debug_assert!(defaxis < 6);
    let _ = track_axis;
    let is_neg_axis = defaxis > 2;
    let invert_vgroup = (flag & MOD_CURVE_INVERT_VGROUP) != 0;
    let use_dverts = vert_weight.is_some();

    let mut cd = init_curve_deform(ob_curve_to_world, ob_target_to_world);

    if (cu_flag & CU_DEFORM_BOUNDS_OFF) != 0 {
        /* Dummy bounds. */
        if !is_neg_axis {
            cd.dmin = Vec3::ZERO;
            cd.dmax = Vec3::splat(1.0);
        } else {
            /* Negative, these bounds give a good rest position. */
            cd.dmin = Vec3::splat(-1.0);
            cd.dmax = Vec3::ZERO;
        }
    } else {
        /* Set mesh min/max bounds. */
        cd.dmin = Vec3::splat(f32::MAX);
        cd.dmax = Vec3::splat(-f32::MAX);
    }

    let weight_of = |a: usize| -> f32 {
        let w = vert_weight.unwrap()(a);
        if invert_vgroup {
            1.0 - w
        } else {
            w
        }
    };

    if use_dverts {
        if (cu_flag & CU_DEFORM_BOUNDS_OFF) != 0 {
            /* DEFORM_OP. */
            for (a, co) in vert_coords.iter_mut().enumerate() {
                let weight = weight_of(a);
                if weight > 0.0 {
                    *co = cd.curvespace.transform_point(*co);
                    let mut vec = *co;
                    calc_curve_deform(
                        cu_flag,
                        &mut vec,
                        defaxis,
                        &cd,
                        anim_path_len,
                        where_on_path,
                        None,
                    );
                    *co = *co + (vec - *co) * weight;
                    *co = cd.objectspace.transform_point(*co);
                }
            }
        } else {
            /* DEFORM_OP_MINMAX. */
            for (a, co) in vert_coords.iter_mut().enumerate() {
                let weight = weight_of(a);
                if weight > 0.0 {
                    *co = cd.curvespace.transform_point(*co);
                    cd.dmin = cd.dmin.min(*co);
                    cd.dmax = cd.dmax.max(*co);
                }
            }
            /* DEFORM_OP_CLAMPED — already in curvespace. */
            for (a, co) in vert_coords.iter_mut().enumerate() {
                let weight = weight_of(a);
                if weight > 0.0 {
                    let mut vec = *co;
                    calc_curve_deform(
                        cu_flag,
                        &mut vec,
                        defaxis,
                        &cd,
                        anim_path_len,
                        where_on_path,
                        None,
                    );
                    *co = *co + (vec - *co) * weight;
                    *co = cd.objectspace.transform_point(*co);
                }
            }
        }
    } else {
        if (cu_flag & CU_DEFORM_BOUNDS_OFF) != 0 {
            for co in vert_coords.iter_mut() {
                *co = cd.curvespace.transform_point(*co);
                calc_curve_deform(
                    cu_flag,
                    co,
                    defaxis,
                    &cd,
                    anim_path_len,
                    where_on_path,
                    None,
                );
                *co = cd.objectspace.transform_point(*co);
            }
        } else {
            for co in vert_coords.iter_mut() {
                *co = cd.curvespace.transform_point(*co);
                cd.dmin = cd.dmin.min(*co);
                cd.dmax = cd.dmax.max(*co);
            }
            for co in vert_coords.iter_mut() {
                /* Already in 'cd.curvespace', previous loop. */
                calc_curve_deform(
                    cu_flag,
                    co,
                    defaxis,
                    &cd,
                    anim_path_len,
                    where_on_path,
                    None,
                );
                *co = cd.objectspace.transform_point(*co);
            }
        }
    }
}

/// `BKE_curve_deform_coords`.
#[allow(clippy::too_many_arguments)]
pub fn bke_curve_deform_coords(
    cu_flag: i32,
    ob_curve_to_world: Mat4,
    ob_target_to_world: Mat4,
    vert_coords: &mut [Vec3],
    vert_weight: Option<&dyn Fn(usize) -> f32>,
    flag: i16,
    defaxis: u8,
    anim_path_len: &dyn Fn() -> f32,
    where_on_path: &mut dyn FnMut(f32) -> Option<WhereOnPath>,
) {
    curve_deform_coords_impl(
        cu_flag,
        ob_curve_to_world,
        ob_target_to_world,
        vert_coords,
        vert_weight,
        flag,
        defaxis,
        0,
        anim_path_len,
        where_on_path,
    );
}

/// `BKE_curve_deform_coords_with_editmesh` — same impl; the editmesh
/// variant differs only by the `vert_weight` source (BMesh CD layer
/// instead of a `dvert` array, resolved by the caller).
#[allow(clippy::too_many_arguments)]
pub fn bke_curve_deform_coords_with_editmesh(
    cu_flag: i32,
    ob_curve_to_world: Mat4,
    ob_target_to_world: Mat4,
    vert_coords: &mut [Vec3],
    vert_weight: Option<&dyn Fn(usize) -> f32>,
    flag: i16,
    defaxis: u8,
    anim_path_len: &dyn Fn() -> f32,
    where_on_path: &mut dyn FnMut(f32) -> Option<WhereOnPath>,
) {
    curve_deform_coords_impl(
        cu_flag,
        ob_curve_to_world,
        ob_target_to_world,
        vert_coords,
        vert_weight,
        flag,
        defaxis,
        0,
        anim_path_len,
        where_on_path,
    );
}

/// `BKE_curve_deform_co` — single-co deform with rotation output.
/// Returns `false` when `ob_curve->type != OB_CURVES_LEGACY`
/// (caller's early-out); `r_mat` is identity then.
#[allow(clippy::too_many_arguments)]
pub fn bke_curve_deform_co(
    cu_flag: i32,
    ob_curve_to_world: Mat4,
    ob_target_to_world: Mat4,
    orco: Vec3,
    vec: &mut Vec3,
    no_rot_axis: i32,
    trackflag: u8,
    anim_path_len: &dyn Fn() -> f32,
    where_on_path: &mut dyn FnMut(f32) -> Option<WhereOnPath>,
    r_mat: &mut Mat3,
) {
    let mut cd = init_curve_deform(ob_curve_to_world, ob_target_to_world);
    /* Option to only rotate for XY, for example. */
    cd.no_rot_axis = no_rot_axis;

    cd.dmin = orco;
    cd.dmax = orco;

    *vec = cd.curvespace.transform_point(*vec);

    let mut quat = Quaternion::IDENTITY;
    if calc_curve_deform(
        cu_flag,
        vec,
        trackflag,
        &cd,
        anim_path_len,
        where_on_path,
        Some(&mut quat),
    ) {
        let qmat = quat.to_mat3();
        *r_mat = qmat * cd.objectspace3;
    } else {
        *r_mat = Mat3::IDENTITY;
    }

    *vec = cd.objectspace.transform_point(*vec);
}
