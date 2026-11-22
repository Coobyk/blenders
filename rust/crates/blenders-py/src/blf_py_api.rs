/// Port of `python/blf_py_api.cc` — BPY
/// `blf_py_api`.
/// `BlfPyApi` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BlfPyApi {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
