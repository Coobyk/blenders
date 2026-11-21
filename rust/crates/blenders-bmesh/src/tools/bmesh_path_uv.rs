//! Port of `bmesh_path_uv.cc` — Dijkstra shortest-path over UV-topology
//! (vert/edge/face variants of `BM_mesh_calc_path_uv_*`).
//! `cd_loop_uv_offset` CustomData access is replaced by a
//! `uv_at: impl Fn(&BMesh, usize) -> Vec2` accessor (same convention as
//! `query_uv.rs`) until CustomData layers land.

use crate::iterators::*;
use crate::{BMesh, BM_NIL};
use crate::{BM_ELEM_TAG, BM_FACE};
use blenders_bli::heap_simple::HeapSimple;
use blenders_math::vector::Vec2;

const COST_INIT_MAX: f32 = f32::MAX;

/// `BMCalcPathUVParams`.
#[derive(Clone, Copy)]
pub struct BMCalcPathUVParams {
    /// If `use_topology_distance`, this is calculated on the fly.
    pub aspect_y: f32,
    /// Use face-center to step over (avoids creating a path that zig-zags
    /// back and forth over a single continuous string of vertices).
    pub use_step_face: bool,
    /// Only calculate the distance based on the topology.
    pub use_topology_distance: bool,
    /* `cd_loop_uv_offset` replaced by `uv_at` accessor (PORT_PENDING
     * CustomData). */
}

/// `step_cost_3_v2_ex`.
fn step_cost_3_v2_ex(v1: Vec2, v2: Vec2, v3: Vec2, skip_12: bool, skip_23: bool) -> f32 {
    let mut d1 = v2 - v1;
    let mut d2 = v3 - v2;
    let cost_12 = d1.length();
    let cost_23 = d2.length();
    if cost_12 > 0.0 {
        d1 /= cost_12;
    }
    if cost_23 > 0.0 {
        d2 /= cost_23;
    }
    let cost = (if skip_12 { 0.0 } else { cost_12 }) + (if skip_23 { 0.0 } else { cost_23 });
    cost * (1.0 + 0.5 * (2.0 - d1.dot(d2).abs().sqrt()))
}
/// `step_cost_3_v2`.
fn step_cost_3_v2(v1: Vec2, v2: Vec2, v3: Vec2) -> f32 {
    step_cost_3_v2_ex(v1, v2, v3, false, false)
}

/// `verttag_add_adjacent_uv`.
fn verttag_add_adjacent_uv(
    bm: &mut BMesh,
    heap: &mut HeapSimple<usize>,
    l_a: usize,
    loops_prev: &mut [usize],
    cost: &mut [f32],
    params: &BMCalcPathUVParams,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) {
    debug_assert_ne!(params.aspect_y, 0.0);
    let l_a_index = bm.loo(l_a).head.index as usize;
    let luv_a = uv_at(bm, l_a);
    let uv_a = Vec2::new(luv_a.x, luv_a.y / params.aspect_y);
    for l in LoopsOfVert::new(bm, bm.loo(l_a).v).collect::<Vec<_>>() {
        let luv = uv_at(bm, l);
        if luv_a == luv {
            bm.lpool[l].head.hflag |= BM_ELEM_TAG;
            let mut l_b = bm.loo(l).next;
            loop {
                if bm.loo(l_b).head.hflag & BM_ELEM_TAG == 0 {
                    let luv_b = uv_at(bm, l_b);
                    let uv_b = Vec2::new(luv_b.x, luv_b.y / params.aspect_y);
                    let l_b_index = bm.loo(l_b).head.index as usize;
                    let cost_cut = if params.use_topology_distance {
                        1.0
                    } else {
                        (uv_a - uv_b).length()
                    };
                    let cost_new = cost[l_a_index] + cost_cut;
                    if cost[l_b_index] > cost_new {
                        cost[l_b_index] = cost_new;
                        loops_prev[l_b_index] = l_a;
                        heap.insert(cost_new, l_b);
                    }
                }
                if !params.use_step_face {
                    if l_b == bm.loo(l).next {
                        l_b = bm.loo(bm.loo(l).prev).prev;
                    }
                }
                l_b = bm.loo(l_b).next;
                if l_b == l {
                    break;
                }
            }
        }
    }
}

