//! Port of `intern/wm_files_link.cc` — `WM_OT_link`/
//! `WM_OT_append`/`WM_OT_id_linked_relocate`/
//! `WM_OT_lib_relocate`/`WM_OT_lib_reload` operators:
//! flag assembly, item poll, property registration.

/// `FILE_*`/`BLO_LIBLINK_*` verbatim (`BLO_readfile.hh`).
pub const FILE_AUTOSELECT: i32 = 1 << 0;
pub const FILE_ACTIVE_COLLECTION: i32 = 1 << 2;
pub const FILE_RELPATH: i32 = 1 << 4;
pub const FILE_LINK: i32 = 1 << 5;
pub const BLO_LIBLINK_APPEND_RECURSIVE: i32 = 1 << 7;
pub const BLO_LIBLINK_APPEND_SET_FAKEUSER: i32 = 1 << 8;
pub const BLO_LIBLINK_APPEND_LOCAL_ID_REUSE: i32 = 1 << 9;
pub const BLO_LIBLINK_APPEND_ASSET_DATA_CLEAR: i32 = 1 << 11;
pub const BLO_LIBLINK_COLLECTION_INSTANCE: i32 = 1 << 6;
pub const BLO_LIBLINK_OBDATA_INSTANCE: i32 = 1 << 10;
/// `FILE_TYPE_*` extras.
pub const FILE_TYPE_FOLDER: i32 = 4;
pub const FILE_TYPE_BLENDERLIB: i32 = 8;
pub const FILE_LOADLIB: i32 = 1 << 1;
pub const FILE_OPENFILE: i32 = 1;
/// `WM_FILESEL_*` verbatim.
pub const WM_FILESEL_FILEPATH: i32 = 1 << 0;
pub const WM_FILESEL_DIRECTORY: i32 = 1 << 1;
pub const WM_FILESEL_FILENAME: i32 = 1 << 2;
pub const WM_FILESEL_RELPATH: i32 = 1 << 3;
pub const WM_FILESEL_FILES: i32 = 1 << 4;
pub const WM_FILESEL_SHOW_PROPS: i32 = 1 << 5;
pub const FILE_DEFAULTDISPLAY: i32 = 0;
pub const FILE_SORT_DEFAULT: i32 = 0;

/// Operator boolean properties consulted (verbatim
/// names).
pub struct LinkAppendProps {
    /// `autoselect`/`active_collection`/`relative_path`/
    /// `link`/`use_recursive`/`set_fake`/
    /// `do_reuse_local_id`/`clear_asset_data`/
    /// `instance_collections`/`instance_object_data`.
    pub autoselect: bool,
    pub active_collection: bool,
    pub relative_path: Option<bool>,
    pub link: bool,
    pub use_recursive: bool,
    pub set_fake: bool,
    pub do_reuse_local_id: bool,
    pub clear_asset_data: bool,
    pub instance_collections: bool,
    pub instance_object_data: bool,
}

/// `wm_link_append_flag` (verbatim — bit assembly).
pub fn wm_link_append_flag(p: &LinkAppendProps) -> i32 {
    let mut flag = 0;
    if p.autoselect {
        flag |= FILE_AUTOSELECT;
    }
    if p.active_collection {
        flag |= FILE_ACTIVE_COLLECTION;
    }
    if p.relative_path == Some(true) {
        flag |= FILE_RELPATH;
    }
    if p.link {
        flag |= FILE_LINK;
    } else {
        if p.use_recursive {
            flag |= BLO_LIBLINK_APPEND_RECURSIVE;
        }
        if p.set_fake {
            flag |= BLO_LIBLINK_APPEND_SET_FAKEUSER;
        }
        if p.do_reuse_local_id {
            flag |= BLO_LIBLINK_APPEND_LOCAL_ID_REUSE;
        }
        if p.clear_asset_data {
            flag |= BLO_LIBLINK_APPEND_ASSET_DATA_CLEAR;
        }
    }
    if p.instance_collections {
        flag |= BLO_LIBLINK_COLLECTION_INSTANCE;
    }
    if p.instance_object_data {
        flag |= BLO_LIBLINK_OBDATA_INSTANCE;
    }
    flag
}

/// `wm_link_append_item_poll` (verbatim — group/name
/// required + linkable check + report).
pub fn wm_link_append_item_poll(
    filepath: &str,
    group: Option<&str>,
    name: Option<&str>,
    do_append: bool,
    idtype_is_linkable: &dyn Fn(i32) -> bool,
    idtype_is_only_appendable: &dyn Fn(i32) -> bool,
    idcode_from_name: &dyn Fn(&str) -> i32,
    report: &mut dyn FnMut(&str),
    clog_warn: &mut dyn FnMut(&str),
) -> bool {
    if group.is_none() || name.is_none() {
        (clog_warn)(&format!("Skipping {}", filepath));
        return false;
    }
    let idcode = (idcode_from_name)(group.unwrap());
    if !(idtype_is_linkable)(idcode)
        || (!do_append && (idtype_is_only_appendable)(idcode))
    {
        if do_append {
            (report)(&format!(
                "Cannot append data-block '{}' of type '{}'",
                name.unwrap(),
                group.unwrap()
            ));
        } else {
            (report)(&format!(
                "Cannot link data-block '{}' of type '{}'",
                name.unwrap(),
                group.unwrap()
            ));
        }
        return false;
    }
    true
}

/// `WM_OT_link`/`WM_OT_append` registration (verbatim
/// strings/flags).
pub fn wm_ot_link() -> (&'static str, &'static str, &'static str, i32) {
    (
        "WM_OT_link",
        "Link",
        "Link from a Library .blend file",
        crate::wm_files_colorspace::OPTYPE_UNDO,
    )
}
pub fn wm_ot_append() -> (&'static str, &'static str, &'static str, i32) {
    (
        "WM_OT_append",
        "Append",
        "Append from a Library .blend file",
        crate::wm_files_colorspace::OPTYPE_UNDO,
    )
}

/// `WM_OT_id_linked_relocate`/`WM_OT_lib_relocate`/
/// `WM_OT_lib_reload` registration (verbatim).
pub fn wm_ot_id_linked_relocate() -> (&'static str, &'static str, &'static str, i32) {
    (
        "WM_OT_id_linked_relocate",
        "Relocate Library Item",
        "Relocate the selected library data-block to a new file",
        crate::wm_files_colorspace::OPTYPE_UNDO,
    )
}
pub fn wm_ot_lib_relocate() -> (&'static str, &'static str, &'static str, i32) {
    (
        "WM_OT_lib_relocate",
        "Relocate Library",
        "Relocate the current library to a new blend-file",
        crate::wm_files_colorspace::OPTYPE_UNDO,
    )
}
pub fn wm_ot_lib_reload() -> (&'static str, &'static str, &'static str, i32) {
    (
        "WM_OT_lib_reload",
        "Reload Library",
        "Reload the current library from its blend-file",
        crate::wm_files_colorspace::OPTYPE_UNDO,
    )
}
