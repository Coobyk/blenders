//! Port of `DNA_pointcloud_types.h` — `PointCloud` ID.

use crate::customdata_types::CustomData;
use crate::id::{ID, NIL};
use crate::mesh_types::AttributeStorage;

/// `ePointCloud_Flag`.
pub type PointCloudFlag = i32;
pub const PT_DS_EXPAND: PointCloudFlag = 1 << 0;

/// `enum class PointCloudType : short`.
pub type PointCloudType = i16;
pub const POINTCLOUD_TYPE_POINTS: PointCloudType = 0;
pub const POINTCLOUD_TYPE_GSPLAT: PointCloudType = 1;

/// `POINTCLOUD_MATERIAL_NR`.
pub const POINTCLOUD_MATERIAL_NR: i32 = 1;

/// `PointCloud` — point cloud ID block.
#[derive(Clone)]
pub struct PointCloud {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub type_: PointCloudType,
    pub flag: PointCloudFlag,
    pub totpoint: i32,
    pub attribute_storage: AttributeStorage,
    /// `CustomData pdata_legacy`.
    pub pdata_legacy: CustomData,
    pub attributes_active_index: i32,
    /// `Material **mat` → Vec of refs.
    pub mat: Vec<usize>,
    pub totcol: i16,
    /// `bke::PointCloudRuntime *runtime`.
    pub runtime: usize,
    /// `draw::PointCloudBatchCache *pointcloud_batch_cache`.
    pub pointcloud_batch_cache: usize,
    /// `draw::GSplatBatchCache *gsplat_batch_cache`.
    pub gsplat_batch_cache: usize,
}
impl PointCloud {
    /// `positions()` / `radius()` — attribute accessors (PORT_PENDING until
    /// `bke::AttributeAccessor` lands; mirrored as TODO in blenkernel port).
    pub fn tag_positions_changed(&mut self) {
        // PORT_PENDING: attribute dirty tagging.
    }
    pub fn tag_radii_changed(&mut self) {
        // PORT_PENDING: attribute dirty tagging.
    }
}
impl Default for PointCloud {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            type_: POINTCLOUD_TYPE_POINTS,
            flag: 0,
            totpoint: 0,
            attribute_storage: AttributeStorage::default(),
            pdata_legacy: CustomData::default(),
            attributes_active_index: -1,
            mat: Vec::new(),
            totcol: 0,
            runtime: NIL,
            pointcloud_batch_cache: NIL,
            gsplat_batch_cache: NIL,
        }
    }
}
