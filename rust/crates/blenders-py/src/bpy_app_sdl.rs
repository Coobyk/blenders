/// Port of `python/bpy_app_sdl.cc` — BPY
/// `bpy_app_sdl`.
/// `BpyAppSdl` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAppSdl {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
