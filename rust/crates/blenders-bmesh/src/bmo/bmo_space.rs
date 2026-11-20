//! Port of `bmo_space_edge_loops_evenly.cc` — natural-cubic or linear
//! redistribution of edge-loop verts by arc length.

use crate::edgeloop::{BMEdgeLoopFindParams, bm_edgeloop_is_closed, bm_edgeloop_verts_get,
                      bm_mesh_edgeloops_find};
use crate::operators::*;
use crate::{BMesh, BM_EDGE};
use blenders_math::vector::Vec3;

const DUPLICATE_POSITION_THRESHOLD: f32 = 1e-6;
const SPACE_EPSILON: f32 = 1e-8;

const INTERP_CUBIC: i32 = 0;
const INTERP_LINEAR: i32 = 1;

struct SpaceChainData {
    verts: Vec<usize>,
    is_closed: bool,
}

pub struct SplineCoeffs {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub x: f32,
}

/// `get_space_input_chains`.
fn get_space_input_chains(bm: &mut BMesh) -> Vec<SpaceChainData> {
    let params = BMEdgeLoopFindParams { use_vert_junction: true };
    let eloops = bm_mesh_edgeloops_find(
        bm, &|bm: &BMesh, e| bm.edge(e).head.hflag & crate::BM_ELEM_TAG != 0,
        Some(&params));
    let mut chains = Vec::new();
    for el in &eloops {
        let verts: Vec<usize> = bm_edgeloop_verts_get(el).iter().copied().collect();
        let is_closed = bm_edgeloop_is_closed(el);
        let mut all_duplicate = true;
        for i in 0..verts.len().saturating_sub(1) {
            if (bm.vert(verts[i]).co - bm.vert(verts[i + 1]).co).length_squared()
                > DUPLICATE_POSITION_THRESHOLD * DUPLICATE_POSITION_THRESHOLD
            {
                all_duplicate = false;
                break;
            }
        }
        if !all_duplicate {
            chains.push(SpaceChainData { verts, is_closed });
        }
    }
    chains
}

/// `measure_chain` — positions + cumulative knot distances.
fn measure_chain(bm: &BMesh, chain: &SpaceChainData) -> (Vec<Vec3>, Vec<f32>) {
    let positions: Vec<Vec3> = chain.verts.iter().map(|&v| bm.vert(v).co).collect();
    let n = chain.verts.len();
    let mut knot_distances = vec![0.0f32; n + chain.is_closed as usize];
    for i in 1..n {
        knot_distances[i] = knot_distances[i - 1]
            + (positions[i] - positions[i - 1]).length();
    }
    if chain.is_closed {
        knot_distances[n] = knot_distances[n - 1]
            + (positions[0] - positions[n - 1]).length();
    }
    (positions, knot_distances)
}

/// `calculate_splines_axis` — natural cubic spline coefficients per axis.
pub fn calculate_splines_axis(
    distances: &[f32], coords: &[f32], is_closed: bool) -> Vec<SplineCoeffs>
{
    let mut coeffs = Vec::new();
    let verts_num = coords.len();
    if verts_num < 2 { return coeffs; }
    let num_segments = if is_closed { verts_num } else { verts_num - 1 };
    let mut segment_length = vec![0.0f32; num_segments];
    for i in 0..num_segments {
        segment_length[i] = distances[i + 1] - distances[i];
        if !(segment_length[i] > 0.0) { segment_length[i] = SPACE_EPSILON; }
    }
    let mut c_vals = vec![0.0f32; verts_num];
    if is_closed {
        let mut lower = vec![0.0f32; verts_num];
        let mut diag = vec![0.0f32; verts_num];
        let mut upper = vec![0.0f32; verts_num];
        let mut rhs = vec![0.0f32; verts_num];
        for i in 0..verts_num {
            let i_prev = (i + verts_num - 1) % verts_num;
            let i_next = (i + 1) % verts_num;
            lower[i] = segment_length[i_prev];
            diag[i] = 2.0 * (segment_length[i_prev] + segment_length[i]);
            upper[i] = segment_length[i];
            rhs[i] = 3.0 * (((coords[i_next] - coords[i]) / segment_length[i])
                - ((coords[i] - coords[i_prev]) / segment_length[i_prev]));
        }
        blenders_bli::math_solvers::tridiagonal_solve_cyclic(
            &lower, &diag, &upper, &rhs, &mut c_vals);
    } else {
        let interior = verts_num.saturating_sub(2);
        let mut lower = vec![0.0f32; interior];
        let mut diag = vec![0.0f32; interior];
        let mut upper = vec![0.0f32; interior];
        let mut rhs = vec![0.0f32; interior];
        for i in 0..interior {
            lower[i] = segment_length[i];
            diag[i] = 2.0 * (segment_length[i] + segment_length[i + 1]);
            upper[i] = segment_length[i + 1];
            rhs[i] = 3.0 * (((coords[i + 2] - coords[i + 1]) / segment_length[i + 1])
                - ((coords[i + 1] - coords[i]) / segment_length[i]));
        }
        blenders_bli::math_solvers::tridiagonal_solve(
            &lower, &diag, &upper, &rhs, &mut c_vals[1..1 + interior]);
    }
    for i in 0..num_segments {
        let i_next = if is_closed { (i + 1) % verts_num } else { i + 1 };
        let a = coords[i];
        let b = ((coords[i_next] - coords[i]) / segment_length[i])
            - (segment_length[i] * (c_vals[i_next] + 2.0 * c_vals[i])) / 3.0;
        let c = c_vals[i];
        let d = (c_vals[i_next] - c_vals[i]) / (3.0 * segment_length[i]);
        coeffs.push(SplineCoeffs { a, b, c, d, x: distances[i] });
    }
    coeffs
}

