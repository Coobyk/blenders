/// Port of `metal/mtl_immediate.cc` — Metal backend
/// `immediate`.
/// `MTLImmediate` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlImmediate {
    /// `mtl_` object handle.
    pub handle: u64,
}
