//! Port of `bmo_offset_edgeloops.cc` (USE_CAP_OPTION on).
//! C marks "new" verts via `elem_index == -1`; here `new_verts` is a side-set
//! populated by `bm_edge_split` returns.

use std::collections::HashSet;
use crate::operators::*;
use crate::{BMesh, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};

const ELE_NEW: u8 = 1;
const ELE_VERT_ENDPOINT: u8 = 2;
const OFFSET: f32 = 0.0;

/// `bm_face_split_walk_back`.
fn bm_face_split_walk_back(bm: &mut BMesh, l_src: usize, r_l: &mut usize) -> usize {
    /* walk back over contiguous new verts, collecting co's */
    let mut cos_rev: Vec<blenders_math::vector::Vec3> = Vec::new();
    let mut l_dst = bm.loo(l_src).prev;
    while NEW_VERT.with(|s| s.borrow().contains(&bm.loo(bm.loo(l_dst).prev).v)) {
        cos_rev.push(bm.vert(bm.loo(l_dst).v).co);
        l_dst = bm.loo(l_dst).prev;
    }
    let cos: Vec<_> = cos_rev.into_iter().rev().collect();
    crate::mods::bm_face_split_n(
        bm, bm.loo(l_src).f, bm.loo(l_dst).prev, bm.loo(l_src).next, &cos, Some(r_l))
}

thread_local! {
    static NEW_VERT: std::cell::RefCell<HashSet<usize>> = std::cell::RefCell::new(HashSet::new());
}

/// `bmo_offset_edgeloops_exec`.
pub fn bmo_offset_edgeloops_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let edges_num = bmo_slot_buffer(op, false, "edges").len();
    let use_cap_endpoint = bmo_slot_bool_get(op, false, "use_cap_endpoint");
    let mut v_edges_max = 0usize;
    NEW_VERT.with(|s| s.borrow_mut().clear());
    let mut new_verts: HashSet<usize> = HashSet::new();
    crate::marking::bm_mesh_elem_hflag_disable_all(
        bm, (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE), crate::BM_ELEM_TAG);
    let mut verts: Vec<usize> = Vec::with_capacity(edges_num * 2);
    for (ht, e) in bmo_slot_buffer(op, false, "edges").to_vec() {
        if ht != BM_EDGE { continue; }
        bm.epool[e].head.hflag |= crate::BM_ELEM_TAG;
        for v in [bm.edge(e).v1, bm.edge(e).v2] {
            if bm.vert(v).head.hflag & crate::BM_ELEM_TAG == 0 {
                bm.vpool[v].head.hflag |= crate::BM_ELEM_TAG;
                verts.push(v);
            }
        }
    }
    /* keep only verts on the boundary (have tagged + untagged edges) */
    verts.retain(|&v| {
        let mut flag = 0;
        for e in crate::iterators::EdgesOfVert::new(bm, v) {
            flag |= if bm.edge(e).head.hflag & crate::BM_ELEM_TAG != 0 { 1 } else { 2 };
            if flag == 3 { break; }
        }
        flag == 3
    });
    if verts.is_empty() { return; }
    for i in 0..verts.len() {
        let v = verts[i];
        let mut v_edges_num = 0;
        let mut v_edges_num_untag = 0;
        for e in crate::iterators::EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
            if bm.edge(e).head.hflag & crate::BM_ELEM_TAG == 0 {
                for l in crate::iterators::LoopsOfEdge::new(bm, e).collect::<Vec<_>>() {
                    let f = bm.loo(l).f;
                    bm.fpool[f].head.hflag |= crate::BM_ELEM_TAG;
                }
                let v_other = crate::structure::bm_edge_other_vert(bm, e, v);
                let nv = crate::mods::bm_edge_split(bm, e, v_other, None, 1.0 - OFFSET);
                new_verts.insert(nv);
            } else {
                v_edges_num_untag += 1;
            }
            v_edges_num += 1;
        }
        if v_edges_num_untag == 1 {
            bmo_elem_flag_enable(bm, (BM_VERT, v), ELE_VERT_ENDPOINT);
        }
        v_edges_max = v_edges_max.max(v_edges_num);
    }
    NEW_VERT.with(|s| *s.borrow_mut() = new_verts.clone());
    for &v in &verts {
        for l in crate::iterators::LoopsOfVert::new(bm, v).collect::<Vec<_>>() {
            let f = bm.loo(l).f;
            if bm.face(f).head.hflag & crate::BM_ELEM_TAG == 0 || bm.face(f).len == 3 {
                continue;
            }
            let (ln, lp) = (bm.loo(l).next, bm.loo(l).prev);
            let nv = |lv: usize| new_verts.contains(&bm.loo(lv).v);
            if !nv(ln) && !nv(lp) {
                if use_cap_endpoint || !bmo_elem_flag_test(bm, (BM_VERT, v), ELE_VERT_ENDPOINT) {
                    let mut l_new = BM_NIL;
                    if !crate::core::bm_face_split_check_double_face(bm, lp, ln, 3)
                        && crate::mods::bm_face_split(bm, f, lp, ln, Some(&mut l_new), true) != BM_NIL
                    {
                        bmo_elem_flag_enable(bm, (BM_EDGE, bm.loo(l_new).e), ELE_NEW);
                    }
                }
            } else if bm.face(f).len > 4 {
                if (bm.edge(bm.loo(l).e).head.hflag & crate::BM_ELEM_TAG != 0)
                    != (bm.edge(bm.loo(lp).e).head.hflag & crate::BM_ELEM_TAG != 0)
                {
                    if nv(ln) {
                        let lpp = bm.loo(lp).prev;
                        if nv(lpp) {
                            let mut l_new = BM_NIL;
                            if !crate::core::bm_face_split_check_double_face(bm, lpp, ln, 4)
                                && crate::mods::bm_face_split(
                                    bm, f, lpp, ln, Some(&mut l_new), true) != BM_NIL
                            {
                                bmo_elem_flag_enable(bm, (BM_EDGE, bm.loo(l_new).e), ELE_NEW);
                                bm.fpool[f].head.hflag &= !crate::BM_ELEM_TAG;
                            }
                        } else {
                            let mut l_new = BM_NIL;
                            bm_face_split_walk_back(bm, l, &mut l_new);
                            loop {
                                bmo_elem_flag_enable(bm, (BM_EDGE, bm.loo(l_new).e), ELE_NEW);
                                l_new = bm.loo(l_new).next;
                                if !crate::query::bm_vert_is_edge_pair(bm, bm.loo(l_new).v) {
                                    break;
                                }
                            }
                            bm.fpool[f].head.hflag &= !crate::BM_ELEM_TAG;
                        }
                    }
                }
            }
        }
    }
    if !use_cap_endpoint {
        let mut varr: Vec<usize> = Vec::with_capacity(v_edges_max);
        for &v in &verts {
            varr.clear();
            for e in crate::iterators::EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
                let v_other = crate::structure::bm_edge_other_vert(bm, e, v);
                if new_verts.contains(&v_other)
                    && crate::query::bm_vert_is_edge_pair(bm, v_other)
                {
                    bm.vpool[v_other].e = e;
                    varr.push(v_other);
                }
            }
            for vo in varr.clone() {
                let e = bm.vert(vo).e;
                crate::core::bmesh_kernel_join_edge_kill_vert(bm, e, vo, true, false, false, true);
            }
        }
    }
    NEW_VERT.with(|s| s.borrow_mut().clear());
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "edges.out", BM_EDGE, ELE_NEW);
}
