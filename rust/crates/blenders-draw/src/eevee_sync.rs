/// Port of `engines/eevee/eevee_sync.cc` — EEVEE
/// `sync` module.
/// `EeveeSync` (verbatim —
/// pass/pipeline state).
#[derive(Default)]
pub struct EeveeSync {
    /// `handle` (engine state).
    pub handle: usize,
}
