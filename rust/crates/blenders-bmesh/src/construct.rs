//! Port of `bmesh_construct.cc` — mesh construction helpers.
//!
//! `BM_elem_attrs_copy` / `BM_mesh_copy_init_customdata*` / `BM_mesh_copy`
//! CustomData plumbing is PORT_PENDING (CustomData unported); the flag-copy
//! portions are ported.

use crate::core::*;
use crate::structure::*;
use crate::{BMesh, BMCreateFlag, BM_NIL};
use crate::{BM_EDGE, BM_FACE, BM_LOOP, BM_VERT};
use blenders_math::vector::Vec3;

/// `BM_verts_from_edges` — fill vert array from edge array (v1 of each).
pub fn bm_verts_from_edges(bm: &BMesh, edge_arr: &[usize]) -> Vec<usize> {
    edge_arr.iter().map(|&e| bm.edge(e).v1).collect()
}

/// `BM_edges_from_verts` — fill edge array from vert array.
pub fn bm_edges_from_verts(bm: &BMesh, vert_arr: &[usize]) -> Vec<usize> {
    let len = vert_arr.len();
    (0..len)
        .map(|i| {
            bmesh_disk_edge_exists(bm, vert_arr[i], vert_arr[(i + 1) % len])
        })
        .collect()
}

/// `BM_edges_from_verts_ensure` — like above, creating missing edges.
pub fn bm_edges_from_verts_ensure(
    bm: &mut BMesh,
    vert_arr: &[usize],
) -> Vec<usize> {
    let len = vert_arr.len();
    (0..len)
        .map(|i| {
            let v1 = vert_arr[i];
            let v2 = vert_arr[(i + 1) % len];
            let mut e = bmesh_disk_edge_exists(bm, v1, v2);
            if e == BM_NIL {
                e = bm_edge_create(bm, v1, v2, None, 0);
            }
            e
        })
        .collect()
}

/// `BM_face_create_quad_tri` — create a quad or tri face; v4 == BM_NIL gives tri.
pub fn bm_face_create_quad_tri(
    bm: &mut BMesh,
    v1: usize,
    v2: usize,
    v3: usize,
    v4: usize,
    f_example: Option<usize>,
    create_flag: BMCreateFlag,
) -> usize {
    let _ = f_example;
    let _ = create_flag;
    let verts: Vec<usize> = if v4 != BM_NIL {
        vec![v1, v2, v3, v4]
    } else {
        vec![v1, v2, v3]
    };
    bm_face_create_verts(bm, &verts, None, create_flag, true, true)
}

/// `BM_face_create_ngon` — create an n-gon face from unordered vert list,
/// using `edges_arr` order hints.
pub fn bm_face_create_ngon(
    bm: &mut BMesh,
    v1: usize,
    v2: usize,
    edges_arr: &[usize],
    f_example: Option<usize>,
    create_flag: BMCreateFlag,
) -> usize {
    /* Reproduce C's winding walk: start at v1, follow edges where one vert
     * is already placed. */
    let len = edges_arr.len() + 1;
    let mut verts: Vec<usize> = vec![BM_NIL; len];
    let mut edges: Vec<usize> = Vec::with_capacity(len);
    verts[0] = v1;
    verts[1] = v2;
    let mut v = v2;
    for _i in 0..edges_arr.len() {
        /* find edge in edges_arr incident on v not yet used */
        let mut found = BM_NIL;
        let mut v_next = BM_NIL;
        for &e in edges_arr {
            if edges.contains(&e) {
                continue;
            }
            let (a, b) = (bm.edge(e).v1, bm.edge(e).v2);
            if a == v {
                found = e;
                v_next = b;
                break;
            } else if b == v {
                found = e;
                v_next = a;
                break;
            }
        }
        if found == BM_NIL {
            return BM_NIL;
        }
        edges.push(found);
        v = v_next;
    }
    /* Place verts in order along the edge chain. */
    let mut ordered: Vec<usize> = vec![v1];
    let mut cur = v1;
    for &e in &edges {
        let (a, b) = (bm.edge(e).v1, bm.edge(e).v2);
        let nxt = if a == cur { b } else if b == cur { a } else {
            return BM_NIL;
        };
        ordered.push(nxt);
        cur = nxt;
    }
    /* drop the duplicate closing vert */
    if ordered.len() > 1 && ordered.last() == ordered.first() {
        ordered.pop();
    }
    bm_face_create(bm, &ordered, &edges, f_example, create_flag)
}

/// `BM_face_create_ngon_verts` — n-gon from ordered verts.
pub fn bm_face_create_ngon_verts(
    bm: &mut BMesh,
    vert_arr: &[usize],
    f_example: Option<usize>,
    create_flag: BMCreateFlag,
) -> usize {
    bm_face_create_verts(bm, vert_arr, f_example, create_flag, true, true)
}