/// `BM_mesh_calc_path_uv_vert` → Vec of loop indices (path), empty if none.
pub fn bm_mesh_calc_path_uv_vert(
    bm: &mut BMesh,
    l_src: usize,
    l_dst: usize,
    params: &BMCalcPathUVParams,
    filter_fn: &dyn Fn(&BMesh, usize) -> bool,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) -> Vec<usize> {
    /* tag loop elems, assign index */
    let mut i = 0i32;
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            let ok = filter_fn(bm, l_iter);
            if ok {
                bm.lpool[l_iter].head.hflag &= !BM_ELEM_TAG;
            } else {
                bm.lpool[l_iter].head.hflag |= BM_ELEM_TAG;
            }
            bm.lpool[l_iter].head.index = i;
            i += 1;
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
    let totloop = bm.totloop as usize;
    let mut loops_prev = vec![BM_NIL; totloop];
    let mut cost = vec![COST_INIT_MAX; totloop];
    let mut heap: HeapSimple<usize> = HeapSimple::new();
    heap.insert(0.0, l_src);
    cost[bm.loo(l_src).head.index as usize] = 0.0;
    let mut l = BM_NIL;
    while !heap.is_empty() {
        l = heap.pop_min().unwrap();
        if bm.loo(l).v == bm.loo(l_dst).v
            && crate::query_uv::bm_loop_uv_share_vert_check(bm, l, l_dst, uv_at)
        {
            break;
        }
        if bm.loo(l).head.hflag & BM_ELEM_TAG == 0 {
            bm.lpool[l].head.hflag |= BM_ELEM_TAG;
            verttag_add_adjacent_uv(bm, &mut heap, l, &mut loops_prev, &mut cost, params, uv_at);
        }
    }
    let mut path = Vec::new();
    if l != BM_NIL
        && bm.loo(l).v == bm.loo(l_dst).v
        && crate::query_uv::bm_loop_uv_share_vert_check(bm, l, l_dst, uv_at)
    {
        loop {
            path.push(l);
            l = loops_prev[bm.loo(l).head.index as usize];
            if l == BM_NIL {
                break;
            }
        }
        path.reverse();
    }
    path
}

/// `edgetag_cut_cost_vert_uv`.
fn edgetag_cut_cost_vert_uv(
    bm: &BMesh,
    l_e_a: usize,
    l_e_b: usize,
    l_v: usize,
    aspect_y: f32,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) -> f32 {
    let l_v1 = if bm.loo(l_v).v == bm.loo(l_e_a).v {
        bm.loo(l_e_a).next
    } else {
        l_e_a
    };
    let l_v2 = if bm.loo(l_v).v == bm.loo(l_e_b).v {
        bm.loo(l_e_b).next
    } else {
        l_e_b
    };
    let luv_v1 = uv_at(bm, l_v1);
    let luv_v2 = uv_at(bm, l_v2);
    let luv_v = uv_at(bm, l_v);
    let uv_v1 = Vec2::new(luv_v1.x, luv_v1.y / aspect_y);
    let uv_v2 = Vec2::new(luv_v2.x, luv_v2.y / aspect_y);
    let uv_v = Vec2::new(luv_v.x, luv_v.y / aspect_y);
    step_cost_3_v2(uv_v1, uv_v, uv_v2)
}

/// `edgetag_cut_cost_face_uv`.
fn edgetag_cut_cost_face_uv(
    bm: &BMesh,
    l_e_a: usize,
    l_e_b: usize,
    f: usize,
    aspect_v2: [f32; 2],
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) -> f32 {
    let luv_e_a = uv_at(bm, l_e_a);
    let luv_e_b = uv_at(bm, l_e_b);
    let mut l_e_a_cent = (luv_e_a + luv_e_a) * 0.5;
    let mut l_e_b_cent = (luv_e_b + luv_e_b) * 0.5;
    l_e_a_cent = l_e_a_cent * Vec2::new(aspect_v2[0], aspect_v2[1]);
    l_e_b_cent = l_e_b_cent * Vec2::new(aspect_v2[0], aspect_v2[1]);
    let f_cent =
        crate::query_uv::bm_face_uv_calc_center_median_weighted(bm, f, aspect_v2, uv_at);
    step_cost_3_v2(l_e_a_cent, l_e_b_cent, f_cent)
}

