//! Port of `bmesh_delete.cc` — flagged/context delete operations.
//!
//! BMO oflag operations ride on `head.api_flag` (BMO_*_flag_* map to
//! api_flag bits in this port since operator flag layers are unported).

use crate::core::*;
use crate::structure::bmesh_radial_length;
use crate::{BMesh, BM_EDGE, BM_FACE, BM_NIL, BM_VERT};

/// Delete context types (`DEL_*` from bmesh_delete.hh / bmesh_tools).
pub const DEL_VERTS: i32 = 1;
pub const DEL_EDGES: i32 = 2;
pub const DEL_EDGESFACES: i32 = 5;
pub const DEL_ONLYFACES: i32 = 3;
pub const DEL_ONLYTAGGED: i32 = 4;
pub const DEL_FACES: i32 = 6;
pub const DEL_FACES_KEEP_BOUNDARY: i32 = 7;
pub const DEL_ALL: i32 = 8;

/// `BM_ALL_NOLOOP` — vert|edge|face htype mask.
pub const BM_ALL_NOLOOP: u8 = BM_VERT | BM_EDGE | BM_FACE;

/// `BM_edge_is_boundary` — edge with at most one radial loop.
pub fn bm_edge_is_boundary(bm: &BMesh, e: usize) -> bool {
    let l = bm.edge(e).l;
    l == BM_NIL || bm.loo(l).radial_next == l
}

fn flag_test_v(bm: &BMesh, v: usize, hflag: u8) -> bool {
    (bm.vert(v).head.hflag & hflag) != 0
}
fn flag_test_e(bm: &BMesh, e: usize, hflag: u8) -> bool {
    (bm.edge(e).head.hflag & hflag) != 0
}
fn flag_test_f(bm: &BMesh, f: usize, hflag: u8) -> bool {
    (bm.face(f).head.hflag & hflag) != 0
}
fn flag_enable_v(bm: &mut BMesh, v: usize, hflag: u8) {
    bm.vpool[v].head.hflag |= hflag;
}
fn flag_enable_e(bm: &mut BMesh, e: usize, hflag: u8) {
    bm.epool[e].head.hflag |= hflag;
}
fn flag_disable_v(bm: &mut BMesh, v: usize, hflag: u8) {
    bm.vpool[v].head.hflag &= !hflag;
}
fn flag_disable_e(bm: &mut BMesh, e: usize, hflag: u8) {
    bm.epool[e].head.hflag &= !hflag;
}

fn bm_remove_tagged_faces(bm: &mut BMesh, hflag: u8) {
    let faces: Vec<usize> = (0..bm.fpool.len())
        .filter(|&f| bm.face(f).head.htype == BM_FACE && flag_test_f(bm, f, hflag))
        .collect();
    for f in faces {
        bm_face_kill(bm, f);
    }
}

fn bm_remove_tagged_edges(bm: &mut BMesh, hflag: u8) {
    let edges: Vec<usize> = (0..bm.epool.len())
        .filter(|&e| bm.edge(e).head.htype == BM_EDGE && flag_test_e(bm, e, hflag))
        .collect();
    for e in edges {
        bm_edge_kill(bm, e);
    }
}

fn bm_remove_tagged_verts(bm: &mut BMesh, hflag: u8) {
    let verts: Vec<usize> = (0..bm.vpool.len())
        .filter(|&v| bm.vert(v).head.htype == BM_VERT && flag_test_v(bm, v, hflag))
        .collect();
    for v in verts {
        bm_vert_kill(bm, v);
    }
}

fn bm_remove_tagged_verts_loose(bm: &mut BMesh, hflag: u8) {
    let verts: Vec<usize> = (0..bm.vpool.len())
        .filter(|&v| {
            bm.vert(v).head.htype == BM_VERT && flag_test_v(bm, v, hflag) && bm.vert(v).e == BM_NIL
        })
        .collect();
    for v in verts {
        bm_kill_only_vert(bm, v);
    }
}

/// `BM_mesh_delete_hflag_tagged`.
pub fn bm_mesh_delete_hflag_tagged(bm: &mut BMesh, hflag: u8, htype: u8) {
    if (htype & BM_FACE) != 0 {
        bm_remove_tagged_faces(bm, hflag);
    }
    if (htype & BM_EDGE) != 0 {
        bm_remove_tagged_edges(bm, hflag);
    }
    if (htype & BM_VERT) != 0 {
        bm_remove_tagged_verts(bm, hflag);
    }
}

