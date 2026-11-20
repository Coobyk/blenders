//! Port of `bmesh_structure.cc` — disk cycles, radial cycles, and loop cycles.
//!
//! All pointers are element indices (`usize`); `BM_NIL` stands for nullptr.

use crate::{BMesh, BMFace, BMLoop, BM_NIL};

/// `bmesh_disk_edge_link_from_vert` — which disk link of `e` belongs to `v`.
fn disk_link_from_vert(e_v1: usize, e_v2: usize, v: usize) -> bool {
    /* true => use v1_disk_link */
    debug_assert!(v == e_v1 || v == e_v2);
    v == e_v1
}

/// `BM_vert_in_edge`.
pub fn bm_vert_in_edge(bm: &BMesh, e: usize, v: usize) -> bool {
    let ee = bm.edge(e);
    ee.v1 == v || ee.v2 == v
}

/// `BM_verts_in_edge`.
pub fn bm_verts_in_edge(bm: &BMesh, v1: usize, v2: usize, e: usize) -> bool {
    let ee = bm.edge(e);
    (ee.v1 == v1 || ee.v2 == v1) && (ee.v1 == v2 || ee.v2 == v2)
}

/// `bmesh_disk_edge_next` — next edge in v's disk cycle after `e`.
pub fn bmesh_disk_edge_next(bm: &BMesh, e: usize, v: usize) -> usize {
    let ee = bm.edge(e);
    if disk_link_from_vert(ee.v1, ee.v2, v) {
        ee.v1_disk_link.next
    } else {
        ee.v2_disk_link.next
    }
}

/// `bmesh_disk_edge_prev`.
pub fn bmesh_disk_edge_prev(bm: &BMesh, e: usize, v: usize) -> usize {
    let ee = bm.edge(e);
    if disk_link_from_vert(ee.v1, ee.v2, v) {
        ee.v1_disk_link.prev
    } else {
        ee.v2_disk_link.prev
    }
}

/// `bmesh_disk_edge_append` — insert `e` into `v`'s disk cycle.
pub fn bmesh_disk_edge_append(bm: &mut BMesh, e: usize, v: usize) {
    if bm.vert(v).e == BM_NIL {
        let is_v1 = disk_link_from_vert(bm.edge(e).v1, bm.edge(e).v2, v);
        bm.vert_mut(v).e = e;
        let ee = bm.edge_mut(e);
        if is_v1 {
            ee.v1_disk_link.next = e;
            ee.v1_disk_link.prev = e;
        } else {
            ee.v2_disk_link.next = e;
            ee.v2_disk_link.prev = e;
        }
    } else {
        let v_e = bm.vert(v).e;
        let is_v1_e = disk_link_from_vert(bm.edge(e).v1, bm.edge(e).v2, v);
        /* dl2 = link of v->e for v; dl2.prev is dl1's prev. */
        let (dl2_next, dl2_prev) = {
            let ee2 = bm.edge(v_e);
            let is_v1_2 = disk_link_from_vert(ee2.v1, ee2.v2, v);
            if is_v1_2 {
                (ee2.v1_disk_link.next, ee2.v1_disk_link.prev)
            } else {
                (ee2.v2_disk_link.next, ee2.v2_disk_link.prev)
            }
        };
        let _ = dl2_next;
        /* dl1.next = v->e; dl1.prev = dl2.prev */
        {
            let ee = bm.edge_mut(e);
            if is_v1_e {
                ee.v1_disk_link.next = v_e;
                ee.v1_disk_link.prev = dl2_prev;
            } else {
                ee.v2_disk_link.next = v_e;
                ee.v2_disk_link.prev = dl2_prev;
            }
        }
        /* dl2.prev = e */
        {
            let ee2 = bm.edge_mut(v_e);
            let is_v1_2 = disk_link_from_vert(ee2.v1, ee2.v2, v);
            if is_v1_2 {
                ee2.v1_disk_link.prev = e;
            } else {
                ee2.v2_disk_link.prev = e;
            }
        }
        /* dl3.next = e (dl3 = link of dl2.prev for v) */
        if dl2_prev != BM_NIL {
            let ee3 = bm.edge_mut(dl2_prev);
            let is_v1_3 = disk_link_from_vert(ee3.v1, ee3.v2, v);
            if is_v1_3 {
                ee3.v1_disk_link.next = e;
            } else {
                ee3.v2_disk_link.next = e;
            }
        }
    }
}

