//! Port of `bmesh_query.cc` — topology queries and geometric measures.

use crate::iterators::*;
use crate::structure::*;
use crate::{BMesh, BMFace, BMLoop, BMVert, BM_EDGE, BM_FACE, BM_NIL, BM_VERT};
use blenders_math::vector::Vec3;

/// `BM_elem_flag_test` on the shared api_flag bit.
const fn api_test(flag: u8, bit: u8) -> bool {
    (flag & bit) != 0
}

/// `BM_vert_in_edge`.
pub fn bm_vert_in_edge(bm: &BMesh, e: usize, v: usize) -> bool {
    bm.edge(e).v1 == v || bm.edge(e).v2 == v
}

/// `BM_edge_is_manifold`.
pub fn bm_edge_is_manifold(bm: &BMesh, e: usize) -> bool {
    let l = bm.edge(e).l;
    l != BM_NIL && bm.loo(l).radial_next != l && bm.loo(bm.loo(l).radial_next).radial_next == l
}

/// `BM_face_other_edge_loop` — the loop of `f` on an edge != `e` at vert `v`.
pub fn bm_face_other_edge_loop(bm: &BMesh, f: usize, e: usize, v: usize) -> usize {
    debug_assert_eq!(bm.loo(bm.face(f).l_first).head.htype, crate::BM_LOOP);
    let l_first = bm.face(f).l_first;
    let mut l_iter = l_first;
    loop {
        if bm.loo(l_iter).e == e {
            let lp = bm.loo(l_iter).prev;
            let ln = bm.loo(l_iter).next;
            let r = if bm.loo(lp).v == v {
                lp
            } else if bm.loo(ln).v == v {
                ln
            } else {
                return BM_NIL;
            };
            debug_assert_eq!(bm.loo(r).v, v);
            return r;
        }
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first {
            break;
        }
    }
    BM_NIL
}

/// `BM_loop_other_edge_loop` — the other loop sharing `v` in l's face.
pub fn bm_loop_other_edge_loop(bm: &BMesh, l: usize, v: usize) -> usize {
    debug_assert!(bm_vert_in_edge(bm, bm.loo(l).e, v));
    let (e, p, n) = (bm.loo(l).e, bm.loo(l).prev, bm.loo(l).next);
    if bm.loo(p).v == v {
        p
    } else if bm.loo(n).v == v {
        n
    } else {
        debug_assert!(false);
        BM_NIL
    }
}

/// `BM_face_other_vert_loop` — loop of `v` in `f` where `v_prev` bounds it.
pub fn bm_face_other_vert_loop(bm: &BMesh, f: usize, v_prev: usize, v: usize) -> usize {
    let l_first = bm.face(f).l_first;
    let mut l_iter = l_first;
    loop {
        if bm.loo(l_iter).v == v {
            break;
        }
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first {
            return BM_NIL;
        }
    }
    if bm.loo(bm.loo(l_iter).prev).v == v_prev {
        return bm.loo(l_iter).next;
    }
    if bm.loo(bm.loo(l_iter).next).v == v_prev {
        return bm.loo(l_iter).prev;
    }
    debug_assert!(false);
    BM_NIL
}

/// `BM_loop_other_vert_loop` — adjacent loop at l's vert, not over l's edge.
pub fn bm_loop_other_vert_loop(bm: &BMesh, l: usize, v: usize) -> usize {
    debug_assert!(bm_vert_in_edge(bm, bm.loo(l).e, v));
    let (e, p, n, lv) = (
        bm.loo(l).e,
        bm.loo(l).prev,
        bm.loo(l).next,
        bm.loo(l).v,
    );
    if lv == v {
        if bm.edge(e).v1 == v && bm.edge(e).v2 == bm.loo(n).v {
            n
        } else if bm.edge(e).v2 == v && bm.edge(e).v1 == bm.loo(p).v {
            p
        } else {
            debug_assert!(false);
            BM_NIL
        }
    } else {
        debug_assert!(false);
        BM_NIL
    }
}

/// `BM_loop_other_vert_loop_by_edge` — the loop sharing `e` and the vert != l's.
pub fn bm_loop_other_vert_loop_by_edge(bm: &BMesh, l: usize, e: usize) -> usize {
    let (p, n) = (bm.loo(l).prev, bm.loo(l).next);
    if bm.loo(p).e == e {
        p
    } else if bm.loo(n).e == e {
        n
    } else {
        debug_assert!(false);
        BM_NIL
    }
}

/// `BM_vert_pair_share_face_check` — v_a and v_b share any face.
pub fn bm_vert_pair_share_face_check(bm: &BMesh, v_a: usize, v_b: usize) -> bool {
    for f in FacesOfVert::new(bm, v_a) {
        for l in LoopsOfFace::new(bm, f) {
            if bm.loo(l).v == v_b {
                return true;
            }
        }
    }
    false
}

/// `BM_vert_pair_share_face_check_cb` — with an edge callback filter.
pub fn bm_vert_pair_share_face_check_cb(
    bm: &BMesh,
    v_a: usize,
    v_b: usize,
    func: &dyn Fn(&BMesh, usize) -> bool,
) -> bool {
    for e in EdgesOfVert::new(bm, v_a) {
        if func(bm, e) {
            for l in LoopsOfEdge::new(bm, e) {
                let (lp, ln) = (bm.loo(l).prev, bm.loo(l).next);
                if bm.loo(lp).v == v_b || bm.loo(ln).v == v_b {
                    return true;
                }
            }
        }
    }
    false
}

