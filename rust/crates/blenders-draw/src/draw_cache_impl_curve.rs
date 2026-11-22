/// Port of `intern/draw_cache_impl_curve.cc`
/// — curve → draw-cache batch (edge
/// polyline + control points).
/// `CurveBatchCache` (verbatim — wire/
/// surface batches).
#[derive(Default)]
pub struct CurveBatchCache {
    /// `batch`.
    pub batch: Vec<usize>,
}
