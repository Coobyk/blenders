//! Port of `intern/lib_id_delete.cc` — ID freeing & batch deletion.
//!
//! Everything touching `Main` (listbases, namemap, invariants,
//! collection sync), lib_remap/unlink, anim/idprop/override freeing,
//! depsgraph tags and Python is injected through `IdDeleteHooks` —
//! the control flow matches the C exactly.

use std::collections::HashSet;

/// `ID_TAG_*`.
pub const ID_TAG_NO_MAIN: i32 = 1 << 0;
pub const ID_TAG_NO_USER_REFCOUNT: i32 = 1 << 1;
pub const ID_TAG_NOT_ALLOCATED: i32 = 1 << 2;
pub const ID_TAG_DOIT: i32 = 1 << 3;

/// `LIB_ID_FREE_*`.
pub const LIB_ID_FREE_NO_UI_USER: i32 = 1 << 0;
pub const LIB_ID_FREE_NO_MAIN: i32 = 1 << 1;
pub const LIB_ID_FREE_NO_USER_REFCOUNT: i32 = 1 << 2;
pub const LIB_ID_FREE_NOT_ALLOCATED: i32 = 1 << 3;
pub const LIB_ID_FREE_NO_DEG_TAG: i32 = 1 << 4;
pub const LIB_ID_FREE_NO_NAMEMAP_REMOVE: i32 = 1 << 5;

/// `ID_REMAP_*`.
pub const ID_REMAP_SKIP_USER_CLEAR: i32 = 1 << 0;
pub const ID_REMAP_STORE_NEVER_NULL_USAGE: i32 = 1 << 1;
pub const ID_REMAP_FORCE_NEVER_NULL_USAGE: i32 = 1 << 2;
pub const ID_REMAP_FORCE_INTERNAL_RUNTIME_POINTERS: i32 = 1 << 3;
pub const ID_REMAP_TYPE_CLEANUP: i32 = 1 << 4;

/// `ID_Type` codes used here.
pub const ID_SCE: i16 = 1;
pub const ID_GR: i16 = 2;
pub const ID_OB: i16 = 3;
pub const ID_LI: i16 = 4;

/// `ID_FAKE_USERS(id)` — `us` floor (extra user counts).
pub fn id_fake_users(us: i32) -> i32 {
    us & 0xffff
}

/// `BKEIDDeleteOptions`.
#[derive(Default, Clone, Copy)]
pub struct BkeIdDeleteOptions {
    pub extra_remapping_flags: i32,
    pub prevent_invariants_update: bool,
    pub prevent_liboverride_hierarchy_root_ensure: bool,
}

/// Injected Main/ID hooks (the `bmain`, `lib_remap`, `idtype`,
/// depsgraph, Python and UI surfaces this file drives).
pub struct IdDeleteHooks<'a> {
    /// `BKE_libblock_free_data` inner steps, per handle: idprops,
    /// override, asset, weak ref, animdata, runtime.
    pub free_data: &'a mut dyn FnMut(usize, bool),
    /// `BKE_libblock_free_data_py`.
    pub free_data_py: &'a mut dyn FnMut(usize),
    /// `BKE_libblock_free_datablock` (idtype `free_data`).
    pub free_datablock: &'a mut dyn FnMut(usize),
    /// `BKE_libblock_relink_ex`.
    pub relink_ex: &'a mut dyn FnMut(usize, i32),
    /// `BKE_key_from_id` → shape-key ID handle.
    pub key_from_id: &'a mut dyn FnMut(usize) -> Option<usize>,
    /// `BKE_libblock_unlink`.
    pub unlink: &'a mut dyn FnMut(usize),
    /// `DEG_id_type_tag`.
    pub deg_id_type_tag: &'a mut dyn FnMut(i16),
    /// `free_notifier_reference_cb`.
    pub free_notifier_reference: Option<&'a mut dyn FnMut(usize)>,
    /// `remap_editor_id_reference_cb(remapper)`.
    pub remap_editor_id_reference: Option<&'a mut dyn FnMut(&[usize])>,
    /// Main listbase ops: `remlink(id)` + `namemap_remove(id)` +
    /// `owner_bmain_set(id, None)` fused.
    pub main_remove_id: &'a mut dyn FnMut(usize),
    /// `BKE_main_lock`/`unlock`.
    pub main_lock: &'a mut dyn FnMut(),
    pub main_unlock: &'a mut dyn FnMut(),
    /// `BKE_layer_collection_resync_forbid`/`allow`.
    pub resync_forbid: &'a mut dyn FnMut(),
    pub resync_allow: &'a mut dyn FnMut(),
    /// `BKE_main_collection_sync_remap`.
    pub collection_sync_remap: &'a mut dyn FnMut(),
    /// `BKE_library_main_rebuild_hierarchy`.
    pub library_rebuild_hierarchy: &'a mut dyn FnMut(),
    /// `BKE_main_ensure_invariants`.
    pub ensure_invariants: &'a mut dyn FnMut(),
    /// `BKE_lib_override_library_main_hierarchy_root_ensure`.
    pub liboverride_hierarchy_root_ensure: &'a mut dyn FnMut(),
    /// `bke::library::main_cleanup_parent_archives`.
    pub cleanup_parent_archives: &'a mut dyn FnMut(),
    /// `MEM_delete(id)` — final dealloc.
    pub delete_id: &'a mut dyn FnMut(usize),
}

