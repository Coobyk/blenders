//! Port of `intern/mesh_remesh_voxel.cc` — remesh helpers: edge/face
//! center calc, nearest-tri/face/edge projection (`find_nearest_*`),
//! attribute reprojection orchestration and `BKE_mesh_remesh_voxel`
//! / `fix_poles` entry points.
//!
//! OpenVDB level-set + Quadriflow (`remesh_voxel`/`remesh_quadriflow`)
//! are injected (`RemeshBackend`); attribute gather/sample callbacks
//! and BMesh ops for `fix_poles` are injected too. Threading is
//! serial (`parallel_for` → plain loops per project convention).

use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::Vec3;

/// `mesh::face_center_calc` over `corner_verts`.
fn face_center_calc(positions: &[Vec3], corner_verts: &[i32]) -> Vec3 {
    if corner_verts.is_empty() {
        return Vec3::ZERO;
    }
    let mut cent = Vec3::ZERO;
    for &v in corner_verts {
        cent += positions[v as usize];
    }
    cent / corner_verts.len() as f32
}

/// `calc_edge_centers`.
pub fn calc_edge_centers(positions: &[Vec3], edges: &[[i32; 2]]) -> Vec<Vec3> {
    edges
        .iter()
        .map(|e| (positions[e[0] as usize] + positions[e[1] as usize]) / 2.0)
        .collect()
}

/// `calc_face_centers`.
pub fn calc_face_centers<'a>(
    positions: &[Vec3],
    faces: OffsetIndices<'a>,
    corner_verts: &[i32],
) -> Vec<Vec3> {
    (0..faces.len())
        .map(|i| {
            let g = faces.group(i);
            face_center_calc(positions, &corner_verts[g.start() as usize..g.one_after_last() as usize])
        })
        .collect()
}

/// `bvh::Tree::closest_point` result.
pub struct ClosestPoint {
    /// `index` — triangle index.
    pub index: i32,
    /// `bary_coord` — barycentric coords on that tri.
    pub bary_coord: Vec3,
}

/// `find_nearest_tris` — `closest_point` is injected (the BVH
/// wrapper `bvhutils` isn't in the crate yet).
pub fn find_nearest_tris(
    positions: &[Vec3],
    closest_point: &dyn Fn(Vec3) -> Option<ClosestPoint>,
    bary_coords: Option<&mut Vec<Vec3>>,
) -> Vec<i32> {
    let mut tris = Vec::with_capacity(positions.len());
    let mut bc: Vec<Vec3> = Vec::new();
    let want_bary = bary_coords.is_some();
    for &p in positions {
        let nearest = closest_point(p).expect("bvh closest_point failed");
        tris.push(nearest.index);
        if want_bary {
            bc.push(nearest.bary_coord);
        }
    }
    if let Some(out) = bary_coords {
        *out = bc;
    }
    tris
}

/// `find_nearest_faces` — face centers → nearest src face via tris.
#[allow(clippy::too_many_arguments)]
pub fn find_nearest_faces(
    src_tri_faces: &[i32],
    dst_positions: &[Vec3],
    dst_faces: OffsetIndices,
    dst_corner_verts: &[i32],
    closest_point: &dyn Fn(Vec3) -> Option<ClosestPoint>,
) -> Vec<i32> {
    let face_centers = calc_face_centers(dst_positions, dst_faces, dst_corner_verts);
    let tri_indices = find_nearest_tris(&face_centers, closest_point, None);
    tri_indices
        .iter()
        .map(|&t| src_tri_faces[t as usize])
        .collect()
}

/// `find_nearest_edges` — nearest src face's edge closest to each
/// dst edge center.
#[allow(clippy::too_many_arguments)]
pub fn find_nearest_edges(
    src_positions: &[Vec3],
    src_edges: &[[i32; 2]],
    src_faces: OffsetIndices,
    src_corner_edges: &[i32],
    src_tri_faces: &[i32],
    dst_positions: &[Vec3],
    dst_edges: &[[i32; 2]],
    closest_point: &dyn Fn(Vec3) -> Option<ClosestPoint>,
) -> Vec<i32> {
    let edge_centers = calc_edge_centers(dst_positions, dst_edges);
    let tri_indices = find_nearest_tris(&edge_centers, closest_point, None);

    let mut nearest_edges = vec![0i32; dst_edges.len()];
    for (i, dst_center) in edge_centers.iter().enumerate() {
        let src_face = src_tri_faces[tri_indices[i] as usize];
        let face_edges = src_faces.group(src_face as usize);
        let src_face_edges =
            &src_corner_edges[face_edges.start() as usize..face_edges.one_after_last() as usize];

        let mut min_dist = f32::INFINITY;
        let mut min_edge = src_face_edges[0];
        for &e in src_face_edges {
            let src_edge = src_edges[e as usize];
            let src_center =
                (src_positions[src_edge[0] as usize] + src_positions[src_edge[1] as usize]) / 2.0;
            let d = src_center.distance_squared(*dst_center);
            if d < min_dist {
                min_dist = d;
                min_edge = e;
            }
        }
        nearest_edges[i] = min_edge;
    }
    nearest_edges
}

