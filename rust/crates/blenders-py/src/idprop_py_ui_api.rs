/// Port of `python/idprop_py_ui_api.cc` — BPY
/// `idprop_py_ui_api`.
/// `IdpropPyUiApi` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct IdpropPyUiApi {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
