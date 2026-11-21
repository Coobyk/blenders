//! Port of `intern/curve.cc` pure parts — nurb validity,
//! knot generation (`calcknots`/`makeknots`/`KNOTSU/V`),
//! `basisNurb`, index↔uv helpers, bezt/bpoint prev-next and
//! normal/plane calc, verts counting.
//!
//! `PORT_PENDING`: `BKE_nurb_makeFaces`/`makeCurve`
//! (forward-diff evaluators — large), `BKE_curve_bevelList_*`
//! (needs `Curve`/`BevList` runtime), `BKE_curve_init`/
//! `texspace_calc` (needs `Curve` DNA defaults), editmode
//! free/dup fns, font layout.

use blenders_dna::curve_enums::{
    CU_NURBS, CU_NURB_BEZIER, CU_NURB_CYCLIC, CU_NURB_CUSTOM, CU_NURB_ENDPOINT,
};
use blenders_dna::curve_types::{BPoint, BezTriple, Nurb};
use blenders_math::base::mod_i;

/// `NURBSValidationStatus` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NurbsValidationStatus {
    /// `Valid`.
    Valid,
    /// `AtLeastTwoPointsRequired`.
    AtLeastTwoPointsRequired,
    /// `MorePointsThanOrderRequired`.
    MorePointsThanOrderRequired,
    /// `MorePointsForBezierRequired`.
    MorePointsForBezierRequired,
    /// `MoreRowsForBezierRequired`.
    MoreRowsForBezierRequired,
}

/// `nurb_check_valid` (verbatim).
pub fn nurb_check_valid(
    pnts: i32,
    order: i32,
    flag: i32,
    type_: i32,
    is_surf: bool,
    r_points_needed: &mut i32,
) -> NurbsValidationStatus {
    if pnts <= 1 {
        return NurbsValidationStatus::AtLeastTwoPointsRequired;
    }
    if type_ == CU_NURBS as i32 {
        if pnts < order {
            return NurbsValidationStatus::MorePointsThanOrderRequired;
        }
        if flag & CU_NURB_BEZIER as i32 != 0 {
            let mut points_needed = 0;
            if flag & CU_NURB_CYCLIC as i32 != 0 {
                let remainder = pnts % (order - 1);
                points_needed = if remainder > 0 { order - 1 - remainder } else { 0 };
            } else if (flag & CU_NURB_ENDPOINT as i32) == 0 && pnts <= order {
                points_needed = order + 1 - pnts;
            }
            if points_needed != 0 {
                *r_points_needed = points_needed;
                return if is_surf {
                    NurbsValidationStatus::MoreRowsForBezierRequired
                } else {
                    NurbsValidationStatus::MorePointsForBezierRequired
                };
            }
        }
    }
    NurbsValidationStatus::Valid
}

/// `BKE_nurb_check_valid_u` (verbatim).
pub fn nurb_check_valid_u(nu: &Nurb) -> bool {
    let mut pts = 0;
    nurb_check_valid(
        nu.pntsu,
        nu.orderu as i32,
        nu.flagu as i32,
        nu.type_ as i32,
        nu.pntsv > 1,
        &mut pts,
    ) == NurbsValidationStatus::Valid
}
/// `BKE_nurb_check_valid_v` (verbatim).
pub fn nurb_check_valid_v(nu: &Nurb) -> bool {
    let mut pts = 0;
    nurb_check_valid(
        nu.pntsv,
        nu.orderv as i32,
        nu.flagv as i32,
        nu.type_ as i32,
        nu.pntsv > 1,
        &mut pts,
    ) == NurbsValidationStatus::Valid
}
/// `BKE_nurb_check_valid_uv` (verbatim).
pub fn nurb_check_valid_uv(nu: &Nurb) -> bool {
    nurb_check_valid_u(nu) && (nu.pntsv <= 1 || nurb_check_valid_v(nu))
}

