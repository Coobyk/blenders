/// Port of `intern/gpu_state.cc` — GPUState
/// manager (blend/depth/stencil/write/cull/
/// program-point/size).
/// `eGPUBlend` ordinals (verbatim).
pub const GPU_BLEND_NONE: i32 = 0;
pub const GPU_BLEND_ALPHA: i32 = 1;
pub const GPU_BLEND_ADDITIVE: i32 = 2;
pub const GPU_BLEND_MODULATE: i32 = 3;
pub const GPU_BLEND_SUBTRACT: i32 = 4;
pub const GPU_BLEND_ADDITIVE_PREMULT: i32 = 5;
pub const GPU_BLEND_MULTIPLY: i32 = 6;
pub const GPU_BLEND_INVERT: i32 = 7;
pub const GPU_BLEND_OIT_PREFER_LESS: i32 = 8;
pub const GPU_BLEND_CUSTOM: i32 = 9;
/// `eGPUDepthTest` ordinals (verbatim).
pub const GPU_DEPTH_TEST_NONE: i32 = 0;
pub const GPU_DEPTH_TEST_ALWAYS: i32 = 1;
pub const GPU_DEPTH_TEST_LESS: i32 = 2;
pub const GPU_DEPTH_TEST_LESS_EQUAL: i32 = 3;
pub const GPU_DEPTH_TEST_EQUAL: i32 = 4;
pub const GPU_DEPTH_TEST_GREATER: i32 = 5;
pub const GPU_DEPTH_TEST_GREATER_EQUAL: i32 = 6;
/// `eGPUStencilTest` ordinals (verbatim).
pub const GPU_STENCIL_TEST_NONE: i32 = 0;
pub const GPU_STENCIL_TEST_EQUAL: i32 = 1;
pub const GPU_STENCIL_TEST_NEQUAL: i32 = 2;
pub const GPU_STENCIL_TEST_GREATER: i32 = 3;
/// `eGPUStencilOp` ordinals (verbatim).
pub const GPU_STENCIL_OP_NONE: i32 = 0;
pub const GPU_STENCIL_OP_REPLACE: i32 = 1;
pub const GPU_STENCIL_OP_COUNT: i32 = 2;
pub const GPU_STENCIL_OP_COUNT_CLAMPED: i32 = 3;
/// `GPUState` (verbatim — current blend/
/// depth/stencil/write_mask/provoking/
/// cull/program-point/size).
#[derive(Default)]
pub struct GpuState {
    /// `blend`.
    pub blend: i32,
    /// `depth_test`.
    pub depth_test: i32,
    /// `stencil_test`.
    pub stencil_test: i32,
    /// `stencil_op`.
    pub stencil_op: i32,
    /// `write_mask` (RGBA bits).
    pub write_mask: i32,
    /// `provoking_vert` (first/last).
    pub provoking_vert: i32,
    /// `cull_test` (back/front/none).
    pub cull_test: i32,
    /// `program_point_size`.
    pub program_point_size: bool,
}
/// `GPUStateMutable` (verbatim — dynamic
/// state: depth_bias/stencil refs/line
/// width/scissor).
#[derive(Default)]
pub struct GpuStateMutable {
    /// `depth_bias`.
    pub depth_bias: i32,
    /// `stencil_write_mask`.
    pub stencil_write_mask: i32,
    /// `stencil_compare_mask`.
    pub stencil_compare_mask: i32,
    /// `stencil_reference`.
    pub stencil_reference: i32,
}
