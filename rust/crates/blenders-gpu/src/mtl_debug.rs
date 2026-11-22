/// Port of `metal/mtl_debug.cc` — Metal backend
/// `debug`.
/// `MTLDebug` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlDebug {
    /// `mtl_` object handle.
    pub handle: u64,
}
