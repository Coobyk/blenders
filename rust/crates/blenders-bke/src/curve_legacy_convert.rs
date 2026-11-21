//! Port of `intern/curve_legacy_convert.cc` —
//! `bke::curve_legacy_to_curves` (legacy `Curve`/`Nurb` → new `Curves`).
//!
//! The attribute/geometry machinery becomes a plain output struct —
//! every field the C fills is present (`positions`, `offsets`,
//! `curve_types`, `cyclic`, `radii`, `tilts`, handle + nurbs arrays,
//! `normal_mode`, `mat`, `totcol`). `curves_new_nomain` / attribute
//! accessor plumbing is PORT_PENDING.

use blenders_dna::curve_enums::*;
use blenders_dna::curve_types::{Curve, Nurb};
use blenders_dna::curves_types::*;
use blenders_dna::vec::Vec3;

/// `curve_type_from_legacy`.
fn curve_type_from_legacy(type_: NurbType) -> CurveType {
    match type_ {
        CU_POLY => CURVE_TYPE_POLY,
        CU_BEZIER => CURVE_TYPE_BEZIER,
        CU_NURBS => CURVE_TYPE_NURBS,
        _ => unreachable!("unknown legacy curve type"),
    }
}

/// `handle_type_from_legacy`.
fn handle_type_from_legacy(handle_type_legacy: u8) -> HandleType {
    match handle_type_legacy {
        x if x == HD_FREE => BEZIER_HANDLE_FREE,
        x if x == HD_AUTO => BEZIER_HANDLE_AUTO,
        x if x == HD_VECT => BEZIER_HANDLE_VECTOR,
        x if x == HD_ALIGN => BEZIER_HANDLE_ALIGN,
        x if x == HD_AUTO_ANIM => BEZIER_HANDLE_AUTO,
        x if x == HD_ALIGN_DOUBLESIDE => BEZIER_HANDLE_ALIGN,
        _ => unreachable!("unknown legacy handle type"),
    }
}

/// `normal_mode_from_legacy`.
fn normal_mode_from_legacy(twist_mode: CurveTwistMode) -> NormalMode {
    match twist_mode {
        CU_TWIST_Z_UP | CU_TWIST_TANGENT => NORMAL_MODE_Z_UP,
        CU_TWIST_MINIMUM => NORMAL_MODE_MINIMUM_TWIST,
        _ => unreachable!("unknown legacy twist mode"),
    }
}

/// `knots_mode_from_legacy`.
pub fn knots_mode_from_legacy(flag: i16) -> KnotsMode {
    if flag & CU_NURB_CUSTOM != 0 {
        return NURBS_KNOT_MODE_CUSTOM;
    }
    match flag & (CU_NURB_ENDPOINT | CU_NURB_BEZIER) {
        x if x == CU_NURB_ENDPOINT => NURBS_KNOT_MODE_ENDPOINT,
        x if x == CU_NURB_BEZIER => NURBS_KNOT_MODE_BEZIER,
        x if x == (CU_NURB_ENDPOINT | CU_NURB_BEZIER) => NURBS_KNOT_MODE_ENDPOINT_BEZIER,
        0 => NURBS_KNOT_MODE_NORMAL,
        _ => unreachable!("unknown legacy knots flag"),
    }
}

/// The `Curves` data the conversion writes (the C mutates
/// `CurvesGeometry` spans; here the caller receives the filled arrays).
#[derive(Default)]
pub struct LegacyCurvesOut {
    /// `offsets_for_write` — pntsu prefix sums + total.
    pub offsets: Vec<i32>,
    /// `curve_types_for_write`.
    pub curve_types: Vec<i8>,
    /// `cyclic_for_write`.
    pub cyclic: Vec<bool>,
    /// `positions_for_write` (curve point count = last offset).
    pub positions: Vec<Vec3>,
    /// `"radius"` point attribute.
    pub radii: Vec<f32>,
    /// `tilt_for_write`.
    pub tilts: Vec<f32>,
    /// `resolution_for_write` (per curve; bezier+nurbs).
    pub resolutions: Vec<i32>,
    /// `handle_positions_left/right_for_write`.
    pub handle_positions_left: Vec<Vec3>,
    pub handle_positions_right: Vec<Vec3>,
    /// `handle_types_left/right_for_write`.
    pub handle_types_left: Vec<i8>,
    pub handle_types_right: Vec<i8>,
    /// `nurbs_weights/orders/knots_modes_for_write`.
    pub nurbs_weights: Vec<f32>,
    pub nurbs_orders: Vec<i8>,
    pub nurbs_knots_modes: Vec<i8>,
    /// `nurbs_custom_knots_for_write` — concatenated per-curve knots
    /// (only for `NURBS_KNOT_MODE_CUSTOM` curves).
    pub nurbs_custom_knots: Vec<f32>,
    /// `nurbs_custom_knots_by_curve` offsets (len = curves+1).
    pub nurbs_custom_knots_offsets: Vec<i32>,
    /// `"normal_mode"` curve attribute.
    pub normal_mode: i8,
    /// `curves_id->mat` (`MEM_dupalloc` → cloned).
    pub mat: Vec<usize>,
    /// `curves_id->totcol`.
    pub totcol: i16,
}

