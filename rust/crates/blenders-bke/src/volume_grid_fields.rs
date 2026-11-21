//! Port of `intern/volume_grid_fields.cc` — `VoxelFieldContext` +
//! `TilesFieldContext`. OpenVDB (`openvdb::math::Transform`, `Coord`) is a
//! foreign dep: `Coord` → `[i32;3]`, `Transform` → the affine
//! `indexToWorld`/`worldToIndex` pair the file actually uses.
//!
//! `fn::FieldInput`/`GVArray` → `usize` handles (PORT_PENDING fields system);
//! the two lookups the file implements ("position" voxel lookup and the
//! index-field passthrough) are real.

use blenders_math::vector::Vec3;

/// `openvdb::math::Transform` — affine index↔world transform (voxel scale +
/// translate; OpenVDB's full map types PORT_PENDING).
#[derive(Clone, Copy)]
pub struct VoxelTransform {
    pub scale: Vec3,
    pub translate: Vec3,
}

impl VoxelTransform {
    /// `transform.indexToWorld(voxel)` (uniform voxel-size path).
    pub fn index_to_world(&self, voxel: [i32; 3]) -> Vec3 {
        Vec3::new(
            voxel[0] as f32 * self.scale.x + self.translate.x,
            voxel[1] as f32 * self.scale.y + self.translate.y,
            voxel[2] as f32 * self.scale.z + self.translate.z,
        )
    }
}

/// `VoxelFieldContext` — field context over voxel coordinates.
pub struct VoxelFieldContext<'a> {
    pub transform: VoxelTransform,
    /// `voxels_` — openvdb::Coord list.
    pub voxels: &'a [[i32; 3]],
}

impl<'a> VoxelFieldContext<'a> {
    /// `get_varray_for_input` "position" branch — world-space position per
    /// voxel (`field_input` dispatch is PORT_PENDING; this is the part the C
    /// computes itself).
    pub fn voxel_world_positions(&self) -> Vec<Vec3> {
        self.voxels
            .iter()
            .map(|&v| self.transform.index_to_world(v))
            .collect()
    }
}

/// `TilesFieldContext` — same shape over tile coordinates
/// (`openvdb::CoordBBox` tiles → `Vec<[i32;3]>` per tile → bounds).
pub struct TilesFieldContext<'a> {
    pub transform: VoxelTransform,
    /// One `[i32;3]` min-corner + `[i32;3]` max-corner per tile.
    pub tiles: &'a [([i32; 3], [i32; 3])],
}

impl<'a> TilesFieldContext<'a> {
    /// `get_varray_for_input` "position" branch — tile center in world space.
    pub fn tile_world_centers(&self) -> Vec<Vec3> {
        self.tiles
            .iter()
            .map(|&(min, max)| {
                let cx = [
                    (min[0] + max[0]) / 2,
                    (min[1] + max[1]) / 2,
                    (min[2] + max[2]) / 2,
                ];
                self.transform.index_to_world(cx)
            })
            .collect()
    }
}
