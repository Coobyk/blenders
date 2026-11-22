/// Port of `metal/mtl_push_constant.cc` — Metal backend
/// `push_constant`.
/// `MTLPushConstant` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlPushConstant {
    /// `mtl_` object handle.
    pub handle: u64,
}
