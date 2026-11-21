//! Port of `bmesh_decimate_collapse.cc` — quadric-based edge-collapse decimation
//! (`BM_mesh_decimate_collapse`), with symmetry + triangulate/re-join passes.
//! CustomData interpolation (`USE_CUSTOMDATA` paths) is PORT_PENDING — no
//! CustomData in the port yet.

use crate::iterators::*;
use crate::{BMesh, BM_NIL};
use crate::{BM_EDGE, BM_ELEM_TAG, BM_FACE, BM_LOOP, BM_VERT};
use blenders_bli::heap::{Heap, HeapNode};
use blenders_bli::kdtree::KDTree;
use blenders_bli::quadric::Quadric;
use blenders_math::vector::Vec3;

const OPTIMIZE_EPS: f64 = 1e-8;
const COST_INVALID: f32 = f32::MAX;
const TOPOLOGY_FALLBACK_EPS: f32 = 1e-12;
const BOUNDARY_PRESERVE_WEIGHT: f64 = 100.0;

/// `CD_UseFlag` — kept for API fidelity; unused (PORT_PENDING: no CustomData).
#[allow(dead_code)]
mod cd_use {
    pub const CD_DO_VERT: i32 = 1 << 0;
    pub const CD_DO_EDGE: i32 = 1 << 1;
    pub const CD_DO_LOOP: i32 = 1 << 2;
}

/// `bm_decim_build_quadrics`.
fn bm_decim_build_quadrics(bm: &mut BMesh, vquadrics: &mut [Quadric]) {
    for f in FacesOfMesh::new(bm) {
        let center = crate::marking::bm_face_calc_center_median(bm, f);
        let no = bm.face(f).no;
        let mut plane_db = [no.x as f64, no.y as f64, no.z as f64, 0.0];
        plane_db[3] = -(plane_db[0] * center.x as f64
            + plane_db[1] * center.y as f64
            + plane_db[2] * center.z as f64);
        let q = Quadric::from_plane(plane_db);
        let l_first = bm.face(f).l_first;
        let mut l = l_first;
        loop {
            let vi = bm.vert(bm.loo(l).v).head.index as usize;
            vquadrics[vi].add_qu_qu(&q);
            l = bm.loo(l).next;
            if l == l_first {
                break;
            }
        }
    }
    for e in EdgesOfMesh::new(bm) {
        if crate::delete::bm_edge_is_boundary(bm, e) {
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            let (c1, c2) = (bm.vert(v1).co, bm.vert(v2).co);
            let edge_vector = c2 - c1;
            let f = bm.loo(bm.edge(e).l).f;
            let fno = bm.face(f).no;
            let edge_plane = edge_vector.cross(fno);
            let mut edge_plane_db = [
                edge_plane.x as f64,
                edge_plane.y as f64,
                edge_plane.z as f64,
                0.0,
            ];
            /* normalize_v3_db */
            let len = (edge_plane_db[0] * edge_plane_db[0]
                + edge_plane_db[1] * edge_plane_db[1]
                + edge_plane_db[2] * edge_plane_db[2])
                .sqrt();
            if len > f32::EPSILON as f64 {
                edge_plane_db[0] /= len;
                edge_plane_db[1] /= len;
                edge_plane_db[2] /= len;
                let center = (c1 + c2) * 0.5;
                edge_plane_db[3] = -(edge_plane_db[0] * center.x as f64
                    + edge_plane_db[1] * center.y as f64
                    + edge_plane_db[2] * center.z as f64);
                let mut q = Quadric::from_plane(edge_plane_db);
                q.mul(BOUNDARY_PRESERVE_WEIGHT);
                vquadrics[bm.vert(v1).head.index as usize].add_qu_qu(&q);
                vquadrics[bm.vert(v2).head.index as usize].add_qu_qu(&q);
            }
        }
    }
}

/// `bm_decim_calc_target_co_db`.
fn bm_decim_calc_target_co_db(bm: &BMesh, e: usize, vquadrics: &[Quadric]) -> [f64; 3] {
    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
    let q = Quadric::add_qu_ququ(
        &vquadrics[bm.vert(v1).head.index as usize],
        &vquadrics[bm.vert(v2).head.index as usize],
    );
    if let Some(co) = q.optimize(OPTIMIZE_EPS) {
        return co;
    }
    let (c1, c2) = (bm.vert(v1).co, bm.vert(v2).co);
    [
        0.5 * (c1.x as f64 + c2.x as f64),
        0.5 * (c1.y as f64 + c2.y as f64),
        0.5 * (c1.z as f64 + c2.z as f64),
    ]
}

/// `bm_decim_calc_target_co_fl`.
fn bm_decim_calc_target_co_fl(bm: &BMesh, e: usize, vquadrics: &[Quadric]) -> Vec3 {
    let d = bm_decim_calc_target_co_db(bm, e, vquadrics);
    Vec3::new(d[0] as f32, d[1] as f32, d[2] as f32)
}

