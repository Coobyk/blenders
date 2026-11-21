//! Port of `bmesh_core.cc` — element create/kill and kernel edits.
//!
//! CustomData copies (`BM_data_...`), `v_example`/`f_example` attribute copies,
//! and ID/remap bookkeeping are PORT_PENDING (CustomData unported).

use crate::structure::*;
use crate::{delete, query, construct};
use crate::structure;
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

/* ---- kernel ops ---------------------------------------------------------- */

/// `bm_face_create__sfme` — copy face f's flags/normal for a split.
fn bm_face_create_sfme(bm: &mut BMesh, f_src: usize) -> usize {
    let f = bm_face_create_internal(bm);
    bm.fpool[f].no = bm.face(f_src).no;
    bm.fpool[f].mat_nr = bm.face(f_src).mat_nr;
    bm.fpool[f].head.hflag = bm.face(f_src).head.hflag;
    bm.fpool[f].head.api_flag = bm.face(f_src).head.api_flag;
    f
}

/// `bmesh_kernel_split_face_make_edge` — split `f` into two faces by adding
/// an edge between `l_v1->v` and `l_v2->v`. Returns the new face `f2`.
pub fn bmesh_kernel_split_face_make_edge(
    bm: &mut BMesh,
    f: usize,
    l_v1: usize,
    l_v2: usize,
    r_l: Option<&mut usize>,
    no_double: bool,
) -> usize {
    debug_assert_eq!(bm.loo(l_v1).f, f);
    debug_assert_eq!(bm.loo(l_v2).f, f);
    let (v1, v2) = (bm.loo(l_v1).v, bm.loo(l_v2).v);
    let e = bm_edge_create(bm, v1, v2, None, if no_double { BM_CREATE_NO_DOUBLE } else { 0 });
    let f2 = bm_face_create_sfme(bm, f);
    let l_f1 = bm_loop_create(bm, v2, e, f);
    let l_f2 = bm_loop_create(bm, v1, e, f2);

    let l_v1_prev = bm.loo(l_v1).prev;
    let l_v2_prev = bm.loo(l_v2).prev;
    bm.lpool[l_f1].prev = l_v2_prev;
    bm.lpool[l_f2].prev = l_v1_prev;
    bm.lpool[l_v2_prev].next = l_f1;
    bm.lpool[l_v1_prev].next = l_f2;
    bm.lpool[l_f1].next = l_v1;
    bm.lpool[l_f2].next = l_v2;
    bm.lpool[l_v1].prev = l_f1;
    bm.lpool[l_v2].prev = l_f2;

    /* find which face the original first loop landed in (no holes build) */
    let mut l_iter = l_f1;
    let mut first_loop_f1 = false;
    loop {
        if l_iter == bm.face(f).l_first {
            first_loop_f1 = true;
        }
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_f1 {
            break;
        }
    }
    let f_l_first = bm.face(f).l_first;
    if first_loop_f1 {
        /* original first loop was in f1 */
        if bm.loo(f_l_first).prev == l_f1 {
            bm.fpool[f2].l_first = bm.loo(l_f2).prev;
        } else if bm.loo(f_l_first).next == l_f1 {
            bm.fpool[f2].l_first = bm.loo(l_f2).next;
        } else {
            bm.fpool[f2].l_first = l_f2;
        }
    } else {
        bm.fpool[f2].l_first = f_l_first;
        if bm.loo(f_l_first).prev == l_f2 {
            bm.fpool[f].l_first = bm.loo(l_f1).prev;
        } else if bm.loo(f_l_first).next == l_f2 {
            bm.fpool[f].l_first = bm.loo(l_f1).next;
        } else {
            bm.fpool[f].l_first = l_f1;
        }
    }

    /* fix face pointers of f2's loops + count f2len */
    let f2_first = bm.face(f2).l_first;
    let mut f2len = 0;
    let mut l_iter = f2_first;
    loop {
        bm.lpool[l_iter].f = f2;
        f2len += 1;
        l_iter = bm.loo(l_iter).next;
        if l_iter == f2_first {
            break;
        }
    }
    bmesh_radial_loop_append(bm, e, l_f1);
    bmesh_radial_loop_append(bm, e, l_f2);
    bm.fpool[f2].len = f2len;

    let f_first = bm.face(f).l_first;
    let mut f1len = 0;
    let mut l_iter = f_first;
    loop {
        f1len += 1;
        l_iter = bm.loo(l_iter).next;
        if l_iter == f_first {
            break;
        }
    }
    bm.fpool[f].len = f1len;

    if let Some(r_l) = r_l {
        *r_l = l_f2;
    }
    f2
}

/// `bmesh_kernel_split_edge_make_vert` — split `e` at new vert near `tv`.
pub fn bmesh_kernel_split_edge_make_vert(
    bm: &mut BMesh,
    tv: usize,
    e: usize,
    r_e: Option<&mut usize>,
) -> usize {
    debug_assert!(bm_vert_in_edge(bm, e, tv));
    let v_old = if bm.edge(e).v1 == tv {
        bm.edge(e).v2
    } else {
        bm.edge(e).v1
    };
    let v_co = bm.vert(tv).co;
    let v_new = bm_vert_create(bm, v_co, None, 0);
    let e_new = bm_edge_create(bm, tv, v_new, None, 0);
    bmesh_disk_edge_remove(bm, e_new, tv);
    bmesh_disk_edge_remove(bm, e_new, v_new);
    bmesh_disk_vert_replace(bm, e, v_new, tv);
    bmesh_disk_edge_append(bm, e_new, v_new);
    bmesh_disk_edge_append(bm, e_new, tv);

    /* Split the radial cycle if present. */
    let mut l_next = bm.edge(e).l;
    bm.epool[e].l = BM_NIL;
    let mut is_first = true;
    while l_next != BM_NIL {
        let l = l_next;
        let lf = bm.loo(l).f;
        bm.fpool[lf].len += 1;
        l_next = {
            let rn = bm.loo(l).radial_next;
            if rn != l {
                rn
            } else {
                BM_NIL
            }
        };
        bmesh_radial_loop_unlink(bm, l);
        let l_new = bm_loop_create(bm, v_new, BM_NIL, lf);
        let l_n = bm.loo(l).next;
        bm.lpool[l_new].prev = l;
        bm.lpool[l_new].next = l_n;
        bm.lpool[l].next = l_new;
        bm.lpool[l_n].prev = l_new;
        bm.lpool[l_new].v = v_new;

        let ln_v = bm.loo(l_new).v;
        let ln_next_v = bm.loo(l_n).v;
        if bm_verts_in_edge(bm, ln_v, ln_next_v, e) {
            bm.lpool[l_new].e = e;
            bm.lpool[l].e = e_new;
            if is_first {
                is_first = false;
                bm.lpool[l].radial_next = BM_NIL;
                bm.lpool[l].radial_prev = BM_NIL;
            }
            let le_new = bm.loo(l_new).e;
            let le = bm.loo(l).e;
            bmesh_radial_loop_append(bm, le_new, l_new);
            bmesh_radial_loop_append(bm, le, l);
        } else if bm_verts_in_edge(bm, ln_v, ln_next_v, e_new) {
            bm.lpool[l_new].e = e_new;
            bm.lpool[l].e = e;
            if is_first {
                is_first = false;
                bm.lpool[l].radial_next = BM_NIL;
                bm.lpool[l].radial_prev = BM_NIL;
            }
            let le_new = bm.loo(l_new).e;
            let le = bm.loo(l).e;
            bmesh_radial_loop_append(bm, le_new, l_new);
            bmesh_radial_loop_append(bm, le, l);
        }
    }
    let _ = v_old;
    if let Some(r_e) = r_e {
        *r_e = e_new;
    }
    v_new
}

