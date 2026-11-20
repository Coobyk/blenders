//! Port of `bmesh_polygon_edgenet.cc` — `BM_face_split_edgenet` and
//! `BM_face_split_edgenet_connect_islands`.
//!
//! api_flag bits used here (all cleared on exit): FACE_NET=0, EDGE_NET=1,
//! VERT_VISIT=2, VERT_IN_QUEUE=3. hflag bits: INTERNAL_TAG for the
//! stack/array/valid marks, SELECT_UV for EDGE_IS_NEW.
//! CustomData interpolation of new face loops is PORT_PENDING.
//! The BVH over `edge_arr` is replaced by a linear scan (same semantics).

use crate::structure;
use crate::structure::*;
use std::collections::VecDeque;
use crate::{delete, query, construct, core};
use crate::{BMesh, BM_NIL, BM_ELEM_SELECT_UV, BM_ELEM_INTERNAL_TAG, BM_ELEM_SELECT};
use blenders_math::geom;
use blenders_math::matrix::Mat3;
use blenders_math::matrix_c::mul_v2_m3v3;
use blenders_math::vector::{angle_signed_on_axis, Vec2, Vec3};

const FACE_NET: u8 = 0;
const EDGE_NET: u8 = 1;
const VERT_VISIT: u8 = 2;
const VERT_IN_QUEUE: u8 = 3;

#[inline]
fn api(bm: &BMesh, htype: u8, i: usize) -> u8 {
    match htype {
        crate::BM_VERT => bm.vert(i).head.api_flag,
        crate::BM_EDGE => bm.edge(i).head.api_flag,
        _ => bm.face(i).head.api_flag,
    }
}
#[inline]
fn api_enable(bm: &mut BMesh, htype: u8, i: usize, bit: u8) {
    match htype {
        crate::BM_VERT => bm.vpool[i].head.api_flag |= 1 << bit,
        crate::BM_EDGE => bm.epool[i].head.api_flag |= 1 << bit,
        _ => bm.fpool[i].head.api_flag |= 1 << bit,
    }
}
#[inline]
fn api_disable(bm: &mut BMesh, htype: u8, i: usize, bit: u8) {
    match htype {
        crate::BM_VERT => bm.vpool[i].head.api_flag &= !(1 << bit),
        crate::BM_EDGE => bm.epool[i].head.api_flag &= !(1 << bit),
        _ => bm.fpool[i].head.api_flag &= !(1 << bit),
    }
}
#[inline]
fn api_test(bm: &BMesh, htype: u8, i: usize, bit: u8) -> bool {
    api(bm, htype, i) & (1 << bit) != 0
}

/// `BM_edge_is_wire` — no face uses this edge.
#[inline]
pub fn edge_is_wire(bm: &BMesh, e: usize) -> bool {
    bm.edge(e).l == BM_NIL
}

/// `bm_edge_flagged_radial_count` — radial loops on `e` whose face is FACE_NET.
fn edge_flagged_radial_count(bm: &BMesh, e: usize) -> usize {
    let mut count = 0;
    let mut l = bm.edge(e).l;
    if l != BM_NIL {
        let first = l;
        loop {
            if api_test(bm, crate::BM_FACE, bm.loo(l).f, FACE_NET) {
                count += 1;
            }
            l = bm.loo(l).radial_next;
            if l == first {
                break;
            }
        }
    }
    count
}

/// `bm_edge_flagged_radial_first`.
fn edge_flagged_radial_first(bm: &BMesh, e: usize) -> usize {
    let mut l = bm.edge(e).l;
    if l != BM_NIL {
        let first = l;
        loop {
            if api_test(bm, crate::BM_FACE, bm.loo(l).f, FACE_NET) {
                return l;
            }
            l = bm.loo(l).radial_next;
            if l == first {
                break;
            }
        }
    }
    BM_NIL
}

/// `normalize_v2_m3_v3v3`.
fn normalize_v2_m3_v3v3(axis_mat: &Mat3, v1: Vec3, v2: Vec3) -> Vec2 {
    let dir = v1 - v2;
    let mut out = mul_v2_m3v3(*axis_mat, dir);
    if out.length_squared() > 0.0 {
        out = out.normalized();
    }
    out
}

/// `bm_face_split_edgenet_find_loop_pair` — pick 2 edges at `v_init` that
/// can seed a new face loop: a boundary edge and the best wire/boundary
/// continuation, ordered by winding.
fn find_loop_pair(
    bm: &BMesh,
    v_init: usize,
    face_normal: Vec3,
    face_normal_matrix: &Mat3,
) -> Option<(usize, usize)> {
    let mut edges_boundary: Vec<usize> = Vec::new();
    let mut edges_wire: Vec<usize> = Vec::new();
    if bm.vert(v_init).e != BM_NIL {
        let first = bm.vert(v_init).e;
        let mut e = first;
        loop {
            if api_test(bm, crate::BM_EDGE, e, EDGE_NET) {
                match edge_flagged_radial_count(bm, e) {
                    1 => edges_boundary.push(e),
                    0 => edges_wire.push(e),
                    _ => {}
                }
            }
            e = bmesh_disk_edge_next(bm, e, v_init);
            if e == first {
                break;
            }
        }
    }
    if edges_boundary.is_empty() {
        return None;
    }
    let e0 = edges_boundary.pop().unwrap();
    let mut edges_search: Vec<usize> = Vec::new();
    let e1;
    if edges_wire.is_empty() {
        if edges_boundary.is_empty() {
            return None;
        }
        e1 = edges_boundary.pop().unwrap();
        if !edges_boundary.is_empty() {
            edges_search = edges_boundary;
        }
    } else {
        e1 = edges_wire.pop().unwrap();
        if !edges_wire.is_empty() {
            edges_search = edges_wire;
        }
    }
    let mut e1 = e1;
    if !edges_search.is_empty() {
        let v_prev = structure::bm_edge_other_vert(bm, e0, v_init);
        let mut v_next = structure::bm_edge_other_vert(bm, e1, v_init);
        let dir_prev = normalize_v2_m3_v3v3(face_normal_matrix, bm.vert(v_prev).co, bm.vert(v_init).co);
        let dir_next = normalize_v2_m3_v3v3(face_normal_matrix, bm.vert(v_next).co, bm.vert(v_init).co);
        let mut angle_best_cos = dir_next.dot(dir_prev);
        for e in edges_search {
            v_next = structure::bm_edge_other_vert(bm, e, v_init);
            let dir_test =
                normalize_v2_m3_v3v3(face_normal_matrix, bm.vert(v_next).co, bm.vert(v_init).co);
            let angle_test_cos = dir_prev.dot(dir_test);
            if angle_test_cos > angle_best_cos {
                angle_best_cos = angle_test_cos;
                e1 = e;
            }
        }
    }
    /* flip based on winding */
    let l_walk = edge_flagged_radial_first(bm, e0);
    let mut swap = false;
    if l_walk != BM_NIL {
        if face_normal == bm.face(bm.loo(l_walk).f).no {
            swap = !swap;
        }
        if bm.loo(l_walk).v != v_init {
            swap = !swap;
        }
    }
    Some(if swap { (e1, e0) } else { (e0, e1) })
}

