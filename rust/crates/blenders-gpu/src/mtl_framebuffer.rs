/// Port of `metal/mtl_framebuffer.cc` — Metal backend
/// `framebuffer`.
/// `MTLFramebuffer` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlFramebuffer {
    /// `mtl_` object handle.
    pub handle: u64,
}
