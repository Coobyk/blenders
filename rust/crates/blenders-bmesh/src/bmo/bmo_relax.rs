//! Port of `bmo_relax_edge_loops.cc` — Gauss–Seidel-style two-phase
//! spline relaxation of edge loops.

use crate::bmo::bmo_space::calculate_splines_axis;
use crate::edgeloop::{BMEdgeLoopFindParams, bm_edgeloop_is_closed, bm_edgeloop_verts_get,
                      bm_mesh_edgeloops_find};
use crate::operators::*;
use crate::{BMesh, BM_EDGE, BM_VERT};
use blenders_math::vector::Vec3;

const RELAX_EDGE_LOOPS_INTERP_LINEAR: i32 = 1;

struct RelaxPhase {
    knot_indices: Vec<usize>,
    point_indices: Vec<usize>,
}

struct RelaxChainData {
    verts: Vec<usize>,
    is_closed: bool,
    phases: Vec<RelaxPhase>,
}

/// `build_relax_phases`.
fn build_relax_phases(verts_num: usize, is_closed: bool) -> Vec<RelaxPhase> {
    let mut phases = Vec::new();
    if !is_closed {
        for phase_index in 0..2usize {
            let mut phase = RelaxPhase { knot_indices: vec![], point_indices: vec![] };
            for i in 0..verts_num {
                if i % 2 == phase_index {
                    phase.knot_indices.push(i);
                } else if i > 0 && i < verts_num - 1 {
                    phase.point_indices.push(i);
                }
            }
            phases.push(phase);
        }
        return phases;
    }
    let mut vert_indices: Vec<usize> = (0..verts_num).collect();
    for j in 0..2usize {
        let extend = if verts_num % 2 == 1 { j == 1 } else { j == 0 };
        let knot_start = if !extend && j == 1 { 1 } else { 0 };
        let point_start = if !extend && j == 1 { 2 } else { 1 };
        if extend {
            let last_vert = *vert_indices.last().unwrap();
            let first_vert = vert_indices[0];
            vert_indices.insert(0, last_vert);
            vert_indices.push(first_vert);
        }
        let mut phase = RelaxPhase { knot_indices: vec![], point_indices: vec![] };
        let mut i = knot_start;
        while i < vert_indices.len() {
            phase.knot_indices.push(vert_indices[i]);
            i += 2;
        }
        let mut i = point_start;
        while i < vert_indices.len() {
            let val = vert_indices[i];
            if phase.point_indices.is_empty() || val != phase.point_indices[0] {
                phase.point_indices.push(val);
            }
            i += 2;
        }
        if !phase.knot_indices.is_empty()
            && phase.knot_indices[0] != *phase.knot_indices.last().unwrap()
        {
            let first = phase.knot_indices[0];
            phase.knot_indices.push(first);
        }
        if !phase.point_indices.is_empty() {
            phases.push(phase);
        }
        if extend {
            vert_indices.remove(0);
            vert_indices.pop();
        }
    }
    phases
}

/// `get_relax_input_chains`.
fn get_relax_input_chains(bm: &mut BMesh) -> Vec<RelaxChainData> {
    let params = BMEdgeLoopFindParams { use_vert_junction: true };
    let eloops = bm_mesh_edgeloops_find(
        bm, &|bm: &BMesh, e| bm.edge(e).head.hflag & crate::BM_ELEM_TAG != 0,
        Some(&params));
    let mut chains = Vec::new();
    for el in &eloops {
        let verts: Vec<usize> = bm_edgeloop_verts_get(el).iter().copied().collect();
        if verts.len() >= 3 {
            chains.push(RelaxChainData {
                verts,
                is_closed: bm_edgeloop_is_closed(el),
                phases: vec![],
            });
        }
    }
    chains
}