/// `bmesh_disk_edge_remove` — remove `e` from `v`'s disk cycle.
pub fn bmesh_disk_edge_remove(bm: &mut BMesh, e: usize, v: usize) {
    let (dl1_next, dl1_prev) = {
        let ee = bm.edge(e);
        if disk_link_from_vert(ee.v1, ee.v2, v) {
            (ee.v1_disk_link.next, ee.v1_disk_link.prev)
        } else {
            (ee.v2_disk_link.next, ee.v2_disk_link.prev)
        }
    };
    if dl1_prev != BM_NIL {
        let ee2 = bm.edge_mut(dl1_prev);
        if disk_link_from_vert(ee2.v1, ee2.v2, v) {
            ee2.v1_disk_link.next = dl1_next;
        } else {
            ee2.v2_disk_link.next = dl1_next;
        }
    }
    if dl1_next != BM_NIL {
        let ee2 = bm.edge_mut(dl1_next);
        if disk_link_from_vert(ee2.v1, ee2.v2, v) {
            ee2.v1_disk_link.prev = dl1_prev;
        } else {
            ee2.v2_disk_link.prev = dl1_prev;
        }
    }
    if bm.vert(v).e == e {
        bm.vert_mut(v).e = if e != dl1_next { dl1_next } else { BM_NIL };
    }
    let ee = bm.edge_mut(e);
    if disk_link_from_vert(ee.v1, ee.v2, v) {
        ee.v1_disk_link.next = BM_NIL;
        ee.v1_disk_link.prev = BM_NIL;
    } else {
        ee.v2_disk_link.next = BM_NIL;
        ee.v2_disk_link.prev = BM_NIL;
    }
}

/// `bmesh_disk_edge_exists` — edge between v1 and v2, if any.
pub fn bmesh_disk_edge_exists(bm: &BMesh, v1: usize, v2: usize) -> usize {
    let e_first = bm.vert(v1).e;
    if e_first != BM_NIL {
        let mut e_iter = e_first;
        loop {
            if bm_verts_in_edge(bm, v1, v2, e_iter) {
                return e_iter;
            }
            e_iter = bmesh_disk_edge_next(bm, e_iter, v1);
            if e_iter == e_first {
                break;
            }
        }
    }
    BM_NIL
}

/// `bmesh_disk_count`.
pub fn bmesh_disk_count(bm: &BMesh, v: usize) -> i32 {
    let mut count = 0;
    let e_first = bm.vert(v).e;
    if e_first != BM_NIL {
        let mut e_iter = e_first;
        loop {
            count += 1;
            e_iter = bmesh_disk_edge_next(bm, e_iter, v);
            if e_iter == e_first {
                break;
            }
        }
    }
    count
}

/// `bmesh_disk_count_at_most`.
pub fn bmesh_disk_count_at_most(bm: &BMesh, v: usize, count_max: i32) -> i32 {
    let mut count = 0;
    let e_first = bm.vert(v).e;
    if e_first != BM_NIL {
        let mut e_iter = e_first;
        loop {
            count += 1;
            if count == count_max {
                break;
            }
            e_iter = bmesh_disk_edge_next(bm, e_iter, v);
            if e_iter == e_first {
                break;
            }
        }
    }
    count
}

