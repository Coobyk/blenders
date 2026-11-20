//! Port of `bmesh_mesh_partial_update.cc` — `BMPartialUpdate` bookkeeping
//! for re-running normals/tessellation on a sub-region of the mesh.

use crate::structure::bmesh_disk_edge_next;
use crate::{BMesh, BM_FACE, BM_NIL, BM_VERT};

/// `BMPartialUpdate_Params`.
#[derive(Clone, Copy, Default)]
pub struct BMPartialUpdateParams {
    pub do_normals: bool,
    pub do_tessellate: bool,
}

/// `BMPartialUpdate`.
pub struct BMPartialUpdate {
    pub verts: Vec<usize>,
    pub faces: Vec<usize>,
    pub params: BMPartialUpdateParams,
}

fn partial_elem_vert_ensure(bmp: &mut BMPartialUpdate, tag: &mut [bool], v: usize) {
    if !tag[v] {
        tag[v] = true;
        bmp.verts.push(v);
    }
}
fn partial_elem_face_ensure(bmp: &mut BMPartialUpdate, tag: &mut [bool], f: usize) {
    if !tag[f] {
        tag[f] = true;
        bmp.faces.push(f);
    }
}

/// `BM_mesh_partial_create_from_verts`.
pub fn bm_mesh_partial_create_from_verts(
    bm: &mut BMesh,
    params: BMPartialUpdateParams,
    verts_mask: &[bool],
) -> BMPartialUpdate {
    let mut bmp = BMPartialUpdate {
        verts: Vec::with_capacity(verts_mask.iter().filter(|&&b| b).count()),
        faces: Vec::new(),
        params,
    };
    crate::mesh::bm_mesh_elem_index_ensure(bm, 1 << crate::BM_FACE);

    if params.do_normals || params.do_tessellate {
        let mut faces_tag = vec![false; bm.fpool.len()];
        for i in 0..bm.vpool.len() {
            if bm.vert(i).head.htype != BM_VERT {
                continue;
            }
            if !verts_mask.get(bm.vert(i).head.index as usize).copied().unwrap_or(false)
                && !verts_mask.get(i).copied().unwrap_or(false)
            {
                continue;
            }
            let mut e_iter = bm.vert(i).e;
            if e_iter == BM_NIL {
                continue;
            }
            let e_first = e_iter;
            loop {
                let l0 = bm.edge(e_iter).l;
                if l0 != BM_NIL {
                    let mut l_iter = l0;
                    loop {
                        if bm.loo(l_iter).v == i {
                            let f = bm.loo(l_iter).f;
                            partial_elem_face_ensure(&mut bmp, &mut faces_tag, f);
                        }
                        l_iter = bm.loo(l_iter).radial_next;
                        if l_iter == l0 {
                            break;
                        }
                    }
                }
                e_iter = bmesh_disk_edge_next(bm, e_iter, i);
                if e_iter == e_first || e_iter == BM_NIL {
                    break;
                }
            }
        }
    }

    if params.do_normals {
        let mut verts_tag = vec![false; bm.vpool.len()];
        let faces = bmp.faces.clone();
        for f in faces {
            let l_first = bm.face(f).l_first;
            let mut l = l_first;
            loop {
                let v = bm.loo(l).v;
                partial_elem_vert_ensure(&mut bmp, &mut verts_tag, v);
                l = bm.loo(l).next;
                if l == l_first {
                    break;
                }
            }
        }
    }
    bmp
}

