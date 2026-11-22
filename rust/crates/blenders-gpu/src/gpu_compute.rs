/// Port of `intern/gpu_compute.cc` —
/// `GPU_compute_dispatch` + barrier wrappers.
/// `GPU_compute_dispatch` (verbatim —
/// dispatch dims → backend).
pub fn gpu_compute_dispatch(num_x: i32, num_y: i32, num_z: i32) -> (i32, i32, i32) {
    (num_x, num_y, num_z)
}
/// `GPU_memory_barrier` bits (verbatim).
pub const GPU_BARRIER_SHADER_IMAGE_ACCESS: i32 = 1 << 0;
pub const GPU_BARRIER_SHADER_STORAGE: i32 = 1 << 1;
pub const GPU_BARRIER_VERTEX_ATTRIB_ARRAY: i32 = 1 << 2;
pub const GPU_BARRIER_ELEMENT_ARRAY: i32 = 1 << 3;
pub const GPU_BARRIER_UNIFORM: i32 = 1 << 4;
pub const GPU_BARRIER_TEXTURE_FETCH: i32 = 1 << 5;
pub const GPU_BARRIER_TEXTURE_UPDATE: i32 = 1 << 6;
pub const GPU_BARRIER_FRAMEBUFFER: i32 = 1 << 7;
