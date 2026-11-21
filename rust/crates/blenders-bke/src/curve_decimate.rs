//! Port of `source/blender/blenkernel/intern/curve_decimate.cc`.
//!
//! Heap-driven bezier-knot decimation by cubic-fit error. The
//! `curve_fit_cubic_to_points_single_fl` fitter (`extern/curve_fit_nd`)
//! and `BKE_curve_calc_coords_axis*` evaluation are injected
//! (`PORT_PENDING` until those modules are ported).

use blenders_bli::heap::{Heap, HeapNode};
use blenders_dna::curve_enums::{
    CU_NURB_CYCLIC, HD_ALIGN, HD_AUTO, HD_AUTO_ANIM, HD_FREE, HD_VECT,
};
use blenders_dna::curve_types::{BezTriple, Nurb};
use blenders_dna::id::NIL;
use blenders_math::vector::Vec3;

/// `Knot` — doubly-linked removal candidate (indices into
/// `Vec<Knot>`).
#[derive(Clone, Default)]
pub struct Knot {
    pub next: usize,
    pub prev: usize,
    /// Index in point array.
    pub point_index: u32,
    /// Index in knot array.
    pub knot_index: u32,
    pub tan: [Vec3; 2],
    pub handles: [f32; 2],
    /// Node in the heap if the knot can be removed.
    pub heap_node: Option<HeapNode>,
    /// New handles for the previous and next knots when this knot is
    /// removed.
    pub removal_handles: [f32; 2],
    pub can_remove: bool,
    pub is_removed: bool,
}

/// `curve_fit_cubic_to_points_single_fl` signature —
/// `(points, tan_l, tan_r) -> (handle_l, handle_r, error_sq)`.
pub type CurveFitSingle = dyn Fn(&[Vec3], Vec3, Vec3) -> ([f32; 3], [f32; 3], f32);

/// `knot_remove_error_value`.
fn knot_remove_error_value(
    fit_fn: &CurveFitSingle,
    tan_l: Vec3,
    tan_r: Vec3,
    points: &[Vec3],
    r_handle_factors: &mut [f32; 2],
) -> f32 {
    let points_len = points.len();
    let (handle_l, handle_r, error_sq) = fit_fn(points, tan_l, tan_r);

    let hl = Vec3::new(handle_l[0], handle_l[1], handle_l[2]) - points[0];
    r_handle_factors[0] = tan_l.dot(hl);
    let hr = Vec3::new(handle_r[0], handle_r[1], handle_r[2]) - points[points_len - 1];
    r_handle_factors[1] = tan_r.dot(hr);

    error_sq
}

/// `knot_remove_error_recalculate` — recompute removal cost and
/// update the knot's heap node.
fn knot_remove_error_recalculate(
    fit_fn: &CurveFitSingle,
    heap: &mut Heap<usize>,
    knots: &mut [Knot],
    points: &[Vec3],
    points_len: usize,
    k: usize,
    error_sq_max: f32,
) {
    debug_assert!(knots[k].can_remove);
    let mut handles = [0.0f32; 2];

    let (p_prev, p_next) = (knots[k].prev, knots[k].next);
    let (pi_prev, pi_next) = (
        knots[p_prev].point_index as usize,
        knots[p_next].point_index as usize,
    );

    let points_offset: &[Vec3];
    let points_offset_len: usize;
    if pi_prev < pi_next {
        points_offset = &points[pi_prev..];
        points_offset_len = (pi_next - pi_prev) + 1;
    } else {
        points_offset = &points[pi_prev..];
        points_offset_len = ((pi_next + points_len) - pi_prev) + 1;
    }

    let cost_sq = knot_remove_error_value(
        fit_fn,
        knots[p_prev].tan[1],
        knots[p_next].tan[0],
        &points_offset[..points_offset_len],
        &mut handles,
    );

    if cost_sq < error_sq_max {
        knots[k].removal_handles = handles;
        heap.insert_or_update(&mut knots[k].heap_node, cost_sq, k);
    } else if let Some(node) = knots[k].heap_node.take() {
        heap.remove(node);
    }
}