/// `ID` handle view the frees need.
#[derive(Clone, Copy)]
pub struct IdInfo {
    pub handle: usize,
    pub tag: i32,
    pub us: i32,
    /// `id->id_type()`.
    pub id_type: i16,
    /// `ID_IS_LINKED`.
    pub is_linked: bool,
    /// `ID_IS_OVERRIDE_LIBRARY`.
    pub is_override_library: bool,
}

/// `BKE_libblock_free_data`.
pub fn bke_libblock_free_data(id: usize, do_id_user: bool, hooks: &mut IdDeleteHooks) {
    (hooks.free_data)(id, do_id_user);
    (hooks.free_datablock)(id);
}

/// `BKE_libblock_free_runtime_data` — folded into `free_data` in the
/// Rust model (runtime dealloc is part of it).

/// `id_free` — core free path; returns the effective flag.
fn id_free(
    bmain_present: bool,
    id: &IdInfo,
    mut flag: i32,
    use_flag_from_idtag: bool,
    hooks: &mut IdDeleteHooks,
) -> i32 {
    if use_flag_from_idtag {
        if (id.tag & ID_TAG_NO_MAIN) != 0 {
            flag |= LIB_ID_FREE_NO_MAIN | LIB_ID_FREE_NO_UI_USER | LIB_ID_FREE_NO_DEG_TAG;
        } else {
            flag &= !LIB_ID_FREE_NO_MAIN;
        }

        if (id.tag & ID_TAG_NO_USER_REFCOUNT) != 0 {
            flag |= LIB_ID_FREE_NO_USER_REFCOUNT;
        } else {
            flag &= !LIB_ID_FREE_NO_USER_REFCOUNT;
        }

        if (id.tag & ID_TAG_NOT_ALLOCATED) != 0 {
            flag |= LIB_ID_FREE_NOT_ALLOCATED;
        } else {
            flag &= !LIB_ID_FREE_NOT_ALLOCATED;
        }
    }

    debug_assert!((flag & LIB_ID_FREE_NO_MAIN) != 0 || bmain_present);
    debug_assert!((flag & LIB_ID_FREE_NO_MAIN) != 0 || (flag & LIB_ID_FREE_NOT_ALLOCATED) == 0);
    debug_assert!(
        (flag & LIB_ID_FREE_NO_MAIN) != 0 || (flag & LIB_ID_FREE_NO_USER_REFCOUNT) == 0
    );

    let type_ = id.id_type;

    if bmain_present && (flag & LIB_ID_FREE_NO_DEG_TAG) == 0 {
        (hooks.deg_id_type_tag)(type_);
    }

    (hooks.free_data_py)(id.handle);

    let key = if (flag & LIB_ID_FREE_NO_MAIN) == 0 {
        (hooks.key_from_id)(id.handle)
    } else {
        None
    };

    if (flag & LIB_ID_FREE_NO_USER_REFCOUNT) == 0 {
        (hooks.relink_ex)(id.handle, ID_REMAP_SKIP_USER_CLEAR);
    }

    if (flag & LIB_ID_FREE_NO_MAIN) == 0 {
        if let Some(key_id) = key {
            let key_info = IdInfo {
                handle: key_id,
                tag: 0,
                us: 0,
                id_type: 0,
                is_linked: false,
                is_override_library: false,
            };
            id_free(bmain_present, &key_info, flag, use_flag_from_idtag, hooks);
        }
    }

    (hooks.free_datablock)(id.handle);

    /* avoid notifying on removed data */
    if (flag & LIB_ID_FREE_NO_MAIN) == 0 {
        (hooks.main_lock)();
    }

    if (flag & LIB_ID_FREE_NO_UI_USER) == 0 {
        if let Some(cb) = hooks.free_notifier_reference.as_deref_mut() {
            cb(id.handle);
        }
        if let Some(cb) = hooks.remap_editor_id_reference.as_deref_mut() {
            cb(&[id.handle]);
        }
    }

    if (flag & LIB_ID_FREE_NO_MAIN) == 0 {
        (hooks.main_remove_id)(id.handle);
    }

    (hooks.free_data)(id.handle, (flag & LIB_ID_FREE_NO_USER_REFCOUNT) == 0);

    if (flag & LIB_ID_FREE_NO_MAIN) == 0 {
        (hooks.main_unlock)();
    }

    if (flag & LIB_ID_FREE_NOT_ALLOCATED) == 0 {
        (hooks.delete_id)(id.handle);
    }

    flag
}

