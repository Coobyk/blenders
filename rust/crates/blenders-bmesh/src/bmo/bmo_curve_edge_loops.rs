//! Port of `bmo_curve_edge_loops_between_selected.cc` — cubic/linear spline
//! interpolation through tagged verts along walked edge loops.

use crate::iterators::*;
use crate::operators::*;
use crate::BMesh;
use crate::{BM_ELEM_HIDDEN, BM_ELEM_TAG, BM_VERT};
use blenders_math::vector::Vec3;
use std::collections::HashSet;

const CURVE_EPSILON: f32 = 1e-8;

const CURVE_INTERP_LINEAR: i32 = 0;
const CURVE_INTERP_CUBIC: i32 = 1;

const CURVE_CLAMP_ELEVATION_NONE: i32 = 0;
const CURVE_CLAMP_ELEVATION_RAISE: i32 = 1;
const CURVE_CLAMP_ELEVATION_LOWER: i32 = 2;

/// `CurveChainData` — a chain of vertices collected from a walk along
/// connected edges.
struct CurveChainData {
    verts: Vec<usize>,
    is_closed: bool,
}

/// `SplineCoeffs` — coefficients for the cubic spline curve equation.
#[derive(Clone, Copy, Default)]
struct SplineCoeffs {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    /// Parameter value at the start of the segment.
    x: f32,
}

fn mod_periodic(i: i32, n: i32) -> usize {
    i.rem_euclid(n) as usize
}

fn bm_edge_is_wire(bm: &BMesh, e: usize) -> bool {
    bm.edge(e).l == crate::BM_NIL
}

/// `calculate_splines_axis` — cubic spline coeffs for one coordinate axis.
/// Natural cubic: `BLI_tridiagonal_solve` for open, `_cyclic` for closed.
fn calculate_splines_axis(
    distances: &[f32],
    coords: &[f32],
    is_closed: bool,
) -> Vec<SplineCoeffs> {
    let verts_num = coords.len();
    if verts_num < 2 {
        return Vec::new();
    }
    let segments_num = if is_closed { verts_num } else { verts_num - 1 };
    let mut segment_length = vec![0.0f32; segments_num];
    for i in 0..segments_num {
        segment_length[i] = distances[i + 1] - distances[i];
        if segment_length[i] <= 0.0 {
            segment_length[i] = CURVE_EPSILON;
        }
    }
    /* Second derivative coefficients; natural spline bounds → 0. */
    let mut c_vals = vec![0.0f32; verts_num];
    if is_closed {
        let mut lower_diag = vec![0.0; verts_num];
        let mut diag = vec![0.0; verts_num];
        let mut upper_diag = vec![0.0; verts_num];
        let mut rhs = vec![0.0; verts_num];
        for i in 0..verts_num {
            let i_prev = mod_periodic(i as i32 - 1, verts_num as i32);
            let i_next = mod_periodic(i as i32 + 1, verts_num as i32);
            lower_diag[i] = segment_length[i_prev];
            diag[i] = 2.0 * (segment_length[i_prev] + segment_length[i]);
            upper_diag[i] = segment_length[i];
            rhs[i] = 3.0
                * (((coords[i_next] - coords[i]) / segment_length[i])
                    - ((coords[i] - coords[i_prev]) / segment_length[i_prev]));
        }
        blenders_bli::math_solvers::tridiagonal_solve_cyclic(
            &lower_diag, &diag, &upper_diag, &rhs, &mut c_vals);
    } else {
        let interior = verts_num as i32 - 2;
        if interior > 0 {
            let interior = interior as usize;
            let mut lower_diag = vec![0.0; interior];
            let mut diag = vec![0.0; interior];
            let mut upper_diag = vec![0.0; interior];
            let mut rhs = vec![0.0; interior];
            for i_curr in 0..interior {
                let i_next = i_curr + 1;
                lower_diag[i_curr] = segment_length[i_curr];
                diag[i_curr] =
                    2.0 * (segment_length[i_curr] + segment_length[i_next]);
                upper_diag[i_curr] = segment_length[i_next];
                rhs[i_curr] = 3.0
                    * (((coords[i_next + 1] - coords[i_next]) / segment_length[i_next])
                        - ((coords[i_next] - coords[i_curr]) / segment_length[i_curr]));
            }
            blenders_bli::math_solvers::tridiagonal_solve(
                &lower_diag, &diag, &upper_diag, &rhs, &mut c_vals[1..1 + interior]);
        }
    }
    /* Build polynomial coefficients for each segment. */
    let mut r_coeffs = vec![SplineCoeffs::default(); segments_num];
    for i in 0..segments_num {
        let i_next = if is_closed {
            mod_periodic(i as i32 + 1, verts_num as i32)
        } else {
            i + 1
        };
        let coeff_a = coords[i];
        let coeff_b = ((coords[i_next] - coords[i]) / segment_length[i])
            - (segment_length[i] * (c_vals[i_next] + 2.0 * c_vals[i])) / 3.0;
        let coeff_c = c_vals[i];
        let coeff_d = (c_vals[i_next] - c_vals[i]) / (3.0 * segment_length[i]);
        r_coeffs[i] = SplineCoeffs {
            a: coeff_a,
            b: coeff_b,
            c: coeff_c,
            d: coeff_d,
            x: distances[i],
        };
    }
    r_coeffs
}

