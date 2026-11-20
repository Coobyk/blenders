//! Port of `bmo_edgenet.cc` — Edge-Net fill operators.
use crate::iterators::EdgesOfVert;
use crate::{BMesh, BM_ELEM_TAG, BM_FACE};
use crate::operators::BMOperator;
use crate::operators::*;

const EDGE_MARK: u8 = 0;
const EDGE_VIS: u8 = 1;
const ELE_NEW: u8 = 2;

#[inline]
fn edge_flag(bm: &BMesh, e: usize, bit: u8) -> bool {
    bm.edge(e).head.api_flag & (1 << bit) != 0
}
#[inline]
fn edge_flag_set(bm: &mut BMesh, e: usize, bit: u8, v: bool) {
    bm.epool[e].head.api_flag &= !(1 << bit);
    if v { bm.epool[e].head.api_flag |= 1 << bit; }
}

/// `bmo_edgenet_fill_exec`.
pub fn bmo_edgenet_fill_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mat_nr = bmo_slot_int_get(op, false, "mat_nr") as i16;
    let use_smooth = bmo_slot_bool_get(op, false, "use_smooth");
    if bm.totvert == 0 || bm.totedge == 0 {
        return;
    }
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, crate::BM_EDGE, BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(bm, op, false, "edges", crate::BM_EDGE, BM_ELEM_TAG, false);
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_FACE, BM_ELEM_TAG);
    crate::edgenet::bm_mesh_edgenet(bm, true, true); /* TODO: sides. */
    bmo_slot_buffer_from_enabled_hflag(bm, op, true, "faces.out", BM_FACE, BM_ELEM_TAG);
    for i in 0..bmo_slot_buffer_len(op, true, "faces.out") {
        let (t, f) = bmo_slot_buffer(op, true, "faces.out")[i];
        debug_assert_eq!(t, BM_FACE);
        bm.fpool[f].mat_nr = mat_nr;
        if use_smooth {
            bm.fpool[f].head.hflag |= crate::BM_ELEM_SMOOTH;
        }
        /* Normals are zeroed. */
        crate::normals::bm_face_calc_normals(bm, f);
    }
    /* --- Attribute Fill --- */
    /* may as well since we have the faces already in a buffer */
    let op_attr = bmo_op_callf(bm, op.flag, "face_attribute_fill", &[
        CallfArg::SlotRef("faces", op, "faces.out"),
        CallfArg::Bool("use_normals", true),
        CallfArg::Bool("use_data", true),
    ]);
    /* check if some faces couldn't be touched */
    if bmo_slot_buffer_len(&op_attr, true, "faces_fail.out") != 0 {
        bmo_op_callf(bm, op.flag, "recalc_face_normals", &[
            CallfArg::SlotRef("faces", &op_attr, "faces_fail.out"),
        ]);
    }
    bmo_op_finish(bm, op_attr);
}

/// `edge_next` — next marked-but-unvisited edge around either vert of `e`.
fn edge_next(bm: &BMesh, e: usize) -> Option<usize> {
    for i in 0..2 {
        let v = if i == 1 { bm.edge(e).v2 } else { bm.edge(e).v1 };
        for e2 in EdgesOfVert::new(bm, v) {
            if edge_flag(bm, e2, EDGE_MARK) && !edge_flag(bm, e2, EDGE_VIS) && e2 != e {
                return Some(e2);
            }
        }
    }
    None
}

