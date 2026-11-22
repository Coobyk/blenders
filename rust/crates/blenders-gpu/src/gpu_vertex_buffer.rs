/// Port of `intern/gpu_vertex_buffer.cc` —
/// GPUVertBuf (CPU-side vertex data then
/// backend upload).
/// `GPUVertBuf` (verbatim — format + data +
/// backend handle).
pub struct GpuVertBuf {
    /// `format`.
    pub format: super::gpu_vertex_format::GpuVertFormat,
    /// `data` packed.
    pub data: Vec<u8>,
    /// `vertex_len`.
    pub vertex_len: i32,
}
impl GpuVertBuf {
    /// `GPU_vertbuf_data_step` (verbatim —
    /// nth vertex ptr).
    pub fn vertex_at(&self, i: i32) -> &[u8] {
        let off = i as usize * self.format.stride as usize;
        &self.data[off..off + self.format.stride as usize]
    }
}
