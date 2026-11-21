//! Port of `intern/mesh_debug.cc` — `BKE_mesh_debug_info`/`BKE_mesh_debug_print`.
//!
//! Debug-only (NDEBUG) helpers; ported against mesh attribute accessors via
//! plain counts (CustomData printing is PORT_PENDING — it needs
//! `CustomData_type_to_name` and attribute iteration).

/// `BKE_mesh_debug_info` — summary string like the C version emits.
pub fn bke_mesh_debug_info(verts: usize, edges: usize, corners: usize, faces: usize) -> String {
    format!(
        "{{\n  'verts_num': {},\n  'edges_num': {},\n  'corners_num': {},\n  'faces_num': {},\n}}",
        verts, edges, corners, faces
    )
}

/// `BKE_mesh_debug_print`.
pub fn bke_mesh_debug_print(verts: usize, edges: usize, corners: usize, faces: usize) {
    print!("{}", bke_mesh_debug_info(verts, edges, corners, faces));
}
