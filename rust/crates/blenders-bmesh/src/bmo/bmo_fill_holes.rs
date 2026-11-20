//! Port of `bmo_fill_holes.cc` — `bmo_holes_fill_exec`.
use crate::{BMesh, BM_ELEM_TAG, BM_FACE};
use crate::operators::*;

/// `bmo_holes_fill_exec`.
pub fn bmo_holes_fill_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let sides = bmo_slot_int_get(op, false, "sides") as usize;
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, crate::BM_EDGE, BM_ELEM_TAG);
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_FACE, BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(bm, op, false, "edges", crate::BM_EDGE, BM_ELEM_TAG, false);
    crate::edgenet::bm_mesh_edgenet(bm, true, true); /* TODO: sides. */
    /* bad - remove faces after as a workaround */
    if sides != 0 {
        bmo_slot_buffer_from_enabled_hflag(bm, op, true, "faces.out", BM_FACE, BM_ELEM_TAG);
        for i in 0..bmo_slot_buffer_len(op, true, "faces.out") {
            let f = bmo_slot_buffer(op, true, "faces.out")[i].1;
            if bm.face(f).len as usize > sides {
                crate::core::bm_face_kill(bm, f);
            }
        }
    }
    bmo_slot_buffer_from_enabled_hflag(bm, op, true, "faces.out", BM_FACE, BM_ELEM_TAG);
    /* --- Attribute Fill --- */
    let op_attr = bmo_op_callf(bm, op.flag, "face_attribute_fill", &[
        CallfArg::SlotRef("faces", op, "faces.out"),
        CallfArg::Bool("use_normals", true),
        CallfArg::Bool("use_data", true),
    ]);
    /* check if some faces couldn't be touched */
    if bmo_slot_buffer_len(&op_attr, true, "faces_fail.out") != 0 {
        for i in 0..bmo_slot_buffer_len(&op_attr, true, "faces_fail.out") {
            let f = bmo_slot_buffer(&op_attr, true, "faces_fail.out")[i].1;
            crate::normals::bm_face_calc_normals(bm, f); /* Normals are zeroed. */
        }
        bmo_op_callf(bm, op.flag, "recalc_face_normals", &[
            CallfArg::SlotRef("faces", &op_attr, "faces_fail.out"),
        ]);
    }
    bmo_op_finish(bm, op_attr);
}