/// `traverse_edge_loop` — walk continuously in one direction along quad or
/// wire edges until it loops back or hits the end.
fn traverse_edge_loop(
    bm: &BMesh,
    v_start: usize,
    e_start: usize,
    visited_edges: &mut HashSet<usize>,
    r_loop: &mut Vec<usize>,
    r_circular: &mut bool,
) {
    let mut v_curr = v_start;
    let mut e_curr = e_start;
    visited_edges.insert(e_curr);
    loop {
        let v_next =
            if bm.edge(e_curr).v1 == v_curr { bm.edge(e_curr).v2 } else { bm.edge(e_curr).v1 };
        r_loop.push(v_next);
        if v_next == v_start {
            *r_circular = true;
            r_loop.pop();
            break;
        }
        /* Never step between wire & face edges. */
        let is_wire = bm_edge_is_wire(bm, e_curr);
        let mut edge_count = 0;
        for e in EdgesOfVert::new(bm, v_next) {
            if bm_edge_is_wire(bm, e) != is_wire {
                continue;
            }
            if bm.edge(e).head.hflag & BM_ELEM_HIDDEN == 0 {
                edge_count += 1;
            }
        }
        if is_wire {
            /* Stop at the end of a wire chain or where it branches. */
            if edge_count != 2 {
                break;
            }
        } else {
            /* Stop at poles & corners, where the loop has no single continuation. */
            if !(3..=4).contains(&edge_count) {
                break;
            }
        }
        let mut e_next = crate::BM_NIL;
        for e in EdgesOfVert::new(bm, v_next) {
            if bm_edge_is_wire(bm, e) != is_wire {
                continue;
            }
            if e == e_curr
                || bm.edge(e).head.hflag & BM_ELEM_HIDDEN != 0
                || visited_edges.contains(&e)
            {
                continue;
            }
            if !crate::query::bm_edge_share_quad_check(bm, e_curr, e) {
                e_next = e;
                break;
            }
        }
        if e_next == crate::BM_NIL {
            break;
        }
        e_curr = e_next;
        v_curr = v_next;
        visited_edges.insert(e_curr);
    }
}

/// `build_curve_chain_from_edge`.
fn build_curve_chain_from_edge(
    bm: &BMesh,
    e_start: usize,
    visited_edges: &mut HashSet<usize>,
) -> CurveChainData {
    let mut chain = CurveChainData {
        verts: Vec::new(),
        is_closed: false,
    };
    let mut forward_trace: Vec<usize> = Vec::new();
    let mut is_forward_circular = false;
    traverse_edge_loop(
        bm,
        bm.edge(e_start).v1,
        e_start,
        visited_edges,
        &mut forward_trace,
        &mut is_forward_circular,
    );
    if is_forward_circular {
        chain.is_closed = true;
        chain.verts.push(bm.edge(e_start).v1);
        chain.verts.extend(forward_trace);
    } else {
        let mut backward_trace: Vec<usize> = Vec::new();
        let mut is_backward_circular = false;
        traverse_edge_loop(
            bm,
            bm.edge(e_start).v2,
            e_start,
            visited_edges,
            &mut backward_trace,
            &mut is_backward_circular,
        );
        backward_trace.reverse();
        chain.verts.extend(backward_trace);
        chain.verts.extend(forward_trace);
    }
    chain
}

/// `get_perpendicular_chains`.
fn get_perpendicular_chains(
    bm: &BMesh,
    chain: &CurveChainData,
    r_chains: &mut Vec<CurveChainData>,
    visited_edges: &mut HashSet<usize>,
) {
    for &v in &chain.verts {
        for e in EdgesOfVert::new(bm, v) {
            if bm.edge(e).head.hflag & BM_ELEM_HIDDEN != 0 || visited_edges.contains(&e) {
                continue;
            }
            r_chains.push(build_curve_chain_from_edge(bm, e, visited_edges));
        }
    }
}

