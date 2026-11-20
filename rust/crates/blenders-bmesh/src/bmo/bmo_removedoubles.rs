//! Port of `bmo_removedoubles.cc` — `weld_verts` (targetmap weld).
//! `find_doubles`/`remove_doubles` (BVH-based) and the edit-selection
//! remap are PORT_PENDING; `average_vert_data` interp is a no-op
//! (CustomData).

use std::collections::HashMap;
use crate::operators::*;
use crate::{BMesh, BMEdge, BMFace, BMVert, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};

const ELE_DEL: u8 = 1;
const EDGE_COL: u8 = 2;
const VERT_IN_FACE: u8 = 4;

/// `remdoubles_splitface` — split faces whose loops jump across a
/// welded vert.
fn remdoubles_splitface(
    bm: &mut BMesh,
    f: usize,
    op: &BMOperator,
    slot_targetmap: &str,
) {
    let mut split = false;
    let mut l_double = BM_NIL;
    let mut l_tar = BM_NIL;
    for l in crate::iterators::LoopsOfFace::new(bm, f) {
        let lv = bm.loo(l).v;
        if let Some((ht, v_tar)) = bmo_slot_map_get(op, false, slot_targetmap, (BM_VERT, lv)) {
            if ht == BM_VERT {
                let lt = crate::query::bm_face_vert_share_loop(bm, f, v_tar);
                if lt != BM_NIL && lt != l && !crate::mods::bm_loop_is_adjacent(bm, lt, l) {
                    l_double = l;
                    l_tar = lt;
                    split = true;
                    break;
                }
            }
        }
    }
    if split {
        let f_new = crate::mods::bm_face_split(bm, f, l_double, l_tar, None, false);
        remdoubles_splitface(bm, f, op, slot_targetmap);
        if f_new != BM_NIL {
            remdoubles_splitface(bm, f_new, op, slot_targetmap);
        }
    }
}

/// `remdoubles_createface` — rebuild a face with welded verts swapped in.
/// Returns (face index, created).
fn remdoubles_createface(
    bm: &mut BMesh,
    f: usize,
    op: &BMOperator,
    slot_targetmap: &str,
) -> (usize, bool) {
    let f_len = bm.face(f).len as usize;
    let mut edges: Vec<usize> = Vec::with_capacity(f_len);
    let mut loops: Vec<usize> = Vec::with_capacity(f_len);
    let mut verts: Vec<usize> = Vec::with_capacity(f_len);
    let mut failed = false;

    let map_vert = |bm: &BMesh, v: usize| -> (usize, bool) {
        let is_del = bmo_elem_flag_test(bm, (BM_VERT, v), ELE_DEL);
        if is_del {
            if let Some((_, vt)) = bmo_slot_map_get(op, false, slot_targetmap, (BM_VERT, v)) {
                return (vt, true);
            }
        }
        (v, is_del)
    };

    let l_first = bm.face(f).l_first;
    let mut l_curr = l_first;
    let (mut v_curr, mut is_del_v_curr) = map_vert(bm, bm.loo(l_curr).v);
    loop {
        let l_next = bm.loo(l_curr).next;
        let (v_next, is_del_v_next) = map_vert(bm, bm.loo(l_next).v);
        let mut e_new = BM_NIL;
        if !is_del_v_curr && !is_del_v_next {
            e_new = bm.loo(l_curr).e;
        } else if v_curr == v_next {
            e_new = BM_NIL; /* skip */
        } else {
            e_new = crate::query::bm_edge_exists(bm, v_curr, v_next);
            debug_assert!(e_new != BM_NIL);
        }
        if e_new != BM_NIL {
            if bmo_elem_flag_test(bm, (BM_VERT, v_curr), VERT_IN_FACE) {
                failed = true;
                break;
            }
            bmo_elem_flag_enable(bm, (BM_VERT, v_curr), VERT_IN_FACE);
            edges.push(e_new);
            loops.push(l_curr);
            verts.push(v_curr);
        }
        v_curr = v_next;
        is_del_v_curr = is_del_v_next;
        l_curr = l_next;
        if l_curr == l_first { break; }
    }
    for &v in &verts {
        bmo_elem_flag_disable(bm, (BM_VERT, v), VERT_IN_FACE);
    }
    if failed || verts.len() < 3 {
        return (BM_NIL, false);
    }
    /* check whether the new face would equal the old one */
    if edges.len() == f_len && !failed {
        let mut identical = true;
        for (i, &l) in loops.iter().enumerate() {
            if bm.loo(l).v != verts[i] || bm.loo(l).e != edges[i] {
                identical = false;
                break;
            }
        }
        if identical {
            return (f, false);
        }
    }
    let f_new = crate::core::bm_face_create(bm, &verts, &edges, Some(f), 0);
    (f_new, f_new != BM_NIL)
}