/// `bm_face_split_edgenet_find_loop_pair_exists` — cheap version of the check.
fn find_loop_pair_exists(bm: &BMesh, v_init: usize) -> bool {
    let mut edges_boundary_len = 0;
    let mut edges_wire_len = 0;
    if bm.vert(v_init).e != BM_NIL {
        let first = bm.vert(v_init).e;
        let mut e = first;
        loop {
            if api_test(bm, crate::BM_EDGE, e, EDGE_NET) {
                match edge_flagged_radial_count(bm, e) {
                    1 => edges_boundary_len += 1,
                    0 => edges_wire_len += 1,
                    _ => {}
                }
            }
            e = bmesh_disk_edge_next(bm, e, v_init);
            if e == first {
                break;
            }
        }
    }
    !(edges_boundary_len == 0 || (edges_wire_len == 0 && edges_boundary_len == 1))
}

/// `bm_face_split_edgenet_find_loop_walk` — greedy DFS from `e_pair.0` to
/// `e_pair.1`'s far vert. On success the parent chain is encoded by each
/// vert's `e` field (like the C code) so the caller can walk back.
fn find_loop_walk(bm: &mut BMesh, v_init: usize, face_normal: Vec3, e_pair: (usize, usize)) -> bool {
    let mut vert_stack: Vec<usize> = Vec::new();
    let mut vert_stack_next: Vec<usize> = Vec::new();
    let mut vert_visit: Vec<usize> = Vec::new();
    let mut edge_order: Vec<(f32, usize)> = Vec::new();
    let mut found = false;

    let v = structure::bm_edge_other_vert(bm, e_pair.0, v_init);
    bm.vpool[v].e = e_pair.0; /* parent edge */
    vert_stack.push(v);
    let v_dst = structure::bm_edge_other_vert(bm, e_pair.1, v_init);

    'outer: while let Some(mut v) = vert_stack.pop() {
        loop {
            if v == v_dst {
                found = true;
                break 'outer;
            }
            edge_order.clear();
            let e_first = bm.vert(v).e;
            let mut e_next = bmesh_disk_edge_next(bm, e_first, v);
            if e_next != e_first {
                loop {
                    if api_test(bm, crate::BM_EDGE, e_next, EDGE_NET)
                        && edge_flagged_radial_count(bm, e_next) < 2
                    {
                        let v_next = structure::bm_edge_other_vert(bm, e_next, v);
                        if !api_test(bm, crate::BM_VERT, v_next, VERT_VISIT) {
                            edge_order.push((0.0, v_next));
                            bm.vpool[v_next].e = e_next; /* parent edge */
                        }
                    }
                    e_next = bmesh_disk_edge_next(bm, e_next, v);
                    if e_next == e_first {
                        break;
                    }
                }
            }
            if edge_order.len() == 1 {
                /* fast-path: no fork, don't tag visited */
                v = edge_order[0].1;
                edge_order.clear();
                continue;
            }
            if edge_order.len() > 1 {
                let v_prev = structure::bm_edge_other_vert(bm, e_first, v);
                for o in &mut edge_order {
                    o.0 = angle_signed_on_axis(
                        bm.vert(v_prev).co - bm.vert(v).co,
                        bm.vert(o.1).co - bm.vert(v).co,
                        face_normal,
                    );
                }
                edge_order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                /* only tag forks */
                vert_visit.push(v);
                api_enable(bm, crate::BM_VERT, v, VERT_VISIT);
            }
            for &(_, vn) in edge_order.iter().rev() {
                vert_stack_next.push(vn);
            }
            edge_order.clear();
            if !vert_stack_next.is_empty() {
                std::mem::swap(&mut vert_stack, &mut vert_stack_next);
            }
            break;
        }
    }
    for v in vert_visit {
        api_disable(bm, crate::BM_VERT, v, VERT_VISIT);
    }
    found
}

/// `bm_face_split_edgenet_find_loop` — find a closed loop of EDGE_NET edges
/// starting at `v_init`; returns (verts, edges, check_face_exists).
fn find_loop(
    bm: &mut BMesh,
    v_init: usize,
    face_normal: Vec3,
    face_normal_matrix: &Mat3,
) -> Option<(Vec<usize>, Vec<usize>, bool)> {
    let e_pair = find_loop_pair(bm, v_init, face_normal, face_normal_matrix)?;
    debug_assert!(
        edge_flagged_radial_count(bm, e_pair.0) == 1 || edge_flagged_radial_count(bm, e_pair.1) == 1
    );
    if !find_loop_walk(bm, v_init, face_normal, e_pair) {
        return None;
    }
    let mut check_face_exists = true;
    let mut v = structure::bm_edge_other_vert(bm, e_pair.1, v_init);
    let mut face_verts = vec![v_init];
    let mut face_edges = vec![e_pair.1];
    if edge_is_wire(bm, e_pair.1) {
        check_face_exists = false;
    }
    loop {
        face_verts.push(v);
        let e = bm.vert(v).e;
        face_edges.push(e);
        if check_face_exists && edge_is_wire(bm, e) {
            check_face_exists = false;
        }
        v = structure::bm_edge_other_vert(bm, e, v);
        if v == v_init {
            break;
        }
    }
    if face_verts.len() < 3 {
        return None;
    }
    Some((face_verts, face_edges, check_face_exists))
}