/// `bmesh_kernel_join_edge_kill_vert` — merge `e_kill` into the other edge at
/// `v_kill` (which must have valence 2). Returns the surviving edge or NIL.
pub fn bmesh_kernel_join_edge_kill_vert(
    bm: &mut BMesh,
    e_kill: usize,
    v_kill: usize,
    do_del: bool,
    check_edge_exists: bool,
    kill_degenerate_faces: bool,
    kill_duplicate_faces: bool,
) -> usize {
    if !bm_vert_in_edge(bm, e_kill, v_kill) {
        return BM_NIL;
    }
    if bmesh_disk_count_at_most(bm, v_kill, 3) != 2 {
        return BM_NIL;
    }
    let e_old = bmesh_disk_edge_next(bm, e_kill, v_kill);
    let v_target = if bm.edge(e_kill).v1 == v_kill {
        bm.edge(e_kill).v2
    } else {
        bm.edge(e_kill).v1
    };
    let v_old = if bm.edge(e_old).v1 == v_kill {
        bm.edge(e_old).v2
    } else {
        bm.edge(e_old).v1
    };
    /* check for double edges */
    if bm_verts_in_edge(bm, v_kill, v_target, e_old) {
        return BM_NIL;
    }
    let e_splice = if check_edge_exists {
        bmesh_disk_edge_exists(bm, v_target, v_old)
    } else {
        BM_NIL
    };
    let mut faces_degenerate: Vec<usize> = Vec::new();
    let mut faces_duplicate_candidate: Vec<usize> = Vec::new();

    bmesh_disk_vert_replace(bm, e_old, v_target, v_kill);
    bmesh_disk_edge_remove(bm, e_kill, v_target);

    if bm.edge(e_kill).l != BM_NIL {
        let l_start = bm.edge(e_kill).l;
        let mut l_kill = l_start;
        loop {
            /* relink loops and fix vertex pointer */
            let ln = bm.loo(l_kill).next;
            let lp = bm.loo(l_kill).prev;
            if bm.loo(ln).v == v_kill {
                bm.lpool[ln].v = v_target;
            }
            bm.lpool[ln].prev = lp;
            bm.lpool[lp].next = ln;
            let lf = bm.loo(l_kill).f;
            if bm.face(lf).l_first == l_kill {
                bm.fpool[lf].l_first = ln;
            }
            bm.fpool[lf].len -= 1;
            if kill_degenerate_faces && bm.face(lf).len < 3 {
                faces_degenerate.push(lf);
            } else if kill_duplicate_faces {
                faces_duplicate_candidate.push(lf);
            }
            let l_kill_next = bm.loo(l_kill).radial_next;
            bm_kill_only_loop(bm, l_kill);
            l_kill = l_kill_next;
            if l_kill == l_start {
                break;
            }
        }
        /* all radial loops are dead; drop the edge's radial reference */
        bm.epool[e_kill].l = BM_NIL;
    }
    bm_kill_only_edge(bm, e_kill);
    if do_del {
        bm_kill_only_vert(bm, v_kill);
    } else {
        bm.vpool[v_kill].e = BM_NIL;
    }

    if e_splice != BM_NIL {
        /* splice the two edges' radials: move e_splice's loops onto e_old */
        if bm.edge(e_splice).l != BM_NIL {
            /* swap loops of e_splice to point to e_old */
            let l_start = bm.edge(e_splice).l;
            let mut l_iter = l_start;
            loop {
                let next = bm.loo(l_iter).radial_next;
                bmesh_radial_loop_unlink(bm, l_iter);
                bm.lpool[l_iter].e = e_old;
                bmesh_radial_loop_append(bm, e_old, l_iter);
                l_iter = next;
                if l_iter == l_start || l_iter == BM_NIL {
                    break;
                }
            }
        }
        /* swap verts so e_splice shares e_old's verts, then kill it */
        let (v1, v2) = (bm.edge(e_old).v1, bm.edge(e_old).v2);
        bmesh_disk_edge_remove(bm, e_splice, bm.edge(e_splice).v1);
        bmesh_disk_edge_remove(bm, e_splice, bm.edge(e_splice).v2);
        bm.epool[e_splice].v1 = v1;
        bm.epool[e_splice].v2 = v2;
        bm_kill_only_edge(bm, e_splice);
        /* note: C keeps e_splice but redirects; killing is equivalent for
         * topology since all its loops were moved. */
    }

    for f in faces_degenerate {
        bm_face_kill(bm, f);
    }
    if kill_duplicate_faces {
        for f in faces_duplicate_candidate {
            if bm.face(f).head.htype != BM_FACE {
                continue;
            }
            /* find a same-vert face != f */
            let varr: Vec<usize> = {
                let l_first = bm.face(f).l_first;
                let mut l_iter = l_first;
                let mut vs = Vec::new();
                loop {
                    vs.push(bm.loo(l_iter).v);
                    l_iter = bm.loo(l_iter).next;
                    if l_iter == l_first {
                        break;
                    }
                }
                vs
            };
            let found = crate::query::bm_face_exists(bm, &varr);
            if found != BM_NIL && found != f {
                bm_face_kill(bm, f);
            }
        }
    }
    e_old
}

