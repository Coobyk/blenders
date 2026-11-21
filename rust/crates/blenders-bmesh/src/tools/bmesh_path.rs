//! Port of `bmesh_path.cc` — Dijkstra shortest path between 2 elements.
//! All 3 variants (vert/edge/face). `elem_index` scratch → HashMap side-tables.

use crate::query::bm_loop_share_edge_check;
use crate::{BMesh, BM_ELEM_TAG};
use blenders_bli::heap_simple::HeapSimple;
use blenders_math::geom::{isect_line_line, line_point_factor};
use blenders_math::vector::Vec3;
use std::collections::{HashMap, HashSet};

const COST_INIT_MAX: f32 = f32::MAX;

/// `BMCalcPathParams`.
#[derive(Default)]
pub struct BMCalcPathParams {
    /// `use_step_face`: for verts allows stepping across a face; for edges, only steps over faces.
    pub use_step_face: bool,
    /// `use_topology_distance`: measures steps as 1 (topology) instead of geometric distance.
    pub use_topology_distance: bool,
}

fn step_cost_3_v3_ex(v1: Vec3, v2: Vec3, v3: Vec3, skip_12: bool, skip_23: bool) -> f32 {
    let d1 = v2 - v1;
    let d2 = v3 - v2;
    let cost_12 = d1.length();
    let cost_23 = d2.length();
    let (d1, d2) = (d1 / cost_12.max(f32::EPSILON), d2 / cost_23.max(f32::EPSILON));
    let cost = (if skip_12 { 0.0 } else { cost_12 }) + (if skip_23 { 0.0 } else { cost_23 });
    cost * (1.0 + 0.5 * (2.0 - d1.dot(d2).abs().sqrt()))
}
fn step_cost_3_v3(v1: Vec3, v2: Vec3, v3: Vec3) -> f32 {
    step_cost_3_v3_ex(v1, v2, v3, false, false)
}

/* ------------------------------------------------------------------ */
/** BM_mesh_calc_path_vert */

fn verttag_add_adjacent(
    bm: &BMesh,
    heap: &mut HeapSimple<usize>,
    v_a: usize,
    verts_prev: &mut HashMap<usize, usize>,
    cost: &mut HashMap<usize, f32>,
    visited: &HashSet<usize>,
    params: &BMCalcPathParams,
) {
    let cost_a = cost[&v_a];
    for e in crate::iterators::EdgesOfVert::new(bm, v_a) {
        let v_b = crate::structure::bm_edge_other_vert(bm, e, v_a);
        if !visited.contains(&v_b) {
            let cost_cut = if params.use_topology_distance {
                1.0
            } else {
                (bm.vert(v_a).co - bm.vert(v_b).co).length()
            };
            let cost_new = cost_a + cost_cut;
            if cost.get(&v_b).copied().unwrap_or(COST_INIT_MAX) > cost_new {
                cost.insert(v_b, cost_new);
                verts_prev.insert(v_b, v_a);
                heap.insert(cost_new, v_b);
            }
        }
    }
    if params.use_step_face {
        for l in crate::iterators::LoopsOfVert::new(bm, v_a) {
            let f = bm.loo(l).f;
            if bm.face(f).len > 3 {
                let lo = bm.loo(l);
                let mut l_iter = bm.loo(lo.next).next;
                let l_prev = lo.prev;
                loop {
                    let v_b = bm.loo(l_iter).v;
                    if !visited.contains(&v_b) {
                        let cost_cut = if params.use_topology_distance {
                            1.0
                        } else {
                            (bm.vert(v_a).co - bm.vert(v_b).co).length()
                        };
                        let cost_new = cost_a + cost_cut;
                        if cost.get(&v_b).copied().unwrap_or(COST_INIT_MAX) > cost_new {
                            cost.insert(v_b, cost_new);
                            verts_prev.insert(v_b, v_a);
                            heap.insert(cost_new, v_b);
                        }
                    }
                    l_iter = bm.loo(l_iter).next;
                    if l_iter == l_prev {
                        break;
                    }
                }
            }
        }
    }
}

