/// Port of `python/bpy_app_build_options.cc` — BPY
/// `bpy_app_build_options`.
/// `BpyAppBuildOptions` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAppBuildOptions {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