/// `BM_face_split_edgenet` — tessellate face `f` around `edge_net`.
/// Returns newly created faces (first entry is `f` itself when any were made).
pub fn bm_face_split_edgenet(bm: &mut BMesh, f: usize, edge_net: &[usize]) -> Vec<usize> {
    if edge_net.is_empty() {
        return Vec::new();
    }
    let f_len = bm.face(f).len;
    api_enable(bm, crate::BM_FACE, f, FACE_NET);
    for &e in edge_net {
        api_enable(bm, crate::BM_EDGE, e, EDGE_NET);
        api_disable(bm, crate::BM_VERT, bm.edge(e).v1, VERT_IN_QUEUE);
        api_disable(bm, crate::BM_VERT, bm.edge(e).v2, VERT_IN_QUEUE);
    }
    /* face boundary edges join the net */
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        api_enable(bm, crate::BM_EDGE, bm.loo(l).e, EDGE_NET);
        api_disable(bm, crate::BM_VERT, bm.loo(l).v, VERT_IN_QUEUE);
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    let face_normal_matrix = geom::axis_dominant_to_m3(bm.face(f).no);
    let f_no = bm.face(f).no;
    let mut vert_queue: Vec<usize> = Vec::with_capacity(f_len as usize + edge_net.len());
    vert_queue.push(bm.loo(l_first).v);
    api_enable(bm, crate::BM_VERT, bm.loo(l_first).v, VERT_IN_QUEUE);

    let mut face_arr: Vec<usize> = Vec::new();
    while let Some(v) = vert_queue.pop() {
        api_disable(bm, crate::BM_VERT, v, VERT_IN_QUEUE);
        if let Some((face_verts, face_edges, check_face_exists)) =
            find_loop(bm, v, f_no, &face_normal_matrix)
        {
            let mut f_new = BM_NIL;
            if !(check_face_exists && query::bm_face_exists(bm, &face_verts) != BM_NIL) {
                f_new = core::bm_face_create(bm, &face_verts, &face_edges, Some(f), 0);
            }
            if f_new != BM_NIL {
                face_arr.push(f_new);
                bm.fpool[f_new].no = f_no;
                bm.fpool[f_new].head.hflag = bm.face(f).head.hflag;
                if bm.face(f).head.hflag & BM_ELEM_SELECT != 0 {
                    bm.totfacesel += 1;
                }
                api_enable(bm, crate::BM_FACE, f_new, FACE_NET);
                /* queue new verts that could seed another loop */
                let nf_first = bm.face(f_new).l_first;
                let mut li = nf_first;
                loop {
                    let lv = bm.loo(li).v;
                    if !api_test(bm, crate::BM_VERT, lv, VERT_IN_QUEUE)
                        && find_loop_pair_exists(bm, lv)
                    {
                        vert_queue.push(lv);
                        api_enable(bm, crate::BM_VERT, lv, VERT_IN_QUEUE);
                    }
                    li = bm.loo(li).next;
                    if li == nf_first {
                        break;
                    }
                }
            }
        }
    }
    /* PORT_PENDING: CustomData interpolation for interior loops. */

    /* cleanup flags */
    for &e in edge_net {
        api_disable(bm, crate::BM_EDGE, e, EDGE_NET);
        api_disable(bm, crate::BM_VERT, bm.edge(e).v1, VERT_VISIT);
        api_disable(bm, crate::BM_VERT, bm.edge(e).v2, VERT_VISIT);
    }
    let mut l = l_first;
    loop {
        api_disable(bm, crate::BM_EDGE, bm.loo(l).e, EDGE_NET);
        api_disable(bm, crate::BM_VERT, bm.loo(l).v, VERT_VISIT);
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    if !face_arr.is_empty() {
        core::bmesh_face_swap_data(bm, f, face_arr[0]);
        core::bm_face_kill(bm, face_arr[0]);
        face_arr[0] = f;
    } else {
        api_disable(bm, crate::BM_FACE, f, FACE_NET);
    }
    face_arr
}

/* -------------------------------------------------------------------- */
/* connect islands                                                      */

/// `edge_isect_verts_point_2d` — does segment v_a→v_b cross edge `e` in 2D?
fn edge_isect_verts_point_2d(bm: &BMesh, e: usize, v_a: usize, v_b: usize, r_isect: &mut Vec2) -> bool {
    const ENDPOINT_BIAS: f32 = 1e-4;
    let e_v1 = bm.edge(e).v1;
    let e_v2 = bm.edge(e).v2;
    geom::isect_seg_seg_v2_point_ex(
        bm.vert(v_a).co.xy(),
        bm.vert(v_b).co.xy(),
        bm.vert(e_v1).co.xy(),
        bm.vert(e_v2).co.xy(),
        ENDPOINT_BIAS,
        r_isect,
    ) == 1
        && e_v1 != v_a
        && e_v2 != v_a
        && e_v1 != v_b
        && e_v2 != v_b
}

/// `axis_pt_cmp` — lexicographic compare.
fn axis_pt_cmp(a: [f32; 2], b: [f32; 2]) -> i32 {
    if a[0] < b[0] {
        return -1;
    }
    if a[0] > b[0] {
        return 1;
    }
    if a[1] < b[1] {
        return -1;
    }
    if a[1] > b[1] {
        return 1;
    }
    0
}

/// `EdgeGroupIsland` — one connected component of the edge array.
struct EdgeGroupIsland {
    edges: Vec<usize>,
    vert_len: usize,
    has_prev_edge: bool,
    vert_span_min: usize,
    vert_span_max: usize,
    min_axis: [f32; 2],
    max_axis: [f32; 2],
}

struct FindConnectionArgs<'a> {
    edge_arr: &'a [usize],
    edge_arr_new_len: usize,
    vert_range: [usize; 2],
}

/// `test_edges_isect_2d_vert` — nearest edge hit between v_origin→v_other.
/// (Linear scan replacing the BVH ray-cast; identical results.)
fn test_edges_isect_2d_vert(
    bm: &BMesh,
    args: &FindConnectionArgs,
    edge_arr_new: &[usize],
    v_origin: usize,
    v_other: usize,
) -> usize {
    let mut r_isect = Vec2::ZERO;
    let len = (bm.vert(v_other).co.xy() - bm.vert(v_origin).co.xy()).length();
    let mut hit_dist = len;
    let mut e_hit = BM_NIL;
    for (i, &e) in args.edge_arr.iter().enumerate() {
        let _ = i;
        if edge_isect_verts_point_2d(bm, e, v_origin, v_other, &mut r_isect) {
            let t = geom::line_point_factor_v2(
                r_isect,
                bm.vert(v_origin).co.xy(),
                bm.vert(v_other).co.xy(),
            );
            let dist_new = len * t;
            if dist_new < hit_dist && dist_new > 0.0 {
                let v1_index = bm.edge(e).v1;
                let v1_index = bm.vert(v1_index).head.index as usize;
                if v1_index < args.vert_range[0] || v1_index >= args.vert_range[1] {
                    hit_dist = dist_new;
                    e_hit = e;
                }
            }
        }
    }
    if e_hit == BM_NIL {
        let mut t_best = 1.0f32;
        for &e in &edge_arr_new[..args.edge_arr_new_len] {
            if edge_isect_verts_point_2d(bm, e, v_origin, v_other, &mut r_isect) {
                let t_test = geom::line_point_factor_v2(
                    r_isect,
                    bm.vert(v_origin).co.xy(),
                    bm.vert(v_other).co.xy(),
                );
                if t_test < t_best {
                    t_best = t_test;
                    e_hit = e;
                }
            }
        }
    }
    e_hit
}

