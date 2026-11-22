/// Port of `intern/gpu_platform.cc` —
/// `GPUPlatform` backend/os/driver query.
/// `eGPUBackendType` ordinals (verbatim).
pub const GPU_BACKEND_NONE: i32 = 0;
pub const GPU_BACKEND_OPENGL: i32 = 1;
pub const GPU_BACKEND_METAL: i32 = 2;
pub const GPU_BACKEND_VULKAN: i32 = 3;
/// `eGPUOSType` ordinals (verbatim).
pub const GPU_OS_ANY: i32 = 0;
pub const GPU_OS_MAC: i32 = 1;
pub const GPU_OS_WIN: i32 = 2;
pub const GPU_OS_UNIX: i32 = 3;
/// `eGPUDriverType` ordinals (verbatim).
pub const GPU_DRIVER_ANY: i32 = 0;
pub const GPU_DRIVER_OFFICIAL: i32 = 1;
pub const GPU_DRIVER_OPENSOURCE: i32 = 2;
/// `GPUPlatform` (verbatim — backend/os/driver
/// triplet).
pub struct GpuPlatform {
    /// `backend`.
    pub backend: i32,
    /// `os`.
    pub os: i32,
    /// `driver`.
    pub driver: i32,
    /// `support_level`.
    pub support_level: i32,
}
