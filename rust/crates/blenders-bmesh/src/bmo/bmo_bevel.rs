//! Port of `bmo_bevel.cc` — bevel operator wrapper around `bm_mesh_bevel`.

use crate::operators::*;
use crate::{BMesh, BM_EDGE, BM_FACE, BM_VERT};

/// `bmo_bevel_exec`.
pub fn bmo_bevel_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let offset = bmo_slot_float_get(op, false, "offset");
    let _offset_type = bmo_slot_int_get(op, false, "offset_type");
    let _profile_type = bmo_slot_int_get(op, false, "profile_type");
    let _seg = bmo_slot_int_get(op, false, "segments");
    let _affect_type = bmo_slot_int_get(op, false, "affect");
    let _profile = bmo_slot_float_get(op, false, "profile");
    let _clamp_overlap = bmo_slot_bool_get(op, false, "clamp_overlap");
    let _material = bmo_slot_int_get(op, false, "material");
    let _loop_slide = bmo_slot_bool_get(op, false, "loop_slide");
    let _mark_seam = bmo_slot_bool_get(op, false, "mark_seam");
    let _mark_sharp = bmo_slot_bool_get(op, false, "mark_sharp");
    let _harden_normals = bmo_slot_bool_get(op, false, "harden_normals");
    let _face_strength_mode = bmo_slot_int_get(op, false, "face_strength_mode");
    let _miter_outer = bmo_slot_int_get(op, false, "miter_outer");
    let _miter_inner = bmo_slot_int_get(op, false, "miter_inner");
    let _spread = bmo_slot_float_get(op, false, "spread");
    let _vmesh_method = bmo_slot_int_get(op, false, "vmesh_method");
    /* `custom_profile` CurveProfile ptr: PORT_PENDING. */
    if offset > 0.0 {
        crate::marking::bm_mesh_elem_hflag_disable_all(
            bm, (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE), crate::BM_ELEM_TAG);
        for (ht, i) in bmo_slot_buffer(op, false, "geom").to_vec() {
            if ht == BM_VERT {
                bm.vpool[i].head.hflag |= crate::BM_ELEM_TAG;
            } else if ht == BM_EDGE {
                if crate::query::bm_edge_is_manifold(bm, i) {
                    bm.epool[i].head.hflag |= crate::BM_ELEM_TAG;
                    let (v1, v2) = (bm.edge(i).v1, bm.edge(i).v2);
                    bm.vpool[v1].head.hflag |= crate::BM_ELEM_TAG;
                    bm.vpool[v2].head.hflag |= crate::BM_ELEM_TAG;
                }
            }
        }
        bm.use_toolflags = true;
        crate::tools::bmesh_bevel::bm_mesh_bevel(
            bm,
            offset,
            _offset_type,
            _profile_type,
            _seg,
            _profile,
            _affect_type,
            false,
            _clamp_overlap,
            None,
            -1,
            _material,
            _loop_slide,
            _mark_seam,
            _mark_sharp,
            _harden_normals,
            _face_strength_mode,
            _miter_outer,
            _miter_inner,
            _spread,
            None,
            _vmesh_method,
            -1, /* `bevel_weight_vert` CustomData offset: PORT_PENDING. */
            -1, /* `bevel_weight_edge` CustomData offset: PORT_PENDING. */
        );
        bmo_slot_buffer_from_enabled_hflag(bm, op, true, "faces.out", BM_FACE, crate::BM_ELEM_TAG);
        bmo_slot_buffer_from_enabled_hflag(bm, op, true, "edges.out", BM_EDGE, crate::BM_ELEM_TAG);
        bmo_slot_buffer_from_enabled_hflag(bm, op, true, "verts.out", BM_VERT, crate::BM_ELEM_TAG);
    }
}
