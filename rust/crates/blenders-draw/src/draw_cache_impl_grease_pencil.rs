/// Port of `intern/draw_cache_impl_grease_pencil.cc`
/// — grease-pencil → draw-cache batch.
/// `GreasePencilBatchCache` (verbatim —
/// stroke/fill batches).
#[derive(Default)]
pub struct GreasePencilBatchCache {
    /// `batch`.
    pub batch: Vec<usize>,
}
