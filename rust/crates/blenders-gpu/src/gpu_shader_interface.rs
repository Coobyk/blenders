/// Port of `intern/gpu_shader_interface.cc` —
/// reflected uniform/ubo/ssbo/input/output
/// table.
/// `GPUShaderInterface` (verbatim — name→
/// location lists).
#[derive(Default)]
pub struct GpuShaderInterface {
    /// `uniforms` (name → loc).
    pub uniforms: Vec<(String, i32)>,
    /// `ubos`.
    pub ubos: Vec<(String, i32)>,
    /// `ssbos`.
    pub ssbos: Vec<(String, i32)>,
    /// `inputs`.
    pub inputs: Vec<(String, i32)>,
    /// `outputs`.
    pub outputs: Vec<(String, i32)>,
}
impl GpuShaderInterface {
    /// `GPU_shaderinterface_uniform_lookup`
    /// (verbatim — name→loc linear).
    pub fn uniform_lookup(&self, name: &str) -> i32 {
        self.uniforms
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, l)| *l)
            .unwrap_or(-1)
    }
}
