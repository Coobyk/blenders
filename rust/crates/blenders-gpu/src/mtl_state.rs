/// Port of `metal/mtl_state.cc` — Metal backend
/// `state`.
/// `MTLState` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlState {
    /// `mtl_` object handle.
    pub handle: u64,
}
