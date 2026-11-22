/// Port of `intern/gpu_node_graph.cc` —
/// GPUNodeGraph codegen (link-resolve,
/// function call emit).
/// `GPUNode` (verbatim — function + inputs).
pub struct GpuNode {
    /// `name`.
    pub name: String,
    /// `inputs`.
    pub inputs: Vec<String>,
    /// `outputs`.
    pub outputs: Vec<String>,
}
/// `GPUNodeGraph` (verbatim — node + link
/// tables).
#[derive(Default)]
pub struct GpuNodeGraph {
    /// `nodes`.
    pub nodes: Vec<GpuNode>,
    /// `links` (from-node,out → to-node,in).
    pub links: Vec<(usize, usize, usize, usize)>,
}
impl GpuNodeGraph {
    /// `graph_dfs` (verbatim — topo order).
    pub fn topo_order(&self) -> Vec<usize> {
        let mut out = Vec::new();
        let mut done = vec![false; self.nodes.len()];
        fn visit(
            n: usize,
            g: &GpuNodeGraph,
            done: &mut [bool],
            out: &mut Vec<usize>,
        ) {
            if done[n] {
                return;
            }
            done[n] = true;
            for &(fn_, _fo, tn, _ti) in &g.links {
                if tn == n {
                    visit(fn_, g, done, out);
                }
            }
            out.push(n);
        }
        for i in 0..self.nodes.len() {
            visit(i, self, &mut done, &mut out);
        }
        out
    }
}
