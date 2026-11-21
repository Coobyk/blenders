//! Port of `source/blender/blenkernel/intern/curve_bevel.cc`.
//!
//! Curve bevel/extrude display-list builders. `Curve.bevel_profile`
//! and `Curve.bevobj` object access are injected handles.

use std::f32::consts::{FRAC_PI_2, PI};

use blenders_dna::curve_enums::{CU_BACK, CU_BEV_MODE_OBJECT, CU_BEV_MODE_ROUND, CU_FRONT};
use blenders_dna::curve_types::Curve;
use blenders_dna::curveprofile_types::CurveProfile;

use crate::displist::{
    displist_addtail, DispList, DL_BACK_CURVE, DL_FRONT_CURVE, DL_POLY, DL_SEGM,
};

/// `CurveBevelFillType`.
pub type CurveBevelFillType = i32;
pub const CURVE_BEVEL_FILL_BACK: CurveBevelFillType = 0;
pub const CURVE_BEVEL_FILL_FRONT: CurveBevelFillType = 1;
pub const CURVE_BEVEL_FILL_HALF: CurveBevelFillType = 2;
pub const CURVE_BEVEL_FILL_FULL: CurveBevelFillType = 3;

/// `curve_bevel_get_fill_type`.
fn curve_bevel_get_fill_type(curve: &Curve) -> CurveBevelFillType {
    if curve.flag & (CU_FRONT | CU_BACK) == 0 {
        return CURVE_BEVEL_FILL_FULL;
    }
    if (curve.flag & CU_FRONT) != 0 && (curve.flag & CU_BACK) != 0 {
        return CURVE_BEVEL_FILL_HALF;
    }
    if (curve.flag & CU_FRONT) != 0 {
        CURVE_BEVEL_FILL_FRONT
    } else {
        CURVE_BEVEL_FILL_BACK
    }
}

/// `bevel_quarter_fill` — one bevel quadrant's (x, y) profile.
fn bevel_quarter_fill<'a>(
    curve: &Curve,
    profile_of: &'a dyn Fn(usize) -> Option<&'a CurveProfile>,
    quarter_coords_x: &mut [f32],
    quarter_coords_y: &mut [f32],
) {
    if curve.bevel_mode == CU_BEV_MODE_ROUND {
        let mut angle = 0.0f32;
        let dangle = FRAC_PI_2 / (curve.bevresol as f32 + 1.0);
        for i in 0..(curve.bevresol as usize + 1) {
            quarter_coords_x[i] = angle.cos() * curve.bevel_radius;
            quarter_coords_y[i] = angle.sin() * curve.bevel_radius;
            angle += dangle;
        }
    } else {
        /* The curve profile evaluation should be done when the
         * resolution is set. */
        let profile = profile_of(curve.bevel_profile).expect("bevel_profile");
        debug_assert!(profile.segments.len() == curve.bevresol as usize + 1);

        /* If there aren't enough samples, the curveprofile won't
         * sample the start vertex, so set it manually instead. */
        quarter_coords_x[0] = curve.bevel_radius;
        quarter_coords_y[0] = 0.0;
        for i in 1..(curve.bevresol as usize + 1) {
            quarter_coords_x[i] = profile.segments[i].x * curve.bevel_radius;
            quarter_coords_y[i] = profile.segments[i].y * curve.bevel_radius;
        }
    }
}

