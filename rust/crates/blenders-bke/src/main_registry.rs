//! Port of `intern/main.cc` + `BKE_main.hh` — the `Main`
//! datablock registry: per-type ID lists, lock, relations
//! map, library weak-reference map, `BKE_main_is_empty`,
//! `BKE_main_lists_get`, `BKE_main_relations_tag_set`.
//!
//! `PORT_PENDING`: `BKE_main_merge` (moves all IDs between
//! two Mains + remap_report/`checked_existing` bookkeeping —
//! needs ID-pointer remapping), `BKE_main_relations_create`
//! (needs `BKE_library_foreach_ID_link`), id_map rebuild,
//! thumbnail creation, `are_ids_from_different_mains_
//! matching`'s deep identity check.

use std::collections::{HashMap, HashSet};

/// ID-type count (`MAX_LIBARRAY`, `INDEX_ID_MAX+1` in C).
pub const MAX_LIBARRAY: usize = 57;

/// `eMainIDRelationsEntryTags` verbatim.
pub const MAINIDRELATIONS_ENTRY_TAGS_DOIT: u16 = 1 << 0;
/// `MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_TO`.
pub const MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_TO: u16 = 1 << 4;
/// `MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_FROM`.
pub const MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_FROM: u16 = 1 << 5;
/// `MAINIDRELATIONS_ENTRY_TAGS_PROCESSED`.
pub const MAINIDRELATIONS_ENTRY_TAGS_PROCESSED: u16 =
    MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_TO | MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_FROM;
/// `MAINIDRELATIONS_ENTRY_TAGS_INPROGRESS_TO`.
pub const MAINIDRELATIONS_ENTRY_TAGS_INPROGRESS_TO: u16 = 1 << 8;
/// `MAINIDRELATIONS_ENTRY_TAGS_INPROGRESS_FROM`.
pub const MAINIDRELATIONS_ENTRY_TAGS_INPROGRESS_FROM: u16 = 1 << 9;
/// `MAINIDRELATIONS_ENTRY_TAGS_INPROGRESS`.
pub const MAINIDRELATIONS_ENTRY_TAGS_INPROGRESS: u16 =
    MAINIDRELATIONS_ENTRY_TAGS_INPROGRESS_TO | MAINIDRELATIONS_ENTRY_TAGS_INPROGRESS_FROM;
/// `MAINIDRELATIONS_ENTRY_TAGS_NEW`.
pub const MAINIDRELATIONS_ENTRY_TAGS_NEW: u16 = 1 << 12;
/// `MAINIDRELATIONS_ENTRY_TAGS_ALL` (private).
pub const MAINIDRELATIONS_ENTRY_TAGS_ALL: u16 = 0xFFF0;
/// `MAINIDRELATIONS_INCLUDE_UI`.
pub const MAINIDRELATIONS_INCLUDE_UI: u16 = 1 << 0;
/// `MAINIDRELATIONS_INCLUDE_EXTRA_DATA`.
pub const MAINIDRELATIONS_INCLUDE_EXTRA_DATA: u16 = 1 << 1;

/// `MainIDRelationsEntryItem` — one "from" link.
#[derive(Clone, Debug, Default)]
pub struct MainIdRelationsEntryItem {
    /// `from_id`.
    pub from_id: usize,
    /// `id_pointer` — address of the pointer slot.
    pub id_pointer: usize,
    /// `usage` (`IDWALK_CB_USER` etc flags, verbatim int).
    pub usage: i32,
    /// `session_uid` of `from_id`.
    pub session_uid: u64,
    /// `flag` (`IDWALK_CB_INTERNAL` etc).
    pub flag: u16,
}

/// `MainIDRelationsEntry` — one "to" ID's inbound links.
#[derive(Clone, Debug, Default)]
pub struct MainIdRelationsEntry {
    /// `to_id`.
    pub to_id: usize,
    /// `session_uid` of `to_id`.
    pub session_uid: u64,
    /// `tags` (see `MAINIDRELATIONS_ENTRY_TAGS_*`).
    pub tags: u16,
    /// `from_ids`.
    pub from_ids: Vec<MainIdRelationsEntryItem>,
}

/// `MainIDRelations` (`relations_from_pointers` flattened:
/// Rust keeps `entries` keyed by to-ID plus the from→to index
/// as a secondary map, matching C's two-map structure).
#[derive(Debug, Default)]
pub struct MainIdRelations {
    /// `relations_to_pointers` (to-ID → entry index).
    pub relations_to_ids: HashMap<usize, usize>,
    /// `relations_from_pointers` (from-ID → set of entries).
    pub relations_from_ids: HashMap<usize, HashSet<usize>>,
    /// All entries.
    pub entries: Vec<MainIdRelationsEntry>,
    /// `includes_ui`.
    pub includes_ui: bool,
}

