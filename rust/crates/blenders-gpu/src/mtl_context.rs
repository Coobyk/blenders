/// Port of `metal/mtl_context.cc` — Metal backend
/// `context`.
/// `MTLContext` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlContext {
    /// `mtl_` object handle.
    pub handle: u64,
}
