/// Port of `metal/mtl_texture_util.cc` — Metal backend
/// `texture_util`.
/// `MTLTextureUtil` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlTextureUtil {
    /// `mtl_` object handle.
    pub handle: u64,
}
