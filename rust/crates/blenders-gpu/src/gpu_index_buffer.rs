/// Port of `intern/gpu_index_buffer.cc` —
/// GPUIndexBuf (u16/u32, prim builders).
/// `GPUIndexBufBuilder` (verbatim — index list
/// + prim type).
#[derive(Default)]
pub struct GpuIndexBufBuilder {
    /// `index` list.
    pub index: Vec<u32>,
    /// `prim_type` (eGPUPrimType).
    pub prim_type: i32,
}
/// `GPU_PRIM_*` ordinals (verbatim).
pub const GPU_PRIM_POINTS: i32 = 0;
pub const GPU_PRIM_LINES: i32 = 1;
pub const GPU_PRIM_TRIS: i32 = 2;
pub const GPU_PRIM_LINE_STRIP: i32 = 3;
pub const GPU_PRIM_LINE_LOOP: i32 = 4;
pub const GPU_PRIM_TRI_STRIP: i32 = 5;
pub const GPU_PRIM_TRI_FAN: i32 = 6;
pub const GPU_PRIM_LINES_ADJ: i32 = 7;
pub const GPU_PRIM_TRIS_ADJ: i32 = 8;
pub const GPU_PRIM_LINE_STRIP_ADJ: i32 = 9;
/// `GPU_INDEX_U16`/`GPU_INDEX_U32` (verbatim).
pub const GPU_INDEX_U16: i32 = 0;
pub const GPU_INDEX_U32: i32 = 1;
/// `GPU_indices_per_primitive` (verbatim —
/// verts per prim).
pub fn gpu_indices_per_primitive(prim_type: i32) -> i32 {
    match prim_type {
        GPU_PRIM_POINTS => 1,
        GPU_PRIM_LINES | GPU_PRIM_LINES_ADJ => 2,
        GPU_PRIM_TRIS | GPU_PRIM_TRIS_ADJ => 3,
        _ => 2,
    }
}