/// `edgetag_add_adjacent_uv`.
fn edgetag_add_adjacent_uv(
    bm: &mut BMesh,
    heap: &mut HeapSimple<usize>,
    l_a: usize,
    loops_prev: &mut [usize],
    cost: &mut [f32],
    params: &BMCalcPathUVParams,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) {
    debug_assert_ne!(params.aspect_y, 0.0);
    let l_a_verts = [l_a, bm.loo(l_a).next];
    let l_a_index = bm.loo(l_a).head.index as usize;
    if !params.use_step_face {
        for &l_av in &l_a_verts {
            if loops_prev[l_a_index] != BM_NIL {
                let mut l_prev = loops_prev[l_a_index];
                let mut has_prev = true;
                if bm.loo(l_av).v != bm.loo(l_prev).v {
                    l_prev = if bm.loo(l_av).v == bm.loo(bm.loo(l_prev).next).v {
                        bm.loo(l_prev).next
                    } else {
                        has_prev = false;
                        BM_NIL
                    };
                }
                if has_prev
                    && crate::query_uv::bm_loop_uv_share_vert_check(bm, l_av, l_prev, uv_at)
                {
                    continue;
                }
            }
            for e_b in EdgesOfVert::new(bm, bm.loo(l_av).v).collect::<Vec<_>>() {
                if bm.edge(e_b).l == BM_NIL {
                    continue;
                }
                let l_first = bm.edge(e_b).l;
                let mut l_b = l_first;
                loop {
                    if bm.loo(l_b).head.hflag & BM_ELEM_TAG == 0 {
                        let l_b_vert = if bm.loo(l_av).v == bm.loo(l_b).v {
                            l_b
                        } else {
                            bm.loo(l_b).next
                        };
                        if crate::query_uv::bm_loop_uv_share_vert_check(
                            bm, l_av, l_b_vert, uv_at,
                        ) {
                            let l_b_index = bm.loo(l_b).head.index as usize;
                            let cost_cut = if params.use_topology_distance {
                                1.0
                            } else {
                                edgetag_cut_cost_vert_uv(
                                    bm,
                                    l_a,
                                    l_b,
                                    l_av,
                                    params.aspect_y,
                                    uv_at,
                                )
                            };
                            let cost_new = cost[l_a_index] + cost_cut;
                            if cost[l_b_index] > cost_new {
                                cost[l_b_index] = cost_new;
                                loops_prev[l_b_index] = l_a;
                                heap.insert(cost_new, l_b);
                            }
                        }
                    }
                    l_b = bm.loo(l_b).radial_next;
                    if l_b == l_first {
                        break;
                    }
                }
            }
        }
    } else {
        let aspect_v2 = [1.0f32, 1.0 / params.aspect_y];
        let l_first = l_a;
        let mut l_iter = l_first;
        loop {
            if !crate::query_uv::bm_loop_uv_share_edge_check(bm, l_a, l_iter, uv_at) {
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
                continue;
            }
            let mut l_cycle_iter = bm.loo(l_iter).next;
            let l_cycle_end = l_iter;
            loop {
                let l_b = l_cycle_iter;
                if bm.loo(l_b).head.hflag & BM_ELEM_TAG == 0 {
                    let l_b_index = bm.loo(l_b).head.index as usize;
                    let cost_cut = if params.use_topology_distance {
                        1.0
                    } else {
                        edgetag_cut_cost_face_uv(
                            bm,
                            l_a,
                            l_b,
                            bm.loo(l_iter).f,
                            aspect_v2,
                            uv_at,
                        )
                    };
                    let cost_new = cost[l_a_index] + cost_cut;
                    if cost[l_b_index] > cost_new {
                        cost[l_b_index] = cost_new;
                        loops_prev[l_b_index] = l_a;
                        heap.insert(cost_new, l_b);
                    }
                }
                l_cycle_iter = bm.loo(l_cycle_iter).next;
                if l_cycle_iter == l_cycle_end {
                    break;
                }
            }
            l_iter = bm.loo(l_iter).radial_next;
            if l_iter == l_first {
                break;
            }
        }
    }
}

