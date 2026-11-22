/// Port of `intern/gpu_storage_buffer.cc` —
/// GPUStorageBuf (shader-visible data block).
/// `GPUStorageBuf` (verbatim — size + usage +
/// data).
pub struct GpuStorageBuf {
    /// `size_in_bytes`.
    pub size: usize,
    /// `data` (CPU copy for upload).
    pub data: Vec<u8>,
    /// `usage` (static/dynamic/stream).
    pub usage: i32,
}
