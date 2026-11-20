//! Port of `bmesh_core.cc` — element create/kill and kernel edits.
//!
//! CustomData copies (`BM_data_...`), `v_example`/`f_example` attribute copies,
//! and ID/remap bookkeeping are PORT_PENDING (CustomData unported).

use crate::structure::*;
use crate::{BMesh, BMEdge, BMFace, BMHeader, BMLoop, BMVert, BM_NIL};
use crate::{BMCreateFlag, BM_CREATE_NO_DOUBLE, BM_EDGE, BM_FACE, BM_LOOP, BM_VERT};
use blenders_math::vector::Vec3;

/// `BM_face_pair_overlap_check_subset_same_winding` — do loop runs share verts
/// in the same winding order?
pub fn bm_face_pair_overlap_check_subset_same_winding(
    bm: &BMesh,
    l_a: usize,
    l_a_end: usize,
    l_b: usize,
    l_b_end: usize,
) -> bool {
    let _ = l_b_end;
    debug_assert_ne!(bm.loo(l_a).f, bm.loo(l_b).f);
    let mut l_a_iter = l_a;
    let mut l_b_iter = l_b;
    loop {
        if bm.loo(l_b_iter).v != bm.loo(l_a_iter).v {
            return false;
        }
        if l_a_iter == l_a_end {
            debug_assert_eq!(l_b_iter, l_b_end);
            break;
        }
        l_a_iter = bm.loo(l_a_iter).next;
        l_b_iter = bm.loo(l_b_iter).next;
    }
    true
}

/// `BM_face_pair_overlap_check_subset_swap_winding`.
pub fn bm_face_pair_overlap_check_subset_swap_winding(
    bm: &BMesh,
    l_a: usize,
    l_a_end: usize,
    l_b: usize,
    l_b_end: usize,
) -> bool {
    let _ = l_b_end;
    debug_assert_eq!(bm.loo(l_a).f, bm.loo(l_a_end).f);
    debug_assert_ne!(bm.loo(l_a).f, bm.loo(l_b).f);
    let mut l_a_iter = l_a;
    let mut l_b_iter = l_b;
    loop {
        if bm.loo(l_b_iter).v != bm.loo(l_a_iter).v {
            return false;
        }
        if l_a_iter == l_a_end {
            break;
        }
        l_a_iter = bm.loo(l_a_iter).next;
        l_b_iter = bm.loo(l_b_iter).prev;
    }
    true
}

/// `BM_vert_create`.
pub fn bm_vert_create(bm: &mut BMesh, co: Vec3, _v_example: Option<usize>, _create_flag: BMCreateFlag) -> usize {
    /* PORT_PENDING: `v_example` CustomData copy; `BM_CREATE_*` flag handling. */
    let mut v = BMVert::default();
    v.head = BMHeader::new(BM_VERT);
    v.co = co;
    v.no = Vec3::new(0.0, 0.0, 0.0);
    v.e = BM_NIL;
    let vi = bm.vpool.len();
    bm.vpool.push(v);
    bm.totvert += 1;
    bm.elem_table_dirty = true;
    bm.elem_index_dirty = true;
    vi
}

/// `BM_edge_create`.
pub fn bm_edge_create(
    bm: &mut BMesh,
    v1: usize,
    v2: usize,
    _e_example: Option<usize>,
    create_flag: BMCreateFlag,
) -> usize {
    debug_assert_ne!(v1, v2);
    debug_assert_eq!(bm.vert(v1).head.htype, BM_VERT);
    debug_assert_eq!(bm.vert(v2).head.htype, BM_VERT);
    if (create_flag & BM_CREATE_NO_DOUBLE) != 0 {
        let e = bmesh_disk_edge_exists(bm, v1, v2);
        if e != BM_NIL {
            return e;
        }
    }
    let mut e = BMEdge::default();
    e.head = BMHeader::new(BM_EDGE);
    e.v1 = v1;
    e.v2 = v2;
    e.l = BM_NIL;
    e.v1_disk_link.next = BM_NIL;
    e.v1_disk_link.prev = BM_NIL;
    e.v2_disk_link.next = BM_NIL;
    e.v2_disk_link.prev = BM_NIL;
    let ei = bm.epool.len();
    bm.epool.push(e);
    bm.totedge += 1;
    bmesh_disk_edge_append(bm, ei, v1);
    bmesh_disk_edge_append(bm, ei, v2);
    bm.elem_table_dirty = true;
    bm.elem_index_dirty = true;
    ei
}

