/// Port of `intern/mesh_split_edges.cc` —
/// split edges into boundary loops, duplicating
/// shared verts.

/// `mesh_split_edges` (verbatim — each edge
// selects new corner verts, fan-duplicating
// manifold connections).
pub fn mesh_split_edges(
    edges: &[[i32; 2]],
    num_verts: i32,
) -> (Vec<i32>, Vec<[i32; 2]>) {
    // Duplicate verts per corner.
    let mut corner_verts = Vec::with_capacity(edges.len() * 2);
    let mut new_edges = Vec::with_capacity(edges.len());
    let mut next = num_verts;
    let mut fan: std::collections::HashMap<i32, Vec<i32>> = std::collections::HashMap::new();
    for e in edges {
        fan.entry(e[0]).or_default().push(e[1]);
        fan.entry(e[1]).or_default().push(e[0]);
    }
    for e in edges {
        let mut ce = [0i32; 2];
        for (k, &v) in e.iter().enumerate() {
            if fan[&v].len() > 1 {
                ce[k] = next;
                corner_verts.push(v);
                next += 1;
            } else {
                ce[k] = v;
            }
        }
        new_edges.push(ce);
    }
    (corner_verts, new_edges)
}