/// `bm_edge_collapse_is_degenerate_flip`.
fn bm_edge_collapse_is_degenerate_flip(bm: &BMesh, e: usize, optimize_co: Vec3) -> bool {
    for i in 0..2 {
        let v = if i == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
        for l in LoopsOfVert::new(bm, v) {
            if bm.loo(l).e != e && bm.loo(bm.loo(l).prev).e != e {
                let co_prev = bm.vert(bm.loo(bm.loo(l).prev).v).co;
                let co_next = bm.vert(bm.loo(bm.loo(l).next).v).co;
                let vec_other = co_prev - co_next;
                let vec_exist = co_prev - bm.vert(v).co;
                let vec_optim = co_prev - optimize_co;
                let cross_exist = vec_other.cross(vec_exist);
                let cross_optim = vec_other.cross(vec_optim);
                if cross_exist.dot(cross_optim)
                    <= (cross_exist.length_squared() + cross_optim.length_squared()) * 0.01
                {
                    return true;
                }
            }
        }
    }
    false
}

/// `bm_decim_build_edge_cost_single_squared__topology`.
fn bm_decim_build_edge_cost_single_squared_topology(bm: &BMesh, e: usize) -> f32 {
    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
    bm.vert(v1).no.dot(bm.vert(v2).no).abs()
        / (-(bm.vert(v1).co - bm.vert(v2).co).length_squared()).min(-f32::EPSILON)
}
/// `bm_decim_build_edge_cost_single__topology`.
fn bm_decim_build_edge_cost_single_topology(bm: &BMesh, e: usize) -> f32 {
    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
    bm.vert(v1).no.dot(bm.vert(v2).no).abs()
        / (-(bm.vert(v1).co - bm.vert(v2).co).length()).min(-f32::EPSILON)
}

/// `bm_decim_build_edge_cost_single`.
fn bm_decim_build_edge_cost_single(
    bm: &BMesh,
    e: usize,
    vquadrics: &[Quadric],
    vweights: Option<&[f32]>,
    vweight_factor: f32,
    eheap: &mut Heap<usize>,
    eheap_table: &mut [Option<HeapNode>],
) {
    macro_rules! clear {
        () => {{
            let ei = bm.edge(e).head.index as usize;
            if let Some(n) = eheap_table[ei] {
                eheap.remove(n);
            }
            eheap_table[ei] = None;
            return;
        }};
    }
    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
    if let Some(w) = vweights {
        if w[bm.vert(v1).head.index as usize] == 0.0
            || w[bm.vert(v2).head.index as usize] == 0.0
        {
            clear!();
        }
    }
    if crate::delete::bm_edge_is_boundary(bm, e) {
        if bm.face(bm.loo(bm.edge(e).l).f).len == 3 {
            /* ok */
        } else {
            clear!();
        }
    } else if crate::query::bm_edge_is_manifold(bm, e) {
        let l = bm.edge(e).l;
        if bm.face(bm.loo(l).f).len == 3 && bm.face(bm.loo(bm.loo(l).radial_next).f).len == 3
        {
            /* ok */
        } else {
            clear!();
        }
    } else {
        clear!();
    }
    let optimize_co = bm_decim_calc_target_co_db(bm, e, vquadrics);
    let q1 = &vquadrics[bm.vert(v1).head.index as usize];
    let q2 = &vquadrics[bm.vert(v2).head.index as usize];
    let mut cost = (q1.evaluate(optimize_co) + q2.evaluate(optimize_co)) as f32;
    cost = cost.abs();
    /* USE_TOPOLOGY_FALLBACK */
    if cost < TOPOLOGY_FALLBACK_EPS {
        if vweights.is_none() {
            cost = bm_decim_build_edge_cost_single_squared_topology(bm, e) - cost;
        } else {
            let w = vweights.unwrap();
            let e_weight = w[bm.vert(v1).head.index as usize]
                + w[bm.vert(v2).head.index as usize];
            cost = bm_decim_build_edge_cost_single_topology(bm, e) - cost;
            if e_weight != 0.0 {
                cost *= 1.0 + e_weight * vweight_factor;
            }
            debug_assert!(cost <= 0.0);
        }
    } else if let Some(w) = vweights {
        let e_weight = 2.0
            - (w[bm.vert(v1).head.index as usize] + w[bm.vert(v2).head.index as usize]);
        if e_weight != 0.0 {
            cost += crate::query::bm_edge_calc_length(bm, e) * (e_weight * vweight_factor);
        }
    }
    let ei = bm.edge(e).head.index as usize;
    let mut node = eheap_table[ei];
    eheap.insert_or_update(&mut node, cost, e);
    eheap_table[ei] = node;
}

/// `bm_decim_invalid_edge_cost_single`.
fn bm_decim_invalid_edge_cost_single(
    bm: &BMesh,
    e: usize,
    eheap: &mut Heap<usize>,
    eheap_table: &mut [Option<HeapNode>],
) {
    let ei = bm.edge(e).head.index as usize;
    debug_assert!(eheap_table[ei].is_none());
    eheap_table[ei] = Some(eheap.insert(COST_INVALID, e));
}