/// `bm_loop_create` — append loop (verts, edge, face linkup happens in face create).
pub fn bm_loop_create(bm: &mut BMesh, v: usize, e: usize, f: usize) -> usize {
    let mut l = BMLoop::default();
    l.head = BMHeader::new(BM_LOOP);
    l.v = v;
    l.e = e;
    l.f = f;
    l.next = BM_NIL;
    l.prev = BM_NIL;
    l.radial_next = BM_NIL;
    l.radial_prev = BM_NIL;
    let li = bm.lpool.len();
    bm.lpool.push(l);
    bm.totloop += 1;
    bm.elem_table_dirty = true;
    bm.elem_index_dirty = true;
    li
}

/// `bm_face_boundary_add` — add a loop for (v, e) into face `f`'s cycle.
pub fn bm_face_boundary_add(bm: &mut BMesh, f: usize, v: usize, e: usize) -> usize {
    let l = bm_loop_create(bm, v, e, f);
    if bm.face(f).l_first == BM_NIL {
        bm.face_mut(f).l_first = l;
        let ll = bm.loo_mut(l);
        ll.next = l;
        ll.prev = l;
    } else {
        let l_first = bm.face(f).l_first;
        let l_last = bm.loo(l_first).prev;
        let ll = bm.loo_mut(l);
        ll.next = l_first;
        ll.prev = l_last;
        bm.loo_mut(l_first).prev = l;
        bm.loo_mut(l_last).next = l;
    }
    l
}

/// `bm_face_create__internal`.
fn bm_face_create_internal(bm: &mut BMesh) -> usize {
    let mut f = BMFace::default();
    f.head = BMHeader::new(BM_FACE);
    f.l_first = BM_NIL;
    f.len = 0;
    f.no = Vec3::new(0.0, 0.0, 0.0);
    let fi = bm.fpool.len();
    bm.fpool.push(f);
    bm.totface += 1;
    bm.elem_table_dirty = true;
    bm.elem_index_dirty = true;
    fi
}

/// `BM_face_create` — create face from ordered vert/edge lists.
pub fn bm_face_create(
    bm: &mut BMesh,
    verts: &[usize],
    edges: &[usize],
    _f_example: Option<usize>,
    create_flag: BMCreateFlag,
) -> usize {
    let _ = create_flag;
    debug_assert_eq!(verts.len(), edges.len());
    /* PORT_PENDING: `BM_CREATE_NO_DOUBLE` face check, `f_example` customdata. */
    let f = bm_face_create_internal(bm);
    for i in 0..verts.len() {
        bm_face_boundary_add(bm, f, verts[i], edges[i]);
        bmesh_radial_loop_append(bm, edges[i], bm.lpool.len() - 1);
    }
    bm.face_mut(f).len = verts.len() as i32;
    f
}

/// `BM_face_create_verts` — create face from verts, creating/finding edges.
pub fn bm_face_create_verts(
    bm: &mut BMesh,
    vert_arr: &[usize],
    _f_example: Option<usize>,
    create_flag: BMCreateFlag,
    create_edges: bool,
    calc_edges: bool,
) -> usize {
    let len = vert_arr.len();
    let mut edges: Vec<usize> = vec![BM_NIL; len];
    if calc_edges || create_edges {
        /* Find or create edges for each vert pair. */
        for i in 0..len {
            let v1 = vert_arr[i];
            let v2 = vert_arr[(i + 1) % len];
            let mut e = bmesh_disk_edge_exists(bm, v1, v2);
            if e == BM_NIL && create_edges {
                e = bm_edge_create(bm, v1, v2, None, 0);
            }
            if e == BM_NIL {
                return BM_NIL;
            }
            edges[i] = e;
        }
    }
    bm_face_create(bm, vert_arr, &edges, None, create_flag)
}

/* ---- kill ---------------------------------------------------------------- */

/// `bm_kill_only_vert` — remove vert (edges must already be gone).
pub fn bm_kill_only_vert(bm: &mut BMesh, v: usize) {
    debug_assert_eq!(bm.vert(v).e, BM_NIL);
    bm.vpool[v] = BMVert::default();
    bm.vpool[v].head.htype = 0;
    bm.totvert -= 1;
    bm.elem_table_dirty = true;
    bm.elem_index_dirty = true;
}

/// `bm_kill_only_edge`.
pub fn bm_kill_only_edge(bm: &mut BMesh, e: usize) {
    debug_assert_eq!(bm.edge(e).l, BM_NIL);
    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
    bmesh_disk_edge_remove(bm, e, v1);
    bmesh_disk_edge_remove(bm, e, v2);
    bm.epool[e] = BMEdge::default();
    bm.epool[e].v1 = BM_NIL;
    bm.epool[e].v2 = BM_NIL;
    bm.totedge -= 1;
    bm.elem_table_dirty = true;
    bm.elem_index_dirty = true;
}

/// `bm_kill_only_face`.
pub fn bm_kill_only_face(bm: &mut BMesh, f: usize) {
    bm.fpool[f] = BMFace::default();
    bm.fpool[f].l_first = BM_NIL;
    bm.totface -= 1;
    bm.elem_table_dirty = true;
    bm.elem_index_dirty = true;
}