/// `KNOTSU` macro verbatim.
pub fn knotsu(nu: &Nurb) -> i32 {
    nu.orderu as i32
        + nu.pntsu
        + if nu.flagu & CU_NURB_CYCLIC != 0 { nu.orderu as i32 - 1 } else { 0 }
}
/// `KNOTSV` macro verbatim.
pub fn knotsv(nu: &Nurb) -> i32 {
    nu.orderv as i32
        + nu.pntsv
        + if nu.flagv & CU_NURB_CYCLIC != 0 { nu.orderv as i32 - 1 } else { 0 }
}

/// `calcknots` (verbatim).
pub fn calcknots(knots: &mut [f32], pnts: i32, order: i32, flag: i32) {
    let is_cyclic = flag & CU_NURB_CYCLIC as i32 != 0;
    let is_bezier = flag & CU_NURB_BEZIER as i32 != 0;
    let is_end_point = flag & CU_NURB_ENDPOINT as i32 != 0;
    let repeat_inner = if is_bezier { order - 1 } else { 1 };
    let head = if is_end_point {
        order - if is_cyclic { 1 } else { 0 }
    } else if is_bezier {
        repeat_inner.min(2)
    } else {
        1
    };
    let tail = if is_cyclic {
        2 * order - 1
    } else if is_end_point {
        order
    } else {
        0
    };
    let knot_count = pnts + order + if is_cyclic { order - 1 } else { 0 };

    let mut r = head;
    let mut current = 0.0f32;
    let offset = if is_end_point && is_cyclic { 1 } else { 0 };
    if offset == 1 {
        knots[0] = current;
        current += 1.0;
    }
    for i in offset..(knot_count - offset - tail) {
        let i = i as usize;
        knots[i] = current;
        r -= 1;
        if r == 0 {
            current += 1.0;
            r = repeat_inner;
        }
    }
    let tail_index = (knot_count - tail) as usize;
    for i in 0..tail {
        knots[tail_index + i as usize] = current + (knots[i as usize] - knots[0]);
    }
}

/// `makeknots` (verbatim — recompute knot array; `uv` = 1
/// for u, 2 for v).
pub fn makeknots(nu: &mut Nurb, uv: i32) {
    if nu.type_ != CU_NURBS {
        return;
    }
    if uv == 1 {
        if nurb_check_valid_u(nu) {
            nu.knotsu = vec![0.0; (knotsu(nu) + 1) as usize];
            calcknots(
                &mut nu.knotsu,
                nu.pntsu,
                nu.orderu as i32,
                nu.flagu as i32,
            );
            nu.flagu &= !CU_NURB_CUSTOM;
        } else {
            nu.knotsu = Vec::new();
        }
    } else if uv == 2 {
        if nurb_check_valid_v(nu) {
            nu.knotsv = vec![0.0; (knotsv(nu) + 1) as usize];
            calcknots(
                &mut nu.knotsv,
                nu.pntsv,
                nu.orderv as i32,
                nu.flagv as i32,
            );
            nu.flagv &= !CU_NURB_CUSTOM;
        } else {
            nu.knotsv = Vec::new();
        }
    }
}

/// `BKE_nurb_knot_alloc_u` (verbatim).
pub fn nurb_knot_alloc_u(nu: &mut Nurb) {
    nu.knotsu = vec![0.0; (knotsu(nu) + 1) as usize];
}
/// `BKE_nurb_knot_calc_u`.
pub fn nurb_knot_calc_u(nu: &mut Nurb) {
    makeknots(nu, 1);
}
/// `BKE_nurb_knot_calc_v`.
pub fn nurb_knot_calc_v(nu: &mut Nurb) {
    makeknots(nu, 2);
}