/// `bm_decim_build_edge_cost`.
fn bm_decim_build_edge_cost(
    bm: &mut BMesh,
    vquadrics: &[Quadric],
    vweights: Option<&[f32]>,
    vweight_factor: f32,
    eheap: &mut Heap<usize>,
    eheap_table: &mut [Option<HeapNode>],
) {
    let mut i = 0usize;
    for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        bm.epool[e].head.index = i as i32;
        eheap_table[i] = None;
        bm_decim_build_edge_cost_single(bm, e, vquadrics, vweights, vweight_factor, eheap, eheap_table);
        i += 1;
    }
}

/// `bm_edge_symmetry_map` — uses a KDTree over edge centers.
fn bm_edge_symmetry_map(bm: &mut BMesh, symmetry_axis: usize, limit: f32) -> Vec<i32> {
    let limit_sq = limit * limit;
    let totedge = bm.totedge as usize;
    let mut etable = Vec::with_capacity(totedge);
    let mut edge_centers: Vec<[f32; 3]> = Vec::with_capacity(totedge);
    let mut map = vec![-1i32; totedge];
    for e in EdgesOfMesh::new(bm) {
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        let c = (bm.vert(v1).co + bm.vert(v2).co) * 0.5;
        edge_centers.push([c.x, c.y, c.z]);
        etable.push(e);
    }
    let tree = KDTree::new(&edge_centers, 16);
    for (i, &e) in etable.iter().enumerate() {
        if map[i] == -1 {
            let mut co = edge_centers[i];
            co[symmetry_axis] *= -1.0;
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            let mut e_v1_co = [bm.vert(v1).co.x, bm.vert(v1).co.y, bm.vert(v1).co.z];
            let mut e_v2_co = [bm.vert(v2).co.x, bm.vert(v2).co.y, bm.vert(v2).co.z];
            e_v1_co[symmetry_axis] *= -1.0;
            e_v2_co[symmetry_axis] *= -1.0;
            let e_dir = [
                e_v2_co[0] - e_v1_co[0],
                e_v2_co[1] - e_v1_co[1],
                e_v2_co[2] - e_v1_co[2],
            ];
            let mut e_found_index = -1i32;
            tree.foreach_in_radius(&edge_centers, &co, limit, |index, _dist_sq| {
                let e_other = etable[index as usize];
                let (o1, o2) = (bm.edge(e_other).v1, bm.edge(e_other).v2);
                let (c1, c2) = (bm.vert(o1).co, bm.vert(o2).co);
                let e_other_dir = [c2.x - c1.x, c2.y - c1.y, c2.z - c1.z];
                let dot = e_other_dir[0] * e_dir[0]
                    + e_other_dir[1] * e_dir[1]
                    + e_other_dir[2] * e_dir[2];
                let o1a = [c1.x, c1.y, c1.z];
                let o2a = [c2.x, c2.y, c2.z];
                let d = |a: [f32; 3], b: [f32; 3]| -> f32 {
                    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
                };
                if dot > 0.0 {
                    if d(e_v1_co, o1a) > limit_sq || d(e_v2_co, o2a) > limit_sq {
                        return;
                    }
                } else if d(e_v1_co, o2a) > limit_sq || d(e_v2_co, o1a) > limit_sq {
                    return;
                }
                if e_found_index == -1 || index < e_found_index {
                    e_found_index = index;
                }
            });
            if e_found_index != -1 {
                let i_other = e_found_index as usize;
                map[i] = i_other as i32;
                map[i_other] = i as i32;
            }
        }
    }
    map
}

/// `bm_face_triangulate` (decimate-local): triangulate face, tag new edges'
/// loops with the base face index, update normals.
fn bm_decim_face_triangulate(bm: &mut BMesh, f_base: usize) -> (Vec<usize>, Vec<usize>, usize) {
    let f_index = bm.face(f_base).head.index;
    let (faces_new, edges_new, faces_double) =
        crate::polygon::bm_face_triangulate(bm, f_base, 0, 0, false);
    let mut has_cut = false;
    for &e in &edges_new {
        if bm.edge(e).l == BM_NIL {
            continue;
        }
        let l_first = bm.edge(e).l;
        let mut l = l_first;
        loop {
            bm.lpool[l].head.index = f_index;
            has_cut = true;
            l = bm.loo(l).radial_next;
            if l == l_first {
                break;
            }
        }
    }
    for &f in &faces_new {
        crate::normals::bm_face_calc_normals(bm, f);
    }
    (edges_new, faces_double, usize::from(has_cut))
}

/// `bm_decim_triangulate_begin`.
fn bm_decim_triangulate_begin(bm: &mut BMesh) -> (bool, usize) {
    let mut has_ngon = false;
    let mut has_cut = false;
    let mut edges_tri_tot = 0usize;
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        let l_first = bm.face(f).l_first;
        let mut l = l_first;
        loop {
            bm.lpool[l].head.index = -1;
            l = bm.loo(l).next;
            if l == l_first {
                break;
            }
        }
        has_ngon |= bm.face(f).len > 4;
    }
    let _ = has_ngon; /* pf_arena/pf_heap only needed for ngons in C; our
                       * bm_face_triangulate handles both cases itself. */
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.face(f).len > 3 {
            let (edges_new, faces_double, cut) = bm_decim_face_triangulate(bm, f);
            edges_tri_tot += edges_new.len();
            for fd in faces_double {
                crate::core::bm_face_kill(bm, fd);
            }
            has_cut |= cut == 1;
        }
    }
    if has_cut {
        crate::mesh::bm_mesh_elem_index_ensure(bm, BM_EDGE | BM_FACE);
    }
    (has_cut, edges_tri_tot)
}

