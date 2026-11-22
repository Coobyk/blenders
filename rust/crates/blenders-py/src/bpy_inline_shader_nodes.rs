/// Port of `python/bpy_inline_shader_nodes.cc` — BPY
/// `bpy_inline_shader_nodes`.
/// `BpyInlineShaderNodes` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyInlineShaderNodes {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
