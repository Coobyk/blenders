/// Port of `intern/draw_cache_impl_lattice.cc`
/// — lattice → draw-cache wire batch.
/// `LatticeBatchCache` (verbatim — grid
/// wire).
#[derive(Default)]
pub struct LatticeBatchCache {
    /// `batch`.
    pub batch: Vec<usize>,
}
