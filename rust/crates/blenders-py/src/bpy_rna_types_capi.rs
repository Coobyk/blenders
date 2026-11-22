/// Port of `python/bpy_rna_types_capi.cc` — BPY
/// `bpy_rna_types_capi`.
/// `BpyRnaTypesCapi` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaTypesCapi {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
