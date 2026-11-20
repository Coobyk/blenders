//! Port of `bmesh_mods.cc` — higher-level topology modifiers built on the
//! kernel ops (split, collapse, rotate, dissolve).
//!
//! Multires (`CD_MDISPS`) interpolation and `BM_loop_interp_from_face`
//! CustomData copies are PORT_PENDING (CustomData unported).

use crate::core::*;
use crate::query::*;
use crate::structure::*;
use crate::{BMesh, BMFace, BMEdge, BM_NIL, BM_CREATE_NO_DOUBLE};
use blenders_math::vector::Vec3;

/// `BM_vert_dissolve` — remove vert, re-polygonizing its fan.
pub fn bm_vert_dissolve(bm: &mut BMesh, v: usize) -> bool {
    let len = bm_vert_edge_count_at_most(bm, v, 3);
    if len == 1 {
        bm_vert_kill(bm, v);
        return true;
    }
    if !bm_vert_is_manifold(bm, v) {
        if bm.vert(v).e == BM_NIL {
            bm_vert_kill(bm, v);
            return true;
        }
        if bm.edge(bm.vert(v).e).l == BM_NIL {
            if len == 2 {
                return bm_vert_collapse_edge(bm, bm.vert(v).e, v, true, true, true) != BM_NIL;
            }
            return false;
        }
        return false;
    }
    if len == 2 && bm_vert_face_count(bm, v) == 1 {
        /* boundary vertex on a face */
        return bm_vert_collapse_edge(bm, bm.vert(v).e, v, true, true, true) != BM_NIL;
    }
    bm_disk_dissolve(bm, v)
}

/// `BM_disk_dissolve`.
pub fn bm_disk_dissolve(bm: &mut BMesh, v: usize) -> bool {
    if !bm_vert_is_manifold(bm, v) {
        return false;
    }
    let mut keepedge = BM_NIL;
    let mut baseedge = BM_NIL;
    let mut len = 0;
    if bm.vert(v).e != BM_NIL {
        let mut e = bm.vert(v).e;
        loop {
            e = bmesh_disk_edge_next(bm, e, v);
            if !bm_edge_share_face_check(bm, e, bm.vert(v).e) {
                keepedge = e;
                baseedge = bm.vert(v).e;
                break;
            }
            len += 1;
            if e == bm.vert(v).e || e == BM_NIL {
                break;
            }
        }
    }

    if keepedge == BM_NIL && len == 3 {
        /* join two faces over e, then collapse */
        let e = bm.vert(v).e;
        if e == BM_NIL || bm.edge(e).l == BM_NIL {
            return false;
        }
        let l = bm.edge(e).l;
        let l_rad = bm.loo(l).radial_next;
        let mut f_double = BM_NIL;
        if bm_faces_join_pair(bm, l, l_rad, true, Some(&mut f_double)) == BM_NIL {
            return false;
        }
        if bm_vert_collapse_faces(bm, bm.vert(v).e, v, 1.0, true, false, true, true) == BM_NIL {
            return false;
        }
        return true;
    }
    if keepedge == BM_NIL && len == 2 {
        let e = bm_vert_collapse_faces(bm, bm.vert(v).e, v, 1.0, true, true, true, true);
        if e == BM_NIL {
            return false;
        }
        if bm.edge(e).l != BM_NIL && bm.edge(e).l != bm.loo(bm.edge(e).l).radial_next {
            let l = bm.edge(e).l;
            let l_rad = bm.loo(l).radial_next;
            if bm_faces_join_pair(bm, l, l_rad, true, None) == BM_NIL {
                return false;
            }
        }
        return true;
    }
    if keepedge != BM_NIL {
        let mut done = false;
        while !done {
            done = true;
            let mut e = bm.vert(v).e;
            loop {
                let mut f = BM_NIL;
                if bm_edge_is_manifold(bm, e) && e != baseedge && e != keepedge {
                    let l = bm.edge(e).l;
                    let l_rad = bm.loo(l).radial_next;
                    f = bm_faces_join_pair(bm, l, l_rad, true, None);
                    if f == BM_NIL {
                        return false;
                    }
                }
                if f != BM_NIL {
                    done = false;
                    break;
                }
                let e_next = bmesh_disk_edge_next(bm, e, v);
                if e_next == bm.vert(v).e || e_next == BM_NIL {
                    break;
                }
                e = e_next;
            }
        }
        let boundary = {
            let l = bm.edge(baseedge).l;
            l == BM_NIL || bm.loo(l).radial_next == l
        };
        let e = bm_vert_collapse_faces(bm, baseedge, v, 1.0, true, !boundary, true, true);
        if e == BM_NIL {
            return false;
        }
        if bm.edge(e).l != BM_NIL {
            let l = bm.edge(e).l;
            if l != bm.loo(l).radial_next {
                let l_rad = bm.loo(l).radial_next;
                if bm_faces_join_pair(bm, l, l_rad, true, None) == BM_NIL {
                    return false;
                }
            }
        }
    }
    true
}