/// `curve_bevel_make_extrude_and_fill`.
fn curve_bevel_make_extrude_and_fill<'a>(
    cu: &Curve,
    profile_of: &'a dyn Fn(usize) -> Option<&'a CurveProfile>,
    disp: &mut Vec<DispList>,
    use_extrude: bool,
    fill_type: CurveBevelFillType,
) {
    let mut dl = DispList::default();

    /* Calculate the profile of the bevel once to reuse it for each
     * quarter. We will need to flip around the indices for every other
     * section in order to build around the circle in a consistent
     * direction.
     *
     * These should be small enough for stack allocations because the
     * current limit for `Curve.bevresol` is 32. */
    let mut quarter_coords_x = vec![0.0f32; cu.bevresol as usize + 1];
    let mut quarter_coords_y = vec![0.0f32; cu.bevresol as usize + 1];
    bevel_quarter_fill(cu, profile_of, &mut quarter_coords_x, &mut quarter_coords_y);

    let nr;
    if fill_type == CURVE_BEVEL_FILL_FULL {
        /* The full loop. */
        nr = 4 * cu.bevresol as i32 + if use_extrude { 6 } else { 4 };
        dl.flag = DL_FRONT_CURVE | DL_BACK_CURVE;
    } else if fill_type == CURVE_BEVEL_FILL_HALF {
        /* Half the loop. */
        nr = 2 * (cu.bevresol as i32 + 1) + if use_extrude { 2 } else { 1 };
        dl.flag = DL_FRONT_CURVE | DL_BACK_CURVE;
    } else {
        /* One quarter of the loop (just front or back). */
        nr = if use_extrude {
            cu.bevresol as i32 + 3
        } else {
            cu.bevresol as i32 + 2
        };
        dl.flag = if fill_type == CURVE_BEVEL_FILL_FRONT {
            DL_FRONT_CURVE
        } else {
            DL_BACK_CURVE
        };
    }

    /* Use a different type depending on whether the loop is complete
     * or not. */
    dl.type_ = if fill_type == CURVE_BEVEL_FILL_FULL {
        DL_POLY
    } else {
        DL_SEGM
    };
    dl.parts = 1;
    dl.nr = nr;
    let mut verts = Vec::with_capacity(3 * nr as usize);

    let mut emit = |x: f32, y: f32, z: f32| {
        verts.extend_from_slice(&[x, y, z]);
    };

    /* Build the back section. */
    if matches!(fill_type, CURVE_BEVEL_FILL_BACK | CURVE_BEVEL_FILL_HALF | CURVE_BEVEL_FILL_FULL) {
        /* Add the bottom vertex. */
        emit(0.0, 0.0, -cu.extrude - cu.bevel_radius);
        for i in (0..=cu.bevresol as usize).rev() {
            emit(0.0, quarter_coords_x[i], -quarter_coords_y[i] - cu.extrude);
        }
    }

    /* Add the extrusion if we're only building either the back or the
     * front. */
    if use_extrude && matches!(fill_type, CURVE_BEVEL_FILL_FRONT | CURVE_BEVEL_FILL_BACK) {
        emit(
            0.0,
            cu.bevel_radius,
            if fill_type == CURVE_BEVEL_FILL_FRONT {
                -cu.extrude
            } else {
                cu.extrude
            },
        );
    }

    /* Build the front section. */
    if matches!(fill_type, CURVE_BEVEL_FILL_FRONT | CURVE_BEVEL_FILL_HALF | CURVE_BEVEL_FILL_FULL) {
        /* Don't duplicate the last back vertex. */
        let front_start = if !use_extrude
            && matches!(fill_type, CURVE_BEVEL_FILL_HALF | CURVE_BEVEL_FILL_FULL)
        {
            1
        } else {
            0
        };
        for i in front_start..(cu.bevresol as usize + 1) {
            emit(0.0, quarter_coords_x[i], quarter_coords_y[i] + cu.extrude);
        }
        /* Add the top vertex. */
        emit(0.0, 0.0, cu.extrude + cu.bevel_radius);
    }

    /* Build the other half only if we're building the full loop. */
    if fill_type == CURVE_BEVEL_FILL_FULL {
        for i in (1..=cu.bevresol as usize).rev() {
            emit(0.0, -quarter_coords_x[i], quarter_coords_y[i] + cu.extrude);
        }

        if use_extrude {
            /* Add the extrusion. */
            emit(0.0, -cu.bevel_radius, cu.extrude);
        }

        for i in 0..(cu.bevresol as usize + 1) {
            emit(0.0, -quarter_coords_x[i], -quarter_coords_y[i] - cu.extrude);
        }
    }

    debug_assert_eq!(verts.len(), 3 * nr as usize);
    dl.verts = verts;
    displist_addtail(disp, dl);
}

/// `curve_bevel_make_full_circle`.
fn curve_bevel_make_full_circle(cu: &Curve, disp: &mut Vec<DispList>) {
    let nr = 4 + 2 * cu.bevresol as i32;

    let mut dl = DispList {
        type_: DL_POLY,
        parts: 1,
        flag: DL_BACK_CURVE,
        nr,
        ..Default::default()
    };

    let mut verts = Vec::with_capacity(3 * nr as usize);
    let dangle = 2.0 * PI / nr as f32;
    let mut angle = -(nr as f32 - 1.0) * dangle;
    for _ in 0..nr {
        verts.extend_from_slice(&[
            0.0,
            angle.cos() * cu.bevel_radius,
            angle.sin() * cu.bevel_radius - cu.extrude,
        ]);
        angle += dangle;
    }
    dl.verts = verts;
    displist_addtail(disp, dl);
}

