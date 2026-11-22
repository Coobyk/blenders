/// Port of `python/bpy_app_translations.cc` — BPY
/// `bpy_app_translations`.
/// `BpyAppTranslations` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAppTranslations {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
