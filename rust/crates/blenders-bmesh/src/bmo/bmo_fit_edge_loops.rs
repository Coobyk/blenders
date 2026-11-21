//! Port of `bmo_fit_edge_loops_to_annotation.cc` — stretch selected verts to
//! the active annotation stroke.

use crate::edgeloop::*;
use crate::iterators::*;
use crate::operators::*;
use crate::edgeloop::BMEdgeLoopStore;
use crate::{BMesh, BM_EDGE, BM_ELEM_TAG};
use blenders_math::vector::Vec3;

const RAY_LENGTH: f32 = 1000.0;
const MAX_RAY_DISTANCE: f32 = 0.01;
const SEGMENT_EPSILON: f32 = 1e-4;
const NORMAL_LENGTH_EPSILON: f32 = 1e-3;

const TO_ANNOTATION_PROJECT: i32 = 0;
const TO_ANNOTATION_SPREAD_EVENLY: i32 = 1;
const TO_ANNOTATION_SPREAD_BY_CHAIN: i32 = 2;

/// `ToAnnotationChain`.
struct ToAnnotationChain {
    verts: Vec<usize>,
    center: Vec3,
    accumulated_lengths: Vec<f32>,
}

/// `StrokeData`.
struct StrokeData {
    points: Vec<Vec3>,
    center: Vec3,
    accumulated_lengths: Vec<f32>,
    use_count: i32,
}

/// `calc_annotation_input_chains_from_bm`.
fn calc_annotation_input_chains_from_bm(bm: &mut BMesh) -> Vec<ToAnnotationChain> {
    let params = BMEdgeLoopFindParams {
        use_vert_junction: true,
    };
    let eloops = bm_mesh_edgeloops_find(
        bm,
        &|bm: &BMesh, e| bm.edge(e).head.hflag & BM_ELEM_TAG != 0,
        Some(&params),
    );
    eloops
        .iter()
        .map(|el| ToAnnotationChain {
            verts: el.verts.iter().copied().collect(),
            center: Vec3::ZERO,
            accumulated_lengths: Vec::new(),
        })
        .collect()
}

/// `measure_positions`.
fn measure_positions(positions: &[Vec3], r_center: &mut Vec3, r_lengths: &mut Vec<f32>) {
    let n = positions.len();
    *r_center = Vec3::ZERO;
    for p in positions {
        *r_center += *p;
    }
    *r_center /= n as f32;
    r_lengths.clear();
    for i in 0..n - 1 {
        let prev = r_lengths.last().copied().unwrap_or(0.0);
        r_lengths.push(prev + (positions[i + 1] - positions[i]).length());
    }
}

/// `is_total_length_valid`.
fn is_total_length_valid(accumulated_lengths: &[f32]) -> bool {
    *accumulated_lengths.last().unwrap() > 0.0
}

/// `calculate_chain_lengths`.
fn calculate_chain_lengths(bm: &BMesh, chain: &mut ToAnnotationChain) {
    let positions: Vec<Vec3> = chain.verts.iter().map(|&v| bm.vert(v).co).collect();
    measure_positions(&positions, &mut chain.center, &mut chain.accumulated_lengths);
}

/// `calculate_stroke_lengths`.
fn calculate_stroke_lengths(stroke: &mut StrokeData) {
    let mut center = Vec3::ZERO;
    let mut lengths = Vec::new();
    measure_positions(&stroke.points, &mut center, &mut lengths);
    stroke.center = center;
    stroke.accumulated_lengths = lengths;
}

/// `sample_stroke_positions`.
fn sample_stroke_positions(
    stroke: &StrokeData,
    chain: &ToAnnotationChain,
    method: i32,
) -> Vec<Vec3> {
    use blenders_bli::length_parameterize::*;
    let verts_num = chain.verts.len();
    let (segment_indices, factors) = if method == TO_ANNOTATION_SPREAD_EVENLY {
        sample_uniform(&stroke.accumulated_lengths, true, verts_num)
    } else {
        let chain_length_total = *chain.accumulated_lengths.last().unwrap();
        let stroke_length_total = *stroke.accumulated_lengths.last().unwrap();
        let sample_lengths: Vec<f32> = (0..verts_num)
            .map(|i| {
                let sf = if i == 0 {
                    0.0
                } else {
                    chain.accumulated_lengths[i - 1] / chain_length_total
                };
                sf * stroke_length_total
            })
            .collect();
        sample_at_lengths(&stroke.accumulated_lengths, &sample_lengths)
    };
    (0..verts_num)
        .map(|i| {
            let si = segment_indices[i];
            let f = factors[i];
            stroke.points[si].lerp(stroke.points[si + 1], f)
        })
        .collect()
}

