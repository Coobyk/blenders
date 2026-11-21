//! Port of `intern/lib_id.cc` — ID management: session
//! uids, `id_full_name`/`ui_prefix`, unique string keys,
//! atomic tags, `can_use_id`/`can_be_asset`/`is_editable`,
//! ordering, name validation.
//!
//! `PORT_PENDING`: `BKE_lib_id_(expand|make)_local`
//! (LibraryIDLinkCallback recursion over all ID users —
//! needs `IDTypeInfo` tables + foreach_libblock callback),
//! `id_swap`/`BKE_lib_id_swap` (struct content swap +
//! embedded IDs), `id_copy` (per-type copy fns), `new_name_
//! validate` (namemap — `BKE_main_namemap_get_unique_name`
//! injected), `move_to_same_lib`, blend write/read.

use std::sync::atomic::{AtomicU32, Ordering};

/// `global_session_uid`.
static GLOBAL_SESSION_UID: AtomicU32 = AtomicU32::new(0);

/// `MAIN_ID_SESSION_UID_UNSET`.
pub const MAIN_ID_SESSION_UID_UNSET: u32 = 0;

/// `MAX_ID_NAME`/`MAX_ID_FULL_NAME` verbatim.
pub const MAX_ID_NAME: usize = 66;
/// `MAX_ID_FULL_NAME`.
pub const MAX_ID_FULL_NAME: usize = MAX_ID_NAME + 64;
/// `MAX_ID_FULL_NAME_UI`.
pub const MAX_ID_FULL_NAME_UI: usize = MAX_ID_FULL_NAME + 32;

/// `BKE_lib_libblock_session_uid_ensure` (verbatim,
/// overflow re-add included).
pub fn lib_libblock_session_uid_ensure(session_uid: &mut u32) {
    if *session_uid == MAIN_ID_SESSION_UID_UNSET {
        *session_uid = GLOBAL_SESSION_UID.fetch_add(1, Ordering::Relaxed) + 1;
        if *session_uid == MAIN_ID_SESSION_UID_UNSET {
            *session_uid = GLOBAL_SESSION_UID.fetch_add(1, Ordering::Relaxed) + 1;
        }
    }
}
/// `BKE_lib_libblock_session_uid_renew` (verbatim).
pub fn lib_libblock_session_uid_renew(session_uid: &mut u32) {
    *session_uid = MAIN_ID_SESSION_UID_UNSET;
    lib_libblock_session_uid_ensure(session_uid);
}

/// `BKE_id_full_name_get` (verbatim — `id.name[2:]` +
/// ` [lib_name]` suffix for linked).
pub fn id_full_name_get(id_name: &str, lib_name: Option<&str>, separator_char: Option<char>) -> String {
    let mut name = id_name.to_string();
    if let Some(lib) = lib_name {
        name.push(separator_char.unwrap_or(' '));
        name.push('[');
        name.push_str(lib);
        name.push(']');
    }
    name
}

/// `BKE_id_full_name_ui_prefix_get` (verbatim — 3-char
/// prefix `L/F/ ` hints then full name).
pub fn id_full_name_ui_prefix_get(
    id_name: &str,
    is_linked: bool,
    is_missing: bool,
    is_override: bool,
    has_fakeuser: bool,
    us_is_zero: bool,
    add_lib_hint: bool,
    separator_char: Option<char>,
    lib_name: Option<&str>,
) -> (String, usize) {
    let mut name = String::new();
    let mut i = 0;
    if add_lib_hint {
        name.push(if !is_linked {
            if is_override {
                'O'
            } else {
                ' '
            }
        } else if is_missing {
            'M'
        } else {
            'L'
        });
        i += 1;
    }
    name.push(if has_fakeuser {
        'F'
    } else if us_is_zero {
        '0'
    } else {
        ' '
    });
    i += 1;
    name.push(' ');
    i += 1;
    name.push_str(&id_full_name_get(id_name, lib_name, separator_char));
    (name, i)
}