/// `calc_curve_input_chains_from_bm`.
fn calc_curve_input_chains_from_bm(bm: &BMesh) -> Vec<CurveChainData> {
    let tag_verts: Vec<usize> = VertsOfMesh::new(bm)
        .filter(|&v| bm.vert(v).head.hflag & BM_ELEM_TAG != 0)
        .collect();
    let mut visited_edges: HashSet<usize> = HashSet::new();
    let mut r_chains = Vec::new();
    for v_start in tag_verts {
        for e_start in EdgesOfVert::new(bm, v_start) {
            if bm.edge(e_start).head.hflag & BM_ELEM_HIDDEN != 0
                || visited_edges.contains(&e_start)
            {
                continue;
            }
            let chain = build_curve_chain_from_edge(bm, e_start, &mut visited_edges);
            if chain.verts.len() < 3 {
                continue;
            }
            let tag_count = chain
                .verts
                .iter()
                .filter(|&&v| bm.vert(v).head.hflag & BM_ELEM_TAG != 0)
                .count();
            /* If a whole edge loop is tagged, all edge loops perpendicular to it are
             * operated on, with the tagged edge loop being used as control points. */
            if tag_count == chain.verts.len() {
                get_perpendicular_chains(bm, &chain, &mut r_chains, &mut visited_edges);
            } else if tag_count >= 1 {
                r_chains.push(chain);
            }
        }
    }
    r_chains
}

/// `find_longest_unselected_gap`.
fn find_longest_unselected_gap(
    bm: &BMesh,
    verts: &[usize],
) -> (i32, i32) {
    let verts_num = verts.len() as i32;
    let selected_indices: Vec<i32> = verts
        .iter()
        .enumerate()
        .filter(|(_, &v)| bm.vert(v).head.hflag & BM_ELEM_TAG != 0)
        .map(|(i, _)| i as i32)
        .collect();
    if selected_indices.is_empty() {
        return (verts_num, 0);
    }
    let mut longest_gap = -1;
    let mut best_gap_start = -1;
    for i in 0..selected_indices.len() {
        let i_curr = selected_indices[i];
        let i_next = selected_indices
            [mod_periodic(i as i32 + 1, selected_indices.len() as i32)];
        let gap_size = (i_next - i_curr - 1).rem_euclid(verts_num);
        if gap_size > longest_gap {
            longest_gap = gap_size;
            best_gap_start = (i_curr + 1).rem_euclid(verts_num);
        }
    }
    (longest_gap, best_gap_start)
}

/// `trim_chains_to_boundaries`.
fn trim_chains_to_boundaries(bm: &BMesh, chains: &mut Vec<CurveChainData>) {
    for chain in chains.iter_mut() {
        if chain.is_closed {
            let (longest_gap, best_gap_start) =
                find_longest_unselected_gap(bm, &chain.verts);
            if longest_gap > 0 {
                let new_start_index = mod_periodic(
                    best_gap_start + longest_gap,
                    chain.verts.len() as i32,
                );
                /* Pushes the unselected gap to the end so it's easy to trim off. */
                chain.verts.rotate_left(new_start_index);
            }
            chain.is_closed = false;
        }
        let mut first_selected = -1i32;
        let mut last_selected = -1i32;
        for (i, &v) in chain.verts.iter().enumerate() {
            if bm.vert(v).head.hflag & BM_ELEM_TAG != 0 {
                if first_selected == -1 {
                    first_selected = i as i32;
                }
                last_selected = i as i32;
            }
        }
        if first_selected > 0 || last_selected < chain.verts.len() as i32 - 1 {
            let trimmed: Vec<usize> =
                chain.verts[first_selected as usize..=last_selected as usize].to_vec();
            chain.verts = trimmed;
        }
        debug_assert!(!chain.verts.is_empty());
    }
}

