//! Port of `intern/lib_query.cc` — the foreach-ID-link walk driver,
//! usage queries (`ID_use_ID`, `is_locally/indirectly_used`,
//! `test_usages`) and id-type usage filters. `Main` list iteration
//! and each IDType's `foreach_id` are injected.

use std::collections::HashSet;

/// `LibraryForeachIDCallbackFlag`.
pub type LibraryForeachIDCallbackFlag = i32;
/// `LibraryForeachIDFlag`.
pub type LibraryForeachIDFlag = i32;

/// `IDWALK_CB_INTERNAL`.
pub const IDWALK_CB_INTERNAL: i32 = 1 << 0;
/// `IDWALK_CB_LOOPBACK`.
pub const IDWALK_CB_LOOPBACK: i32 = 1 << 1;
/// `IDWALK_CB_OVERRIDE_LIBRARY_REFERENCE`.
pub const IDWALK_CB_OVERRIDE_LIBRARY_REFERENCE: i32 = 1 << 2;
/// `IDWALK_CB_OVERRIDE_LIBRARY_NOT_OVERRIDABLE`.
pub const IDWALK_CB_OVERRIDE_LIBRARY_NOT_OVERRIDABLE: i32 = 1 << 3;
/// `IDWALK_CB_USER`.
pub const IDWALK_CB_USER: i32 = 1 << 4;
/// `IDWALK_CB_USER_ONE`.
pub const IDWALK_CB_USER_ONE: i32 = 1 << 5;
/// `IDWALK_CB_INDIRECT_USAGE`.
pub const IDWALK_CB_INDIRECT_USAGE: i32 = 1 << 6;
/// `IDWALK_CB_DEPRECATION_WARNING`.
pub const IDWALK_CB_DEPRECATION_WARNING: i32 = 1 << 7;
/// `IDWALK_CB_EMBEDDED`.
pub const IDWALK_CB_EMBEDDED: i32 = 1 << 8;
/// `IDWALK_CB_WEAK_LINK`.
pub const IDWALK_CB_WEAK_LINK: i32 = 1 << 9;

/// `IDWALK_READONLY`.
pub const IDWALK_READONLY: i32 = 1 << 0;
/// `IDWALK_RECURSE`.
pub const IDWALK_RECURSE: i32 = 1 << 1;
/// `IDWALK_DO_INTERNAL_RUNTIME_POINTERS`.
pub const IDWALK_DO_INTERNAL_RUNTIME_POINTERS: i32 = 1 << 2;
/// `IDWALK_DO_LIBRARY_POINTER`.
pub const IDWALK_DO_LIBRARY_POINTER: i32 = 1 << 3;
/// `IDWALK_INCLUDE_UI`.
pub const IDWALK_INCLUDE_UI: i32 = 1 << 4;
/// `IDWALK_FOREACHID_LINK_NOT_MAIN`.
pub const IDWALK_FOREACHID_LINK_NOT_MAIN: i32 = 1 << 5;

/// `IDWALK_STOP` (status bit).
pub const IDWALK_STOP: i32 = 1 << 0;
/// `IDWALK_RET_NOP`.
pub const IDWALK_RET_NOP: i32 = 0;
/// `IDWALK_RET_STOP_ITER`.
pub const IDWALK_RET_STOP_ITER: i32 = 1 << 0;
/// `IDWALK_RET_STOP_RECURSION`.
pub const IDWALK_RET_STOP_RECURSION: i32 = 1 << 1;
/// `IDWALK_RET_CLEAR_PTR`.
pub const IDWALK_RET_CLEAR_PTR: i32 = 1 << 2;

/// `LibraryForeachIDData`.
pub struct LibraryForeachIDData {
    /// `bmain` — opaque handle.
    pub bmain: usize,
    /// `owner_id`.
    pub owner_id: usize,
    /// `self_id`.
    pub self_id: usize,
    /// `flag`.
    pub flag: LibraryForeachIDFlag,
    /// `cb_flag`.
    pub cb_flag: LibraryForeachIDCallbackFlag,
    /// `cb_flag_clear`.
    pub cb_flag_clear: LibraryForeachIDCallbackFlag,
    /// `status`.
    pub status: i32,
    /// `ids_handled` — recursion guard.
    pub ids_handled: Option<HashSet<usize>>,
    /// `ids_todo`.
    pub ids_todo: Vec<usize>,
}