/// `test_edges_isect_2d_ray` — nearest edge hit along `dir` from `v_origin`.
fn test_edges_isect_2d_ray(
    bm: &BMesh,
    args: &FindConnectionArgs,
    edge_arr_new: &[usize],
    v_origin: usize,
    dir: Vec2,
) -> usize {
    let mut hit_dist = f32::MAX;
    let mut e_hit = BM_NIL;
    for &e in args.edge_arr {
        let (e_v1, e_v2) = (bm.edge(e).v1, bm.edge(e).v2);
        if let Some((dist_new, _)) = geom::isect_ray_seg_v2(
            bm.vert(v_origin).co.xy(),
            dir,
            bm.vert(e_v1).co.xy(),
            bm.vert(e_v2).co.xy(),
        ) {
            if dist_new < hit_dist && dist_new > 0.0 {
                if e_v1 != v_origin && e_v2 != v_origin {
                    let v1_index = bm.vert(e_v1).head.index as usize;
                    if v1_index < args.vert_range[0] || v1_index >= args.vert_range[1] {
                        hit_dist = dist_new;
                        e_hit = e;
                    }
                }
            }
        }
    }
    for &e in &edge_arr_new[..args.edge_arr_new_len] {
        let (e_v1, e_v2) = (bm.edge(e).v1, bm.edge(e).v2);
        if let Some((dist_new, _)) = geom::isect_ray_seg_v2(
            bm.vert(v_origin).co.xy(),
            dir,
            bm.vert(e_v1).co.xy(),
            bm.vert(e_v2).co.xy(),
        ) {
            if e_v1 != v_origin && e_v2 != v_origin && dist_new < hit_dist {
                hit_dist = dist_new;
                e_hit = e;
            }
        }
    }
    e_hit
}

const SORT_AXIS: usize = 0;

/// `bm_face_split_edgenet_find_connection` — vertex index reachable from
/// `v_origin` along ±SORT_AXIS, or -1.
fn find_connection(
    bm: &BMesh,
    args: &FindConnectionArgs,
    edge_arr_new: &[usize],
    v_origin: usize,
    direction_sign: bool,
) -> i32 {
    let mut dir = Vec2::ZERO;
    dir[SORT_AXIS] = if direction_sign { 1.0 } else { -1.0 };
    let mut e_hit = test_edges_isect_2d_ray(bm, args, edge_arr_new, v_origin, dir);
    let mut v_other = BM_NIL;
    if e_hit != BM_NIL {
        let mut vert_search: Vec<usize> = Vec::new();
        let mut vert_blacklist: Vec<usize> = Vec::new();
        let mut v_other_fallback = BM_NIL;
        loop {
            let (e_v1, e_v2) = (bm.edge(e_hit).v1, bm.edge(e_hit).v2);
            let v_pair = if (bm.vert(v_origin).co.xy() - bm.vert(e_v1).co.xy()).length_squared()
                > (bm.vert(v_origin).co.xy() - bm.vert(e_v2).co.xy()).length_squared()
            {
                [e_v1, e_v2]
            } else {
                [e_v2, e_v1]
            };
            for v_iter in v_pair {
                if bm.vert(v_iter).head.hflag & BM_ELEM_INTERNAL_TAG != 0 {
                    let ok = if direction_sign {
                        bm.vert(v_iter).co[SORT_AXIS] > bm.vert(v_origin).co[SORT_AXIS]
                    } else {
                        bm.vert(v_iter).co[SORT_AXIS] < bm.vert(v_origin).co[SORT_AXIS]
                    };
                    if ok {
                        vert_search.push(v_iter);
                        vert_blacklist.push(v_iter);
                    }
                }
            }
            v_other_fallback = v_other;
            match vert_search.pop() {
                Some(v) => {
                    v_other = v;
                    /* mark visited via blacklist removal later */
                    let idx = vert_blacklist.iter().rposition(|&x| x == v);
                    if let Some(i) = idx {
                        vert_blacklist.remove(i);
                    }
                    e_hit = test_edges_isect_2d_vert(bm, args, edge_arr_new, v_origin, v_other);
                    if e_hit == BM_NIL {
                        break;
                    }
                }
                None => {
                    v_other = BM_NIL;
                    break;
                }
            }
        }
        if v_other == BM_NIL {
            v_other = v_other_fallback;
        }
        /* VERT_IS_VALID blacklist flags were never toggled — nothing to reset */
    }
    if v_other != BM_NIL {
        bm.vert(v_other).head.index
    } else {
        -1
    }
}

/// `bm_vert_partial_connect_check_overlap` — remap would make a double edge.
fn partial_connect_check_overlap(remap: &[i32], a: usize, b: usize) -> bool {
    remap[a] == b as i32 || remap[b] == a as i32
}

/// `test_tagged_and_notface` — edge tagged & not in face `f`.
fn test_tagged_and_notface(bm: &BMesh, e: usize, f: usize) -> bool {
    bm.edge(e).head.hflag & BM_ELEM_INTERNAL_TAG != 0 && !query::bm_edge_in_face(bm, e, f)
}

