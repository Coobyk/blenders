//! Port of `bmo_dissolve.cc` — dissolve_faces / dissolve_verts / dissolve_limit.
//! `dissolve_edges` + `dissolve_degenerate*` remain PORT_PENDING.
//! `BMO_*_flag` oflags → `head.api_flag`; BMW_ISLAND_MANIFOLD walk via walkers.

use crate::core::*;
use crate::operators::*;
use crate::query::*;
use crate::structure::bm_edge_other_vert;
use crate::walkers::{BMWType, BMWalker};
use crate::{BMesh, BM_EDGE, BM_FACE, BM_VERT, BM_NIL};

const EDGE_MARK: u8 = 1;
const EDGE_ISGC: u8 = 8;
const VERT_MARK: u8 = 1;
const VERT_MARK_PAIR: u8 = 4;
const VERT_TAG: u8 = 2;
const VERT_ISGC: u8 = 8;
const VERT_MARK_TEAR: u8 = 16;
const FACE_MARK: u8 = 1;
const FACE_ORIG: u8 = 2;
const FACE_NEW: u8 = 4;
const FACE_TAG: u8 = 8;
const DEL_FACES: i32 = 3;

/// `bm_vert_collapse_edge_and_merge` — merge hidden flags, collapse,
/// never leave visible edge on hidden verts.
fn bm_vert_collapse_edge_and_merge(bm: &mut BMesh, v: usize, do_del: bool) -> usize {
    let Some((e0, e1)) = bm_vert_edge_pair(bm, v) else { return BM_NIL };
    /* BM_elem_flag_merge_ex(e0, e1, HIDDEN): e0 |= e1 & HIDDEN */
    bm.epool[e0].head.hflag |= bm.edge(e1).head.hflag & crate::BM_ELEM_HIDDEN;
    let e = bm.vert(v).e;
    let e_new = crate::mods::bm_vert_collapse_edge(bm, e, v, do_del, true, true);
    if e_new != BM_NIL {
        if bm.edge(e_new).head.hflag & crate::BM_ELEM_HIDDEN == 0
            && (bm.vert(bm.edge(e_new).v1).head.hflag & crate::BM_ELEM_HIDDEN != 0
                || bm.vert(bm.edge(e_new).v2).head.hflag & crate::BM_ELEM_HIDDEN != 0)
        {
            if bm.edge(e_new).head.hflag & crate::BM_ELEM_SELECT != 0 {
                crate::marking::bm_edge_select_set_noflush(bm, e_new, false);
            }
            bm.epool[e_new].head.hflag |= crate::BM_ELEM_HIDDEN;
        }
    }
    e_new
}

/// `bm_face_split` — split tagged-vert faces on untagged diagonals,
/// optionally killing the vert's remaining edges after.
fn bm_face_split_op(bm: &mut BMesh, oflag: u8, use_edge_delete: bool) {
    let mut edge_delete_verts: Vec<usize> = Vec::new();
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(v).head.api_flag & oflag == 0 { continue; }
        if !bm_vert_is_edge_pair(bm, v) {
            for l in crate::iterators::LoopsOfVert::new(bm, v).collect::<Vec<_>>() {
                if bm.face(bm.loo(l).f).len > 3 {
                    let ln = bm.loo(l).next;
                    let lp = bm.loo(l).prev;
                    if bm.vert(bm.loo(ln).v).head.api_flag & oflag == 0
                        && bm.vert(bm.loo(lp).v).head.api_flag & oflag == 0
                    {
                        crate::mods::bm_face_split(bm, bm.loo(l).f, ln, lp, None, true);
                    }
                }
            }
            if use_edge_delete {
                edge_delete_verts.push(v);
            }
        }
    }
    if use_edge_delete {
        for v in edge_delete_verts {
            while bm.vert(v).e != BM_NIL {
                let e = bm.vert(v).e;
                bm_edge_kill(bm, e);
            }
        }
    }
}

/// `bmo_dissolve_faces_exec`.
pub fn bmo_dissolve_faces_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_verts = bmo_slot_bool_get(op, false, "use_verts");
    if use_verts {
        for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if !bm_vert_is_edge_pair(bm, v) {
                bm.vpool[v].head.api_flag |= VERT_MARK;
            } else {
                bm.vpool[v].head.api_flag &= !VERT_MARK;
            }
        }
    }
    bmo_slot_buffer_flag_enable(bm, op, false, "faces", FACE_MARK | FACE_TAG);
    let faces_in: Vec<usize> = op.slots_in.iter().find(|s| s.name == "faces")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_FACE).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    let mut regions: Vec<Vec<usize>> = Vec::new();
    for f in faces_in {
        if bm.face(f).head.api_flag & FACE_TAG == 0 { continue; }
        /* BMW_ISLAND_MANIFOLD over FACE_MARK-masked faces */
        let mut w = BMWalker::new(BMWType::IslandManifold, 0, 0);
        w.mask_face = FACE_MARK;
        w.begin(bm, BM_FACE, f);
        let faces: Vec<usize> = crate::walkers::bmw_walk_all(bm, &mut w)
            .into_iter().filter(|(ht, _)| *ht == BM_FACE).map(|(_, i)| i).collect();
        if faces.len() < 2 { continue; }
        for &face in &faces {
            bm.fpool[face].head.api_flag &= !FACE_TAG;
            bm.fpool[face].head.api_flag |= FACE_ORIG;
        }
        regions.push(faces);
    }
    let mut totface_target = bm.totface;
    for faces in regions {
        let faces_len = faces.len() as i32;
        let mut f_double = BM_NIL;
        let mut f_new = bm_faces_join(bm, &faces, true, Some(&mut f_double));
        if f_new != BM_NIL {
            totface_target -= faces_len - 1;
            if f_double != BM_NIL {
                bm_face_kill(bm, f_new);
                totface_target -= 1;
                f_new = f_double;
            }
            bm.fpool[f_new].head.api_flag &= !FACE_ORIG;
            bm.fpool[f_new].head.api_flag |= FACE_NEW;
        } else {
            for face in faces {
                bm.fpool[face].head.api_flag &= !FACE_ORIG;
            }
        }
    }
    if totface_target != bm.totface {
        crate::operators::bmo_op_callf(
            bm, op.flag, "delete",
            &[CallfArg::OFlag("geom".into(), 1 << BM_FACE, FACE_ORIG),
              CallfArg::Int("context".into(), DEL_FACES)]);
    }
    if use_verts {
        for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.vert(v).head.api_flag & VERT_MARK == 0 { continue; }
            if bm_vert_is_edge_pair(bm, v) {
                bm_vert_collapse_edge_and_merge(bm, v, true);
            }
        }
    }
    bmo_slot_buffer_from_enabled_flag(
        bm, op, true, "region.out", 1 << BM_FACE, FACE_NEW);
}

