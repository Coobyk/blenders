//! Port of `bmo_split_edges.cc`.
use crate::{BMesh, BM_ELEM_TAG};
use crate::operators::*;

/// `bmo_split_edges_exec`.
pub fn bmo_split_edges_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_verts = bmo_slot_bool_get(op, false, "use_verts");
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, crate::BM_EDGE, BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(bm, op, false, "edges", crate::BM_EDGE, BM_ELEM_TAG, false);
    if use_verts {
        /* slows down but ok: the modifier doesn't use */
        bmo_slot_buffer_hflag_enable(bm, op, false, "verts", crate::BM_VERT, BM_ELEM_TAG, false);
    }
    /* this is where everything happens */
    crate::tools::bmesh_edgesplit::bm_mesh_edgesplit(bm, use_verts, true, false);
    bmo_slot_buffer_from_enabled_hflag(bm, op, true, "edges.out", crate::BM_EDGE, BM_ELEM_TAG);
}