/// `bm_face_split_edgenet_partial_connect` — split a delimiting vert in two.
/// All edges/verts must have INTERNAL_TAG hflag enabled.
fn partial_connect(bm: &mut BMesh, v_delimit: usize, f: usize) -> usize {
    /* quick check: 3+ tagged edges? */
    let mut e_delimit_list: Vec<usize> = Vec::new();
    let mut e_face_init = BM_NIL;
    let first = bm.vert(v_delimit).e;
    let mut e_iter = first;
    loop {
        if bm.edge(e_iter).head.hflag & BM_ELEM_INTERNAL_TAG != 0 {
            e_delimit_list.push(e_iter);
            if bm.edge(e_iter).l != BM_NIL && query::bm_edge_in_face(bm, e_iter, f) {
                e_face_init = e_iter;
            }
        }
        e_iter = bmesh_disk_edge_next(bm, e_iter, v_delimit);
        if e_iter == first {
            break;
        }
    }
    if e_delimit_list.len() <= 2 {
        return BM_NIL;
    }
    /* walk the net from v_delimit clearing VERT_NOT_IN_STACK as we go */
    let mut vert_stack: Vec<usize> = vec![v_delimit];
    bm.vpool[v_delimit].head.hflag &= !BM_ELEM_INTERNAL_TAG;
    {
        let mut search: Vec<usize> = Vec::new();
        let seed_e = if e_face_init != BM_NIL {
            e_face_init
        } else {
            bm.vert(v_delimit).e
        };
        let v_other = structure::bm_edge_other_vert(bm, seed_e, v_delimit);
        search.push(v_other);
        if bm.vert(v_other).head.hflag & BM_ELEM_INTERNAL_TAG != 0 {
            bm.vpool[v_other].head.hflag &= !BM_ELEM_INTERNAL_TAG;
            vert_stack.push(v_other);
        }
        while let Some(v_other) = search.pop() {
            let first = bm.vert(v_other).e;
            let mut e_iter = first;
            loop {
                let v_step = structure::bm_edge_other_vert(bm, e_iter, v_other);
                if bm.edge(e_iter).head.hflag & BM_ELEM_INTERNAL_TAG != 0
                    && bm.vert(v_step).head.hflag & BM_ELEM_INTERNAL_TAG != 0
                {
                    bm.vpool[v_step].head.hflag &= !BM_ELEM_INTERNAL_TAG;
                    search.push(v_step);
                    vert_stack.push(v_step);
                }
                e_iter = bmesh_disk_edge_next(bm, e_iter, v_other);
                if e_iter == first {
                    break;
                }
            }
        }
    }
    /* is_delimit: any adjacent vert still tagged on a non-face edge? */
    let mut is_delimit = false;
    let first = bm.vert(v_delimit).e;
    let mut e_iter = first;
    loop {
        let v_step = structure::bm_edge_other_vert(bm, e_iter, v_delimit);
        if bm.vert(v_step).head.hflag & BM_ELEM_INTERNAL_TAG != 0
            && !query::bm_edge_in_face(bm, e_iter, f)
        {
            is_delimit = true;
        } else {
            bm.epool[e_iter].head.hflag &= !BM_ELEM_INTERNAL_TAG;
        }
        e_iter = bmesh_disk_edge_next(bm, e_iter, v_delimit);
        if e_iter == first {
            break;
        }
    }
    let mut v_split = BM_NIL;
    if is_delimit {
        v_split = core::bm_vert_create(bm, bm.vert(v_delimit).co, None, 0);
        core::bm_vert_separate_tested_edges(bm, v_split, v_delimit, |b, e| {
            test_tagged_and_notface(b, e, f)
        });
        bm.vpool[v_split].head.hflag |= BM_ELEM_INTERNAL_TAG;
        if bm.vert(v_split).e == BM_NIL {
            core::bm_vert_kill(bm, v_split);
            v_split = BM_NIL;
        }
    }
    /* restore flags */
    for v in vert_stack {
        bm.vpool[v].head.hflag |= BM_ELEM_INTERNAL_TAG;
    }
    for e in e_delimit_list {
        bm.epool[e].head.hflag |= BM_ELEM_INTERNAL_TAG;
    }
    v_split
}

