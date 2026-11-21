//! Port of `intern/mesh_runtime.cc` — `MeshRuntime` caches
//! (corner-to-face / vert-to-face / vert-to-corner / edge-to-corner
//! maps, loose-element masks, corner tris), `TrianglesCache`
//! freeze/unfreeze, and all the `Mesh::tag_*_changed` dirty-flow.
//!
//! `SharedCache<T>` → `Mutex<Option<T>>` (`ensure`/`tag_dirty`/
//! `is_cached`); mesh topology data is supplied as slices; the
//! `mesh_eval`/draw `batch_cache` pointers are `usize` handles with
//! injected free/dirty-tag callbacks.

use std::sync::Mutex;

use blenders_bli::index_mask::IndexMask;
use blenders_bli::offset_indices;
use blenders_math::vector::Vec3;

use blenders_math::geom::poly_to_tri_count;
use crate::mesh_tessellate::{
    corner_tris_calc, corner_tris_calc_face_indices, corner_tris_calc_with_normals,
};

/// `ME_NO_OVERLAPPING_TOPOLOGY` / `ME_FLAG_UV_SELECT_SYNC_VALID`
/// (cleared by `clear_geometry`).
pub const ME_NO_OVERLAPPING_TOPOLOGY: u32 = 1 << 0;
pub const ME_FLAG_UV_SELECT_SYNC_VALID: u32 = 1 << 1;

/// `SharedCache` — lazily computed data invalidated by `tag_dirty`.
/// `ensure` computes at most once until the next dirty tag.
pub struct SharedCache<T> {
    data: Mutex<Option<T>>,
}

impl<T> Default for SharedCache<T> {
    fn default() -> Self {
        Self { data: Mutex::new(None) }
    }
}

impl<T: Clone> SharedCache<T> {
    /// `SharedCache::ensure` — compute if not cached.
    pub fn ensure(&self, compute: impl FnOnce() -> T) -> T {
        let mut g = self.data.lock().unwrap();
        if g.is_none() {
            *g = Some(compute());
        }
        g.clone().unwrap()
    }
    /// `SharedCache::is_cached`.
    pub fn is_cached(&self) -> bool {
        self.data.lock().unwrap().is_some()
    }
    /// `SharedCache::tag_dirty` — drop the cached data.
    pub fn tag_dirty(&self) {
        *self.data.lock().unwrap() = None;
    }
    /// Current data (`data().as_span()` semantics).
    pub fn get(&self) -> Option<T> {
        self.data.lock().unwrap().clone()
    }
    /// Overwrite (`r_data.mask = {}` semantics).
    pub fn set(&self, v: T) {
        *self.data.lock().unwrap() = Some(v);
    }
}

/// `LooseGeomCache` — mask + allocator simplified to just the mask.
#[derive(Clone)]
pub struct LooseGeomCache {
    pub mask: IndexMask,
}

impl LooseGeomCache {
    pub fn empty() -> Self {
        Self {
            mask: IndexMask::from_range(0, 0),
        }
    }
}

/// `TrianglesCache` — freeze/defer dirty-tagging.
pub struct TrianglesCache {
    pub data: SharedCache<Vec<[i32; 3]>>,
    frozen: Mutex<bool>,
    dirty_while_frozen: std::sync::atomic::AtomicBool,
}

