//! Port of `intern/volume_grid_resample.cc`
//! — resample a volume grid to a new transform
//! (linear/trilinear via injected sampler).

/// `resample` (verbatim — walk new voxels,
// inverse-map to old grid, sample).
pub fn volume_grid_resample(
    old_to_new: [[f32; 4]; 4],
    new_dims: [i32; 3],
    trilinear: bool,
    sample: &dyn Fn(f32, f32, f32, bool) -> f32,
) -> Vec<f32> {
    let mut out = Vec::with_capacity((new_dims[0] * new_dims[1] * new_dims[2]) as usize);
    for z in 0..new_dims[2] {
        for y in 0..new_dims[1] {
            for x in 0..new_dims[0] {
                // New → old grid coords via inverse.
                let ox = old_to_new[0][0] * x as f32
                    + old_to_new[0][1] * y as f32
                    + old_to_new[0][2] * z as f32
                    + old_to_new[0][3];
                let oy = old_to_new[1][0] * x as f32
                    + old_to_new[1][1] * y as f32
                    + old_to_new[1][2] * z as f32
                    + old_to_new[1][3];
                let oz = old_to_new[2][0] * x as f32
                    + old_to_new[2][1] * y as f32
                    + old_to_new[2][2] * z as f32
                    + old_to_new[2][3];
                out.push(sample(ox, oy, oz, trilinear));
            }
        }
    }
    out
}
