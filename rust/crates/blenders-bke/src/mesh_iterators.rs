//! Port of `intern/mesh_iterators.cc` — mapped foreach loops over
//! mesh verts/edges/loops/face centers honoring edit-mode `BMesh`
//! wrappers and `CD_ORIGINDEX` remapping.
//!
//! `Mesh` runtime fields (`edit_mesh`, `bmesh_wrapper_data`) and
//! `CustomData` ORIGINDEX layers are injected per call; editmesh cache
//! normal/center getters take injected accessors.

use blenders_bmesh::mesh::bm_mesh_elem_index_ensure_ex;
use blenders_bmesh::{BMesh, BM_VERT};
use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::vector::Vec3;

/// `MeshForeachFlag`.
pub const MESH_FOREACH_NOP: i32 = 0;
pub const MESH_FOREACH_USE_NORMAL: i32 = 1 << 0;
pub const MESH_FOREACH_NO_SKIPPED: i32 = 1 << 1;

/// `ORIGINDEX_NONE` (`BKE_customdata.hh`).
pub const ORIGINDEX_NONE: i32 = -1;

/// Edit-mode branch input: `BMesh` + optional override positions
/// (`bmesh_wrapper_data->vert_positions`) + cache accessors.
pub struct EditBranch<'a> {
    pub bm: &'a BMesh,
    /// `bmesh_wrapper_data->vert_positions` — empty = use `eve->co`.
    pub vert_positions: &'a [Vec3],
}

/// `BKE_mesh_foreach_mapped_vert`.
///
/// Edit branch iterates bm verts (positions from wrapper or `eve->co`);
/// else positions + ORIGINDEX (`vert_origindex: None` → identity).
/// `vert_normals_edit` = `BKE_editmesh_cache_ensure_vert_normals`
/// (evaluated only when `USE_NORMAL` and positions overridden).
#[allow(clippy::too_many_arguments)]
pub fn bke_mesh_foreach_mapped_vert(
    positions: &[Vec3],
    vert_origindex: Option<&[i32]>,
    vert_normals: Option<&[Vec3]>,
    edit: Option<EditBranch>,
    vert_normals_edit: Option<&[Vec3]>,
    flag: i32,
    func: &mut dyn FnMut(i32, Vec3, Option<Vec3>),
) {
    let use_normal = flag & MESH_FOREACH_USE_NORMAL != 0;
    if let Some(edit) = edit {
        let bm = edit.bm;
        if !edit.vert_positions.is_empty() {
            for i in 0..bm.totvert as usize {
                let no = if use_normal {
                    vert_normals_edit.map(|n| n[i])
                } else {
                    None
                };
                func(i as i32, edit.vert_positions[i], no);
            }
        } else {
            for i in 0..bm.totvert as usize {
                let eve = &bm.vpool[i];
                let no = if use_normal { Some(eve.no.into()) } else { None };
                func(i as i32, eve.co.into(), no);
            }
        }
        return;
    }

    match vert_origindex {
        Some(index) => {
            for (i, &orig) in index.iter().enumerate() {
                let no = if use_normal {
                    vert_normals.map(|n| n[i])
                } else {
                    None
                };
                if orig == ORIGINDEX_NONE {
                    continue;
                }
                func(orig, positions[i], no);
            }
        }
        None => {
            for (i, &co) in positions.iter().enumerate() {
                let no = if use_normal {
                    vert_normals.map(|n| n[i])
                } else {
                    None
                };
                func(i as i32, co, no);
            }
        }
    }
}

/// `BKE_mesh_foreach_mapped_edge`.
#[allow(clippy::too_many_arguments)]
pub fn bke_mesh_foreach_mapped_edge(
    positions: &[Vec3],
    edges: &[[i32; 2]],
    edge_origindex: Option<&[i32]>,
    tot_edges: i32,
    edit: Option<EditBranch>,
    func: &mut dyn FnMut(i32, Vec3, Vec3),
) {
    if let Some(edit) = edit {
        let bm = edit.bm;
        if !edit.vert_positions.is_empty() {
            /* `BM_mesh_elem_index_ensure(bm, BM_VERT)` — caller-side
             * in this port (bm is shared ref). */
            for i in 0..bm.totedge as usize {
                let eed = &bm.epool[i];
                func(
                    i as i32,
                    edit.vert_positions[eed.v1 as usize],
                    edit.vert_positions[eed.v2 as usize],
                );
            }
        } else {
            for i in 0..bm.totedge as usize {
                let eed = &bm.epool[i];
                func(
                    i as i32,
                    bm.vpool[eed.v1 as usize].co.into(),
                    bm.vpool[eed.v2 as usize].co.into(),
                );
            }
        }
        return;
    }

    match edge_origindex {
        Some(index) => {
            for (i, e) in edges.iter().enumerate() {
                let orig = index[i];
                if orig == ORIGINDEX_NONE {
                    continue;
                }
                func(orig, positions[e[0] as usize], positions[e[1] as usize]);
            }
        }
        None if edges.len() as i32 == tot_edges => {
            for (i, e) in edges.iter().enumerate() {
                func(i as i32, positions[e[0] as usize], positions[e[1] as usize]);
            }
        }
        _ => {}
    }
}