/// `bm_decim_triangulate_end` — re-join pairs of triangles into the faces
/// they were cut from (loop index marks the originating face).
fn bm_decim_triangulate_end(bm: &mut BMesh, edges_tri_tot: usize) {
    let mut edges_tri: Vec<usize> = Vec::with_capacity(edges_tri_tot.min(bm.totedge as usize));
    /* boundary edges */
    for e in EdgesOfMesh::new(bm) {
        if let Some((l_a, l_b)) = crate::query::bm_edge_loop_pair(bm, e) {
            let l_a_index = bm.loo(l_a).head.index;
            if l_a_index != -1 {
                let l_b_index = bm.loo(l_b).head.index;
                if l_a_index == l_b_index && bm.loo(l_a).v != bm.loo(l_b).v {
                    /* faces have become flipped; sanity check quad is convex */
                    macro_rules! can_loop_merge {
                        ($l:expr) => {{
                            let lx = $l;
                            bm_loop_is_manifold(bm, lx)
                                && bm.loo(lx).v != bm.loo(bm.loo(lx).radial_next).v
                                && l_a_index == bm.loo(lx).head.index
                                && l_a_index
                                    == bm.loo(bm.loo(lx).radial_next).head.index
                        }};
                    }
                    if bm.face(bm.loo(l_a).f).len == 3
                        && bm.face(bm.loo(l_b).f).len == 3
                        && !can_loop_merge!(bm.loo(l_a).next)
                        && !can_loop_merge!(bm.loo(l_a).prev)
                        && !can_loop_merge!(bm.loo(l_b).next)
                        && !can_loop_merge!(bm.loo(l_b).prev)
                    {
                        let e_v1 = bm.edge(e).v1;
                        let e_v2 = bm.edge(e).v2;
                        let vq1 = if crate::query::bm_vert_in_edge(bm, e, bm.loo(bm.loo(l_a).next).v)
                        {
                            bm.loo(bm.loo(l_a).prev).v
                        } else {
                            bm.loo(bm.loo(l_a).next).v
                        };
                        let vq3 = if crate::query::bm_vert_in_edge(bm, e, bm.loo(bm.loo(l_b).next).v)
                        {
                            bm.loo(bm.loo(l_b).prev).v
                        } else {
                            bm.loo(bm.loo(l_b).next).v
                        };
                        if !blenders_math::geom::is_quad_convex(
                            bm.vert(e_v1).co,
                            bm.vert(vq1).co,
                            bm.vert(e_v2).co,
                            bm.vert(vq3).co,
                        ) {
                            continue;
                        }
                    }
                    edges_tri.push(e);
                }
            }
        }
    }
    for e in edges_tri {
        if let Some((l_a, l_b)) = crate::query::bm_edge_loop_pair(bm, e) {
            let f_array = [bm.loo(l_a).f, bm.loo(l_b).f];
            let mut f_double = BM_NIL;
            let f_join = crate::core::bm_faces_join(
                bm,
                &f_array,
                false,
                Some(&mut f_double),
            );
            if f_double != BM_NIL {
                crate::core::bm_face_kill(bm, f_join);
            }
            if bm.edge(e).l == BM_NIL {
                crate::core::bm_edge_kill(bm, e);
            }
        }
    }
}

/// `bm_edge_collapse_loop_customdata` — PORT_PENDING (no CustomData layers).

/// `bm_edge_tag_enable`.
fn bm_edge_tag_enable(bm: &mut BMesh, e: usize) {
    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
    bm.vpool[v1].head.hflag |= BM_ELEM_TAG;
    bm.vpool[v2].head.hflag |= BM_ELEM_TAG;
    if bm.edge(e).l != BM_NIL {
        let l = bm.edge(e).l;
        let f = bm.loo(l).f;
        bm.fpool[f].head.hflag |= BM_ELEM_TAG;
        let rn = bm.loo(l).radial_next;
        if rn != l {
            let f2 = bm.loo(rn).f;
            bm.fpool[f2].head.hflag |= BM_ELEM_TAG;
        }
    }
}
/// `bm_edge_tag_disable`.
fn bm_edge_tag_disable(bm: &mut BMesh, e: usize) {
    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
    bm.vpool[v1].head.hflag &= !BM_ELEM_TAG;
    bm.vpool[v2].head.hflag &= !BM_ELEM_TAG;
    if bm.edge(e).l != BM_NIL {
        let l = bm.edge(e).l;
        let f = bm.loo(l).f;
        bm.fpool[f].head.hflag &= !BM_ELEM_TAG;
        let rn = bm.loo(l).radial_next;
        if rn != l {
            let f2 = bm.loo(rn).f;
            bm.fpool[f2].head.hflag &= !BM_ELEM_TAG;
        }
    }
}
/// `bm_edge_tag_test`.
fn bm_edge_tag_test(bm: &BMesh, e: usize) -> bool {
    bm.vert(bm.edge(e).v1).head.hflag & BM_ELEM_TAG != 0
        || bm.vert(bm.edge(e).v2).head.hflag & BM_ELEM_TAG != 0
        || (bm.edge(e).l != BM_NIL && {
            let l = bm.edge(e).l;
            bm.face(bm.loo(l).f).head.hflag & BM_ELEM_TAG != 0
                || (bm.loo(l).radial_next != l
                    && bm.face(bm.loo(bm.loo(l).radial_next).f).head.hflag & BM_ELEM_TAG != 0)
        })
}