impl Default for TrianglesCache {
    fn default() -> Self {
        Self {
            data: SharedCache::default(),
            frozen: Mutex::new(false),
            dirty_while_frozen: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl TrianglesCache {
    pub fn freeze(&self) {
        *self.frozen.lock().unwrap() = true;
        self.dirty_while_frozen
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn unfreeze(&self) {
        *self.frozen.lock().unwrap() = false;
        if self.dirty_while_frozen.swap(false, std::sync::atomic::Ordering::Relaxed) {
            self.data.tag_dirty();
        }
    }
    pub fn tag_dirty(&self) {
        if *self.frozen.lock().unwrap() {
            self.dirty_while_frozen
                .store(true, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.data.tag_dirty();
        }
    }
}

/// `GroupedSpan` — offsets + indices.
pub struct GroupedSpan {
    pub offsets: Vec<i32>,
    pub indices: Vec<i32>,
}

/// Topology view of a `Mesh` — the slices the runtime caches need.
pub struct MeshTopo<'a> {
    pub verts_num: i32,
    pub edges_num: i32,
    pub corners_num: i32,
    pub faces_num: i32,
    pub vert_positions: &'a [Vec3],
    pub corner_verts: &'a [i32],
    pub corner_edges: &'a [i32],
    /// `faces()` as `Range` per face.
    pub faces: &'a [std::ops::Range<i64>],
    /// `face_normals()` — empty when dirty.
    pub face_normals: &'a [Vec3],
    /// `edges()` flat vertex pairs for `loose_verts`.
    pub edge_verts: &'a [i32],
}

/// `MeshRuntime` — the cache fields ported.
#[derive(Default)]
pub struct MeshRuntime {
    /// `mesh_eval` (evaluated `Mesh` ID handle).
    pub mesh_eval: usize,
    /// `batch_cache` (`draw::MeshBatchCache` handle).
    pub batch_cache: usize,
    pub bvh_cache_edges: SharedCache<usize>,
    pub bvh_cache_faces: SharedCache<usize>,
    pub bvh_cache_corner_tris: SharedCache<usize>,
    pub bvh_cache_corner_tris_no_hidden: SharedCache<usize>,
    pub bvh_cache_loose_verts: SharedCache<usize>,
    pub bvh_cache_loose_verts_no_hidden: SharedCache<usize>,
    pub bvh_cache_loose_edges: SharedCache<usize>,
    pub bvh_cache_loose_edges_no_hidden: SharedCache<usize>,
    pub bvh_embree_tris_cache: SharedCache<usize>,
    pub bvh_embree_verts_cache: SharedCache<usize>,
    pub bvh_embree_edges_cache: SharedCache<usize>,
    /// `subdiv_ccg` handle (`SubdivCCG` until subdiv ports).
    pub subdiv_ccg: Mutex<usize>,
    pub bounds_cache: SharedCache<usize>,
    pub corner_to_face_map_cache: SharedCache<Vec<i32>>,
    pub vert_to_face_offset_cache: SharedCache<Vec<i32>>,
    pub vert_to_face_map_cache: SharedCache<Vec<i32>>,
    pub vert_to_corner_map_cache: SharedCache<Vec<i32>>,
    pub edge_to_corner_offset_cache: SharedCache<Vec<i32>>,
    pub edge_to_corner_map_cache: SharedCache<Vec<i32>>,
    pub vert_normals_cache: SharedCache<usize>,
    pub vert_normals_true_cache: SharedCache<usize>,
    pub face_normals_cache: SharedCache<usize>,
    pub face_normals_true_cache: SharedCache<usize>,
    pub corner_normals_cache: SharedCache<usize>,
    pub loose_edges_cache: SharedCache<LooseGeomCache>,
    pub loose_verts_cache: SharedCache<LooseGeomCache>,
    pub verts_no_face_cache: SharedCache<LooseGeomCache>,
    pub corner_tris_cache: TrianglesCache,
    pub corner_tri_faces_cache: SharedCache<Vec<i32>>,
    pub shrinkwrap_boundary_cache: SharedCache<usize>,
    pub max_material_index: SharedCache<usize>,
    pub subsurf_face_dot_tags: Mutex<Vec<u8>>,
    pub subsurf_optimal_display_edges: Mutex<Vec<u8>>,
    /// `bmesh_wrapper_data` (`MeshWrapperCacheData` handle).
    pub bmesh_wrapper_data: Mutex<usize>,
    /// `spatial_groups` handle.
    pub spatial_groups: Mutex<usize>,
    /// `flag` mirror of `Mesh.flag` bits the runtime clears.
    pub flag: Mutex<u32>,
}

/// `free_bvh_caches`.
fn free_bvh_caches(rt: &MeshRuntime) {
    rt.bvh_cache_edges.tag_dirty();
    rt.bvh_cache_faces.tag_dirty();
    rt.bvh_cache_corner_tris.tag_dirty();
    rt.bvh_cache_corner_tris_no_hidden.tag_dirty();
    rt.bvh_cache_loose_verts.tag_dirty();
    rt.bvh_cache_loose_verts_no_hidden.tag_dirty();
    rt.bvh_cache_loose_edges.tag_dirty();
    rt.bvh_cache_loose_edges_no_hidden.tag_dirty();
    rt.bvh_embree_tris_cache.tag_dirty();
    rt.bvh_embree_verts_cache.tag_dirty();
    rt.bvh_embree_edges_cache.tag_dirty();
}

/// `set_bools`.
fn set_bools(bools: &mut [bool], indices_to_set: &[i32]) {
    for &i in indices_to_set {
        bools[i as usize] = true;
    }
}

/// `Mesh::corner_to_face_map`.
pub fn corner_to_face_map(rt: &MeshRuntime, faces: &[std::ops::Range<i64>]) -> Vec<i32> {
    rt.corner_to_face_map_cache.ensure(|| {
        let mut map = Vec::new();
        for (i, face) in faces.iter().enumerate() {
            map.resize(map.len().max(face.end as usize), -1);
            for c in face.start..face.end {
                map[c as usize] = i as i32;
            }
        }
        map
    })
}

/// `Mesh::vert_to_face_map_offsets`.
pub fn vert_to_face_map_offsets(rt: &MeshRuntime, topo: &MeshTopo) -> Vec<i32> {
    rt.vert_to_face_offset_cache.ensure(|| {
        let mut offsets = vec![0i32; topo.verts_num as usize + 1];
        offset_indices::build_reverse_offsets(topo.corner_verts, &mut offsets);
        offsets
    })
}

/// `Mesh::vert_to_face_map` — gather from corner map when both caches
/// are hot, else `build_vert_to_face_indices`.
pub fn vert_to_face_map(rt: &MeshRuntime, topo: &MeshTopo) -> GroupedSpan {
    let offsets = vert_to_face_map_offsets(rt, topo);
    let indices = rt.vert_to_face_map_cache.ensure(|| {
        let mut data = vec![0i32; topo.corners_num as usize];
        if let (Some(corner_to_face), Some(vert_to_corner)) = (
            rt.corner_to_face_map_cache.get(),
            rt.vert_to_corner_map_cache.get(),
        ) {
            /* The vertex to face cache can be built from the vertex to
             * face corner and face corner to face maps if they are
             * both already cached. */
            for (i, &corner) in vert_to_corner.iter().enumerate() {
                data[i] = corner_to_face[corner as usize];
            }
        } else {
            /* `bke::mesh::build_vert_to_face_indices` — scatter
             * face index to each corner's vert group. */
            let mut positions = offsets.clone();
            for (face_i, face) in topo.faces.iter().enumerate() {
                for c in face.start..face.end {
                    let vert = topo.corner_verts[c as usize] as usize;
                    data[positions[vert] as usize] = face_i as i32;
                    positions[vert] += 1;
                }
            }
        }
        data
    });
    GroupedSpan { offsets, indices }
}

/// `Mesh::vert_to_corner_map` — `build_vert_to_corner_indices`.
pub fn vert_to_corner_map(rt: &MeshRuntime, topo: &MeshTopo) -> GroupedSpan {
    let offsets = vert_to_face_map_offsets(rt, topo);
    let indices = rt.vert_to_corner_map_cache.ensure(|| {
        let mut data = vec![0i32; topo.corner_verts.len()];
        let mut positions = offsets.clone();
        for (corner, &vert) in topo.corner_verts.iter().enumerate() {
            data[positions[vert as usize] as usize] = corner as i32;
            positions[vert as usize] += 1;
        }
        data
    });
    GroupedSpan { offsets, indices }
}

/// `Mesh::edge_to_corner_map`.
pub fn edge_to_corner_map(rt: &MeshRuntime, topo: &MeshTopo) -> GroupedSpan {
    let offsets = rt.edge_to_corner_offset_cache.ensure(|| {
        let mut offsets = vec![0i32; topo.edges_num as usize + 1];
        offset_indices::build_reverse_offsets(topo.corner_edges, &mut offsets);
        offsets
    });
    let indices = rt.edge_to_corner_map_cache.ensure(|| {
        let mut data = vec![0i32; offsets.last().copied().unwrap_or(0) as usize];
        let mut positions = offsets.clone();
        for (corner, &edge) in topo.corner_edges.iter().enumerate() {
            data[positions[edge as usize] as usize] = corner as i32;
            positions[edge as usize] += 1;
        }
        data
    });
    GroupedSpan { offsets, indices }
}

/// `Mesh::loose_verts` — verts used by no edge.
pub fn loose_verts(rt: &MeshRuntime, topo: &MeshTopo) -> IndexMask {
    rt.loose_verts_cache
        .ensure(|| {
            let mut bools = vec![false; topo.verts_num as usize];
            set_bools(&mut bools, topo.edge_verts);
            LooseGeomCache {
                mask: IndexMask::from_bools_inverse(
                    &IndexMask::from_range(0, topo.verts_num as i64),
                    &bools,
                ),
            }
        })
        .mask
}

/// `Mesh::verts_no_face` — verts used by no face.
pub fn verts_no_face(rt: &MeshRuntime, topo: &MeshTopo) -> IndexMask {
    rt.verts_no_face_cache
        .ensure(|| {
            let mut bools = vec![false; topo.verts_num as usize];
            set_bools(&mut bools, topo.corner_verts);
            LooseGeomCache {
                mask: IndexMask::from_bools_inverse(
                    &IndexMask::from_range(0, topo.verts_num as i64),
                    &bools,
                ),
            }
        })
        .mask
}

/// `Mesh::no_overlapping_topology`.
pub fn no_overlapping_topology(rt: &MeshRuntime) -> bool {
    *rt.flag.lock().unwrap() & ME_NO_OVERLAPPING_TOPOLOGY != 0
}

/// `Mesh::loose_edges`.
pub fn loose_edges(rt: &MeshRuntime, topo: &MeshTopo) -> IndexMask {
    rt.loose_edges_cache
        .ensure(|| {
            let mut bools = vec![false; topo.edges_num as usize];
            set_bools(&mut bools, topo.corner_edges);
            LooseGeomCache {
                mask: IndexMask::from_bools_inverse(
                    &IndexMask::from_range(0, topo.edges_num as i64),
                    &bools,
                ),
            }
        })
        .mask
}

/// `try_tag_verts_no_face_none` — if no loose edges/verts exist, all
/// verts are used by faces.
fn try_tag_verts_no_face_none(rt: &MeshRuntime, topo: &MeshTopo) {
    if !rt.loose_edges_cache.is_cached() || !loose_edges(rt, topo).is_empty() {
        return;
    }
    if !rt.loose_verts_cache.is_cached() || !loose_verts(rt, topo).is_empty() {
        return;
    }
    rt.verts_no_face_cache.set(LooseGeomCache::empty());
}

/// `Mesh::tag_loose_verts_none`.
pub fn tag_loose_verts_none(rt: &MeshRuntime, topo: &MeshTopo) {
    rt.loose_verts_cache.set(LooseGeomCache::empty());
    try_tag_verts_no_face_none(rt, topo);
}

/// `Mesh::tag_loose_edges_none`.
pub fn tag_loose_edges_none(rt: &MeshRuntime, topo: &MeshTopo) {
    rt.loose_edges_cache.set(LooseGeomCache::empty());
    try_tag_verts_no_face_none(rt, topo);
}

/// `Mesh::tag_overlapping_none`.
pub fn tag_overlapping_none(rt: &MeshRuntime) {
    *rt.flag.lock().unwrap() |= ME_NO_OVERLAPPING_TOPOLOGY;
}

/// `Mesh::corner_tris`.
pub fn corner_tris(rt: &MeshRuntime, topo: &MeshTopo, face_normals_dirty: bool) -> Vec<[i32; 3]> {
    rt.corner_tris_cache.data.ensure(|| {
        let mut tris =
            vec![[0i32; 3]; poly_to_tri_count(topo.faces_num, topo.corners_num) as usize];
        if face_normals_dirty || topo.face_normals.is_empty() {
            corner_tris_calc(topo.vert_positions, topo.faces, topo.corner_verts, &mut tris);
        } else {
            corner_tris_calc_with_normals(
                topo.vert_positions,
                topo.faces,
                topo.corner_verts,
                topo.face_normals,
                &mut tris,
            );
        }
        tris
    })
}

/// `Mesh::corner_tri_faces`.
pub fn corner_tri_faces(rt: &MeshRuntime, topo: &MeshTopo) -> Vec<i32> {
    rt.corner_tri_faces_cache.ensure(|| {
        let mut tri_faces =
            vec![0i32; poly_to_tri_count(topo.faces_num, topo.corners_num) as usize];
        corner_tris_calc_face_indices(topo.faces, &mut tri_faces);
        tri_faces
    })
}

/// `BKE_mesh_runtime_corner_tris_len` — size without computing.
pub fn bke_mesh_runtime_corner_tris_len(faces_num: i32, corners_num: i32) -> i32 {
    poly_to_tri_count(faces_num, corners_num)
}

/// `BKE_mesh_runtime_clear_geometry`.
pub fn bke_mesh_runtime_clear_geometry(rt: &MeshRuntime) {
    free_bvh_caches(rt);
    *rt.subdiv_ccg.lock().unwrap() = 0;
    rt.bounds_cache.tag_dirty();
    rt.vert_to_face_offset_cache.tag_dirty();
    rt.vert_to_face_map_cache.tag_dirty();
    rt.vert_to_corner_map_cache.tag_dirty();
    rt.corner_to_face_map_cache.tag_dirty();
    rt.edge_to_corner_offset_cache.tag_dirty();
    rt.edge_to_corner_map_cache.tag_dirty();
    rt.vert_normals_cache.tag_dirty();
    rt.vert_normals_true_cache.tag_dirty();
    rt.face_normals_cache.tag_dirty();
    rt.face_normals_true_cache.tag_dirty();
    rt.corner_normals_cache.tag_dirty();
    rt.loose_edges_cache.tag_dirty();
    rt.loose_verts_cache.tag_dirty();
    rt.verts_no_face_cache.tag_dirty();
    rt.corner_tris_cache.data.tag_dirty();
    rt.corner_tri_faces_cache.tag_dirty();
    rt.shrinkwrap_boundary_cache.tag_dirty();
    rt.max_material_index.tag_dirty();
    rt.subsurf_face_dot_tags.lock().unwrap().clear();
    rt.subsurf_optimal_display_edges.lock().unwrap().clear();
    *rt.spatial_groups.lock().unwrap() = 0;
    *rt.flag.lock().unwrap() &= !(ME_NO_OVERLAPPING_TOPOLOGY | ME_FLAG_UV_SELECT_SYNC_VALID);
}

/// `Mesh::tag_edges_split` — triangulation didn't change because
/// vertex positions and loop vertex indices didn't change.
pub fn tag_edges_split(rt: &MeshRuntime) {
    free_bvh_caches(rt);
    rt.vert_normals_cache.tag_dirty();
    rt.corner_normals_cache.tag_dirty();
    *rt.subdiv_ccg.lock().unwrap() = 0;
    rt.vert_to_face_offset_cache.tag_dirty();
    rt.vert_to_face_map_cache.tag_dirty();
    rt.vert_to_corner_map_cache.tag_dirty();
    rt.edge_to_corner_offset_cache.tag_dirty();
    rt.edge_to_corner_map_cache.tag_dirty();
    if let Some(c) = rt.loose_edges_cache.get() {
        if !c.mask.is_empty() {
            rt.loose_edges_cache.tag_dirty();
        }
    }
    if let Some(c) = rt.loose_verts_cache.get() {
        if !c.mask.is_empty() {
            rt.loose_verts_cache.tag_dirty();
        }
    }
    if let Some(c) = rt.verts_no_face_cache.get() {
        if !c.mask.is_empty() {
            rt.verts_no_face_cache.tag_dirty();
        }
    }
    rt.subsurf_face_dot_tags.lock().unwrap().clear();
    rt.subsurf_optimal_display_edges.lock().unwrap().clear();
    rt.shrinkwrap_boundary_cache.tag_dirty();
}

/// `Mesh::tag_sharpness_changed` / `tag_custom_normals_changed`.
pub fn tag_normals_changed(rt: &MeshRuntime) {
    rt.vert_normals_cache.tag_dirty();
    rt.face_normals_cache.tag_dirty();
    rt.corner_normals_cache.tag_dirty();
}

/// `Mesh::tag_face_winding_changed`.
pub fn tag_face_winding_changed(rt: &MeshRuntime) {
    rt.vert_normals_cache.tag_dirty();
    rt.face_normals_cache.tag_dirty();
    rt.vert_normals_true_cache.tag_dirty();
    rt.face_normals_true_cache.tag_dirty();
    rt.corner_normals_cache.tag_dirty();
    rt.vert_to_corner_map_cache.tag_dirty();
    /* Reversing faces swaps edges stored for each of its corners, but
     * the number of corners using each edge doesn't change, so only
     * the indices are recalculated, not the offsets. */
    rt.edge_to_corner_map_cache.tag_dirty();
    rt.shrinkwrap_boundary_cache.tag_dirty();
}

/// `Mesh::tag_positions_changed`.
pub fn tag_positions_changed(rt: &MeshRuntime) {
    rt.vert_normals_cache.tag_dirty();
    rt.face_normals_cache.tag_dirty();
    rt.vert_normals_true_cache.tag_dirty();
    rt.face_normals_true_cache.tag_dirty();
    rt.corner_normals_cache.tag_dirty();
    rt.shrinkwrap_boundary_cache.tag_dirty();
    tag_positions_changed_no_normals(rt);
}

/// `Mesh::tag_positions_changed_no_normals`.
pub fn tag_positions_changed_no_normals(rt: &MeshRuntime) {
    free_bvh_caches(rt);
    rt.corner_tris_cache.tag_dirty();
    rt.bounds_cache.tag_dirty();
    rt.shrinkwrap_boundary_cache.tag_dirty();
}

/// `Mesh::tag_positions_changed_uniformly` — normals and
/// triangulation didn't change (all verts moved by the same amount).
pub fn tag_positions_changed_uniformly(rt: &MeshRuntime) {
    free_bvh_caches(rt);
    rt.bounds_cache.tag_dirty();
}

/// `Mesh::tag_topology_changed`.
pub fn tag_topology_changed(rt: &MeshRuntime) {
    bke_mesh_runtime_clear_geometry(rt);
}

/// `Mesh::tag_visibility_changed`.
pub fn tag_visibility_changed(rt: &MeshRuntime) {
    rt.bvh_cache_corner_tris_no_hidden.tag_dirty();
    rt.bvh_cache_loose_verts_no_hidden.tag_dirty();
    rt.bvh_cache_loose_edges_no_hidden.tag_dirty();
}

/// `Mesh::tag_material_index_changed`.
pub fn tag_material_index_changed(rt: &MeshRuntime) {
    rt.max_material_index.tag_dirty();
}

/// `BKE_mesh_runtime_clear_cache` — eval mesh + batch cache +
/// bmesh wrapper, then geometry.
pub fn bke_mesh_runtime_clear_cache(
    rt: &mut MeshRuntime,
    free_mesh_eval: &mut dyn FnMut(usize),
    free_batch_cache: &mut dyn FnMut(usize),
) {
    if rt.mesh_eval != 0 {
        free_mesh_eval(rt.mesh_eval);
        rt.mesh_eval = 0;
    }
    if rt.batch_cache != 0 {
        free_batch_cache(rt.batch_cache);
        rt.batch_cache = 0;
    }
    *rt.bmesh_wrapper_data.lock().unwrap() = 0;
    bke_mesh_runtime_clear_geometry(rt);
}

/// `BKE_mesh_batch_cache_dirty_tag` — tag own + evaluated mesh's
/// batch cache (`dirty_tag_cb` injected, draw engine registers it).
pub fn bke_mesh_batch_cache_dirty_tag(
    rt: &MeshRuntime,
    mode: i32,
    mesh_eval_batch_cache: usize,
    dirty_tag_cb: &dyn Fn(usize, i32),
) {
    if rt.batch_cache != 0 {
        dirty_tag_cb(rt.batch_cache, mode);
    }
    /* Also tag batch cache for subdivided mesh, if it exists this will
     * be the mesh that is actually being drawn. */
    if rt.mesh_eval != 0 && mesh_eval_batch_cache != 0 {
        dirty_tag_cb(mesh_eval_batch_cache, mode);
    }
}