/// `BM_mesh_calc_path_vert` — returns ordered verts src→dst or empty.
pub fn bm_mesh_calc_path_vert(
    bm: &mut BMesh,
    v_src: usize,
    v_dst: usize,
    params: &BMCalcPathParams,
    mut filter_fn: impl FnMut(&BMesh, usize) -> bool,
) -> Vec<usize> {
    /* BM_ELEM_TAG = "blocked" (inverse of filter); visited tracked in a set. */
    let mut visited: HashSet<usize> = HashSet::new();
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        let blocked = !filter_fn(bm, v);
        if blocked {
            visited.insert(v);
        }
    }
    let mut verts_prev: HashMap<usize, usize> = HashMap::new();
    let mut cost: HashMap<usize, f32> = HashMap::new();
    let mut heap: HeapSimple<usize> = HeapSimple::new();
    heap.insert(0.0, v_src);
    cost.insert(v_src, 0.0);

    let mut found = false;
    while !heap.is_empty() {
        let v = heap.pop_min().unwrap();
        if v == v_dst {
            found = true;
            break;
        }
        if !visited.contains(&v) {
            visited.insert(v);
            verttag_add_adjacent(bm, &mut heap, v, &mut verts_prev, &mut cost, &visited, params);
        }
    }
    let mut path = Vec::new();
    if found {
        let mut v = v_dst;
        loop {
            path.insert(0, v);
            match verts_prev.get(&v) {
                Some(&p) => v = p,
                None => break,
            }
        }
    }
    path
}

/* ------------------------------------------------------------------ */
/** BM_mesh_calc_path_edge */

fn edgetag_cut_cost_vert(bm: &BMesh, e_a: usize, e_b: usize, v: usize) -> f32 {
    let v1 = crate::structure::bm_edge_other_vert(bm, e_a, v);
    let v2 = crate::structure::bm_edge_other_vert(bm, e_b, v);
    step_cost_3_v3(bm.vert(v1).co, bm.vert(v).co, bm.vert(v2).co)
}

fn edgetag_cut_cost_face(bm: &BMesh, e_a: usize, e_b: usize, f: usize) -> f32 {
    let ea = bm.edge(e_a);
    let eb = bm.edge(e_b);
    /* NOTE: C uses v1 twice (midpoint of a single point) — faithful port. */
    let e_a_cent = (bm.vert(ea.v1).co + bm.vert(ea.v1).co) * 0.5;
    let e_b_cent = (bm.vert(eb.v1).co + bm.vert(eb.v1).co) * 0.5;
    let f_cent = crate::polygon::bm_face_calc_center_median_weighted(bm, f);
    step_cost_3_v3(e_a_cent, e_b_cent, f_cent)
}