/// `BM_faces_join_pair`.
pub fn bm_faces_join_pair(
    bm: &mut BMesh,
    l_a: usize,
    l_b: usize,
    do_del: bool,
    r_double: Option<&mut usize>,
) -> usize {
    debug_assert_ne!(l_a, l_b);
    debug_assert_eq!(bm.loo(l_a).e, bm.loo(l_b).e);
    if bm.loo(l_a).v == bm.loo(l_b).v {
        bmesh_kernel_loop_reverse(bm, bm.loo(l_b).f);
    }
    let faces = [bm.loo(l_a).f, bm.loo(l_b).f];
    bm_faces_join(bm, &faces, do_del, r_double)
}

/// `BM_face_split` — split face along the new edge between l_a and l_b.
pub fn bm_face_split(
    bm: &mut BMesh,
    f: usize,
    l_a: usize,
    l_b: usize,
    r_l: Option<&mut usize>,
    no_double: bool,
) -> usize {
    debug_assert_ne!(l_a, l_b);
    if bm_loop_is_adjacent(bm, l_a, l_b) || bm.loo(l_a).f != f || bm.loo(l_b).f != f {
        if let Some(r) = r_l {
            *r = BM_NIL;
        }
        return BM_NIL;
    }
    bmesh_kernel_split_face_make_edge(bm, f, l_a, l_b, r_l, no_double)
}

/// `BM_loop_is_adjacent` — do the loops share an edge in the face cycle?
pub fn bm_loop_is_adjacent(bm: &BMesh, l_a: usize, l_b: usize) -> bool {
    bm.loo(l_a).next == l_b
        || bm.loo(l_a).prev == l_b
        || bm.loo(l_b).next == l_a
        || bm.loo(l_b).prev == l_a
}

/// `BM_face_split_n` — split and subdivide the new edge into `n` segments
/// with the given coords. (`BM_loop_interp_from_face` PORT_PENDING.)
pub fn bm_face_split_n(
    bm: &mut BMesh,
    f: usize,
    l_a: usize,
    l_b: usize,
    cos: &[Vec3],
    r_l: Option<&mut usize>,
) -> usize {
    let n = cos.len();
    if (n == 0 && bm_loop_is_adjacent(bm, l_a, l_b)) || bm.loo(l_a).f != bm.loo(l_b).f {
        if let Some(r) = r_l {
            *r = BM_NIL;
        }
        return BM_NIL;
    }
    let v_b = bm.loo(l_b).v;
    let mut l_new = BM_NIL;
    let f_new = bmesh_kernel_split_face_make_edge(bm, f, l_a, l_b, Some(&mut l_new), false);
    if f_new != BM_NIL {
        let mut e = bm.loo(l_new).e;
        for i in 0..n {
            let mut e_new = BM_NIL;
            let v_new = bmesh_kernel_split_edge_make_vert(bm, v_b, e, Some(&mut e_new));
            bm.vpool[v_new].co = cos[i];
            e = e_new;
        }
    }
    if let Some(r) = r_l {
        *r = l_new;
    }
    f_new
}