/// `BKE_lib_query_foreachid_iter_stop`.
pub fn bke_lib_query_foreachid_iter_stop(data: &LibraryForeachIDData) -> bool {
    data.status & IDWALK_STOP != 0
}

/// `LibraryIDLinkCallbackData` (reduced).
pub struct LibraryIDLinkCallbackData {
    /// `bmain`.
    pub bmain: usize,
    /// `owner_id`.
    pub owner_id: usize,
    /// `self_id`.
    pub self_id: usize,
    /// `id_pointer` — handle of the pointed-at ID slot.
    pub id_pointer: usize,
    /// `cb_flag`.
    pub cb_flag: LibraryForeachIDCallbackFlag,
}

/// `BKE_lib_query_foreachid_process`.
pub fn bke_lib_query_foreachid_process(
    data: &mut LibraryForeachIDData,
    id_pp: usize,
    old_id: usize,
    cb_flag: LibraryForeachIDCallbackFlag,
    callback: &mut dyn FnMut(&LibraryIDLinkCallbackData) -> i32,
) {
    if bke_lib_query_foreachid_iter_stop(data) {
        return;
    }
    let flag = data.flag;
    let mut cb_flag = (cb_flag | data.cb_flag) & !data.cb_flag_clear;
    if cb_flag
        & (IDWALK_CB_INTERNAL
            | IDWALK_CB_LOOPBACK
            | IDWALK_CB_OVERRIDE_LIBRARY_REFERENCE)
        != 0
    {
        cb_flag |= IDWALK_CB_OVERRIDE_LIBRARY_NOT_OVERRIDABLE;
    }

    let callback_data = LibraryIDLinkCallbackData {
        bmain: data.bmain,
        owner_id: data.owner_id,
        self_id: data.self_id,
        id_pointer: id_pp,
        cb_flag,
    };
    let callback_return = callback(&callback_data);

    if old_id != 0 && flag & IDWALK_RECURSE != 0 {
        let first_time = data
            .ids_handled
            .as_mut()
            .map(|s| s.insert(old_id))
            .unwrap_or(true);
        if first_time && callback_return & IDWALK_RET_STOP_RECURSION == 0 {
            data.ids_todo.push(old_id);
        }
    }
    if callback_return & IDWALK_RET_STOP_ITER != 0 {
        data.status |= IDWALK_STOP;
    }
}

/// `BKE_lib_query_foreachid_process_callback_flag_override` —
/// returns the previous `cb_flag`.
pub fn bke_lib_query_foreachid_process_callback_flag_override(
    data: &mut LibraryForeachIDData,
    cb_flag: LibraryForeachIDCallbackFlag,
    do_replace: bool,
) -> LibraryForeachIDCallbackFlag {
    let backup = data.cb_flag;
    if do_replace {
        data.cb_flag = cb_flag;
    } else {
        data.cb_flag |= cb_flag;
    }
    backup
}

/// `BKE_library_update_ID_link_user` — adjust user counts after a
/// link change. `us_min`/`us_plus`/`us_ensure_real` injected.
pub fn bke_library_update_id_link_user(
    cb_flag: i32,
    us_min: &mut dyn FnMut(),
    us_plus: &mut dyn FnMut(),
    us_ensure_real: &mut dyn FnMut(),
) {
    if cb_flag & IDWALK_CB_USER != 0 {
        us_min();
        us_plus();
    } else if cb_flag & IDWALK_CB_USER_ONE != 0 {
        us_ensure_real();
    }
}

/// `FILTER_ID_ALL`.
pub const FILTER_ID_ALL: u64 = u64::MAX;

