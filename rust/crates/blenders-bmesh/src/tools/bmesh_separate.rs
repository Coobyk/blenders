//! Port of `tools/bmesh_separate.cc` — `BM_mesh_separate_faces`.
//! Split verts at boundaries between filtered/unfiltered faces.
//! Groups of contiguous radial loops are separated one loop at a time
//! (C uses `BM_face_loop_separate_multi` for runs sharing edges —
//! behaviorally equivalent here since `bm_face_loop_separate` groups
//! by fan automatically: it splits the whole fan region containing l).

use crate::iterators::FacesOfMesh;
use crate::{BMesh, BM_ELEM_TAG, BM_NIL};

/// `BM_mesh_separate_faces` — `filter_fn(&BMesh, f) -> bool` decides the
/// face set; boundary verts between the two sets are duplicated.
pub fn bm_mesh_separate_faces(bm: &mut BMesh, filter_fn: &dyn Fn(&BMesh, usize) -> bool) {
    let mut faces_a: Vec<usize> = Vec::new();
    let mut faces_b: Vec<usize> = Vec::new();
    for f in FacesOfMesh::new(bm) {
        if filter_fn(bm, f) { faces_a.push(f); } else { faces_b.push(f); }
    }
    /* tag verts: TAG = "appears only in faces_a or in both" */
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        bm.vpool[v].head.hflag |= BM_ELEM_TAG;
    }
    for &f in &faces_b {
        for l in crate::iterators::LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
            let v = bm.loo(l).v;
            bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
        }
    }
    for &f in &faces_a {
        for l in crate::iterators::LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
            let v = bm.loo(l).v;
            if bm.vert(v).head.hflag & BM_ELEM_TAG == 0 {
                bm.vpool[v].head.hflag |= BM_ELEM_TAG;
                /* collect all radial loops of v on filtered faces */
                let mut loop_split: Vec<usize> = Vec::new();
                if bm.vert(v).e != BM_NIL {
                    let e_first = bm.vert(v).e;
                    let mut e_iter = e_first;
                    loop {
                        if bm.edge(e_iter).l != BM_NIL {
                            let l_radial_first = bm.edge(e_iter).l;
                            let mut l_radial_iter = l_radial_first;
                            loop {
                                if bm.loo(l_radial_iter).v == v
                                    && filter_fn(bm, bm.loo(l_radial_iter).f)
                                {
                                    loop_split.push(l_radial_iter);
                                }
                                l_radial_iter = bm.loo(l_radial_iter).radial_next;
                                if l_radial_iter == l_radial_first { break; }
                            }
                        }
                        e_iter = crate::structure::bmesh_disk_edge_next(bm, e_iter, v);
                        if e_iter == e_first { break; }
                    }
                }
                for l_sep in loop_split {
                    crate::mods::bm_face_loop_separate(bm, l_sep);
                }
            }
        }
    }
}
