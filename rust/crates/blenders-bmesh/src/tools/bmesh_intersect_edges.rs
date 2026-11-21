//! Port of `bmesh/tools/bmesh_intersect_edges.cc`.
//!
//! Detects near-intersecting verts/edges (BVHTree overlap queries), welds
//! vert-pairs and splits edges at crossings. Optionally splits faces along
//! the new "edge-net" between cut points.

use crate::iterators::*;
use crate::{BMesh, BM_NIL};
use crate::{BM_EDGE, BM_VERT};
use crate::{BM_ELEM_HIDDEN, BM_ELEM_TAG};
use blenders_bli::kdopbvh::{bvhtree_balance, bvhtree_insert, bvhtree_new, bvhtree_overlap_ex,
    BVH_OVERLAP_SELF, BVH_OVERLAP_USE_THREADING, BVHTree};
use blenders_math::vector::Vec3;
use std::collections::HashMap;

const KDOP_TREE_TYPE: u8 = 4;
const KDOP_AXIS_LEN: u8 = 14;

const EDGE_ACT_TO_TEST: i32 = 1;
const EDGE_REMAIN_TO_TEST: i32 = 2;

/// `EDBMSplitElem`: either a vert, or an edge + split lambda
/// (`vert` is overwritten with the created vert after splitting).
#[derive(Clone, Copy)]
struct SplitElem {
    htype: u8,
    edge: usize,
    vert: usize,
    lambda: f32,
}

impl SplitElem {
    fn vert(v: usize) -> Self {
        Self { htype: BM_VERT, edge: BM_NIL, vert: v, lambda: 0.0 }
    }
    fn edge(e: usize, lambda: f32) -> Self {
        Self { htype: BM_EDGE, edge: e, vert: BM_NIL, lambda }
    }
}

