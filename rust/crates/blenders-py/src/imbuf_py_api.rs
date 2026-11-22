/// Port of `python/imbuf_py_api.cc` — BPY
/// `imbuf_py_api`.
/// `ImbufPyApi` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct ImbufPyApi {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
