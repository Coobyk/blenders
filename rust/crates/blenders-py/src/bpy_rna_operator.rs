/// Port of `python/bpy_rna_operator.cc` — BPY
/// `bpy_rna_operator`.
/// `BpyRnaOperator` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaOperator {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
