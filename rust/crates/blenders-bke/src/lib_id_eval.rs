//! Port of `intern/lib_id_eval.cc` — `BKE_id_eval_properties_copy`.
//!
//! Copies eval-time properties for ID types supporting param copies without
//! CoW. Currently only `ID_ME` (Mesh) — dispatch on the id code, calling
//! `BKE_mesh_copy_parameters` (PORT_PENDING body until mesh internals land).

use blenders_dna::id_enums::ID_ME;

/// `BKE_id_eval_properties_copy(ID *id_cow, ID *id)`.
///
/// # Safety
/// Both pointers must be live `ID`s; `id_cow` must have `ID_TAG_COPIED_ON_EVAL`
/// and `id` must not (mirroring the C asserts).
pub unsafe fn BKE_id_eval_properties_copy(id_cow: *mut blenders_dna::id::ID, id: *const blenders_dna::id::ID) {
    unsafe {
        let nb = (*id).name;
        let id_type = blenders_dna::id::gs(nb[..2].try_into().unwrap());
        debug_assert!((*id_cow).tag & blenders_dna::id::ID_TAG_COPIED_ON_EVAL != 0);
        debug_assert!((*id).tag & blenders_dna::id::ID_TAG_COPIED_ON_EVAL == 0);
        if id_type == ID_ME as blenders_dna::id::IDType {
            // BKE_mesh_copy_parameters(dst, src) — PORT_PENDING.
        } else {
            debug_assert!(false, "unreachable: id type without param-copy support");
        }
    }
}