/// `BM_mesh_calc_path_uv_edge`.
pub fn bm_mesh_calc_path_uv_edge(
    bm: &mut BMesh,
    l_src: usize,
    l_dst: usize,
    params: &BMCalcPathUVParams,
    filter_fn: &dyn Fn(&BMesh, usize) -> bool,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) -> Vec<usize> {
    let mut i = 0i32;
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            let ok = filter_fn(bm, l_iter);
            if ok {
                bm.lpool[l_iter].head.hflag &= !BM_ELEM_TAG;
            } else {
                bm.lpool[l_iter].head.hflag |= BM_ELEM_TAG;
            }
            bm.lpool[l_iter].head.index = i;
            i += 1;
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
    let totloop = bm.totloop as usize;
    let mut loops_prev = vec![BM_NIL; totloop];
    let mut cost = vec![COST_INIT_MAX; totloop];
    let mut heap: HeapSimple<usize> = HeapSimple::new();
    heap.insert(0.0, l_src);
    cost[bm.loo(l_src).head.index as usize] = 0.0;
    let mut l = BM_NIL;
    while !heap.is_empty() {
        l = heap.pop_min().unwrap();
        if bm.loo(l).e == bm.loo(l_dst).e
            && crate::query_uv::bm_loop_uv_share_edge_check(bm, l, l_dst, uv_at)
        {
            break;
        }
        if bm.loo(l).head.hflag & BM_ELEM_TAG == 0 {
            bm.lpool[l].head.hflag |= BM_ELEM_TAG;
            edgetag_add_adjacent_uv(bm, &mut heap, l, &mut loops_prev, &mut cost, params, uv_at);
        }
    }
    let mut path = Vec::new();
    if l != BM_NIL
        && bm.loo(l).e == bm.loo(l_dst).e
        && crate::query_uv::bm_loop_uv_share_edge_check(bm, l, l_dst, uv_at)
    {
        loop {
            path.push(l);
            l = loops_prev[bm.loo(l).head.index as usize];
            if l == BM_NIL {
                break;
            }
        }
        path.reverse();
    }
    path
}

/// `facetag_cut_cost_edge_uv`.
fn facetag_cut_cost_edge_uv(
    bm: &BMesh,
    f_a: usize,
    f_b: usize,
    l_edge: usize,
    f_endpoints: [usize; 2],
    aspect_v2: [f32; 2],
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) -> f32 {
    let mut f_a_cent =
        crate::query_uv::bm_face_uv_calc_center_median_weighted(bm, f_a, aspect_v2, uv_at);
    let mut f_b_cent =
        crate::query_uv::bm_face_uv_calc_center_median_weighted(bm, f_b, aspect_v2, uv_at);
    let co_v1 = uv_at(bm, l_edge);
    let co_v2 = uv_at(bm, bm.loo(l_edge).next);
    /* intersect edge segment with the line between face centers */
    let (_, ix_e) =
        blenders_math::geom::isect_line_line_v2_point(co_v1, co_v2, f_a_cent, f_b_cent);
    let factor = blenders_math::geom::line_point_factor_v2(ix_e, co_v1, co_v2);
    let mut e_cent = if factor < 0.0 {
        co_v1
    } else if factor > 1.0 {
        co_v2
    } else {
        ix_e
    };
    let av = Vec2::new(aspect_v2[0], aspect_v2[1]);
    f_a_cent = f_a_cent * av;
    f_b_cent = f_b_cent * av;
    e_cent = e_cent * av;
    step_cost_3_v2_ex(
        f_a_cent,
        e_cent,
        f_b_cent,
        f_a == f_endpoints[0],
        f_b == f_endpoints[1],
    )
}

/// `facetag_cut_cost_vert_uv`.
fn facetag_cut_cost_vert_uv(
    bm: &BMesh,
    f_a: usize,
    f_b: usize,
    l_vert: usize,
    f_endpoints: [usize; 2],
    aspect_v2: [f32; 2],
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) -> f32 {
    let mut f_a_cent =
        crate::query_uv::bm_face_uv_calc_center_median_weighted(bm, f_a, aspect_v2, uv_at);
    let mut f_b_cent =
        crate::query_uv::bm_face_uv_calc_center_median_weighted(bm, f_b, aspect_v2, uv_at);
    let mut v_cent = uv_at(bm, l_vert);
    let av = Vec2::new(aspect_v2[0], aspect_v2[1]);
    f_a_cent = f_a_cent * av;
    f_b_cent = f_b_cent * av;
    v_cent = v_cent * av;
    step_cost_3_v2_ex(
        f_a_cent,
        v_cent,
        f_b_cent,
        f_a == f_endpoints[0],
        f_b == f_endpoints[1],
    )
}

