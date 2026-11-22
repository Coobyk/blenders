/// Port of `python/bpy_operator_function.cc` — BPY
/// `bpy_operator_function`.
/// `BpyOperatorFunction` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyOperatorFunction {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