/// `bmo_dissolve_verts_exec`.
pub fn bmo_dissolve_verts_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_face_split = bmo_slot_bool_get(op, false, "use_face_split");
    let use_boundary_tear = bmo_slot_bool_get(op, false, "use_boundary_tear");
    let verts_in: Vec<usize> = op.slots_in.iter().find(|s| s.name == "verts")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_VERT).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    for &v in &verts_in {
        bm.vpool[v].head.api_flag |= VERT_MARK | VERT_ISGC;
    }
    if use_face_split {
        bm_face_split_op(bm, VERT_MARK, false);
    }
    if use_boundary_tear {
        for &v in &verts_in {
            if bm.vert(v).e == BM_NIL { continue; }
            if !bm_vert_is_edge_pair(bm, v) {
                for e in crate::iterators::EdgesOfVert::new(bm, v) {
                    if crate::delete::bm_edge_is_boundary(bm, e) {
                        bm.vpool[v].head.api_flag |= VERT_MARK_TEAR;
                        break;
                    }
                }
            }
        }
        bm_face_split_op(bm, VERT_MARK_TEAR, true);
    }
    for &v in &verts_in {
        if bm.vert(v).e == BM_NIL { continue; }
        let mut e_first = BM_NIL;
        for l_first in crate::iterators::LoopsOfVert::new(bm, v).collect::<Vec<_>>() {
            let mut l_iter = l_first;
            loop {
                let (lv, le) = (bm.loo(l_iter).v, bm.loo(l_iter).e);
                bm.vpool[lv].head.api_flag |= VERT_ISGC;
                bm.epool[le].head.api_flag |= EDGE_ISGC;
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first { break; }
            }
            e_first = bm.loo(l_first).e;
        }
        if e_first != BM_NIL {
            let mut e = e_first;
            loop {
                let e_next = crate::structure::bmesh_disk_edge_next(bm, e, v);
                if crate::edgenet::edge_is_wire(bm, e) {
                    bm_edge_kill(bm, e);
                }
                e = e_next;
                if e == e_first { break; }
            }
        }
    }
    for &v in &verts_in {
        if bm.vert(v).e != BM_NIL && bm_vert_is_edge_pair(bm, v) {
            bm.vpool[v].head.api_flag |= VERT_MARK_PAIR;
        }
    }
    for &v in &verts_in {
        if bm.vert(v).e == BM_NIL { continue; }
        if bm.vert(v).head.api_flag & VERT_MARK_PAIR == 0 {
            for e in crate::iterators::EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
                if let Some((l_a, l_b)) = bm_edge_loop_pair(bm, e) {
                    crate::mods::bm_faces_join_pair(bm, l_a, l_b, false, None);
                }
            }
        }
    }
    /* kill loop-less ISGC edges */
    for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.edge(e).l == BM_NIL && bm.edge(e).head.api_flag & EDGE_ISGC != 0 {
            bm_edge_kill(bm, e);
        }
    }
    for &v in &verts_in {
        if bm.vert(v).e != BM_NIL && bm_vert_is_edge_pair(bm, v) {
            bm_vert_collapse_edge_and_merge(bm, v, false);
        }
    }
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(v).e == BM_NIL && bm.vert(v).head.api_flag & VERT_ISGC != 0 {
            bm_vert_kill(bm, v);
        }
    }
}

/// `bmo_dissolve_limit_exec`.
pub fn bmo_dissolve_limit_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let einput: Vec<usize> = op.slots_in.iter().find(|s| s.name == "edges")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_EDGE).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    let mut vinput: Vec<Option<usize>> = op.slots_in.iter().find(|s| s.name == "verts")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_VERT)
            .map(|(_, i)| Some(*i)).collect())
        .unwrap_or_default();
    let angle_max = std::f32::consts::FRAC_PI_2;
    let angle_limit = angle_max.min(bmo_slot_float_get(op, false, "angle_limit"));
    let do_dissolve_boundaries = bmo_slot_bool_get(op, false, "use_dissolve_boundaries");
    let delimit = bmo_slot_int_get(op, false, "delimit") as u32;
    crate::tools::bmesh_decimate_dissolve::bm_mesh_decimate_dissolve_ex(
        bm, angle_limit, do_dissolve_boundaries, delimit,
        &mut vinput, &einput, FACE_NEW);
    bmo_slot_buffer_from_enabled_flag(
        bm, op, true, "region.out", 1 << BM_FACE, FACE_NEW);
}