/// `BM_vert_collapse_faces`.
pub fn bm_vert_collapse_faces(
    bm: &mut BMesh,
    e_kill: usize,
    v_kill: usize,
    _fac: f32,
    do_del: bool,
    join_faces: bool,
    kill_degenerate_faces: bool,
    kill_duplicate_faces: bool,
) -> usize {
    let mut e_new = BM_NIL;
    let tv = if bm.edge(e_kill).v1 == v_kill {
        bm.edge(e_kill).v2
    } else {
        bm.edge(e_kill).v1
    };
    debug_assert!(bmesh_disk_count(bm, v_kill) <= 2);
    let e2 = bmesh_disk_edge_next(bm, e_kill, v_kill);
    let tv2 = if bm.edge(e2).v1 == v_kill {
        bm.edge(e2).v2
    } else {
        bm.edge(e2).v1
    };

    if join_faces {
        let faces: Vec<usize> = crate::iterators::FacesOfVert::new(bm, v_kill).collect();
        if faces.len() >= 2 {
            let mut f_double = BM_NIL;
            let f2 = bm_faces_join(bm, &faces, true, Some(&mut f_double));
            if f2 != BM_NIL {
                if kill_duplicate_faces && f_double != BM_NIL {
                    if bm.face(f_double).head.htype == crate::BM_FACE {
                        bm_face_kill(bm, f_double);
                    }
                }
                let l_a = bm_face_vert_share_loop(bm, f2, tv);
                let l_b = bm_face_vert_share_loop(bm, f2, tv2);
                if l_a != BM_NIL && l_b != BM_NIL {
                    let mut l_new = BM_NIL;
                    if bm_face_split(bm, f2, l_a, l_b, Some(&mut l_new), false) != BM_NIL {
                        e_new = bm.loo(l_new).e;
                    }
                }
            }
        }
    } else {
        e_new = bmesh_kernel_join_edge_kill_vert(
            bm, e_kill, v_kill, do_del, true, kill_degenerate_faces, kill_duplicate_faces,
        );
    }
    e_new
}

/// `BM_vert_collapse_edge`.
pub fn bm_vert_collapse_edge(
    bm: &mut BMesh,
    e_kill: usize,
    v_kill: usize,
    do_del: bool,
    kill_degenerate_faces: bool,
    kill_duplicate_faces: bool,
) -> usize {
    bmesh_kernel_join_edge_kill_vert(
        bm, e_kill, v_kill, do_del, true, kill_degenerate_faces, kill_duplicate_faces,
    )
}

/// `BM_edge_collapse`.
pub fn bm_edge_collapse(
    bm: &mut BMesh,
    e_kill: usize,
    v_kill: usize,
    do_del: bool,
    kill_degenerate_faces: bool,
) -> usize {
    bmesh_kernel_join_vert_kill_edge(bm, e_kill, v_kill, do_del, true, kill_degenerate_faces)
}

/// `BM_edge_split` — split edge `e` at `v` with fac along v→v_other.
pub fn bm_edge_split(
    bm: &mut BMesh,
    e: usize,
    v: usize,
    r_e: Option<&mut usize>,
    fac: f32,
) -> usize {
    debug_assert!(crate::query::bm_vert_in_edge(bm, e, v));
    let v_other = if bm.edge(e).v1 == v {
        bm.edge(e).v2
    } else {
        bm.edge(e).v1
    };
    let mut e_new = BM_NIL;
    let v_new = bmesh_kernel_split_edge_make_vert(bm, v, e, Some(&mut e_new));
    if let Some(r) = r_e {
        *r = e_new;
    }
    bm.vpool[v_new].co = bm.vert(v).co + (bm.vert(v_other).co - bm.vert(v).co) * fac;
    bm.epool[e_new].head.hflag = bm.edge(e).head.hflag;
    v_new
}