/// Attribute domains the reprojection covers (mirrors the C's
/// per-domain dispatch).
pub const REPROJECT_POINT: i32 = 0;
pub const REPROJECT_CORNER: i32 = 1;
pub const REPROJECT_EDGE: i32 = 2;
pub const REPROJECT_FACE: i32 = 3;

/// `gather_attributes` — per named attribute, gather
/// `src[index_map[i]]` into dst; typed access injected.
pub fn gather_attributes(
    names: &[String],
    index_map: &[i32],
    gather: &mut dyn FnMut(&str, &[i32]),
) {
    for name in names {
        gather(name, index_map);
    }
}

/// `sample_vertex_attributes` — per named point-domain attribute,
/// barycentric sample over `src_corner_tris[tri]`; injected.
pub fn sample_vertex_attributes(
    names: &[String],
    tri_indices: &[i32],
    bary_coords: &[Vec3],
    sample: &mut dyn FnMut(&str, &[i32], &[Vec3]),
) {
    for name in names {
        sample(name, tri_indices, bary_coords);
    }
}

/// `sample_corner_attributes` — corner-domain sample via corner ids.
pub fn sample_corner_attributes(
    names: &[String],
    tri_indices: &[i32],
    bary_coords: &[Vec3],
    sample: &mut dyn FnMut(&str, &[i32], &[Vec3]),
) {
    for name in names {
        sample(name, tri_indices, bary_coords);
    }
}

/// `mesh_remesh_reproject_attributes` — the full dispatch: vertex +
/// corner attrs sampled by bary coords, edge/face attrs gathered by
/// index map.
#[allow(clippy::too_many_arguments)]
pub fn mesh_remesh_reproject_attributes(
    point_ids: &[String],
    corner_ids: &[String],
    edge_ids: &[String],
    face_ids: &[String],
    src_tri_faces: &[i32],
    src_positions: &[Vec3],
    src_edges: &[[i32; 2]],
    src_faces: OffsetIndices,
    src_corner_edges: &[i32],
    dst_positions: &[Vec3],
    dst_faces: OffsetIndices,
    dst_corner_verts: &[i32],
    dst_edges: &[[i32; 2]],
    closest_point: &dyn Fn(Vec3) -> Option<ClosestPoint>,
    sample_vertex_attr: &mut dyn FnMut(&str, &[i32], &[Vec3]),
    sample_corner_attr: &mut dyn FnMut(&str, &[i32], &[Vec3]),
    gather_attr: &mut dyn FnMut(&str, &[i32]),
) {
    if !point_ids.is_empty() || !corner_ids.is_empty() {
        let mut bary_coords = Vec::new();
        let vert_nearest_tris =
            find_nearest_tris(dst_positions, closest_point, Some(&mut bary_coords));

        if !point_ids.is_empty() {
            /* `BKE_defgroup_copy_list` — caller copies vgroup names. */
            sample_vertex_attributes(
                point_ids,
                &vert_nearest_tris,
                &bary_coords,
                sample_vertex_attr,
            );
        }
        if !corner_ids.is_empty() {
            sample_corner_attributes(
                corner_ids,
                &vert_nearest_tris,
                &bary_coords,
                sample_corner_attr,
            );
        }
    }

    if !edge_ids.is_empty() {
        let map = find_nearest_edges(
            src_positions,
            src_edges,
            src_faces,
            src_corner_edges,
            src_tri_faces,
            dst_positions,
            dst_edges,
            closest_point,
        );
        gather_attributes(edge_ids, &map, gather_attr);
    }

    if !face_ids.is_empty() {
        let map = find_nearest_faces(
            src_tri_faces,
            dst_positions,
            dst_faces,
            dst_corner_verts,
            closest_point,
        );
        gather_attributes(face_ids, &map, gather_attr);
    }
}

/// `RemeshBackend` — the OpenVDB/Quadriflow parts that are not yet
/// ported (`PORT_PENDING`).
pub trait RemeshBackend {
    /// `remesh_voxel` — voxel remesh → `(positions, faces_offsets,
    /// corner_verts)` or error string.
    fn remesh_voxel(
        &mut self,
        positions: &[Vec3],
        faces: &[Vec<i32>],
        corner_verts: &[i32],
        voxel_size: f32,
        adaptivity: f32,
        isovalue: f32,
    ) -> Result<(Vec<Vec3>, Vec<i32>, Vec<i32>), String>;

    /// `remesh_quadriflow` — quadriflow remesh.
    #[allow(clippy::type_complexity)]
    fn remesh_quadriflow(
        &mut self,
        positions: &[Vec3],
        faces: &[Vec<i32>],
        corner_verts: &[i32],
        target_faces: i32,
        seed: i32,
        preserve_sharp: bool,
        preserve_boundary: bool,
        adaptive_scale: bool,
        minimum_cost_flow: bool,
        aggressiveness: f32,
    ) -> Result<(Vec<Vec3>, Vec<i32>, Vec<i32>), String> {
        let _ = (
            positions, faces, corner_verts, target_faces, seed, preserve_sharp,
            preserve_boundary, adaptive_scale, minimum_cost_flow, aggressiveness,
        );
        Err("quadriflow backend unavailable".to_string())
    }
}

