/// Port of `engines/eevee/eevee_lightcache.cc` — EEVEE
/// `lightcache` module.
/// `EeveeLightcache` (verbatim —
/// pass/pipeline state).
#[derive(Default)]
pub struct EeveeLightcache {
    /// `handle` (engine state).
    pub handle: usize,
}
