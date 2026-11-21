//! Port of `bmo_fill_grid.cc` — `bmo_grid_fill_exec` (BARYCENTRIC_INTERP
//! + USE_FLIP_DETECT both enabled as in C).

use std::collections::HashSet;
use std::collections::VecDeque;

use crate::edgeloop::*;
use crate::iterators::*;
use crate::operators::*;
use crate::{BMesh, BM_NIL};
use crate::{BM_EDGE, BM_FACE, BM_VERT};
use crate::{BM_ELEM_HIDDEN, BM_ELEM_SMOOTH, BM_ELEM_TAG};
use blenders_math::vector::{Vec2, Vec3};

const EDGE_MARK: u8 = 4;
const FACE_OUT: u8 = 16;

fn quad_edges_to_normal(co_a1: Vec3, co_a2: Vec3, co_b1: Vec3, co_b2: Vec3) -> Vec3 {
    let diff_a = (co_a2 - co_a1).normalized();
    let diff_b = (co_b2 - co_b1).normalized();
    (diff_a + diff_b).normalized()
}

/// `quad_verts_to_barycentric_tri` — `tri` is `[Vec3; 3]` out.
fn quad_verts_to_barycentric_tri(
    co_a: Vec3,
    co_b: Vec3,
    co_a_next: Vec3,
    co_b_next: Vec3,
    prev: Option<(Vec3, Vec3)>,
    is_flip: bool,
) -> [Vec3; 3] {
    let mut tri = [co_a, co_b, Vec3::ZERO];
    let mut no = quad_edges_to_normal(co_a, co_a_next, co_b, co_b_next);
    if let Some((co_a_prev, co_b_prev)) = prev {
        let no_t = quad_edges_to_normal(co_a_prev, co_a, co_b_prev, co_b);
        no = (no + no_t).normalized();
    }
    if is_flip {
        no = -no;
    }
    let no = no * tri[0].distance(tri[1]);
    tri[2] = (tri[0] + tri[1]) * 0.5 + no;
    tri
}

/// `bm_loop_pair_from_verts` — `(l_a, l_b)` along edge, or `(BM_NIL, BM_NIL)`.
fn bm_loop_pair_from_verts(bm: &BMesh, v_a: usize, v_b: usize) -> (usize, usize) {
    let e = crate::query::bm_edge_exists(bm, v_a, v_b);
    if e == BM_NIL || bm.edge(e).l == BM_NIL {
        return (BM_NIL, BM_NIL);
    }
    let l = bm.edge(e).l;
    if bm.loo(l).v == v_a {
        (l, bm.loo(l).next)
    } else {
        (bm.loo(l).next, l)
    }
}

/// `bm_loop_pair_test_copy` — mirror one side's missing pair.
fn bm_loop_pair_test_copy(a: &mut (usize, usize), b: &mut (usize, usize)) {
    if a.0 != BM_NIL && b.0 == BM_NIL {
        *b = (a.1, a.0);
    } else if b.0 != BM_NIL && a.0 == BM_NIL {
        *a = (b.1, b.0);
    }
}

/// `barycentric_weights_v2_grid_cache` → flat `xtot*ytot` table of `[f32;4]`.
fn barycentric_weights_v2_grid_cache(xtot: usize, ytot: usize) -> Vec<[f32; 4]> {
    let x_step = 1.0 / (xtot - 1) as f32;
    let y_step = 1.0 / (ytot - 1) as f32;
    let mut table = Vec::with_capacity(xtot * ytot);
    for y in 0..ytot {
        for x in 0..xtot {
            let xy = Vec2::new(x_step * x as f32, y_step * y as f32);
            table.push(blenders_math::geom::barycentric_weights_v2_quad(
                Vec2::new(xy.x, 0.0),
                Vec2::new(0.0, xy.y),
                Vec2::new(xy.x, 1.0),
                Vec2::new(1.0, xy.y),
                xy,
            ));
        }
    }
    table
}