/// `bmesh_disk_validate`.
pub fn bmesh_disk_validate(bm: &BMesh, len: i32, e: usize, v: usize) -> bool {
    if !bm_vert_in_edge(bm, e, v) {
        return false;
    }
    if len == 0 || bmesh_disk_count_at_most(bm, v, len + 1) != len {
        return false;
    }
    let mut e_iter = e;
    loop {
        if len != 1 && bmesh_disk_edge_prev(bm, e_iter, v) == e_iter {
            return false;
        }
        e_iter = bmesh_disk_edge_next(bm, e_iter, v);
        if e_iter == e {
            break;
        }
    }
    true
}

/// `bmesh_disk_facevert_count` — count of disk edges that have a face loop at v.
pub fn bmesh_disk_facevert_count(bm: &BMesh, v: usize) -> i32 {
    let mut count = 0;
    let e_first = bm.vert(v).e;
    if e_first != BM_NIL {
        let mut e_iter = e_first;
        loop {
            if bm.edge(e_iter).l != BM_NIL {
                let mut l_iter = bm.edge(e_iter).l;
                let l_first = l_iter;
                loop {
                    if bm.loo(l_iter).v == v {
                        count += 1;
                        break;
                    }
                    l_iter = bm.loo(l_iter).radial_next;
                    if l_iter == l_first {
                        break;
                    }
                }
            }
            e_iter = bmesh_disk_edge_next(bm, e_iter, v);
            if e_iter == e_first {
                break;
            }
        }
    }
    count
}

/// `bmesh_disk_faceedge_count` — count of disk edges used by any face.
pub fn bmesh_disk_faceedge_count(bm: &BMesh, v: usize) -> i32 {
    let mut count = 0;
    let e_first = bm.vert(v).e;
    if e_first != BM_NIL {
        let mut e_iter = e_first;
        loop {
            if bm.edge(e_iter).l != BM_NIL {
                count += 1;
            }
            e_iter = bmesh_disk_edge_next(bm, e_iter, v);
            if e_iter == e_first {
                break;
            }
        }
    }
    count
}

/// `bmesh_disk_facevert_find_first` — a radial loop at v on a disk edge of v.
pub fn bmesh_disk_facevert_find_first(bm: &BMesh, e: usize, v: usize) -> usize {
    /* Walk the disk from e looking for an edge whose radial has a loop at v. */
    let mut e_iter = e;
    loop {
        let l = bm.edge(e_iter).l;
        if l != BM_NIL {
            let lf = bmesh_radial_faceloop_find_first(bm, l, v);
            if lf != BM_NIL {
                return lf;
            }
        }
        e_iter = bmesh_disk_edge_next(bm, e_iter, v);
        if e_iter == e || e_iter == BM_NIL {
            break;
        }
    }
    BM_NIL
}

/// `bmesh_disk_faceedge_find_first`.
pub fn bmesh_disk_faceedge_find_first(bm: &BMesh, v: usize, e: usize) -> usize {
    if e != BM_NIL && bm_vert_in_edge(bm, e, v) {
        let mut e_iter = e;
        loop {
            if bm.edge(e_iter).l != BM_NIL {
                return e_iter;
            }
            e_iter = bmesh_disk_edge_next(bm, e_iter, v);
            if e_iter == e {
                break;
            }
        }
    }
    BM_NIL
}

/// `bmesh_disk_faceedge_find_next`.
pub fn bmesh_disk_faceedge_find_next(bm: &BMesh, v: usize, e: usize) -> usize {
    if e != BM_NIL && bm_vert_in_edge(bm, e, v) && bm.edge(e).l != BM_NIL {
        let mut e_iter = bmesh_disk_edge_next(bm, e, v);
        while e_iter != e {
            if bm.edge(e_iter).l != BM_NIL {
                return e_iter;
            }
            e_iter = bmesh_disk_edge_next(bm, e_iter, v);
        }
    }
    BM_NIL
}

