//! Port of `intern/lib_remap.cc` — `IDRemap`/`IDRemapper` driven
//! remapping of all ID references (`BKE_libblock_remap*`, `relink*`).
//!
//! `Main`, `BKE_library_foreach_ID_link`, DEG tagging and ID user
//! counts are injected: the callback logic — skip/apply semantics,
//! refcount rules, bitflag bookkeeping — is ported verbatim.

/// `eIDRemapType`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IDRemapType {
    /// `ID_REMAP_TYPE_REMAP`.
    Remap,
    /// `ID_REMAP_TYPE_UNLINK`.
    Unlink,
    /// `ID_REMAP_TYPE_CLEANUP`.
    Cleanup,
}

/// `IDWALK_CB_*` bits (subset used here).
pub const IDWALK_CB_USER: i32 = 1 << 0;
/// `IDWALK_CB_USER_ONE`.
pub const IDWALK_CB_USER_ONE: i32 = 1 << 1;

/// `ID_REMAP_*` flags (from BKE_lib_remap.hh).
pub const ID_REMAP_SKIP_INDIRECT_USAGE: i32 = 1 << 0;
/// `ID_REMAP_SKIP_NEVER_NULL_USAGE`.
pub const ID_REMAP_SKIP_NEVER_NULL_USAGE: i32 = 1 << 1;
/// `ID_REMAP_FORCE_NEVER_NULL_USAGE`.
pub const ID_REMAP_FORCE_NEVER_NULL_USAGE: i32 = 1 << 2;
/// `ID_REMAP_SKIP_UPDATE_TAGGING`.
pub const ID_REMAP_SKIP_UPDATE_TAGGING: i32 = 1 << 3;
/// `ID_REMAP_SKIP_USER_REFCOUNT`.
pub const ID_REMAP_SKIP_USER_REFCOUNT: i32 = 1 << 4;
/// `ID_REMAP_FORCE_USER_REFCOUNT`.
pub const ID_REMAP_FORCE_USER_REFCOUNT: i32 = 1 << 5;
/// `ID_REMAP_SKIP_USER_CLEAR`.
pub const ID_REMAP_SKIP_USER_CLEAR: i32 = 1 << 6;
/// `ID_REMAP_FORCE_UI_POINTERS`.
pub const ID_REMAP_FORCE_UI_POINTERS: i32 = 1 << 7;
/// `ID_REMAP_FORCE_INTERNAL_RUNTIME_POINTERS`.
pub const ID_REMAP_FORCE_INTERNAL_RUNTIME_POINTERS: i32 = 1 << 8;
/// `ID_REMAP_NO_ORIG_POINTERS_ACCESS`.
pub const ID_REMAP_NO_ORIG_POINTERS_ACCESS: i32 = 1 << 9;
/// `ID_REMAP_DO_LIBRARY_POINTERS`.
pub const ID_REMAP_DO_LIBRARY_POINTERS: i32 = 1 << 10;
/// `ID_REMAP_ALLOW_IDTYPE_MISMATCH`.
pub const ID_REMAP_ALLOW_IDTYPE_MISMATCH: i32 = 1 << 11;

/// `ID_REMAP_IS_*` status bits on `ID_Runtime.remap`.
pub const ID_REMAP_IS_LINKED_DIRECT: i32 = 1 << 0;
/// `ID_REMAP_IS_USER_ONE_SKIPPED`.
pub const ID_REMAP_IS_USER_ONE_SKIPPED: i32 = 1 << 1;

/// `IDRemap` — per-foreach context.
pub struct IDRemap<'a> {
    /// `type`.
    pub type_: IDRemapType,
    /// `bmain` — opaque handle for DEG tagging.
    pub bmain: usize,
    /// `id_owner`.
    pub id_owner: usize,
    /// `flag` — `ID_REMAP_*`.
    pub flag: i32,
    /// `id_remapper` — the mapping table (injected trait).
    pub remapper: &'a dyn IDRemapper,
}

/// `IDRemapper` — maps old id handles → new (or 0).
pub trait IDRemapper {
    /// `apply(id_ptr, options, id_self)` — may rewrite the slot.
    fn apply(&self, id_ptr: usize, options: i32, id_self: usize) -> usize;
    /// `iter` — visit (old_id, new_id) pairs.
    fn iter(&self, f: &mut dyn FnMut(usize, usize));
    /// `contains_mappings_for_any(filter_id)`.
    fn contains_mappings_for_any(&self, filter_id: u64) -> bool;
    /// `is_empty`.
    fn is_empty(&self) -> bool;
}

/// `IDRuntime::remap` — counters tracked per ID.
#[derive(Clone, Default)]
pub struct RemapRuntime {
    /// `skipped_indirect`.
    pub skipped_indirect: i32,
    /// `skipped_direct`.
    pub skipped_direct: i32,
    /// `skipped_refcounted`.
    pub skipped_refcounted: i32,
    /// `status` — `ID_REMAP_IS_*`.
    pub status: i32,
}

