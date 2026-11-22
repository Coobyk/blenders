/// Port of `python/bpy_rna_context.cc` — BPY
/// `bpy_rna_context`.
/// `BpyRnaContext` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaContext {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
