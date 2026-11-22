//! Port of `intern/mesh_selection.cc` —
//! grow/shrink/random selection ops on
//! vert/edge/face masks.

/// `select_more` (verbatim — any-vertex
// boundary flood +1 ring).
pub fn mesh_selection_grow(
    selection: &[bool],
    corner_verts: &[i32],
    corner_edges: &[i32],
    edges: &[[i32; 2]],
) -> Vec<bool> {
    let mut out = selection.to_vec();
    let mut vert_sel: Vec<bool> = vec![false; selection.len().max(1)];
    // Seed vert selection from corner→vert.
    for (i, &v) in corner_verts.iter().enumerate() {
        if selection.get(i).copied().unwrap_or(false) {
            vert_sel[v as usize] = true;
        }
    }
    // Grow along edges.
    for e in edges {
        let a = vert_sel[e[0] as usize];
        let b = vert_sel[e[1] as usize];
        if a != b {
            vert_sel[e[0] as usize] = true;
            vert_sel[e[1] as usize] = true;
        }
    }
    for (i, &v) in corner_verts.iter().enumerate() {
        if vert_sel[v as usize] {
            out[i] = true;
        }
    }
    out
}
/// `select_less` (verbatim — invert grow of
// inverse).
pub fn mesh_selection_shrink(
    selection: &[bool],
    corner_verts: &[i32],
    edges: &[[i32; 2]],
) -> Vec<bool> {
    let inv: Vec<bool> = selection.iter().map(|s| !s).collect();
    let grown = mesh_selection_grow(&inv, corner_verts, &[], edges);
    grown.into_iter().map(|s| !s).collect()
}
