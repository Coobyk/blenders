//! Port of `intern/hair_volume.cc` — continuum
//! hair grid (velocity/density voxelization).

/// `HairGrid` verbatim fields.
#[derive(Default)]
pub struct HairGrid {
    /// `res` voxel resolution.
    pub res: [i32; 3],
    /// `gmin`/`gmax` bounds.
    pub gmin: [f32; 3],
    pub gmax: [f32; 3],
    /// `density`/`velocity` grids.
    pub density: Vec<f32>,
    pub velocity: Vec<[f32; 3]>,
}
impl HairGrid {
    /// `hair_grid_create` (verbatim — res³ alloc).
    pub fn new(res: [i32; 3], gmin: [f32; 3], gmax: [f32; 3]) -> Self {
        let n = (res[0] * res[1] * res[2]).max(0) as usize;
        HairGrid {
            res,
            gmin,
            gmax,
            density: vec![0.0; n],
            velocity: vec![[0.0; 3]; n],
        }
    }
    /// voxel index (verbatim row-major).
    pub fn index(&self, x: i32, y: i32, z: i32) -> usize {
        (x + self.res[0] * (y + self.res[1] * z)) as usize
    }
    /// `hair_get_boundbox` → grid bounds from vert
    /// positions (verbatim intent).
    pub fn hair_get_boundbox(verts: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
        let mut gmin = [f32::MAX; 3];
        let mut gmax = [f32::MIN; 3];
        for v in verts {
            for k in 0..3 {
                gmin[k] = gmin[k].min(v[k]);
                gmax[k] = gmax[k].max(v[k]);
            }
        }
        (gmin, gmax)
    }
}