/// `bm_grid_fill_array` — fill the interior of `v_grid` and create the quads.
/// Loop/vert CustomData interpolation is PORT_PENDING (`larr_*` built anyway
/// for fidelity but interpolation bodies are no-ops).
#[allow(clippy::too_many_arguments)]
fn bm_grid_fill_array(
    bm: &mut BMesh,
    v_grid: &mut [usize],
    xtot: usize,
    ytot: usize,
    mat_nr: i16,
    use_smooth: bool,
    use_flip: bool,
    use_interp_simple: bool,
) {
    let xy = |x: usize, y: usize| x + y * xtot;

    /* PORT_PENDING: `use_vert_interp`/`use_loop_interp` need CustomData. */
    let weight_table = if use_interp_simple {
        Some(barycentric_weights_v2_grid_cache(xtot, ytot))
    } else {
        None
    };
    let tri_a = quad_verts_to_barycentric_tri(
        bm.vpool[v_grid[xy(0, 0)]].co,
        bm.vpool[v_grid[xy(xtot - 1, 0)]].co,
        bm.vpool[v_grid[xy(0, 1)]].co,
        bm.vpool[v_grid[xy(xtot - 1, 1)]].co,
        None,
        false,
    );
    let tri_b = quad_verts_to_barycentric_tri(
        bm.vpool[v_grid[xy(0, ytot - 1)]].co,
        bm.vpool[v_grid[xy(xtot - 1, ytot - 1)]].co,
        bm.vpool[v_grid[xy(0, ytot - 2)]].co,
        bm.vpool[v_grid[xy(xtot - 1, ytot - 2)]].co,
        None,
        true,
    );

    for y in 1..ytot - 1 {
        let tri_t = quad_verts_to_barycentric_tri(
            bm.vpool[v_grid[xy(0, y)]].co,
            bm.vpool[v_grid[xy(xtot - 1, y)]].co,
            bm.vpool[v_grid[xy(0, y + 1)]].co,
            bm.vpool[v_grid[xy(xtot - 1, y + 1)]].co,
            Some((bm.vpool[v_grid[xy(0, y - 1)]].co,
                  bm.vpool[v_grid[xy(xtot - 1, y - 1)]].co)),
            false,
        );
        for x in 1..xtot - 1 {
            debug_assert_eq!(v_grid[xy(x, y)], BM_NIL);
            let co = if !use_interp_simple {
                let co_a = blenders_math::geom::transform_point_by_tri(
                    bm.vpool[v_grid[x]].co, &tri_t, &tri_a);
                let co_b = blenders_math::geom::transform_point_by_tri(
                    bm.vpool[v_grid[xy(x, ytot - 1)]].co, &tri_t, &tri_b);
                co_a.lerp(co_b, y as f32 / (ytot - 1) as f32)
            } else {
                let w = weight_table.as_ref().unwrap()[xy(x, y)];
                bm.vpool[v_grid[xy(x, 0)]].co * w[0]
                    + bm.vpool[v_grid[xy(0, y)]].co * w[1]
                    + bm.vpool[v_grid[xy(x, ytot - 1)]].co * w[2]
                    + bm.vpool[v_grid[xy(xtot - 1, y)]].co * w[3]
            };
            v_grid[xy(x, y)] = crate::core::bm_vert_create(bm, co, None, 0);
            /* PORT_PENDING: vert CustomData interp with `w`. */
        }
    }

    for x in 0..xtot - 1 {
        for y in 0..ytot - 1 {
            let f = if use_flip {
                crate::construct::bm_face_create_quad_tri(
                    bm,
                    v_grid[xy(x, y)],
                    v_grid[xy(x, y + 1)],
                    v_grid[xy(x + 1, y + 1)],
                    v_grid[xy(x + 1, y)],
                    None,
                    0,
                )
            } else {
                crate::construct::bm_face_create_quad_tri(
                    bm,
                    v_grid[xy(x + 1, y)],
                    v_grid[xy(x + 1, y + 1)],
                    v_grid[xy(x, y + 1)],
                    v_grid[xy(x, y)],
                    None,
                    0,
                )
            };
            /* PORT_PENDING: loop interp from boundary loops (`larr_*`,
             * `bm_loop_interp_from_grid_boundary_4/_2`). */
            bmo_elem_flag_enable(bm, (BM_FACE, f), FACE_OUT);
            bm.fpool[f].mat_nr = mat_nr;
            if use_smooth {
                bm.fpool[f].head.hflag |= BM_ELEM_SMOOTH;
            }
        }
    }
}

