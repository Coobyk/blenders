/// Port of `intern/draw_cache_impl_pointcloud.cc`
/// — pointcloud → draw-cache batch.
/// `PointCloudBatchCache` (verbatim —
/// point prim batch).
#[derive(Default)]
pub struct PointCloudBatchCache {
    /// `batch`.
    pub batch: Vec<usize>,
}