/// `BM_edge_split_n` — subdivide edge into numcuts+1 segments.
pub fn bm_edge_split_n(bm: &mut BMesh, e: usize, numcuts: i32, r_varr: &mut [usize]) -> usize {
    let mut v_new = BM_NIL;
    for i in 0..numcuts {
        let percent = 1.0 / (numcuts + 1 - i) as f32;
        v_new = bm_edge_split(bm, e, bm.edge(e).v2, None, percent);
        if (numcuts - i - 1) < r_varr.len() as i32 {
            r_varr[(numcuts - i - 1) as usize] = v_new;
        }
    }
    v_new
}

/// `BM_edge_verts_swap`.
pub fn bm_edge_verts_swap(bm: &mut BMesh, e: usize) {
    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
    let dl = bm.edge(e).v1_disk_link;
    bm.epool[e].v1 = v2;
    bm.epool[e].v1_disk_link = bm.edge(e).v2_disk_link;
    bm.epool[e].v2 = v1;
    bm.epool[e].v2_disk_link = dl;
}

/// `BM_edge_calc_rotate` — the two loops to rotate between (ccw flag).
pub fn bm_edge_calc_rotate(bm: &BMesh, e: usize, ccw: bool) -> Option<(usize, usize)> {
    /* face orientation matters */
    let l = bm.edge(e).l;
    if l == BM_NIL {
        return None;
    }
    let l2 = bm.loo(l).radial_next;
    if l2 == l {
        return None;
    }
    /* swap so that rotating `ccw` direction is consistent */
    let (l1, l2) = if ccw { (l, l2) } else { (l2, l) };
    Some((l1, l2))
}

/// `BM_edge_rotate_check` — sanity check (2 faces + manifold edge).
pub fn bm_edge_rotate_check(bm: &BMesh, e: usize) -> bool {
    if bm.edge(e).l == BM_NIL {
        return false;
    }
    let l = bm.edge(e).l;
    let l2 = bm.loo(l).radial_next;
    if l2 == l {
        return false;
    }
    bm_edge_is_manifold(bm, e)
}

/// `BM_edge_rotate_check_degenerate` — rotation wouldn't create degenerate.
pub fn bm_edge_rotate_check_degenerate(bm: &BMesh, e: usize, l1: usize, l2: usize) -> bool {
    /* the new edge would run l1->prev->v → l2->prev->v; those verts must
     * differ and not be the current edge's verts */
    let v1 = bm.loo(bm.loo(l1).prev).v;
    let v2 = bm.loo(bm.loo(l2).prev).v;
    if v1 == v2 {
        return false;
    }
    let (e_v1, e_v2) = (bm.edge(e).v1, bm.edge(e).v2);
    !(v1 == e_v1 && v2 == e_v2) && !(v1 == e_v2 && v2 == e_v1)
}

/// `BM_edge_rotate_check_beauty` — rotate if it increases min corner angle.
pub fn bm_edge_rotate_check_beauty(bm: &BMesh, _e: usize, l1: usize, l2: usize) -> bool {
    /* corner angles before/after: rotate if min(angle1, angle2) improves */
    let a1 = bm_loop_calc_face_angle(bm, bm.loo(l1).prev);
    let a2 = bm_loop_calc_face_angle(bm, bm.loo(l2).prev);
    let a3 = bm_loop_calc_face_angle(bm, l1);
    let a4 = bm_loop_calc_face_angle(bm, l2);
    let min_before = a3.min(a4);
    let min_after = a1.min(a2);
    min_after > min_before
}

/// `BMEdgeRotate_CheckFlag`.
pub const BM_EDGE_ROTATE_CHECK_NONE: i16 = 0;
pub const BM_EDGE_ROTATE_CHECK_DEGENERATE: i16 = 1 << 0;
pub const BM_EDGE_ROTATE_CHECK_BEAUTY: i16 = 1 << 1;

