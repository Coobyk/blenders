/// Port of `intern/gpu_framebuffer.cc` —
/// GPUFrameBuf (attachment slots + update).
/// `GPUFrameBuffer` (verbatim — color+depth+
/// stencil slots).
pub struct GpuFrameBuffer {
    /// `attachments` (slot → tex).
    pub attachments: Vec<(i32, usize)>,
    /// `size`.
    pub size: [i32; 2],
    /// `dirty_attachments`.
    pub dirty: bool,
}
/// `GPU_ATTACHMENT_*` slots (verbatim).
pub const GPU_ATTACHMENT_COLOR0: i32 = 0;
pub const GPU_ATTACHMENT_COLOR1: i32 = 1;
pub const GPU_ATTACHMENT_COLOR2: i32 = 2;
pub const GPU_ATTACHMENT_COLOR3: i32 = 3;
pub const GPU_ATTACHMENT_COLOR4: i32 = 4;
pub const GPU_ATTACHMENT_COLOR5: i32 = 5;
pub const GPU_ATTACHMENT_COLOR6: i32 = 6;
pub const GPU_ATTACHMENT_COLOR7: i32 = 7;
pub const GPU_ATTACHMENT_DEPTH: i32 = 8;
pub const GPU_ATTACHMENT_STENCIL: i32 = 9;