fn bm_grid_fill(
    bm: &mut BMesh,
    estore_a: &BMEdgeLoopStore,
    estore_b: &BMEdgeLoopStore,
    estore_rail_a: &BMEdgeLoopStore,
    estore_rail_b: &BMEdgeLoopStore,
    mat_nr: i16,
    use_smooth: bool,
    use_interp_simple: bool,
) {
    let xtot = bm_edgeloop_length_get(estore_a) as usize;
    let ytot = bm_edgeloop_length_get(estore_rail_a) as usize;
    let lb_a = bm_edgeloop_verts_get(estore_a);
    let lb_b = bm_edgeloop_verts_get(estore_b);
    let lb_rail_a = bm_edgeloop_verts_get(estore_rail_a);
    let lb_rail_b = bm_edgeloop_verts_get(estore_rail_b);
    debug_assert_eq!(lb_a.front(), lb_rail_a.front()); /* BL */
    debug_assert_eq!(lb_b.front(), lb_rail_a.back()); /* TL */
    debug_assert_eq!(lb_b.back(), lb_rail_b.back()); /* TR */
    debug_assert_eq!(lb_a.back(), lb_rail_b.front()); /* BR */

    let mut v_grid = vec![BM_NIL; xtot * ytot];
    for (i, &v) in lb_a.iter().enumerate() {
        v_grid[i] = v;
    }
    for (i, &v) in lb_b.iter().enumerate() {
        v_grid[ytot * xtot - xtot + i] = v;
    }
    for (i, &v) in lb_rail_a.iter().enumerate() {
        v_grid[xtot * i] = v;
    }
    for (i, &v) in lb_rail_b.iter().enumerate() {
        v_grid[xtot * i + (xtot - 1)] = v;
    }

    /* USE_FLIP_DETECT: winding votes over the four rails. */
    let lbs: [&VecDeque<usize>; 4] = [lb_a, lb_b, lb_rail_a, lb_rail_b];
    let lb_iter_dir = [-1, 1, 1, -1];
    let mut winding_votes = 0;
    for i in 0..4 {
        for j in 0..lbs[i].len().saturating_sub(1) {
            let e = crate::query::bm_edge_exists(bm, lbs[i][j], lbs[i][j + 1]);
            if e != BM_NIL && crate::delete::bm_edge_is_boundary(bm, e) {
                winding_votes += if bm.loo(bm.edge(e).l).v == lbs[i][j] {
                    lb_iter_dir[i]
                } else {
                    -lb_iter_dir[i]
                };
            }
        }
    }
    let use_flip = winding_votes < 0;

    bm_grid_fill_array(
        bm, &mut v_grid, xtot, ytot, mat_nr, use_smooth, use_flip, use_interp_simple);
}

/// `bm_edgeloop_flag_set` — set `hflag` on all edges along the loop.
fn bm_edgeloop_flag_set(bm: &mut BMesh, estore: &BMEdgeLoopStore, hflag: u8, set: bool) {
    let verts = bm_edgeloop_verts_get(estore).clone();
    for j in 1..verts.len() {
        let e = crate::query::bm_edge_exists(bm, verts[j], verts[j - 1]);
        if e != BM_NIL {
            if set {
                bm.epool[e].head.hflag |= hflag;
            } else {
                bm.epool[e].head.hflag &= !hflag;
            }
        }
    }
}