/// `BM_edge_rotate` — rotate edge's face loops between its two faces.
pub fn bm_edge_rotate(bm: &mut BMesh, e: usize, ccw: bool, check_flag: i16) -> usize {
    /* rotation requires exactly 2 manifold faces */
    let Some((l1, l2)) = bm_edge_calc_rotate(bm, e, ccw) else {
        return BM_NIL;
    };
    if (check_flag & BM_EDGE_ROTATE_CHECK_DEGENERATE) != 0
        && !bm_edge_rotate_check_degenerate(bm, e, l1, l2)
    {
        return BM_NIL;
    }
    if (check_flag & BM_EDGE_ROTATE_CHECK_BEAUTY) != 0
        && !bm_edge_rotate_check_beauty(bm, e, l1, l2)
    {
        return BM_NIL;
    }

    let f1 = bm.loo(l1).f;
    let f2 = bm.loo(l2).f;
    /* new edge verts: l1->prev->v, l2->prev->v */
    let v1 = bm.loo(bm.loo(l1).prev).v;
    let v2 = bm.loo(bm.loo(l2).prev).v;

    /* remove both loops from cycle & radial */
    bmesh_radial_loop_remove(bm, e, l1);
    bmesh_radial_loop_remove(bm, e, l2);
    /* swap edge verts (disk stays since verts change in place) */
    bmesh_disk_edge_remove(bm, e, bm.edge(e).v1);
    bmesh_disk_edge_remove(bm, e, bm.edge(e).v2);
    bm.epool[e].v1 = v1;
    bm.epool[e].v2 = v2;
    bmesh_disk_edge_append(bm, e, v1);
    bmesh_disk_edge_append(bm, e, v2);

    /* relink loops: move l1's neighbors around the new corner */
    let l1_prev = bm.loo(l1).prev;
    let l1_next = bm.loo(l1).next;
    let l2_prev = bm.loo(l2).prev;
    let l2_next = bm.loo(l2).next;
    /* l1 goes to f2's cycle at v2's position; l2 to f1's at v1's */
    bm.lpool[l1_prev].next = l1_next;
    bm.lpool[l1_next].prev = l1_prev;
    bm.lpool[l2_prev].next = l2_next;
    bm.lpool[l2_next].prev = l2_prev;

    /* re-insert: l1 between l2_prev's side... standard rotate splice */
    bm.lpool[l1].v = v2;
    bm.lpool[l2].v = v1;
    bm.lpool[l1].f = f2;
    bm.lpool[l2].f = f1;

    bm.lpool[l1].next = l2_next;
    bm.lpool[l1].prev = l2_prev;
    bm.lpool[l2_next].prev = l1;
    bm.lpool[l2_prev].next = l1;

    bm.lpool[l2].next = l1_next;
    bm.lpool[l2].prev = l1_prev;
    bm.lpool[l1_next].prev = l2;
    bm.lpool[l1_prev].next = l2;

    /* faces keep l_first if it wasn't l1/l2; else repoint */
    if bm.face(f1).l_first == l1 {
        bm.fpool[f1].l_first = l1_next;
    }
    if bm.face(f2).l_first == l2 {
        bm.fpool[f2].l_first = l2_next;
    }
    /* len unchanged; relink radial */
    bmesh_radial_loop_append(bm, e, l1);
    bmesh_radial_loop_append(bm, e, l2);
    e
}

/// `BM_face_loop_separate` — detach `l_sep`'s loop-run from its vert.
/// Full vert-separate logic is PORT_PENDING (see vert_separate family).
pub fn bm_face_loop_separate(bm: &mut BMesh, l_sep: usize) -> usize {
    /* duplicate the vert at l_sep */
    let v_old = bm.loo(l_sep).v;
    let co = bm.vert(v_old).co;
    let v_new = bm_vert_create(bm, co, None, 0);
    /* move l_sep and contiguous face-loops sharing v_old onto v_new */
    bm.lpool[l_sep].v = v_new;
    v_new
}
