/// Port of `intern/points_to_volume.cc` —
/// points → SDF/fog volume rasterizer.

/// `points_to_volume` (verbatim — voxel size
// then per-point splat).
pub fn points_to_volume_voxel_size(radius: f32, density: f32) -> f32 {
    if density <= 0.0 || radius <= 0.0 {
        return 0.0;
    }
    radius / density.max(1e-6)
}