/// `curve_decimate` — pop lowest-error removable knots until the
/// target count or error bound.
fn curve_decimate(
    fit_fn: &CurveFitSingle,
    points: &[Vec3],
    points_len: usize,
    knots: &mut Vec<Knot>,
    knots_len: usize,
    error_sq_max: f32,
    error_target_len: u32,
) {
    let mut heap: Heap<usize> = Heap::with_capacity(knots_len);
    for i in 0..knots_len {
        if knots[i].can_remove {
            knot_remove_error_recalculate(fit_fn, &mut heap, knots, points, points_len, i, error_sq_max);
        }
    }

    let mut knots_len_remaining = knots_len as u32;

    while knots_len_remaining > error_target_len && !heap.is_empty() {
        let k = heap.pop_min().unwrap();
        knots[k].heap_node = None;
        let (k_prev, k_next) = (knots[k].prev, knots[k].next);
        knots[k_prev].handles[1] = knots[k].removal_handles[0];
        knots[k_next].handles[0] = knots[k].removal_handles[1];

        /* remove ourselves */
        knots[k_next].prev = k_prev;
        knots[k_prev].next = k_next;

        knots[k].next = NIL;
        knots[k].prev = NIL;
        knots[k].is_removed = true;

        if knots[k_prev].can_remove {
            knot_remove_error_recalculate(fit_fn, &mut heap, knots, points, points_len, k_prev, error_sq_max);
        }
        if knots[k_next].can_remove {
            knot_remove_error_recalculate(fit_fn, &mut heap, knots, points, points_len, k_next, error_sq_max);
        }

        knots_len_remaining -= 1;
    }
}

/// `BKE_curve_calc_coords_axis` signature — evaluates the sampled
/// positions along one axis into `co` (`stride` folded away).
pub type CalcCoordsAxis = dyn Fn(&[BezTriple], u32, bool, usize, &mut [f32]);
/// `BKE_curve_calc_coords_axis_len`.
pub type CalcCoordsAxisLen = dyn Fn(u32, u32, bool, bool) -> u32;

