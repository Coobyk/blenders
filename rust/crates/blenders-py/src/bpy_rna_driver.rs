/// Port of `python/bpy_rna_driver.cc` — BPY
/// `bpy_rna_driver`.
/// `BpyRnaDriver` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaDriver {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