/// `BM_face_split_edgenet_connect_islands` — detect islands (holes) in the
/// edge-net and connect them with new edges. Returns the new edge list
/// (including the originals) — the caller owns it.
pub fn bm_face_split_edgenet_connect_islands(
    bm: &mut BMesh,
    f: usize,
    edge_net_init: &[usize],
    use_partial_connect: bool,
) -> Vec<usize> {
    let f_len = bm.face(f).len as usize;
    let mut edge_arr: Vec<usize> = edge_net_init.to_vec();
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        edge_arr.push(bm.loo(l).e);
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    let edge_arr_len = edge_arr.len();
    /* tag all edges+verts NOT_IN_STACK */
    for &e in &edge_arr {
        bm.epool[e].head.hflag |= BM_ELEM_INTERNAL_TAG;
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        bm.vpool[v1].head.hflag |= BM_ELEM_INTERNAL_TAG;
        bm.vpool[v2].head.hflag |= BM_ELEM_INTERNAL_TAG;
    }
    /* split delimiting verts for partial connections */
    let mut temp_vert_pairs: Vec<(usize, usize)> = Vec::new(); /* (v_temp, v_orig) */
    let mut use_partial_connect = use_partial_connect;
    if use_partial_connect {
        for &e in edge_net_init {
            for j in 0..2 {
                let v_delimit = if j == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
                loop {
                    let v_other = partial_connect(bm, v_delimit, f);
                    if v_other == BM_NIL {
                        break;
                    }
                    temp_vert_pairs.push((v_other, v_delimit));
                }
            }
        }
        if temp_vert_pairs.is_empty() {
            use_partial_connect = false;
        }
    }
    /* flood-fill groups */
    let mut group_arr: Vec<EdgeGroupIsland> = Vec::new();
    {
        let mut edge_index = edge_arr_len - 1;
        let mut edge_in_group_tot = 0usize;
        let mut vstack: Vec<usize> = Vec::new();
        loop {
            let mut edges_in_group: Vec<usize> = Vec::new();
            let mut unique_verts_in_group = 0usize;
            let v_start = bm.edge(edge_arr[edge_index]).v1;
            vstack.push(v_start);
            bm.vpool[v_start].head.hflag &= !BM_ELEM_INTERNAL_TAG;
            while let Some(v_iter) = vstack.pop() {
                unique_verts_in_group += 1;
                let first = bm.vert(v_iter).e;
                let mut e_iter = first;
                loop {
                    if bm.edge(e_iter).head.hflag & BM_ELEM_INTERNAL_TAG != 0 {
                        bm.epool[e_iter].head.hflag &= !BM_ELEM_INTERNAL_TAG;
                        edges_in_group.push(e_iter);
                        let v_other = structure::bm_edge_other_vert(bm, e_iter, v_iter);
                        if bm.vert(v_other).head.hflag & BM_ELEM_INTERNAL_TAG != 0 {
                            bm.vpool[v_other].head.hflag &= !BM_ELEM_INTERNAL_TAG;
                            vstack.push(v_other);
                        }
                    }
                    e_iter = bmesh_disk_edge_next(bm, e_iter, v_iter);
                    if e_iter == first {
                        break;
                    }
                }
            }
            let edge_cnt = edges_in_group.len();
            edge_in_group_tot += edge_cnt;
            group_arr.push(EdgeGroupIsland {
                edges: edges_in_group,
                vert_len: unique_verts_in_group,
                has_prev_edge: false,
                vert_span_min: BM_NIL,
                vert_span_max: BM_NIL,
                min_axis: [f32::MAX; 2],
                max_axis: [-f32::MAX; 2],
            });
            if edge_in_group_tot == edge_arr_len {
                break;
            }
            while bm.edge(edge_arr[edge_index]).head.hflag & BM_ELEM_INTERNAL_TAG == 0 {
                edge_index -= 1;
            }
        }
    }
    let mut edge_net_new_len;
    let mut edge_net_new: Vec<usize>;
    if group_arr.len() == 1 {
        /* single group - no holes */
        edge_net_new_len = edge_net_init.len();
        edge_net_new = edge_net_init.to_vec();
        for &e in &edge_arr {
            bm.epool[e].head.hflag &= !(BM_ELEM_INTERNAL_TAG | BM_ELEM_SELECT_UV);
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            bm.vpool[v1].head.hflag &= !BM_ELEM_INTERNAL_TAG;
            bm.vpool[v2].head.hflag &= !BM_ELEM_INTERNAL_TAG;
        }
        let _ = edge_net_new_len;
        return edge_net_new;
    }
    /* multiple groups: compute spans, project verts to 2D, connect */
    let axis_mat = geom::axis_dominant_to_m3(bm.face(f).no);
    /* fill group spans (reverse order so boundary group is first) */
    let mut vert_arr_len = 0usize;
    for g in &mut group_arr {
        g.vert_span_min = bm.edge(g.edges[0]).v1;
        g.vert_span_max = bm.edge(g.edges[0]).v2;
        for &e in &g.edges {
            for j in 0..2 {
                let v = if j == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
                let p = mul_v2_m3v3(axis_mat, bm.vert(v).co);
                /* SORT_AXIS=0: axis_value = (x, y) as-is */
                let axis_value = [p.x, p.y];
                if axis_pt_cmp(axis_value, g.min_axis) == -1 {
                    g.vert_span_min = v;
                    g.min_axis = axis_value;
                }
                if axis_pt_cmp(axis_value, g.max_axis) == 1 {
                    g.vert_span_max = v;
                    g.max_axis = axis_value;
                }
            }
        }
        vert_arr_len += g.vert_len;
    }
    /* sort groups by lowest span vertex (boundary group first after reverse) */
    group_arr.reverse();
    group_arr.sort_by(|a, b| {
        axis_pt_cmp(a.min_axis, b.min_axis)
            .cmp(&0)
            .then(axis_pt_cmp(a.max_axis, b.max_axis).cmp(&0))
    });
    /* project verts, assign indices */
    let mut vert_arr: Vec<usize> = Vec::with_capacity(vert_arr_len);
    let mut verts_group_table: Vec<usize> = Vec::with_capacity(vert_arr_len);
    let mut vert_coords_backup: Vec<Vec3> = Vec::with_capacity(vert_arr_len);
    {
        let f_co_ref = bm.vert(bm.loo(l_first).v).co;
        let mut v_index = 0usize;
        for (g_index, g) in group_arr.iter().enumerate() {
            for &e in &g.edges {
                for j in 0..2 {
                    let v = if j == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
                    if bm.vert(v).head.hflag & BM_ELEM_INTERNAL_TAG == 0 {
                        bm.vpool[v].head.hflag |= BM_ELEM_INTERNAL_TAG;
                        vert_coords_backup.push(bm.vert(v).co);
                        let mut co = bm.vert(v).co - f_co_ref;
                        let p = mul_v2_m3v3(axis_mat, co);
                        co = Vec3::new(p.x, p.y, 0.0);
                        bm.vpool[v].co = co;
                        bm.vpool[v].head.index = v_index as i32;
                        vert_arr.push(v);
                        verts_group_table.push(g_index);
                        v_index += 1;
                    }
                }
            }
        }
        vert_arr_len = vert_arr.len();
    }
    bm.elem_index_dirty = true;
    let _ = vert_arr_len;
    /* temp->orig remap for partial connect */
    let mut remap: Vec<i32> = vec![-1; vert_arr.len()];
    if use_partial_connect {
        for &(v_temp, v_orig) in &temp_vert_pairs {
            remap[bm.vert(v_temp).head.index as usize] = bm.vert(v_orig).head.index;
        }
    }
    /* create connections */
    let mut edge_net_new: Vec<usize> = edge_net_init.to_vec();
    {
        let mut vert_range = [0usize, group_arr[0].vert_len];
        for g_index in 1..group_arr.len() {
            vert_range[0] = vert_range[1];
            vert_range[1] += group_arr[g_index].vert_len;
            let g_edges_len = edge_net_new.len() - edge_net_init.len();
            let mut args = FindConnectionArgs {
                edge_arr: &edge_arr,
                edge_arr_new_len: g_edges_len,
                vert_range,
            };
            for dir_sign in [false, true] {
                let v_origin = if !dir_sign {
                    if group_arr[g_index].has_prev_edge {
                        continue;
                    }
                    group_arr[g_index].vert_span_min
                } else {
                    group_arr[g_index].vert_span_max
                };
                let new_len = edge_net_new.len() - edge_net_init.len();
                args.edge_arr_new_len = new_len;
                let index_other = find_connection(
                    bm,
                    &args,
                    &edge_net_new[edge_net_init.len()..],
                    v_origin,
                    dir_sign,
                );
                if index_other != -1 {
                    let overlaps = use_partial_connect
                        && partial_connect_check_overlap(
                            &remap,
                            bm.vert(v_origin).head.index as usize,
                            index_other as usize,
                        );
                    if !overlaps {
                        let v_end = vert_arr[index_other as usize];
                        let totedge_prev = bm.totedge;
                        let e_new = core::bm_edge_create(
                            bm,
                            v_origin,
                            v_end,
                            None,
                            crate::BM_CREATE_NO_DOUBLE,
                        );
                        if e_new != BM_NIL {
                            bm.epool[e_new].head.index = edge_net_new.len() as i32;
                            let is_new = bm.totedge != totedge_prev;
                            if is_new {
                                bm.epool[e_new].head.hflag |= BM_ELEM_SELECT_UV;
                            } else {
                                bm.epool[e_new].head.hflag &= !BM_ELEM_SELECT_UV;
                            }
                            edge_net_new.push(e_new);
                        }
                    }
                }
                if dir_sign && index_other != -1 {
                    let g_other = verts_group_table[index_other as usize];
                    group_arr[g_other].has_prev_edge = true;
                }
            }
        }
    }
    /* restore coords */
    for (i, &v) in vert_arr.iter().enumerate() {
        bm.vpool[v].co = vert_coords_backup[i];
    }
    /* splice partial-connect temp verts back */
    if use_partial_connect {
        for &(v_temp, v_orig) in &temp_vert_pairs {
            if query::bm_edge_exists(bm, v_orig, v_temp) == BM_NIL
                && !core::bm_vert_splice_check_double_face(bm, v_orig, v_temp)
            {
                core::bm_vert_splice(bm, v_orig, v_temp);
            }
        }
        /* remove doubles among the newly created edges */
        edge_net_new_len = edge_net_new.len();
        let mut i = edge_net_init.len();
        while i < edge_net_new_len {
            while query::bm_edge_find_double(bm, edge_net_new[i]) != BM_NIL {
                if bm.edge(edge_net_new[i]).head.hflag & BM_ELEM_SELECT_UV != 0 {
                    let e = edge_net_new[i];
                    core::bm_edge_kill(bm, e);
                }
                edge_net_new_len -= 1;
                if i == edge_net_new_len {
                    break;
                }
                edge_net_new[i] = edge_net_new[edge_net_new_len];
            }
            i += 1;
        }
        edge_net_new.truncate(edge_net_new_len);
    }
    /* cleanup tags */
    for &e in &edge_arr {
        bm.epool[e].head.hflag &= !(BM_ELEM_INTERNAL_TAG | BM_ELEM_SELECT_UV);
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        bm.vpool[v1].head.hflag &= !BM_ELEM_INTERNAL_TAG;
        bm.vpool[v2].head.hflag &= !BM_ELEM_INTERNAL_TAG;
    }
    edge_net_new
}