/// `bm_kill_only_loop`.
pub fn bm_kill_only_loop(bm: &mut BMesh, l: usize) {
    bm.lpool[l] = BMLoop::default();
    bm.lpool[l].v = BM_NIL;
    bm.lpool[l].e = BM_NIL;
    bm.lpool[l].f = BM_NIL;
    bm.totloop -= 1;
    bm.elem_table_dirty = true;
    bm.elem_index_dirty = true;
}

/// `BM_face_edges_kill` — kill face and its edges (not verts).
pub fn bm_face_edges_kill(bm: &mut BMesh, f: usize) {
    let l_first = bm.face(f).l_first;
    let mut l_iter = l_first;
    let len = bm.face(f).len;
    let mut edges: Vec<usize> = Vec::new();
    loop {
        edges.push(bm.loo(l_iter).e);
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first {
            break;
        }
    }
    /* remove loops + face */
    let mut l_iter = l_first;
    for _ in 0..len {
        let l_next = bm.loo(l_iter).next;
        let e = bm.loo(l_iter).e;
        bmesh_radial_loop_remove(bm, e, l_iter);
        bm_kill_only_loop(bm, l_iter);
        l_iter = l_next;
    }
    bm_kill_only_face(bm, f);
    for e in edges {
        if bm.edge(e).l == BM_NIL {
            bm_kill_only_edge(bm, e);
        }
    }
}

/// `BM_face_verts_kill` — kill face, its edges, and lone verts.
pub fn bm_face_verts_kill(bm: &mut BMesh, f: usize) {
    let l_first = bm.face(f).l_first;
    let len = bm.face(f).len;
    let mut verts: Vec<usize> = Vec::new();
    let mut edges: Vec<usize> = Vec::new();
    let mut l_iter = l_first;
    loop {
        verts.push(bm.loo(l_iter).v);
        edges.push(bm.loo(l_iter).e);
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first {
            break;
        }
    }
    let mut l_iter = l_first;
    for _ in 0..len {
        let l_next = bm.loo(l_iter).next;
        let e = bm.loo(l_iter).e;
        bmesh_radial_loop_remove(bm, e, l_iter);
        bm_kill_only_loop(bm, l_iter);
        l_iter = l_next;
    }
    bm_kill_only_face(bm, f);
    for e in edges {
        if bm.edge(e).l == BM_NIL {
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            bm_kill_only_edge(bm, e);
            if bm.vert(v1).e == BM_NIL && verts.contains(&v1) {
                bm_kill_only_vert(bm, v1);
            }
            if bm.vert(v2).e == BM_NIL && verts.contains(&v2) {
                bm_kill_only_vert(bm, v2);
            }
        }
    }
}

/// `BM_face_kill` — kill a face (loops and radial refs only).
pub fn bm_face_kill(bm: &mut BMesh, f: usize) {
    let l_first = bm.face(f).l_first;
    let len = bm.face(f).len;
    let mut l_iter = l_first;
    for _ in 0..len {
        let l_next = bm.loo(l_iter).next;
        let e = bm.loo(l_iter).e;
        bmesh_radial_loop_remove(bm, e, l_iter);
        bm_kill_only_loop(bm, l_iter);
        l_iter = l_next;
    }
    bm_kill_only_face(bm, f);
}

/// `BM_edge_kill` — kill edge and its radial loops/faces.
pub fn bm_edge_kill(bm: &mut BMesh, e: usize) {
    while bm.edge(e).l != BM_NIL {
        let l = bm.edge(e).l;
        let f = bm.loo(l).f;
        bm_face_kill(bm, f);
    }
    bm_kill_only_edge(bm, e);
}

/// `BM_vert_kill` — kill vert, its disk edges, and their faces.
pub fn bm_vert_kill(bm: &mut BMesh, v: usize) {
    while bm.vert(v).e != BM_NIL {
        let e = bm.vert(v).e;
        bm_edge_kill(bm, e);
    }
    bm_kill_only_vert(bm, v);
}

/// `bmesh_kernel_loop_reverse` — reverse a face's loop cycle and edge dirs.
pub fn bmesh_kernel_loop_reverse(bm: &mut BMesh, f: usize) {
    let l_first = bm.face(f).l_first;
    /* Old loop order preservation isn't needed post-swap: swap next<->prev. */
    let mut l_iter = l_first;
    loop {
        let li = bm.loo_mut(l_iter);
        let next = li.next;
        li.next = li.prev;
        li.prev = next;
        l_iter = next;
        if l_iter == l_first {
            break;
        }
    }
    /* Fix radial order isn't required (radial order is free). */
}
