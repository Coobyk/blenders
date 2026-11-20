//! Port of `bmo_wireframe.cc` — `bmo_wireframe_exec`.
use crate::{BMesh, BM_ELEM_TAG, BM_FACE};
use crate::operators::*;

/// `bmo_wireframe_exec`.
pub fn bmo_wireframe_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let offset = bmo_slot_float_get(op, false, "thickness");
    let offset_fac = bmo_slot_float_get(op, false, "offset");
    let use_replace = bmo_slot_bool_get(op, false, "use_replace");
    let use_boundary = bmo_slot_bool_get(op, false, "use_boundary");
    let use_even_offset = bmo_slot_bool_get(op, false, "use_even_offset");
    let use_relative_offset = bmo_slot_bool_get(op, false, "use_relative_offset");
    let use_crease = bmo_slot_bool_get(op, false, "use_crease");
    let crease_weight = bmo_slot_float_get(op, false, "crease_weight");
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, crate::BM_EDGE, BM_ELEM_TAG);
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_FACE, BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(bm, op, false, "faces", BM_FACE, BM_ELEM_TAG, false);
    crate::tools::bmesh_wireframe::bm_mesh_wireframe(
        bm,
        offset,
        offset_fac,
        0.0,
        use_replace,
        use_boundary,
        use_even_offset,
        use_relative_offset,
        use_crease,
        crease_weight,
        -1,     /* dummy vgroup */
        false,
        0,
        i16::MAX as i32, /* MAXMAT */
        true,
    );
    bmo_slot_buffer_from_enabled_hflag(bm, op, true, "faces.out", BM_FACE, BM_ELEM_TAG);
}
