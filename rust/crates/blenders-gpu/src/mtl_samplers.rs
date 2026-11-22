/// Port of `metal/mtl_samplers.cc` — Metal backend
/// `samplers`.
/// `MTLSamplers` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlSamplers {
    /// `mtl_` object handle.
    pub handle: u64,
}
