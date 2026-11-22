//! Port of `intern/mesh_to_volume.cc` — mesh
//! → fog/SDF volume (OpenVDB adapter → injected).

/// `volume_compute_voxel_size` (verbatim —
// density → voxel size from bounds+res).
pub fn volume_compute_voxel_size(
    bounds_extent: [f32; 3],
    resolution_amount: f32,
) -> f32 {
    let largest = bounds_extent[0].max(bounds_extent[1]).max(bounds_extent[2]);
    if resolution_amount <= 0.0 {
        return 0.0;
    }
    largest / resolution_amount
}
