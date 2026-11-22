/// Port of `python/bpy_app_ffmpeg.cc` — BPY
/// `bpy_app_ffmpeg`.
/// `BpyAppFfmpeg` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAppFfmpeg {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