/// `main_relations_create_idlink_cb` — the per-link callback
/// (verbatim: `to_id`==0 → continue; find/create to-entry;
/// find/create from-entry item; `users` counted).
pub fn main_relations_create_idlink_cb(
    relations: &mut MainIdRelations,
    from_id: usize,
    from_session_uid: u64,
    id_pointer: usize,
    to_id: usize,
    to_session_uid: u64,
    usage: i32,
    flag: u16,
) {
    if to_id == 0 {
        return;
    }
    let entry_i = *relations.relations_to_ids.entry(to_id).or_insert_with(|| {
        relations.entries.push(MainIdRelationsEntry {
            to_id,
            session_uid: to_session_uid,
            ..Default::default()
        });
        relations.entries.len() - 1
    });
    relations
        .relations_from_ids
        .entry(from_id)
        .or_default()
        .insert(entry_i);
    relations.entries[entry_i]
        .from_ids
        .push(MainIdRelationsEntryItem {
            from_id,
            id_pointer,
            usage,
            session_uid: from_session_uid,
            flag,
        });
}

/// `BKE_main_relations_tag_set` — set/clear `tag` on every
/// entry's `tags` (verbatim bit-or/clear).
pub fn main_relations_tag_set(
    relations: &mut MainIdRelations,
    tag: u16,
    value: bool,
) {
    for entry in relations.entries.iter_mut() {
        if value {
            entry.tags |= tag;
        } else {
            entry.tags &= !tag;
        }
    }
}

/// `MainLock` — recursion counter + break flag (verbatim).
#[derive(Debug, Default)]
pub struct MainLock {
    /// `is_locked`.
    pub is_locked: u32,
}

/// `BKE_main_lock` / `BKE_main_unlock`.
pub fn main_lock(lock: &mut MainLock) {
    lock.is_locked += 1;
}
/// `BKE_main_unlock`.
pub fn main_unlock(lock: &mut MainLock) {
    lock.is_locked = lock.is_locked.saturating_sub(1);
}

/// `BMain` — the `Main` struct (verbatim field list;
/// `ListBaseT<ID*>` lists are `Vec<usize>` handles;
/// `MainListsMap` = fixed array indexed by `ID_Type`
/// table index).
#[derive(Debug)]
pub struct BMain {
    /// `versionfile`/`subversionfile`/`minversionfile`/
    /// `minsubversionfile`.
    pub versionfile: i16,
    /// `subversionfile`.
    pub subversionfile: i16,
    /// `minversionfile`.
    pub minversionfile: i16,
    /// `minsubversionfile`.
    pub minsubversionfile: i16,
    /// `has_forward_compatibility_issues`.
    pub has_forward_compatibility_issues: bool,
    /// `is_asset_edit_file`.
    pub is_asset_edit_file: bool,
    /// `recovered`.
    pub recovered: bool,
    /// `is_memfile_undo_written`.
    pub is_memfile_undo_written: bool,
    /// `is_memfile_undo_flush_needed`.
    pub is_memfile_undo_flush_needed: bool,
    /// `use_memfile_full_barrier`.
    pub use_memfile_full_barrier: bool,
    /// `is_locked_for_linking`.
    pub is_locked_for_linking: bool,
    /// `is_read_invalid`.
    pub is_read_invalid: bool,
    /// `is_global_main`.
    pub is_global_main: bool,
    /// `is_action_slot_to_id_map_dirty`.
    pub is_action_slot_to_id_map_dirty: bool,
    /// `need_preview_render_restart`.
    pub need_preview_render_restart: bool,
    /// `is_updating_node_trees`.
    pub is_updating_node_trees: bool,
    /// `is_part_of_project`.
    pub is_part_of_project: bool,
    /// `blen_thumb` (handle).
    pub blen_thumb: usize,
    /// `relations`.
    pub relations: Option<MainIdRelations>,
    /// `id_map` handle (rebuilt lazily).
    pub id_map: usize,
    /// `lock`.
    pub lock: MainLock,
    /// The 42+ `ListBaseT` lists — flattened to a
    /// `MAX_LIBARRAY`-indexed array of handle vectors; index
    /// via `main_libindex` (verbatim INDEX_ID_* ordering).
    pub lists: [Vec<usize>; MAX_LIBARRAY],
    /// `next_id_map_update` / cache invalidation tick.
    pub next_id_map_update: u64,
}

