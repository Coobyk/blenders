/// Port of `intern/gpu_pass.cc` — GPUPass
/// (compiled program + uniform dispatch).
/// `GPUPass` (verbatim — shader ref +
/// interface).
pub struct GpuPass {
    /// `shader`.
    pub shader: usize,
    /// `interface`.
    pub interface: super::gpu_shader_interface::GpuShaderInterface,
}