/// `curve_bevel_make_only_extrude`.
fn curve_bevel_make_only_extrude(cu: &Curve, disp: &mut Vec<DispList>) {
    let mut dl = DispList {
        type_: DL_SEGM,
        parts: 1,
        flag: DL_FRONT_CURVE | DL_BACK_CURVE,
        nr: 2,
        ..Default::default()
    };
    dl.verts = vec![0.0, 0.0, -cu.extrude, 0.0, 0.0, cu.extrude];
    displist_addtail(disp, dl);
}

/// `curve_bevel_make_from_object` — copy a bevel-object's own display
/// list, scaled by `bevobj.scale` (`x↔y` swap is verbatim).
///
/// The `bevobj` (Object) + its curve data + curve-cache display list
/// are all injected (`PORT_PENDING` until `Object`/curve_cache are
/// fully resident).
pub fn curve_bevel_make_from_object<'a>(
    cu: &Curve,
    disp: &mut Vec<DispList>,
    bevobj_info: &dyn Fn(usize) -> Option<(i16, usize)>,
    bevobj_curve: &'a dyn Fn(usize) -> Option<&'a Curve>,
    bevobj_disp: &dyn Fn(usize) -> Option<Vec<DispList>>,
    bevobj_scale: &dyn Fn(usize) -> [f32; 3],
) {
    if cu.bevobj == blenders_dna::id::NIL {
        return;
    }
    /* `bevobj->type != OB_CURVES_LEGACY` → skip (OB_CURVES_LEGACY
     * resolved by the injected `bevobj_info` returning
     * `(is_legacy_curve, bevcu_handle)`). */
    let Some((_, bevcu_h)) = bevobj_info(cu.bevobj) else {
        return;
    };
    let Some(bevcu) = bevobj_curve(bevcu_h) else {
        return;
    };
    if bevcu.extrude == 0.0 && bevcu.bevel_radius == 0.0 {
        let facx = bevobj_scale(cu.bevobj)[0];
        let facy = bevobj_scale(cu.bevobj)[1];
        if let Some(src) = bevobj_disp(cu.bevobj) {
            for dl in src {
                if !matches!(dl.type_, DL_POLY | DL_SEGM) {
                    continue;
                }
                let mut dlnew = dl.clone();
                if dlnew.type_ == DL_SEGM {
                    dlnew.flag |= DL_FRONT_CURVE | DL_BACK_CURVE;
                }
                let nr = (dlnew.parts * dlnew.nr) as usize;
                for i in 0..nr {
                    let (x, y) = (dlnew.verts[3 * i], dlnew.verts[3 * i + 1]);
                    dlnew.verts[3 * i + 2] = y * facy;
                    dlnew.verts[3 * i + 1] = -x * facx;
                    dlnew.verts[3 * i] = 0.0;
                }
                displist_addtail(disp, dlnew);
            }
        }
    }
}

/// `BKE_curve_bevel_make` — the bevel display list for `curve`.
pub fn bke_curve_bevel_make<'a>(
    curve: &Curve,
    profile_of: &'a dyn Fn(usize) -> Option<&'a CurveProfile>,
) -> Vec<DispList> {
    let mut bevel_shape: Vec<DispList> = Vec::new();

    if curve.bevel_mode == CU_BEV_MODE_OBJECT {
        /* `curve_bevel_make_from_object` needs Object-graph access —
         * callers wire it via `curve_bevel_make_from_object`
         * (`PORT_PENDING`). */
    } else {
        let use_extrude = curve.extrude != 0.0;
        let use_bevel = curve.bevel_radius != 0.0;
        /* Pass. */
        if use_extrude && !use_bevel {
            curve_bevel_make_only_extrude(curve, &mut bevel_shape);
        } else if use_extrude || use_bevel {
            let fill_type = curve_bevel_get_fill_type(curve);

            if !use_extrude && fill_type == CURVE_BEVEL_FILL_FULL
                && curve.bevel_mode == CU_BEV_MODE_ROUND
            {
                curve_bevel_make_full_circle(curve, &mut bevel_shape);
            } else {
                /* The general case for nonzero extrusion or an
                 * incomplete loop. */
                curve_bevel_make_extrude_and_fill(
                    curve,
                    profile_of,
                    &mut bevel_shape,
                    use_extrude,
                    fill_type,
                );
            }
        }
    }

    bevel_shape
}
