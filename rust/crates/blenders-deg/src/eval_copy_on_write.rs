//! Port of `intern/eval/deg_eval_copy_on_write.cc` —
/// evaluated-copy create/update/free/validate.

use crate::depsgraph::ID_TAG_COPIED_ON_EVAL;
use crate::depsgraph_query::ID_TAG_LOCALIZED;
use blenders_dna::id::gs;

/// `ID_TYPE_USE_COPY_ON_EVAL` verbatim (DNA_ID.h id-code
/// filter — types that get deep evaluated copies).
pub fn id_type_use_copy_on_eval(id_type: u16) -> bool {
    /* Verbatim from `DNA_ID.h`: all except library, WM,
     * screen, VFont-less data handled by shallow copies. */
    !matches!(
        id_type,
        t if t == gs(b"LI") || t == gs(b"SR") || t == gs(b"VF")
            || t == gs(b"BR") || t == gs(b"WM") || t == gs(b"PL")
            || t == gs(b"PC") || t == gs(b"WS") || t == gs(b"IM")
    )
}

/// `check_datablock_expanded` (verbatim — expanded copies
/// keep their name set; freed ones have `name[0]=='\0'`).
pub fn check_datablock_expanded(id_name_first_byte: u8) -> bool {
    id_name_first_byte != 0
}

/// `deg_eval_copy_is_expanded` (verbatim).
pub fn deg_eval_copy_is_expanded(id_name_first_byte: u8) -> bool {
    check_datablock_expanded(id_name_first_byte)
}

/// `deg_eval_copy_is_needed` (verbatim).
pub fn deg_eval_copy_is_needed(id_type: u16) -> bool {
    id_type_use_copy_on_eval(id_type)
}

/// `deg_tag_eval_copy_id` (verbatim — sets tag/orig_id/
/// depsgraph back-pointer on the cow id).
pub fn deg_tag_eval_copy_id(
    cow_tag: &mut i32,
    cow_orig_id: &mut usize,
    cow_depsgraph: &mut usize,
    orig_id: usize,
    depsgraph: usize,
) {
    *cow_tag |= ID_TAG_COPIED_ON_EVAL as i32;
    *cow_tag &= !(ID_TAG_LOCALIZED as i32);
    *cow_orig_id = orig_id;
    *cow_depsgraph = depsgraph;
}

/// Hooks for the heavy DNA copy/remap machinery — verbatim
/// entry points keep control flow here while `BKE_id_copy`,
/// `BKE_library_foreach_ID_link`, scene/layer helpers and
/// the per-type edit-mode pointer updates are injected.
pub struct CowHooks<'a> {
    /// `id->name[0]` read (expansion check).
    pub id_name0: &'a dyn Fn(usize) -> u8,
    /// `GS(id->name)`.
    pub id_gs: &'a dyn Fn(usize) -> u16,
    /// `id->recalc` get/set.
    pub id_recalc: &'a mut dyn FnMut(usize, Option<u32>) -> u32,
    /// `id_node->id_orig`/`id_cow`.
    pub node_ids: &'a dyn Fn(usize) -> (usize, usize),
    /// `id_node->is_cow_explicitly_tagged`.
    pub is_cow_explicitly_tagged: &'a dyn Fn(usize) -> bool,
    /// `id->py_instance`/`id->runtime` null checks.
    pub id_py_runtime_null: &'a dyn Fn(usize) -> bool,
    /// `scene_copy_inplace_no_main` (SCE only) → done.
    pub scene_copy_inplace: &'a mut dyn FnMut(usize, usize) -> bool,
    /// `scene_setup_view_layers_before_remap`.
    pub scene_setup_view_layers_before_remap: &'a mut dyn FnMut(usize),
    /// `id_copy_inplace_no_main` → done.
    pub id_copy_inplace: &'a mut dyn FnMut(usize, usize) -> bool,
    /// `BKE_library_foreach_ID_link` with remap callback —
    /// the callee walks links and remaps cow→cow ids.
    pub foreach_id_link_remap: &'a mut dyn FnMut(usize),
    /// `update_id_after_copy` (per-type pointer updates).
    pub update_id_after_copy: &'a mut dyn FnMut(usize, usize, usize),
    /// `update_edit_mode_pointers` (edit-mode update path).
    pub update_edit_mode_pointers: &'a mut dyn FnMut(usize, usize),
    /// `BKE_object_data_is_in_editmode`.
    pub data_is_in_editmode: &'a dyn Fn(usize) -> bool,
    /// `RuntimeBackup` init/restore for the cow id.
    pub backup_init: &'a mut dyn FnMut(usize),
    /// `backup.restore_to_id`.
    pub backup_restore: &'a mut dyn FnMut(usize),
    /// Free pieces of `deg_free_eval_copy_datablock`:
    /// object data/sculpt_session reset.
    pub object_reset_data_sculpt: &'a mut dyn FnMut(usize),
    /// `discard_edit_mode_pointers` per type.
    pub discard_edit_mode_pointers: &'a mut dyn FnMut(usize),
    /// `BKE_libblock_free_data_py/datablock/data`.
    pub free_libblock: &'a mut dyn FnMut(usize),
    /// `id_cow->name[0] = '\0'`.
    pub set_name0_empty: &'a mut dyn FnMut(usize),
    /// `BKE_library_foreach_ID_link` validate — returns
    /// false on invalid link.
    pub foreach_id_link_validate: &'a mut dyn FnMut(usize) -> bool,
    /// `deg_tag_eval_copy_id` concrete writes.
    pub tag_eval_copy_id: &'a mut dyn FnMut(usize, usize),
    /// Scene-id check for `deg_create_eval_copy`.
    pub scene_id: &'a dyn Fn() -> usize,
}