/// `calc_spline_segment`.
fn calc_spline_segment(knot_distances: &[f32], target: f32) -> usize {
    let i = knot_distances.partition_point(|&v| v <= target).saturating_sub(1);
    i.min(knot_distances.len().saturating_sub(2))
}

/// `evaluate_cubic`.
fn evaluate_cubic(
    knots: &[f32], cx: &[SplineCoeffs], cy: &[SplineCoeffs], cz: &[SplineCoeffs],
    target: f32) -> Vec3
{
    let seg = calc_spline_segment(knots, target);
    let dt = target - cx[seg].x;
    let (x, y, z) = (&cx[seg], &cy[seg], &cz[seg]);
    Vec3::new(
        x.a + dt * (x.b + dt * (x.c + dt * x.d)),
        y.a + dt * (y.b + dt * (y.c + dt * y.d)),
        z.a + dt * (z.b + dt * (z.c + dt * z.d)))
}

/// `bmo_space_edge_loops_evenly_exec`.
pub fn bmo_space_edge_loops_evenly_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let factor = bmo_slot_float_get(op, false, "factor");
    let interpolation = bmo_slot_int_get(op, false, "interpolation");
    let lock_x = bmo_slot_bool_get(op, false, "lock_x");
    let lock_y = bmo_slot_bool_get(op, false, "lock_y");
    let lock_z = bmo_slot_bool_get(op, false, "lock_z");
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, 1 << BM_EDGE, crate::BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(bm, op, false, "geom", 1 << BM_EDGE, crate::BM_ELEM_TAG, false);
    let chains = get_space_input_chains(bm);
    for chain in &chains {
        let verts_num = chain.verts.len();
        let (positions, knot_distances) = measure_chain(bm, chain);
        let (sample_indices, sample_factors) = blenders_bli::length_parameterize::sample_uniform(
            &knot_distances[1..], !chain.is_closed, verts_num);
        let mut new_positions = vec![Vec3::ZERO; verts_num];
        if interpolation == INTERP_LINEAR {
            for i in 0..verts_num {
                let seg = sample_indices[i];
                let f = sample_factors[i];
                new_positions[i] = positions[seg] + (positions[seg + 1] - positions[seg]) * f;
            }
        } else {
            let coords_x: Vec<f32> = positions.iter().map(|p| p.x).collect();
            let coords_y: Vec<f32> = positions.iter().map(|p| p.y).collect();
            let coords_z: Vec<f32> = positions.iter().map(|p| p.z).collect();
            let cx = calculate_splines_axis(&knot_distances, &coords_x, chain.is_closed);
            let cy = calculate_splines_axis(&knot_distances, &coords_y, chain.is_closed);
            let cz = calculate_splines_axis(&knot_distances, &coords_z, chain.is_closed);
            for i in 0..verts_num {
                let seg = sample_indices[i];
                let target = knot_distances[seg]
                    + (knot_distances[seg + 1] - knot_distances[seg]) * sample_factors[i];
                new_positions[i] = evaluate_cubic(&knot_distances, &cx, &cy, &cz, target);
            }
        }
        for i in 0..verts_num {
            if !chain.is_closed && (i == 0 || i == verts_num - 1) { continue; }
            let mut np = new_positions[i];
            if lock_x { np.x = positions[i].x; }
            if lock_y { np.y = positions[i].y; }
            if lock_z { np.z = positions[i].z; }
            bm.vpool[chain.verts[i]].co = positions[i] + (np - positions[i]) * factor;
        }
    }
}