/// `bmo_grid_fill_exec`.
pub fn bmo_grid_fill_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mat_nr = bmo_slot_int_get(op, false, "mat_nr") as i16;
    let use_smooth = bmo_slot_bool_get(op, false, "use_smooth");
    let use_interp_simple = bmo_slot_bool_get(op, false, "use_interp_simple");
    let mut split_edges: HashSet<usize> = HashSet::new();
    let mut changed = false;

    bmo_slot_buffer_flag_enable(bm, op, false, "edges", EDGE_MARK);
    let eloops = bm_mesh_edgeloops_find(
        bm,
        &|bm: &BMesh, e| bmo_elem_flag_test(bm, (BM_EDGE, e), EDGE_MARK),
        None,
    );
    if eloops.len() != 2 {
        /* BMO_ERROR_CANCEL: "Select two edge loops or a single closed edge
         * loop from which two edge loops can be calculated" */
        return;
    }
    let mut estore_a = bm_edgeloop_copy(&eloops[0]);
    let mut estore_b = bm_edgeloop_copy(&eloops[1]);
    let v_a_first = *bm_edgeloop_verts_get(&estore_a).front().unwrap();
    let v_a_last = *bm_edgeloop_verts_get(&estore_a).back().unwrap();
    let v_b_first = *bm_edgeloop_verts_get(&estore_b).front().unwrap();
    let v_b_last = *bm_edgeloop_verts_get(&estore_b).back().unwrap();
    if bm_edgeloop_is_closed(&estore_a) || bm_edgeloop_is_closed(&estore_b) {
        /* BMO_ERROR_CANCEL: "Closed loops unsupported" */
        return;
    }

    let edge_test_rail = |bm: &BMesh, e: usize| {
        if bm.edge(e).head.hflag & BM_ELEM_HIDDEN != 0 {
            return false;
        }
        bm.edge(e).l == BM_NIL || crate::delete::bm_edge_is_boundary(bm, e)
    };

    bm_edgeloop_flag_set(bm, &estore_a, BM_ELEM_HIDDEN, true);
    bm_edgeloop_flag_set(bm, &estore_b, BM_ELEM_HIDDEN, true);
    let mut eloops_rail: Vec<BMEdgeLoopStore> = Vec::new();
    let mut flipped = false;
    if let Some(rail_a) = bm_mesh_edgeloops_find_path(
        bm, Some(&edge_test_rail), v_a_first, v_b_first)
    {
        if let Some(rail_b) = bm_mesh_edgeloops_find_path(
            bm, Some(&edge_test_rail), v_a_last, v_b_last)
        {
            eloops_rail = vec![rail_a, rail_b];
        }
    }
    if eloops_rail.is_empty() {
        if let Some(rail_a) = bm_mesh_edgeloops_find_path(
            bm, Some(&edge_test_rail), v_a_first, v_b_last)
        {
            if let Some(rail_b) = bm_mesh_edgeloops_find_path(
                bm, Some(&edge_test_rail), v_a_last, v_b_first)
            {
                eloops_rail = vec![rail_a, rail_b];
                flipped = true;
            }
        }
    }
    bm_edgeloop_flag_set(bm, &estore_a, BM_ELEM_HIDDEN, false);
    bm_edgeloop_flag_set(bm, &estore_b, BM_ELEM_HIDDEN, false);
    if flipped {
        bm_edgeloop_flip(&mut estore_b);
    }
    if eloops_rail.is_empty() {
        /* BMO_ERROR_CANCEL: "Loops are not connected by wire/boundary edges" */
        return;
    }
    debug_assert!(v_a_last != v_b_last);
    if bm_edgeloop_overlap_check(bm, &eloops_rail[0], &eloops_rail[1]) {
        /* BMO_ERROR_CANCEL: "Connecting edge loops overlap" */
        return;
    }

    /* equalize opposite rail lengths via `bm_edgeloop_expand`. */
    let mut rails = [bm_edgeloop_copy(&estore_a), bm_edgeloop_copy(&estore_b),
                     bm_edgeloop_copy(&eloops_rail[0]), bm_edgeloop_copy(&eloops_rail[1])];
    for i in 0..2 {
        let len_a = bm_edgeloop_length_get(&rails[i * 2]);
        let len_b = bm_edgeloop_length_get(&rails[i * 2 + 1]);
        if len_a != len_b {
            if len_a < len_b {
                bm_edgeloop_expand(bm, &mut rails[i * 2], len_b, true, &mut split_edges);
            } else {
                bm_edgeloop_expand(bm, &mut rails[i * 2 + 1], len_a, true, &mut split_edges);
            }
        }
    }
    bm_grid_fill(
        bm, &rails[0], &rails[1], &rails[2], &rails[3],
        mat_nr, use_smooth, use_interp_simple);
    changed = true;
    for e in split_edges {
        let v2 = bm.edge(e).v2;
        crate::mods::bm_edge_collapse(bm, e, v2, true, true);
    }
    if changed {
        bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", BM_FACE, FACE_OUT);
    }
}