fn bm_edgexvert_isect_impl(
    bm: &mut BMesh,
    v: usize,
    e: usize,
    co: Vec3,
    dir: Vec3,
    lambda: f32,
    dist_sq: f32,
    r_pair: &mut [SplitElem; 2],
) -> bool {
    if !(0.0..=1.0).contains(&lambda) {
        return false;
    }
    let (e_v, dist_sq_vert_factor) = if lambda < 0.5 {
        (bm.edge(e).v1, lambda)
    } else {
        (bm.edge(e).v2, 1.0 - lambda)
    };
    if v != e_v {
        let dist_sq_vert = dist_sq_vert_factor * dist_sq_vert_factor * dir.length_squared();
        if dist_sq_vert < dist_sq {
            return false;
        }
        let closest = co + dir * lambda;
        if bm.vpool[v].co.distance_squared(closest) < dist_sq {
            r_pair[0] = SplitElem::edge(e, lambda);
            r_pair[1] = SplitElem::vert(v);
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn bm_edgexedge_isect_impl(
    bm: &mut BMesh,
    e_a: usize,
    e_b: usize,
    co_a: Vec3,
    dir_a: Vec3,
    co_b: Vec3,
    dir_b: Vec3,
    lambda_a: f32,
    lambda_b: f32,
    dist_sq: f32,
    r_pair: &mut [SplitElem; 2],
) -> bool {
    let (e_a_v, dist_sq_va_factor) = if lambda_a < 0.5 {
        (bm.edge(e_a).v1, lambda_a)
    } else {
        (bm.edge(e_a).v2, 1.0 - lambda_a)
    };
    let (e_b_v, dist_sq_vb_factor) = if lambda_b < 0.5 {
        (bm.edge(e_b).v1, lambda_b)
    } else {
        (bm.edge(e_b).v2, 1.0 - lambda_b)
    };
    if e_a_v == e_b_v {
        return false;
    }
    if !(0.0..=1.0).contains(&lambda_a) || !(0.0..=1.0).contains(&lambda_b) {
        return false;
    }
    let dist_sq_va = dist_sq_va_factor * dist_sq_va_factor * dir_a.length_squared();
    let dist_sq_vb = dist_sq_vb_factor * dist_sq_vb_factor * dir_b.length_squared();
    if dist_sq_va < dist_sq || dist_sq_vb < dist_sq {
        return false;
    }
    let near_a = co_a + dir_a * lambda_a;
    let near_b = co_b + dir_b * lambda_b;
    if near_a.distance_squared(near_b) < dist_sq {
        r_pair[0] = SplitElem::edge(e_a, lambda_a);
        r_pair[1] = SplitElem::edge(e_b, lambda_b);
        return true;
    }
    false
}

/* `bm_vert_pair_share_best_splittable_face_cb` state. */
struct BestFaceData {
    edgenet: Vec<usize>,
    best_range: f32,
    best_face: usize,
}

fn bm_vert_pair_share_best_splittable_face(
    bm: &BMesh,
    f: usize,
    l_a: usize,
    l_b: usize,
    data: &mut BestFaceData,
) -> bool {
    let no = bm.face(f).no;
    let mut min = bm.vpool[bm.loo(l_a).v].co.dot(no);
    let mut max = bm.vpool[bm.loo(l_b).v].co.dot(no);
    if min > max {
        std::mem::swap(&mut min, &mut max);
    }
    let e_next = bm.edge(data.edgenet[1]);
    let mut v_test = if bm.edge(data.edgenet[0]).v1 == e_next.v1
        || bm.edge(data.edgenet[0]).v1 == e_next.v2
    {
        bm.edge(data.edgenet[0]).v2
    } else {
        bm.edge(data.edgenet[0]).v1
    };
    for i in 0..data.edgenet.len() - 1 {
        v_test = crate::structure::bm_edge_other_vert(bm, data.edgenet[i], v_test);
        if !crate::polygon::bm_face_point_inside_test(bm, f, bm.vpool[v_test].co) {
            return false;
        }
        let dot = bm.vpool[v_test].co.dot(no);
        min = min.min(dot);
        max = max.max(dot);
    }
    let range = max - min;
    if range < data.best_range {
        data.best_range = range;
        data.best_face = f;
    }
    false
}

fn bm_vert_pair_share_splittable_face(
    bm: &BMesh,
    l_a: usize,
    l_b: usize,
    v_a_co: Vec3,
    v_a_b_dir: Vec3,
) -> bool {
    let range_min = -f32::EPSILON;
    let range_max = 1.0 + f32::EPSILON;
    let co = bm.vpool[bm.loo(bm.loo(l_a).prev).v].co;
    let dir = bm.vpool[bm.loo(bm.loo(l_a).next).v].co - co;
    if let Some((_, lambda_b)) =
        blenders_math::geom::isect_ray_ray(v_a_co, v_a_b_dir, co, dir)
    {
        if (range_min..range_max).contains(&lambda_b) {
            return true;
        }
        let co = bm.vpool[bm.loo(bm.loo(l_b).prev).v].co;
        let dir = bm.vpool[bm.loo(bm.loo(l_b).next).v].co - co;
        if let Some((_, lambda_b)) =
            blenders_math::geom::isect_ray_ray(v_a_co, v_a_b_dir, co, dir)
        {
            return (range_min..range_max).contains(&lambda_b);
        }
    }
    false
}

/// `BM_vert_pair_shared_face_cb` (C signature: callback gets `(f, l_a, l_b)`).
fn vert_pair_shared_face_cb(
    bm: &BMesh,
    v_a: usize,
    v_b: usize,
    allow_adjacent: bool,
    callback: &mut dyn FnMut(&BMesh, usize, usize, usize) -> bool,
) -> usize {
    if bm.vert(v_a).e == BM_NIL || bm.vert(v_b).e == BM_NIL {
        return BM_NIL;
    }
    for l_a in LoopsOfVert::new(bm, v_a) {
        let f = bm.loo(l_a).f;
        let l_b = crate::query::bm_face_vert_share_loop(bm, f, v_b);
        if l_b != BM_NIL
            && (allow_adjacent || !crate::mods::bm_loop_is_adjacent(bm, l_a, l_b))
            && callback(bm, f, l_a, l_b)
        {
            return f;
        }
    }
    BM_NIL
}

fn bm_vert_pair_best_face_get(
    bm: &BMesh,
    v_a: usize,
    v_b: usize,
    edgenet: &[usize],
    epsilon: f32,
) -> usize {
    debug_assert!(v_a != v_b);
    if edgenet.len() == 1 {
        let v_b_co = bm.vpool[v_b].co;
        let dir = bm.vpool[v_a].co - v_b_co;
        let mut cb = |bm: &BMesh, _f: usize, l_a: usize, l_b: usize| {
            bm_vert_pair_share_splittable_face(bm, l_a, l_b, v_b_co, dir)
        };
        let best_face = vert_pair_shared_face_cb(bm, v_a, v_b, false, &mut cb);
        debug_assert!(best_face == BM_NIL
            || !crate::query::bm_edge_in_face(bm, edgenet[0], best_face));
        best_face
    } else {
        let mut data = BestFaceData {
            edgenet: edgenet.to_vec(),
            best_range: f32::MAX,
            best_face: BM_NIL,
        };
        {
            let mut cb = |bm: &BMesh, f: usize, l_a: usize, l_b: usize| {
                bm_vert_pair_share_best_splittable_face(bm, f, l_a, l_b, &mut data)
            };
            vert_pair_shared_face_cb(bm, v_a, v_b, true, &mut cb);
        }
        if data.best_face != BM_NIL {
            let no = bm.face(data.best_face).no;
            let mut min = f32::MAX;
            let mut max = -f32::MAX;
            for v_test in VertsOfFace::new(bm, data.best_face) {
                let dot = bm.vpool[v_test].co.dot(no);
                min = min.min(dot);
                max = max.max(dot);
            }
            if max - min + 2.0 * epsilon < data.best_range {
                data.best_face = BM_NIL;
            }
        }
        data.best_face
    }
}

/// `BM_mesh_intersect_edges`. `r_targetmap` maps welded vert -> surviving vert.
/// Returns whether any faces were split.
pub fn bm_mesh_intersect_edges(
    bm: &mut BMesh,
    hflag: u8,
    dist: f32,
    split_faces: bool,
    mut r_targetmap: Option<&mut HashMap<usize, usize>>,
) -> bool {
    let mut ok = false;
    let dist_sq = dist * dist;
    let dist_half = dist / 2.0;
    let dist_sq_sq = dist_sq * dist_sq;

    crate::mesh::bm_mesh_elem_index_ensure(bm, BM_VERT | BM_EDGE);
    let mut vtable: Vec<usize> = Vec::with_capacity(bm.totvert as usize);
    let mut etable: Vec<usize> = Vec::with_capacity(bm.totedge as usize);
    for v in VertsOfMesh::new(bm) {
        vtable.push(v);
    }
    vtable.sort_by_key(|&v| bm.vpool[v].head.index);
    for e in EdgesOfMesh::new(bm) {
        etable.push(e);
    }
    etable.sort_by_key(|&e| bm.epool[e].head.index);

    /* pair_stack_vertxvert / pair_stack_edgexelem merged into flat pair list. */
    let mut pair_vertxvert: Vec<[SplitElem; 2]> = Vec::new();
    let mut pair_edgexelem: Vec<[SplitElem; 2]> = Vec::new();

    let mut verts_act_len = 0;
    let mut verts_remain_len = 0;
    for v in VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vpool[v].head.hflag & hflag != 0 {
            bm.vpool[v].head.hflag |= BM_ELEM_TAG;
            verts_act_len += 1;
        } else {
            bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
            if bm.vpool[v].head.hflag & BM_ELEM_HIDDEN == 0 {
                verts_remain_len += 1;
            }
        }
        bm.vpool[v].head.index = -1;
    }
    bm.elem_index_dirty = true;

    let mut tree_verts_act: Option<BVHTree> = (verts_act_len > 0)
        .then(|| bvhtree_new(verts_act_len, dist_half, KDOP_TREE_TYPE, KDOP_AXIS_LEN));
    let mut tree_verts_remain: Option<BVHTree> = (verts_remain_len > 0)
        .then(|| bvhtree_new(verts_remain_len, dist_half, KDOP_TREE_TYPE, KDOP_AXIS_LEN));
    if tree_verts_act.is_some() || tree_verts_remain.is_some() {
        for v in VertsOfMesh::new(bm) {
            let i = vtable.iter().position(|&x| x == v).unwrap() as i32;
            if bm.vpool[v].head.hflag & BM_ELEM_TAG != 0 {
                if let Some(t) = &mut tree_verts_act {
                    bvhtree_insert(t, i, &[bm.vpool[v].co.x, bm.vpool[v].co.y, bm.vpool[v].co.z], 1);
                }
            } else if bm.vpool[v].head.hflag & BM_ELEM_HIDDEN == 0 {
                if let Some(t) = &mut tree_verts_remain {
                    bvhtree_insert(t, i, &[bm.vpool[v].co.x, bm.vpool[v].co.y, bm.vpool[v].co.z], 1);
                }
            }
        }
        if let Some(t) = &mut tree_verts_act {
            bvhtree_balance(t);
            let vtable_ref = &vtable;
            let pairs = &mut pair_vertxvert;
            bvhtree_overlap_ex(
                t,
                t,
                Some(&mut |ia: i32, ib: i32, _thread: i32| {
                    if ia < ib {
                        pairs.push([
                            SplitElem::vert(vtable_ref[ia as usize]),
                            SplitElem::vert(vtable_ref[ib as usize]),
                        ]);
                    }
                    false
                }),
                1,
                BVH_OVERLAP_USE_THREADING | BVH_OVERLAP_SELF,
            );
        }
        if let Some(t) = &mut tree_verts_remain {
            bvhtree_balance(t);
        }
        if let (Some(tr), Some(ta)) = (&tree_verts_remain, &tree_verts_act) {
            let vtable_ref = &vtable;
            let pairs = &mut pair_vertxvert;
            bvhtree_overlap_ex(
                tr,
                ta,
                Some(&mut |ia: i32, ib: i32, _thread: i32| {
                    pairs.push([
                        SplitElem::vert(vtable_ref[ia as usize]),
                        SplitElem::vert(vtable_ref[ib as usize]),
                    ]);
                    false
                }),
                1,
                BVH_OVERLAP_USE_THREADING,
            );
        }
    }
    let _vertxvert_pair_len = pair_vertxvert.len();

    /* edges */
    let mut edges_act_len = 0;
    let mut edges_remain_len = 0;
    for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        if bm.epool[e].head.hflag & BM_ELEM_HIDDEN != 0
            || bm.vpool[v1].co.distance_squared(bm.vpool[v2].co) < dist_sq
        {
            bm.epool[e].head.index = 0;
            if split_faces {
                bm.epool[e].head.hflag |= BM_ELEM_TAG;
            }
            continue;
        }
        if bm.vpool[v1].head.hflag & BM_ELEM_TAG != 0
            || bm.vpool[v2].head.hflag & BM_ELEM_TAG != 0
        {
            bm.epool[e].head.index = EDGE_ACT_TO_TEST;
            edges_act_len += 1;
        } else {
            bm.epool[e].head.index = EDGE_REMAIN_TO_TEST;
            edges_remain_len += 1;
            if split_faces {
                bm.epool[e].head.hflag |= BM_ELEM_TAG;
            }
        }
    }

    let mut tree_edges_act: Option<BVHTree> = (edges_act_len > 0)
        .then(|| bvhtree_new(edges_act_len, dist_half, KDOP_TREE_TYPE, KDOP_AXIS_LEN));
    let mut tree_edges_remain: Option<BVHTree> = (edges_remain_len > 0
        && (tree_edges_act.is_some() || tree_verts_act.is_some()))
    .then(|| bvhtree_new(edges_remain_len, dist_half, KDOP_TREE_TYPE, KDOP_AXIS_LEN));

    if tree_edges_act.is_some() || tree_edges_remain.is_some() {
        for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
            let edge_test = bm.epool[e].head.index;
            let i = etable.iter().position(|&x| x == e).unwrap() as i32;
            if edge_test == EDGE_ACT_TO_TEST || edge_test == EDGE_REMAIN_TO_TEST {
                let mut co = [0.0f32; 6];
                co[..3].copy_from_slice(&[
                    bm.vpool[bm.edge(e).v1].co.x,
                    bm.vpool[bm.edge(e).v1].co.y,
                    bm.vpool[bm.edge(e).v1].co.z]);
                co[3..].copy_from_slice(&[
                    bm.vpool[bm.edge(e).v2].co.x,
                    bm.vpool[bm.edge(e).v2].co.y,
                    bm.vpool[bm.edge(e).v2].co.z]);
                bm.epool[e].head.index = 0;
                let t = if edge_test == EDGE_ACT_TO_TEST {
                    &mut tree_edges_act
                } else {
                    &mut tree_edges_remain
                };
                if let Some(t) = t {
                    bvhtree_insert(t, i, &co, 2);
                }
            }
            bm.epool[e].head.hflag &= !BM_ELEM_TAG;
        }
        bm.elem_index_dirty = true;
        if let Some(t) = &mut tree_edges_act {
            bvhtree_balance(t);
        }
        if let Some(t) = &mut tree_edges_remain {
            bvhtree_balance(t);
        }

        /* cut-count tracking via `head.index` on edges (as in C). */
        let mut cut_edges_len = 0i32;
        macro_rules! edge_bump {
            ($e:expr) => {{
                if bm.epool[$e].head.index == 0 {
                    cut_edges_len += 1;
                }
                bm.epool[$e].head.index += 1;
            }};
        }

        let mut edgexedge_pair_len = 0usize;
        if let Some(ta) = &tree_edges_act {
            {
                let etable_ref = &etable;
                let pairs = &mut pair_edgexelem;
                let mut tmp = Vec::new();
                bvhtree_overlap_ex(
                    ta,
                    ta,
                    Some(&mut |ia: i32, ib: i32, _t: i32| {
                        if ia < ib {
                            tmp.push((etable_ref[ia as usize], etable_ref[ib as usize]));
                        }
                        false
                    }),
                    1,
                    BVH_OVERLAP_USE_THREADING | BVH_OVERLAP_SELF,
                );
                for (e_a, e_b) in tmp {
                    if crate::query::bm_edge_share_vert_check(bm, e_a, e_b) {
                        continue;
                    }
                    let co_a = bm.vpool[bm.edge(e_a).v1].co;
                    let dir_a = bm.vpool[bm.edge(e_a).v2].co - co_a;
                    let co_b = bm.vpool[bm.edge(e_b).v1].co;
                    let dir_b = bm.vpool[bm.edge(e_b).v2].co - co_b;
                    if let Some((la, lb)) = blenders_math::geom::isect_ray_ray_epsilon(
                        co_a, dir_a, co_b, dir_b, dist_sq_sq,
                    ) {
                        let mut pair = [SplitElem::vert(BM_NIL); 2];
                        if bm_edgexedge_isect_impl(
                            bm, e_a, e_b, co_a, dir_a, co_b, dir_b, la, lb, dist_sq, &mut pair,
                        ) {
                            edge_bump!(e_a);
                            edge_bump!(e_b);
                            pairs.push(pair);
                        }
                    }
                }
            }
            if let Some(tr) = &tree_edges_remain {
                let etable_ref = &etable;
                let pairs = &mut pair_edgexelem;
                let mut tmp = Vec::new();
                bvhtree_overlap_ex(
                    tr,
                    ta,
                    Some(&mut |ia: i32, ib: i32, _t: i32| {
                        tmp.push((etable_ref[ia as usize], etable_ref[ib as usize]));
                        false
                    }),
                    1,
                    BVH_OVERLAP_USE_THREADING,
                );
                for (e_a, e_b) in tmp {
                    /* note: (tr, ta) order — a is remain, b is act. */
                    if crate::query::bm_edge_share_vert_check(bm, e_a, e_b) {
                        continue;
                    }
                    let co_a = bm.vpool[bm.edge(e_a).v1].co;
                    let dir_a = bm.vpool[bm.edge(e_a).v2].co - co_a;
                    let co_b = bm.vpool[bm.edge(e_b).v1].co;
                    let dir_b = bm.vpool[bm.edge(e_b).v2].co - co_b;
                    if let Some((la, lb)) = blenders_math::geom::isect_ray_ray_epsilon(
                        co_a, dir_a, co_b, dir_b, dist_sq_sq,
                    ) {
                        let mut pair = [SplitElem::vert(BM_NIL); 2];
                        if bm_edgexedge_isect_impl(
                            bm, e_a, e_b, co_a, dir_a, co_b, dir_b, la, lb, dist_sq, &mut pair,
                        ) {
                            edge_bump!(e_a);
                            edge_bump!(e_b);
                            pairs.push(pair);
                        }
                    }
                }
            }
            edgexedge_pair_len = pair_edgexelem.len();

            /* edge x vert against both vert trees. */
            for tree_v in [&tree_verts_act, &tree_verts_remain].into_iter().flatten() {
                let etable_ref = &etable;
                let vtable_ref = &vtable;
                let mut tmp = Vec::new();
                bvhtree_overlap_ex(
                    ta,
                    tree_v,
                    Some(&mut |ia: i32, ib: i32, _t: i32| {
                        tmp.push((etable_ref[ia as usize], vtable_ref[ib as usize]));
                        false
                    }),
                    1,
                    BVH_OVERLAP_USE_THREADING,
                );
                for (e, v) in tmp {
                    let co = bm.vpool[bm.edge(e).v1].co;
                    let dir = bm.vpool[bm.edge(e).v2].co - co;
                    let lambda = blenders_math::geom::ray_point_factor_ex(
                        bm.vpool[v].co, co, dir, 0.0, -1.0,
                    );
                    let mut pair = [SplitElem::vert(BM_NIL); 2];
                    if bm_edgexvert_isect_impl(
                        bm, v, e, co, dir, lambda, dist_sq, &mut pair,
                    ) {
                        edge_bump!(e);
                        pair_edgexelem.push(pair);
                    }
                }
            }
        }
        if let (Some(tv), Some(tr)) = (&tree_verts_act, &tree_edges_remain) {
            let etable_ref = &etable;
            let vtable_ref = &vtable;
            let mut tmp = Vec::new();
            bvhtree_overlap_ex(
                tr,
                tv,
                Some(&mut |ia: i32, ib: i32, _t: i32| {
                    tmp.push((etable_ref[ia as usize], vtable_ref[ib as usize]));
                    false
                }),
                1,
                BVH_OVERLAP_USE_THREADING,
            );
            for (e, v) in tmp {
                let co = bm.vpool[bm.edge(e).v1].co;
                let dir = bm.vpool[bm.edge(e).v2].co - co;
                let lambda = blenders_math::geom::ray_point_factor_ex(
                    bm.vpool[v].co, co, dir, 0.0, -1.0,
                );
                let mut pair = [SplitElem::vert(BM_NIL); 2];
                if bm_edgexvert_isect_impl(bm, v, e, co, dir, lambda, dist_sq, &mut pair) {
                    edge_bump!(e);
                    pair_edgexelem.push(pair);
                }
            }
        }

        let _ = cut_edges_len;
        let edgexelem_pair_len = pair_edgexelem.len();
        let edgexvert_pair_len = edgexelem_pair_len - edgexedge_pair_len;
        let _ = edgexvert_pair_len;

        if edgexelem_pair_len > 0 {
            /* flat list of edge-carrying pair elems (as `pair_flat`). */
            let pair_flat: Vec<SplitElem> = pair_edgexelem
                .iter()
                .flat_map(|p| p.iter().copied())
                .collect();
            /* group cut indices per edge (in first-seen order). */
            let mut e_map: Vec<(usize, Vec<usize>)> = Vec::new();
            let mut e_lookup: HashMap<usize, usize> = HashMap::new();
            for (i, pe) in pair_flat.iter().enumerate() {
                if pe.htype != BM_EDGE {
                    continue;
                }
                let e = pe.edge;
                if let Some(&slot) = e_lookup.get(&e) {
                    e_map[slot].1.push(i);
                } else {
                    e_lookup.insert(e, e_map.len());
                    e_map.push((e, vec![i]));
                }
            }
            /* split each edge once per cut, adjusting lambda sequentially. */
            let mut created: HashMap<usize, usize> = HashMap::new();
            for (e, cuts) in &mut e_map {
                cuts.sort_by(|&a, &b| pair_flat[a].lambda.total_cmp(&pair_flat[b].lambda));
                let mut lambda_prev = 0.0f32;
                let mut e_cur = *e;
                for &index in cuts.iter() {
                    let pe = pair_flat[index];
                    let lambda = (pe.lambda - lambda_prev) / (1.0 - lambda_prev);
                    lambda_prev = pe.lambda;
                    if split_faces {
                        bm.epool[e_cur].head.hflag &= !BM_ELEM_TAG;
                    }
                    let v_new = crate::mods::bm_edge_split(
                        bm, e_cur, bm.edge(e_cur).v1, None, lambda,
                    );
                    created.insert(index, v_new);
                    if bm.edge(e_cur).l != BM_NIL {
                        let l_start = bm.edge(e_cur).l;
                        let mut l_iter = l_start;
                        loop {
                            crate::normals::bm_face_calc_normals(bm, bm.loo(l_iter).f);
                            l_iter = bm.loo(l_iter).radial_next;
                            if l_iter == l_start {
                                break;
                            }
                        }
                    }
                }
            }
            /* C writes `pair_elem->vert = v_new` into pair_flat — the edge elem
             * becomes a vert elem so the targetmap weld below uses it. */
            for (flat_i, p) in pair_edgexelem.iter_mut().flat_map(|p| p.iter_mut()).enumerate() {
                if p.htype == BM_EDGE {
                    p.htype = BM_VERT;
                    p.vert = created[&flat_i];
                }
            }
        }
    }

    drop(tree_verts_act);
    drop(tree_verts_remain);
    drop(tree_edges_act);
    drop(tree_edges_remain);

    /* Merge all pairs (vert pairs first, then edgexelem verts). */
    let mut pair_array: Vec<[SplitElem; 2]> = Vec::new();
    pair_array.extend_from_slice(&pair_vertxvert);
    pair_array.extend_from_slice(&pair_edgexelem);
    let pair_len = pair_array.len();

    if let Some(targetmap) = r_targetmap.as_deref_mut() {
        if pair_len > 0 {
            for pair in &pair_array {
                let v_key = pair[0].vert;
                let v_val = pair[1].vert;
                if v_key == BM_NIL || v_val == BM_NIL || v_key == v_val {
                    continue;
                }
                targetmap.insert(v_key, v_val);
            }
            for (i, pair) in pair_array.iter().enumerate() {
                let v_key = pair[0].vert;
                let mut v_val = pair[1].vert;
                if v_key == BM_NIL || v_val == BM_NIL {
                    continue;
                }
                while let Some(&v_target) = targetmap.get(&v_val) {
                    v_val = v_target;
                }
                if v_val != pair[1].vert {
                    targetmap.insert(v_key, v_val);
                }
                if split_faces {
                    bm.vpool[v_key].head.index = (i * 2) as i32;
                    bm.vpool[v_val].head.index = (i * 2 + 1) as i32;
                }
            }
            if split_faces {
                /* pair_flat for cut vert lookup. */
                let pair_flat: Vec<SplitElem> = pair_array
                    .iter()
                    .flat_map(|p| p.iter().copied())
                    .collect();
                for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
                    if bm.epool[e].head.hflag & BM_ELEM_TAG != 0 {
                        continue;
                    }
                    let mut va = bm.edge(e).v1;
                    let mut vb = bm.edge(e).v2;
                    let mut v_cut = bm.vpool[va].head.index;
                    let mut v_cut_other = bm.vpool[vb].head.index;
                    if v_cut == -1 && v_cut_other == -1 {
                        if bm.vpool[va].head.hflag & BM_ELEM_TAG == 0
                            && bm.vpool[vb].head.hflag & BM_ELEM_TAG == 0
                        {
                            bm.epool[e].head.hflag |= BM_ELEM_TAG;
                        }
                        continue;
                    }
                    bm.epool[e].head.hflag |= BM_ELEM_TAG;
                    if v_cut == -1 {
                        std::mem::swap(&mut va, &mut vb);
                        v_cut = v_cut_other;
                        v_cut_other = -1;
                    }
                    v_cut += if v_cut % 2 != 0 { -1 } else { 1 };
                    let va_dest = pair_flat[v_cut as usize].vert;
                    if crate::query::bm_vert_pair_share_face_check(bm, va, va_dest) {
                        continue;
                    }
                    let mut best_face = BM_NIL;
                    let mut v_other = vb;
                    let mut e_net = e;
                    let mut edgenet: Vec<usize> = Vec::new();
                    loop {
                        let v_other_dest = if v_cut_other != -1 {
                            v_cut_other += if v_cut_other % 2 != 0 { -1 } else { 1 };
                            let vod = pair_flat[v_cut_other as usize].vert;
                            if crate::query::bm_vert_pair_share_face_check(bm, v_other, vod) {
                                break;
                            }
                            vod
                        } else {
                            v_other
                        };
                        if va_dest == v_other_dest {
                            break;
                        }
                        if edgenet.is_empty()
                            && crate::query::bm_edge_exists(bm, va_dest, v_other_dest) != BM_NIL
                        {
                            break;
                        }
                        edgenet.push(e_net);
                        best_face = bm_vert_pair_best_face_get(
                            bm, va_dest, v_other_dest, &edgenet, dist,
                        );
                        if best_face != BM_NIL {
                            if va_dest != va
                                && crate::query::bm_edge_exists(bm, va_dest, va) == BM_NIL
                            {
                                let e0 = edgenet[0];
                                let vb_new = if edgenet.len() > 1 {
                                    crate::structure::bm_edge_other_vert(bm, e0, va)
                                } else {
                                    v_other_dest
                                };
                                edgenet[0] = crate::core::bm_edge_create(
                                    bm, va_dest, vb_new, Some(e0), 0,
                                );
                            }
                            if edgenet.len() > 1
                                && v_other_dest != v_other
                                && crate::query::bm_edge_exists(bm, v_other_dest, v_other)
                                    == BM_NIL
                            {
                                let li = edgenet.len() - 1;
                                let el = edgenet[li];
                                let other = crate::structure::bm_edge_other_vert(bm, el, v_other);
                                edgenet[li] = crate::core::bm_edge_create(
                                    bm, v_other_dest, other, Some(el), 0,
                                );
                            }
                            break;
                        }
                        /* walk to next edge on `v_other`'s disk. */
                        let mut e_test = e_net;
                        let mut e_next = BM_NIL;
                        loop {
                            e_test = crate::structure::bmesh_disk_edge_next(bm, e_test, v_other);
                            if e_test == e_net {
                                break;
                            }
                            if bm.edge(e_test).l != BM_NIL {
                                if bm.edge(e_test).head.hflag & BM_ELEM_TAG != 0 {
                                    continue;
                                }
                                if bm.vpool[bm.edge(e_test).v1].head.hflag & BM_ELEM_TAG == 0
                                    && bm.vpool[bm.edge(e_test).v2].head.hflag & BM_ELEM_TAG == 0
                                {
                                    continue;
                                }
                                bm.epool[e_test].head.hflag |= BM_ELEM_TAG;
                            } else if bm.edge(e_net).l != BM_NIL {
                                continue;
                            }
                            e_next = e_test;
                            break;
                        }
                        if e_next == BM_NIL {
                            break;
                        }
                        e_net = e_next;
                        v_other = crate::structure::bm_edge_other_vert(bm, e_net, v_other);
                        if v_other == va {
                            break;
                        }
                        v_cut_other = bm.vpool[v_other].head.index;
                    }
                    if best_face != BM_NIL {
                        let face_arr =
                            crate::edgenet::bm_face_split_edgenet(bm, best_face, &edgenet);
                        for face in face_arr {
                            crate::normals::bm_face_calc_normals(bm, face);
                        }
                    }
                }
            }
            ok = true;
        }
    }
    ok
}
