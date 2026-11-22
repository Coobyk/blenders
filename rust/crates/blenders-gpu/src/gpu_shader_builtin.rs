/// Port of `intern/gpu_shader_builtin.cc` —
/// `GPU_SHADER_*` builtin id → create-info.
/// `eGPUBuiltinShader` ordinals (verbatim).
pub const GPU_SHADER_TEXT: i32 = 0;
pub const GPU_SHADER_KEYFRAME_SHAPE: i32 = 1;
pub const GPU_SHADER_3D_IMAGE: i32 = 2;
pub const GPU_SHADER_3D_IMAGE_MODULATE_ALPHA: i32 = 3;
pub const GPU_SHADER_3D_IMAGE_COLOR: i32 = 4;
pub const GPU_SHADER_3D_IMAGE_SAFE_COLOR: i32 = 5;
pub const GPU_SHADER_3D_IMAGE_SAFE_AREA: i32 = 6;
pub const GPU_SHADER_2D_COMMON: i32 = 7;
pub const GPU_SHADER_3D_COMMON: i32 = 8;
pub const GPU_SHADER_3D_CLIPPED_UNIFORM_COLOR: i32 = 9;
pub const GPU_SHADER_3D_UNIFORM_COLOR: i32 = 10;
pub const GPU_SHADER_3D_FLAT_COLOR: i32 = 11;
pub const GPU_SHADER_3D_SMOOTH_COLOR: i32 = 12;
pub const GPU_SHADER_3D_DEPTH_ONLY: i32 = 13;
pub const GPU_SHADER_2D_POINT_UNIFORM_SIZE_UNIFORM_COLOR_AA: i32 = 14;
/// `GPU_shader_builtin` (verbatim — id→
/// create-info lookup).
pub fn gpu_shader_builtin(shader: i32) -> i32 {
    shader
}
