/// Port of `python/bl_math_py_api.cc` — BPY
/// `bl_math_py_api`.
/// `BlMathPyApi` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BlMathPyApi {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
