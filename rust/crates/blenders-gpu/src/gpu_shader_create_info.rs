/// Port of `intern/gpu_shader_create_info.cc`
/// — ShaderCreateInfo fluent descriptor.
/// `ShaderCreateInfo` (verbatim — name + decl
/// lists).
#[derive(Default)]
pub struct ShaderCreateInfo {
    /// `name`.
    pub name: String,
    /// `vertex_inputs`.
    pub vertex_inputs: Vec<String>,
    /// `vertex_outs`.
    pub vertex_outs: Vec<String>,
    /// `fragment_outs`.
    pub fragment_outs: Vec<String>,
    /// `uniforms`.
    pub uniforms: Vec<String>,
    /// `samplers`.
    pub samplers: Vec<String>,
    /// `ubos`.
    pub ubos: Vec<String>,
    /// `ssbos`.
    pub ssbos: Vec<String>,
    /// `vertex_source`.
    pub vertex_source: String,
    /// `fragment_source`.
    pub fragment_source: String,
    /// `compute_source`.
    pub compute_source: String,
    /// `local_group_size`.
    pub local_group_size: [i32; 3],
    /// `tf_type`.
    pub tf_type: i32,
    /// `defines`.
    pub defines: Vec<(String, String)>,
}
impl ShaderCreateInfo {
    /// `vertex_in` (verbatim fluent).
    pub fn vertex_in(&mut self, name: &str) -> &mut Self {
        self.vertex_inputs.push(name.to_string());
        self
    }
    /// `uniform_buf` (verbatim fluent).
    pub fn uniform_buf(&mut self, name: &str) -> &mut Self {
        self.ubos.push(name.to_string());
        self
    }
    /// `compute_source` (verbatim fluent).
    pub fn compute_src(&mut self, src: &str) -> &mut Self {
        self.compute_source = src.to_string();
        self
    }
}