/// `BM_mesh_delete_hflag_context`.
pub fn bm_mesh_delete_hflag_context(bm: &mut BMesh, hflag: u8, ty: i32) {
    match ty {
        DEL_VERTS => bm_remove_tagged_verts(bm, hflag),
        DEL_EDGES => {
            /* flush down to vert */
            for e in 0..bm.epool.len() {
                if bm.edge(e).head.htype == BM_EDGE && flag_test_e(bm, e, hflag) {
                    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
                    flag_enable_v(bm, v1, hflag);
                    flag_enable_v(bm, v2, hflag);
                }
            }
            bm_remove_tagged_edges(bm, hflag);
            bm_remove_tagged_verts_loose(bm, hflag);
        }
        DEL_EDGESFACES => bm_remove_tagged_edges(bm, hflag),
        DEL_ONLYFACES => bm_remove_tagged_faces(bm, hflag),
        DEL_ONLYTAGGED => bm_mesh_delete_hflag_tagged(bm, hflag, BM_ALL_NOLOOP),
        DEL_FACES | DEL_FACES_KEEP_BOUNDARY => {
            /* mark verts+edges of tagged faces for delete */
            for f in 0..bm.fpool.len() {
                if bm.face(f).head.htype == BM_FACE && flag_test_f(bm, f, hflag) {
                    let l_first = bm.face(f).l_first;
                    let mut l_iter = l_first;
                    loop {
                        let (v, e) = (bm.loo(l_iter).v, bm.loo(l_iter).e);
                        flag_enable_v(bm, v, hflag);
                        flag_enable_e(bm, e, hflag);
                        l_iter = bm.loo(l_iter).next;
                        if l_iter == l_first {
                            break;
                        }
                    }
                }
            }
            /* unmark verts+edges used by remaining faces */
            for f in 0..bm.fpool.len() {
                if bm.face(f).head.htype == BM_FACE && !flag_test_f(bm, f, hflag) {
                    let l_first = bm.face(f).l_first;
                    let mut l_iter = l_first;
                    loop {
                        let (v, e) = (bm.loo(l_iter).v, bm.loo(l_iter).e);
                        flag_disable_v(bm, v, hflag);
                        flag_disable_e(bm, e, hflag);
                        l_iter = bm.loo(l_iter).next;
                        if l_iter == l_first {
                            break;
                        }
                    }
                }
            }
            /* keep verts of remaining edges; keep boundary edges */
            for e in 0..bm.epool.len() {
                if bm.edge(e).head.htype != BM_EDGE {
                    continue;
                }
                if ty == DEL_FACES_KEEP_BOUNDARY && bm_edge_is_boundary(bm, e) {
                    flag_disable_e(bm, e, hflag);
                }
                if !flag_test_e(bm, e, hflag) {
                    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
                    flag_disable_v(bm, v1, hflag);
                    flag_disable_v(bm, v2, hflag);
                }
            }
            bm_remove_tagged_faces(bm, hflag);
            bm_remove_tagged_edges(bm, hflag);
            bm_remove_tagged_verts(bm, hflag);
        }
        _ => debug_assert!(false),
    }
}

/* ---- oflag (api_flag) variants ------------------------------------------- */

fn oflag_test_v(bm: &BMesh, v: usize, oflag: u8) -> bool {
    (bm.vert(v).head.api_flag & oflag) != 0
}
fn oflag_test_e(bm: &BMesh, e: usize, oflag: u8) -> bool {
    (bm.edge(e).head.api_flag & oflag) != 0
}
fn oflag_test_f(bm: &BMesh, f: usize, oflag: u8) -> bool {
    (bm.face(f).head.api_flag & oflag) != 0
}
fn oflag_enable_v(bm: &mut BMesh, v: usize, oflag: u8) {
    bm.vpool[v].head.api_flag |= oflag;
}
fn oflag_enable_e(bm: &mut BMesh, e: usize, oflag: u8) {
    bm.epool[e].head.api_flag |= oflag;
}
fn oflag_disable_v(bm: &mut BMesh, v: usize, oflag: u8) {
    bm.vpool[v].head.api_flag &= !oflag;
}
fn oflag_disable_e(bm: &mut BMesh, e: usize, oflag: u8) {
    bm.epool[e].head.api_flag &= !oflag;
}

fn bmo_remove_tagged_faces(bm: &mut BMesh, oflag: u8) {
    let faces: Vec<usize> = (0..bm.fpool.len())
        .filter(|&f| bm.face(f).head.htype == BM_FACE && oflag_test_f(bm, f, oflag))
        .collect();
    for f in faces {
        bm_face_kill(bm, f);
    }
}
fn bmo_remove_tagged_edges(bm: &mut BMesh, oflag: u8) {
    let edges: Vec<usize> = (0..bm.epool.len())
        .filter(|&e| bm.edge(e).head.htype == BM_EDGE && oflag_test_e(bm, e, oflag))
        .collect();
    for e in edges {
        bm_edge_kill(bm, e);
    }
}
fn bmo_remove_tagged_verts(bm: &mut BMesh, oflag: u8) {
    let verts: Vec<usize> = (0..bm.vpool.len())
        .filter(|&v| bm.vert(v).head.htype == BM_VERT && oflag_test_v(bm, v, oflag))
        .collect();
    for v in verts {
        bm_vert_kill(bm, v);
    }
}
fn bmo_remove_tagged_verts_loose(bm: &mut BMesh, oflag: u8) {
    let verts: Vec<usize> = (0..bm.vpool.len())
        .filter(|&v| {
            bm.vert(v).head.htype == BM_VERT && oflag_test_v(bm, v, oflag) && bm.vert(v).e == BM_NIL
        })
        .collect();
    for v in verts {
        bm_kill_only_vert(bm, v);
    }
}