/* ------------------------------------------------------------------ */
/* tools/bmesh_edgenet.cc — BM_mesh_edgenet fill                        */
/* ------------------------------------------------------------------ */

const VNINFO_FLAG_IS_MIXFACE: i32 = 1;

/// `VertNetInfo` — BFS path-walking state per vert.
#[derive(Clone, Copy, Default)]
struct VertNetInfo {
    /// previous vertex in path.
    prev: usize,
    /// path scanning pass value.
    pass: i32,
    /// face index connected to the edge between this and the previous vert.
    face: i32,
    flag: i32,
}

/// `bm_edge_step_ok` — tagged edge with ≤1 face (wire or boundary).
fn bm_edge_step_ok_fill(bm: &BMesh, e: usize) -> bool {
    let ed = bm.edge(e);
    ed.head.hflag & crate::BM_ELEM_TAG != 0 && { let l = ed.l; l == BM_NIL || bm.loo(l).radial_next == l }
}
/// `bm_edge_face` — face index of `e`'s first loop face, or -1.
fn bm_edge_face_fill(bm: &BMesh, e: usize) -> i32 {
    let l = bm.edge(e).l;
    if l == BM_NIL { -1 } else { bm.face(bm.loo(l).f).head.index }
}

/// `bm_edgenet_edge_get_next` — pop a usable edge from queue, else scan mesh.
fn bm_edgenet_edge_get_next(bm: &BMesh, edge_queue: &mut VecDeque<usize>) -> Option<usize> {
    while let Some(e) = edge_queue.pop_front() {
        if bm_edge_step_ok_fill(bm, e) { return Some(e); }
    }
    for e in 0..bm.totedge as usize {
        if bm_edge_step_ok_fill(bm, e) { return Some(e); }
    }
    None
}

/// `bm_edgenet_path_from_pass` — collect half a loop following `prev` while
/// pass matches; prepends to `v_ls`, returns count added.
fn bm_edgenet_path_from_pass(mut v: usize, v_ls: &mut Vec<usize>, vnet_info: &[VertNetInfo]) -> usize {
    let mut v_ls_tot = 0;
    let mut vn = vnet_info[bm_vert_index(v)];
    let pass = vn.pass;
    loop {
        v_ls.push(v);
        v_ls_tot += 1;
        v = vn.prev;
        vn = vnet_info[bm_vert_index(v)];
        if vn.pass != pass { break; }
    }
    v_ls_tot
}

fn bm_vert_index(_v: usize) -> usize { _v } /* index arenas: index == arena slot */

/// `bm_edgenet_path_check_overlap` — would the union of the two
/// vert-paths ending at (v1, v2) already form a face?
fn bm_edgenet_path_check_overlap(bm: &BMesh, v1: usize, v2: usize, vnet_info: &[VertNetInfo]) -> bool {
    let mut vert_arr: Vec<usize> = Vec::new();
    for &v0 in &[v1, v2] {
        let mut v = v0;
        let mut vn = vnet_info[v];
        let pass = vn.pass;
        loop {
            vert_arr.push(v);
            v = vn.prev;
            vn = vnet_info[v];
            if vn.pass != pass { break; }
        }
    }
    if !vert_arr.is_empty() {
        return crate::query::bm_face_exists_overlap(bm, &vert_arr) != BM_NIL;
    }
    false
}

/// `bm_edgenet_face_from_path` — build face from ordered path verts.
fn bm_edgenet_face_from_path(bm: &mut BMesh, path: &[usize]) -> usize {
    let edge_arr = crate::construct::bm_edges_from_verts(bm, path);
    crate::core::bm_face_create(bm, path, &edge_arr, None, 0)
}

/// `bm_edgenet_path_step` — expand BFS frontier from v_curr.
/// Returns the connecting edge when a face-loop is closed, else None.
fn bm_edgenet_path_step(bm: &BMesh, v_curr: usize, v_ls: &mut Vec<usize>, vnet_info: &mut [VertNetInfo]) -> Option<usize> {
    let mut v_curr = v_curr;
    'begin: loop {
        let mut tot = 0;
        let mut v_ls_tot = 0;
        let vn_curr = vnet_info[v_curr];
        let mut edges = Vec::new();
        { /* EdgesOfVert iteration */
            let mut it = crate::iterators::EdgesOfVert::new(bm, v_curr);
            while let Some(e) = it.next() { edges.push(e); }
        }
        for e in edges {
            let v_next = structure::bm_edge_other_vert(bm, e, v_curr);
            if v_next == vn_curr.prev {
                tot += 1;
                continue;
            }
            if bm_edge_step_ok_fill(bm, e) {
                let vn_next = vnet_info[v_next];
                if vn_curr.pass != vn_next.pass {
                    if vn_curr.pass == -vn_next.pass {
                        if (vn_curr.flag | vn_next.flag) & VNINFO_FLAG_IS_MIXFACE != 0
                            && !bm_edgenet_path_check_overlap(bm, v_curr, v_next, vnet_info)
                        {
                            return Some(e);
                        }
                    } else {
                        let nn = &mut vnet_info[v_next];
                        nn.face = bm_edge_face_fill(bm, e);
                        nn.pass = vn_curr.pass;
                        nn.prev = v_curr;
                        nn.flag &= !VNINFO_FLAG_IS_MIXFACE;
                        if vn_curr.flag & VNINFO_FLAG_IS_MIXFACE != 0
                            || nn.face == -1
                            || nn.face != vn_curr.face
                        {
                            nn.flag |= VNINFO_FLAG_IS_MIXFACE;
                        }
                        v_ls.push(v_next);
                        v_ls_tot += 1;
                    }
                }
            }
            tot += 1;
        }
        /* trick to walk along wire-edge paths (C `goto begin`) */
        if v_ls_tot == 1 && tot == 1 {
            v_curr = v_ls.pop().unwrap();
            continue 'begin;
        }
        return None;
    }
}