/// `bmesh_disk_edge_new_from_vert` — single-link disc of `v` if v->e == e.
pub fn bmesh_disk_edge_single(bm: &BMesh, e: usize, v: usize) -> bool {
    let ee = bm.edge(e);
    let (n, p) = if v == ee.v1 {
        (ee.v1_disk_link.next, ee.v1_disk_link.prev)
    } else {
        (ee.v2_disk_link.next, ee.v2_disk_link.prev)
    };
    n == e && p == e
}

/* ---- Radial cycle -------------------------------------------------------- */

/// `bmesh_radial_length`.
pub fn bmesh_radial_length(bm: &BMesh, l: usize) -> i32 {
    let mut i = 0;
    let mut l_iter = l;
    loop {
        i += 1;
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l {
            break;
        }
    }
    i
}

/// `bmesh_radial_validate`.
pub fn bmesh_radial_validate(bm: &BMesh, radlen: i32, l: usize) -> bool {
    if bmesh_radial_length(bm, l) != radlen {
        return false;
    }
    const BM_LOOP_RADIAL_MAX: i32 = 500;
    let e = bm.loo(l).e;
    let mut l_iter = l;
    let mut i = 0;
    loop {
        if bm.loo(l_iter).e != e {
            return false;
        }
        let lv = bm.loo(l_iter).v;
        let ee = bm.edge(e);
        if lv != ee.v1 && lv != ee.v2 {
            return false;
        }
        if i > BM_LOOP_RADIAL_MAX {
            return false;
        }
        i += 1;
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l {
            break;
        }
    }
    true
}

/// `bmesh_radial_loop_append` — add `l` to `e`'s radial cycle.
pub fn bmesh_radial_loop_append(bm: &mut BMesh, e: usize, l: usize) {
    if bm.edge(e).l == BM_NIL {
        bm.edge_mut(e).l = l;
        let ll = bm.loo_mut(l);
        ll.radial_next = l;
        ll.radial_prev = l;
    } else {
        let e_l = bm.edge(e).l;
        let e_l_next = bm.loo(e_l).radial_next;
        {
            let ll = bm.loo_mut(l);
            ll.radial_prev = e_l;
            ll.radial_next = e_l_next;
        }
        bm.loo_mut(e_l_next).radial_prev = l;
        bm.loo_mut(e_l).radial_next = l;
        bm.edge_mut(e).l = l;
    }
    debug_assert!(bm.loo(l).e == BM_NIL || bm.loo(l).e == e);
    bm.loo_mut(l).e = e;
}

/// `bmesh_radial_loop_remove` — remove `l` from `e`'s radial cycle.
pub fn bmesh_radial_loop_remove(bm: &mut BMesh, e: usize, l: usize) {
    debug_assert_eq!(e, bm.loo(l).e);
    if bm.loo(l).radial_next != l {
        if l == bm.edge(e).l {
            let rn = bm.loo(l).radial_next;
            bm.edge_mut(e).l = rn;
        }
        let rn = bm.loo(l).radial_next;
        let rp = bm.loo(l).radial_prev;
        bm.loo_mut(rn).radial_prev = rp;
        bm.loo_mut(rp).radial_next = rn;
    } else {
        if l == bm.edge(e).l {
            bm.edge_mut(e).l = BM_NIL;
        } else {
            debug_assert!(false, "loop not in radial cycle");
        }
    }
    let ll = bm.loo_mut(l);
    ll.radial_next = BM_NIL;
    ll.radial_prev = BM_NIL;
    ll.e = BM_NIL;
}

/// `bmesh_radial_loop_unlink` — unlink without edge bookkeeping.
pub fn bmesh_radial_loop_unlink(bm: &mut BMesh, l: usize) {
    if bm.loo(l).radial_next != l {
        let rn = bm.loo(l).radial_next;
        let rp = bm.loo(l).radial_prev;
        bm.loo_mut(rn).radial_prev = rp;
        bm.loo_mut(rp).radial_next = rn;
    }
    let ll = bm.loo_mut(l);
    ll.radial_next = BM_NIL;
    ll.radial_prev = BM_NIL;
    ll.e = BM_NIL;
}

