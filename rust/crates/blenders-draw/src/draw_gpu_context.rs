/// Port of `intern/draw_gpu_context.cc` —
/// DRW GPU context wrapper (init,
/// make-current, viewport).
/// `DRWGpuContext` (verbatim — GPUContext
/// + shared flag).
#[derive(Default)]
pub struct DrwGpuContext {
    /// `gpu_context` (GPUContext handle).
    pub gpu_context: usize,
    /// `is_shared` (shared GL context).
    pub is_shared: bool,
}