/// `BM_mesh_partial_create_from_verts_group_single` — faces fully inside
/// (or fully outside) the mask, plus verts of those faces.
pub fn bm_mesh_partial_create_from_verts_group_single(
    bm: &mut BMesh,
    params: BMPartialUpdateParams,
    verts_mask: &[bool],
) -> BMPartialUpdate {
    const SIDE_A: u8 = 1;
    const SIDE_B: u8 = 2;
    let mut bmp = BMPartialUpdate {
        verts: Vec::new(),
        faces: Vec::new(),
        params,
    };
    crate::mesh::bm_mesh_elem_index_ensure(bm, 1 << crate::BM_FACE);

    if params.do_normals || params.do_tessellate {
        let mut faces_tag = vec![false; bm.fpool.len()];
        for i in 0..bm.fpool.len() {
            if bm.face(i).head.htype != BM_FACE {
                continue;
            }
            let mut side_flag = 0u8;
            let l_first = bm.face(i).l_first;
            let mut l = l_first;
            loop {
                let j = bm.loo(l).v;
                side_flag |= if verts_mask.get(j).copied().unwrap_or(false) {
                    SIDE_A
                } else {
                    SIDE_B
                };
                l = bm.loo(l).next;
                if l == l_first {
                    break;
                }
            }
            /* faces that mix sides are excluded */
            if side_flag == SIDE_A {
                partial_elem_face_ensure(&mut bmp, &mut faces_tag, i);
            }
        }
    }

    if params.do_normals {
        let mut verts_tag = vec![false; bm.vpool.len()];
        let faces = bmp.faces.clone();
        for f in faces {
            let l_first = bm.face(f).l_first;
            let mut l = l_first;
            loop {
                let v = bm.loo(l).v;
                partial_elem_vert_ensure(&mut bmp, &mut verts_tag, v);
                l = bm.loo(l).next;
                if l == l_first {
                    break;
                }
            }
        }
        /* loose tagged verts */
        for i in 0..bm.vpool.len() {
            if bm.vert(i).head.htype != BM_VERT {
                continue;
            }
            if verts_mask.get(i).copied().unwrap_or(false)
                && crate::query::bm_vert_find_first_loop(bm, i) == BM_NIL
            {
                partial_elem_vert_ensure(&mut bmp, &mut verts_tag, i);
            }
        }
    }
    bmp
}

/// `BM_mesh_partial_create_from_verts_group_multi` — faces whose verts all
/// share one non-(-1) group id, plus those verts.
pub fn bm_mesh_partial_create_from_verts_group_multi(
    bm: &mut BMesh,
    params: BMPartialUpdateParams,
    verts_group: &[i32],
) -> BMPartialUpdate {
    let mut bmp = BMPartialUpdate {
        verts: Vec::new(),
        faces: Vec::new(),
        params,
    };
    crate::mesh::bm_mesh_elem_index_ensure(bm, 1 << crate::BM_FACE);

    if params.do_normals || params.do_tessellate {
        let mut faces_tag = vec![false; bm.fpool.len()];
        for i in 0..bm.fpool.len() {
            if bm.face(i).head.htype != BM_FACE {
                continue;
            }
            let l_first = bm.face(i).l_first;
            let group_test = verts_group
                .get(bm.loo(bm.loo(l_first).prev).v)
                .copied()
                .unwrap_or(-1);
            if group_test == -1 {
                continue;
            }
            let mut uniform = true;
            let mut l = l_first;
            loop {
                let g = verts_group.get(bm.loo(l).v).copied().unwrap_or(-1);
                if g != group_test || g == -1 {
                    uniform = false;
                    break;
                }
                l = bm.loo(l).next;
                if l == l_first {
                    break;
                }
            }
            if uniform {
                partial_elem_face_ensure(&mut bmp, &mut faces_tag, i);
            }
        }
    }

    if params.do_normals {
        let mut verts_tag = vec![false; bm.vpool.len()];
        let faces = bmp.faces.clone();
        for f in faces {
            let l_first = bm.face(f).l_first;
            let mut l = l_first;
            loop {
                let v = bm.loo(l).v;
                partial_elem_vert_ensure(&mut bmp, &mut verts_tag, v);
                l = bm.loo(l).next;
                if l == l_first {
                    break;
                }
            }
        }
        for i in 0..bm.vpool.len() {
            if bm.vert(i).head.htype != BM_VERT {
                continue;
            }
            if verts_group.get(i).copied().unwrap_or(0) != 0
                && crate::query::bm_vert_find_first_loop(bm, i) == BM_NIL
            {
                partial_elem_vert_ensure(&mut bmp, &mut verts_tag, i);
            }
        }
    }
    bmp
}
