/// Port of `python/bpy_operator_wrap.cc` — BPY
/// `bpy_operator_wrap`.
/// `BpyOperatorWrap` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyOperatorWrap {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