/// `BMO_mesh_delete_oflag_tagged`.
pub fn bmo_mesh_delete_oflag_tagged(bm: &mut BMesh, oflag: u8, htype: u8) {
    if (htype & BM_FACE) != 0 {
        bmo_remove_tagged_faces(bm, oflag);
    }
    if (htype & BM_EDGE) != 0 {
        bmo_remove_tagged_edges(bm, oflag);
    }
    if (htype & BM_VERT) != 0 {
        bmo_remove_tagged_verts(bm, oflag);
    }
}

/// `BMO_mesh_delete_oflag_context`.
pub fn bmo_mesh_delete_oflag_context(
    bm: &mut BMesh,
    oflag: u8,
    ty: i32,
    prepare_fn: Option<&dyn Fn(&mut BMesh)>,
) {
    match ty {
        DEL_VERTS => {
            if let Some(f) = prepare_fn {
                f(bm);
            }
            bmo_remove_tagged_verts(bm, oflag);
        }
        DEL_EDGES => {
            for e in 0..bm.epool.len() {
                if bm.edge(e).head.htype == BM_EDGE && oflag_test_e(bm, e, oflag) {
                    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
                    oflag_enable_v(bm, v1, oflag);
                    oflag_enable_v(bm, v2, oflag);
                }
            }
            if let Some(f) = prepare_fn {
                f(bm);
            }
            bmo_remove_tagged_edges(bm, oflag);
            bmo_remove_tagged_verts_loose(bm, oflag);
        }
        DEL_EDGESFACES => {
            if let Some(f) = prepare_fn {
                f(bm);
            }
            bmo_remove_tagged_edges(bm, oflag);
        }
        DEL_ONLYFACES => {
            if let Some(f) = prepare_fn {
                f(bm);
            }
            bmo_remove_tagged_faces(bm, oflag);
        }
        DEL_ONLYTAGGED => {
            if let Some(f) = prepare_fn {
                f(bm);
            }
            bmo_mesh_delete_oflag_tagged(bm, oflag, BM_ALL_NOLOOP);
        }
        DEL_FACES | DEL_FACES_KEEP_BOUNDARY => {
            for f in 0..bm.fpool.len() {
                if bm.face(f).head.htype == BM_FACE && oflag_test_f(bm, f, oflag) {
                    let l_first = bm.face(f).l_first;
                    let mut l_iter = l_first;
                    loop {
                        let (v, e) = (bm.loo(l_iter).v, bm.loo(l_iter).e);
                        oflag_enable_v(bm, v, oflag);
                        oflag_enable_e(bm, e, oflag);
                        l_iter = bm.loo(l_iter).next;
                        if l_iter == l_first {
                            break;
                        }
                    }
                }
            }
            for f in 0..bm.fpool.len() {
                if bm.face(f).head.htype == BM_FACE && !oflag_test_f(bm, f, oflag) {
                    let l_first = bm.face(f).l_first;
                    let mut l_iter = l_first;
                    loop {
                        let (v, e) = (bm.loo(l_iter).v, bm.loo(l_iter).e);
                        oflag_disable_v(bm, v, oflag);
                        oflag_disable_e(bm, e, oflag);
                        l_iter = bm.loo(l_iter).next;
                        if l_iter == l_first {
                            break;
                        }
                    }
                }
            }
            for e in 0..bm.epool.len() {
                if bm.edge(e).head.htype != BM_EDGE {
                    continue;
                }
                if ty == DEL_FACES_KEEP_BOUNDARY && bm_edge_is_boundary(bm, e) {
                    oflag_disable_e(bm, e, oflag);
                }
                if !oflag_test_e(bm, e, oflag) {
                    let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
                    oflag_disable_v(bm, v1, oflag);
                    oflag_disable_v(bm, v2, oflag);
                }
            }
            if let Some(f) = prepare_fn {
                f(bm);
            }
            bmo_remove_tagged_faces(bm, oflag);
            bmo_remove_tagged_edges(bm, oflag);
            bmo_remove_tagged_verts(bm, oflag);
        }
        _ => debug_assert!(false),
    }
}
