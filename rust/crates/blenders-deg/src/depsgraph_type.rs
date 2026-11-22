//! Port of `intern/depsgraph_type.cc` —
/// `DEG_register_node_types`/`free` + `DEGCustomDataMeshMasks`.

/// `DEGCustomDataMeshMasks`.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct DegCustomDataMeshMasks {
    /// `vert_mask` (`vmask`).
    pub vert_mask: u64,
    /// `edge_mask` (`emask`).
    pub edge_mask: u64,
    /// `face_mask` (`fmask`).
    pub face_mask: u64,
    /// `loop_mask` (`lmask`).
    pub loop_mask: u64,
    /// `poly_mask` (`pmask`).
    pub poly_mask: u64,
}

impl DegCustomDataMeshMasks {
    /// `DEGCustomDataMeshMasks(const CustomData_MeshMasks *)`.
    pub fn from_mesh_masks(vmask: u64, emask: u64, fmask: u64, lmask: u64, pmask: u64) -> Self {
        Self {
            vert_mask: vmask,
            edge_mask: emask,
            face_mask: fmask,
            loop_mask: lmask,
            poly_mask: pmask,
        }
    }
}

/// `DEG_register_node_types` — calls the three per-class
/// registrars (injected so base/component/operation node
/// modules register their factories).
pub fn deg_register_node_types(
    register_base: &mut dyn FnMut(),
    register_component: &mut dyn FnMut(),
    register_operation: &mut dyn FnMut(),
) {
    register_base();
    register_component();
    register_operation();
}

/// `DEG_free_node_types` (verbatim — no-op).
pub fn deg_free_node_types() {}