/// `BKE_id_free_ex`.
pub fn bke_id_free_ex(
    bmain_present: bool,
    id: &IdInfo,
    flag_orig: i32,
    use_flag_from_idtag: bool,
    hooks: &mut IdDeleteHooks,
) {
    /* ViewLayer resync needs to be delayed during Scene freeing. */
    if bmain_present && (flag_orig & LIB_ID_FREE_NO_MAIN) == 0 {
        (hooks.resync_forbid)();
    }

    let id_type = id.id_type;
    let flag_final = id_free(bmain_present, id, flag_orig, use_flag_from_idtag, hooks);

    if bmain_present {
        if (flag_orig & LIB_ID_FREE_NO_MAIN) == 0 {
            (hooks.resync_allow)();
        }
        if (flag_final & LIB_ID_FREE_NO_MAIN) == 0 {
            if [ID_SCE, ID_GR, ID_OB].contains(&id_type) {
                (hooks.collection_sync_remap)();
            }
        }
    }
}

/// `BKE_id_free`.
pub fn bke_id_free(bmain_present: bool, id: &IdInfo, hooks: &mut IdDeleteHooks) {
    bke_id_free_ex(bmain_present, id, 0, true, hooks);
}

/// `BKE_id_free_us` — "test users" path (`id_us_min` + object
/// `user_one` clearing folded into `us`/`unlink` hooks).
pub fn bke_id_free_us(id: &mut IdInfo, hooks: &mut IdDeleteHooks) {
    /* `id_us_min`. */
    id.us -= 1;

    /* XXX This is a temp (2.77) hack so that we keep same behavior
     * as in 2.76 regarding collections when deleting an object. */
    if id.id_type == ID_OB && id.us == 1 && !id.is_linked {
        /* `id_us_clear_real`. */
        id.us = id_fake_users(id.us);
    }

    if id.us == 0 {
        let is_lib = id.id_type == ID_LI;

        (hooks.unlink)(id.handle);
        bke_id_free(true, id, hooks);

        if is_lib {
            (hooks.library_rebuild_hierarchy)();
        }
    }
}

