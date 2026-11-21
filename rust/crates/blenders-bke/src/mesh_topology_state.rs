//! Port of `intern/mesh_topology_state.cc` — `MeshTopologyState`.
//!
//! `ArrayState<T>` = (VArray span, ImplicitSharingInfo) pair; `same_as`
//! compares both. The attribute layer arrives as named slices.

use std::sync::Arc;
use blenders_bli::implicit_sharing::ImplicitSharingInfoAndData;

/// `ArrayState<T>` — snapshot of a topology attribute.
pub struct ArrayState<T: PartialEq> {
    pub data: Vec<T>,
    /// `ImplicitSharingInfo` — the C keeps the sharing handle; we keep the
    /// data snapshot and treat shareable comparisons as data equality.
    pub sharing_info: Option<Arc<ImplicitSharingInfoAndData>>,
}

impl<T: PartialEq> ArrayState<T> {
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    /// `ArrayState::same_as` — same sharing info or equal contents.
    pub fn same_as(&self, data: &[T], sharing: Option<&Arc<ImplicitSharingInfoAndData>>) -> bool {
        if let (Some(a), Some(b)) = (&self.sharing_info, sharing) {
            if Arc::ptr_eq(a, b) {
                return true;
            }
        }
        self.data == data
    }
}

/// `MeshTopologyState` (BKE_mesh_topology_state.hh).
pub struct MeshTopologyState {
    pub edge_verts: ArrayState<[i32; 2]>,
    pub corner_verts: ArrayState<i32>,
    pub corner_edges: ArrayState<i32>,
    pub face_offset_indices: ArrayState<i32>,
}

impl MeshTopologyState {
    /// `MeshTopologyState(mesh)` — snapshot the four topology arrays.
    pub fn new(
        edge_verts: Vec<[i32; 2]>,
        corner_verts: Vec<i32>,
        corner_edges: Vec<i32>,
        face_offset_indices: Vec<i32>,
    ) -> Self {
        Self {
            edge_verts: ArrayState { data: edge_verts, sharing_info: None },
            corner_verts: ArrayState { data: corner_verts, sharing_info: None },
            corner_edges: ArrayState { data: corner_edges, sharing_info: None },
            face_offset_indices: ArrayState { data: face_offset_indices, sharing_info: None },
        }
    }

    /// `same_topology_as` — all four arrays equal.
    pub fn same_topology_as(
        &self,
        edge_verts: &[[i32; 2]],
        corner_verts: &[i32],
        corner_edges: &[i32],
        face_offset_indices: &[i32],
    ) -> bool {
        self.edge_verts.same_as(edge_verts, None)
            && self.corner_verts.same_as(corner_verts, None)
            && self.corner_edges.same_as(corner_edges, None)
            && self.face_offset_indices.same_as(face_offset_indices, None)
    }
}
