//! Port of `intern/anim_data_bmain_utils.cc` — `BKE_animdata_main_cb`.
//!
//! Walks every `Main` list calling `func(id, adt)` for IDs with AnimData,
//! incl. embedded nodetree AnimData. `Main` listbases arrive as id-handle
//! slices; `BKE_animdata_from_id` is passed in (AnimData lookup per ID type
//! is the caller's until Main lands).

/// `ANIMDATA_IDS_CB` — iterate ids, call `func` for ones with AnimData.
pub fn animdata_ids_cb(ids: &[usize], adt_of: &dyn Fn(usize) -> Option<usize>, func: &mut dyn FnMut(usize, usize)) {
    for &id in ids {
        if let Some(adt) = adt_of(id) {
            func(id, adt);
        }
    }
}

/// `ANIMDATA_NODETREE_IDS_CB` — like above plus the embedded `nodetree`'s
/// AnimData (`ntp->nodetree` handle per id, `NIL` when absent).
pub fn animdata_nodetree_ids_cb(
    ids: &[usize],
    adt_of: &dyn Fn(usize) -> Option<usize>,
    nodetree_of: &dyn Fn(usize) -> usize,
    func: &mut dyn FnMut(usize, usize),
) {
    for &id in ids {
        let adt = adt_of(id);
        let nt = nodetree_of(id);
        if nt != blenders_dna::id::NIL {
            if let Some(adt2) = adt_of(nt) {
                func(nt, adt2);
            }
        }
        if let Some(adt) = adt {
            func(id, adt);
        }
    }
}
