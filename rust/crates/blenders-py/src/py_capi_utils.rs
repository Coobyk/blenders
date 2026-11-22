/// Port of `python/py_capi_utils.cc` — BPY
/// `py_capi_utils`.
/// `PyCapiUtils` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct PyCapiUtils {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