/// `BKE_library_id_can_use_filter_id` — injected owner capability
/// checks (`has_properties`, `is_node_tree_owner`, `has_animdata`,
/// `is_override_real`, `dependencies_id_types`).
pub fn bke_library_id_can_use_filter_id(
    has_properties: bool,
    has_system_properties: bool,
    include_ui_and_is_screen: bool,
    is_node_tree_owner: bool,
    has_animdata: bool,
    is_override_library_real: bool,
    dependencies_id_types: Option<u64>,
) -> u64 {
    if has_properties
        || has_system_properties
        || include_ui_and_is_screen
        || is_node_tree_owner
        || has_animdata
        || is_override_library_real
    {
        return FILTER_ID_ALL;
    }
    dependencies_id_types.unwrap_or(0)
}

/// `BKE_library_id_can_use_idtype`.
pub fn bke_library_id_can_use_idtype(
    filter_id_type_used: u64,
    can_be_used: u64,
) -> bool {
    can_be_used & filter_id_type_used != 0
}

/// `IDUsersIter` accumulation from `foreach_libblock_id_users_callback`.
#[derive(Default)]
pub struct IDUsersIter {
    /// `count_direct`.
    pub count_direct: i32,
    /// `count_indirect`.
    pub count_indirect: i32,
}

/// `foreach_libblock_id_users_callback` — called for every
/// `(used_id, cb_flag)` link of `curr_id`.
pub fn foreach_libblock_id_users_callback(
    iter: &mut IDUsersIter,
    searched_id: usize,
    used_id: usize,
    cb_flag: LibraryForeachIDCallbackFlag,
) {
    if used_id == 0 || cb_flag & IDWALK_CB_LOOPBACK != 0 {
        return;
    }
    if used_id == searched_id {
        if cb_flag & IDWALK_CB_INDIRECT_USAGE != 0 {
            iter.count_indirect += 1;
        } else {
            iter.count_direct += 1;
        }
    }
}

/// `BKE_library_ID_use_ID` — `links(id)` yields `(used_id, cb_flag)`
/// pairs (the IDType's `foreach_id`, injected).
pub fn bke_library_id_use_id(
    id_user: usize,
    id_used: usize,
    links: &dyn Fn(usize) -> Vec<(usize, i32)>,
) -> i32 {
    let mut iter = IDUsersIter::default();
    for (used, flag) in links(id_user) {
        foreach_libblock_id_users_callback(&mut iter, id_used, used, flag);
    }
    iter.count_direct + iter.count_indirect
}

/// `library_ID_is_used` — `candidates` yields `(id, can_use_idtype)`
/// pairs for every ID that may reference `idv`.
pub fn library_id_is_used(
    id: usize,
    check_linked: bool,
    candidates: &dyn Fn() -> Vec<(usize, bool)>,
    links: &dyn Fn(usize) -> Vec<(usize, i32)>,
) -> bool {
    let mut iter = IDUsersIter::default();
    for (id_curr, can_use) in candidates() {
        if !can_use || id_curr == id {
            continue;
        }
        for (used, flag) in links(id_curr) {
            foreach_libblock_id_users_callback(&mut iter, id, used, flag);
        }
        let defined = if check_linked {
            iter.count_indirect != 0
        } else {
            iter.count_direct != 0
        };
        if defined {
            return true;
        }
    }
    false
}

/// `BKE_library_ID_test_usages` — both direct and indirect needed
/// to stop early.
pub fn bke_library_id_test_usages(
    id: usize,
    candidates: &dyn Fn() -> Vec<(usize, bool)>,
    links: &dyn Fn(usize) -> Vec<(usize, i32)>,
) -> (bool, bool) {
    let mut iter = IDUsersIter::default();
    for (id_curr, can_use) in candidates() {
        if !can_use || id_curr == id {
            continue;
        }
        for (used, flag) in links(id_curr) {
            foreach_libblock_id_users_callback(&mut iter, id, used, flag);
        }
        if iter.count_direct != 0 && iter.count_indirect != 0 {
            break;
        }
    }
    (iter.count_direct != 0, iter.count_indirect != 0)
}