/// `calculate_curve_distances`.
fn calculate_curve_distances(
    bm: &BMesh,
    verts: &[usize],
    knot_indices: &[usize],
    point_indices: &[usize],
    is_closed: bool,
    regular: bool,
    r_knot_distances: &mut Vec<f32>,
    r_point_distances: &mut Vec<f32>,
) {
    let verts_num = verts.len();
    let positions: Vec<Vec3> = verts.iter().map(|&v| bm.vert(v).co).collect();
    let cumulative_len = verts_num + is_closed as usize;
    let mut cumulative = vec![0.0f32; cumulative_len];
    for i in 0..verts_num - 1 + is_closed as usize {
        cumulative[i + 1] = cumulative[i]
            + (positions[(i + 1) % verts_num] - positions[i]).length();
    }
    *r_knot_distances = vec![0.0; knot_indices.len() + is_closed as usize];
    *r_point_distances = vec![0.0; point_indices.len()];
    let mut knot_index = 0;
    let mut point_index = 0;
    for i in 0..verts_num {
        if knot_index < knot_indices.len() && knot_indices[knot_index] == i {
            r_knot_distances[knot_index] = cumulative[i];
            knot_index += 1;
        } else {
            r_point_distances[point_index] = cumulative[i];
            point_index += 1;
        }
    }
    if is_closed {
        let total_length = *cumulative.last().unwrap();
        r_knot_distances[knot_index] = total_length;
    }
    if regular {
        let total = *cumulative.last().unwrap();
        if total > CURVE_EPSILON {
            let spacing = total / (if is_closed { verts_num } else { verts_num - 1 }) as f32;
            for (i, &v_index) in point_indices.iter().enumerate() {
                r_point_distances[i] = v_index as f32 * spacing;
            }
            for ki in 0..knot_indices.len() {
                r_knot_distances[ki] = knot_indices[ki] as f32 * spacing;
            }
            if is_closed {
                r_knot_distances[knot_indices.len()] = total;
            }
        }
    }
}

/// `calculate_curve_splines`.
fn calculate_curve_splines(
    knot_positions: &[Vec3],
    knot_distances: &[f32],
    is_closed: bool,
) -> [Vec<SplineCoeffs>; 3] {
    let knots_num = knot_positions.len();
    let mut coords = [vec![0.0f32; knots_num], vec![0.0; knots_num], vec![0.0; knots_num]];
    for i in 0..knots_num {
        coords[0][i] = knot_positions[i].x;
        coords[1][i] = knot_positions[i].y;
        coords[2][i] = knot_positions[i].z;
    }
    [
        calculate_splines_axis(knot_distances, &coords[0], is_closed),
        calculate_splines_axis(knot_distances, &coords[1], is_closed),
        calculate_splines_axis(knot_distances, &coords[2], is_closed),
    ]
}