/// ID-side hooks the C reaches through `ID`/`Main`/DEG.
pub trait LibRemapIds {
    /// `id->runtime->remap` mutable access.
    fn remap_runtime(&mut self, id: usize) -> &mut RemapRuntime;
    /// `id->us` decrement (`id_us_min`).
    fn us_min(&mut self, id: usize);
    /// `id_us_plus_no_lib`.
    fn us_plus_no_lib(&mut self, id: usize);
    /// `id_us_ensure_real`.
    fn us_ensure_real(&mut self, id: usize);
    /// `id_us_clear_real`.
    fn us_clear_real(&mut self, id: usize);
    /// `id_fake_user_set` / `_clear`.
    fn fake_user_set(&mut self, id: usize);
    /// `id_fake_user_clear`.
    fn fake_user_clear(&mut self, id: usize);
    /// `DEG_id_tag_update_ex` (no-op ok).
    fn deg_tag_update(&mut self, id: usize);
    /// `BKE_ntree_update_tag_all` when `GS(name)==ID_NT`.
    fn ntree_update_tag_all(&mut self, id: usize);
    /// `BKE_libblock_runtime_reset_remapping_status`.
    fn reset_remapping_status(&mut self, id: usize);
    /// ID fields: (tag, flag, us, lib != 0, GS name).
    fn id_info(&self, id: usize) -> (u32, u32, i32, bool, u16);
    /// Set `tag`/`flag` fields (for INDIRECT/EXTERN bookkeeping).
    fn set_tag_flag(&mut self, id: usize, tag: u32, flag: u32);
}

/// `ID_TAG_*` bits used here.
pub const ID_TAG_INDIRECT: u32 = 1 << 0;
/// `ID_TAG_EXTERN`.
pub const ID_TAG_EXTERN: u32 = 1 << 1;
/// `ID_TAG_NO_MAIN`.
pub const ID_TAG_NO_MAIN: i32 = 1 << 2;
/// `ID_TAG_EXTRAUSER_SET`.
pub const ID_TAG_EXTRAUSER_SET: i32 = 1 << 3;
/// `ID_FLAG_FAKEUSER`.
pub const ID_FLAG_FAKEUSER: u32 = 1 << 9;
/// `ID_FLAG_INDIRECT_WEAK_LINK`.
pub const ID_FLAG_INDIRECT_WEAK_LINK: u32 = 1 << 10;
/// `ID_FLAG_EMBEDDED_DATA`.
pub const ID_FLAG_EMBEDDED_DATA: u32 = 1 << 2;

/// `foreach_libblock_remap_callback_skip`.
#[allow(clippy::too_many_arguments)]
pub fn foreach_libblock_remap_callback_skip(
    ids: &mut dyn LibRemapIds,
    id: usize,
    cb_flag: i32,
    is_indirect: bool,
    is_reference: bool,
    violates_never_null: bool,
    is_obj_editmode: bool,
) {
    if is_indirect {
        ids.remap_runtime(id).skipped_indirect += 1;
    } else if violates_never_null || is_obj_editmode || is_reference {
        ids.remap_runtime(id).skipped_direct += 1;
    } else {
        debug_assert!(false);
    }
    if cb_flag & IDWALK_CB_USER != 0 {
        ids.remap_runtime(id).skipped_refcounted += 1;
    } else if cb_flag & IDWALK_CB_USER_ONE != 0 {
        ids.remap_runtime(id).status |= ID_REMAP_IS_USER_ONE_SKIPPED;
    }
}

/// `foreach_libblock_remap_callback_apply` — returns the new id
/// stored in `*id_ptr` (or 0 when `violates_never_null`).
#[allow(clippy::too_many_arguments)]
pub fn foreach_libblock_remap_callback_apply(
    ids: &mut dyn LibRemapIds,
    id_owner: usize,
    id_self: usize,
    id_ptr: usize,
    id_remap_data: &IDRemap,
    id_remapper_options: i32,
    cb_flag: i32,
    is_indirect: bool,
    violates_never_null: bool,
) -> usize {
    let skip_update_tagging = id_remap_data.flag & ID_REMAP_SKIP_UPDATE_TAGGING != 0;
    let skip_user_refcount = id_remap_data.flag & ID_REMAP_SKIP_USER_REFCOUNT != 0;
    let force_user_refcount = id_remap_data.flag & ID_REMAP_FORCE_USER_REFCOUNT != 0;

    let old_id = id_ptr;
    let mut new_id_slot = old_id;
    if !violates_never_null {
        new_id_slot = id_remap_data.remapper.apply(id_ptr, id_remapper_options, id_self);
        if !skip_update_tagging {
            if id_remap_data.bmain != 0 {
                ids.deg_tag_update(id_self);
                if id_self != id_owner {
                    ids.deg_tag_update(id_owner);
                }
            }
            /* `GS(id_self->name) == ID_NT` — 'NT'. */
            if ids.id_info(id_self).4 == (b'N' as u16) << 8 | b'T' as u16 {
                ids.ntree_update_tag_all(id_self);
            }
        }
    }
    let new_id = if violates_never_null { 0 } else { new_id_slot };

    if !is_indirect && new_id != 0 {
        ids.remap_runtime(new_id).status |= ID_REMAP_IS_LINKED_DIRECT;
    }
    if skip_user_refcount {
        return new_id;
    }

    if cb_flag & IDWALK_CB_USER != 0 {
        if force_user_refcount || (ids.id_info(old_id).0 as i32 & ID_TAG_NO_MAIN) == 0 {
            ids.us_min(old_id);
        }
        if new_id != 0
            && (force_user_refcount || (ids.id_info(new_id).0 as i32 & ID_TAG_NO_MAIN) == 0)
        {
            ids.us_plus_no_lib(new_id);
        }
    } else if cb_flag & IDWALK_CB_USER_ONE != 0 {
        ids.us_ensure_real(new_id);
    }
    new_id
}