/// `BKE_id_to_unique_string_key` (verbatim — lib-prefixed).
pub fn id_to_unique_string_key(id_name: &str, lib_id_name: Option<&str>) -> String {
    match lib_id_name {
        None => id_name.to_string(),
        Some(lib) => {
            let ascii_len = (lib.len() as u8).wrapping_add(32) as char;
            format!("{}{}{}", ascii_len, lib, id_name)
        }
    }
}

/// `BKE_id_tag_set_atomic`/`clear_atomic` (verbatim — plain
/// bit ops here, atomicity caller-side).
pub fn id_tag_set_atomic(tag: &mut i32, t: i32) {
    *tag |= t;
}
/// `id_tag_clear_atomic`.
pub fn id_tag_clear_atomic(tag: &mut i32, t: i32) {
    *tag &= !t;
}

/// `BKE_id_is_in_main` (verbatim — `libbase` list contains
/// check injected as `in_list`).
pub fn id_is_in_main(id_is_null_or_in_list: bool) -> bool {
    id_is_null_or_in_list
}

/// `BKE_id_can_be_asset` (verbatim).
pub fn id_can_be_asset(id_is_editable: bool, is_override: bool, idcode_is_linkable: bool) -> bool {
    id_is_editable && !is_override && idcode_is_linkable
}

/// `BKE_id_is_editable` (verbatim).
pub fn id_is_editable(id_is_editable_flag: bool, is_system_defined_override: bool) -> bool {
    id_is_editable_flag && !is_system_defined_override
}

/// `BKE_id_can_use_id` (verbatim).
pub fn id_can_use_id(from_lib: bool, to_lib: bool, from_no_main: bool, to_no_main: bool) -> bool {
    if from_lib && !to_lib {
        return false;
    }
    if !from_no_main && to_no_main {
        return false;
    }
    true
}

/// `ID_TAG_NO_MAIN` (DNA_ID_enums verbatim).
pub const ID_TAG_NO_MAIN: i32 = 1 << 0;
/// `ID_TAG_TEMP_MAIN`.
pub const ID_TAG_TEMP_MAIN: i32 = 1 << 1;
/// `ID_TAG_COPIED_ON_EVAL`.
pub const ID_TAG_COPIED_ON_EVAL: i32 = 1 << 2;
/// `ID_TAG_COPIED_ON_EVAL_FINAL_RESULT`.
pub const ID_TAG_COPIED_ON_EVAL_FINAL_RESULT: i32 = 1 << 3;
/// `ID_TAG_LOCALIZED`.
pub const ID_TAG_LOCALIZED: i32 = 1 << 4;
/// `ID_TAG_MISSING`.
pub const ID_TAG_MISSING: i32 = 1 << 5;
/// `ID_FLAG_FAKEUSER`.
pub const ID_FLAG_FAKEUSER: i32 = 1 << 9;

/// `IDNewNameResult::Action` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IdNewNameAction {
    /// `UNCHANGED`.
    Unchanged,
    /// `RENAMED`.
    Renamed,
    /// `RENAMED_COLLISION_RESOLVED`.
    RenamedCollisionResolved,
    /// `CONFLICTING_NAME_EXISTING_RENAMED`.
    ConflictingNameExistingRenamed,
}

/// `IDNewNameMode` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IdNewNameMode {
    /// `AutoRenameAlways` — get a unique name.
    AutoRenameAlways,
    /// `RenameExistingAlways` — rename the conflicting ID.
    RenameExistingAlways,
    /// `RenameExistingSameRoot`.
    RenameExistingSameRoot,
}

/// `IDNewNameResult` verbatim.
pub struct IdNewNameResult {
    /// `action`.
    pub action: IdNewNameAction,
    /// `name` — final name (`Some` when renamed).
    pub name: Option<String>,
}