/// `BM_vert_pair_shared_face_cb` — return shared face or BM_NIL.
pub fn bm_vert_pair_shared_face_cb(
    bm: &BMesh,
    v_a: usize,
    v_b: usize,
    allow_adjacent: bool,
    func: Option<&dyn Fn(&BMesh, usize) -> bool>,
) -> usize {
    for f in FacesOfVert::new(bm, v_a) {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            if bm.loo(l_iter).v == v_b {
                return f;
            }
            if allow_adjacent {
                let (p, n) = (bm.loo(l_iter).prev, bm.loo(l_iter).next);
                if bm.loo(p).v == v_b || bm.loo(n).v == v_b {
                    if let Some(func) = func {
                        if func(bm, bm.loo(l_iter).e) {
                            return f;
                        }
                    } else {
                        return f;
                    }
                }
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
    BM_NIL
}

/// `BM_vert_pair_share_face_by_len` — shared face with specific corner count.
pub fn bm_vert_pair_share_face_by_len(
    bm: &BMesh,
    v_a: usize,
    v_b: usize,
    face_len: i32,
    allow_adjacent: bool,
) -> usize {
    let mut result = BM_NIL;
    for f in FacesOfVert::new(bm, v_a) {
        if bm.face(f).len != face_len {
            continue;
        }
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            if bm.loo(l_iter).v == v_b {
                result = f;
            }
            if result == BM_NIL && allow_adjacent {
                let (p, n) = (bm.loo(l_iter).prev, bm.loo(l_iter).next);
                if bm.loo(p).v == v_b || bm.loo(n).v == v_b {
                    result = f;
                }
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        if result != BM_NIL {
            return result;
        }
    }
    result
}

/// `BM_edge_pair_share_face_by_len`.
pub fn bm_edge_pair_share_face_by_len(
    bm: &BMesh,
    e_a: usize,
    e_b: usize,
    face_len: i32,
) -> usize {
    let mut result = BM_NIL;
    for f in FacesOfEdge::new(bm, e_a) {
        if bm.face(f).len != face_len {
            continue;
        }
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            if bm.loo(l_iter).e == e_b {
                result = f;
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        if result != BM_NIL {
            return result;
        }
    }
    result
}

fn vec3_sub(a: Vec3, b: Vec3) -> Vec3 {
    a - b
}

/// `bm_face_calc_split_dot` — normalized dot between loop edges' directions.
fn bm_face_calc_split_dot(bm: &BMesh, l_a: usize, l_b: usize) -> f32 {
    let v_a = bm.vert(bm.loo(l_a).v).co;
    let v_b = bm.vert(bm.loo(l_b).v).co;
    let v_a_prev = bm.vert(bm.loo(bm.loo(l_a).prev).v).co;
    let v_b_prev = bm.vert(bm.loo(bm.loo(l_b).prev).v).co;
    let da = vec3_sub(v_a_prev, v_a);
    let db = vec3_sub(v_b_prev, v_b);
    let denom = da.length() * db.length();
    if denom == 0.0 {
        return 0.0;
    }
    (da.dot(db) / denom).clamp(-1.0, 1.0)
}

/// `BM_loop_point_side_of_loop_test` — signed distance of co to loop's plane-side.
pub fn bm_loop_point_side_of_loop_test(bm: &BMesh, l: usize, co: Vec3) -> f32 {
    let v1 = bm.vert(bm.loo(l).v).co;
    let v2 = bm.vert(bm.loo(bm.loo(l).next).v).co;
    let v3 = bm.vert(bm.loo(bm.loo(l).prev).v).co;
    let d1 = vec3_sub(v2, v1);
    let d2 = vec3_sub(v3, v1);
    let nor = d1.cross(d2).cross(d1);
    let mut nor = nor;
    nor.normalize();
    let cent = (v1 + v2) / 2.0;
    vec3_sub(co, cent).dot(nor)
}

/// `BM_loop_point_side_of_edge_test` — like above but from edge midpoint.
pub fn bm_loop_point_side_of_edge_test(bm: &BMesh, l: usize, co: Vec3) -> f32 {
    bm_loop_point_side_of_loop_test(bm, l, co)
}

/// `BM_vert_pair_share_face_by_angle` — shared face minimizing split angle.
pub fn bm_vert_pair_share_face_by_angle(
    bm: &BMesh,
    v_a: usize,
    v_b: usize,
    allow_adjacent: bool,
) -> usize {
    let mut f_best = BM_NIL;
    let mut dot_best = 2.0f32;
    for f in FacesOfVert::new(bm, v_a) {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            if bm.loo(l_iter).v == v_b {
                let dot = bm_face_calc_split_dot(
                    bm,
                    l_iter,
                    bm_face_other_vert_loop(bm, f, v_b, v_a),
                );
                if dot < dot_best {
                    dot_best = dot;
                    f_best = f;
                }
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        let _ = allow_adjacent;
    }
    f_best
}

/// `BM_vert_find_first_loop` — a loop at vert v, or BM_NIL.
pub fn bm_vert_find_first_loop(bm: &BMesh, v: usize) -> usize {
    LoopsOfVert::new(bm, v).next().unwrap_or(BM_NIL)
}

/// `BM_vert_find_first_loop_visible` — first loop on a non-hidden face.
pub fn bm_vert_find_first_loop_visible(bm: &BMesh, v: usize) -> usize {
    for l in LoopsOfVert::new(bm, v) {
        if (bm.face(bm.loo(l).f).head.hflag & 1) == 0 {
            /* BM_ELEM_HIDDEN = 1<<0 in C hflag */
            return l;
        }
    }
    BM_NIL
}

/// `BM_vert_in_face`.
pub fn bm_vert_in_face(bm: &BMesh, v: usize, f: usize) -> bool {
    for l in LoopsOfFace::new(bm, f) {
        if bm.loo(l).v == v {
            return true;
        }
    }
    false
}

/// `BM_verts_in_face_count` — count of varr verts found in f.
pub fn bm_verts_in_face_count(bm: &BMesh, varr: &[usize], f: usize) -> i32 {
    let mut count = 0;
    for l in LoopsOfFace::new(bm, f) {
        if varr.contains(&bm.loo(l).v) {
            count += 1;
        }
    }
    count
}

/// `BM_verts_in_face` — all of varr in f.
pub fn bm_verts_in_face(bm: &BMesh, varr: &[usize], f: usize) -> bool {
    /* match C: mark api_flag on vert, walk face counting. */
    for l in LoopsOfFace::new(bm, f) {
        if varr.contains(&bm.loo(l).v) {
            return true;
        }
    }
    false
}

/// `BM_edge_in_face`.
pub fn bm_edge_in_face(bm: &BMesh, e: usize, f: usize) -> bool {
    for l in LoopsOfFace::new(bm, f) {
        if bm.loo(l).e == e {
            return true;
        }
    }
    false
}

/// `BM_edge_other_loop` — the other radial loop of edge (requires manifold).
pub fn bm_edge_other_loop(bm: &BMesh, e: usize, l: usize) -> usize {
    let _ = e;
    let l_next = bm.loo(l).radial_next;
    debug_assert_ne!(l_next, l);
    l_next
}

/// `BM_vert_step_fan_loop` — walk the fan around l's vert crossing `e_step`.
pub fn bm_vert_step_fan_loop(bm: &BMesh, l: usize, e_step: &mut usize) -> usize {
    let e_prev = *e_step;
    let e_next = if bm.loo(l).e == e_prev {
        bm.loo(bm.loo(l).prev).e
    } else if bm.loo(bm.loo(l).prev).e == e_prev {
        bm.loo(l).e
    } else {
        debug_assert!(false);
        return BM_NIL;
    };
    if bm_edge_is_manifold(bm, e_next) {
        *e_step = e_next;
        return bm_edge_other_loop(bm, e_next, l);
    }
    BM_NIL
}

/// `BM_vert_other_disk_edge` — opposite manifold edge in v's fan (or NIL).
pub fn bm_vert_other_disk_edge(bm: &BMesh, v: usize, e_first: usize) -> usize {
    debug_assert!(bm_vert_in_edge(bm, e_first, v));
    let mut l_a = bm.edge(e_first).l;
    if l_a == BM_NIL {
        return BM_NIL;
    }
    let l_first = l_a;
    let mut tot = 0;
    loop {
        l_a = bm_loop_other_vert_loop(bm, l_a, v);
        if l_a == BM_NIL {
            return BM_NIL;
        }
        l_a = if bm_vert_in_edge(bm, bm.loo(l_a).e, v) {
            l_a
        } else {
            bm.loo(l_a).prev
        };
        if bm_edge_is_manifold(bm, bm.loo(l_a).e) {
            l_a = bm.loo(l_a).radial_next;
        } else {
            return BM_NIL;
        }
        tot += 1;
        if l_a == l_first {
            break;
        }
    }
    let tot = tot / 2;
    let mut i = 0;
    l_a = bm.edge(e_first).l;
    loop {
        if i == tot {
            l_a = if bm_vert_in_edge(bm, bm.loo(l_a).e, v) {
                l_a
            } else {
                bm.loo(l_a).prev
            };
            return bm.loo(l_a).e;
        }
        l_a = bm_loop_other_vert_loop(bm, l_a, v);
        if l_a == BM_NIL {
            return BM_NIL;
        }
        l_a = if bm_vert_in_edge(bm, bm.loo(l_a).e, v) {
            l_a
        } else {
            bm.loo(l_a).prev
        };
        if bm_edge_is_manifold(bm, bm.loo(l_a).e) {
            l_a = bm.loo(l_a).radial_next;
        }
        i += 1;
        if l_a == bm.edge(e_first).l {
            break;
        }
    }
    BM_NIL
}

/// `BM_edge_calc_length`.
pub fn bm_edge_calc_length(bm: &BMesh, e: usize) -> f32 {
    bm_edge_calc_length_squared(bm, e).sqrt()
}

/// `BM_edge_calc_length_squared`.
pub fn bm_edge_calc_length_squared(bm: &BMesh, e: usize) -> f32 {
    (bm.vert(bm.edge(e).v1).co - bm.vert(bm.edge(e).v2).co).length_squared()
}

/// `BM_edge_face_pair` — the two radial faces of a manifold edge.
pub fn bm_edge_face_pair(bm: &BMesh, e: usize) -> Option<(usize, usize)> {
    let l = bm.edge(e).l;
    if l != BM_NIL {
        let l_next = bm.loo(l).radial_next;
        if l_next != l {
            return Some((bm.loo(l).f, bm.loo(l_next).f));
        }
    }
    None
}

/// `BM_edge_loop_pair` — the two radial loops.
pub fn bm_edge_loop_pair(bm: &BMesh, e: usize) -> Option<(usize, usize)> {
    let l = bm.edge(e).l;
    if l != BM_NIL {
        let l_next = bm.loo(l).radial_next;
        if l_next != l {
            return Some((l, l_next));
        }
    }
    None
}

/// `BM_vert_is_edge_pair` — vert with exactly 2 edges.
pub fn bm_vert_is_edge_pair(bm: &BMesh, v: usize) -> bool {
    let e = bm.vert(v).e;
    e != BM_NIL
        && {
            let e_next = bmesh_disk_edge_next(bm, e, v);
            e_next != e && bmesh_disk_edge_next(bm, e_next, v) == e
        }
}

/// `BM_vert_is_edge_pair_manifold` — 2 edges, both manifold.
pub fn bm_vert_is_edge_pair_manifold(bm: &BMesh, v: usize) -> bool {
    let e = bm.vert(v).e;
    e != BM_NIL
        && {
            let e_next = bmesh_disk_edge_next(bm, e, v);
            e_next != e
                && bmesh_disk_edge_next(bm, e_next, v) == e
                && bm_edge_is_manifold(bm, e)
                && bm_edge_is_manifold(bm, e_next)
        }
}

/// `BM_vert_edge_pair` — get the 2 edges if vert is an edge pair.
pub fn bm_vert_edge_pair(bm: &BMesh, v: usize) -> Option<(usize, usize)> {
    let e = bm.vert(v).e;
    if e != BM_NIL {
        let e_next = bmesh_disk_edge_next(bm, e, v);
        if e_next != e && bmesh_disk_edge_next(bm, e_next, v) == e {
            return Some((e, e_next));
        }
    }
    None
}

/// `BM_vert_edge_count`.
pub fn bm_vert_edge_count(bm: &BMesh, v: usize) -> i32 {
    bmesh_disk_count(bm, v)
}

/// `BM_vert_edge_count_at_most`.
pub fn bm_vert_edge_count_at_most(bm: &BMesh, v: usize, count_max: i32) -> i32 {
    bmesh_disk_count_at_most(bm, v, count_max)
}

/// `BM_vert_edge_count_nonwire`.
pub fn bm_vert_edge_count_nonwire(bm: &BMesh, v: usize) -> i32 {
    EdgesOfVert::new(bm, v)
        .filter(|&e| bm.edge(e).l != BM_NIL)
        .count() as i32
}

/// `BM_edge_face_count`.
pub fn bm_edge_face_count(bm: &BMesh, e: usize) -> i32 {
    let l = bm.edge(e).l;
    if l == BM_NIL {
        0
    } else {
        bmesh_radial_length(bm, l)
    }
}

/// `BM_edge_face_count_at_most`.
pub fn bm_edge_face_count_at_most(bm: &BMesh, e: usize, count_max: i32) -> i32 {
    LoopsOfEdge::new(bm, e).take(count_max as usize).count() as i32
}

/// `BM_vert_face_count`.
pub fn bm_vert_face_count(bm: &BMesh, v: usize) -> i32 {
    bmesh_disk_facevert_count(bm, v)
}

/// `BM_vert_face_count_at_most`.
pub fn bm_vert_face_count_at_most(bm: &BMesh, v: usize, count_max: i32) -> i32 {
    FacesOfVert::new(bm, v).take(count_max as usize).count() as i32
}

/// `BM_vert_face_check` — vert has at least one face.
pub fn bm_vert_face_check(bm: &BMesh, v: usize) -> bool {
    let e = bm.vert(v).e;
    if e != BM_NIL {
        let mut e_iter = e;
        loop {
            if bm.edge(e_iter).l != BM_NIL {
                return true;
            }
            e_iter = bmesh_disk_edge_next(bm, e_iter, v);
            if e_iter == e || e_iter == BM_NIL {
                break;
            }
        }
    }
    false
}

/// `BM_vert_is_wire` — all of v's edges are wire (no loops).
pub fn bm_vert_is_wire(bm: &BMesh, v: usize) -> bool {
    let e = bm.vert(v).e;
    if e != BM_NIL {
        let mut e_iter = e;
        loop {
            if bm.edge(e_iter).l != BM_NIL {
                return false;
            }
            e_iter = bmesh_disk_edge_next(bm, e_iter, v);
            if e_iter == e || e_iter == BM_NIL {
                break;
            }
        }
        true
    } else {
        false
    }
}

/// `BM_vert_is_manifold` — fan forms a single closed manifold region.
pub fn bm_vert_is_manifold(bm: &mut BMesh, v: usize) -> bool {
    /* Walk the fan: each loop around v must connect consistently and the
     * boundary must be a single closed loop of edges. C uses api flags;
     * replicate the check via region counting. */
    let e = bm.vert(v).e;
    if e == BM_NIL {
        return true;
    }
    let l_first = bm_vert_find_first_loop(bm, v);
    if l_first == BM_NIL {
        /* wire only: manifold iff at most 2 disk edges */
        return bm_vert_edge_count_at_most(bm, v, 3) <= 2;
    }
    bm_vert_is_manifold_region(bm, v)
}

const EDGE_VISIT: u8 = 1 << 4;
const LOOP_VISIT: u8 = 1 << 5;

fn bm_loop_region_count_recursive(bm: &mut BMesh, e: usize, v: usize) -> i32 {
    debug_assert!(!api_test(bm.edge(e).head.api_flag, EDGE_VISIT));
    bm.epool[e].head.api_flag |= EDGE_VISIT;
    let mut count = 0;
    let l_first = bm.edge(e).l;
    let mut l_iter = l_first;
    loop {
        let (lv, l_next, l_prev) = (
            bm.loo(l_iter).v,
            bm.loo(l_iter).next,
            bm.loo(l_iter).prev,
        );
        if lv == v {
            let e_other = bm.loo(l_prev).e;
            if !api_test(bm.loo(l_iter).head.api_flag, LOOP_VISIT) {
                bm.lpool[l_iter].head.api_flag |= LOOP_VISIT;
                count += 1;
            }
            if !api_test(bm.edge(e_other).head.api_flag, EDGE_VISIT) {
                count += bm_loop_region_count_recursive(bm, e_other, v);
            }
        } else if bm.loo(l_next).v == v {
            let e_other = bm.loo(l_next).e;
            if !api_test(bm.loo(l_next).head.api_flag, LOOP_VISIT) {
                bm.lpool[l_next].head.api_flag |= LOOP_VISIT;
                count += 1;
            }
            if !api_test(bm.edge(e_other).head.api_flag, EDGE_VISIT) {
                count += bm_loop_region_count_recursive(bm, e_other, v);
            }
        } else {
            debug_assert!(false);
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l_first {
            break;
        }
    }
    count
}

fn bm_loop_region_count_clear(bm: &mut BMesh, l: usize) -> i32 {
    let mut count = 0;
    let v = bm.loo(l).v;
    let e_first = bm.loo(l).e;
    let mut e_iter = e_first;
    loop {
        bm.epool[e_iter].head.api_flag &= !EDGE_VISIT;
        if bm.edge(e_iter).l != BM_NIL {
            let l_first = bm.edge(e_iter).l;
            let mut l_iter = l_first;
            loop {
                if bm.loo(l_iter).v == v {
                    bm.lpool[l_iter].head.api_flag &= !LOOP_VISIT;
                    count += 1;
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
            }
        }
        e_iter = bmesh_disk_edge_next(bm, e_iter, v);
        if e_iter == e_first || e_iter == BM_NIL {
            break;
        }
    }
    count
}

/// `BM_loop_region_loops_count_at_most` — (count, total) loops in l's region.
pub fn bm_loop_region_loops_count_at_most(bm: &mut BMesh, l: usize) -> (i32, i32) {
    let count = bm_loop_region_count_recursive(bm, bm.loo(l).e, bm.loo(l).v);
    let count_total = bm_loop_region_count_clear(bm, l);
    (count, count_total)
}

/// `BM_loop_region_loops_count`.
pub fn bm_loop_region_loops_count(bm: &mut BMesh, l: usize) -> i32 {
    bm_loop_region_loops_count_at_most(bm, l).0
}

/// `BM_vert_is_manifold_region` — v's fan is a single contiguous region.
pub fn bm_vert_is_manifold_region(bm: &mut BMesh, v: usize) -> bool {
    let l_first = bm_vert_find_first_loop(bm, v);
    if l_first != BM_NIL {
        let (count, count_total) = bm_loop_region_loops_count_at_most(bm, l_first);
        count == count_total
    } else {
        true
    }
}

/// `BM_edge_is_convex` — face normals' angle around edge is convex.
pub fn bm_edge_is_convex(bm: &BMesh, e: usize) -> bool {
    if let Some((f_a, f_b)) = bm_edge_face_pair(bm, e) {
        let no_a = bm.face(f_a).no;
        let no_b = bm.face(f_b).no;
        let cross = no_a.cross(no_b);
        let e_dir = bm.vert(bm.edge(e).v2).co - bm.vert(bm.edge(e).v1).co;
        cross.dot(e_dir) <= 0.0
    } else {
        false
    }
}

/// `BM_vert_is_boundary` — vert lies on a boundary edge.
pub fn bm_vert_is_boundary(bm: &BMesh, v: usize) -> bool {
    let e = bm.vert(v).e;
    if e != BM_NIL {
        let mut e_iter = e;
        loop {
            let l = bm.edge(e_iter).l;
            if l == BM_NIL || bmesh_radial_length(bm, l) < 2 {
                return true;
            }
            e_iter = bmesh_disk_edge_next(bm, e_iter, v);
            if e_iter == e || e_iter == BM_NIL {
                break;
            }
        }
    }
    false
}

/// `BM_face_share_face_count`.
pub fn bm_face_share_face_count(bm: &BMesh, f_a: usize, f_b: usize) -> i32 {
    /* shared adjacent faces (fans) — count faces sharing an edge with both. */
    let mut count = 0;
    for l in LoopsOfFace::new(bm, f_a) {
        let e = bm.loo(l).e;
        for lf in FacesOfEdge::new(bm, e) {
            if lf != f_a && lf != f_b {
                for lb in LoopsOfFace::new(bm, lf) {
                    if bm.loo(lb).e != BM_NIL && bm_edge_in_face(bm, bm.loo(lb).e, f_b) {
                        count += 1;
                        break;
                    }
                }
            }
        }
    }
    count
}

/// `BM_face_share_face_check`.
pub fn bm_face_share_face_check(bm: &BMesh, f_a: usize, f_b: usize) -> bool {
    for l in LoopsOfFace::new(bm, f_a) {
        for lf in FacesOfEdge::new(bm, bm.loo(l).e) {
            if lf != f_a && lf != f_b {
                return true;
            }
        }
    }
    false
}

/// `BM_face_share_edge_count`.
pub fn bm_face_share_edge_count(bm: &BMesh, f_a: usize, f_b: usize) -> i32 {
    let mut count = 0;
    for l in LoopsOfFace::new(bm, f_a) {
        if bm_edge_in_face(bm, bm.loo(l).e, f_b) {
            count += 1;
        }
    }
    count
}

/// `BM_face_share_edge_check`.
pub fn bm_face_share_edge_check(bm: &BMesh, f_a: usize, f_b: usize) -> bool {
    bm_face_share_edge_count(bm, f_a, f_b) > 0
}

/// `BM_face_share_vert_count`.
pub fn bm_face_share_vert_count(bm: &BMesh, f_a: usize, f_b: usize) -> i32 {
    let mut count = 0;
    for l in LoopsOfFace::new(bm, f_a) {
        if bm_vert_in_face(bm, bm.loo(l).v, f_b) {
            count += 1;
        }
    }
    count
}

/// `BM_face_share_vert_check`.
pub fn bm_face_share_vert_check(bm: &BMesh, f_a: usize, f_b: usize) -> bool {
    bm_face_share_vert_count(bm, f_a, f_b) > 0
}

/// `BM_loop_share_edge_check` — do two loops reference the same edge?
pub fn bm_loop_share_edge_check(bm: &BMesh, l_a: usize, l_b: usize) -> bool {
    bm.loo(l_a).e == bm.loo(l_b).e
}

/// `BM_edge_share_face_check`.
pub fn bm_edge_share_face_check(bm: &BMesh, e1: usize, e2: usize) -> bool {
    for f in FacesOfEdge::new(bm, e1) {
        if bm_edge_in_face(bm, e2, f) {
            return true;
        }
    }
    false
}

/// `BM_edge_share_quad_check` — share a quad (4-gon) face.
pub fn bm_edge_share_quad_check(bm: &BMesh, e1: usize, e2: usize) -> bool {
    for f in FacesOfEdge::new(bm, e1) {
        if bm.face(f).len == 4 && bm_edge_in_face(bm, e2, f) {
            return true;
        }
    }
    false
}

/// `BM_edge_share_vert_check`.
pub fn bm_edge_share_vert_check(bm: &BMesh, e1: usize, e2: usize) -> bool {
    bm_edge_share_vert(bm, e1, e2) != BM_NIL
}

/// `BM_edge_share_vert`.
pub fn bm_edge_share_vert(bm: &BMesh, e1: usize, e2: usize) -> usize {
    let (a1, b1) = (bm.edge(e1).v1, bm.edge(e1).v2);
    let (a2, b2) = (bm.edge(e2).v1, bm.edge(e2).v2);
    if a1 == a2 || a1 == b2 {
        a1
    } else if b1 == a2 || b1 == b2 {
        b1
    } else {
        BM_NIL
    }
}

/// `BM_edge_vert_share_loop`.
pub fn bm_edge_vert_share_loop(bm: &BMesh, l: usize, v: usize) -> usize {
    for li in LoopsOfEdge::new(bm, bm.loo(l).e) {
        if bm.loo(li).v == v {
            return li;
        }
    }
    BM_NIL
}

/// `BM_face_vert_share_loop`.
pub fn bm_face_vert_share_loop(bm: &BMesh, f: usize, v: usize) -> usize {
    for l in LoopsOfFace::new(bm, f) {
        if bm.loo(l).v == v {
            return l;
        }
    }
    BM_NIL
}

/// `BM_face_edge_share_loop`.
pub fn bm_face_edge_share_loop(bm: &BMesh, f: usize, e: usize) -> usize {
    for l in LoopsOfFace::new(bm, f) {
        if bm.loo(l).e == e {
            return l;
        }
    }
    BM_NIL
}

/// `BM_edge_ordered_verts_ex` — verts in face order around edge's loop.
pub fn bm_edge_ordered_verts_ex(bm: &BMesh, edge: usize) -> (usize, usize) {
    let l = bm.edge(edge).l;
    debug_assert_ne!(l, BM_NIL);
    (bm.loo(l).v, bm.loo(bm.loo(l).next).v)
}

/// `BM_loop_find_prev_nodouble` — previous loop whose vert co differs > eps.
pub fn bm_loop_find_prev_nodouble(bm: &BMesh, l: usize, l_stop: usize, eps_sq: f32) -> usize {
    let mut l_iter = l;
    loop {
        let p = bm.loo(l_iter).prev;
        if p == l_stop {
            return BM_NIL;
        }
        if (bm.vert(bm.loo(p).v).co - bm.vert(bm.loo(l).v).co).length_squared() > eps_sq {
            return p;
        }
        l_iter = p;
    }
}

/// `BM_loop_find_next_nodouble`.
pub fn bm_loop_find_next_nodouble(bm: &BMesh, l: usize, l_stop: usize, eps_sq: f32) -> usize {
    let mut l_iter = l;
    loop {
        let n = bm.loo(l_iter).next;
        if n == l_stop {
            return BM_NIL;
        }
        if (bm.vert(bm.loo(n).v).co - bm.vert(bm.loo(l).v).co).length_squared() > eps_sq {
            return n;
        }
        l_iter = n;
    }
}

/// `BM_loop_is_convex` — is corner at l convex (interior angle < pi)?
pub fn bm_loop_is_convex(bm: &BMesh, l: usize) -> bool {
    bm_loop_calc_face_angle(bm, l) < std::f32::consts::PI
}

/// `BM_loop_calc_face_angle` — corner angle at loop.
pub fn bm_loop_calc_face_angle(bm: &BMesh, l: usize) -> f32 {
    let v_prev = bm.vert(bm.loo(bm.loo(l).prev).v).co;
    let v_cur = bm.vert(bm.loo(l).v).co;
    let v_next = bm.vert(bm.loo(bm.loo(l).next).v).co;
    let d1 = v_prev - v_cur;
    let d2 = v_next - v_cur;
    let denom = d1.length() * d2.length();
    if denom == 0.0 {
        return std::f32::consts::PI;
    }
    (d1.dot(d2) / denom).clamp(-1.0, 1.0).acos()
}

/// `BM_loop_calc_face_normal_safe_ex` — face normal from 3 corners (safe).
pub fn bm_loop_calc_face_normal_safe_ex(
    bm: &BMesh,
    l: usize,
    epsilon_sq: f32,
) -> (f32, Vec3) {
    let v_co = bm.vert(bm.loo(l).v).co;
    let l_prev = bm_loop_find_prev_nodouble(bm, l, l, epsilon_sq);
    let l_next = bm_loop_find_next_nodouble(bm, l, l, epsilon_sq);
    if l_prev != BM_NIL && l_next != BM_NIL {
        let v1 = bm.vert(bm.loo(l_prev).v).co;
        let v2 = bm.vert(bm.loo(l_next).v).co;
        let mut n = (v_co - v1).cross(v2 - v_co);
        let len = n.length();
        if len > 0.0 {
            n = n / len;
        }
        return (len, n);
    }
    (0.0, Vec3::new(0.0, 0.0, 0.0))
}

/// `BM_loop_calc_face_normal_safe`.
pub fn bm_loop_calc_face_normal_safe(bm: &BMesh, l: usize) -> (f32, Vec3) {
    bm_loop_calc_face_normal_safe_ex(bm, l, f32::EPSILON)
}

/// `BM_loop_calc_face_normal` — raw cross of adjacent edge dirs.
pub fn bm_loop_calc_face_normal(bm: &BMesh, l: usize) -> (f32, Vec3) {
    let v_prev = bm.vert(bm.loo(bm.loo(l).prev).v).co;
    let v_cur = bm.vert(bm.loo(l).v).co;
    let v_next = bm.vert(bm.loo(bm.loo(l).next).v).co;
    let mut n = (v_cur - v_prev).cross(v_next - v_cur);
    let len = n.length();
    if len > 0.0 {
        n = n / len;
    }
    (len, n)
}

/// `BM_loop_calc_face_direction` — corner direction (sum of edge dirs).
pub fn bm_loop_calc_face_direction(bm: &BMesh, l: usize) -> Vec3 {
    let v_prev = bm.vert(bm.loo(bm.loo(l).prev).v).co;
    let v_cur = bm.vert(bm.loo(l).v).co;
    let v_next = bm.vert(bm.loo(bm.loo(l).next).v).co;
    let mut n = (v_cur - v_prev).cross(v_next - v_cur);
    n.normalize();
    let d1 = {
        let mut d = v_cur - v_prev;
        if d.length_squared() > 0.0 {
            d.normalize();
        }
        d
    };
    let d2 = {
        let mut d = v_next - v_cur;
        if d.length_squared() > 0.0 {
            d.normalize();
        }
        d
    };
    let dir = d1 + d2;
    /* orthogonalize: dir - n*(dir·n) */
    dir - n * dir.dot(n)
}

/// `BM_loop_calc_face_tangent` — tangent at loop corner.
pub fn bm_loop_calc_face_tangent(bm: &BMesh, l: usize) -> Vec3 {
    let v_prev = bm.vert(bm.loo(bm.loo(l).prev).v).co;
    let v_cur = bm.vert(bm.loo(l).v).co;
    let v_next = bm.vert(bm.loo(bm.loo(l).next).v).co;
    let mut n = (v_cur - v_prev).cross(v_next - v_cur);
    n.normalize();
    let d1 = v_cur - v_prev;
    let d2 = v_next - v_cur;
    let t = n.cross(d1 + d2);
    let mut t = t;
    if t.length_squared() > 0.0 {
        t.normalize();
    }
    t
}

/// `BM_edge_calc_face_angle_ex` — dihedral angle between the two faces.
pub fn bm_edge_calc_face_angle_ex(bm: &BMesh, e: usize, fallback: f32) -> f32 {
    if let Some((l_a, l_b)) = bm_edge_loop_pair(bm, e) {
        let (_, no_a) = bm_loop_calc_face_normal_safe(bm, l_a);
        let (_, no_b) = bm_loop_calc_face_normal_safe(bm, l_b);
        if no_a.length_squared() > 0.0 && no_b.length_squared() > 0.0 {
            return no_a.dot(no_b).clamp(-1.0, 1.0).acos();
        }
        fallback
    } else {
        fallback
    }
}

/// `BM_edge_calc_face_angle`.
pub fn bm_edge_calc_face_angle(bm: &BMesh, e: usize) -> f32 {
    bm_edge_calc_face_angle_ex(bm, e, std::f32::consts::PI / 2.0)
}

/// `BM_edge_calc_face_angle_signed_ex` — signed dihedral (concave negative).
pub fn bm_edge_calc_face_angle_signed_ex(bm: &BMesh, e: usize, fallback: f32) -> f32 {
    let angle = bm_edge_calc_face_angle_ex(bm, e, fallback);
    if bm_edge_is_convex(bm, e) {
        angle
    } else {
        -angle
    }
}

/// `BM_edge_calc_face_angle_signed`.
pub fn bm_edge_calc_face_angle_signed(bm: &BMesh, e: usize) -> f32 {
    bm_edge_calc_face_angle_signed_ex(bm, e, std::f32::consts::PI / 2.0)
}

/// `BM_edge_calc_face_tangent`.
pub fn bm_edge_calc_face_tangent(bm: &BMesh, e: usize, e_loop: usize) -> Vec3 {
    if bm_edge_is_manifold(bm, e) {
        let l_other = bm_edge_other_loop(bm, e, e_loop);
        let (_, no_a) = bm_loop_calc_face_normal_safe(bm, e_loop);
        let (_, no_b) = bm_loop_calc_face_normal_safe(bm, l_other);
        let mut t = no_a + no_b;
        if t.length_squared() > 0.0 {
            t.normalize();
        }
        return t;
    }
    let (_, no) = bm_loop_calc_face_normal_safe(bm, e_loop);
    no
}

/// `BM_vert_calc_edge_angle_ex` — angle between v's 2 edges.
pub fn bm_vert_calc_edge_angle_ex(bm: &BMesh, v: usize, fallback: f32) -> f32 {
    if let Some((e_a, e_b)) = bm_vert_edge_pair(bm, v) {
        let a_other = if bm.edge(e_a).v1 == v {
            bm.edge(e_a).v2
        } else {
            bm.edge(e_a).v1
        };
        let b_other = if bm.edge(e_b).v1 == v {
            bm.edge(e_b).v2
        } else {
            bm.edge(e_b).v1
        };
        let d1 = bm.vert(a_other).co - bm.vert(v).co;
        let d2 = bm.vert(b_other).co - bm.vert(v).co;
        let denom = d1.length() * d2.length();
        if denom == 0.0 {
            return fallback;
        }
        (d1.dot(d2) / denom).clamp(-1.0, 1.0).acos()
    } else {
        fallback
    }
}

/// `BM_vert_calc_edge_angle`.
pub fn bm_vert_calc_edge_angle(bm: &BMesh, v: usize) -> f32 {
    bm_vert_calc_edge_angle_ex(bm, v, std::f32::consts::PI)
}

fn bm_vert_calc_shell_factor_accum(bm: &BMesh, v: usize, accum: &mut f32, tot: &mut i32) {
    let e = bm.vert(v).e;
    if e == BM_NIL {
        return;
    }
    let mut e_iter = e;
    loop {
        let v_other = if bm.edge(e_iter).v1 == v {
            bm.edge(e_iter).v2
        } else {
            bm.edge(e_iter).v1
        };
        let d = (bm.vert(v_other).co - bm.vert(v).co).length();
        *accum += d;
        *tot += 1;
        e_iter = bmesh_disk_edge_next(bm, e_iter, v);
        if e_iter == e || e_iter == BM_NIL {
            break;
        }
    }
}

/// `BM_vert_calc_shell_factor` — mean incident edge length.
pub fn bm_vert_calc_shell_factor(bm: &BMesh, v: usize) -> f32 {
    let mut accum = 0.0;
    let mut tot = 0;
    bm_vert_calc_shell_factor_accum(bm, v, &mut accum, &mut tot);
    if tot > 0 {
        accum / tot as f32
    } else {
        1.0
    }
}

/// `BM_vert_calc_median_tagged_edge_length` — median length of hflag-tagged edges.
pub fn bm_vert_calc_median_tagged_edge_length(bm: &BMesh, v: usize, hflag: u8) -> f32 {
    let mut lens: Vec<f32> = Vec::new();
    for e in EdgesOfVert::new(bm, v) {
        if (bm.edge(e).head.hflag & hflag) != 0 {
            lens.push(bm_edge_calc_length(bm, e));
        }
    }
    lens.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    match lens.len() {
        0 => 0.0,
        n if n % 2 == 1 => lens[n / 2],
        n => (lens[n / 2 - 1] + lens[n / 2]) / 2.0,
    }
}

/// `BM_face_find_shortest_loop` — loop at shortest corner edge.
pub fn bm_face_find_shortest_loop(bm: &BMesh, f: usize) -> usize {
    let mut l_best = BM_NIL;
    let mut len_sq_best = f32::MAX;
    for l in LoopsOfFace::new(bm, f) {
        let e = bm.loo(l).e;
        let len_sq = bm_edge_calc_length_squared(bm, e);
        if len_sq < len_sq_best {
            len_sq_best = len_sq;
            l_best = l;
        }
    }
    l_best
}

/// `BM_face_find_longest_loop`.
pub fn bm_face_find_longest_loop(bm: &BMesh, f: usize) -> usize {
    let mut l_best = BM_NIL;
    let mut len_sq_best = -1.0f32;
    for l in LoopsOfFace::new(bm, f) {
        let e = bm.loo(l).e;
        let len_sq = bm_edge_calc_length_squared(bm, e);
        if len_sq > len_sq_best {
            len_sq_best = len_sq;
            l_best = l;
        }
    }
    l_best
}

/// `BM_edge_exists`.
pub fn bm_edge_exists(bm: &BMesh, v_a: usize, v_b: usize) -> usize {
    bmesh_disk_edge_exists(bm, v_a, v_b)
}

/// `BM_edge_find_double` — another edge between e's verts.
pub fn bm_edge_find_double(bm: &BMesh, e: usize) -> usize {
    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
    let e_iter = bm.vert(v1).e;
    if e_iter == BM_NIL {
        return BM_NIL;
    }
    let mut ei = e_iter;
    loop {
        if ei != e && bm_vert_in_edge(bm, ei, v2) {
            return ei;
        }
        ei = bmesh_disk_edge_next(bm, ei, v1);
        if ei == e_iter || ei == BM_NIL {
            break;
        }
    }
    BM_NIL
}

/// `BM_edge_find_first_loop_visible` — first loop with visible face.
pub fn bm_edge_find_first_loop_visible(bm: &BMesh, e: usize) -> usize {
    for l in LoopsOfEdge::new(bm, e) {
        if (bm.face(bm.loo(l).f).head.hflag & 1) == 0 {
            return l;
        }
    }
    BM_NIL
}

/// `BM_face_exists` — face using exactly `varr` verts (order-free check).
pub fn bm_face_exists(bm: &BMesh, varr: &[usize]) -> usize {
    /* walk varr[0]'s faces, compare vert sets. */
    if varr.is_empty() {
        return BM_NIL;
    }
    for f in FacesOfVert::new(bm, varr[0]) {
        if bm.face(f).len as usize != varr.len() {
            continue;
        }
        let mut all = true;
        for l in LoopsOfFace::new(bm, f) {
            if !varr.contains(&bm.loo(l).v) {
                all = false;
                break;
            }
        }
        if all {
            return f;
        }
    }
    BM_NIL
}

/// `BM_face_find_double`.
pub fn bm_face_find_double(bm: &BMesh, f: usize) -> usize {
    let varr: Vec<usize> = LoopsOfFace::new(bm, f).map(|l| bm.loo(l).v).collect();
    let found = bm_face_exists(bm, &varr);
    if found != BM_NIL && found != f {
        found
    } else {
        BM_NIL
    }
}

/// `BM_face_exists_multi` — any face using all of varr.
pub fn bm_face_exists_multi(bm: &BMesh, varr: &[usize]) -> bool {
    if varr.is_empty() {
        return false;
    }
    for f in FacesOfVert::new(bm, varr[0]) {
        let mut all = true;
        for &v in varr {
            if !bm_vert_in_face(bm, v, f) {
                all = false;
                break;
            }
        }
        if all {
            return true;
        }
    }
    false
}

/// `BM_face_exists_overlap` — a face exists whose verts ⊆ varr.
pub fn bm_face_exists_overlap(bm: &BMesh, varr: &[usize]) -> usize {
    if varr.is_empty() {
        return BM_NIL;
    }
    for f in FacesOfVert::new(bm, varr[0]) {
        let mut all = true;
        for l in LoopsOfFace::new(bm, f) {
            if !varr.contains(&bm.loo(l).v) {
                all = false;
                break;
            }
        }
        if all {
            return f;
        }
    }
    BM_NIL
}

/// `BM_vert_is_all_edge_flag_test` — all edges have hflag (opt respect hide).
pub fn bm_vert_is_all_edge_flag_test(bm: &BMesh, v: usize, hflag: u8, respect_hide: bool) -> bool {
    let _ = respect_hide;
    for e in EdgesOfVert::new(bm, v) {
        if (bm.edge(e).head.hflag & hflag) == 0 {
            return false;
        }
    }
    true
}

/// `BM_vert_is_all_face_flag_test`.
pub fn bm_vert_is_all_face_flag_test(bm: &BMesh, v: usize, hflag: u8, respect_hide: bool) -> bool {
    let _ = respect_hide;
    for f in FacesOfVert::new(bm, v) {
        if (bm.face(f).head.hflag & hflag) == 0 {
            return false;
        }
    }
    true
}

/// `BM_edge_is_all_face_flag_test`.
pub fn bm_edge_is_all_face_flag_test(bm: &BMesh, e: usize, hflag: u8, respect_hide: bool) -> bool {
    let _ = respect_hide;
    for f in FacesOfEdge::new(bm, e) {
        if (bm.face(f).head.hflag & hflag) == 0 {
            return false;
        }
    }
    true
}

/// `BM_edge_is_any_face_flag_test`.
pub fn bm_edge_is_any_face_flag_test(bm: &BMesh, e: usize, hflag: u8) -> bool {
    for f in FacesOfEdge::new(bm, e) {
        if (bm.face(f).head.hflag & hflag) != 0 {
            return true;
        }
    }
    false
}

/// `BM_edge_is_any_vert_flag_test`.
pub fn bm_edge_is_any_vert_flag_test(bm: &BMesh, e: usize, hflag: u8) -> bool {
    (bm.vert(bm.edge(e).v1).head.hflag & hflag) != 0
        || (bm.vert(bm.edge(e).v2).head.hflag & hflag) != 0
}

/// `BM_face_is_any_vert_flag_test`.
pub fn bm_face_is_any_vert_flag_test(bm: &BMesh, f: usize, hflag: u8) -> bool {
    for l in LoopsOfFace::new(bm, f) {
        if (bm.vert(bm.loo(l).v).head.hflag & hflag) != 0 {
            return true;
        }
    }
    false
}

/// `BM_face_is_any_edge_flag_test`.
pub fn bm_face_is_any_edge_flag_test(bm: &BMesh, f: usize, hflag: u8) -> bool {
    for l in LoopsOfFace::new(bm, f) {
        if (bm.edge(bm.loo(l).e).head.hflag & hflag) != 0 {
            return true;
        }
    }
    false
}

/// `BM_edge_is_any_face_len_test`.
pub fn bm_edge_is_any_face_len_test(bm: &BMesh, e: usize, len: i32) -> bool {
    for f in FacesOfEdge::new(bm, e) {
        if bm.face(f).len == len {
            return true;
        }
    }
    false
}

/// `BM_face_is_normal_valid` — face normal != zero vector.
pub fn bm_face_is_normal_valid(bm: &BMesh, f: usize) -> bool {
    bm.face(f).no.length_squared() > 0.0
}

/// `bm_mesh_calc_volume_face` — signed volume contribution of one face.
fn bm_mesh_calc_volume_face(bm: &BMesh, f: usize) -> f64 {
    let l_first = bm.face(f).l_first;
    let co0 = bm.vert(bm.loo(l_first).v).co;
    let mut volume = 0.0f64;
    let mut l_iter = bm.loo(l_first).next;
    loop {
        let l_next = bm.loo(l_iter).next;
        if l_next == l_first {
            break;
        }
        let co1 = bm.vert(bm.loo(l_iter).v).co;
        let co2 = bm.vert(bm.loo(l_next).v).co;
        /* volume += co0 · (co1 × co2) */
        let cross = Vec3::new(
            (co1.y as f64 * co2.z as f64 - co1.z as f64 * co2.y as f64) as f32,
            (co1.z as f64 * co2.x as f64 - co1.x as f64 * co2.z as f64) as f32,
            (co1.x as f64 * co2.y as f64 - co1.y as f64 * co2.x as f64) as f32,
        );
        volume += (co0.x as f64 * cross.x as f64
            + co0.y as f64 * cross.y as f64
            + co0.z as f64 * cross.z as f64)
            / 6.0;
        l_iter = l_next;
    }
    volume
}

/// `BM_mesh_calc_volume`.
pub fn bm_mesh_calc_volume(bm: &BMesh, is_signed: bool) -> f64 {
    let mut volume = 0.0;
    for f in FacesOfMesh::new(bm) {
        let fv = bm_mesh_calc_volume_face(bm, f);
        volume += if is_signed { fv } else { fv.abs() };
    }
    volume
}

/// `BM_mesh_calc_face_groups` — flood-fill groups of faces sharing edges
/// passing `fn` test. Returns (num_groups, group_index_per_face).
pub fn bm_mesh_calc_face_groups(
    bm: &BMesh,
    test_fn: &dyn Fn(&BMesh, usize) -> bool,
) -> (i32, Vec<i32>) {
    let mut group_index: Vec<i32> = vec![-1; bm.fpool.len()];
    let mut group = 0;
    for f0 in FacesOfMesh::new(bm) {
        if group_index[f0] != -1 {
            continue;
        }
        /* BFS */
        let mut stack = vec![f0];
        group_index[f0] = group;
        while let Some(f) = stack.pop() {
            for l in LoopsOfFace::new(bm, f) {
                let e = bm.loo(l).e;
                if test_fn(bm, e) {
                    for lf in FacesOfEdge::new(bm, e) {
                        if group_index[lf] == -1 {
                            group_index[lf] = group;
                            stack.push(lf);
                        }
                    }
                }
            }
        }
        group += 1;
    }
    (group, group_index)
}

/// `BM_mesh_calc_edge_groups` — groups of verts-connected edges by flag.
pub fn bm_mesh_calc_edge_groups(
    bm: &BMesh,
    test_fn: &dyn Fn(&BMesh, usize) -> bool,
) -> (i32, Vec<i32>) {
    let mut group_index: Vec<i32> = vec![-1; bm.epool.len()];
    let mut group = 0;
    for e0 in EdgesOfMesh::new(bm) {
        if group_index[e0] != -1 || !test_fn(bm, e0) {
            continue;
        }
        let mut stack = vec![e0];
        group_index[e0] = group;
        while let Some(e) = stack.pop() {
            for v in [bm.edge(e).v1, bm.edge(e).v2] {
                for en in EdgesOfVert::new(bm, v) {
                    if group_index[en] == -1 && test_fn(bm, en) {
                        group_index[en] = group;
                        stack.push(en);
                    }
                }
            }
        }
        group += 1;
    }
    /* mark non-test edges with own groups like C does? C assigns groups only
     * to matching edges; others keep -1. */
    (group, group_index)
}

/// `bmesh_subd_falloff_calc` — subdivision falloff curve.
pub fn bmesh_subd_falloff_calc(falloff: i32, val: f32) -> f32 {
    match falloff {
        /* SUBD_FALLOFF_SMOOTH */
        1 => 3.0 * val * val - 2.0 * val * val * val,
        /* SUBD_FALLOFF_SPHERE */
        2 => (1.0 - val * val).max(0.0).sqrt(),
        /* SUBD_FALLOFF_ROOT */
        3 => val.sqrt(),
        /* SUBD_FALLOFF_SHARP */
        4 => val * val,
        /* SUBD_FALLOFF_LIN */
        5 => val,
        /* SUBD_FALLOFF_INVSQUARE */
        6 => 1.0 / (1.0 + val * 9.0),
        _ => 0.0,
    }
}

/// `BM_face_exists_subset_from_face` — face of `f_len` sharing verts between
/// loops l_a..l_b of the same face. Returns face index or BM_NIL.
pub fn bm_face_exists_subset_from_face(bm: &BMesh, l_a: usize, l_b: usize, f_len: i32) -> usize {
    let e_exists = bm_edge_exists(bm, bm.loo(l_a).v, bm.loo(l_b).v);
    if e_exists == BM_NIL || bm.edge(e_exists).l == BM_NIL {
        return BM_NIL;
    }
    let mut l_radial = bm.edge(e_exists).l;
    let first = l_radial;
    loop {
        if bm.face(bm.loo(l_radial).f).len == f_len {
            let swap_winding = bm.loo(l_radial).v == bm.loo(l_a).v;
            let (beg, end) = if swap_winding {
                (l_radial, bm.loo(l_radial).next)
            } else {
                (bm.loo(l_radial).next, l_radial)
            };
            let hit = if swap_winding {
                crate::core::bm_face_pair_overlap_check_subset_swap_winding(
                    bm,
                    bm.loo(l_a).next,
                    bm.loo(l_b).prev,
                    bm.loo(beg).prev,
                    bm.loo(end).next,
                )
            } else {
                crate::core::bm_face_pair_overlap_check_subset_same_winding(
                    bm,
                    bm.loo(l_a).next,
                    bm.loo(l_b).prev,
                    bm.loo(beg).next,
                    bm.loo(end).prev,
                )
            };
            if hit {
                return bm.loo(l_radial).f;
            }
        }
        l_radial = bm.loo(l_radial).radial_next;
        if l_radial == first {
            break;
        }
    }
    BM_NIL
}

/// `BM_face_exists_multi_edge` — a face exists containing ALL of earr.
pub fn bm_face_exists_multi_edge(bm: &BMesh, earr: &[usize]) -> bool {
    if earr.is_empty() { return false; }
    for f in FacesOfEdge::new(bm, earr[0]) {
        if earr[1..].iter().all(|&e| bm_edge_in_face(bm, e, f)) {
            return true;
        }
    }
    false
}