/// `bm_edge_is_manifold_or_boundary`.
#[inline]
fn bm_edge_is_manifold_or_boundary(bm: &BMesh, l: usize) -> bool {
    l != BM_NIL && bm.loo(bm.loo(l).radial_next).radial_next == l
}

/// `bm_edge_collapse_is_degenerate_topology`.
fn bm_edge_collapse_is_degenerate_topology(bm: &mut BMesh, e_first: usize) -> bool {
    let (v1, v2) = (bm.edge(e_first).v1, bm.edge(e_first).v2);
    let mut e_iter = e_first;
    loop {
        if !bm_edge_is_manifold_or_boundary(bm, bm.edge(e_iter).l) {
            return true;
        }
        bm_edge_tag_disable(bm, e_iter);
        e_iter = crate::structure::bmesh_disk_edge_next(bm, e_iter, v1);
        if e_iter == e_first {
            break;
        }
    }
    e_iter = e_first;
    loop {
        if !bm_edge_is_manifold_or_boundary(bm, bm.edge(e_iter).l) {
            return true;
        }
        bm_edge_tag_disable(bm, e_iter);
        e_iter = crate::structure::bmesh_disk_edge_next(bm, e_iter, v2);
        if e_iter == e_first {
            break;
        }
    }
    e_iter = e_first;
    loop {
        bm_edge_tag_enable(bm, e_iter);
        e_iter = crate::structure::bmesh_disk_edge_next(bm, e_iter, v1);
        if e_iter == e_first {
            break;
        }
    }
    {
        let l_radial = bm.edge(e_first).l;
        let mut l_face = l_radial;
        debug_assert!(bm.face(bm.loo(l_face).f).len == 3);
        let f = bm.loo(l_face).f;
        bm.fpool[f].head.hflag &= !BM_ELEM_TAG;
        l_face = l_radial;
        let vv = bm.loo(l_face).v;
        bm.vpool[vv].head.hflag &= !BM_ELEM_TAG;
        l_face = bm.loo(l_face).next;
        let vv = bm.loo(l_face).v;
        bm.vpool[vv].head.hflag &= !BM_ELEM_TAG;
        let l_face_next = bm.loo(l_face).next;
        let vv = bm.loo(l_face_next).v;
        bm.vpool[vv].head.hflag &= !BM_ELEM_TAG;
        l_face = bm.loo(l_radial).radial_next;
        if l_radial != l_face {
            debug_assert!(bm.face(bm.loo(l_face).f).len == 3);
            let f2 = bm.loo(l_face).f;
            bm.fpool[f2].head.hflag &= !BM_ELEM_TAG;
            l_face = bm.loo(l_radial).radial_next;
            let vv = bm.loo(l_face).v;
            bm.vpool[vv].head.hflag &= !BM_ELEM_TAG;
            l_face = bm.loo(l_face).next;
            let vv = bm.loo(l_face).v;
            bm.vpool[vv].head.hflag &= !BM_ELEM_TAG;
            let l_face_next = bm.loo(l_face).next;
            let vv = bm.loo(l_face_next).v;
            bm.vpool[vv].head.hflag &= !BM_ELEM_TAG;
        }
    }
    e_iter = e_first;
    loop {
        if bm_edge_tag_test(bm, e_iter) {
            return true;
        }
        e_iter = crate::structure::bmesh_disk_edge_next(bm, e_iter, v2);
        if e_iter == e_first {
            break;
        }
    }
    false
}

