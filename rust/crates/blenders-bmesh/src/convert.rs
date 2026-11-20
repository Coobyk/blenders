//! Port of `bmesh_mesh_convert.cc` — BMesh ↔ Mesh conversion.
//!
//! The conversion needs `Mesh`, `CustomData`, `Main` and depsgraph types,
//! none of which are ported yet, so the bulk of this file is a faithful
//! API skeleton: every public/entry function exists and returns/panics
//! via `unimplemented!()` until `blenders-bke` lands (`Mesh`, `CustomData`,
//! shape keys). What IS fully ported: `BM_attribute_stored_in_bmesh_builtin`
//! and the params structs.

use crate::BMesh;

/// `BM_attribute_stored_in_bmesh_builtin` — attribute names that map onto
/// built-in BMesh state rather than custom layers.
pub fn bm_attribute_stored_in_bmesh_builtin(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if name.as_bytes()[0] == b'.' {
        return matches!(
            name,
            ".edge_verts"
                | ".corner_vert"
                | ".corner_edge"
                | ".hide_vert"
                | ".hide_edge"
                | ".hide_poly"
                | ".select_vert"
                | ".select_edge"
                | ".select_poly"
                | ".uv_select_vert"
                | ".uv_select_edge"
                | ".uv_select_face"
        );
    }
    matches!(
        name,
        "position" | "uv_seam" | "material_index" | "sharp_face" | "sharp_edge"
    )
}

/// `BMeshFromMeshParams` (bmesh_mesh_convert.hh).
#[derive(Clone, Copy, Default)]
pub struct BMeshFromMeshParams {
    pub calc_face_normal: bool,
    pub calc_vert_normal: bool,
    /// add a vertex CD_SHAPE_KEYINDEX layer.
    pub add_key_index: bool,
    /// set vertex coordinates from the shapekey.
    pub use_shapekey: bool,
    /// the active shape key (index + 1).
    pub active_shapekey: i32,
    /* `CustomData_MeshMasks cd_mask_extra` — PORT_PENDING (CustomData). */
}

/// `BMeshToMeshParams`.
#[derive(Clone, Copy, Default)]
pub struct BMeshToMeshParams {
    /// Update object hook indices & vertex parents.
    pub calc_object_remap: bool,
    /// Re-assign shape-key indices (needed when the BMesh keeps being used
    /// to update mesh/shape keys; see upstream #71865).
    pub update_shapekey_indices: bool,
    /// Copy BMVert.co directly to the Mesh position array (undo reads).
    pub active_shapekey_to_mvert: bool,
    /* `CustomData_MeshMasks cd_mask_extra` — PORT_PENDING. */
}

/* PORT_PENDING — these need `Mesh`, `CustomData`, `Main`, `KeyBlock` and the
 * object-remap API (`BKE_mesh`, `DEG_depsgraph_query`). Port targets:
 *   BM_mesh_bm_from_me            (Mesh → BMesh, ~420 lines)
 *   BM_mesh_bm_to_me              (BMesh → Mesh, ~300 lines + helpers)
 *   mesh_bm_to_me_compact         (compact remaps, ~250 lines)
 *   BM_mesh_bm_to_me_for_eval
 *   BM_mesh_bm_to_me_only_select_and_hide
 *   bm_to_mesh_shape / shape-layer helpers
 *   bmesh_to_mesh_calc_object_remap
 * They will be ported when `blenders-bke` provides `Mesh`/`CustomData`.
 */

/// `BM_mesh_bm_from_me` — unimplemented until `Mesh`/`CustomData` are ported.
pub fn bm_mesh_bm_from_me(_bm: &mut BMesh, _params: &BMeshFromMeshParams) {
    unimplemented!("PORT_PENDING: needs Mesh + CustomData (blenders-bke)")
}
/// `BM_mesh_bm_to_me` — unimplemented until `Mesh`/`CustomData` are ported.
pub fn bm_mesh_bm_to_me(_bm: &mut BMesh, _params: &BMeshToMeshParams) {
    unimplemented!("PORT_PENDING: needs Mesh + CustomData (blenders-bke)")
}