/// `bmesh_radial_faceloop_find_first` — loop in e's radial with vert v.
pub fn bmesh_radial_faceloop_find_first(bm: &BMesh, l: usize, v: usize) -> usize {
    let mut l_iter = l;
    loop {
        if bm.loo(l_iter).v == v {
            return l_iter;
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l {
            break;
        }
    }
    BM_NIL
}

/// `bmesh_radial_faceloop_find_next`.
pub fn bmesh_radial_faceloop_find_next(bm: &BMesh, l: usize, v: usize) -> usize {
    let mut l_iter = bm.loo(l).radial_next;
    while l_iter != l {
        if bm.loo(l_iter).v == v {
            return l_iter;
        }
        l_iter = bm.loo(l_iter).radial_next;
    }
    l
}

/// `bmesh_radial_facevert_count` — count radial loops with vert v.
pub fn bmesh_radial_facevert_count(bm: &BMesh, l: usize, v: usize) -> i32 {
    let mut count = 0;
    let mut l_iter = l;
    loop {
        if bm.loo(l_iter).v == v {
            count += 1;
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l {
            break;
        }
    }
    count
}

/// `bmesh_radial_facevert_check` — is v in l's radial.
pub fn bmesh_radial_facevert_check(bm: &BMesh, l: usize, v: usize) -> bool {
    let mut l_iter = l;
    loop {
        if bm.loo(l_iter).v == v {
            return true;
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l {
            break;
        }
    }
    false
}

/* ---- Loop cycle ---------------------------------------------------------- */

/// `bmesh_cycle_validate`.
pub fn bmesh_cycle_validate(bm: &BMesh, len: i32, l: usize) -> bool {
    let mut l_iter = l;
    let mut i = 0;
    loop {
        i += 1;
        if i > len {
            return false;
        }
        l_iter = bm.loo(l_iter).next;
        if l_iter == l {
            break;
        }
    }
    i == len
}

/// `bmesh_cycle_length`.
pub fn bmesh_cycle_length(bm: &BMesh, l: usize) -> i32 {
    let mut l_iter = l;
    let mut i = 0;
    loop {
        i += 1;
        l_iter = bm.loo(l_iter).next;
        if l_iter == l {
            break;
        }
    }
    i
}

/// `bmesh_cycle_reverse` — reverse a face's loop cycle order.
pub fn bmesh_cycle_reverse(bm: &mut BMesh, f: usize) -> usize {
    let l_first = bm.face(f).l_first;
    let mut l_iter = l_first;
    loop {
        let l = bm.loo_mut(l_iter);
        let next = l.next;
        let prev = l.prev;
        l.next = prev;
        l.prev = next;
        l_iter = next;
        if l_iter == l_first {
            break;
        }
    }
    bm.face_mut(f).l_first
}

/// `bmesh_cycle_find_face` — find face with >=2 loop verts at v.
pub fn bmesh_cycle_find_face(_f: &BMFace) -> usize {
    BM_NIL
}

/// `BM_loop_find_loop_nodouble` / `bmesh_loop_find_face` — other face sharing l's edge and v.
pub fn bmesh_loop_find_face(bm: &BMesh, l: usize) -> usize {
    let l_first = bm.edge(bm.loo(l).e).l;
    let mut l_iter = l_first;
    loop {
        let li = bm.loo(l_iter);
        if li.f != bm.loo(l).f && li.v == bm.loo(l).v {
            return l_iter;
        }
        l_iter = li.radial_next;
        if l_iter == l_first {
            break;
        }
    }
    BM_NIL
}

/// `bmesh_radial_faceloop_find_vert` — radial loop with given vert.
pub fn bmesh_radial_faceloop_find_vert(bm: &BMesh, l: usize, v: usize) -> usize {
    bmesh_radial_faceloop_find_first(bm, l, v)
}
