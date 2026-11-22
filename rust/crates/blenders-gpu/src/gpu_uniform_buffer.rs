/// Port of `intern/gpu_uniform_buffer.cc` —
/// GPUUniformBuf (ubo bind+update).
/// `GPUUniformBuf` (verbatim — size + data +
/// bind-point).
pub struct GpuUniformBuf {
    /// `size_in_bytes`.
    pub size: usize,
    /// `data`.
    pub data: Vec<u8>,
    /// `bindpoint`.
    pub bindpoint: i32,
}