/// `deg_expand_eval_copy_datablock` (verbatim — copy +
/// view-layer setup + remap + after-copy updates).
pub fn deg_expand_eval_copy_datablock(id_node: usize, h: &mut CowHooks) -> usize {
    let (id_orig, id_cow) = (h.node_ids)(id_node);
    let id_cow_recalc = (h.id_recalc)(id_cow, None);
    if !deg_eval_copy_is_needed((h.id_gs)(id_orig)) {
        return id_cow;
    }
    debug_assert!(!check_datablock_expanded((h.id_name0)(id_cow)));
    debug_assert!((h.id_py_runtime_null)(id_cow));
    let mut done = false;
    let id_type = (h.id_gs)(id_orig);
    if id_type == gs(b"SC") {
        done = (h.scene_copy_inplace)(id_orig, id_cow);
        if done {
            (h.scene_setup_view_layers_before_remap)(id_cow);
        }
    }
    if !done {
        done = (h.id_copy_inplace)(id_orig, id_cow);
    }
    debug_assert!(done, "No idea how to perform evaluated copy on datablock");
    /* Tag as evaluated copy before remapping. */
    (h.tag_eval_copy_id)(id_cow, id_orig);
    (h.foreach_id_link_remap)(id_cow);
    (h.update_id_after_copy)(id_node, id_orig, id_cow);
    (h.id_recalc)(id_cow, Some(id_cow_recalc));
    id_cow
}

/// `deg_update_eval_copy_datablock` (verbatim).
pub fn deg_update_eval_copy_datablock(id_node: usize, h: &mut CowHooks) -> usize {
    let (id_orig, id_cow) = (h.node_ids)(id_node);
    if !deg_eval_copy_is_needed((h.id_gs)(id_orig)) {
        return id_cow;
    }
    /* Edit-mode objects don't need a re-copy, just pointer
     * updates. */
    if check_datablock_expanded((h.id_name0)(id_cow))
        && !(h.is_cow_explicitly_tagged)(id_node)
        && (h.data_is_in_editmode)(id_orig)
    {
        (h.update_edit_mode_pointers)(id_orig, id_cow);
        return id_cow;
    }
    (h.backup_init)(id_cow);
    deg_free_eval_copy_datablock(id_cow, h);
    deg_expand_eval_copy_datablock(id_node, h);
    (h.backup_restore)(id_cow);
    id_cow
}

/// `deg_free_eval_copy_datablock` (verbatim).
pub fn deg_free_eval_copy_datablock(id_cow: usize, h: &mut CowHooks) {
    if !check_datablock_expanded((h.id_name0)(id_cow)) {
        return;
    }
    if (h.id_gs)(id_cow) == gs(b"OB") {
        (h.object_reset_data_sculpt)(id_cow);
    }
    (h.discard_edit_mode_pointers)(id_cow);
    (h.free_libblock)(id_cow);
    (h.set_name0_empty)(id_cow);
}

/// `deg_create_eval_copy` (verbatim — scene handled by
/// eval-ctx setup).
pub fn deg_create_eval_copy(id_node: usize, h: &mut CowHooks) {
    let (id_orig, _id_cow) = (h.node_ids)(id_node);
    if id_orig == (h.scene_id)() {
        return;
    }
    deg_update_eval_copy_datablock(id_node, h);
}

/// `deg_validate_eval_copy_datablock` (verbatim).
pub fn deg_validate_eval_copy_datablock(id_cow: usize, h: &mut CowHooks) -> bool {
    if id_cow == 0 {
        return false;
    }
    (h.foreach_id_link_validate)(id_cow)
}