/// `calculate_alignment_distance`.
fn calculate_alignment_distance(
    bm: &BMesh,
    chain: &ToAnnotationChain,
    stroke: &StrokeData,
    method: i32,
) -> f32 {
    let stroke_positions = sample_stroke_positions(stroke, chain, method);
    let mut total = 0.0;
    for (i, &v) in chain.verts.iter().enumerate() {
        total += (bm.vert(v).co - stroke_positions[i]).length();
    }
    total
}

/// `edge_shares_face_with_tagged_edge`.
fn edge_shares_face_with_tagged_edge(bm: &BMesh, e: usize, v: usize) -> bool {
    let l_first = bm.edge(e).l;
    let mut l = l_first;
    loop {
        let l_other = crate::query::bm_loop_other_edge_loop(bm, l, v);
        let e_other = bm.loo(l_other).e;
        if bm.edge(e_other).head.hflag & BM_ELEM_TAG != 0 {
            return true;
        }
        l = bm.loo(l).radial_next;
        if l == l_first {
            return false;
        }
    }
}

/// `calculate_verts`.
fn calculate_verts(
    bm: &mut BMesh,
    stroke: &StrokeData,
    method: i32,
    factor: f32,
    lock_x: bool,
    lock_y: bool,
    lock_z: bool,
    chain: &mut ToAnnotationChain,
) {
    let verts_num = chain.verts.len();
    if method == TO_ANNOTATION_PROJECT {
        for i in 0..verts_num {
            let v = chain.verts[i];
            let v_co = bm.vert(v).co;
            let mut v_co_best = v_co;
            let mut dist_sq_best = f32::MAX;
            let mut has_best = false;
            let check_intersection = |bm: &BMesh, ray_dir: Vec3, best: &mut (Vec3, f32, bool)| {
                let ray_end = v_co + ray_dir * RAY_LENGTH;
                let ray_start = v_co - ray_dir * RAY_LENGTH;
                for j in 0..stroke.points.len() - 1 {
                    let seg_start = stroke.points[j];
                    let seg_end = stroke.points[j + 1];
                    let (ir, closest_on_ray, closest_on_stroke) =
                        blenders_math::geom::isect_line_line(
                            ray_start, ray_end, seg_start, seg_end);
                    if ir != 0 {
                        if (closest_on_ray - closest_on_stroke).length_squared()
                            < MAX_RAY_DISTANCE * MAX_RAY_DISTANCE
                        {
                            let dist_seg = (seg_start - seg_end).length();
                            let dist_to_start = (seg_start - closest_on_stroke).length();
                            let dist_to_end = (seg_end - closest_on_stroke).length();
                            if dist_to_start <= dist_seg + SEGMENT_EPSILON
                                && dist_to_end <= dist_seg + SEGMENT_EPSILON
                            {
                                let d = (v_co - closest_on_stroke).length_squared();
                                if d < best.1 {
                                    *best = (closest_on_stroke, d, true);
                                }
                            }
                        }
                    }
                }
            };
            /* shoot rays forwards/backwards along the direction of connected
             * unselected verts (except those on the same loop), fallback: normal. */
            for e in EdgesOfVert::new(bm, v) {
                if bm.edge(e).head.hflag & BM_ELEM_TAG == 0
                    && edge_shares_face_with_tagged_edge(bm, e, v)
                {
                    let v_other =
                        if bm.edge(e).v1 == v { bm.edge(e).v2 } else { bm.edge(e).v1 };
                    let ray_dir = (bm.vert(v_other).co - v_co).normalized();
                    let mut best = (v_co_best, dist_sq_best, has_best);
                    check_intersection(bm, ray_dir, &mut best);
                    v_co_best = best.0;
                    dist_sq_best = best.1;
                    has_best = best.2;
                }
            }
            if !has_best {
                let ray_dir = bm.vert(v).no;
                if ray_dir.length_squared() > NORMAL_LENGTH_EPSILON * NORMAL_LENGTH_EPSILON {
                    let mut best = (v_co_best, dist_sq_best, has_best);
                    check_intersection(bm, ray_dir.normalized(), &mut best);
                    v_co_best = best.0;
                    dist_sq_best = best.1;
                    has_best = best.2;
                }
            }
            if has_best {
                let mut new_co = v_co.lerp(v_co_best, factor);
                if lock_x {
                    new_co.x = v_co.x;
                }
                if lock_y {
                    new_co.y = v_co.y;
                }
                if lock_z {
                    new_co.z = v_co.z;
                }
                bm.vpool[v].co = new_co;
            }
        }
    } else {
        let stroke_positions = sample_stroke_positions(stroke, chain, method);
        for i in 0..verts_num {
            let v = chain.verts[i];
            let v_co = bm.vert(v).co;
            let mut new_co = v_co.lerp(stroke_positions[i], factor);
            if lock_x {
                new_co.x = v_co.x;
            }
            if lock_y {
                new_co.y = v_co.y;
            }
            if lock_z {
                new_co.z = v_co.z;
            }
            bm.vpool[v].co = new_co;
        }
    }
}

