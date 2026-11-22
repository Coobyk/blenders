/// Port of `python/python_compat.cc` — BPY
/// `python_compat`.
/// `PythonCompat` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct PythonCompat {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