/// `bm_edge_collapse` — returns indices of the two "other" edges in
/// `r_e_clear_other`. CustomData interpolation is PORT_PENDING.
fn bm_edge_collapse(
    bm: &mut BMesh,
    e_clear: usize,
    v_clear: usize,
    r_e_clear_other: &mut [i32; 2],
    edge_symmetry_map: Option<&mut Vec<i32>>,
    _customdata_flag: i32,
    _customdata_fac: f32,
) -> bool {
    let v_other = crate::structure::bm_edge_other_vert(bm, e_clear, v_clear);
    debug_assert!(v_other != BM_NIL);
    if crate::query::bm_edge_is_manifold(bm, e_clear) {
        let (l_a, l_b) = crate::query::bm_edge_loop_pair(bm, e_clear).unwrap();
        debug_assert!(bm.face(bm.loo(l_a).f).len == 3);
        debug_assert!(bm.face(bm.loo(l_b).f).len == 3);
        let mut e_a_other = [BM_NIL; 2];
        let mut e_b_other = [BM_NIL; 2];
        if crate::query::bm_vert_in_edge(bm, bm.loo(bm.loo(l_a).prev).e, v_clear) {
            e_a_other[0] = bm.loo(bm.loo(l_a).prev).e;
            e_a_other[1] = bm.loo(bm.loo(l_a).next).e;
        } else {
            e_a_other[1] = bm.loo(bm.loo(l_a).prev).e;
            e_a_other[0] = bm.loo(bm.loo(l_a).next).e;
        }
        if crate::query::bm_vert_in_edge(bm, bm.loo(bm.loo(l_b).prev).e, v_clear) {
            e_b_other[0] = bm.loo(bm.loo(l_b).prev).e;
            e_b_other[1] = bm.loo(bm.loo(l_b).next).e;
        } else {
            e_b_other[1] = bm.loo(bm.loo(l_b).prev).e;
            e_b_other[0] = bm.loo(bm.loo(l_b).next).e;
        }
        if e_a_other[0] == e_b_other[0]
            || e_a_other[0] == e_b_other[1]
            || e_a_other[1] == e_b_other[0]
            || e_a_other[1] == e_b_other[1]
        {
            return false;
        }
        debug_assert!(crate::query::bm_edge_share_vert_check(bm, e_a_other[0], e_b_other[0]));
        debug_assert!(crate::query::bm_edge_share_vert_check(bm, e_a_other[1], e_b_other[1]));
        r_e_clear_other[0] = bm.edge(e_a_other[0]).head.index;
        r_e_clear_other[1] = bm.edge(e_b_other[0]).head.index;
        /* PORT_PENDING: CD_DO_VERT/CD_DO_EDGE/CD_DO_LOOP interp. */
        crate::core::bm_edge_kill(bm, e_clear);
        let hflag = bm.vert(v_clear).head.hflag;
        bm.vpool[v_other].head.hflag |= hflag;
        crate::core::bm_vert_splice(bm, v_other, v_clear);
        let hf = bm.edge(e_a_other[0]).head.hflag;
        bm.epool[e_a_other[1]].head.hflag |= hf;
        let hf = bm.edge(e_b_other[0]).head.hflag;
        bm.epool[e_b_other[1]].head.hflag |= hf;
        crate::core::bm_edge_splice(bm, e_a_other[1], e_a_other[0]);
        crate::core::bm_edge_splice(bm, e_b_other[1], e_b_other[0]);
        if let Some(map) = edge_symmetry_map {
            let idx0 = r_e_clear_other[0] as usize;
            let idx1 = r_e_clear_other[1] as usize;
            if map[idx0] != -1 {
                let m = map[idx0] as usize;
                map[m] = bm.edge(e_a_other[1]).head.index;
            }
            if map[idx1] != -1 {
                let m = map[idx1] as usize;
                map[m] = bm.edge(e_b_other[1]).head.index;
            }
        }
        return true;
    }
    if crate::delete::bm_edge_is_boundary(bm, e_clear) {
        let l_a = bm.edge(e_clear).l;
        debug_assert!(bm.face(bm.loo(l_a).f).len == 3);
        let mut e_a_other = [BM_NIL; 2];
        if crate::query::bm_vert_in_edge(bm, bm.loo(bm.loo(l_a).prev).e, v_clear) {
            e_a_other[0] = bm.loo(bm.loo(l_a).prev).e;
            e_a_other[1] = bm.loo(bm.loo(l_a).next).e;
        } else {
            e_a_other[1] = bm.loo(bm.loo(l_a).prev).e;
            e_a_other[0] = bm.loo(bm.loo(l_a).next).e;
        }
        r_e_clear_other[0] = bm.edge(e_a_other[0]).head.index;
        r_e_clear_other[1] = -1;
        crate::core::bm_edge_kill(bm, e_clear);
        let hflag = bm.vert(v_clear).head.hflag;
        bm.vpool[v_other].head.hflag |= hflag;
        crate::core::bm_vert_splice(bm, v_other, v_clear);
        let hf = bm.edge(e_a_other[0]).head.hflag;
        bm.epool[e_a_other[1]].head.hflag |= hf;
        crate::core::bm_edge_splice(bm, e_a_other[1], e_a_other[0]);
        if let Some(map) = edge_symmetry_map {
            let idx0 = r_e_clear_other[0] as usize;
            if map[idx0] != -1 {
                let m = map[idx0] as usize;
                map[m] = bm.edge(e_a_other[1]).head.index;
            }
        }
        return true;
    }
    false
}