/// `facetag_add_adjacent_uv`.
fn facetag_add_adjacent_uv(
    bm: &mut BMesh,
    heap: &mut HeapSimple<usize>,
    f_a: usize,
    faces_prev: &mut [usize],
    cost: &mut [f32],
    f_endpoints: [usize; 2],
    aspect_v2: [f32; 2],
    params: &BMCalcPathUVParams,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) {
    let f_a_index = bm.face(f_a).head.index as usize;
    {
        /* edge adjacent faces */
        for l_a in LoopsOfFace::new(bm, f_a).collect::<Vec<_>>() {
            if l_a != bm.loo(l_a).radial_next {
                let l_first = bm.loo(l_a).radial_next;
                let mut l_iter = l_first;
                loop {
                    let f_b = bm.loo(l_iter).f;
                    if bm.face(f_b).head.hflag & BM_ELEM_TAG == 0 {
                        if crate::query_uv::bm_loop_uv_share_edge_check(
                            bm, l_a, l_iter, uv_at,
                        ) {
                            let f_b_index = bm.face(f_b).head.index as usize;
                            let cost_cut = if params.use_topology_distance {
                                1.0
                            } else {
                                facetag_cut_cost_edge_uv(
                                    bm,
                                    f_a,
                                    f_b,
                                    l_iter,
                                    f_endpoints,
                                    aspect_v2,
                                    uv_at,
                                )
                            };
                            let cost_new = cost[f_a_index] + cost_cut;
                            if cost[f_b_index] > cost_new {
                                cost[f_b_index] = cost_new;
                                faces_prev[f_b_index] = f_a;
                                heap.insert(cost_new, f_b);
                            }
                        }
                    }
                    l_iter = bm.loo(l_iter).radial_next;
                    if l_iter == l_first {
                        break;
                    }
                }
            }
        }
    }
    if params.use_step_face {
        for l_a in LoopsOfFace::new(bm, f_a).collect::<Vec<_>>() {
            for l_b in LoopsOfVert::new(bm, bm.loo(l_a).v).collect::<Vec<_>>() {
                if l_a != l_b && !crate::query::bm_loop_share_edge_check(bm, l_a, l_b) {
                    let f_b = bm.loo(l_b).f;
                    if bm.face(f_b).head.hflag & BM_ELEM_TAG == 0 {
                        if crate::query_uv::bm_loop_uv_share_vert_check(bm, l_a, l_b, uv_at)
                        {
                            let f_b_index = bm.face(f_b).head.index as usize;
                            let cost_cut = if params.use_topology_distance {
                                1.0
                            } else {
                                facetag_cut_cost_vert_uv(
                                    bm,
                                    f_a,
                                    f_b,
                                    l_a,
                                    f_endpoints,
                                    aspect_v2,
                                    uv_at,
                                )
                            };
                            let cost_new = cost[f_a_index] + cost_cut;
                            if cost[f_b_index] > cost_new {
                                cost[f_b_index] = cost_new;
                                faces_prev[f_b_index] = f_a;
                                heap.insert(cost_new, f_b);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `BM_mesh_calc_path_uv_face`.
pub fn bm_mesh_calc_path_uv_face(
    bm: &mut BMesh,
    f_src: usize,
    f_dst: usize,
    params: &BMCalcPathUVParams,
    filter_fn: &dyn Fn(&BMesh, usize) -> bool,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) -> Vec<usize> {
    let aspect_v2 = [1.0f32, 1.0 / params.aspect_y];
    let f_endpoints = [f_src, f_dst];
    /* tag face elems, assign index */
    let mut i = 0i32;
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        let ok = filter_fn(bm, f);
        if ok {
            bm.fpool[f].head.hflag &= !BM_ELEM_TAG;
        } else {
            bm.fpool[f].head.hflag |= BM_ELEM_TAG;
        }
        bm.fpool[f].head.index = i;
        i += 1;
    }
    let totface = bm.totface as usize;
    let mut faces_prev = vec![BM_NIL; totface];
    let mut cost = vec![COST_INIT_MAX; totface];
    let mut heap: HeapSimple<usize> = HeapSimple::new();
    heap.insert(0.0, f_src);
    cost[bm.face(f_src).head.index as usize] = 0.0;
    let mut f = BM_NIL;
    while !heap.is_empty() {
        f = heap.pop_min().unwrap();
        if f == f_dst {
            break;
        }
        if bm.face(f).head.hflag & BM_ELEM_TAG == 0 {
            bm.fpool[f].head.hflag |= BM_ELEM_TAG;
            facetag_add_adjacent_uv(
                bm,
                &mut heap,
                f,
                &mut faces_prev,
                &mut cost,
                f_endpoints,
                aspect_v2,
                params,
                uv_at,
            );
        }
    }
    let mut path = Vec::new();
    if f == f_dst {
        loop {
            path.push(f);
            f = faces_prev[bm.face(f).head.index as usize];
            if f == BM_NIL {
                break;
            }
        }
        path.reverse();
    }
    path
}
