/// Port of `extract_mesh_ibo_edituv.cc` —
/// UV edit IBO (per-face fan tris + border
/// edges + select points).
/// `extract_edituv` (verbatim — uv-tri +
/// edge + point streams).
pub fn extract_edituv(
    mr: &super::extract_mesh::MeshRenderData,
) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let tris: Vec<u32> = mr
        .loop_tris
        .iter()
        .flat_map(|t| t.iter().map(|&c| c as u32))
        .collect();
    let lines: Vec<u32> = mr
        .edges
        .iter()
        .flat_map(|&(v1, v2)| [v1 as u32, v2 as u32])
        .collect();
    let points: Vec<u32> = (0..mr.corner_verts.len() as u32).collect();
    (tris, lines, points)
}