/// `curve_legacy_to_curves(curve_legacy, nurbs_list)` — `src_curves`
/// is the nurb arena; `nurbs_list` walks `next` handles (idx+1, `NIL`).
pub fn curve_legacy_to_curves(
    curve_legacy: &Curve,
    nurbs: &[Nurb],
    nurbs_list_head: usize,
) -> Option<LegacyCurvesOut> {
    let mut src_curves: Vec<usize> = Vec::new();
    let mut h = nurbs_list_head;
    while h != 0 && h != blenders_dna::id::NIL {
        let i = h - 1;
        src_curves.push(i);
        h = nurbs[i].next;
    }
    if src_curves.is_empty() {
        return None;
    }
    let mut out = LegacyCurvesOut::default();
    let n_curves = src_curves.len();
    /* Offsets + types + cyclic pass (`offsets[i] = offset`). */
    out.offsets.resize(n_curves + 1, 0);
    out.curve_types.resize(n_curves, CURVE_TYPE_POLY);
    out.cyclic.resize(n_curves, false);
    let mut offset = 0i32;
    for (i, &ni) in src_curves.iter().enumerate() {
        out.offsets[i] = offset;
        let src = &nurbs[ni];
        out.curve_types[i] = curve_type_from_legacy(src.type_);
        out.cyclic[i] = src.flagu & CU_NURB_CYCLIC != 0;
        offset += src.pntsu;
    }
    out.offsets[n_curves] = offset;
    let n_points = offset as usize;
    /* `curves.resize(total, curves_num)` → point arrays sized. */
    out.positions.resize(n_points, Vec3::ZERO);
    out.radii.resize(n_points, 0.0);
    out.tilts.resize(n_points, 0.0);
    out.resolutions.resize(n_curves, 0);
    /* Per-type fills (`threading::parallel_for` → serial). */
    let points_of = |i: usize| (out.offsets[i] as usize)..(out.offsets[i + 1] as usize);
    for (i, &ni) in src_curves.iter().enumerate() {
        let src = &nurbs[ni];
        let points = points_of(i);
        match out.curve_types[i] {
            x if x == CURVE_TYPE_POLY => {
                for (j, bp) in src.bp.iter().take(points.len()).enumerate() {
                    let p = points.start + j;
                    out.positions[p] = Vec3::new(bp.vec[0], bp.vec[1], bp.vec[2]);
                    out.radii[p] = bp.radius;
                    out.tilts[p] = bp.tilt;
                }
            }
            x if x == CURVE_TYPE_BEZIER => {
                if out.handle_positions_left.len() != n_points {
                    out.handle_positions_left.resize(n_points, Vec3::ZERO);
                    out.handle_positions_right.resize(n_points, Vec3::ZERO);
                    out.handle_types_left.resize(n_points, 0);
                    out.handle_types_right.resize(n_points, 0);
                }
                out.resolutions[i] = src.resolu as i32;
                for (j, bt) in src.bezt.iter().take(points.len()).enumerate() {
                    let p = points.start + j;
                    out.positions[p] = Vec3::new(bt.vec[1][0], bt.vec[1][1], bt.vec[1][2]);
                    out.handle_positions_left[p] =
                        Vec3::new(bt.vec[0][0], bt.vec[0][1], bt.vec[0][2]);
                    out.handle_types_left[p] = handle_type_from_legacy(bt.h1) as i8;
                    out.handle_positions_right[p] =
                        Vec3::new(bt.vec[2][0], bt.vec[2][1], bt.vec[2][2]);
                    out.handle_types_right[p] = handle_type_from_legacy(bt.h2) as i8;
                    out.radii[p] = bt.radius;
                    out.tilts[p] = bt.tilt;
                }
            }
            x if x == CURVE_TYPE_NURBS => {
                if out.nurbs_weights.len() != n_points {
                    out.nurbs_weights.resize(n_points, 0.0);
                }
                out.resolutions[i] = src.resolu as i32;
                if out.nurbs_orders.len() != n_curves {
                    out.nurbs_orders.resize(n_curves, 0);
                    out.nurbs_knots_modes.resize(n_curves, NURBS_KNOT_MODE_NORMAL);
                }
                out.nurbs_orders[i] = src.orderu as i8;
                out.nurbs_knots_modes[i] = knots_mode_from_legacy(src.flagu);
                for (j, bp) in src.bp.iter().take(points.len()).enumerate() {
                    let p = points.start + j;
                    out.positions[p] = Vec3::new(bp.vec[0], bp.vec[1], bp.vec[2]);
                    out.radii[p] = bp.radius;
                    out.tilts[p] = bp.tilt;
                    out.nurbs_weights[p] = bp.vec[3];
                }
            }
            _ => unreachable!("unexpected curve type"),
        }
    }
    /* `nurbs_custom_knots_update_size` + copy — per-curve knot ranges
     * recorded in `nurbs_custom_knots_offsets`. */
    out.nurbs_custom_knots_offsets.resize(n_curves + 1, 0);
    let mut koff = 0i32;
    for (i, &ni) in src_curves.iter().enumerate() {
        out.nurbs_custom_knots_offsets[i] = koff;
        if out.nurbs_knots_modes.get(i) == Some(&NURBS_KNOT_MODE_CUSTOM) {
            let knots = &nurbs[ni].knotsu;
            out.nurbs_custom_knots.extend_from_slice(knots);
            koff += knots.len() as i32;
        }
    }
    out.nurbs_custom_knots_offsets[n_curves] = koff;
    out.normal_mode = normal_mode_from_legacy(curve_legacy.twist_mode);
    out.mat = curve_legacy.mat.clone();
    out.totcol = curve_legacy.totcol;
    Some(out)
}
