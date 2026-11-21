//! Port of `intern/editmesh_cache.cc` — `BKE_editmesh_cache_ensure_*` +
//! `BKE_editmesh_cache_calc_minmax` over `MeshWrapperCacheData`.

use blenders_bmesh::BMesh;
use blenders_bmesh::iterators::FacesOfMesh;
use blenders_bmesh::mesh::bm_mesh_elem_index_ensure_ex;
use blenders_bmesh::{BM_FACE, BM_VERT};
use blenders_bmesh::normals::bm_verts_calc_normal_vcos;
use blenders_bmesh::polygon::bm_face_calc_normal_vcos;
use blenders_bmesh::marking::bm_face_calc_center_median;
use blenders_math::vector::Vec3;

/// `bke::MeshWrapperCacheData` (BKE_editmesh_cache.hh) — cached derived data.
#[derive(Default)]
pub struct MeshWrapperCacheData {
    pub vert_positions: Vec<Vec3>,
    pub vert_normals: Vec<Vec3>,
    pub face_normals: Vec<Vec3>,
    pub face_centers: Vec<Vec3>,
}

/// `BKE_editmesh_cache_ensure_face_normals`.
pub fn bke_editmesh_cache_ensure_face_normals<'a>(
    bm: &mut BMesh,
    emd: &'a mut MeshWrapperCacheData,
) -> &'a [Vec3] {
    if emd.vert_positions.is_empty() || !emd.face_normals.is_empty() {
        return &emd.face_normals;
    }
    emd.face_normals = vec![Vec3::ZERO; bm.totface as usize];
    let mut offset = [0i32; 4];
    bm_mesh_elem_index_ensure_ex(bm, BM_VERT, &mut offset);
    let faces: Vec<usize> = FacesOfMesh::new(bm).collect();
    for f in faces {
        let i = bm.face(f).head.index as usize;
        emd.face_normals[i] = bm_face_calc_normal_vcos(bm, f, &emd.vert_positions);
    }
    bm.elem_index_dirty = false; /* &~=BM_FACE (port stores bool) */
    &emd.face_normals
}

/// `BKE_editmesh_cache_ensure_vert_normals`.
pub fn bke_editmesh_cache_ensure_vert_normals<'a>(
    bm: &mut BMesh,
    emd: &'a mut MeshWrapperCacheData,
) -> &'a [Vec3] {
    if emd.vert_positions.is_empty() || !emd.vert_normals.is_empty() {
        return &emd.vert_normals;
    }
    let _ = bke_editmesh_cache_ensure_face_normals(bm, emd);
    emd.vert_normals = vec![Vec3::ZERO; bm.totvert as usize];
    let mut offset = [0i32; 4];
    bm_mesh_elem_index_ensure_ex(bm, BM_FACE, &mut offset);
    /* split borrows: face_normals + vert_positions read, vert_normals written */
    let (fnos, vcos) = (emd.face_normals.clone(), emd.vert_positions.clone());
    bm_verts_calc_normal_vcos(bm, &fnos, &vcos, &mut emd.vert_normals);
    &emd.vert_normals
}

/// `BKE_editmesh_cache_ensure_face_centers`.
pub fn bke_editmesh_cache_ensure_face_centers<'a>(
    bm: &mut BMesh,
    emd: &'a mut MeshWrapperCacheData,
) -> &'a [Vec3] {
    if !emd.face_centers.is_empty() {
        return &emd.face_centers;
    }
    emd.face_centers = vec![Vec3::ZERO; bm.totface as usize];
    if emd.vert_positions.is_empty() {
        let faces: Vec<usize> = FacesOfMesh::new(bm).collect();
        for f in faces {
            let i = bm.face(f).head.index as usize;
            emd.face_centers[i] = bm_face_calc_center_median(bm, f);
        }
    } else {
        let mut offset = [0i32; 4];
        bm_mesh_elem_index_ensure_ex(bm, BM_VERT, &mut offset);
        let faces: Vec<usize> = FacesOfMesh::new(bm).collect();
        for f in faces {
            let i = bm.face(f).head.index as usize;
            /* median of the face's corner positions */
            let mut sum = Vec3::ZERO;
            let mut n = 0f32;
            let l_first = bm.face(f).l_first;
            let mut li = l_first;
            loop {
                let v = bm.loo(li).v;
                sum += emd.vert_positions[bm.vert(v).head.index as usize];
                n += 1.0;
                li = bm.loo(li).next;
                if li == l_first { break; }
            }
            emd.face_centers[i] = if n > 0.0 { sum / n } else { Vec3::ZERO };
        }
    }
    &emd.face_centers
}

/// `BKE_editmesh_cache_calc_minmax` — `None` for empty meshes.
pub fn bke_editmesh_cache_calc_minmax(bm: &mut BMesh, emd: &MeshWrapperCacheData) -> Option<(Vec3, Vec3)> {
    if bm.totvert == 0 {
        return None;
    }
    if emd.vert_positions.is_empty() {
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in 0..bm.vpool.len() {
            if bm.vert(v).head.htype != blenders_bmesh::BM_VERT { continue; }
            blenders_math::vector::minmax_v3(&mut min, &mut max, bm.vert(v).co);
        }
        Some((min, max))
    } else {
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for &p in &emd.vert_positions {
            blenders_math::vector::minmax_v3(&mut min, &mut max, p);
        }
        Some((min, max))
    }
}
