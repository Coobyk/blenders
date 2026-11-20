//! Port of `bmesh_mesh_validate.cc` — `BM_mesh_is_valid` debug check.
//! Reports structural errors to stderr; returns true when none are found.

use crate::structure::bm_vert_in_edge;
use crate::{BMesh, BM_EDGE, BM_ELEM_HIDDEN, BM_ELEM_INTERNAL_TAG, BM_ELEM_SELECT, BM_FACE, BM_LOOP,
            BM_NIL, BM_VERT};
use std::collections::HashMap;

/// `BM_mesh_is_valid` — full structural sanity walk. Debug aid; prints
/// every anomaly found and returns whether the mesh is clean.
pub fn bm_mesh_is_valid(bm: &mut BMesh) -> bool {
    let mut errtot = 0;
    eprintln!();
    eprintln!("This is a debugging function and not intended for general use, running slow test!");

    /* force recalc */
    bm.elem_index_dirty = true;
    crate::mesh::bm_mesh_elem_index_ensure(bm, (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_LOOP) | (1 << BM_FACE));

    /* verts */
    for i in 0..bm.vpool.len() {
        if bm.vert(i).head.htype != BM_VERT {
            continue;
        }
        let v = bm.vert(i);
        if (v.head.hflag & (BM_ELEM_SELECT | BM_ELEM_HIDDEN)) == (BM_ELEM_SELECT | BM_ELEM_HIDDEN) {
            eprintln!("vert {i}: is hidden and selected");
            errtot += 1;
        }
        if v.e != BM_NIL && !bm_vert_in_edge(bm, v.e, i) {
            eprintln!("vert {i}: is not in its referenced edge: {}", bm.edge(v.e).head.index);
            errtot += 1;
        }
    }

    /* edges: self-loops + duplicate detection */
    let mut edge_hash: HashMap<(i32, i32), usize> = HashMap::with_capacity(bm.totedge as usize);
    for i in 0..bm.epool.len() {
        if bm.edge(i).head.htype != BM_EDGE {
            continue;
        }
        let e = bm.edge(i);
        if e.v1 == e.v2 {
            eprintln!("edge {i}: duplicate index: {}", bm.vert(e.v1).head.index);
            errtot += 1;
        }
        let mut key = (bm.vert(e.v1).head.index, bm.vert(e.v2).head.index);
        if key.0 > key.1 {
            key = (key.1, key.0);
        }
        if let Some(&other) = edge_hash.get(&key) {
            eprintln!("edge {i}, {}: are duplicates", bm.edge(other).head.index);
            errtot += 1;
        } else {
            edge_hash.insert(key, i);
        }
    }

    /* edge radial structure */
    for i in 0..bm.epool.len() {
        if bm.edge(i).head.htype != BM_EDGE {
            continue;
        }
        let e = bm.edge(i);
        if (e.head.hflag & (BM_ELEM_SELECT | BM_ELEM_HIDDEN)) == (BM_ELEM_SELECT | BM_ELEM_HIDDEN) {
            eprintln!("edge {i}: is hidden and selected");
            errtot += 1;
        }
        if e.l != BM_NIL {
            let l_first = e.l;
            let mut l_iter = l_first;
            loop {
                let l = bm.loo(l_iter);
                if l.e != i {
                    eprintln!(
                        "edge {i}: has invalid loop, loop is of face {}",
                        bm.face(l.f).head.index
                    );
                    errtot += 1;
                } else if !bm_vert_in_edge(bm, i, l.v) {
                    eprintln!(
                        "edge {i}: has invalid loop with vert not in edge, loop is of face {}",
                        bm.face(l.f).head.index
                    );
                    errtot += 1;
                } else if !bm_vert_in_edge(bm, i, bm.loo(l.next).v) {
                    eprintln!(
                        "edge {i}: has invalid loop with next vert not in edge, loop is of face {}",
                        bm.face(l.f).head.index
                    );
                    errtot += 1;
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
            }
        }
    }

    /* faces */
    let mut face_map: HashMap<Vec<i32>, i32> = HashMap::new();
    for i in 0..bm.fpool.len() {
        if bm.face(i).head.htype != BM_FACE {
            continue;
        }
        if (bm.face(i).head.hflag & (BM_ELEM_SELECT | BM_ELEM_HIDDEN))
            == (BM_ELEM_SELECT | BM_ELEM_HIDDEN)
        {
            eprintln!("face {i}: is hidden and selected");
            errtot += 1;
        }

        let l_first = bm.face(i).l_first;
        /* clear tags + measure */
        let mut j = 0;
        let mut l_iter = l_first;
        loop {
            bm.lpool[l_iter].head.hflag &= !BM_ELEM_INTERNAL_TAG;
            let lv = bm.loo(l_iter).v;
            let le = bm.loo(l_iter).e;
            bm.vpool[lv].head.hflag &= !BM_ELEM_INTERNAL_TAG;
            bm.epool[le].head.hflag &= !BM_ELEM_INTERNAL_TAG;
            j += 1;
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        if j != bm.face(i).len {
            eprintln!("face {i}: has length of {} but should be {}", bm.face(i).len, j);
            errtot += 1;
        }

        /* per-corner checks + min-vert loop */
        let mut l_vert_min = l_first;
        let mut corner = 0;
        l_iter = l_first;
        loop {
            if (bm.loo(l_iter).head.hflag & BM_ELEM_INTERNAL_TAG) != 0 {
                eprintln!("face {i}: has duplicate loop at corner: {corner}");
                errtot += 1;
            }
            let lv = bm.loo(l_iter).v;
            let le = bm.loo(l_iter).e;
            if (bm.vert(lv).head.hflag & BM_ELEM_INTERNAL_TAG) != 0 {
                eprintln!(
                    "face {i}: has duplicate vert: {}, at corner: {corner}",
                    bm.vert(lv).head.index
                );
                errtot += 1;
            }
            if (bm.edge(le).head.hflag & BM_ELEM_INTERNAL_TAG) != 0 {
                eprintln!(
                    "face {i}: has duplicate edge: {}, at corner: {corner}",
                    bm.edge(le).head.index
                );
                errtot += 1;
            }
            let l = bm.loo(l_iter);
            if l.f != i {
                eprintln!(
                    "face {i}: has loop that points to face: {} at corner: {corner}",
                    bm.face(l.f).head.index
                );
                errtot += 1;
            }
            if bm.loo(l.prev).next != l_iter {
                eprintln!("face {i}: has invalid 'prev/next' at corner: {corner}");
                errtot += 1;
            }
            if bm.loo(l.next).prev != l_iter {
                eprintln!("face {i}: has invalid 'next/prev' at corner: {corner}");
                errtot += 1;
            }
            if bm.loo(l.radial_prev).radial_next != l_iter {
                eprintln!("face {i}: has invalid 'radial_prev/radial_next' at corner: {corner}");
                errtot += 1;
            }
            if bm.loo(l.radial_next).radial_prev != l_iter {
                eprintln!("face {i}: has invalid 'radial_next/radial_prev' at corner: {corner}");
                errtot += 1;
            }

            bm.lpool[l_iter].head.hflag |= BM_ELEM_INTERNAL_TAG;
            bm.vpool[lv].head.hflag |= BM_ELEM_INTERNAL_TAG;
            bm.epool[le].head.hflag |= BM_ELEM_INTERNAL_TAG;

            if bm.vert(lv).head.index < bm.vert(bm.loo(l_vert_min).v).head.index {
                l_vert_min = l_iter;
            }
            corner += 1;
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }

        /* ordered vert indices from lowest index loop (winding-agnostic) */
        let mut face_verts: Vec<i32> = Vec::with_capacity(j as usize);
        let lv_min = bm.loo(l_vert_min).v;
        let next_idx = bm.vert(bm.loo(bm.loo(l_vert_min).next).v).head.index;
        let prev_idx = bm.vert(bm.loo(bm.loo(l_vert_min).prev).v).head.index;
        let _ = lv_min;
        if next_idx < prev_idx {
            let mut li = l_vert_min;
            loop {
                face_verts.push(bm.vert(bm.loo(li).v).head.index);
                li = bm.loo(li).next;
                if li == l_vert_min {
                    break;
                }
            }
        } else {
            let mut li = l_vert_min;
            loop {
                face_verts.push(bm.vert(bm.loo(li).v).head.index);
                li = bm.loo(li).prev;
                if li == l_vert_min {
                    break;
                }
            }
        }
        if let Some(&dup) = face_map.get(&face_verts) {
            eprintln!("face {i}: duplicate of {dup}");
            errtot += 1;
        } else {
            face_map.insert(face_verts, i as i32);
        }

        /* untag */
        l_iter = l_first;
        loop {
            bm.lpool[l_iter].head.hflag &= !BM_ELEM_INTERNAL_TAG;
            let lv = bm.loo(l_iter).v;
            let le = bm.loo(l_iter).e;
            bm.vpool[lv].head.hflag &= !BM_ELEM_INTERNAL_TAG;
            bm.epool[le].head.hflag &= !BM_ELEM_INTERNAL_TAG;
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }

    eprintln!("Finished - errors {errtot}");
    errtot == 0
}

/// loop type const to satisfy import (loop elements aren't visited above).
const _: u8 = BM_LOOP;