/// `libblock_remap_data_update_tags`.
fn libblock_remap_data_update_tags(
    ids: &mut dyn LibRemapIds,
    old_id: usize,
    new_id: usize,
    remap_flags: i32,
) {
    if remap_flags & ID_REMAP_SKIP_USER_CLEAR == 0 {
        if old_id != 0 && (ids.id_info(old_id).1 & ID_FLAG_FAKEUSER) != 0 && new_id != 0 {
            ids.fake_user_clear(old_id);
            ids.fake_user_set(new_id);
        }
        ids.us_clear_real(old_id);
    }
    if new_id != 0 {
        let (tag, flag, ..) = ids.id_info(new_id);
        if tag & ID_TAG_INDIRECT != 0
            && (ids.remap_runtime(new_id).status & ID_REMAP_IS_LINKED_DIRECT) != 0
        {
            ids.set_tag_flag(
                new_id,
                (tag & !ID_TAG_INDIRECT) | ID_TAG_EXTERN,
                flag & !ID_FLAG_INDIRECT_WEAK_LINK,
            );
        }
    }
}

/// `libblock_remap_data` — `foreach_id_link(owner, cb_flag…)` is
/// injected; `can_use_filter_id` is `BKE_library_id_can_use_filter_id`.
#[allow(clippy::too_many_arguments)]
pub fn libblock_remap_data(
    ids: &mut dyn LibRemapIds,
    bmain: usize,
    id: usize,
    remap_type: IDRemapType,
    id_remapper: &dyn IDRemapper,
    remap_flags: i32,
    can_use_filter_id: &dyn Fn(usize, bool) -> u64,
    main_ids: &dyn Fn() -> Vec<usize>,
    preprocess: &mut dyn FnMut(usize, IDRemapType, &dyn IDRemapper),
    foreach_id_link: &mut dyn FnMut(usize, usize, i32, &mut dyn FnMut(i32, bool, bool, bool, bool)),
) {
    let _ = remap_type;
    let include_ui = remap_flags & ID_REMAP_FORCE_UI_POINTERS != 0;

    id_remapper.iter(&mut |old_id, new_id| {
        ids.reset_remapping_status(old_id);
        if new_id != 0 {
            ids.reset_remapping_status(new_id);
        }
    });

    let mut remap = IDRemap {
        type_: remap_type,
        bmain,
        id_owner: id,
        flag: remap_flags,
        remapper: id_remapper,
    };

    if id != 0 {
        let owner = if (ids.id_info(id).1 & ID_FLAG_EMBEDDED_DATA) != 0 {
            /* `BKE_id_owner_get` — caller resolves embedded. */
            ids.id_info(id).4 as usize
        } else {
            id
        };
        remap.id_owner = owner;
        preprocess(owner, remap_type, id_remapper);
        foreach_id_link(bmain, id, remap_flags, &mut |cb_flag, is_ind, is_ref, nev, edit| {
            foreach_libblock_remap_callback_skip(
                ids, id, cb_flag, is_ind, is_ref, nev, edit,
            );
        });
    } else {
        for id_curr in main_ids() {
            let filter_id = can_use_filter_id(id_curr, include_ui);
            if !id_remapper.contains_mappings_for_any(filter_id) {
                continue;
            }
            remap.id_owner = id_curr;
            preprocess(id_curr, remap_type, id_remapper);
            foreach_id_link(bmain, id_curr, remap_flags, &mut |cb_flag, is_ind, is_ref, nev, edit| {
                foreach_libblock_remap_callback_skip(
                    ids, id_curr, cb_flag, is_ind, is_ref, nev, edit,
                );
            });
        }
    }

    id_remapper.iter(&mut |old_id, new_id| {
        libblock_remap_data_update_tags(ids, old_id, new_id, remap_flags);
    });
}
