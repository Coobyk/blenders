//! Port of `bmo_beautify.cc` — `bmo_beautify_fill_exec`.
use crate::iterators::EdgesOfMesh;
use crate::{BMesh, BM_ELEM_TAG, BM_EDGE, BM_FACE};
use crate::operators::*;
use crate::tools::bmesh_beautify::{EDGE_RESTRICT_DEGENERATE, VERT_RESTRICT_TAG};

const ELE_NEW: u8 = 1;
const FACE_MARK: u8 = 2;

/// `bmo_beautify_fill_exec`.
pub fn bmo_beautify_fill_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_restrict_tag = bmo_slot_bool_get(op, false, "use_restrict_tag");
    let flag = (if use_restrict_tag { VERT_RESTRICT_TAG } else { 0 })
        | EDGE_RESTRICT_DEGENERATE;
    let method = bmo_slot_int_get(op, false, "method") as i16;
    for i in 0..bmo_slot_buffer_len(op, false, "faces") {
        let f = bmo_slot_buffer(op, false, "faces")[i].1;
        if bm.face(f).len == 3 {
            bm.fpool[f].head.api_flag |= 1 << FACE_MARK;
        }
    }
    for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        bm.epool[e].head.hflag &= !BM_ELEM_TAG;
    }
    let mut edge_array: Vec<usize> = Vec::new();
    for i in 0..bmo_slot_buffer_len(op, false, "edges") {
        let e = bmo_slot_buffer(op, false, "edges")[i].1;
        /* edge is manifold and can be rotated, faces tagged */
        let l = bm.edge(e).l;
        if l != crate::BM_NIL
            && crate::mods::bm_edge_rotate_check(bm, e)
            && bm.face(bm.loo(l).f).head.api_flag & (1 << FACE_MARK) != 0
            && bm.face(bm.loo(bm.loo(l).radial_next).f).head.api_flag & (1 << FACE_MARK) != 0
        {
            edge_array.push(e);
        }
    }
    crate::tools::bmesh_beautify::bm_mesh_beautify_fill(
        bm, &mut edge_array, flag, method, ELE_NEW, FACE_MARK | ELE_NEW);
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "geom.out", BM_EDGE | (1 << BM_FACE), ELE_NEW);
}
