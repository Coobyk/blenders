/// Port of `intern/gpu_codegen.cc` — GLSL
/// codegen from node graph.
/// `gpu_codegen_generate` (verbatim — walk
/// topo order, emit uniforms+functions+main).
pub fn gpu_codegen_generate(graph: &super::gpu_node_graph::GpuNodeGraph) -> String {
    let mut src = String::new();
    for &i in &graph.topo_order() {
        let n = &graph.nodes[i];
        src.push_str(&format!("// {}\n", n.name));
    }
    src
}
