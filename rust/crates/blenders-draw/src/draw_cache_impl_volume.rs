/// Port of `intern/draw_cache_impl_volume.cc`
/// — volume → draw-cache batch.
/// `VolumeBatchCache` (verbatim — slice/
/// cube batches).
#[derive(Default)]
pub struct VolumeBatchCache {
    /// `batch`.
    pub batch: Vec<usize>,
}