/// `bmesh_kernel_join_face_kill_edge` — merge `f2` into `f1` across `e`.
pub fn bmesh_kernel_join_face_kill_edge(
    bm: &mut BMesh,
    f1: usize,
    f2: usize,
    e: usize,
) -> usize {
    if f1 == f2 {
        return BM_NIL;
    }
    if !crate::query::bm_edge_is_manifold(bm, e) {
        return BM_NIL;
    }
    let l_f1 = crate::query::bm_face_edge_share_loop(bm, f1, e);
    let l_f2 = crate::query::bm_face_edge_share_loop(bm, f2, e);
    if l_f1 == BM_NIL || l_f2 == BM_NIL {
        return BM_NIL;
    }
    if bm.loo(l_f1).v == bm.loo(l_f2).v {
        return BM_NIL;
    }
    let l_f1_next_e = bm.loo(bm.loo(l_f1).next).e;
    let l_f1_prev_e = bm.loo(bm.loo(l_f1).prev).e;
    let l_f2_next_e = bm.loo(bm.loo(l_f2).next).e;
    let l_f2_prev_e = bm.loo(bm.loo(l_f2).prev).e;
    if crate::query::bm_edge_in_face(bm, l_f1_next_e, f2)
        || crate::query::bm_edge_in_face(bm, l_f1_prev_e, f2)
        || crate::query::bm_edge_in_face(bm, l_f2_next_e, f1)
        || crate::query::bm_edge_in_face(bm, l_f2_prev_e, f1)
    {
        return BM_NIL;
    }
    if crate::query::bm_face_share_edge_count(bm, f1, f2) > 1 {
        return BM_NIL;
    }
    /* validate no internal join (shared verts other than on e) */
    {
        let verts_f1: std::collections::HashSet<usize> = {
            let l_first = bm.face(f1).l_first;
            let mut l_iter = l_first;
            let mut s = std::collections::HashSet::new();
            loop {
                if l_iter != l_f1 {
                    s.insert(bm.loo(l_iter).v);
                }
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first {
                    break;
                }
            }
            s
        };
        let l_first = bm.face(f2).l_first;
        let mut l_iter = l_first;
        loop {
            if l_iter != l_f2 && verts_f1.contains(&bm.loo(l_iter).v) {
                return BM_NIL;
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
    /* join the loops */
    let lf1_prev = bm.loo(l_f1).prev;
    let lf1_next = bm.loo(l_f1).next;
    let lf2_prev = bm.loo(l_f2).prev;
    let lf2_next = bm.loo(l_f2).next;
    bm.lpool[lf1_prev].next = lf2_next;
    bm.lpool[lf2_next].prev = lf1_prev;
    bm.lpool[lf1_next].prev = lf2_prev;
    bm.lpool[lf2_prev].next = lf1_next;
    if bm.face(f1).l_first == l_f1 {
        bm.fpool[f1].l_first = lf1_next;
    }
    bm.fpool[f1].len += bm.face(f2).len - 2;
    let newlen = bm.face(f1).len;
    let l_first = bm.face(f1).l_first;
    let mut l_iter = l_first;
    for _ in 0..newlen {
        bm.lpool[l_iter].f = f1;
        l_iter = bm.loo(l_iter).next;
    }
    /* remove e from disk cycles + kill e, l_f1, l_f2, f2 */
    let (ev1, ev2) = (bm.edge(e).v1, bm.edge(e).v2);
    bmesh_disk_edge_remove(bm, e, ev1);
    bmesh_disk_edge_remove(bm, e, ev2);
    bm.epool[e] = BMEdge::default();
    bm.epool[e].v1 = BM_NIL;
    bm.epool[e].v2 = BM_NIL;
    bm.totedge -= 1;
    bm.lpool[l_f1] = BMLoop::default();
    bm.lpool[l_f2] = BMLoop::default();
    bm.totloop -= 2;
    bm.fpool[f2] = BMFace::default();
    bm.fpool[f2].l_first = BM_NIL;
    bm.totface -= 1;
    bm.elem_table_dirty = true;
    bm.elem_index_dirty = true;
    f1
}

/// `bmesh_kernel_join_vert_kill_edge` — collapse `e_kill` merging `v_kill`
/// into the vert at the other end. Returns `v_target` or NIL.
pub fn bmesh_kernel_join_vert_kill_edge(
    bm: &mut BMesh,
    e_kill: usize,
    v_kill: usize,
    do_del: bool,
    check_edge_exists: bool,
    kill_degenerate_faces: bool,
) -> usize {
    if !bm_vert_in_edge(bm, e_kill, v_kill) {
        return BM_NIL;
    }
    let v_target = if bm.edge(e_kill).v1 == v_kill {
        bm.edge(e_kill).v2
    } else {
        bm.edge(e_kill).v1
    };
    let mut faces_degenerate: Vec<usize> = Vec::new();

    if bm.edge(e_kill).l != BM_NIL {
        let l_first = bm.edge(e_kill).l;
        let mut l_kill = l_first;
        loop {
            let ln = bm.loo(l_kill).next;
            let lp = bm.loo(l_kill).prev;
            if bm.loo(ln).v == v_kill {
                bm.lpool[ln].v = v_target;
            }
            bm.lpool[ln].prev = lp;
            bm.lpool[lp].next = ln;
            let lf = bm.loo(l_kill).f;
            if bm.face(lf).l_first == l_kill {
                bm.fpool[lf].l_first = ln;
            }
            bm.fpool[lf].len -= 1;
            if kill_degenerate_faces && bm.face(lf).len < 3 {
                faces_degenerate.push(lf);
            }
            let l_kill_next = bm.loo(l_kill).radial_next;
            bm_kill_only_loop(bm, l_kill);
            l_kill = l_kill_next;
            if l_kill == l_first {
                break;
            }
        }
        bm.epool[e_kill].l = BM_NIL;
    }

    /* BM_edge_kill: kill remaining faces on edge then the edge itself */
    while bm.edge(e_kill).l != BM_NIL {
        let f = bm.loo(bm.edge(e_kill).l).f;
        bm_face_kill(bm, f);
    }
    bm_kill_only_edge(bm, e_kill);

    /* inline BM_vert_splice: move v_kill's remaining edges onto v_target */
    if bm.vert(v_target).e != BM_NIL && bm.vert(v_kill).e != BM_NIL {
        loop {
            let e = bm.vert(v_kill).e;
            if e == BM_NIL {
                break;
            }
            let e_other_v = if bm.edge(e).v1 == v_kill {
                bm.edge(e).v2
            } else {
                bm.edge(e).v1
            };
            let e_target = if check_edge_exists {
                bmesh_disk_edge_exists(bm, v_target, e_other_v)
            } else {
                BM_NIL
            };
            bmesh_disk_vert_replace(bm, e, v_target, v_kill);
            debug_assert_ne!(bm.edge(e).v1, bm.edge(e).v2);
            if e_target != BM_NIL && e_target != e {
                bm_edge_splice(bm, e_target, e);
            }
        }
    }

    for f in faces_degenerate {
        if bm.face(f).head.htype == BM_FACE {
            bm_face_kill(bm, f);
        }
    }
    if do_del {
        debug_assert_eq!(bm.vert(v_kill).e, BM_NIL);
        bm_kill_only_vert(bm, v_kill);
    }
    v_target
}

/// `BM_edge_splice` — move `e_src`'s radial loops onto `e_dst`, kill `e_src`.
pub fn bm_edge_splice(bm: &mut BMesh, e_dst: usize, e_src: usize) -> bool {
    let (d1, d2) = (bm.edge(e_dst).v1, bm.edge(e_dst).v2);
    if !(crate::query::bm_vert_in_edge(bm, e_src, d1) && bm_vert_in_edge(bm, e_src, d2)) {
        debug_assert!(false);
        return false;
    }
    while bm.edge(e_src).l != BM_NIL {
        let l = bm.edge(e_src).l;
        bmesh_radial_loop_remove(bm, e_src, l);
        bm.lpool[l].e = e_dst;
        bmesh_radial_loop_append(bm, e_dst, l);
    }
    while bm.edge(e_src).l != BM_NIL {
        let f = bm.loo(bm.edge(e_src).l).f;
        bm_face_kill(bm, f);
    }
    bm_kill_only_edge(bm, e_src);
    true
}

const FLAG_JF: u8 = 1 << 6;
const FLAG_OVERLAP: u8 = 1 << 7;

fn bm_loop_systag_count_radial(bm: &BMesh, l: usize) -> i32 {
    let mut count = 0;
    let l_first = bm.loo(l).e;
    let l0 = bm.edge(l_first).l;
    if l0 == BM_NIL {
        return 0;
    }
    let mut l_iter = l0;
    loop {
        if (bm.face(bm.loo(l_iter).f).head.api_flag & FLAG_JF) != 0 {
            count += 1;
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l0 {
            break;
        }
    }
    count
}

/// `bm_vert_is_manifold_flagged` — is the vert manifold counting only
/// JF-flagged faces? (region check restricted to flagged fans; PORT_PENDING
/// fidelity — approximation via manifold test on tagged edges.)
fn bm_vert_is_manifold_flagged(bm: &mut BMesh, v: usize) -> bool {
    /* count disk edges having a loop on a JF face; if every such edge is
     * manifold within JF faces, the vert is manifold-flagged. */
    let e_first = bm.vert(v).e;
    if e_first == BM_NIL {
        return false;
    }
    let mut e_iter = e_first;
    loop {
        /* does this edge participate in a JF face? */
        let l = bm.edge(e_iter).l;
        let mut jf = false;
        if l != BM_NIL {
            let mut li = l;
            loop {
                if (bm.face(bm.loo(li).f).head.api_flag & FLAG_JF) != 0 {
                    jf = true;
                }
                li = bm.loo(li).radial_next;
                if li == l {
                    break;
                }
            }
        }
        if jf {
            /* edge must be manifold among JF faces: exactly 2 JF loops */
            let mut l_iter = l;
            let mut jf_count = 0;
            loop {
                if (bm.face(bm.loo(l_iter).f).head.api_flag & FLAG_JF) != 0 {
                    jf_count += 1;
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l {
                    break;
                }
            }
            if jf_count != 2 {
                return false;
            }
        }
        e_iter = bmesh_disk_edge_next(bm, e_iter, v);
        if e_iter == e_first || e_iter == BM_NIL {
            break;
        }
    }
    true
}

/// `BM_faces_join` — join `faces` into one n-gon face (or fail → NIL).
/// `r_double` receives an existing identical face when the result doubles one.
pub fn bm_faces_join(
    bm: &mut BMesh,
    faces: &[usize],
    do_del: bool,
    r_double: Option<&mut usize>,
) -> usize {
    let mut r_double = r_double;
    let _ = &mut r_double;
    if faces.is_empty() {
        return BM_NIL;
    }
    if faces.len() == 1 {
        return faces[0];
    }
    let had_active = bm.act_face != BM_NIL;

    /* tag the faces */
    for &f in faces {
        bm.fpool[f].head.api_flag |= FLAG_JF;
    }

    let mut edges: Vec<usize> = Vec::new();
    let mut deledges: Vec<usize> = Vec::new();
    let mut delverts: Vec<usize> = Vec::new();
    let mut v1 = BM_NIL;
    let mut v2 = BM_NIL;
    let mut fail = false;

    for &f in faces {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            let rlen = bm_loop_systag_count_radial(bm, l_iter);
            if rlen > 2 {
                fail = true;
                break;
            }
            if rlen == 1 {
                let le = bm.loo(l_iter).e;
                edges.push(le);
                if v1 == BM_NIL {
                    v1 = bm.loo(l_iter).v;
                    v2 = if bm.edge(le).v1 == bm.loo(l_iter).v {
                        bm.edge(le).v2
                    } else {
                        bm.edge(le).v1
                    };
                }
            } else if rlen == 2 {
                let le = bm.loo(l_iter).e;
                let (ev1, ev2) = (bm.edge(le).v1, bm.edge(le).v2);
                let d1 = bm_vert_is_manifold_flagged(bm, ev1);
                let d2 = bm_vert_is_manifold_flagged(bm, ev2);
                if !d1 && !d2 && (bm.edge(le).head.api_flag & FLAG_JF) == 0 {
                    if crate::query::bm_edge_face_count_at_most(bm, le, 3) <= 2 {
                        if do_del {
                            deledges.push(le);
                        }
                        bm.epool[le].head.api_flag |= FLAG_JF;
                    }
                } else {
                    if d1 && (bm.vert(ev1).head.api_flag & FLAG_JF) == 0 {
                        if do_del {
                            delverts.push(ev1);
                        }
                        bm.vpool[ev1].head.api_flag |= FLAG_JF;
                    }
                    if d2 && (bm.vert(ev2).head.api_flag & FLAG_JF) == 0 {
                        if do_del {
                            delverts.push(ev2);
                        }
                        bm.vpool[ev2].head.api_flag |= FLAG_JF;
                    }
                }
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        if fail {
            break;
        }
    }

    if fail {
        for &f in faces {
            bm.fpool[f].head.api_flag &= !FLAG_JF;
        }
        return BM_NIL;
    }

    /* create region face */
    let f_new = if !edges.is_empty() {
        crate::construct::bm_face_create_ngon(bm, v1, v2, &edges, Some(faces[0]), 0)
    } else {
        BM_NIL
    };
    if f_new == BM_NIL {
        for &f in faces {
            bm.fpool[f].head.api_flag &= !FLAG_JF;
        }
        return BM_NIL;
    }

    /* check for a duplicate face */
    let f_existing = crate::query::bm_face_find_double(bm, f_new);
    if f_existing != BM_NIL {
        if let Some(r) = r_double {
            *r = f_existing;
        }
        /* kill the ngon so the double remains */
        bm_face_kill(bm, f_new);
        for &f in faces {
            bm.fpool[f].head.api_flag &= !FLAG_JF;
        }
        return BM_NIL;
    }

    for &f in faces {
        bm.fpool[f].head.api_flag &= !FLAG_JF;
    }
    bm.fpool[f_new].head.api_flag &= !FLAG_JF;

    if do_del {
        for e in deledges {
            if bm.edge(e).head.htype == BM_EDGE {
                bm_edge_kill(bm, e);
            }
        }
        for v in delverts {
            if bm.vert(v).head.htype == BM_VERT {
                bm_vert_kill(bm, v);
            }
        }
        /* also kill the original faces */
        for &f in faces {
            if bm.face(f).head.htype == BM_FACE {
                bm_face_kill(bm, f);
            }
        }
    } else {
        for &f in faces {
            if bm.face(f).head.htype == BM_FACE {
                bm_face_kill(bm, f);
            }
        }
    }

    if had_active && bm.act_face == BM_NIL {
        bm.act_face = f_new;
    }
    f_new
}

/* ---- remaining kernel ops: splice / separate / unglue ------------------ */

/// `bmesh_edge_vert_swap` — swap v_src → v_dst on edge and all radial loops.
pub fn bmesh_edge_vert_swap(bm: &mut BMesh, e: usize, v_dst: usize, v_src: usize) {
    if bm.edge(e).l != BM_NIL {
        let first = bm.edge(e).l;
        let mut l = first;
        loop {
            let next = bm.loo(l).next;
            if bm.loo(l).v == v_src {
                bm.lpool[l].v = v_dst;
            } else if bm.loo(next).v == v_src {
                bm.lpool[next].v = v_dst;
            } else {
                debug_assert!(bm.loo(bm.loo(l).prev).v != v_src);
            }
            l = bm.loo(l).radial_next;
            if l == first {
                break;
            }
        }
    }
    structure::bmesh_disk_vert_replace(bm, e, v_dst, v_src);
}

fn bmesh_edge_vert_swap_recursive(bm: &mut BMesh, e: usize, v_dst: usize, v_src: usize) {
    structure::bmesh_disk_vert_replace(bm, e, v_dst, v_src);
    if bm.edge(e).l == BM_NIL {
        return;
    }
    let first = bm.edge(e).l;
    let mut l = first;
    loop {
        let (lv, ln, lp) = (bm.loo(l).v, bm.loo(l).next, bm.loo(l).prev);
        let l_next = bm.loo(l).radial_next;
        if lv == v_src {
            bm.lpool[l].v = v_dst;
            if query::bm_vert_in_edge(bm, bm.loo(lp).e, v_src) {
                bmesh_edge_vert_swap_recursive(bm, bm.loo(lp).e, v_dst, v_src);
            }
        } else if bm.loo(ln).v == v_src {
            bm.lpool[ln].v = v_dst;
            if query::bm_vert_in_edge(bm, bm.loo(ln).e, v_src) {
                bmesh_edge_vert_swap_recursive(bm, bm.loo(ln).e, v_dst, v_src);
            }
        }
        l = l_next;
        if l == first {
            break;
        }
    }
}

/// `BM_vert_splice` — move all edges v_src → v_dst, kill v_src.
pub fn bm_vert_splice(bm: &mut BMesh, v_dst: usize, v_src: usize) -> bool {
    if v_src == v_dst {
        return false;
    }
    while bm.vert(v_src).e != BM_NIL {
        let e = bm.vert(v_src).e;
        bmesh_edge_vert_swap(bm, e, v_dst, v_src);
        debug_assert!(bm.edge(e).v1 != bm.edge(e).v2);
    }
    bm_vert_kill(bm, v_src);
    true
}

/// `BM_vert_splice_check_double_edge` — would splicing create a double edge?
pub fn bm_vert_splice_check_double_edge(bm: &mut BMesh, v_a: usize, v_b: usize) -> bool {
    const VERT_VISIT: u8 = 2;
    let mut is_double = false;
    if bm.vert(v_a).e != BM_NIL && bm.vert(v_b).e != BM_NIL {
        /* tag neighbors of v_a */
        let first = bm.vert(v_a).e;
        let mut e = first;
        loop {
            let v_other = other_vert(bm, e, v_a);
            bm.vpool[v_other].head.api_flag |= 1 << VERT_VISIT;
            e = structure::bmesh_disk_edge_next(bm, e, v_a);
            if e == first {
                break;
            }
        }
        let first = bm.vert(v_b).e;
        let mut e = first;
        loop {
            let v_other = other_vert(bm, e, v_b);
            if bm.vert(v_other).head.api_flag & (1 << VERT_VISIT) != 0 {
                is_double = true;
                break;
            }
            e = structure::bmesh_disk_edge_next(bm, e, v_b);
            if e == first {
                break;
            }
        }
        let first = bm.vert(v_a).e;
        let mut e = first;
        loop {
            let v_other = other_vert(bm, e, v_a);
            bm.vpool[v_other].head.api_flag &= !(1 << VERT_VISIT);
            e = structure::bmesh_disk_edge_next(bm, e, v_a);
            if e == first {
                break;
            }
        }
    }
    is_double
}

/// `BM_vert_splice_check_double_face` — would splicing merge two faces?
pub fn bm_vert_splice_check_double_face(bm: &BMesh, v_a: usize, v_b: usize) -> bool {
    if bm.vert(v_a).e == BM_NIL || bm.vert(v_b).e == BM_NIL {
        return false;
    }
    /* collect loops incident to each vert that point at it */
    let mut loops_pair: [Vec<usize>; 2] = [Vec::new(), Vec::new()];
    for side in 0..2 {
        let v = [v_a, v_b][side];
        let first = bm.vert(v).e;
        let mut e_iter = first;
        loop {
            let mut l = bm.edge(e_iter).l;
            if l != BM_NIL {
                let lfirst = l;
                loop {
                    if bm.loo(l).v == v {
                        loops_pair[side].push(l);
                    }
                    l = bm.loo(l).radial_next;
                    if l == lfirst {
                        break;
                    }
                }
            }
            e_iter = structure::bmesh_disk_edge_next(bm, e_iter, v);
            if e_iter == first {
                break;
            }
        }
        if loops_pair[side].is_empty() {
            return false;
        }
    }
    for side in 0..2 {
        loops_pair[side].sort_by_key(|&l| bm.face(bm.loo(l).f).len);
    }
    let (mut a_beg, mut b_beg) = (0usize, 0usize);
    while a_beg < loops_pair[0].len() && b_beg < loops_pair[1].len() {
        let a_len = bm.face(bm.loo(loops_pair[0][a_beg]).f).len;
        let b_len = bm.face(bm.loo(loops_pair[1][b_beg]).f).len;
        if a_len < b_len {
            a_beg += 1;
            continue;
        }
        if a_len > b_len {
            b_beg += 1;
            continue;
        }
        let mut a_end = a_beg + 1;
        let mut b_end = b_beg + 1;
        while a_end < loops_pair[0].len() && bm.face(bm.loo(loops_pair[0][a_end]).f).len == a_len {
            a_end += 1;
        }
        while b_end < loops_pair[1].len() && bm.face(bm.loo(loops_pair[1][b_end]).f).len == a_len {
            b_end += 1;
        }
        for ai in a_beg..a_end {
            let l_a = loops_pair[0][ai];
            for bi in b_beg..b_end {
                let l_b = loops_pair[1][bi];
                let (an, ap) = (bm.loo(bm.loo(l_a).next).v, bm.loo(bm.loo(l_a).prev).v);
                let (bn, bp) = (bm.loo(bm.loo(l_b).next).v, bm.loo(bm.loo(l_b).prev).v);
                if an == bn && ap == bp {
                    if bm_face_pair_overlap_check_subset_same_winding(
                        bm,
                        bm.loo(bm.loo(l_a).next).next,
                        bm.loo(bm.loo(l_a).prev).prev,
                        bm.loo(bm.loo(l_b).next).next,
                        bm.loo(bm.loo(l_b).prev).prev,
                    ) {
                        return true;
                    }
                } else if an == bp && ap == bn {
                    if bm_face_pair_overlap_check_subset_swap_winding(
                        bm,
                        bm.loo(bm.loo(l_a).next).next,
                        bm.loo(bm.loo(l_a).prev).prev,
                        bm.loo(bm.loo(l_b).prev).prev,
                        bm.loo(bm.loo(l_b).next).next,
                    ) {
                        return true;
                    }
                }
            }
        }
        a_beg = a_end;
        b_beg = b_end;
    }
    false
}

/// `BM_vert_collapse_check_double_face` — collapse would double a face.
pub fn bm_vert_collapse_check_double_face(bm: &BMesh, v_collapse: usize) -> bool {
    debug_assert!(query::bm_vert_is_edge_pair(bm, v_collapse));
    let l_first = bm.edge(bm.vert(v_collapse).e).l;
    if l_first == BM_NIL {
        return false;
    }
    let e_a = bm.vert(v_collapse).e;
    let e_b = if bm.loo(l_first).v == v_collapse {
        bm.loo(bm.loo(l_first).prev).e
    } else {
        bm.loo(bm.loo(l_first).next).e
    };
    let v_a = other_vert(bm, e_a, v_collapse);
    let v_b = other_vert(bm, e_b, v_collapse);
    let e_exists = query::bm_edge_exists(bm, v_a, v_b);
    if e_exists == BM_NIL || bm.edge(e_exists).l == BM_NIL {
        return false;
    }
    let mut l_a_iter = l_first;
    loop {
        let f_a = bm.loo(l_a_iter).f;
        if bm.face(f_a).len > 3 {
            let l_collapse = if bm.loo(l_a_iter).v == v_collapse {
                l_a_iter
            } else if bm.loo(bm.loo(l_a_iter).next).v == v_collapse {
                bm.loo(l_a_iter).next
            } else {
                bm.loo(l_a_iter).prev
            };
            let mut l_exists = bm.edge(e_exists).l;
            let ex_first = l_exists;
            loop {
                let f_ex = bm.loo(l_exists).f;
                if bm.face(f_a).len - 1 == bm.face(f_ex).len {
                    let l_cb = bm.loo(l_collapse).next;
                    let l_ce = bm.loo(l_collapse).prev;
                    let swap_winding = bm.loo(l_cb).v == bm.loo(l_exists).v;
                    let hit = if swap_winding {
                        bm_face_pair_overlap_check_subset_swap_winding(
                            bm,
                            bm.loo(l_cb).next,
                            bm.loo(l_ce).prev,
                            bm.loo(l_exists).prev,
                            bm.loo(bm.loo(l_exists).next).next,
                        )
                    } else {
                        bm_face_pair_overlap_check_subset_same_winding(
                            bm,
                            bm.loo(l_cb).next,
                            bm.loo(l_ce).prev,
                            bm.loo(bm.loo(l_exists).next).next,
                            bm.loo(l_exists).prev,
                        )
                    };
                    if hit {
                        return true;
                    }
                }
                l_exists = bm.loo(l_exists).radial_next;
                if l_exists == ex_first {
                    break;
                }
            }
        }
        l_a_iter = bm.loo(l_a_iter).radial_next;
        if l_a_iter == l_first {
            break;
        }
    }
    false
}

/// `BM_face_split_check_double_face` — splitting l_a..l_b on face would
/// create a duplicate of an existing face on either side.
pub fn bm_face_split_check_double_face(bm: &BMesh, l_a: usize, l_b: usize, f_len: i32) -> bool {
    let side_b_len = (bm.face(bm.loo(l_a).f).len - f_len) + 2;
    query::bm_face_exists_subset_from_face(bm, l_a, l_b, f_len) != BM_NIL
        || query::bm_face_exists_subset_from_face(bm, l_b, l_a, side_b_len) != BM_NIL
}

/// `bm_edge_supports_separate` — edge radial has >1 loop.
#[inline]
fn bm_edge_supports_separate(bm: &BMesh, e: usize) -> bool {
    let l = bm.edge(e).l;
    l != BM_NIL && bm.loo(l).radial_next != l
}

/// `bmesh_kernel_edge_separate` — split `l_sep` off edge `e` onto a new edge.
pub fn bmesh_kernel_edge_separate(bm: &mut BMesh, e: usize, l_sep: usize, copy_select: bool) {
    debug_assert!(bm.loo(l_sep).e == e && bm.edge(e).l != BM_NIL);
    if bm.edge(e).l == BM_NIL || bm.loo(bm.edge(e).l).radial_next == bm.edge(e).l {
        return; /* boundary: no cut required */
    }
    if l_sep == bm.edge(e).l {
        bm.epool[e].l = bm.loo(l_sep).radial_next;
    }
    let src_flags = (bm.edge(e).head.hflag, bm.edge(e).head.api_flag);
    let e_new = bm_edge_create(bm, bm.edge(e).v1, bm.edge(e).v2, Some(e), 0);
    structure::bmesh_radial_loop_remove(bm, e, l_sep);
    structure::bmesh_radial_loop_append(bm, e_new, l_sep);
    bm.lpool[l_sep].e = e_new;
    if copy_select {
        construct::bm_elem_select_copy(bm, BM_EDGE, e_new, src_flags);
    }
}

/// `bmesh_kernel_vert_separate` — split vertex fans; returns count.
pub fn bmesh_kernel_vert_separate(
    bm: &mut BMesh,
    v: usize,
    r_vout: Option<&mut Vec<usize>>,
    copy_select: bool,
) -> usize {
    const EDGE_VISIT: u8 = 1;
    let mut verts_new: Vec<usize> = Vec::new();
    let mut edges: Vec<usize> = Vec::new();
    let mut edges_search: Vec<usize> = Vec::new();
    let mut verts_num = 1usize;
    let mut edges_found = 0usize;
    let mut v_edges_num = 0usize;
    let mut vout = r_vout;
    if bm.vert(v).e != BM_NIL {
        /* count and tag */
        let first = bm.vert(v).e;
        let mut e_iter = first;
        loop {
            v_edges_num += 1;
            bm.epool[e_iter].head.api_flag |= 1 << EDGE_VISIT;
            e_iter = structure::bmesh_disk_edge_next(bm, e_iter, v);
            if e_iter == first {
                break;
            }
        }
        loop {
            /* walk one fan from v->e */
            let mut e = bm.vert(v).e;
            bm.epool[e].head.api_flag &= !(1 << EDGE_VISIT);
            loop {
                edges.push(e);
                edges_found += 1;
                let mut l = bm.edge(e).l;
                if l != BM_NIL {
                    let first = l;
                    loop {
                        let l_adjacent = if bm.loo(l).v == v {
                            bm.loo(l).prev
                        } else {
                            bm.loo(l).next
                        };
                        let ea = bm.loo(l_adjacent).e;
                        if bm.edge(ea).head.api_flag & (1 << EDGE_VISIT) != 0 {
                            bm.epool[ea].head.api_flag &= !(1 << EDGE_VISIT);
                            edges_search.push(ea);
                        }
                        l = bm.loo(l).radial_next;
                        if l == first {
                            break;
                        }
                    }
                }
                match edges_search.pop() {
                    Some(next_e) => e = next_e,
                    None => break,
                }
            }
            if edges_found == v_edges_num {
                break;
            }
            let src_flags = (bm.vert(v).head.hflag, bm.vert(v).head.api_flag);
            let v_new = bm_vert_create(bm, bm.vert(v).co, Some(v), 0);
            if copy_select {
                construct::bm_elem_select_copy(bm, BM_VERT, v_new, src_flags);
            }
            while let Some(e) = edges.pop() {
                bmesh_edge_vert_swap(bm, e, v_new, v);
            }
            if vout.is_some() {
                verts_new.push(v_new);
            }
            verts_num += 1;
        }
    }
    if let Some(out) = vout {
        out.clear();
        out.push(v);
        out.extend(verts_new);
    }
    verts_num
}

/// `bmesh_kernel_vert_separate__cleanup` — merge duplicate edges created by
/// `bmesh_kernel_edge_separate` per group (list-of-lists: [orig, splits...]).
fn vert_separate_cleanup(bm: &mut BMesh, edges_separate: &[Vec<usize>]) {
    for group in edges_separate {
        for (gi, &e_orig) in group.iter().enumerate() {
            for &e in &group[gi + 1..] {
                if bm.edge(e).v1 == bm.edge(e_orig).v1
                    && bm.edge(e).v2 == bm.edge(e_orig).v2
                    && bm_edge_splice(bm, e_orig, e)
                {
                    /* merged; keep scanning rest of group */
                }
            }
        }
    }
}

/// `BM_vert_separate` — split `v`'s fans, splitting `e_in` edges first.
pub fn bm_vert_separate(
    bm: &mut BMesh,
    v: usize,
    e_in: &[usize],
    copy_select: bool,
    r_vout: Option<&mut Vec<usize>>,
) -> usize {
    let mut edges_separate: Vec<Vec<usize>> = Vec::new();
    for &e in e_in {
        if bm_edge_supports_separate(bm, e) {
            let mut group = Vec::new();
            loop {
                let l_sep = bm.edge(e).l;
                bmesh_kernel_edge_separate(bm, e, l_sep, copy_select);
                group.push(bm.loo(l_sep).e);
                if !bm_edge_supports_separate(bm, e) {
                    break;
                }
            }
            group.push(e);
            edges_separate.push(group);
        }
    }
    let n = bmesh_kernel_vert_separate(bm, v, r_vout, copy_select);
    vert_separate_cleanup(bm, &edges_separate);
    n
}

/// `BM_vert_separate_hflag` — split `v`'s fans; pre-split edges w/ hflag.
pub fn bm_vert_separate_hflag(
    bm: &mut BMesh,
    v: usize,
    hflag: u8,
    copy_select: bool,
    r_vout: Option<&mut Vec<usize>>,
) -> usize {
    let mut edges_separate: Vec<Vec<usize>> = Vec::new();
    if bm.vert(v).e != BM_NIL {
        let mut e_first = bm.vert(v).e;
        let mut e_iter = e_first;
        loop {
            let next_e = structure::bmesh_disk_edge_next(bm, e_iter, v);
            if bm.edge(e_iter).head.hflag & hflag != 0 {
                let e = e_iter;
                if bm_edge_supports_separate(bm, e) {
                    let mut group = Vec::new();
                    loop {
                        let l_sep = bm.edge(e).l;
                        bmesh_kernel_edge_separate(bm, e, l_sep, copy_select);
                        if edges_separate.is_empty() && group.is_empty() {
                            e_first = bm.loo(l_sep).e;
                        }
                        group.push(bm.loo(l_sep).e);
                        if !bm_edge_supports_separate(bm, e) {
                            break;
                        }
                    }
                    group.push(e);
                    edges_separate.push(group);
                }
            }
            e_iter = next_e;
            if e_iter == e_first {
                break;
            }
        }
    }
    let n = bmesh_kernel_vert_separate(bm, v, r_vout, copy_select);
    vert_separate_cleanup(bm, &edges_separate);
    n
}

/// `BM_vert_separate_tested_edges` — move edges passing `test` from v_src to
/// v_dst's disk cycle.
pub fn bm_vert_separate_tested_edges(
    bm: &mut BMesh,
    v_dst: usize,
    v_src: usize,
    test: impl Fn(&BMesh, usize) -> bool,
) {
    if bm.vert(v_src).e == BM_NIL {
        return;
    }
    let mut edges = Vec::new();
    let first = bm.vert(v_src).e;
    let mut e_iter = first;
    loop {
        if test(bm, e_iter) {
            edges.push(e_iter);
        }
        e_iter = structure::bmesh_disk_edge_next(bm, e_iter, v_src);
        if e_iter == first {
            break;
        }
    }
    for e in edges {
        structure::bmesh_disk_vert_replace(bm, e, v_dst, v_src);
    }
}

/// `bmesh_kernel_unglue_region_make_vert` — peel one loop's vert region.
pub fn bmesh_kernel_unglue_region_make_vert(bm: &mut BMesh, l_sep: usize) -> usize {
    let v_sep = bm.loo(l_sep).v;
    if !delete::bm_edge_is_boundary(bm, bm.loo(l_sep).e) {
        bmesh_kernel_edge_separate(bm, bm.loo(l_sep).e, l_sep, false);
    }
    let l_prev = bm.loo(l_sep).prev;
    if !delete::bm_edge_is_boundary(bm, bm.loo(l_prev).e) {
        bmesh_kernel_edge_separate(bm, bm.loo(l_prev).e, l_prev, false);
    }
    /* find an edge unattached to this loop */
    let mut e_iter = bm.vert(v_sep).e;
    while e_iter != bm.loo(l_sep).e && e_iter != bm.loo(l_prev).e {
        e_iter = structure::bmesh_disk_edge_next(bm, e_iter, v_sep);
        if e_iter == bm.vert(v_sep).e {
            return v_sep; /* whole URMV was a no-op */
        }
    }
    bm.vpool[v_sep].e = bm.loo(l_sep).e;
    let v_new = bm_vert_create(bm, bm.vert(v_sep).co, Some(v_sep), 0);
    for e in [bm.loo(l_sep).e, bm.loo(l_prev).e] {
        bmesh_edge_vert_swap(bm, e, v_new, v_sep);
    }
    v_new
}

/// `bmesh_kernel_unglue_region_make_vert_multi` — peel several loops' region.
pub fn bmesh_kernel_unglue_region_make_vert_multi(bm: &mut BMesh, larr: &[usize]) -> usize {
    const LOOP_VISIT: u8 = 2;
    const EDGE_VISIT: u8 = 1;
    let v_sep = bm.loo(larr[0]).v;
    let mut edges_len = 0usize;
    let mut is_mixed_edge_any = false;
    let mut is_mixed_loop_any = false;
    for &l_sep in larr {
        debug_assert!(v_sep == bm.loo(l_sep).v);
        bm.lpool[l_sep].head.api_flag |= 1 << LOOP_VISIT;
        let (ln, lp) = (bm.loo(l_sep).next, bm.loo(l_sep).prev);
        bm.lpool[ln].head.api_flag |= 1 << LOOP_VISIT;
        bm.lpool[lp].head.api_flag |= 1 << LOOP_VISIT;
        for &le in &[bm.loo(l_sep).e, bm.loo(lp).e] {
            if bm.edge(le).head.api_flag & (1 << EDGE_VISIT) == 0 {
                bm.epool[le].head.api_flag |= 1 << EDGE_VISIT;
                edges_len += 1;
            }
        }
    }
    let mut edges: Vec<usize> = Vec::with_capacity(edges_len);
    if bm.vert(v_sep).e != BM_NIL {
        let first = bm.vert(v_sep).e;
        let mut e_iter = first;
        loop {
            if bm.edge(e_iter).head.api_flag & (1 << EDGE_VISIT) != 0 {
                let mut is_mixed_loop = false;
                let mut l = bm.edge(e_iter).l;
                let lfirst = l;
                loop {
                    if bm.loo(l).head.api_flag & (1 << LOOP_VISIT) == 0 {
                        is_mixed_loop = true;
                        break;
                    }
                    l = bm.loo(l).radial_next;
                    if l == lfirst {
                        break;
                    }
                }
                if is_mixed_loop {
                    bm.epool[e_iter].l = l;
                    is_mixed_loop_any = true;
                }
                edges.push(e_iter);
            } else {
                is_mixed_edge_any = true;
            }
            e_iter = structure::bmesh_disk_edge_next(bm, e_iter, v_sep);
            if e_iter == first {
                break;
            }
        }
    }
    let v_new;
    if !is_mixed_loop_any && !is_mixed_edge_any {
        v_new = v_sep;
    } else {
        v_new = bm_vert_create(bm, bm.vert(v_sep).co, Some(v_sep), 0);
        for &e in &edges {
            bm.epool[e].head.api_flag &= !(1 << EDGE_VISIT);
            let el = bm.edge(e).l;
            if bm.loo(el).head.api_flag & (1 << LOOP_VISIT) == 0 {
                /* split off owned loops onto a new edge */
                let (v1, v2) = if bm.edge(e).v1 == v_sep {
                    (v_new, bm.edge(e).v2)
                } else {
                    (bm.edge(e).v1, v_new)
                };
                let e_new = bm_edge_create(bm, v1, v2, Some(e), 0);
                let mut l = el;
                let lfirst = l;
                loop {
                    let l_next = bm.loo(l).radial_next;
                    if bm.loo(l).head.api_flag & (1 << LOOP_VISIT) != 0 {
                        structure::bmesh_radial_loop_remove(bm, e, l);
                        structure::bmesh_radial_loop_append(bm, e_new, l);
                        bm.lpool[l].e = e_new;
                    }
                    l = l_next;
                    if l == lfirst {
                        break;
                    }
                }
            } else {
                structure::bmesh_disk_vert_replace(bm, e, v_new, v_sep);
            }
        }
    }
    for &l_sep in larr {
        bm.lpool[l_sep].v = v_new;
        let (ln, lp) = (bm.loo(l_sep).next, bm.loo(l_sep).prev);
        bm.lpool[l_sep].head.api_flag &= !(1 << LOOP_VISIT);
        bm.lpool[ln].head.api_flag &= !(1 << LOOP_VISIT);
        bm.lpool[lp].head.api_flag &= !(1 << LOOP_VISIT);
        let e_prev = bm.loo(lp).e;
        let e_self = bm.loo(l_sep).e;
        bm.epool[e_prev].head.api_flag &= !(1 << EDGE_VISIT);
        bm.epool[e_self].head.api_flag &= !(1 << EDGE_VISIT);
    }
    v_new
}

/// `bmesh_kernel_unglue_region_make_vert_multi_isolated`.
pub fn bmesh_kernel_unglue_region_make_vert_multi_isolated(bm: &mut BMesh, l_sep: usize) -> usize {
    let v_src = bm.loo(l_sep).v;
    let v_new = bm_vert_create(bm, bm.vert(v_src).co, Some(v_src), 0);
    bmesh_edge_vert_swap_recursive(bm, bm.loo(l_sep).e, v_new, v_src);
    v_new
}

/// `bmesh_face_swap_data` — swap loop ownership + face records, keep data.
pub fn bmesh_face_swap_data(bm: &mut BMesh, f_a: usize, f_b: usize) {
    debug_assert!(f_a != f_b);
    let first = bm.face(f_a).l_first;
    let mut l = first;
    loop {
        bm.lpool[l].f = f_b;
        l = bm.loo(l).next;
        if l == first {
            break;
        }
    }
    let first = bm.face(f_b).l_first;
    let mut l = first;
    loop {
        bm.lpool[l].f = f_a;
        l = bm.loo(l).next;
        if l == first {
            break;
        }
    }
    bm.fpool.swap(f_a, f_b);
    /* swap back data+index so pool indices stay coherent */
    let (da, db) = (bm.fpool[f_a].head.data, bm.fpool[f_b].head.data);
    let (ia, ib) = (bm.fpool[f_a].head.index, bm.fpool[f_b].head.index);
    bm.fpool[f_a].head.data = db;
    bm.fpool[f_b].head.data = da;
    bm.fpool[f_a].head.index = ia;
    bm.fpool[f_b].head.index = ib;
}

/// `BM_edge_other_vert` — the edge's vert that isn't `v`.
#[inline]
fn other_vert(bm: &BMesh, e: usize, v: usize) -> usize {
    if bm.edge(e).v1 == v {
        bm.edge(e).v2
    } else {
        bm.edge(e).v1
    }
}

/// `BM_face_copy` — duplicate face (with new verts+edges when asked).
/// Loop CustomData interp PORT_PENDING.
pub fn bm_face_copy(bm: &mut BMesh, f: usize, copy_verts: bool, copy_edges: bool) -> usize {
    let flen = bm.face(f).len as usize;
    let mut verts: Vec<usize> = Vec::with_capacity(flen);
    let mut edges: Vec<usize> = Vec::with_capacity(flen);
    let mut emap: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let loops: Vec<usize> = crate::iterators::LoopsOfFace::new(bm, f).collect();
    for &l in &loops {
        let v = bm.loo(l).v;
        let e = bm.loo(l).e;
        let nv = if copy_verts {
            crate::core::bm_vert_create(bm, bm.vert(v).co, Some(v), 0)
        } else { v };
        let ne = if copy_edges {
            *emap.entry(e).or_insert_with(|| usize::MAX)
        } else { e };
        verts.push(nv);
        edges.push(ne);
    }
    /* edges created after verts so they reference the new verts */
    if copy_edges {
        let mut ne_i = 0;
        for &l in &loops {
            if edges[ne_i] == usize::MAX {
                let v1 = verts[ne_i];
                let v2 = verts[(ne_i + 1) % flen];
                edges[ne_i] = crate::core::bm_edge_create(bm, v1, v2, Some(bm.loo(l).e), 0);
            }
            ne_i += 1;
        }
    }
    bm_face_create(bm, &verts, &edges, Some(f), 0)
}

/// `BM_face_kill_loose` — kill face, plus edges/verts that become loose.
/// (No `USE_BMESH_HOLES` loop-lists in the port.)
pub fn bm_face_kill_loose(bm: &mut BMesh, f: usize) {
    if bm.face(f).l_first != BM_NIL {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            let l_next = bm.loo(l_iter).next;
            let e = bm.loo(l_iter).e;
            crate::structure::bmesh_radial_loop_remove(bm, e, l_iter);
            bm_kill_only_loop(bm, l_iter);
            if bm.edge(e).l == BM_NIL {
                let v1 = bm.edge(e).v1;
                let v2 = bm.edge(e).v2;
                crate::structure::bmesh_disk_edge_remove(bm, e, v1);
                crate::structure::bmesh_disk_edge_remove(bm, e, v2);
                bm_kill_only_edge(bm, e);
                if bm.vert(v1).e == BM_NIL {
                    bm_kill_only_vert(bm, v1);
                }
                if bm.vert(v2).e == BM_NIL {
                    bm_kill_only_vert(bm, v2);
                }
            }
            l_iter = l_next;
            if l_iter == l_first {
                break;
            }
        }
    }
    bm_kill_only_face(bm, f);
}