/// `BKE_mesh_remesh_voxel` — voxel remesh (delegated to backend).
pub fn bke_mesh_remesh_voxel(
    positions: &[Vec3],
    faces: &[Vec<i32>],
    corner_verts: &[i32],
    voxel_size: f32,
    adaptivity: f32,
    isovalue: f32,
    backend: &mut dyn RemeshBackend,
) -> Result<(Vec<Vec3>, Vec<i32>, Vec<i32>), String> {
    backend.remesh_voxel(positions, faces, corner_verts, voxel_size, adaptivity, isovalue)
}

/// `BKE_mesh_remesh_quadriflow`.
#[allow(clippy::too_many_arguments)]
pub fn bke_mesh_remesh_quadriflow(
    positions: &[Vec3],
    faces: &[Vec<i32>],
    corner_verts: &[i32],
    target_faces: i32,
    seed: i32,
    preserve_sharp: bool,
    preserve_boundary: bool,
    adaptive_scale: bool,
    minimum_cost_flow: bool,
    aggressiveness: f32,
    backend: &mut dyn RemeshBackend,
) -> Result<(Vec<Vec3>, Vec<i32>, Vec<i32>), String> {
    backend.remesh_quadriflow(
        positions,
        faces,
        corner_verts,
        target_faces,
        seed,
        preserve_sharp,
        preserve_boundary,
        adaptive_scale,
        minimum_cost_flow,
        aggressiveness,
    )
}

/// `BKE_mesh_remesh_voxel_fix_poles` — the BMesh clean-up pass
/// (kill faces joining 3-valence verts, collapse the join edges,
/// dissolve remaining non-3-valence-corner faces) — drives injected
/// BMesh hooks (`PoleFixMesh`).
pub trait PoleFixMesh {
    /// `BM_face_kill`.
    fn face_kill(&mut self, f: usize);
    /// `BM_edge_create`.
    fn edge_create(&mut self, v1: usize, v2: usize) -> usize;
    /// `BM_vert_edge_count`.
    fn vert_edge_count(&self, v: usize) -> usize;
    /// `BM_edge_exists`.
    fn edge_exists(&self, v1: usize, v2: usize) -> bool;
    /// `BM_edge_collapse` at `v1`, keeping the other vert; returns
    /// the remaining vert.
    fn edge_collapse(&mut self, e: usize, v1: usize, kill_degenerate: bool) -> usize;
    /// `BM_dissolve_faces`/`BM_face_dissolve`.
    fn face_dissolve(&mut self, f: usize);
    /// `BM_elem_flag_set/get` for `BM_ELEM_TAG`.
    fn elem_flag_set(&mut self, elem: usize, tagged: bool);
    /// `BM_elem_flag_test`.
    fn elem_flag_test(&self, elem: usize) -> bool;
    /// Vert positions for the midpoint calc (`ed->v1->co`).
    fn edge_verts(&self, e: usize) -> (usize, usize);
    /// Set a vert's coords.
    fn vert_set_co(&mut self, v: usize, co: Vec3);
    /// Vert coords.
    fn vert_co(&self, v: usize) -> Vec3;
    /// All faces (handles) — iteration order irrelevant.
    fn faces(&self) -> Vec<usize>;
    /// Face verts.
    fn face_verts(&self, f: usize) -> Vec<usize>;
    /// All edges.
    fn edges(&self) -> Vec<usize>;
}

/// `BKE_mesh_remesh_voxel_fix_poles` over an injected BMesh.
pub fn bke_mesh_remesh_voxel_fix_poles(bm: &mut dyn PoleFixMesh) {
    /* Pass 1: kill faces whose corners contain two 3-valence,
     * non-adjacent verts; create a tagged edge between them. */
    let mut tagged_edges = Vec::new();
    for f in bm.faces() {
        let mut v1 = None;
        let mut v2 = None;
        for v in bm.face_verts(f) {
            if bm.vert_edge_count(v) == 3 {
                if v1.is_some() {
                    v2 = Some(v);
                } else {
                    v1 = Some(v);
                }
            }
        }
        if let (Some(a), Some(b)) = (v1, v2) {
            if a != b && !bm.edge_exists(a, b) {
                bm.face_kill(f);
                let e = bm.edge_create(a, b);
                bm.elem_flag_set(e, true);
                tagged_edges.push(e);
            }
        }
    }

    /* Pass 2: collapse tagged edges at their midpoint. */
    for e in bm.edges() {
        if bm.elem_flag_test(e) {
            let (a, b) = bm.edge_verts(e);
            let co = (bm.vert_co(a) + bm.vert_co(b)) / 2.0;
            let vc = bm.edge_collapse(e, a, true);
            bm.vert_set_co(vc, co);
        }
    }

    /* Pass 3: dissolve faces where any corner vert isn't 3-valence. */
    for f in bm.faces() {
        let dissolve = bm.face_verts(f).iter().all(|&v| bm.vert_edge_count(v) == 3);
        if dissolve {
            bm.face_dissolve(f);
        }
    }
}