/// `BKE_mesh_foreach_mapped_loop`.
#[allow(clippy::too_many_arguments)]
pub fn bke_mesh_foreach_mapped_loop(
    positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
    vert_origindex: Option<&[i32]>,
    face_origindex: Option<&[i32]>,
    corner_normals: Option<&[Vec3]>,
    edit: Option<EditBranch>,
    flag: i32,
    func: &mut dyn FnMut(i32, i32, Vec3, Option<Vec3>),
) {
    let use_normal = flag & MESH_FOREACH_USE_NORMAL != 0;
    if let Some(edit) = edit {
        let bm = edit.bm;
        for f_idx in 0..bm.totface as usize {
            let efa = &bm.fpool[f_idx];
            /* Walk the face loop ring. */
            let l_first = efa.l_first;
            let mut l_iter = l_first;
            loop {
                let l = &bm.lpool[l_iter];
                let eve = l.v;
                let v_idx = bm.vpool[eve].head.index;
                let co = if edit.vert_positions.is_empty() {
                    bm.vpool[eve].co.into()
                } else {
                    edit.vert_positions[v_idx as usize]
                };
                let no = corner_normals.map(|n| n[l.head.index as usize]);
                func(v_idx, f_idx as i32, co, if use_normal { no } else { None });
                l_iter = l.next;
                if l_iter == l_first {
                    break;
                }
            }
        }
        return;
    }

    if vert_origindex.is_some() || face_origindex.is_some() {
        for face_i in 0..faces.len() {
            let range = faces.group(face_i);
            for corner in range.start()..range.one_after_last() {
                let vert = corner_verts[corner as usize];
                let v_idx = vert_origindex.map_or(vert, |vi| vi[vert as usize]);
                let f_idx = face_origindex.map_or(face_i as i32, |fi| fi[face_i]);
                let no = if use_normal {
                    corner_normals.map(|n| n[corner as usize])
                } else {
                    None
                };
                if v_idx == ORIGINDEX_NONE || f_idx == ORIGINDEX_NONE {
                    continue;
                }
                func(v_idx, f_idx, positions[vert as usize], no);
            }
        }
    } else {
        for face_i in 0..faces.len() {
            let range = faces.group(face_i);
            for corner in range.start()..range.one_after_last() {
                let vert = corner_verts[corner as usize];
                let no = if use_normal {
                    corner_normals.map(|n| n[corner as usize])
                } else {
                    None
                };
                func(vert, face_i as i32, positions[vert as usize], no);
            }
        }
    }
}

/// `face_center_calc` — mean of corner positions.
pub fn face_center_calc(positions: &[Vec3], face_verts: &[i32]) -> Vec3 {
    let mut center = Vec3::ZERO;
    for &v in face_verts {
        center = center + positions[v as usize];
    }
    center * (1.0 / face_verts.len() as f32)
}