/// `bmo_fit_edge_loops_to_annotation_exec`.
///
/// The `strokes` slot pointer is a `*const Vec<Vec<Vec3>>` (opaque user data,
/// matching C's `Vector<Span<float3>>*`).
pub fn bmo_fit_edge_loops_to_annotation_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let factor = bmo_slot_float_get(op, false, "factor");
    let method = bmo_slot_int_get(op, false, "method");
    let lock_x = bmo_slot_bool_get(op, false, "lock_x");
    let lock_y = bmo_slot_bool_get(op, false, "lock_y");
    let lock_z = bmo_slot_bool_get(op, false, "lock_z");
    let strokes_ptr = op
        .slots_in
        .iter()
        .find(|s| s.name == "strokes")
        .map(|s| s.p)
        .unwrap_or(0);

    crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_EDGE, BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(bm, op, false, "geom", BM_EDGE, BM_ELEM_TAG, false);

    let mut chains = calc_annotation_input_chains_from_bm(bm);
    let mut strokes: Vec<StrokeData> = Vec::new();
    if strokes_ptr != 0 {
        let strokes_in = unsafe { &*(strokes_ptr as *const Vec<Vec<Vec3>>) };
        for points in strokes_in {
            let mut sd = StrokeData {
                points: points.clone(),
                center: Vec3::ZERO,
                accumulated_lengths: Vec::new(),
                use_count: 0,
            };
            calculate_stroke_lengths(&mut sd);
            strokes.push(sd);
        }
    }

    for chain in chains.iter_mut() {
        calculate_chain_lengths(bm, chain);
        if !is_total_length_valid(&chain.accumulated_lengths) {
            continue;
        }
        /* pick the closest least-used stroke, else fall back to the
         * line between first & last verts of the chain. */
        let stroke_fallback;
        let stroke_best: usize;
        if strokes.is_empty() {
            let chain_start_co = bm.vert(chain.verts[0]).co;
            let chain_end_co = bm.vert(*chain.verts.last().unwrap()).co;
            stroke_fallback = StrokeData {
                points: vec![chain_start_co, chain_end_co],
                center: Vec3::ZERO,
                accumulated_lengths: vec![(chain_start_co - chain_end_co).length()],
                use_count: 0,
            };
            strokes.push(stroke_fallback);
            stroke_best = strokes.len() - 1;
        } else {
            let mut index_best = usize::MAX;
            let mut use_count_best = i32::MAX;
            let mut dist_sq_best = f32::MAX;
            for (i, s) in strokes.iter().enumerate() {
                let d = (chain.center - s.center).length_squared();
                if s.use_count < use_count_best
                    || (s.use_count == use_count_best && d < dist_sq_best)
                {
                    use_count_best = s.use_count;
                    dist_sq_best = d;
                    index_best = i;
                }
            }
            strokes[index_best].use_count += 1;
            stroke_best = index_best;
        }
        let forward_dist =
            calculate_alignment_distance(bm, chain, &strokes[stroke_best], method);
        let mut reversed_chain = ToAnnotationChain {
            verts: chain.verts.iter().rev().copied().collect(),
            center: Vec3::ZERO,
            accumulated_lengths: Vec::new(),
        };
        calculate_chain_lengths(bm, &mut reversed_chain);
        let reversed_dist =
            calculate_alignment_distance(bm, &reversed_chain, &strokes[stroke_best], method);
        if reversed_dist < forward_dist {
            *chain = reversed_chain;
        }
        calculate_verts(
            bm,
            &strokes[stroke_best],
            method,
            factor,
            lock_x,
            lock_y,
            lock_z,
            chain,
        );
    }
}
