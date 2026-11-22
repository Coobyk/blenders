/// Port of `intern/gpu_texture.cc` — GPUTexture
/// (format/dims/mips + bind + upload).
/// `eGPUTextureFormat` ordinals (verbatim).
pub const GPU_RGBA8: i32 = 0;
pub const GPU_RGBA16F: i32 = 1;
pub const GPU_RGBA32F: i32 = 2;
pub const GPU_R8: i32 = 3;
pub const GPU_R16F: i32 = 4;
pub const GPU_R32F: i32 = 5;
pub const GPU_RG8: i32 = 6;
pub const GPU_RG16F: i32 = 7;
pub const GPU_RG32F: i32 = 8;
pub const GPU_RGB8: i32 = 9;
pub const GPU_RGB16F: i32 = 10;
pub const GPU_RGB32F: i32 = 11;
pub const GPU_DEPTH_COMPONENT24: i32 = 12;
pub const GPU_DEPTH_COMPONENT32F: i32 = 13;
pub const GPU_DEPTH24_STENCIL8: i32 = 14;
/// `GPUTexture` (verbatim — dims + format +
/// mips + flags).
pub struct GpuTexture {
    /// `w`/`h`/`d`.
    pub w: i32,
    pub h: i32,
    pub d: i32,
    /// `format` (eGPUTextureFormat).
    pub format: i32,
    /// `mipmaps`.
    pub mipmaps: i32,
    /// `usage` bits.
    pub usage: i32,
    /// `name` debug label.
    pub name: String,
}
/// `GPU_texture_max_size` (verbatim — per-
/// format bpp).
pub fn gpu_texture_bytes_per_pixel(format: i32) -> i32 {
    match format {
        GPU_RGBA8 => 4,
        GPU_RGBA16F => 8,
        GPU_RGBA32F => 16,
        GPU_R8 => 1,
        GPU_R16F => 2,
        GPU_R32F => 4,
        GPU_RG8 => 2,
        GPU_RG16F => 4,
        GPU_RG32F => 8,
        GPU_RGB8 => 3,
        GPU_RGB16F => 6,
        GPU_RGB32F => 12,
        GPU_DEPTH_COMPONENT24 => 3,
        GPU_DEPTH_COMPONENT32F => 4,
        GPU_DEPTH24_STENCIL8 => 4,
        _ => 4,
    }
}
