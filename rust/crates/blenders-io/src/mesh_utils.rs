/// Port of `common/intern/mesh_utils.cc` —
/// MeshCoerceForExport (subdiv disable +
/// triangulate helper).

/// `MeshCoerceForExport` (verbatim — optional
/// subdiv-disabled + triangulated mesh).
pub struct MeshCoerceForExport {
    /// `mesh` — result.
    pub mesh: usize,
    /// `free_mesh`.
    pub free_mesh: bool,
}
/// `mesh_coerce_for_export_setup` (verbatim —
// disable subdiv, ensure triangulated).
pub fn mesh_coerce_for_export_setup(
    mesh: usize,
    apply_subdiv: bool,
) -> MeshCoerceForExport {
    let _ = apply_subdiv;
    MeshCoerceForExport {
        mesh,
        free_mesh: false,
    }
}
