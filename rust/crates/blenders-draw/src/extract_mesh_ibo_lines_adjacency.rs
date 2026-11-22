/// Port of `extract_mesh_ibo_lines_adjacency.cc`
/// — adjacency-expanded line IBO (for
/// line-thickness geometry shader).
/// `extract_lines_adjacency` (verbatim —
/// per-edge [prev,v1,v2,next]).
pub fn extract_lines_adjacency(
    mr: &super::extract_mesh::MeshRenderData,
    edge_to_lines: &std::collections::HashMap<i32, Vec<i32>>,
) -> Vec<u32> {
    let mut ibo = Vec::with_capacity(mr.edges.len() * 4);
    for (i, &(v1, v2)) in mr.edges.iter().enumerate() {
        let prev = edge_to_lines
            .get(&(i as i32))
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(v1) as u32;
        let next = edge_to_lines
            .get(&(i as i32))
            .and_then(|v| v.get(1))
            .copied()
            .unwrap_or(v2) as u32;
        ibo.extend([prev, v1 as u32, v2 as u32, next]);
    }
    ibo
}
