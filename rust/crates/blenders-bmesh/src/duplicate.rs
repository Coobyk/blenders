//! Port of `bmesh_mesh_duplicate.cc` — copy element arrays between meshes.
//! CustomData attr copying (`BM_elem_attrs_copy`/`BMCustomDataCopyMap`)
//! is PORT_PENDING (CustomData unported); elements copy hflag + coords.

use crate::core::*;
use crate::{BMesh, BM_EDGE, BM_NIL, BM_VERT};

/// `BM_mesh_copy_arrays` — copy `verts_src`, `edges_src`, `faces_src`
/// from `bm_src` into `bm_dst` (may be the same mesh for a duplicate op).
/// `verts_dst`/`edges_dst` are filled with the new elements (index-aligned
/// via `head.index`).
pub fn bm_mesh_copy_arrays(
    bm_src: &mut BMesh,
    bm_dst: &mut BMesh,
    verts_src: &[usize],
    edges_src: &[usize],
    faces_src: &[usize],
) {
    /* verts */
    let mut verts_dst = vec![BM_NIL; verts_src.len()];
    for (i, &v_src) in verts_src.iter().enumerate() {
        bm_src.vpool[v_src].head.index = i as i32;
        let co = bm_src.vert(v_src).co;
        let no = bm_src.vert(v_src).no;
        let hflag = bm_src.vert(v_src).head.hflag;
        let v_dst = bm_vert_create(bm_dst, co, None, 0);
        bm_dst.vpool[v_dst].no = no;
        bm_dst.vpool[v_dst].head.hflag = hflag;
        bm_dst.vpool[v_dst].head.index = i as i32;
        verts_dst[i] = v_dst;
    }
    bm_src.elem_index_dirty = true;

    /* edges */
    let mut edges_dst = vec![BM_NIL; edges_src.len()];
    for (i, &e_src) in edges_src.iter().enumerate() {
        bm_src.epool[e_src].head.index = i as i32;
        let (v1, v2) = (bm_src.edge(e_src).v1, bm_src.edge(e_src).v2);
        let d1 = verts_dst[bm_src.vert(v1).head.index as usize];
        let d2 = verts_dst[bm_src.vert(v2).head.index as usize];
        let hflag = bm_src.edge(e_src).head.hflag;
        let e_dst = bm_edge_create(bm_dst, d1, d2, None, 0);
        bm_dst.epool[e_dst].head.hflag = hflag;
        bm_dst.epool[e_dst].head.index = i as i32;
        edges_dst[i] = e_dst;
    }
    bm_src.elem_index_dirty = true;

    /* faces */
    for (i, &f_src) in faces_src.iter().enumerate() {
        let f_len = bm_src.face(f_src).len;
        let mut vtar = Vec::with_capacity(f_len as usize);
        let mut edar = Vec::with_capacity(f_len as usize);
        let l_first = bm_src.face(f_src).l_first;
        let mut l = l_first;
        loop {
            let lv = bm_src.loo(l).v;
            let le = bm_src.loo(l).e;
            vtar.push(verts_dst[bm_src.vert(lv).head.index as usize]);
            edar.push(edges_dst[bm_src.edge(le).head.index as usize]);
            l = bm_src.loo(l).next;
            if l == l_first {
                break;
            }
        }
        let hflag = bm_src.face(f_src).head.hflag;
        let mat_nr = bm_src.face(f_src).mat_nr;
        let no = bm_src.face(f_src).no;
        let f_dst = bm_face_create(bm_dst, &vtar, &edar, Some(f_src), 0);
        bm_dst.fpool[f_dst].head.hflag = hflag;
        bm_dst.fpool[f_dst].mat_nr = mat_nr;
        bm_dst.fpool[f_dst].no = no;
        bm_dst.fpool[f_dst].head.index = i as i32;
    }
}
