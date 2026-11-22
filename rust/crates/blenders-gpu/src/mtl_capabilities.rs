/// Port of `metal/mtl_capabilities.cc` — Metal backend
/// `capabilities`.
/// `MTLCapabilities` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlCapabilities {
    /// `mtl_` object handle.
    pub handle: u64,
}
