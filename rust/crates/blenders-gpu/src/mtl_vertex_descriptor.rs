/// Port of `metal/mtl_vertex_descriptor.cc` — Metal backend
/// `vertex_descriptor`.
/// `MTLVertexDescriptor` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlVertexDescriptor {
    /// `mtl_` object handle.
    pub handle: u64,
}
