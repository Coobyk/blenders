/// Port of `metal/mtl_query.cc` — Metal backend
/// `query`.
/// `MTLQuery` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlQuery {
    /// `mtl_` object handle.
    pub handle: u64,
}