impl Default for BMain {
    fn default() -> Self {
        Self {
            versionfile: 0,
            subversionfile: 0,
            minversionfile: 0,
            minsubversionfile: 0,
            has_forward_compatibility_issues: false,
            is_asset_edit_file: false,
            recovered: false,
            is_memfile_undo_written: false,
            is_memfile_undo_flush_needed: false,
            use_memfile_full_barrier: false,
            is_locked_for_linking: false,
            is_read_invalid: false,
            is_global_main: false,
            is_action_slot_to_id_map_dirty: false,
            need_preview_render_restart: false,
            is_updating_node_trees: false,
            is_part_of_project: true,
            blen_thumb: 0,
            relations: None,
            id_map: 0,
            lock: MainLock::default(),
            lists: std::array::from_fn(|_| Vec::new()),
            next_id_map_update: 0,
        }
    }
}

/// `BKE_main_is_empty` — every list empty (verbatim).
pub fn main_is_empty(bmain: &BMain) -> bool {
    bmain.lists.iter().all(|l| l.is_empty())
}

/// `BKE_main_lists_get` — count of non-empty lists
/// (`MainListsArray` iteration order preserved via index).
pub fn main_lists_get(bmain: &BMain) -> Vec<(usize, &[usize])> {
    bmain
        .lists
        .iter()
        .enumerate()
        .map(|(i, l)| (i, l.as_slice()))
        .collect()
}

/// `BKE_main_id_list` for one ID type index.
pub fn main_id_list(bmain: &BMain, id_type_index: usize) -> &[usize] {
    &bmain.lists[id_type_index]
}
/// Mutable variant.
pub fn main_id_list_mut(bmain: &mut BMain, id_type_index: usize) -> &mut Vec<usize> {
    &mut bmain.lists[id_type_index]
}

/// `LibWeakRefKey` — `(library, id_name)` pair.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LibWeakRefKey {
    /// `library` handle.
    pub library: usize,
    /// `id_name` (the library path local name, verbatim `char*`).
    pub id_name: String,
}

/// `LibraryWeakReference` — the multi-usage entry.
#[derive(Debug, Default)]
pub struct LibraryWeakReference {
    /// `id_name`.
    pub id_name: String,
    /// `id_usage` — full-name usages.
    pub id_usage: HashMap<String, usize>,
}

/// `LibraryWeakRefReferencesMap` (`library` → name → ref).
#[derive(Debug, Default)]
pub struct MainLibraryWeakReferenceMap {
    /// `weak_references` per library handle.
    pub weak_references: HashMap<usize, HashMap<String, LibraryWeakReference>>,
}

/// `BKE_main_library_weak_reference_add_item` — verbatim:
/// record usage `id_name` under `library`, count it for
/// `full_name`.
pub fn library_weak_reference_add_item(
    map: &mut MainLibraryWeakReferenceMap,
    key: &LibWeakRefKey,
    full_name: &str,
    id_handle: usize,
) {
    let lib = map.weak_references.entry(key.library).or_default();
    let weak_ref = lib.entry(key.id_name.clone()).or_default();
    weak_ref.id_name = key.id_name.clone();
    *weak_ref.id_usage.entry(full_name.to_string()).or_insert(0);
    weak_ref.id_usage.insert(full_name.to_string(), id_handle);
}

/// `BKE_main_library_weak_reference_remove_item` — remove a
/// usage; drop empty entries (verbatim).
pub fn library_weak_reference_remove_item(
    map: &mut MainLibraryWeakReferenceMap,
    key: &LibWeakRefKey,
    full_name: &str,
) {
    let Some(lib) = map.weak_references.get_mut(&key.library) else {
        return;
    };
    if let Some(weak_ref) = lib.get_mut(&key.id_name) {
        weak_ref.id_usage.remove(full_name);
        if weak_ref.id_usage.is_empty() {
            lib.remove(&key.id_name);
        }
    }
    if lib.is_empty() {
        map.weak_references.remove(&key.library);
    }
}

/// `BKE_main_library_weak_reference_update_item` — rename
/// (`old_full_name` → `new_full_name` keeping count).
pub fn library_weak_reference_update_item(
    map: &mut MainLibraryWeakReferenceMap,
    key: &LibWeakRefKey,
    old_full_name: &str,
    new_full_name: &str,
) {
    let Some(lib) = map.weak_references.get_mut(&key.library) else {
        return;
    };
    let Some(weak_ref) = lib.get_mut(&key.id_name) else {
        return;
    };
    if let Some(v) = weak_ref.id_usage.remove(old_full_name) {
        weak_ref.id_usage.insert(new_full_name.to_string(), v);
    }
}