/// `calculate_phase_distances`.
fn calculate_phase_distances(
    bm: &BMesh, verts: &[usize], phase: &RelaxPhase, regular: bool)
    -> (Vec<f32>, Vec<f32>)
{
    let total = phase.knot_indices.len() + phase.point_indices.len();
    let mut positions = vec![Vec3::ZERO; total];
    for i in 0..total {
        let vert_index = if i % 2 == 0 {
            phase.knot_indices[i / 2]
        } else if i == total - 1 {
            *phase.knot_indices.last().unwrap()
        } else {
            phase.point_indices[i / 2]
        };
        positions[i] = bm.vert(verts[vert_index]).co;
    }
    let mut cumulative = vec![0.0f32; total];
    for i in 1..total {
        cumulative[i] = cumulative[i - 1] + (positions[i] - positions[i - 1]).length();
    }
    let mut knot_distances = Vec::new();
    let mut point_distances = Vec::new();
    for i in 0..total {
        if i % 2 == 0 || i == total - 1 {
            knot_distances.push(cumulative[i]);
        } else {
            point_distances.push(cumulative[i]);
        }
    }
    if regular {
        point_distances.clear();
        for p in 0..phase.point_indices.len() {
            point_distances.push((knot_distances[p] + knot_distances[p + 1]) / 2.0);
        }
    }
    (knot_distances, point_distances)
}

/// `execute_relax_phase`.
fn execute_relax_phase(
    bm: &mut BMesh, verts: &[usize], phase: &RelaxPhase, is_closed: bool,
    interpolation: i32, regular: bool)
{
    if phase.point_indices.is_empty() { return; }
    let (knot_distances, point_distances) =
        calculate_phase_distances(bm, verts, phase, regular);
    let points_num = phase.point_indices.len();
    let (segment_indices, factors) = blenders_bli::length_parameterize::sample_at_lengths(
        &knot_distances[1..], &point_distances);
    let mut sampled_positions = vec![Vec3::ZERO; points_num];
    if interpolation == RELAX_EDGE_LOOPS_INTERP_LINEAR {
        let knot_positions: Vec<Vec3> = phase.knot_indices.iter()
            .map(|&ki| bm.vert(verts[ki]).co).collect();
        for i in 0..points_num {
            let seg = segment_indices[i];
            sampled_positions[i] = knot_positions[seg]
                + (knot_positions[seg + 1] - knot_positions[seg]) * factors[i];
        }
    } else {
        /* cubic splines on knot coords */
        let coords_num = if is_closed {
            phase.knot_indices.len() - 1
        } else {
            phase.knot_indices.len()
        };
        let mut axis_coords: [Vec<f32>; 3] = [vec![], vec![], vec![]];
        for axis in 0..3 {
            axis_coords[axis] = (0..coords_num).map(|i| {
                let co = bm.vert(verts[phase.knot_indices[i]]).co;
                [co.x, co.y, co.z][axis]
            }).collect();
        }
        let coeffs: Vec<Vec<crate::bmo::bmo_space::SplineCoeffs>> = (0..3)
            .map(|a| calculate_splines_axis(&knot_distances, &axis_coords[a], is_closed))
            .collect();
        for i in 0..points_num {
            let seg = segment_indices[i];
            let dt = point_distances[i] - coeffs[0][seg].x;
            let (cx, cy, cz) = (&coeffs[0][seg], &coeffs[1][seg], &coeffs[2][seg]);
            sampled_positions[i] = Vec3::new(
                cx.a + dt * (cx.b + dt * (cx.c + dt * cx.d)),
                cy.a + dt * (cy.b + dt * (cy.c + dt * cy.d)),
                cz.a + dt * (cz.b + dt * (cz.c + dt * cz.d)));
        }
    }
    for i in 0..points_num {
        let v_index = phase.point_indices[i];
        let cur = bm.vert(verts[v_index]).co;
        bm.vpool[verts[v_index]].co = (cur + sampled_positions[i]) / 2.0;
    }
}

/// `bmo_relax_edge_loops_exec`.
pub fn bmo_relax_edge_loops_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let iterations = bmo_slot_int_get(op, false, "iterations");
    let interpolation = bmo_slot_int_get(op, false, "interpolation");
    let even_spacing = bmo_slot_bool_get(op, false, "even_spacing");
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, 1 << BM_EDGE, crate::BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(bm, op, false, "geom", 1 << BM_EDGE, crate::BM_ELEM_TAG, false);
    let mut chains = get_relax_input_chains(bm);
    for chain in &mut chains {
        chain.phases = build_relax_phases(chain.verts.len(), chain.is_closed);
    }
    for _ in 0..iterations {
        for chain in &chains {
            for phase in &chain.phases {
                execute_relax_phase(
                    bm, &chain.verts, phase, chain.is_closed, interpolation, even_spacing);
            }
        }
    }
    let _ = BM_VERT;
}
