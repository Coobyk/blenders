/// Port of `metal/mtl_common.cc` — Metal backend
/// `common`.
/// `MTLCommon` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlCommon {
    /// `mtl_` object handle.
    pub handle: u64,
}