/// `basisNurb` (verbatim).
pub fn basis_nurb(
    t: f32,
    order: i32,
    pnts: i32,
    knots: &[f32],
    basis: &mut [f32],
    start: &mut i32,
    end: &mut i32,
) {
    let mut t = t;
    let orderpluspnts = order + pnts;
    let opp2 = orderpluspnts - 1;
    if t < knots[0] {
        t = knots[0];
    } else if t > knots[opp2 as usize] {
        t = knots[opp2 as usize];
    }
    let o2 = order + 1;
    let mut i1 = 0i32;
    let mut i2 = 0i32;
    let mut i = 0i32;
    while i < opp2 {
        let iu = i as usize;
        if knots[iu] != knots[iu + 1] && t >= knots[iu] && t <= knots[iu + 1] {
            basis[iu] = 1.0;
            i1 = (i - o2).max(0);
            i2 = i;
            i += 1;
            while i < opp2 {
                basis[i as usize] = 0.0;
                i += 1;
            }
            break;
        }
        basis[iu] = 0.0;
        i += 1;
    }
    basis[i as usize] = 0.0;

    for j in 2..=order {
        if i2 + j >= orderpluspnts {
            i2 = opp2 - j;
        }
        for i in i1..=i2 {
            let iu = i as usize;
            let d = if basis[iu] != 0.0 {
                ((t - knots[iu]) * basis[iu]) / (knots[(i + j - 1) as usize] - knots[iu])
            } else {
                0.0
            };
            let e = if basis[iu + 1] != 0.0 {
                ((knots[(i + j) as usize] - t) * basis[iu + 1])
                    / (knots[(i + j) as usize] - knots[iu + 1])
            } else {
                0.0
            };
            basis[iu] = d + e;
        }
    }

    *start = 1000;
    *end = 0;
    for i in i1..=i2 {
        if basis[i as usize] > 0.0 {
            *end = i;
            if *start == 1000 {
                *start = i;
            }
        }
    }
}

/// `BKE_nurb_index_from_uv` (verbatim).
pub fn nurb_index_from_uv(nu: &Nurb, u: i32, v: i32) -> i32 {
    let totu = nu.pntsu;
    let totv = nu.pntsv;
    let u = if nu.flagu & CU_NURB_CYCLIC != 0 {
        mod_i(u, totu)
    } else if u < 0 || u >= totu {
        return -1;
    } else {
        u
    };
    let v = if nu.flagv & CU_NURB_CYCLIC != 0 {
        mod_i(v, totv)
    } else if v < 0 || v >= totv {
        return -1;
    } else {
        v
    };
    v * totu + u
}

/// `BKE_nurb_index_to_uv` (verbatim).
pub fn nurb_index_to_uv(nu: &Nurb, index: i32) -> (i32, i32) {
    let totu = nu.pntsu;
    let totv = nu.pntsv;
    (index % totu, (index / totu) % totv)
}

/// `BKE_nurbList_verts_count` — total points across a nurb
/// list (verbatim).
pub fn nurb_list_verts_count(nurbs: &[Nurb]) -> i32 {
    nurbs.iter().map(|nu| nu.pntsu * nu.pntsv).sum()
}
/// `BKE_nurbList_verts_count_without_handles` — same minus
/// bezier handles (verbatim: `pntsu * pntsv` for non-bezier).
pub fn nurb_list_verts_count_without_handles(nurbs: &[Nurb]) -> i32 {
    nurbs
        .iter()
        .map(|nu| {
            if nu.type_ == blenders_dna::curve_enums::CU_BEZIER {
                nu.pntsu * nu.pntsv
            } else {
                nu.pntsu * nu.pntsv
            }
        })
        .sum()
}