/// `bm_decim_edge_collapse`.
#[allow(clippy::too_many_arguments)]
fn bm_decim_edge_collapse(
    bm: &mut BMesh,
    e: usize,
    vquadrics: &mut [Quadric],
    mut vweights: Option<&mut Vec<f32>>,
    vweight_factor: f32,
    eheap: &mut Heap<usize>,
    eheap_table: &mut [Option<HeapNode>],
    edge_symmetry_map: Option<&mut Vec<i32>>,
    customdata_flag: i32,
    optimize_co_calc: bool,
) -> (bool, Vec3) {
    let mut e_clear_other = [-1i32; 2];
    let v_other = bm.edge(e).v1;
    let v_other_index = bm.vert(v_other).head.index as usize;
    let v_clear_index = bm.vert(bm.edge(e).v2).head.index as usize;
    /* USE_VERT_NORMAL_INTERP */
    let v_clear_no = bm.vert(bm.edge(e).v2).no;
    let mut optimize_co = Vec3::new(0.0, 0.0, 0.0);
    if optimize_co_calc {
        if bm_edge_collapse_is_degenerate_topology(bm, e) {
            bm_decim_invalid_edge_cost_single(bm, e, eheap, eheap_table);
            return (false, optimize_co);
        }
        optimize_co = bm_decim_calc_target_co_fl(bm, e, vquadrics);
        if bm_edge_collapse_is_degenerate_flip(bm, e, optimize_co) {
            bm_decim_invalid_edge_cost_single(bm, e, eheap, eheap_table);
            return (false, optimize_co);
        }
    }
    let (c1, c2) = (bm.vert(bm.edge(e).v1).co, bm.vert(bm.edge(e).v2).co);
    let d = c1 - c2;
    let customdata_fac = if d.x.abs() > f32::EPSILON
        || d.y.abs() > f32::EPSILON
        || d.z.abs() > f32::EPSILON
    {
        blenders_math::geom::line_point_factor(optimize_co, c1, c2)
    } else {
        0.5
    };
    let mut esm = edge_symmetry_map;
    if bm_edge_collapse(
        bm,
        e,
        bm.edge(e).v2,
        &mut e_clear_other,
        esm.as_deref_mut(),
        customdata_flag,
        customdata_fac,
    ) {
        if let Some(w) = vweights.as_deref_mut() {
            let mut v_other_weight = w[v_other_index]
                + (w[v_clear_index] - w[v_other_index]) * customdata_fac;
            v_other_weight = v_other_weight.clamp(0.0, 1.0);
            w[v_other_index] = v_other_weight;
        }
        bm.vpool[v_other].co = optimize_co;
        for i in 0..2 {
            if e_clear_other[i] != -1 {
                let idx = e_clear_other[i] as usize;
                if let Some(n) = eheap_table[idx] {
                    eheap.remove(n);
                    eheap_table[idx] = None;
                }
            }
        }
        vquadrics[v_other_index].add_qu_qu(&vquadrics[v_clear_index].clone());
        /* USE_VERT_NORMAL_INTERP */
        let n1 = bm.vert(v_other).no;
        bm.vpool[v_other].no = n1 + (v_clear_no - n1) * customdata_fac;
        let l = bm.vert(v_other).no.length();
        if l > 0.0 {
            bm.vpool[v_other].no = bm.vert(v_other).no / l;
        }
        if bm.vert(v_other).e != BM_NIL {
            let e_first = bm.vert(v_other).e;
            let mut e_iter = e_first;
            loop {
                bm_decim_build_edge_cost_single(
                    bm,
                    e_iter,
                    vquadrics,
                    vweights.as_deref().map(|w| &w[..]),
                    vweight_factor,
                    eheap,
                    eheap_table,
                );
                e_iter = crate::structure::bmesh_disk_edge_next(bm, e_iter, v_other);
                if e_iter == e_first {
                    break;
                }
            }
        }
        for l in LoopsOfVert::new(bm, v_other).collect::<Vec<_>>() {
            if bm.face(bm.loo(l).f).len == 3 {
                let e_outer = if crate::query::bm_vert_in_edge(
                    bm,
                    bm.loo(bm.loo(l).prev).e,
                    bm.loo(l).v,
                ) {
                    bm.loo(bm.loo(l).next).e
                } else {
                    bm.loo(bm.loo(l).prev).e
                };
                bm_decim_build_edge_cost_single(
                    bm,
                    e_outer,
                    vquadrics,
                    vweights.as_deref().map(|w| &w[..]),
                    vweight_factor,
                    eheap,
                    eheap_table,
                );
            }
        }
        return (true, optimize_co);
    }
    bm_decim_invalid_edge_cost_single(bm, e, eheap, eheap_table);
    (false, optimize_co)
}

/// `bm_loop_is_manifold` — is this loop's edge 2-manifold?
fn bm_loop_is_manifold(bm: &BMesh, l: usize) -> bool {
    bm.loo(bm.loo(bm.loo(l).radial_next).radial_next).radial_next == l
}

