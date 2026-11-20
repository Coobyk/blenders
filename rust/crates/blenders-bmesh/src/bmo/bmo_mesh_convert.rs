//! Port of `bmo_mesh_convert.cc` — mesh<->bmesh operator shells.
//! All three ops need `Object`, `Mesh`, `G.main` — PORT_PENDING
//! (they call `convert::bm_mesh_bm_*` stubs which panic until
//! `blenders-bke` lands).
use crate::{BMesh};
use crate::operators::*;

/// `bmo_mesh_to_bmesh_exec` — PORT_PENDING (needs Object/Mesh/Key).
pub fn bmo_mesh_to_bmesh_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let _ob = bmo_slot_ptr_get(op, false, "object");
    let _mesh = bmo_slot_ptr_get(op, false, "mesh");
    let set_key = bmo_slot_bool_get(op, false, "use_shapekey");
    let mut params = crate::convert::BMeshFromMeshParams::default();
    params.use_shapekey = set_key;
    /* params.active_shapekey = ob->shapenr — needs Object. */
    crate::convert::bm_mesh_bm_from_me(bm, &params);
    let _ = bm;
}
/// `bmo_object_load_bmesh_exec` — PORT_PENDING (needs Object/Mesh).
pub fn bmo_object_load_bmesh_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let _ob = bmo_slot_ptr_get(op, false, "object");
    /* mesh = id_cast<Mesh*>(ob->data); bmesh_to_mesh mesh=%p object=%p */
    let _ = bm;
    unimplemented!("PORT_PENDING: needs Object/Mesh (blenders-bke)");
}
/// `bmo_bmesh_to_mesh_exec` — PORT_PENDING (needs Mesh + G.main).
pub fn bmo_bmesh_to_mesh_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let _mesh = bmo_slot_ptr_get(op, false, "mesh");
    let mut params = crate::convert::BMeshToMeshParams::default();
    params.calc_object_remap = true;
    crate::convert::bm_mesh_bm_to_me(bm, &params);
}