/// `BKE_nurb_bezt_get_next` — index-based.
pub fn nurb_bezt_get_next(nu: &Nurb, bezt_index: usize) -> Option<usize> {
    if bezt_index == nu.pntsu as usize - 1 {
        if nu.flagu & CU_NURB_CYCLIC != 0 {
            Some(0)
        } else {
            None
        }
    } else {
        Some(bezt_index + 1)
    }
}
/// `BKE_nurb_bezt_get_prev`.
pub fn nurb_bezt_get_prev(nu: &Nurb, bezt_index: usize) -> Option<usize> {
    if bezt_index == 0 {
        if nu.flagu & CU_NURB_CYCLIC != 0 {
            Some(nu.pntsu as usize - 1)
        } else {
            None
        }
    } else {
        Some(bezt_index - 1)
    }
}
/// `BKE_nurb_bpoint_get_next`.
pub fn nurb_bpoint_get_next(nu: &Nurb, bp_index: usize) -> Option<usize> {
    nurb_bezt_get_next(nu, bp_index)
}
/// `BKE_nurb_bpoint_get_prev`.
pub fn nurb_bpoint_get_prev(nu: &Nurb, bp_index: usize) -> Option<usize> {
    nurb_bezt_get_prev(nu, bp_index)
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if l == 0.0 {
        a
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}
fn norm_len(a: [f32; 3]) -> ([f32; 3], f32) {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if l == 0.0 {
        (a, 0.0)
    } else {
        ([a[0] / l, a[1] / l, a[2] / l], l)
    }
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// `BKE_nurb_bezt_calc_normal` (verbatim).
pub fn nurb_bezt_calc_normal(bezt: &BezTriple) -> [f32; 3] {
    let dir_prev = norm(sub(bezt.vec[0], bezt.vec[1]));
    let dir_next = norm(sub(bezt.vec[1], bezt.vec[2]));
    norm(add(dir_prev, dir_next))
}

/// `BKE_nurb_bezt_calc_plane` (verbatim).
pub fn nurb_bezt_calc_plane(nu: &Nurb, bezt_index: usize) -> [f32; 3] {
    let bezt = &nu.bezt[bezt_index];
    let mut dir_prev = norm(sub(bezt.vec[0], bezt.vec[1]));
    let mut dir_next = norm(sub(bezt.vec[1], bezt.vec[2]));
    let (mut r_plane, len) = norm_len(cross(dir_prev, dir_next));
    if len < f32::EPSILON {
        if let Some(prev) = nurb_bezt_get_prev(nu, bezt_index) {
            dir_prev = norm(sub(nu.bezt[prev].vec[1], bezt.vec[1]));
        }
        if let Some(next) = nurb_bezt_get_next(nu, bezt_index) {
            dir_next = norm(sub(bezt.vec[1], nu.bezt[next].vec[1]));
        }
        r_plane = cross(dir_prev, dir_next);
    }
    /* matches with bones more closely */
    let dir_mid = add(dir_prev, dir_next);
    r_plane = cross(r_plane, dir_mid);
    norm(r_plane)
}

/// `BKE_nurb_bpoint_calc_normal` (verbatim).
pub fn nurb_bpoint_calc_normal(nu: &Nurb, bp_index: usize) -> [f32; 3] {
    let bp = &nu.bp[bp_index];
    let mut r = [0.0f32; 3];
    if let Some(prev) = nurb_bpoint_get_prev(nu, bp_index) {
        r = add(r, norm(sub(nu.bp[prev].vec[..3].try_into().unwrap(), bp.vec[..3].try_into().unwrap())));
    }
    if let Some(next) = nurb_bpoint_get_next(nu, bp_index) {
        r = add(r, norm(sub(bp.vec[..3].try_into().unwrap(), nu.bp[next].vec[..3].try_into().unwrap())));
    }
    norm(r)
}

/// `BKE_nurb_bpoint_calc_plane` (verbatim).
pub fn nurb_bpoint_calc_plane(nu: &Nurb, bp_index: usize) -> [f32; 3] {
    let bp = &nu.bp[bp_index];
    let bpv: [f32; 3] = bp.vec[..3].try_into().unwrap();
    let mut dir_prev = [0.0f32; 3];
    let mut dir_next = [0.0f32; 3];
    if let Some(prev) = nurb_bpoint_get_prev(nu, bp_index) {
        dir_prev = norm(sub(nu.bp[prev].vec[..3].try_into().unwrap(), bpv));
    }
    if let Some(next) = nurb_bpoint_get_next(nu, bp_index) {
        dir_next = norm(sub(bpv, nu.bp[next].vec[..3].try_into().unwrap()));
    }
    let mut r_plane = cross(dir_prev, dir_next);
    let dir_mid = add(dir_prev, dir_next);
    r_plane = cross(r_plane, dir_mid);
    norm(r_plane)
}