/// `id_delete` — batch tagged deletion.
///
/// `all_ids` = `BKE_main_lists_get` flattened (forward order);
/// `remap_multiple_locked` → `never_null_users` result set;
/// `relink_multiple` does the cleanup relink pass.
#[allow(clippy::too_many_arguments)]
pub fn id_delete(
    ids_to_delete: &mut HashSet<usize>,
    all_ids: &[IdInfo],
    options: &BkeIdDeleteOptions,
    key_from_id: &dyn Fn(usize) -> Option<usize>,
    lib_of: &dyn Fn(usize) -> Option<usize>,
    remap_multiple_locked: &mut dyn FnMut(&[usize], i32) -> Vec<usize>,
    relink_multiple: &mut dyn FnMut(&[usize], i32, i32),
    id_info_of: &dyn Fn(usize) -> IdInfo,
    set_no_main_and_fake_us: &mut dyn FnMut(usize, i32),
    hooks: &mut IdDeleteHooks,
) -> usize {
    let remapping_flags = ID_REMAP_STORE_NEVER_NULL_USAGE
        | ID_REMAP_FORCE_NEVER_NULL_USAGE
        | ID_REMAP_FORCE_INTERNAL_RUNTIME_POINTERS
        | options.extra_remapping_flags;
    let mut has_deleted_library = false;
    let mut has_deleted_linked_or_liboverride_id = false;

    /* Used by batch tagged deletion: the id is already out of Main
     * with usages unlinked; UI users always cleared by remap. */
    let free_flag =
        LIB_ID_FREE_NO_UI_USER | LIB_ID_FREE_NO_MAIN | LIB_ID_FREE_NO_USER_REFCOUNT;

    (hooks.main_lock)();
    (hooks.resync_forbid)();

    let mut process_id_to_delete =
        |ids_to_delete: &mut HashSet<usize>,
         remap_set: &mut Vec<usize>,
         id_iter: usize,
         has_linked: &mut bool,
         info: &IdInfo,
         hooks: &mut IdDeleteHooks| {
            /* Do not tag as no_main now, we want to unlink it first. */
            (hooks.main_remove_id)(id_iter);
            ids_to_delete.insert(id_iter);
            remap_set.push(id_iter);

            if !*has_linked && (info.is_linked || info.is_override_library) {
                *has_linked = true;
            }
        };

    let mut keep_looping = true;
    while keep_looping {
        keep_looping = false;
        let mut remap_set: Vec<usize> = Vec::new();

        /* First tag and remove from Main all datablocks directly
         * from target lib. Go forward: dependencies before users. */
        for info in all_ids {
            let id_iter = info.handle;
            let lib_of_iter = lib_of(id_iter);
            let lib_deleted = info.is_linked
                && lib_of_iter.is_some_and(|l| ids_to_delete.contains(&l));
            /* NOTE: in case we delete a library, we also delete all
             * its datablocks! */
            if ids_to_delete.contains(&id_iter) || lib_deleted {
                process_id_to_delete(
                    ids_to_delete,
                    &mut remap_set,
                    id_iter,
                    &mut has_deleted_linked_or_liboverride_id,
                    info,
                    hooks,
                );

                /* Forcefully also delete shapekeys of the deleted ID
                 * if any, 'orphaned' shapekeys are not allowed. */
                if let Some(shape_key) = key_from_id(id_iter) {
                    if !ids_to_delete.contains(&shape_key) {
                        let key_info = id_info_of(shape_key);
                        process_id_to_delete(
                            ids_to_delete,
                            &mut remap_set,
                            shape_key,
                            &mut has_deleted_linked_or_liboverride_id,
                            &key_info,
                            hooks,
                        );
                    }
                }
                keep_looping = true;
            }
        }

        /* Will tag 'never nullptr' users of this ID too. */
        let never_null = remap_multiple_locked(&remap_set, remapping_flags);
        for id_never_null_iter in never_null {
            if ids_to_delete.insert(id_never_null_iter) {
                keep_looping = true;
            }
        }
    }

    /* Clean up archived_libraries nullptrs. */
    (hooks.cleanup_parent_archives)();

    /* Since we removed IDs from Main, their own other IDs usages
     * need to be removed 'manually'. */
    let cleanup_ids: Vec<usize> = ids_to_delete.iter().copied().collect();
    relink_multiple(
        &cleanup_ids,
        ID_REMAP_TYPE_CLEANUP,
        ID_REMAP_FORCE_INTERNAL_RUNTIME_POINTERS | ID_REMAP_SKIP_USER_CLEAR,
    );

    /* Now we can safely mark that ID as not being in Main database
     * anymore. */
    for &id in ids_to_delete.iter() {
        /* `id->tag |= ID_TAG_NO_MAIN; id->us = ID_FAKE_USERS(id)`. */
        set_no_main_and_fake_us(id, id_fake_users(id_info_of(id).us));

        let info = id_info_of(id);
        if !has_deleted_library && info.id_type == ID_LI {
            has_deleted_library = true;
        }
        id_free(true, &info, free_flag, false, hooks);
    }

    (hooks.main_unlock)();
    (hooks.resync_allow)();
    (hooks.collection_sync_remap)();
    if !options.prevent_invariants_update {
        (hooks.ensure_invariants)();
    }

    if has_deleted_library {
        (hooks.library_rebuild_hierarchy)();
    }

    /* Deleting a liboverride or linked ID may affect liboverride
     * hierarchy roots. */
    if has_deleted_linked_or_liboverride_id && !options.prevent_liboverride_hierarchy_root_ensure {
        (hooks.liboverride_hierarchy_root_ensure)();
    }

    ids_to_delete.len()
}

