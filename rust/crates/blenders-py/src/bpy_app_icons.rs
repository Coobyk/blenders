/// Port of `python/bpy_app_icons.cc` — BPY
/// `bpy_app_icons`.
/// `BpyAppIcons` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAppIcons {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