fn edgetag_add_adjacent(
    bm: &BMesh,
    heap: &mut HeapSimple<usize>,
    e_a: usize,
    edges_prev: &mut HashMap<usize, usize>,
    cost: &mut HashMap<usize, f32>,
    visited_e: &HashSet<usize>,
    visited_v: &HashSet<usize>,
    params: &BMCalcPathParams,
) {
    let cost_a = cost[&e_a];
    if !params.use_step_face || bm.edge(e_a).l == crate::BM_NIL {
        for v in [bm.edge(e_a).v1, bm.edge(e_a).v2] {
            /* Don't walk over previous vertex. */
            if let Some(&e_prev) = edges_prev.get(&e_a) {
                if crate::query::bm_vert_in_edge(bm, e_prev, v) {
                    continue;
                }
            }
            for e_b in crate::iterators::EdgesOfVert::new(bm, v) {
                /* Prevent path overlap, see #137456. */
                let v_other = crate::structure::bm_edge_other_vert(bm, e_b, v);
                if !visited_e.contains(&e_b) && !visited_v.contains(&v_other) {
                    let cost_cut = if params.use_topology_distance {
                        1.0
                    } else {
                        edgetag_cut_cost_vert(bm, e_a, e_b, v)
                    };
                    let cost_new = cost_a + cost_cut;
                    if cost.get(&e_b).copied().unwrap_or(COST_INIT_MAX) > cost_new {
                        cost.insert(e_b, cost_new);
                        edges_prev.insert(e_b, e_a);
                        heap.insert(cost_new, e_b);
                    }
                }
            }
        }
    } else {
        let l_first = bm.edge(e_a).l;
        let mut l_iter = l_first;
        loop {
            let mut l_cycle_iter = bm.loo(l_iter).next;
            let l_cycle_end = l_iter;
            loop {
                let e_b = bm.loo(l_cycle_iter).e;
                if !visited_e.contains(&e_b) {
                    let cost_cut = if params.use_topology_distance {
                        1.0
                    } else {
                        edgetag_cut_cost_face(bm, e_a, e_b, bm.loo(l_iter).f)
                    };
                    let cost_new = cost_a + cost_cut;
                    if cost.get(&e_b).copied().unwrap_or(COST_INIT_MAX) > cost_new {
                        cost.insert(e_b, cost_new);
                        edges_prev.insert(e_b, e_a);
                        heap.insert(cost_new, e_b);
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

/// `BM_mesh_calc_path_edge`.
pub fn bm_mesh_calc_path_edge(
    bm: &mut BMesh,
    e_src: usize,
    e_dst: usize,
    params: &BMCalcPathParams,
    mut filter_fn: impl FnMut(&BMesh, usize) -> bool,
) -> Vec<usize> {
    let mut visited_e: HashSet<usize> = HashSet::new();
    let mut visited_v: HashSet<usize> = HashSet::new();
    for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if !filter_fn(bm, e) {
            visited_e.insert(e);
        }
    }
    let mut edges_prev: HashMap<usize, usize> = HashMap::new();
    let mut cost: HashMap<usize, f32> = HashMap::new();
    let mut heap: HeapSimple<usize> = HeapSimple::new();
    heap.insert(0.0, e_src);
    cost.insert(e_src, 0.0);

    let mut found = false;
    while !heap.is_empty() {
        let e = heap.pop_min().unwrap();
        if e == e_dst {
            found = true;
            break;
        }
        if !visited_e.contains(&e) {
            visited_e.insert(e);
            visited_v.insert(bm.edge(e).v1);
            visited_v.insert(bm.edge(e).v2);
            edgetag_add_adjacent(
                bm,
                &mut heap,
                e,
                &mut edges_prev,
                &mut cost,
                &visited_e,
                &visited_v,
                params,
            );
        }
    }
    let mut path = Vec::new();
    if found {
        let mut e = e_dst;
        loop {
            path.insert(0, e);
            match edges_prev.get(&e) {
                Some(&p) => e = p,
                None => break,
            }
        }
    }
    path
}

/* ------------------------------------------------------------------ */
/** BM_mesh_calc_path_face */

fn facetag_cut_cost_edge(
    bm: &BMesh,
    f_a: usize,
    f_b: usize,
    e: usize,
    f_endpoints: [usize; 2],
) -> f32 {
    let f_a_cent = crate::polygon::bm_face_calc_center_median_weighted(bm, f_a);
    let f_b_cent = crate::polygon::bm_face_calc_center_median_weighted(bm, f_b);
    let ev = bm.edge(e);
    /* For triangle fans it gives better results to pick a point on the edge. */
    let (_, ix_e, _ix_f) = isect_line_line(
        bm.vert(ev.v1).co,
        bm.vert(ev.v2).co,
        f_a_cent,
        f_b_cent,
    );
    let factor = line_point_factor(ix_e, bm.vert(ev.v1).co, bm.vert(ev.v2).co);
    let e_cent = if factor < 0.0 {
        bm.vert(ev.v1).co
    } else if factor > 1.0 {
        bm.vert(ev.v2).co
    } else {
        ix_e
    };
    step_cost_3_v3_ex(
        f_a_cent,
        e_cent,
        f_b_cent,
        f_a == f_endpoints[0],
        f_b == f_endpoints[1],
    )
}

fn facetag_cut_cost_vert(
    bm: &BMesh,
    f_a: usize,
    f_b: usize,
    v: usize,
    f_endpoints: [usize; 2],
) -> f32 {
    let f_a_cent = crate::polygon::bm_face_calc_center_median_weighted(bm, f_a);
    let f_b_cent = crate::polygon::bm_face_calc_center_median_weighted(bm, f_b);
    step_cost_3_v3_ex(
        f_a_cent,
        bm.vert(v).co,
        f_b_cent,
        f_a == f_endpoints[0],
        f_b == f_endpoints[1],
    )
}

fn facetag_add_adjacent(
    bm: &BMesh,
    heap: &mut HeapSimple<usize>,
    f_a: usize,
    faces_prev: &mut HashMap<usize, usize>,
    cost: &mut HashMap<usize, f32>,
    visited: &HashSet<usize>,
    f_endpoints: [usize; 2],
    params: &BMCalcPathParams,
) {
    let cost_a = cost[&f_a];
    for l_a in crate::iterators::LoopsOfFace::new(bm, f_a) {
        let l_first = l_a;
        let mut l_iter = l_first;
        loop {
            let f_b = bm.loo(l_iter).f;
            if !visited.contains(&f_b) {
                let cost_cut = if params.use_topology_distance {
                    1.0
                } else {
                    facetag_cut_cost_edge(bm, f_a, f_b, bm.loo(l_iter).e, f_endpoints)
                };
                let cost_new = cost_a + cost_cut;
                if cost.get(&f_b).copied().unwrap_or(COST_INIT_MAX) > cost_new {
                    cost.insert(f_b, cost_new);
                    faces_prev.insert(f_b, f_a);
                    heap.insert(cost_new, f_b);
                }
            }
            l_iter = bm.loo(l_iter).radial_next;
            if l_iter == l_first {
                break;
            }
        }
    }
    if params.use_step_face {
        for l_a in crate::iterators::LoopsOfFace::new(bm, f_a) {
            let v_a = bm.loo(l_a).v;
            for l_b in crate::iterators::LoopsOfVert::new(bm, v_a) {
                if l_a != l_b && !bm_loop_share_edge_check(bm, l_a, l_b) {
                    let f_b = bm.loo(l_b).f;
                    if !visited.contains(&f_b) {
                        let cost_cut = if params.use_topology_distance {
                            1.0
                        } else {
                            facetag_cut_cost_vert(bm, f_a, f_b, v_a, f_endpoints)
                        };
                        let cost_new = cost_a + cost_cut;
                        if cost.get(&f_b).copied().unwrap_or(COST_INIT_MAX) > cost_new {
                            cost.insert(f_b, cost_new);
                            faces_prev.insert(f_b, f_a);
                            heap.insert(cost_new, f_b);
                        }
                    }
                }
            }
        }
    }
}

/// `BM_mesh_calc_path_face`.
pub fn bm_mesh_calc_path_face(
    bm: &mut BMesh,
    f_src: usize,
    f_dst: usize,
    params: &BMCalcPathParams,
    mut filter_fn: impl FnMut(&BMesh, usize) -> bool,
) -> Vec<usize> {
    let f_endpoints = [f_src, f_dst];
    let mut visited: HashSet<usize> = HashSet::new();
    for f in crate::iterators::FacesOfMesh::new(bm).collect::<Vec<_>>() {
        if !filter_fn(bm, f) {
            visited.insert(f);
        }
    }
    let mut faces_prev: HashMap<usize, usize> = HashMap::new();
    let mut cost: HashMap<usize, f32> = HashMap::new();
    let mut heap: HeapSimple<usize> = HeapSimple::new();
    heap.insert(0.0, f_src);
    cost.insert(f_src, 0.0);

    let mut found = false;
    while !heap.is_empty() {
        let f = heap.pop_min().unwrap();
        if f == f_dst {
            found = true;
            break;
        }
        if !visited.contains(&f) {
            visited.insert(f);
            facetag_add_adjacent(
                bm,
                &mut heap,
                f,
                &mut faces_prev,
                &mut cost,
                &visited,
                f_endpoints,
                params,
            );
        }
    }
    let mut path = Vec::new();
    if found {
        let mut f = f_dst;
        loop {
            path.insert(0, f);
            match faces_prev.get(&f) {
                Some(&p) => f = p,
                None => break,
            }
        }
    }
    path
}
