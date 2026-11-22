/// Port of `intern/gpu_viewport.cc` — GPUViewport
/// (offscreen color+depth + stereo).
/// `GPUViewport` (verbatim — size, fbos,
/// stereo views).
pub struct GpuViewport {
    /// `size` w/h.
    pub size: [i32; 2],
    /// `fbo` handles.
    pub fbos: Vec<usize>,
    /// `views` (stereo left/right).
    pub views: Vec<usize>,
}
/// `GPU_viewport_size_get`/`set` (verbatim).
pub fn gpu_viewport_size_get(vp: &GpuViewport) -> [i32; 2] {
    vp.size
}
