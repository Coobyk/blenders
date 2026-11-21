//! Port of `intern/volume_to_mesh.cc` — `bke::volume_to_mesh*`
//! (OpenVDB → Mesh conversion).
//!
//! `WITH_OPENVDB` isn't ported — `volumeToMesh` and the resampling
//! `GridTransformer` are injected (`volume_to_mesh_op` closure yields
//! the vertex/tri/quad arrays + error). The resolution-factor math and
//! the mesh-data fill are ported verbatim.

use blenders_math::vector::Vec3;

/// `VolumeToMeshResolutionMode`.
pub const VOLUME_TO_MESH_RESOLUTION_MODE_GRID: i32 = 0;
pub const VOLUME_TO_MESH_RESOLUTION_MODE_VOXEL_SIZE: i32 = 1;
pub const VOLUME_TO_MESH_RESOLUTION_MODE_VOXEL_AMOUNT: i32 = 2;

/// `VolumeToMeshResolution`.
#[derive(Clone, Copy)]
pub struct VolumeToMeshResolution {
    pub mode: i32,
    /// `settings.voxel_size` / `settings.voxel_amount`.
    pub voxel_size: f32,
    pub voxel_amount: f32,
}

/// `OpenVDBMeshData` — the generated mesh arrays.
#[derive(Default)]
pub struct OpenVDBMeshData {
    /// `verts`.
    pub verts: Vec<Vec3>,
    /// `tris` (3 vert indices each).
    pub tris: Vec<[i32; 3]>,
    /// `quads` (4 vert indices each).
    pub quads: Vec<[i32; 4]>,
}

/// `VolumeToMeshDataResult`.
pub struct VolumeToMeshDataResult {
    pub data: OpenVDBMeshData,
    /// `error` — OpenVDB exception text (empty = ok).
    pub error: String,
}

/// `VolumeToMeshOp::compute_desired_voxel_size` — `bbox` is the grid's
/// `indexToWorld(evalActiveVoxelBoundingBox())` extents (`None` =
/// degenerate → `std::nullopt`).
pub fn compute_desired_voxel_size(resolution: &VolumeToMeshResolution, max_extent: Option<f32>) -> Option<f32> {
    if resolution.mode == VOLUME_TO_MESH_RESOLUTION_MODE_VOXEL_SIZE {
        return Some(resolution.voxel_size);
    }
    let max_extent = max_extent?;
    if max_extent == 0.0 {
        return None;
    }
    Some(max_extent / resolution.voxel_amount)
}

/// `VolumeToMeshOp::compute_resolution_factor` —
/// `current_voxel_size / desired` (`max_voxel_size` = max component of
/// the grid's voxel size).
pub fn compute_resolution_factor(
    resolution: &VolumeToMeshResolution,
    current_voxel_size: f32,
    max_extent: Option<f32>,
) -> Option<f32> {
    if resolution.mode == VOLUME_TO_MESH_RESOLUTION_MODE_GRID {
        return None;
    }
    let desired = compute_desired_voxel_size(resolution, max_extent)?;
    Some(current_voxel_size / desired)
}

/// `fill_mesh_from_openvdb_data` — verbatim winding flips
/// (`2 - j` for tris, `3 - j` for quads).
pub fn fill_mesh_from_openvdb_data(
    vdb_verts: &[Vec3],
    vdb_tris: &[[i32; 3]],
    vdb_quads: &[[i32; 4]],
    vert_offset: usize,
    face_offset: usize,
    loop_offset: usize,
    vert_positions: &mut [Vec3],
    face_offsets: &mut [i32],
    corner_verts: &mut [i32],
) {
    vert_positions[vert_offset..vert_offset + vdb_verts.len()].copy_from_slice(vdb_verts);
    for (i, t) in vdb_tris.iter().enumerate() {
        face_offsets[face_offset + i] = (loop_offset + 3 * i) as i32;
        for j in 0..3 {
            corner_verts[loop_offset + 3 * i + j] = vert_offset as i32 + t[2 - j];
        }
    }
    let quad_offset = face_offset + vdb_tris.len();
    let quad_loop_offset = loop_offset + vdb_tris.len() * 3;
    for (i, q) in vdb_quads.iter().enumerate() {
        face_offsets[quad_offset + i] = (quad_loop_offset + 4 * i) as i32;
        for j in 0..4 {
            corner_verts[quad_loop_offset + 4 * i + j] = vert_offset as i32 + q[3 - j];
        }
    }
}

/// `volume_to_mesh_data` — `VolumeToMeshOp` dispatch injected
/// (`run` performs the scalar-type check + optional resample +
/// `volumeToMesh`, returning verts/tris/quads + error).
pub fn volume_to_mesh_data(
    run: &mut dyn FnMut(&VolumeToMeshResolution, f32, f32) -> VolumeToMeshDataResult,
    resolution: &VolumeToMeshResolution,
    threshold: f32,
    adaptivity: f32,
) -> VolumeToMeshDataResult {
    run(resolution, threshold, adaptivity)
}

/// `volume_to_mesh` — `BKE_mesh_new_nomain` + `mesh_calc_edges`/
/// `mesh_smooth_set`/`tag_overlapping_none` injected via `finish_mesh`.
pub fn volume_to_mesh(
    run: &mut dyn FnMut(&VolumeToMeshResolution, f32, f32) -> VolumeToMeshDataResult,
    resolution: &VolumeToMeshResolution,
    threshold: f32,
    adaptivity: f32,
    finish_mesh: &mut dyn FnMut(Vec<Vec3>, Vec<i32>, Vec<i32>),
) {
    let mesh_data = volume_to_mesh_data(run, resolution, threshold, adaptivity).data;
    let tot_loops = 3 * mesh_data.tris.len() + 4 * mesh_data.quads.len();
    let tot_faces = mesh_data.tris.len() + mesh_data.quads.len();
    let vert_positions = mesh_data.verts;
    let mut face_offsets = vec![0i32; tot_faces];
    let mut corner_verts = vec![0i32; tot_loops];
    /* `fill_mesh_from_openvdb_data` — offsets are all zero so the
     * position copy is a move (verts passed through unchanged). */
    for (i, t) in mesh_data.tris.iter().enumerate() {
        face_offsets[i] = (3 * i) as i32;
        for j in 0..3 {
            corner_verts[3 * i + j] = t[2 - j];
        }
    }
    let quad_loop_offset = mesh_data.tris.len() * 3;
    for (i, q) in mesh_data.quads.iter().enumerate() {
        face_offsets[mesh_data.tris.len() + i] = (quad_loop_offset + 4 * i) as i32;
        for j in 0..4 {
            corner_verts[quad_loop_offset + 4 * i + j] = q[3 - j];
        }
    }
    finish_mesh(vert_positions, face_offsets, corner_verts);
}

/// `volume_grid_to_mesh` — GRID resolution mode.
pub fn volume_grid_to_mesh(
    run: &mut dyn FnMut(&VolumeToMeshResolution, f32, f32) -> VolumeToMeshDataResult,
    threshold: f32,
    adaptivity: f32,
    finish_mesh: &mut dyn FnMut(Vec<Vec3>, Vec<i32>, Vec<i32>),
) {
    volume_to_mesh(
        run,
        &VolumeToMeshResolution {
            mode: VOLUME_TO_MESH_RESOLUTION_MODE_GRID,
            voxel_size: 0.0,
            voxel_amount: 0.0,
        },
        threshold,
        adaptivity,
        finish_mesh,
    );
}
