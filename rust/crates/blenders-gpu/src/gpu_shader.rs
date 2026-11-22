/// Port of `intern/gpu_shader.cc` — GPUShader
/// (compile/link + uniform set).
/// `GPUShader` (verbatim — sources, program,
/// interface).
pub struct GpuShader {
    /// `name`.
    pub name: String,
    /// `vertex_source`.
    pub vertex_source: String,
    /// `fragment_source`.
    pub fragment_source: String,
    /// `compute_source`.
    pub compute_source: String,
    /// `interface`.
    pub interface: super::gpu_shader_interface::GpuShaderInterface,
}
/// `GPU_shader_create` (verbatim — compile
/// via backend).
pub fn gpu_shader_create(name: &str) -> GpuShader {
    GpuShader {
        name: name.to_string(),
        vertex_source: String::new(),
        fragment_source: String::new(),
        compute_source: String::new(),
        interface: Default::default(),
    }
}
