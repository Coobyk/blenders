/// Port of `intern/draw_cache_impl_gsplat.cc`
/// — gaussian-splat → draw-cache batch.
/// `GSplatBatchCache` (verbatim — splat
/// points).
#[derive(Default)]
pub struct GSplatBatchCache {
    /// `batch`.
    pub batch: Vec<usize>,
}
