//! Port of `bmo_fill_edgeloop.cc`.

use crate::operators::*;
use crate::{BMesh, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};

const VERT_USED: u8 = 1;
const EDGE_MARK: u8 = 2;
const ELE_OUT: u8 = 4;

/// `bmo_edgeloop_fill_exec`.
pub fn bmo_edgeloop_fill_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let tote = bmo_slot_buffer(op, false, "edges").len();
    let totv = tote;
    let mat_nr = bmo_slot_int_get(op, false, "mat_nr") as i16;
    let use_smooth = bmo_slot_bool_get(op, false, "use_smooth");
    let mut verts: Vec<usize> = Vec::with_capacity(totv);
    let edges: Vec<usize> = bmo_slot_buffer(op, false, "edges")
        .iter().filter(|(ht, _)| *ht == BM_EDGE).map(|&(_, i)| i).collect();
    let mut bailout = false;
    'collect: for &e in &edges {
        bmo_elem_flag_enable(bm, (BM_EDGE, e), EDGE_MARK);
        for v in [bm.edge(e).v1, bm.edge(e).v2] {
            if !bmo_elem_flag_test(bm, (BM_VERT, v), VERT_USED) {
                if verts.len() == tote {
                    bailout = true;
                    break 'collect;
                }
                bmo_elem_flag_enable(bm, (BM_VERT, v), VERT_USED);
                verts.push(v);
            }
        }
    }
    if verts.len() != tote { bailout = true; }
    if !bailout {
        let mut ok = true;
        for &v in &verts {
            let mut count = 0;
            for e in crate::iterators::EdgesOfVert::new(bm, v) {
                if bmo_elem_flag_test(bm, (BM_EDGE, e), EDGE_MARK) { count += 1; }
            }
            if count != 2 { ok = false; break; }
        }
        if ok {
            let mut f_verts: Vec<usize> = vec![BM_NIL; totv];
            let mut e_prev = BM_NIL;
            let mut e_next = BM_NIL;
            let mut totv_used = 0usize;
            while totv_used < totv {
                let mut vi = BM_NIL;
                for &v in &verts {
                    if bmo_elem_flag_test(bm, (BM_VERT, v), VERT_USED) {
                        vi = v;
                        break;
                    }
                }
                let mut v = vi;
                let mut i = 0;
                loop {
                    for e in crate::iterators::EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
                        if bmo_elem_flag_test(bm, (BM_EDGE, e), EDGE_MARK) && e != e_prev {
                            e_next = e;
                            break;
                        }
                    }
                    f_verts[i] = v;
                    bmo_elem_flag_disable(bm, (BM_VERT, v), VERT_USED);
                    totv_used += 1;
                    v = crate::query::bm_edge_other_vert(bm, e_next, v);
                    e_prev = e_next;
                    i += 1;
                    if v == f_verts[0] { break; }
                }
                if crate::query::bm_face_exists(bm, &f_verts[..i]) == BM_NIL {
                    let f = crate::core::bm_face_create_ngon_verts(
                        bm, &f_verts[..i], None, 0, true, false);
                    bmo_elem_flag_enable(bm, (BM_FACE, f), ELE_OUT);
                    bm.fpool[f].mat_nr = mat_nr;
                    if use_smooth {
                        bm.fpool[f].head.hflag |= crate::BM_ELEM_SMOOTH;
                    }
                }
            }
            bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", BM_FACE, ELE_OUT);
        }
    }
}
