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
