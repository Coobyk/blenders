/// Port of `python/idprop_py_api.cc` — BPY
/// `idprop_py_api`.
/// `IdpropPyApi` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct IdpropPyApi {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