/// `execute_curve_chain`.
#[allow(clippy::too_many_arguments)]
fn execute_curve_chain(
    bm: &mut BMesh,
    input_chain: &[usize],
    mut is_closed: bool,
    interpolation: i32,
    clamp_elevation: i32,
    regular: bool,
    factor: f32,
    lock_x: bool,
    lock_y: bool,
    lock_z: bool,
) {
    let mut verts_num = input_chain.len();
    let mut new_start_index = 0usize;
    if is_closed {
        let (longest_gap, best_gap_start) =
            find_longest_unselected_gap(bm, input_chain);
        let offset = verts_num as i32 / 4;
        /* For a closed circle, if more than 50 percent of the vertices in the circle
         * are unselected, in order to prevent a total collapse from happening, the
         * tool's influence is restricted to 25 percent of the total vertices on
         * either side of the selection. */
        if longest_gap > 2 * offset {
            is_closed = false;
            let trim_start = mod_periodic(
                best_gap_start + longest_gap - offset,
                verts_num as i32,
            );
            let trim_end = mod_periodic(best_gap_start - 1 + offset, verts_num as i32);
            new_start_index = trim_start;
            verts_num =
                (trim_end as i32 - trim_start as i32).rem_euclid(verts_num as i32) as usize + 1;
        } else {
            for (i, &v) in input_chain.iter().enumerate() {
                if bm.vert(v).head.hflag & BM_ELEM_TAG != 0 {
                    new_start_index = i;
                    break;
                }
            }
        }
    }
    let verts: Vec<usize> = (0..verts_num)
        .map(|i| input_chain[mod_periodic((i + new_start_index) as i32, input_chain.len() as i32)])
        .collect();
    let mut knot_indices: Vec<usize> = verts
        .iter()
        .enumerate()
        .filter(|(_, &v)| bm.vert(v).head.hflag & BM_ELEM_TAG != 0)
        .map(|(i, _)| i)
        .collect();
    if !is_closed {
        if knot_indices.is_empty() || knot_indices[0] != 0 {
            knot_indices.insert(0, 0);
        }
        if *knot_indices.last().unwrap() != verts_num - 1 {
            knot_indices.push(verts_num - 1);
        }
    }
    let knot_positions: Vec<Vec3> =
        knot_indices.iter().map(|&ki| bm.vert(verts[ki]).co).collect();
    let knot_set: HashSet<usize> = knot_indices.iter().copied().collect();
    let point_indices: Vec<usize> =
        (0..verts_num).filter(|i| !knot_set.contains(i)).collect();
    let mut knot_distances = Vec::new();
    let mut point_distances = Vec::new();
    calculate_curve_distances(
        bm,
        &verts,
        &knot_indices,
        &point_indices,
        is_closed,
        regular,
        &mut knot_distances,
        &mut point_distances,
    );
    if point_indices.is_empty() {
        return;
    }
    let points_num = point_indices.len();
    let mut new_positions = vec![Vec3::ZERO; points_num];
    let (segment_indices, factors) = blenders_bli::length_parameterize::sample_at_lengths(
        &knot_distances[1..],
        &point_distances,
    );
    if interpolation == CURVE_INTERP_CUBIC {
        let axis_coeffs = calculate_curve_splines(&knot_positions, &knot_distances, is_closed);
        for i in 0..points_num {
            let seg = segment_indices[i];
            let dt = point_distances[i] - axis_coeffs[0][seg].x;
            let cx = &axis_coeffs[0][seg];
            let cy = &axis_coeffs[1][seg];
            let cz = &axis_coeffs[2][seg];
            new_positions[i] = Vec3::new(
                cx.a + dt * (cx.b + dt * (cx.c + dt * cx.d)),
                cy.a + dt * (cy.b + dt * (cy.c + dt * cy.d)),
                cz.a + dt * (cz.b + dt * (cz.c + dt * cz.d)),
            );
        }
    } else if interpolation == CURVE_INTERP_LINEAR {
        for i in 0..points_num {
            let si = segment_indices[i];
            new_positions[i] = knot_positions[si].lerp(knot_positions[si + 1], factors[i]);
        }
    }
    for (i, &v_index) in point_indices.iter().enumerate() {
        let vert = verts[v_index];
        let mut new_pos = new_positions[i];
        let old_pos = bm.vert(vert).co;
        let delta = new_pos - old_pos;
        /* Wire vertices have no face normal to clamp against. */
        if clamp_elevation != CURVE_CLAMP_ELEVATION_NONE && !crate::query::bm_vert_is_wire(bm, vert) {
            if delta.length_squared() > CURVE_EPSILON * CURVE_EPSILON {
                let normal = bm.vert(vert).no;
                let dot_val = delta.dot(normal);
                if clamp_elevation == CURVE_CLAMP_ELEVATION_RAISE {
                    if dot_val < -CURVE_EPSILON {
                        continue;
                    }
                } else if clamp_elevation == CURVE_CLAMP_ELEVATION_LOWER && dot_val > CURVE_EPSILON {
                    continue;
                }
            }
        }
        if lock_x {
            new_pos.x = old_pos.x;
        }
        if lock_y {
            new_pos.y = old_pos.y;
        }
        if lock_z {
            new_pos.z = old_pos.z;
        }
        bm.vpool[vert].co = old_pos.lerp(new_pos, factor);
    }
}

/// `bmo_curve_edge_loops_between_selected_exec`.
pub fn bmo_curve_edge_loops_between_selected_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let factor = bmo_slot_float_get(op, false, "factor");
    let clamp_elevation = bmo_slot_int_get(op, false, "clamp_elevation");
    let extend_loop = bmo_slot_bool_get(op, false, "extend_loop");
    let regular = bmo_slot_bool_get(op, false, "regular");
    let lock_x = bmo_slot_bool_get(op, false, "lock_x");
    let lock_y = bmo_slot_bool_get(op, false, "lock_y");
    let lock_z = bmo_slot_bool_get(op, false, "lock_z");
    let interpolation = bmo_slot_int_get(op, false, "interpolation");

    bmo_slot_buffer_hflag_enable(bm, op, false, "geom", BM_VERT, BM_ELEM_TAG, false);

    let mut chains = calc_curve_input_chains_from_bm(bm);
    if !extend_loop {
        trim_chains_to_boundaries(bm, &mut chains);
    }
    for chain in &chains {
        execute_curve_chain(
            bm,
            &chain.verts,
            chain.is_closed,
            interpolation,
            clamp_elevation,
            regular,
            factor,
            lock_x,
            lock_y,
            lock_z,
        );
    }
}
