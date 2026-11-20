//! Port of `bmo_fill_attribute.cc` — flood-fill face attributes from
//! adjacent untagged faces.
use crate::iterators::{FacesOfMesh, LoopsOfFace};
use crate::{BMesh, BM_ELEM_TAG, BM_FACE};
use crate::operators::BMOperator;
use crate::operators::*;

/// `bm_loop_is_all_radial_tag`.
fn bm_loop_is_all_radial_tag(bm: &BMesh, l: usize) -> bool {
    let mut l_iter = bm.loo(l).radial_next;
    loop {
        if bm.face(bm.loo(l_iter).f).head.hflag & BM_ELEM_TAG == 0 {
            return false;
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l {
            return true;
        }
    }
}

/// `bm_loop_is_face_untag` — callback for `bm_face_copy_shared`.
fn bm_loop_is_face_untag(bm: &BMesh, l: usize) -> bool {
    bm.face(bm.loo(l).f).head.hflag & BM_ELEM_TAG == 0
}

/// `bm_face_copy_shared_all` — copy all attributes from adjacent
/// untagged faces; `use_normals` copies winding, `use_data` attrs.
fn bm_face_copy_shared_all(bm: &mut BMesh, l: usize, use_normals: bool, use_data: bool) {
    let mut l_other = bm.loo(l).radial_next;
    let f = bm.loo(l).f;
    while bm.face(bm.loo(l_other).f).head.hflag & BM_ELEM_TAG != 0 {
        l_other = bm.loo(l_other).radial_next;
    }
    let f_other = bm.loo(l_other).f;
    if use_data {
        /* copy face-attrs + loop-attrs — PORT_PENDING (CustomData) */
        let _ = f_other;
        let _ = bm_loop_is_face_untag;
    }
    if use_normals {
        /* copy winding (flipping) */
        if bm.loo(l).v == bm.loo(l_other).v {
            crate::polygon::bm_face_normal_flip(bm, f);
        }
    }
}

/// `bmesh_face_attribute_fill` — flood fill attributes; returns faces filled.
fn bmesh_face_attribute_fill(bm: &mut BMesh, use_normals: bool, use_data: bool) -> usize {
    let mut loop_queue_prev: Vec<usize> = Vec::new();
    let mut loop_queue_next: Vec<usize> = Vec::new();
    let mut face_tot = 0;
    for f in FacesOfMesh::new(bm) {
        if bm.face(f).head.hflag & BM_ELEM_TAG != 0 {
            for l in LoopsOfFace::new(bm, f) {
                if !bm_loop_is_all_radial_tag(bm, l) {
                    loop_queue_prev.push(l);
                }
            }
        }
    }
    while !loop_queue_prev.is_empty() {
        while let Some(l) = loop_queue_prev.pop() {
            /* check we're still un-assigned */
            let lf = bm.loo(l).f;
            if bm.face(lf).head.hflag & BM_ELEM_TAG != 0 {
                bm.fpool[lf].head.hflag &= !BM_ELEM_TAG;
                let mut l_iter = bm.loo(l).next;
                loop {
                    let mut l_radial_iter = bm.loo(l_iter).radial_next;
                    if l_radial_iter != l_iter {
                        loop {
                            if bm.face(bm.loo(l_radial_iter).f).head.hflag & BM_ELEM_TAG != 0 {
                                loop_queue_next.push(l_radial_iter);
                            }
                            l_radial_iter = bm.loo(l_radial_iter).radial_next;
                            if l_radial_iter == l_iter { break; }
                        }
                    }
                    l_iter = bm.loo(l_iter).next;
                    if l_iter == l { break; }
                }
                /* do last because of face flipping */
                bm_face_copy_shared_all(bm, l, use_normals, use_data);
                face_tot += 1;
            }
        }
        std::mem::swap(&mut loop_queue_prev, &mut loop_queue_next);
    }
    face_tot
}

/// `bmo_face_attribute_fill_exec`.
pub fn bmo_face_attribute_fill_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_normals = bmo_slot_bool_get(op, false, "use_normals");
    let use_data = bmo_slot_bool_get(op, false, "use_data");
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_FACE, BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(bm, op, false, "faces", BM_FACE, BM_ELEM_TAG, false);
    let face_tot = bmesh_face_attribute_fill(bm, use_normals, use_data);
    if face_tot != bmo_slot_buffer_len(op, false, "faces") {
        /* any remaining tags will be skipped */
        bmo_slot_buffer_from_enabled_hflag(bm, op, true, "faces_fail.out", BM_FACE, BM_ELEM_TAG);
    }
}
