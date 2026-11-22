/// Port of `python/bpy_operator.cc` — BPY
/// `bpy_operator`.
/// `BpyOperator` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyOperator {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
