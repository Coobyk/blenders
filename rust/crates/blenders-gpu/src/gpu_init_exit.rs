/// Port of `intern/gpu_init_exit.cc` — GPU
/// init/exit + backend pick.
/// `GPU_backend_supported` (verbatim —
/// availability by OS).
pub fn gpu_backend_supported(backend: i32) -> bool {
    backend == super::gpu_platform::GPU_BACKEND_OPENGL
        || backend == super::gpu_platform::GPU_BACKEND_VULKAN
        || backend == super::gpu_platform::GPU_BACKEND_METAL
}