/// `BM_verts_sort_radial_plane` — sort verts radially by angle around a plane.
/// C uses `poly_plane_fit` + polar angle sort.
pub fn bm_verts_sort_radial_plane(bm: &BMesh, vert_arr: &mut [usize]) {
    /* Compute centroid. */
    let mut cent = Vec3::new(0.0, 0.0, 0.0);
    for &v in vert_arr.iter() {
        cent = cent + bm.vert(v).co;
    }
    let n = vert_arr.len().max(1) as f32;
    cent = cent / n;
    /* Basis: first vert direction as X axis; normal from area. */
    let mut nrm = Vec3::new(0.0, 0.0, 0.0);
    for i in 0..vert_arr.len() {
        let a = bm.vert(vert_arr[i]).co - cent;
        let b = bm.vert(vert_arr[(i + 1) % vert_arr.len()]).co - cent;
        nrm = nrm + a.cross(b);
    }
    if nrm.length_squared() < 1e-20 {
        return;
    }
    let mut nrm = nrm;
    nrm.normalize();
    let mut x_axis = bm.vert(vert_arr[0]).co - cent;
    x_axis = x_axis - nrm * x_axis.dot(nrm);
    if x_axis.length_squared() < 1e-20 {
        return;
    }
    x_axis.normalize();
    let y_axis = nrm.cross(x_axis);
    vert_arr.sort_by(|&a, &b| {
        let pa = bm.vert(a).co - cent;
        let pb = bm.vert(b).co - cent;
        let aa = pa.dot(y_axis).atan2(pa.dot(x_axis));
        let bb = pb.dot(y_axis).atan2(pb.dot(x_axis));
        aa.partial_cmp(&bb).unwrap_or(std::cmp::Ordering::Equal)
    });
}

/* ---- attr copy (flag portion; CustomData PORT_PENDING) ----------------- */

/// `BM_elem_select_copy` — copy select/hide flags src → dst.
pub fn bm_elem_select_copy(bm_dst: &mut BMesh, htype: u8, ele_dst: usize, ele_src_flags: (u8, u8)) {
    let (hflag, api_flag) = ele_src_flags;
    match htype {
        BM_VERT => {
            bm_dst.vpool[ele_dst].head.hflag = hflag;
            bm_dst.vpool[ele_dst].head.api_flag = api_flag;
        }
        BM_EDGE => {
            bm_dst.epool[ele_dst].head.hflag = hflag;
            bm_dst.epool[ele_dst].head.api_flag = api_flag;
        }
        BM_FACE => {
            bm_dst.fpool[ele_dst].head.hflag = hflag;
            bm_dst.fpool[ele_dst].head.api_flag = api_flag;
        }
        BM_LOOP => {
            bm_dst.lpool[ele_dst].head.hflag = hflag;
            bm_dst.lpool[ele_dst].head.api_flag = api_flag;
        }
        _ => debug_assert!(false),
    }
}

/// `BM_mesh_copy` — duplicate mesh structure (no CustomData).
pub fn bm_mesh_copy(bm_old: &BMesh) -> BMesh {
    let mut bm = BMesh::new();
    bm.elem_table_dirty = true;
    /* verts */
    let mut v_map: Vec<usize> = vec![BM_NIL; bm_old.vpool.len()];
    for (i, v) in bm_old.vpool.iter().enumerate() {
        if v.head.htype != BM_VERT {
            continue;
        }
        let nv = bm_vert_create(&mut bm, v.co, None, 0);
        bm.vpool[nv].no = v.no;
        bm.vpool[nv].head.hflag = v.head.hflag;
        bm.vpool[nv].head.api_flag = v.head.api_flag;
        v_map[i] = nv;
    }
    let mut e_map: Vec<usize> = vec![BM_NIL; bm_old.epool.len()];
    for (i, e) in bm_old.epool.iter().enumerate() {
        if e.head.htype != BM_EDGE {
            continue;
        }
        let ne = bm_edge_create(&mut bm, v_map[e.v1], v_map[e.v2], None, 0);
        bm.epool[ne].head.hflag = e.head.hflag;
        bm.epool[ne].head.api_flag = e.head.api_flag;
        e_map[i] = ne;
    }
    for f in bm_old.fpool.iter() {
        if f.head.htype != BM_FACE {
            continue;
        }
        let l_first = f.l_first;
        let mut l_iter = l_first;
        let mut verts: Vec<usize> = Vec::new();
        let mut edges: Vec<usize> = Vec::new();
        loop {
            verts.push(v_map[bm_old.loo(l_iter).v]);
            edges.push(e_map[bm_old.loo(l_iter).e]);
            l_iter = bm_old.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        let nf = bm_face_create(&mut bm, &verts, &edges, None, 0);
        bm.fpool[nf].no = f.no;
        bm.fpool[nf].head.hflag = f.head.hflag;
        bm.fpool[nf].head.api_flag = f.head.api_flag;
        bm.fpool[nf].mat_nr = f.mat_nr;
        /* `BM_ELEM_SMOOTH` rides in hflag (already copied). */
    }
    bm
}
