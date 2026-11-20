//! Port of `bmo_unsubdivide.cc`.
use crate::{BMesh, BM_ELEM_TAG, BM_VERT};
use crate::operators::*;

/// `bmo_unsubdivide_exec` — `BMVert.hflag & TAG` = touched;
/// `BMVert.index == VERT_INDEX_DO_COLLAPSE` marked by the tool.
pub fn bmo_unsubdivide_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let iterations = bmo_slot_int_get(op, false, "iterations").max(1);
    /* tag verts */
    for v in 0..bm.totvert as usize {
        bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
    }
    for i in 0..bmo_slot_buffer_len(op, false, "verts") {
        let (t, v) = bmo_slot_buffer(op, false, "verts")[i];
        debug_assert_eq!(t, BM_VERT);
        bm.vpool[v].head.hflag |= BM_ELEM_TAG;
    }
    /* do all the real work here */
    crate::tools::bmesh_decimate_unsubdivide::bm_mesh_decimate_unsubdivide_ex(bm, iterations, true);
}