/// `bmo_weld_verts_exec`.
pub fn bmo_weld_verts_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let slot_targetmap = "targetmap";
    let use_centroid = bmo_slot_bool_get(op, false, "use_centroid");
    let _average_vert_data = bmo_slot_bool_get(op, false, "average_vert_data") || use_centroid;

    /* mark welded-away verts */
    let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        let v_dst = match bmo_slot_map_get(op, false, slot_targetmap, (BM_VERT, v)) {
            Some((_, vd)) => vd,
            None => continue,
        };
        bmo_elem_flag_enable(bm, (BM_VERT, v), ELE_DEL);
        let fl = bm.vert(v).head.hflag & crate::BM_ELEM_HIDDEN;
        bm.vpool[v_dst].head.hflag |= fl;
        if _average_vert_data && v_dst != v {
            clusters.entry(v_dst).or_default().push(v);
        }
    }
    if use_centroid {
        for (v_dst, cluster) in &clusters {
            let mut centroid = bm.vert(*v_dst).co;
            let mut count = 1;
            for &v_dup in cluster {
                centroid = centroid + bm.vert(v_dup).co;
                count += 1;
            }
            bm.vpool[*v_dst].co = centroid / count as f32;
        }
    }
    /* split faces that need it */
    for f in crate::iterators::FacesOfMesh::new(bm).collect::<Vec<_>>() {
        remdoubles_splitface(bm, f, op, slot_targetmap);
    }
    /* rewire edges around welded verts */
    for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        let mut v1 = bm.edge(e).v1;
        let mut v2 = bm.edge(e).v2;
        let is_del_v1 = bmo_elem_flag_test(bm, (BM_VERT, v1), ELE_DEL);
        let is_del_v2 = bmo_elem_flag_test(bm, (BM_VERT, v2), ELE_DEL);
        if is_del_v1 || is_del_v2 {
            if is_del_v1 {
                if let Some((_, vt)) = bmo_slot_map_get(op, false, slot_targetmap, (BM_VERT, v1)) {
                    v1 = vt;
                }
            }
            if is_del_v2 {
                if let Some((_, vt)) = bmo_slot_map_get(op, false, slot_targetmap, (BM_VERT, v2)) {
                    v2 = vt;
                }
            }
            if v1 == v2 {
                bmo_elem_flag_enable(bm, (BM_EDGE, e), EDGE_COL);
            } else {
                let mut e_new = crate::query::bm_edge_exists(bm, v1, v2);
                if e_new == BM_NIL {
                    e_new = crate::core::bm_edge_create(bm, v1, v2, Some(e), 0);
                }
                let fl = bm.edge(e).head.hflag & crate::BM_ELEM_HIDDEN;
                bm.epool[e_new].head.hflag |= fl;
            }
            bmo_elem_flag_enable(bm, (BM_EDGE, e), ELE_DEL);
        }
    }
    /* faces: rebuild or mark deleted */
    for f in crate::iterators::FacesOfMesh::new(bm).collect::<Vec<_>>() {
        let mut vert_delete = false;
        let mut edge_collapse = 0;
        for l in crate::iterators::LoopsOfFace::new(bm, f) {
            if bmo_elem_flag_test(bm, (BM_VERT, bm.loo(l).v), ELE_DEL) {
                vert_delete = true;
            }
            if bmo_elem_flag_test(bm, (BM_EDGE, bm.loo(l).e), EDGE_COL) {
                edge_collapse += 1;
            }
        }
        if vert_delete {
            let mut use_in_place = false;
            let mut f_new = BM_NIL;
            bmo_elem_flag_enable(bm, (BM_FACE, f), ELE_DEL);
            if bm.face(f).len as i32 - edge_collapse >= 3 {
                let (nf, created) = remdoubles_createface(bm, f, op, slot_targetmap);
                f_new = nf;
                if f_new != BM_NIL {
                    if created && f_new != f {
                        /* bmesh_face_swap_data — swap geometry so the
                         * welded face keeps f's slot (oflags preserved) */
                        crate::core::bmesh_face_swap_data(bm, f_new, f);
                        bmo_elem_flag_disable(bm, (BM_FACE, f), ELE_DEL);
                        crate::core::bm_face_kill(bm, f_new);
                        use_in_place = true;
                    } else {
                        let fl = bm.face(f).head.hflag & crate::BM_ELEM_HIDDEN;
                        bm.fpool[f_new].head.hflag |= fl;
                    }
                }
            }
            if !use_in_place && f_new != BM_NIL {
                if bm.act_face == f {
                    bm.act_face = f_new;
                }
            }
        }
    }
    crate::delete::bmo_mesh_delete_oflag_context(bm, ELE_DEL, crate::delete::DEL_ONLYTAGGED, None);
}

/// `bmo_reverse_faces_exec` (from `bmo_utils.cc`).
pub fn bmo_reverse_faces_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let _use_loop_mdisp_flip = bmo_slot_bool_get(op, false, "flip_multires");
    let faces: Vec<usize> = bmo_slot_buffer(op, false, "faces")
        .iter().filter(|(ht, _)| *ht == BM_FACE).map(|&(_, i)| i).collect();
    for f in faces {
        crate::polygon::bm_face_normal_flip(bm, f);
        /* CD_MDISPS flip PORT_PENDING */
    }
}