/// `BKE_curve_decimate_bezt_array` — decimate an array of
/// `BezTriple`s; returns the surviving count.
pub fn bke_curve_decimate_bezt_array(
    bezt_array: &mut [BezTriple],
    resolu: u32,
    is_cyclic: bool,
    flag_test: u8,
    flag_set: u8,
    error_sq_max: f32,
    error_target_len: u32,
    calc_coords_axis_len: &CalcCoordsAxisLen,
    calc_coords_axis: &CalcCoordsAxis,
    fit_fn: &CurveFitSingle,
) -> u32 {
    let bezt_array_len = bezt_array.len() as u32;
    let bezt_array_last = bezt_array_len as usize - 1;
    let points_len =
        calc_coords_axis_len(bezt_array_len, resolu, is_cyclic, true) as usize;

    let mut points = vec![Vec3::ZERO; points_len * if is_cyclic { 2 } else { 1 }];

    /* Sample each axis separately like the strided C version. */
    for axis in 0..3 {
        let mut axis_pts = vec![0.0f32; points_len];
        calc_coords_axis(bezt_array, resolu, is_cyclic, axis, &mut axis_pts);
        for (i, v) in axis_pts.iter().enumerate() {
            points[i][axis] = *v;
        }
    }

    let knots_len = bezt_array_len as usize;
    let mut knots: Vec<Knot> = Vec::with_capacity(knots_len);

    if is_cyclic {
        let (a, b) = points.split_at_mut(points_len);
        b[..points_len].copy_from_slice(&a[..points_len]);
    }

    for i in 0..bezt_array_len as usize {
        let mut k = Knot {
            heap_node: None,
            can_remove: bezt_array[i].f2 & flag_test != 0,
            is_removed: false,
            next: i + 1,
            prev: i.wrapping_sub(1),
            point_index: i as u32 * resolu,
            knot_index: i as u32,
            ..Default::default()
        };

        let tan0 = Vec3::from(bezt_array[i].vec[0]) - Vec3::from(bezt_array[i].vec[1]);
        k.handles[0] = tan0.length();
        k.tan[0] = if k.handles[0] != 0.0 { tan0 / k.handles[0] } else { Vec3::ZERO };
        let tan1 = Vec3::from(bezt_array[i].vec[1]) - Vec3::from(bezt_array[i].vec[2]);
        k.handles[1] = -tan1.length();
        k.tan[1] = if tan1.length() != 0.0 { tan1 / tan1.length() } else { Vec3::ZERO };
        knots.push(k);
    }

    if is_cyclic {
        knots[0].prev = bezt_array_last;
        knots[bezt_array_last].next = 0;
    } else {
        knots[0].prev = NIL;
        knots[bezt_array_last].next = NIL;

        /* always keep end-points */
        knots[0].can_remove = false;
        knots[bezt_array_last].can_remove = false;
    }

    curve_decimate(
        fit_fn,
        &points,
        points_len,
        &mut knots,
        knots_len,
        error_sq_max,
        error_target_len,
    );

    let mut knots_len_decimated = knots_len as u32;

    /* Update handle type on modifications (HANDLE_UPDATE). */
    for i in 0..bezt_array_len as usize {
        if knots[i].is_removed {
            /* caller must remove */
            bezt_array[i].f2 |= flag_set;
            knots_len_decimated -= 1;
        } else {
            bezt_array[i].f2 &= !flag_set;
            if is_cyclic || i != 0 {
                let i_prev = if i != 0 { i - 1 } else { bezt_array_last };
                if knots[i_prev].is_removed {
                    /* madd_v3_v3v3fl(vec[0], vec[1], tan, handles) */
                    let h = Vec3::from(bezt_array[i].vec[1])
                        + knots[i].tan[0] * knots[i].handles[0];
                    bezt_array[i].vec[0] = h.into();
                    if bezt_array[i].h1 == HD_VECT {
                        bezt_array[i].h1 = HD_FREE;
                    } else if matches!(bezt_array[i].h1, HD_AUTO | HD_AUTO_ANIM) {
                        bezt_array[i].h1 = HD_ALIGN;
                    }
                    if matches!(bezt_array[i].h2, HD_AUTO | HD_AUTO_ANIM) {
                        bezt_array[i].h2 = HD_ALIGN;
                    }
                }
            }
            if is_cyclic || i != bezt_array_last {
                let i_next = if i != bezt_array_last { i + 1 } else { 0 };
                if knots[i_next].is_removed {
                    let h = Vec3::from(bezt_array[i].vec[1])
                        + knots[i].tan[1] * knots[i].handles[1];
                    bezt_array[i].vec[2] = h.into();
                    if bezt_array[i].h2 == HD_VECT {
                        bezt_array[i].h2 = HD_FREE;
                    } else if matches!(bezt_array[i].h2, HD_AUTO | HD_AUTO_ANIM) {
                        bezt_array[i].h2 = HD_ALIGN;
                    }
                    if matches!(bezt_array[i].h1, HD_AUTO | HD_AUTO_ANIM) {
                        bezt_array[i].h1 = HD_ALIGN;
                    }
                }
            }
        }
    }

    knots_len_decimated
}

/// `BKE_curve_decimate_nurb` — decimate `nu->bezt` in place and
/// compact the array.
pub fn bke_curve_decimate_nurb(
    nu: &mut Nurb,
    resolu: u32,
    error_sq_max: f32,
    error_target_len: u32,
    calc_coords_axis_len: &CalcCoordsAxisLen,
    calc_coords_axis: &CalcCoordsAxis,
    fit_fn: &CurveFitSingle,
) {
    const SELECT: u8 = 1;
    let flag_test = blenders_dna::curve_enums::BEZT_FLAG_TEMP_TAG;

    let pntsu_dst = bke_curve_decimate_bezt_array(
        &mut nu.bezt,
        resolu,
        (nu.flagu & CU_NURB_CYCLIC) != 0,
        SELECT,
        flag_test,
        error_sq_max,
        error_target_len,
        calc_coords_axis_len,
        calc_coords_axis,
        fit_fn,
    );

    if pntsu_dst as i32 == nu.pntsu {
        return;
    }

    let mut bezt_dst: Vec<BezTriple> = Vec::with_capacity(pntsu_dst as usize);
    for bt in nu.bezt.iter() {
        if bt.f2 & flag_test == 0 {
            bezt_dst.push(bt.clone());
        }
    }
    nu.bezt = bezt_dst;
    nu.pntsu = nu.bezt.len() as i32;
}
