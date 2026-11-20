//! Port of `bmo_create.cc` — `bmo_contextual_create_exec`.

use crate::operators::*;
use crate::{BMesh, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};

const ELE_NEW: u8 = 1;
const ELE_OUT: u8 = 2;

/// `bmo_contextual_create_exec`.
pub fn bmo_contextual_create_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mut totv = 0i32;
    let mut tote = 0i32;
    let mut totf = 0i32;
    let mat_nr = bmo_slot_int_get(op, false, "mat_nr") as i16;
    let use_smooth = bmo_slot_bool_get(op, false, "use_smooth");
    for &(ht, i) in bmo_slot_buffer(op, false, "geom") {
        match ht {
            BM_VERT => { bmo_elem_flag_enable(bm, (BM_VERT, i), ELE_NEW); totv += 1; }
            BM_EDGE => { bmo_elem_flag_enable(bm, (BM_EDGE, i), ELE_NEW); tote += 1; }
            BM_FACE => { bmo_elem_flag_enable(bm, (BM_FACE, i), ELE_NEW); totf += 1; }
            _ => {}
        }
    }
    if totv == 2 && tote == 0 && totf == 0 {
        let verts: Vec<usize> = bmo_slot_buffer(op, false, "geom")
            .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
        if verts.len() == 2 {
            let e = crate::core::bm_edge_create(
                bm, verts[0], verts[1], None, crate::BM_CREATE_NO_DOUBLE);
            bmo_elem_flag_enable(bm, (BM_EDGE, e), ELE_OUT);
            bmo_slot_buffer_from_enabled_flag(bm, op, true, "edges.out", BM_EDGE, ELE_OUT);
        }
        return;
    }
    if totf == 0 && totv >= 4 && totv == tote + 2 {
        let mut v_free = BM_NIL;
        let mut v_a = BM_NIL;
        let mut v_b = BM_NIL;
        let mut ok = true;
        let inverts: Vec<usize> = bmo_slot_buffer(op, false, "geom")
            .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
        for v in inverts {
            let mut tot_edges = 0;
            for e in crate::iterators::EdgesOfVert::new(bm, v) {
                if bmo_elem_flag_test(bm, (BM_EDGE, e), ELE_NEW) { tot_edges += 1; }
            }
            match tot_edges {
                0 => { if v_free == BM_NIL { v_free = v; } else { ok = false; } }
                1 => {
                    if v_a == BM_NIL { v_a = v; }
                    else if v_b == BM_NIL { v_b = v; }
                    else { ok = false; }
                }
                2 => {}
                _ => { ok = false; }
            }
            if !ok { break; }
        }
        if ok && v_free != BM_NIL && v_a != BM_NIL && v_b != BM_NIL {
            let e = crate::core::bm_edge_create(
                bm, v_free, v_a, None, crate::BM_CREATE_NO_DOUBLE);
            bmo_elem_flag_enable(bm, (BM_EDGE, e), ELE_NEW);
            let e = crate::core::bm_edge_create(
                bm, v_free, v_b, None, crate::BM_CREATE_NO_DOUBLE);
            bmo_elem_flag_enable(bm, (BM_EDGE, e), ELE_NEW);
            tote += 2;
        }
    }
    if tote != 0 {
        bmo_op_callf(bm, op.flag, "edgenet_prepare", &[
            CallfArg::OFlag("edges", 1 << BM_EDGE, ELE_NEW)]);
        let sub = bmo_op_callf(bm, op.flag, "edgenet_fill", &[
            CallfArg::OFlag("edges", 1 << BM_EDGE, ELE_NEW),
            CallfArg::Int("mat_nr", mat_nr as i32),
            CallfArg::Bool("use_smooth", use_smooth),
            CallfArg::Int("sides", 10000)]);
        let faces_out: Vec<_> = bmo_slot_buffer(&sub, true, "faces.out").to_vec();
        if !faces_out.is_empty() {
            slot_get_mut(&mut op.slots_out, "faces.out").buf = faces_out;
            bmo_op_finish(bm, sub);
            return;
        }
        bmo_op_finish(bm, sub);
    }
    if totf != 0 {
        /* dissolve_faces exec: PORT_PENDING — callf runs opdefine
         * (exec None → empty op → fall through as in C failure path). */
        let sub = bmo_op_callf(bm, op.flag, "dissolve_faces", &[
            CallfArg::OFlag("faces", 1 << BM_FACE, ELE_NEW)]);
        let region: Vec<_> = bmo_slot_buffer(&sub, true, "region.out").to_vec();
        if !region.is_empty() {
            slot_get_mut(&mut op.slots_out, "faces.out").buf = region;
            bmo_op_finish(bm, sub);
            return;
        }
        bmo_op_finish(bm, sub);
    }
    if tote > 2 {
        let sub = bmo_op_callf(bm, op.flag, "edgeloop_fill", &[
            CallfArg::OFlag("edges", 1 << BM_EDGE, ELE_NEW)]);
        let faces_out: Vec<_> = bmo_slot_buffer(&sub, true, "faces.out").to_vec();
        if !faces_out.is_empty() {
            slot_get_mut(&mut op.slots_out, "faces.out").buf = faces_out;
            bmo_op_finish(bm, sub);
            return;
        }
        bmo_op_finish(bm, sub);
    }
    /* `if (false)` select-history block: disabled in C, skipped. */
    if totv > 2 {
        let mut vert_arr: Vec<usize> = bmo_slot_buffer(op, false, "geom")
            .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
        crate::construct::bm_verts_sort_radial_plane(bm, &mut vert_arr);
        let f = crate::construct::bm_face_create_ngon_verts(
            bm, &vert_arr, None, crate::BM_CREATE_NO_DOUBLE);
        if f != BM_NIL {
            bmo_elem_flag_enable(bm, (BM_FACE, f), ELE_OUT);
            bm.fpool[f].mat_nr = mat_nr;
            if use_smooth {
                bm.fpool[f].head.hflag |= crate::BM_ELEM_SMOOTH;
            }
            /* `BM_face_copy_shared`: CustomData — PORT_PENDING. */
            bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", BM_FACE, ELE_OUT);
        }
    }
}
