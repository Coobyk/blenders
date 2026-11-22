/// Port of `metal/mtl_batch.cc` — Metal backend
/// `batch`.
/// `MTLBatch` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlBatch {
    /// `mtl_` object handle.
    pub handle: u64,
}
