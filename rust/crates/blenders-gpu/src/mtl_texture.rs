/// Port of `metal/mtl_texture.cc` — Metal backend
/// `texture`.
/// `MTLTexture` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlTexture {
    /// `mtl_` object handle.
    pub handle: u64,
}
