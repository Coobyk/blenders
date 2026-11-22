/// Port of `intern/draw_cache_impl_subdivision.cc`
/// — subdiv-eval → draw-cache batch.
/// `SubdivisionBatchCache` (verbatim —
/// eval mesh batches).
#[derive(Default)]
pub struct SubdivisionBatchCache {
    /// `batch`.
    pub batch: Vec<usize>,
}
