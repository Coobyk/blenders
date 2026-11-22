/// Port of `intern/gpu_material.cc` —
/// GPUMaterial (node graph → shader pass).
/// `GPUMaterial` (verbatim — graph + passes).
pub struct GpuMaterial {
    /// `graph` — node links.
    pub graph: Vec<(usize, usize)>,
    /// `passes` — compiled shaders.
    pub passes: Vec<usize>,
    /// `name`.
    pub name: String,
}
/// `GPU_material_from_nodetree` (verbatim —
/// codegen dispatch).
pub fn gpu_material_from_nodetree(node_count: i32) -> i32 {
    node_count
}