/// `face_normal_calc` — Newell's method.
pub fn face_normal_calc(positions: &[Vec3], face_verts: &[i32]) -> Vec3 {
    let mut normal = Vec3::ZERO;
    let n = face_verts.len();
    for i in 0..n {
        let a = positions[face_verts[i] as usize];
        let b = positions[face_verts[(i + 1) % n] as usize];
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    normal
}

/// `BKE_mesh_foreach_mapped_face_center`.
#[allow(clippy::too_many_arguments)]
pub fn bke_mesh_foreach_mapped_face_center(
    positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
    face_origindex: Option<&[i32]>,
    edit: Option<EditBranch>,
    face_centers_edit: Option<&[Vec3]>,
    face_normals_edit: Option<&[Vec3]>,
    flag: i32,
    func: &mut dyn FnMut(i32, Vec3, Option<Vec3>),
) {
    let use_normal = flag & MESH_FOREACH_USE_NORMAL != 0;
    if let Some(edit) = edit {
        let bm = edit.bm;
        let face_centers = face_centers_edit.unwrap_or(&[]);
        if let Some(face_normals) = face_normals_edit {
            for i in 0..bm.totface as usize {
                func(i as i32, face_centers[i], Some(face_normals[i]));
            }
        } else {
            for i in 0..bm.totface as usize {
                let efa = &bm.fpool[i];
                let no = if use_normal { Some(efa.no.into()) } else { None };
                func(i as i32, face_centers[i], no);
            }
        }
        return;
    }

    match face_origindex {
        Some(index) => {
            for i in 0..faces.len() {
                let orig = index[i];
                if orig == ORIGINDEX_NONE {
                    continue;
                }
                let range = faces.group(i);
                let face_verts = &corner_verts
                    [range.start() as usize..range.one_after_last() as usize];
                let center = face_center_calc(positions, face_verts);
                let no = if use_normal {
                    Some(face_normal_calc(positions, face_verts))
                } else {
                    None
                };
                func(orig, center, no);
            }
        }
        None => {
            for i in 0..faces.len() {
                let range = faces.group(i);
                let face_verts = &corner_verts
                    [range.start() as usize..range.one_after_last() as usize];
                let center = face_center_calc(positions, face_verts);
                let no = if use_normal {
                    Some(face_normal_calc(positions, face_verts))
                } else {
                    None
                };
                func(i as i32, center, no);
            }
        }
    }
}

/// `BKE_mesh_foreach_mapped_subdiv_face_center` — subdiv "facedot"
/// tagged verts.
#[allow(clippy::too_many_arguments)]
pub fn bke_mesh_foreach_mapped_subdiv_face_center(
    positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
    vert_normals: Option<&[Vec3]>,
    face_origindex: Option<&[i32]>,
    facedot_tags: &[bool],
    flag: i32,
    func: &mut dyn FnMut(i32, Vec3, Option<Vec3>),
) {
    let use_normal = flag & MESH_FOREACH_USE_NORMAL != 0;
    match face_origindex {
        Some(index) => {
            for i in 0..faces.len() {
                let orig = index[i];
                if orig == ORIGINDEX_NONE {
                    continue;
                }
                let range = faces.group(i);
                for &vert in
                    &corner_verts[range.start() as usize..range.one_after_last() as usize]
                {
                    if facedot_tags[vert as usize] {
                        func(
                            orig,
                            positions[vert as usize],
                            if use_normal {
                                vert_normals.map(|n| n[vert as usize])
                            } else {
                                None
                            },
                        );
                    }
                }
            }
        }
        None => {
            for i in 0..faces.len() {
                let range = faces.group(i);
                for &vert in
                    &corner_verts[range.start() as usize..range.one_after_last() as usize]
                {
                    if facedot_tags[vert as usize] {
                        func(
                            i as i32,
                            positions[vert as usize],
                            if use_normal {
                                vert_normals.map(|n| n[vert as usize])
                            } else {
                                None
                            },
                        );
                    }
                }
            }
        }
    }
}

/// `BKE_mesh_foreach_mapped_vert_coords_get` — first-mapped coords.
///
/// `foreach_mapped_vert` is `BKE_mesh_foreach_mapped_vert` bound by
/// the caller (mesh runtime access `PORT_PENDING`).
pub fn bke_mesh_foreach_mapped_vert_coords_get(
    totcos: usize,
    foreach_mapped_vert: &mut dyn FnMut(&mut dyn FnMut(i32, Vec3, Option<Vec3>)),
) -> Vec<Vec3> {
    let mut r_cos = vec![Vec3::ZERO; totcos];
    let mut vertex_visit = vec![false; totcos];
    foreach_mapped_vert(&mut |index, co, _no| {
        if index >= 0 && (index as usize) < totcos && !vertex_visit[index as usize] {
            /* We need coord from prototype vertex, not from copies —
             * we assume they stored in the beginning of vertex array
             * stored in evaluated mesh (mirror modifier for eg does
             * this). */
            r_cos[index as usize] = co;
            vertex_visit[index as usize] = true;
        }
    });
    r_cos
}

/// Ensure `head.index` on all verts (`BM_mesh_elem_index_ensure`).
pub fn ensure_vert_indices(bm: &mut BMesh) {
    let mut elem_offset = [0i32; 4];
    bm_mesh_elem_index_ensure_ex(bm, 1 << BM_VERT, &mut elem_offset);
}
