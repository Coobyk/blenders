/// Port of `intern/draw_cache_impl_particles.cc`
/// — particles → draw-cache batch.
/// `ParticleBatchCache` (verbatim — point/
/// hair batches).
#[derive(Default)]
pub struct ParticleBatchCache {
    /// `batch`.
    pub batch: Vec<usize>,
}
