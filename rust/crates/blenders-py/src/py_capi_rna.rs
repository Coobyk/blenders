/// Port of `python/py_capi_rna.cc` — BPY
/// `py_capi_rna`.
/// `PyCapiRna` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct PyCapiRna {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
