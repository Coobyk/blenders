/// Port of `intern/draw_cache_impl_curves.cc`
/// — curves-hair → draw-cache batch.
/// `CurvesBatchCache` (verbatim — hair
/// prim batches).
#[derive(Default)]
pub struct CurvesBatchCache {
    /// `batch`.
    pub batch: Vec<usize>,
}
