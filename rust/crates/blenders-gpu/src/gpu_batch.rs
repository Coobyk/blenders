/// Port of `intern/gpu_batch.cc` — GPUBatch
/// (vbo+ibo+inst → draw cmd).
/// `GPUBatch` (verbatim — owns verts/elem/
/// inst + phase).
pub struct GpuBatch {
    /// `verts` (vbo handle).
    pub verts: usize,
    /// `elem` (ibo handle).
    pub elem: usize,
    /// `inst` (instance vbo).
    pub inst: usize,
    /// `phase` (idle/in-frame).
    pub phase: i32,
}
/// `GPU_BATCH_*` phase (verbatim).
pub const GPU_BATCH_INVALID: i32 = -1;
pub const GPU_BATCH_READY: i32 = 0;
pub const GPU_BATCH_BUILDING: i32 = 1;
