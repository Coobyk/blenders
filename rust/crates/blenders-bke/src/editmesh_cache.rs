//! Port of `intern/editmesh_cache.cc` — `BKE_editmesh_cache_ensure_*`
//! lazily-filled caches for edit-mesh wrappers.

use blenders_bmesh::{
    iterators::FacesOfMesh, marking::bm_face_calc_center_median,
    normals::bm_verts_calc_normal_vcos,
    polygon::{bm_face_calc_center_median_vcos, bm_face_calc_normal_vcos},
    BMesh,
};
use blenders_math::vector::Vec3;

/// `bke::MeshWrapperCacheData` (BKE_editmesh_cache.hh).
#[derive(Clone, Default)]
pub struct MeshWrapperCacheData {
    /// `Vector<float3> vert_positions` — empty until filled by a
    /// deformation-aware caller (`mesh_wrapper` PORT_PENDING there).
    pub vert_positions: Vec<Vec3>,
    pub vert_normals: Vec<Vec3>,
    pub face_normals: Vec<Vec3>,
    pub face_centers: Vec<Vec3>,
}

/// `BKE_editmesh_cache_ensure_face_normals`.
pub fn bke_editmesh_cache_ensure_face_normals<'a>(
    bm: &BMesh,
    emd: &'a mut MeshWrapperCacheData,
) -> &'a [Vec3] {
    if emd.vert_positions.is_empty() || !emd.face_normals.is_empty() {
        return &emd.face_normals;
    }
    emd.face_normals = vec![Vec3::ZERO; bm.totface as usize];
    /* `BM_mesh_elem_index_ensure(&bm, BM_VERT)` — elem indexes are
     * maintained by the BMesh; iterate faces in index order. */
    for (i, f) in FacesOfMesh::new(bm).enumerate() {
        emd.face_normals[i] = bm_face_calc_normal_vcos(bm, f, &emd.vert_positions);
    }
    /* `bm.elem_index_dirty &= ~BM_FACE` — n/a to this port. */
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
    emd.vert_normals = vec![Vec3::ZERO; bm.totvert as usize];
    {
        let face_normals = bke_editmesh_cache_ensure_face_normals(bm, emd).to_vec();
        bm_verts_calc_normal_vcos(bm, &face_normals, &emd.vert_positions, &mut emd.vert_normals);
    }
    &emd.vert_normals
}

/// `BKE_editmesh_cache_ensure_face_centers`.
pub fn bke_editmesh_cache_ensure_face_centers<'a>(
    bm: &BMesh,
    emd: &'a mut MeshWrapperCacheData,
) -> &'a [Vec3] {
    if !emd.face_centers.is_empty() {
        return &emd.face_centers;
    }
    emd.face_centers = vec![Vec3::ZERO; bm.totface as usize];
    if emd.vert_positions.is_empty() {
        for (i, f) in FacesOfMesh::new(bm).enumerate() {
            emd.face_centers[i] = bm_face_calc_center_median(bm, f);
        }
    } else {
        for (i, f) in FacesOfMesh::new(bm).enumerate() {
            emd.face_centers[i] = bm_face_calc_center_median_vcos(bm, f, &emd.vert_positions);
        }
    }
    &emd.face_centers
}

/// `BKE_editmesh_cache_calc_minmax` — `(min, max)` or None on 0 verts.
pub fn bke_editmesh_cache_calc_minmax(bm: &BMesh, emd: &MeshWrapperCacheData) -> Option<(Vec3, Vec3)> {
    if bm.totvert == 0 {
        return None;
    }
    if emd.vert_positions.is_empty() {
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in 0..bm.totvert as usize {
            let co = bm.vert(v).co;
            min = Vec3::new(min.x.min(co.x), min.y.min(co.y), min.z.min(co.z));
            max = Vec3::new(max.x.max(co.x), max.y.max(co.y), max.z.max(co.z));
        }
        return Some((min, max));
    }
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    for co in &emd.vert_positions {
        min = Vec3::new(min.x.min(co.x), min.y.min(co.y), min.z.min(co.z));
        max = Vec3::new(max.x.max(co.x), max.y.max(co.y), max.z.max(co.z));
    }
    Some((min, max))
}