/// `bmo_edgenet_prepare_exec`.
pub fn bmo_edgenet_prepare_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mut ok = true;
    bmo_slot_buffer_flag_enable(bm, op, false, "edges", EDGE_MARK);
    /* validate that each edge has at most one other tagged edge in the
     * disk cycle around each of its vertices */
    for i in 0..bmo_slot_buffer_len(op, false, "edges") {
        let e = bmo_slot_buffer(op, false, "edges")[i].1;
        'outer: {
            for i in 0..2 {
                let v = if i == 1 { bm.edge(e).v2 } else { bm.edge(e).v1 };
                let count = bmo_iter_elem_count_flag_edges_of_vert(bm, v, EDGE_MARK);
                if count > 2 {
                    ok = false;
                    break 'outer;
                }
            }
        }
        if !ok { break; }
    }
    if !ok {
        return;
    }
    let mut edges1: Vec<usize> = Vec::new();
    let mut edges2: Vec<usize> = Vec::new();
    /* find connected loops within the input edge */
    let mut count = 0;
    loop {
        let mut e_found = usize::MAX;
        for i in 0..bmo_slot_buffer_len(op, false, "edges") {
            let e = bmo_slot_buffer(op, false, "edges")[i].1;
            if !edge_flag(bm, e, EDGE_VIS) {
                let (v1, v2) = { let ed = bm.edge(e); (ed.v1, ed.v2) };
                if bmo_iter_elem_count_flag_edges_of_vert(bm, v1, EDGE_MARK) == 1
                    || bmo_iter_elem_count_flag_edges_of_vert(bm, v2, EDGE_MARK) == 1
                {
                    e_found = e;
                    break;
                }
            }
        }
        if e_found == usize::MAX { break; }
        let edges = match count {
            0 => &mut edges1,
            1 => &mut edges2,
            _ => break,
        };
        let mut e_opt = Some(e_found);
        while let Some(e) = e_opt {
            edge_flag_set(bm, e, EDGE_VIS, true);
            edges.push(e);
            e_opt = edge_next(bm, e);
        }
        count += 1;
    }
    if edges1.len() > 2
        && crate::query::bm_edge_share_vert_check(bm, *edges1.first().unwrap(), *edges1.last().unwrap())
    {
        if edges2.len() > 2
            && crate::query::bm_edge_share_vert_check(bm, *edges2.first().unwrap(), *edges2.last().unwrap())
        {
            return;
        }
        edges1 = edges2.clone();
        edges2.clear();
    }
    if edges2.len() > 2
        && crate::query::bm_edge_share_vert_check(bm, *edges2.first().unwrap(), *edges2.last().unwrap())
    {
        edges2.clear();
    }
    /* two unconnected loops, connect them */
    if !edges1.is_empty() && !edges2.is_empty() {
        let (v1, v2, mut v3, mut v4);
        if edges1.len() == 1 {
            let ed = bm.edge(edges1[0]);
            v1 = ed.v1; v2 = ed.v2;
        } else {
            let e0 = edges1[0];
            v1 = if crate::structure::bm_vert_in_edge(bm, edges1[1], bm.edge(e0).v1) {
                bm.edge(e0).v2
            } else { bm.edge(e0).v1 };
            let i = edges1.len() - 1;
            let ei = edges1[i];
            v2 = if crate::structure::bm_vert_in_edge(bm, edges1[i - 1], bm.edge(ei).v1) {
                bm.edge(ei).v2
            } else { bm.edge(ei).v1 };
        }
        if edges2.len() == 1 {
            let ed = bm.edge(edges2[0]);
            v3 = ed.v1; v4 = ed.v2;
        } else {
            let e0 = edges2[0];
            v3 = if crate::structure::bm_vert_in_edge(bm, edges2[1], bm.edge(e0).v1) {
                bm.edge(e0).v2
            } else { bm.edge(e0).v1 };
            let i = edges2.len() - 1;
            let ei = edges2[i];
            v4 = if crate::structure::bm_vert_in_edge(bm, edges2[i - 1], bm.edge(ei).v1) {
                bm.edge(ei).v2
            } else { bm.edge(ei).v1 };
        }
        /* Avoid bow tie quads using most planar the triangle pair, see: #30367 & #143905. */
        let (co1, co2, co3, co4) = (bm.vert(v1).co, bm.vert(v2).co, bm.vert(v3).co, bm.vert(v4).co);
        let dvec1 = blenders_math::geom::normal_tri(co1, co2, co4).0;
        let dvec2 = blenders_math::geom::normal_tri(co1, co4, co3).0;
        let dot_24 = dvec1.dot(dvec2);
        let dvec1 = blenders_math::geom::normal_tri(co1, co2, co3).0;
        let dvec2 = blenders_math::geom::normal_tri(co1, co3, co4).0;
        let dot_13 = dvec1.dot(dvec2);
        if dot_24 < dot_13 {
            std::mem::swap(&mut v3, &mut v4);
        }
        let e = crate::core::bm_edge_create(bm, v1, v3, None, crate::BM_CREATE_NO_DOUBLE);
        edge_flag_set(bm, e, ELE_NEW, true);
        let e = crate::core::bm_edge_create(bm, v2, v4, None, crate::BM_CREATE_NO_DOUBLE);
        edge_flag_set(bm, e, ELE_NEW, true);
    } else if !edges1.is_empty() {
        if edges1.len() > 1 {
            let e0 = edges1[0];
            let v1 = if crate::structure::bm_vert_in_edge(bm, edges1[1], bm.edge(e0).v1) {
                bm.edge(e0).v2
            } else { bm.edge(e0).v1 };
            let i = edges1.len() - 1;
            let ei = edges1[i];
            let v2 = if crate::structure::bm_vert_in_edge(bm, edges1[i - 1], bm.edge(ei).v1) {
                bm.edge(ei).v2
            } else { bm.edge(ei).v1 };
            let e = crate::core::bm_edge_create(bm, v1, v2, None, crate::BM_CREATE_NO_DOUBLE);
            edge_flag_set(bm, e, ELE_NEW, true);
        }
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "edges.out", crate::BM_EDGE, ELE_NEW);
}