/// `bm_edgenet_path_calc` — BFS both ways from edge `e` until halves meet.
/// Returns ordered vert loop or None. `path_cost` = BFS depth.
fn bm_edgenet_path_calc(
    bm: &mut BMesh,
    e: usize,
    pass_nr: i32,
    path_cost_max: u32,
    vnet_info: &mut [VertNetInfo],
) -> Option<(Vec<usize>, u32)> {
    debug_assert!(bm_edge_step_ok_fill(bm, e));
    let (v1, v2) = { let ed = bm.edge(e); (ed.v1, ed.v2) };
    let f_index = bm_edge_face_fill(bm, e);
    let mut path_cost_accum: u32 = 0;
    vnet_info[v1] = VertNetInfo {
        pass: pass_nr, prev: v2, face: f_index,
        flag: if f_index == -1 { VNINFO_FLAG_IS_MIXFACE } else { 0 },
    };
    vnet_info[v2] = VertNetInfo {
        pass: -pass_nr, prev: v1, face: f_index,
        flag: if f_index == -1 { VNINFO_FLAG_IS_MIXFACE } else { 0 },
    };
    let mut v_ls_prev: Vec<usize> = vec![v1, v2];
    let mut v_ls_next: Vec<usize> = Vec::new();
    loop {
        let mut found = false;
        if path_cost_accum >= path_cost_max { return None; }
        while let Some(v) = v_ls_prev.pop() {
            let n_before = v_ls_next.len();
            if let Some(e_found) = bm_edgenet_path_step(bm, v, &mut v_ls_next, vnet_info) {
                let mut path: Vec<usize> = Vec::new();
                let (ev1, ev2) = { let ed = bm.edge(e_found); (ed.v1, ed.v2) };
                let mut half1 = Vec::new();
                bm_edgenet_path_from_pass(ev1, &mut half1, vnet_info);
                half1.reverse();
                path.extend(half1);
                bm_edgenet_path_from_pass(ev2, &mut path, vnet_info);
                let path_len = path.len();
                return Some((path[..path_len].to_vec(), path_cost_accum));
            }
            if v_ls_next.len() != n_before { found = true; }
        }
        path_cost_accum += 1;
        std::mem::swap(&mut v_ls_prev, &mut v_ls_next);
        if !found { break; }
    }
    /* tag not to search again */
    bm.epool[e].head.hflag &= !crate::BM_ELEM_TAG;
    None
}

/// `bm_edgenet_path_calc_best` — like path_calc but re-searches from every
/// path edge for a cheaper alternative.
fn bm_edgenet_path_calc_best(
    bm: &mut BMesh,
    e: usize,
    pass_nr: &mut i32,
    path_cost_max: u32,
    vnet_info: &mut [VertNetInfo],
) -> Option<(Vec<usize>, u32)> {
    let (mut path, mut path_cost) =
        match bm_edgenet_path_calc(bm, e, *pass_nr, path_cost_max, vnet_info) {
            Some(r) => r,
            None => return None,
        };
    *pass_nr += 1;
    if path_cost < 1 { return Some((path, path_cost)); }
    /* Check every edge to see if any can give a better path. */
    let path_len = path.len();
    let mut i_prev = path_len - 1;
    for i in 0..path_len {
        let e_other = crate::query::bm_edge_exists(bm, path[i], path[i_prev]);
        if e_other != BM_NIL && e_other != e {
                if let Some((p2, c2)) = bm_edgenet_path_calc(&mut *bm, e_other, *pass_nr, path_cost, vnet_info) {
                    debug_assert!(c2 < path_cost);
                    path = p2;
                    path_cost = c2;
                }
                *pass_nr += 1;
        }
        i_prev = i;
    }
    Some((path, path_cost))
}

/// `BM_mesh_edgenet` — fill in tagged edge-nets with faces (edge-net fill).
/// `use_edge_tag=false` tags all boundary/wire edges first;
/// `use_new_face_tag` tags each new face.
pub fn bm_mesh_edgenet(bm: &mut BMesh, use_edge_tag: bool, use_new_face_tag: bool) {
    let mut vnet_info = vec![VertNetInfo::default(); bm.totvert as usize];
    let mut edge_queue: VecDeque<usize> = VecDeque::new();
    let mut pass_nr: i32 = 1;
    if !use_edge_tag {
        for e in 0..bm.totedge as usize {
            let ok = bm_edge_step_ok_fill(bm, e);
            bm.epool[e].head.hflag &= !crate::BM_ELEM_TAG;
            if ok { bm.epool[e].head.hflag |= crate::BM_ELEM_TAG; }
        }
    }
    crate::mesh::bm_mesh_elem_index_ensure(bm, crate::BM_VERT | crate::BM_FACE);
    loop {
        let e = match bm_edgenet_edge_get_next(bm, &mut edge_queue) {
            Some(e) => e,
            None => break,
        };
        debug_assert!(bm_edge_step_ok_fill(bm, e));
        if let Some((path, _cost)) = bm_edgenet_path_calc_best(&mut *bm, e, &mut pass_nr, u32::MAX, &mut vnet_info) {
            let f = bm_edgenet_face_from_path(bm, &path);
            /* queue edges to operate on */
            let l_first = bm.face(f).l_first;
            let mut l_iter = l_first;
            loop {
                let le = bm.loo(l_iter).e;
                if bm_edge_step_ok_fill(bm, le) { edge_queue.push_back(le); }
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first { break; }
            }
            if use_new_face_tag {
                bm.fpool[f].head.hflag |= crate::BM_ELEM_TAG;
            }
            /* the face index only needs to be unique, not kept valid */
            bm.fpool[f].head.index = bm.totface - 1;
        }
    }
    bm.elem_index_dirty = true;
    bm.elem_table_dirty = true;
}
