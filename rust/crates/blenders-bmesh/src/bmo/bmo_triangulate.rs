//! Port of `bmo_triangulate.cc`.

use crate::operators::*;
use crate::{BMesh, BM_EDGE, BM_FACE};

/// `bmo_triangulate_exec`.
pub fn bmo_triangulate_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let quad_method = bmo_slot_int_get(op, false, "quad_method");
    let ngon_method = bmo_slot_int_get(op, false, "ngon_method");
    crate::marking::bm_mesh_elem_hflag_disable_all(
        bm, (1 << BM_FACE) | (1 << BM_EDGE), crate::BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(bm, op, false, "faces", BM_FACE, crate::BM_ELEM_TAG, false);
    crate::tools::bmesh_triangulate::bm_mesh_triangulate(
        bm, quad_method, ngon_method, 4, true,
        Some(op), Some("face_map.out"), Some("face_map_double.out"));
    bmo_slot_buffer_from_enabled_hflag(bm, op, true, "edges.out", BM_EDGE, crate::BM_ELEM_TAG);
    bmo_slot_buffer_from_enabled_hflag(bm, op, true, "faces.out", BM_FACE, crate::BM_ELEM_TAG);
}