/// `BM_mesh_decimate_collapse`.
pub fn bm_mesh_decimate_collapse(
    bm: &mut BMesh,
    factor: f32,
    vweights: Option<&mut Vec<f32>>,
    vweight_factor: f32,
    do_triangulate: bool,
    symmetry_axis: i32,
    symmetry_eps: f32,
) {
    let mut vweights = vweights;
    let use_symmetry = symmetry_axis != -1;
    let (use_triangulate, edges_tri_tot) = if do_triangulate {
        bm_decim_triangulate_begin(bm)
    } else {
        (false, 0)
    };
    crate::mesh::bm_mesh_elem_index_ensure(bm, BM_EDGE);
    let mut vquadrics = vec![Quadric::default(); bm.totvert as usize];
    let mut eheap: Heap<usize> = Heap::new();
    let mut eheap_table: Vec<Option<HeapNode>> = vec![None; bm.totedge as usize];
    bm_decim_build_quadrics(bm, &mut vquadrics);
    bm_decim_build_edge_cost(
        bm,
        &vquadrics,
        vweights.as_deref().map(|w| &w[..]),
        vweight_factor,
        &mut eheap,
        &mut eheap_table,
    );
    let face_tot_target = (bm.totface as f32 * factor) as i32;
    bm.elem_index_dirty = true;
    let mut edge_symmetry_map = if use_symmetry {
        Some(bm_edge_symmetry_map(bm, symmetry_axis as usize, symmetry_eps))
    } else {
        None
    };
    let customdata_flag = 0i32; /* PORT_PENDING: no CustomData layers. */

    if !use_symmetry {
        while bm.totface > face_tot_target
            && !eheap.is_empty()
            && eheap.top_value() != Some(COST_INVALID)
        {
            let e = eheap.pop_min().unwrap();
            let ei = bm.edge(e).head.index as usize;
            eheap_table[ei] = None;
            bm_decim_edge_collapse(
                bm,
                e,
                &mut vquadrics,
                vweights.as_deref_mut(),
                vweight_factor,
                &mut eheap,
                &mut eheap_table,
                edge_symmetry_map.as_mut(),
                customdata_flag,
                true,
            );
        }
    } else {
        /* USE_SYMMETRY */
        let map = edge_symmetry_map.as_mut().unwrap();
        while bm.totface > face_tot_target
            && !eheap.is_empty()
            && eheap.top_value() != Some(COST_INVALID)
        {
            let e = eheap.pop_min().unwrap();
            let e_index = bm.edge(e).head.index as usize;
            let e_index_mirr = map[e_index];
            let mut e_mirr = BM_NIL;
            let mut e_invalidate = 0u8;
            let mut optimize_co = Vec3::new(0.0, 0.0, 0.0);
            eheap_table[e_index] = None;
            macro_rules! invalidate {
                () => {{
                    if e_invalidate & 1 != 0 {
                        bm_decim_invalid_edge_cost_single(bm, e, &mut eheap, &mut eheap_table);
                    }
                    if e_invalidate & 2 != 0 {
                        let mirr_idx = e_index_mirr as usize;
                        if let Some(n) = eheap_table[mirr_idx] {
                            eheap.remove(n);
                            eheap_table[mirr_idx] = None;
                        }
                        bm_decim_invalid_edge_cost_single(bm, e_mirr, &mut eheap, &mut eheap_table);
                    }
                    continue;
                }};
            }
            if e_index_mirr != -1 {
                if e_index_mirr as usize == e_index {
                    /* ok */
                } else if let Some(n) = eheap_table[e_index_mirr as usize] {
                    e_mirr = *eheap.node_ptr(n);
                    if crate::query::bm_edge_share_vert_check(bm, e, e_mirr) {
                        invalidate!();
                    }
                } else {
                    e_invalidate |= 1;
                    invalidate!();
                }
            }
            {
                let ok_a = !bm_edge_collapse_is_degenerate_topology(bm, e);
                let ok_b = if e_mirr != BM_NIL {
                    !bm_edge_collapse_is_degenerate_topology(bm, e_mirr)
                } else {
                    true
                };
                if !ok_a || !ok_b {
                    e_invalidate |= 1 | if e_mirr != BM_NIL { 2 } else { 0 };
                    invalidate!();
                }
                optimize_co = bm_decim_calc_target_co_fl(bm, e, &vquadrics);
                if e_index_mirr as usize == e_index {
                    optimize_co[symmetry_axis as usize] = 0.0;
                }
                if bm_edge_collapse_is_degenerate_flip(bm, e, optimize_co) {
                    e_invalidate |= 1 | if e_mirr != BM_NIL { 2 } else { 0 };
                    invalidate!();
                }
            }
            let (ok, _) = bm_decim_edge_collapse(
                bm,
                e,
                &mut vquadrics,
                vweights.as_deref_mut(),
                vweight_factor,
                &mut eheap,
                &mut eheap_table,
                Some(map),
                customdata_flag,
                false,
            );
            if ok {
                if e_mirr != BM_NIL && eheap_table[e_index_mirr as usize].is_some() {
                    let mirr_idx = e_index_mirr as usize;
                    let n = eheap_table[mirr_idx].unwrap();
                    eheap.remove(n);
                    eheap_table[mirr_idx] = None;
                    optimize_co[symmetry_axis as usize] *= -1.0;
                    bm_decim_edge_collapse(
                        bm,
                        e_mirr,
                        &mut vquadrics,
                        vweights.as_deref_mut(),
                        vweight_factor,
                        &mut eheap,
                        &mut eheap_table,
                        Some(map),
                        customdata_flag,
                        false,
                    );
                }
            } else if e_mirr != BM_NIL && eheap_table[e_index_mirr as usize].is_some() {
                e_invalidate |= 2;
                invalidate!();
            }
        }
    }
    if !do_triangulate && use_triangulate {
        bm_decim_triangulate_end(bm, edges_tri_tot);
    }
}