/// `BKE_id_delete`.
pub fn bke_id_delete(
    id: &IdInfo,
    all_ids: &[IdInfo],
    options: &BkeIdDeleteOptions,
    key_from_id: &dyn Fn(usize) -> Option<usize>,
    lib_of: &dyn Fn(usize) -> Option<usize>,
    remap_multiple_locked: &mut dyn FnMut(&[usize], i32) -> Vec<usize>,
    relink_multiple: &mut dyn FnMut(&[usize], i32, i32),
    id_info_of: &dyn Fn(usize) -> IdInfo,
    set_no_main_and_fake_us: &mut dyn FnMut(usize, i32),
    hooks: &mut IdDeleteHooks,
) {
    debug_assert_eq!((id.tag & ID_TAG_NO_MAIN), 0, "Cannot be used with IDs outside of Main");
    let mut set = HashSet::from([id.handle]);
    id_delete(
        &mut set,
        all_ids,
        options,
        key_from_id,
        lib_of,
        remap_multiple_locked,
        relink_multiple,
        id_info_of,
        set_no_main_and_fake_us,
        hooks,
    );
}

/// `BKE_id_multi_tagged_delete` — `ID_TAG_DOIT` selection.
#[allow(clippy::too_many_arguments)]
pub fn bke_id_multi_tagged_delete(
    all_ids: &[IdInfo],
    options: &BkeIdDeleteOptions,
    key_from_id: &dyn Fn(usize) -> Option<usize>,
    lib_of: &dyn Fn(usize) -> Option<usize>,
    remap_multiple_locked: &mut dyn FnMut(&[usize], i32) -> Vec<usize>,
    relink_multiple: &mut dyn FnMut(&[usize], i32, i32),
    id_info_of: &dyn Fn(usize) -> IdInfo,
    set_no_main_and_fake_us: &mut dyn FnMut(usize, i32),
    hooks: &mut IdDeleteHooks,
) -> usize {
    let mut ids_to_delete: HashSet<usize> = all_ids
        .iter()
        .filter(|info| (info.tag & ID_TAG_DOIT) != 0)
        .map(|info| info.handle)
        .collect();
    id_delete(
        &mut ids_to_delete,
        all_ids,
        options,
        key_from_id,
        lib_of,
        remap_multiple_locked,
        relink_multiple,
        id_info_of,
        set_no_main_and_fake_us,
        hooks,
    )
}
