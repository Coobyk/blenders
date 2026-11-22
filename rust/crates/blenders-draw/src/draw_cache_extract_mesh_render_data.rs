/// Port of `draw_cache_extract_mesh_render_data.cc`
/// — `MeshRenderData` build from
/// `Mesh`/`BMesh` (positions, edges,
/// faces, loop-tris, loose sets).
/// `mesh_render_data_create` (verbatim —
/// fill fields + loose sets).
pub fn mesh_render_data_create(
    positions: Vec<[f32; 3]>,
    edges: Vec<(i32, i32)>,
    faces: Vec<(i32, i32)>,
    corner_verts: Vec<i32>,
    loop_tris: Vec<[i32; 3]>,
    edge_in_face: &[bool],
    vert_in_edge: &[bool],
) -> super::extract_mesh::MeshRenderData {
    let mut loose_edges = Vec::new();
    for (i, &in_f) in edge_in_face.iter().enumerate() {
        if !in_f {
            loose_edges.push(i as i32);
        }
    }
    let mut loose_verts = Vec::new();
    for (i, &in_e) in vert_in_edge.iter().enumerate() {
        if !in_e {
            loose_verts.push(i as i32);
        }
    }
    super::extract_mesh::MeshRenderData {
        vert_positions: positions,
        edges,
        faces,
        corner_verts,
        loop_tris,
        loose_edges,
        loose_verts,
        efa_act: -1,
        efa_act_uv: -1,
    }
}
/// `edituv_overlay_position` helpers
/// (verbatim — per-corner uv offsets).
pub struct BmUvOffsets {
    /// `uv`/`pin`/`select`/`edge_select`
    /// CD layer offsets.
    pub uv: i32,
    pub pin: i32,
    pub select: i32,
    pub edge_select: i32,
}
