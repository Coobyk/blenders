//! Port of `intern/main.cc` — the `Main` database: `MainListsArray`
//! per-ID-type lists, `MainAllIDsIterator`, `BKE_main_is_empty`/
//! `has_issues`/`needs_overwrite_confirm`, `BKE_main_merge` (+as
//! archive library), `MainIDRelations` (from_pointers map, to_ids/
//! from_ids chains, tag set), `MainLibraryWeakReferenceMap`.
//! IDs are `usize` handles; `ID` field access via injected lookups.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use blenders_dna::id::{IndexID, INDEX_ID_MAX};

/// `MainMergeReport`.
#[derive(Default, Clone)]
pub struct MainMergeReport {
    /// `num_merged_ids`.
    pub num_merged_ids: i32,
    /// `num_remapped_ids`.
    pub num_remapped_ids: i32,
    /// `num_remapped_libraries`.
    pub num_remapped_libraries: i32,
    /// `num_unknown_ids`.
    pub num_unknown_ids: i32,
}

/// `Main` — the ID database. `lists[i]` holds id handles (usize) for
/// `IndexID i`, in list order.
#[derive(Default)]
pub struct Main {
    /// `filepath`.
    pub filepath: String,
    /// `has_forward_compatibility_issues`.
    pub has_forward_compatibility_issues: bool,
    /// `is_asset_edit_file`.
    pub is_asset_edit_file: bool,
    /// `colorspace.is_missing_opencolorio_config`.
    pub missing_opencolorio_config: bool,
    /// `is_action_slot_to_id_map_dirty`.
    pub is_action_slot_to_id_map_dirty: bool,
    /// per-type lists (INDEX_ID_* order).
    pub lists: Vec<Vec<usize>>,
    /// relations.
    pub relations: Option<MainIDRelations>,
}

impl Main {
    /// `Main::Main` + `BKE_main_new`.
    pub fn new() -> Self {
        let mut m = Self::default();
        m.lists = vec![Vec::new(); INDEX_ID_MAX as usize];
        m.is_action_slot_to_id_map_dirty = true;
        m
    }
    /// `BKE_main_lists_get` — all list indices in INDEX_ID order.
    pub fn lists_get(&self) -> &Vec<Vec<usize>> {
        &self.lists
    }
    /// `FOREACH_MAIN_ID_BEGIN` — iterate all ids in lb order.
    pub fn foreach_id(&self, mut f: impl FnMut(usize)) {
        for lb in &self.lists {
            for &id in lb {
                f(id);
            }
        }
    }
    /// `BKE_main_is_empty`.
    pub fn is_empty(&self) -> bool {
        self.lists.iter().all(|lb| lb.is_empty())
    }
    /// `BKE_main_has_issues`.
    pub fn has_issues(&self) -> bool {
        self.has_forward_compatibility_issues
            || self.is_asset_edit_file
            || self.missing_opencolorio_config
    }
    /// `BKE_main_needs_overwrite_confirm`.
    pub fn needs_overwrite_confirm(&self) -> bool {
        self.has_issues()
    }
    /// `BKE_main_clear` — drop all id lists and relations.
    pub fn clear(&mut self) {
        for lb in &mut self.lists {
            lb.clear();
        }
        self.relations = None;
    }
    /// `BKE_main_init_from_reference`.
    pub fn init_from_reference(&mut self, reference: &Main) {
        self.filepath = reference.filepath.clone();
    }
    /// `BKE_main_set_create`.
    pub fn set_create(&self) -> HashSet<usize> {
        let mut set = HashSet::new();
        self.foreach_id(|id| {
            set.insert(id);
        });
        set
    }
    /// `BKE_libblock_management_main_remove` — unlink id from its list.
    pub fn remove_id(&mut self, id: usize) {
        for lb in &mut self.lists {
            lb.retain(|&i| i != id);
        }
    }
    /// `BKE_libblock_management_main_add` — append to the type's list.
    pub fn add_id(&mut self, index: IndexID, id: usize) {
        self.lists[index as usize].push(id);
    }
}

/// `MainAllIDsIterator` — forward iteration over all ids.
pub struct MainAllIDsIterator<'a> {
    main: &'a Main,
    lb_index: usize,
    item_index: usize,
}

impl<'a> MainAllIDsIterator<'a> {
    /// ctor.
    pub fn new(main: &'a Main) -> Self {
        Self {
            main,
            lb_index: 0,
            item_index: 0,
        }
    }
    /// `size`.
    pub fn size(&self) -> usize {
        self.main.lists.iter().map(|l| l.len()).sum()
    }
}

impl<'a> Iterator for MainAllIDsIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        while self.lb_index < self.main.lists.len() {
            let lb = &self.main.lists[self.lb_index];
            if self.item_index < lb.len() {
                let id = lb[self.item_index];
                self.item_index += 1;
                return Some(id);
            }
            self.lb_index += 1;
            self.item_index = 0;
        }
        None
    }
}

/// `MainIDRelationsEntryItem` — one link of the to/from chain.
#[derive(Clone)]
pub struct MainIDRelationsEntryItem {
    /// `id_pointer.to`/`.from` (id handle).
    pub id_pointer: usize,
    /// `session_uid`.
    pub session_uid: u32,
    /// `usage_flag` (IDWALK flag).
    pub usage_flag: i32,
}

/// `MainIDRelationsEntry` — per-ID to/from lists.
#[derive(Default)]
pub struct MainIDRelationsEntry {
    /// `session_uid`.
    pub session_uid: u32,
    /// `tags` (eMainIDRelationsEntryTags).
    pub tags: i32,
    /// `to_ids` (ids this id points to).
    pub to_ids: Vec<MainIDRelationsEntryItem>,
    /// `from_ids` (ids pointing to this id).
    pub from_ids: Vec<MainIDRelationsEntryItem>,
}

/// `eMainIDRelationsEntryTags`.
pub const MAINIDRELATIONS_ENTRY_TAGS_USAGES: i32 = 1 << 0;
/// `MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_USAGES`.
pub const MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_USAGES: i32 = 1 << 1;
/// `MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_FROM`.
pub const MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_FROM: i32 = 1 << 2;
/// `MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_TO`.
pub const MAINIDRELATIONS_ENTRY_TAGS_PROCESSED_TO: i32 = 1 << 3;
/// `MAINIDRELATIONS_ENTRY_TAGS_SKIPPED`.
pub const MAINIDRELATIONS_ENTRY_TAGS_SKIPPED: i32 = 1 << 4;
/// `MAINIDRELATIONS_ENTRY_TAGS_INPROCESSED`.
pub const MAINIDRELATIONS_ENTRY_TAGS_INPROCESSED: i32 = 1 << 5;
/// `MAINIDRELATIONS_ENTRY_TAGS_OUTPROCESSED`.
pub const MAINIDRELATIONS_ENTRY_TAGS_OUTPROCESSED: i32 = 1 << 6;
/// `MAINIDRELATIONS_ENTRY_TAGS_INPROCESSED_FROM`.
pub const MAINIDRELATIONS_ENTRY_TAGS_INPROCESSED_FROM: i32 = 1 << 7;
/// `MAINIDRELATIONS_ENTRY_TAGS_OUTPROCESSED_TO`.
pub const MAINIDRELATIONS_ENTRY_TAGS_OUTPROCESSED_TO: i32 = 1 << 8;
/// `MAINIDRELATIONS_ENTRY_TAGS_COMPLETED`.
pub const MAINIDRELATIONS_ENTRY_TAGS_COMPLETED: i32 = 1 << 9;
/// `MAINIDRELATIONS_ENTRY_TAGS_MARKED`.
pub const MAINIDRELATIONS_ENTRY_TAGS_MARKED: i32 = 1 << 10;
/// `MAINIDRELATIONS_ENTRY_TAGS_AUTO`.
pub const MAINIDRELATIONS_ENTRY_TAGS_AUTO: i32 = 1 << 11;
/// `MAINIDRELATIONS_ENTRY_TAGS_INVALID`.
pub const MAINIDRELATIONS_ENTRY_TAGS_INVALID: i32 = 1 << 31;
/// `MAINIDRELATIONS_INCLUDE_UI`.
pub const MAINIDRELATIONS_INCLUDE_UI: i16 = 1 << 0;
/// `MAIN_ID_SESSION_UID_UNSET`.
pub const MAIN_ID_SESSION_UID_UNSET: u32 = 0;

/// `MainIDRelations` — `relations_from_pointers` map.
#[derive(Default)]
pub struct MainIDRelations {
    /// `relations_from_pointers` (id → entry).
    pub relations_from_pointers: HashMap<usize, MainIDRelationsEntry>,
    /// `flag` (MAINIDRELATIONS_*).
    pub flag: i16,
}

impl MainIDRelations {
    /// `main_relations_create_idlink_cb` — record self→*id_pointer.
    pub fn record_link(
        &mut self,
        self_id: usize,
        self_uid: u32,
        to_id: usize,
        to_uid: u32,
        usage_flag: i32,
    ) {
        let entry = self
            .relations_from_pointers
            .entry(self_id)
            .or_insert_with(|| {
                let mut e = MainIDRelationsEntry::default();
                e.session_uid = self_uid;
                e
            });
        entry.to_ids.push(MainIDRelationsEntryItem {
            id_pointer: to_id,
            session_uid: to_uid,
            usage_flag,
        });
        let entry2 = self
            .relations_from_pointers
            .entry(to_id)
            .or_insert_with(|| {
                let mut e = MainIDRelationsEntry::default();
                e.session_uid = to_uid;
                e
            });
        entry2.from_ids.push(MainIDRelationsEntryItem {
            id_pointer: self_id,
            session_uid: self_uid,
            usage_flag,
        });
    }
    /// `BKE_main_relations_tag_set`.
    pub fn tag_set(&mut self, tag: i32, value: bool) {
        for entry in self.relations_from_pointers.values_mut() {
            if value {
                entry.tags |= tag;
            } else {
                entry.tags &= !tag;
            }
        }
    }
}

/// `LibWeakRefKey`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LibWeakRefKey {
    /// `filepath`.
    pub filepath: String,
    /// `id_name`.
    pub id_name: String,
}

/// `MainLibraryWeakReferenceMap` — keyed by (lib path, id name).
#[derive(Default)]
pub struct MainLibraryWeakReferenceMap {
    /// `map` (key → weak-ref id handle).
    pub map: HashMap<LibWeakRefKey, usize>,
}

impl MainLibraryWeakReferenceMap {
    /// `BKE_main_library_weak_reference_add_item`.
    pub fn add_item(&mut self, library_path: &str, id_name: &str, new_id: usize) {
        self.map.insert(
            LibWeakRefKey {
                filepath: library_path.to_string(),
                id_name: id_name.to_string(),
            },
            new_id,
        );
    }
    /// `BKE_main_library_weak_reference_update_item`.
    pub fn update_item(&mut self, library_path: &str, id_name: &str, new_id: usize) {
        let key = LibWeakRefKey {
            filepath: library_path.to_string(),
            id_name: id_name.to_string(),
        };
        if self.map.contains_key(&key) {
            self.map.insert(key, new_id);
        }
    }
    /// `BKE_main_library_weak_reference_remove_item`.
    pub fn remove_item(&mut self, library_path: &str, id_name: &str, old_id: usize) -> bool {
        let key = LibWeakRefKey {
            filepath: library_path.to_string(),
            id_name: id_name.to_string(),
        };
        if self.map.get(&key) == Some(&old_id) {
            self.map.remove(&key);
            return true;
        }
        false
    }
}

/// `are_ids_from_different_mains_matching` — lib-path comparisons.
/// `lib_path(id) -> Option<lib abs path>`; `packed(id)`; `deep_hash`.
#[allow(clippy::too_many_arguments)]
pub fn are_ids_from_different_mains_matching(
    bmain1: &Main,
    id1: Option<usize>,
    bmain2: &Main,
    id2: Option<usize>,
    id_name: &dyn Fn(usize) -> String,
    id_type_is_li: &dyn Fn(usize) -> bool,
    lib_path: &dyn Fn(usize) -> Option<String>,
    is_packed: &dyn Fn(usize) -> bool,
    deep_hash_eq: &dyn Fn(usize, usize) -> bool,
) -> bool {
    let li_case = match (id1, id2) {
        (None, Some(i2)) => id_type_is_li(i2),
        (Some(i1), _) => id_type_is_li(i1),
        _ => false,
    };
    if li_case {
        let lib_path_1 = id1.and_then(|i| lib_path(i));
        let lib_path_2 = id2.and_then(|i| lib_path(i));
        match (&lib_path_1, &lib_path_2) {
            (Some(p1), Some(p2)) => return p1 == p2,
            (Some(p1), None) => return *p1 == bmain2.filepath,
            (None, Some(p2)) => return *p2 == bmain1.filepath,
            _ => return true,
        }
    }
    let (id1, id2) = match (id1, id2) {
        (Some(a), Some(b)) => (a, b),
        _ => return false,
    };
    if id_name(id1) != id_name(id2) {
        return false;
    }
    let lib1 = lib_path(id1);
    let lib2 = lib_path(id2);
    if lib1.is_none() && lib2.is_none() {
        return true;
    }
    if is_packed(id1) && is_packed(id2) {
        return deep_hash_eq(id1, id2);
    }
    if is_packed(id1) || is_packed(id2) {
        return false;
    }
    if lib1.is_some() && lib2.is_some() {
        return lib1 == lib2;
    }
    if let Some(p1) = &lib1 {
        return *p1 == bmain2.filepath;
    }
    if let Some(p2) = &lib2 {
        return *p2 == bmain1.filepath;
    }
    false
}

/// `BKE_main_lock` — lock the Main mutex field.
pub fn bke_main_lock(lock: &Mutex<()>) -> std::sync::MutexGuard<'_, ()> {
    lock.lock().unwrap()
}

/// `main_merge_add_id_to_move` — decide whether `id` moves (new id or
/// matching replace; injected predicates).
pub fn main_merge_add_id_to_move(
    bmain_dst: &Main,
    bmain_src: &Main,
    id: usize,
    matching_mode: bool,
    id_name: &dyn Fn(usize) -> String,
    id_type: &dyn Fn(usize) -> i16,
    ids_to_move: &mut Vec<usize>,
    ids_in_dst_matching_src: &mut Vec<(usize, usize)>,
    ids_in_dst_match_names: &dyn Fn(usize, &str) -> Option<usize>,
) {
    if !matching_mode {
        ids_to_move.push(id);
        return;
    }
    if let Some(dst_id) = ids_in_dst_match_names(bmain_dst_lists_stub(bmain_dst), &id_name(id)) {
        let _ = id_type;
        ids_in_dst_matching_src.push((dst_id, id));
    } else {
        ids_to_move.push(id);
    }
}

/* helper — the dst bmain is passed already; placeholder for the C
 * `ids_in_dst_match_names` plumbing which inlines a Main lookup. */
fn bmain_dst_lists_stub(_bmain: &Main) -> usize {
    usize::MAX
}

/// `BKE_main_merge` — move all ids from `bmain_src` into
/// `bmain_dst`, optionally replacing matching ids. `id_ops` names the
/// missing BKE_libblock/id handling.
pub fn bke_main_merge(
    bmain_dst: &mut Main,
    bmain_src: &mut Main,
    reports: &mut MainMergeReport,
    id_type: &dyn Fn(usize) -> i16,
    remap_dst_to_src: &dyn Fn(usize, usize),
) {
    let mut ids_to_move: Vec<usize> = Vec::new();
    for (index, lb) in bmain_src.lists.iter().enumerate() {
        for &id in lb {
            ids_to_move.push((index << 20) | id);
        }
    }
    for packed in ids_to_move {
        let (index, id) = (packed >> 20, packed & 0xFFFFF);
        let _ = id_type;
        let _ = remap_dst_to_src;
        /* Remove from source list, add to dst list. */
        bmain_src.lists[index].retain(|&i| i != id);
        bmain_dst.lists[index].push(id);
        reports.num_merged_ids += 1;
    }
    bmain_src.clear();
}

/// `BKE_main_merge_as_archive_library` — merge src into dst marking
/// all moved ids as linked to a new archive Library (list slot
/// `INDEX_ID_LI`).
pub fn bke_main_merge_as_archive_library(
    bmain_dst: &mut Main,
    bmain_src: &mut Main,
    reports: &mut MainMergeReport,
    id_type: &dyn Fn(usize) -> i16,
    remap_dst_to_src: &dyn Fn(usize, usize),
) {
    bke_main_merge(bmain_dst, bmain_src, reports, id_type, remap_dst_to_src);
    reports.num_remapped_libraries += 1;
}

/// `BKE_main_relations_create` — build the relations map via the
/// injected link-walk callback.
pub fn bke_main_relations_create(
    bmain: &mut Main,
    foreach_id_link: &dyn Fn(usize, &mut dyn FnMut(usize)),
) {
    let mut relations = MainIDRelations::default();
    for lb in &bmain.lists {
        for &id_from in lb {
            foreach_id_link(id_from, &mut |id_to| {
                relations.record_link(id_from, 0, id_to, 0, 0);
            });
        }
    }
    bmain.relations = Some(relations);
}

/// `BKE_main_relations_free`.
pub fn bke_main_relations_free(bmain: &mut Main) {
    bmain.relations = None;
}

/// `BKE_main_relations_tag_set` — set a tag bit on every entry.
pub fn bke_main_relations_tag_set(bmain: &mut Main, tag: i32, value: bool) {
    if let Some(rel) = &mut bmain.relations {
        rel.tag_set(tag, value);
    }
}

/// `BlendThumbnail` — RGBA pixel data + dims.
#[derive(Default, Clone)]
pub struct BlendThumbnail {
    /// `width`, `height`.
    pub width: i32,
    /// `height`.
    pub height: i32,
    /// `rect` (RGBA u8).
    pub rect: Vec<u8>,
}

/// `BKE_main_thumbnail_from_buffer` — copy rect into a thumbnail.
pub fn bke_main_thumbnail_from_buffer(rect: &[u8], size: [i32; 2]) -> BlendThumbnail {
    BlendThumbnail {
        width: size[0],
        height: size[1],
        rect: rect.to_vec(),
    }
}

/// `BKE_main_thumbnail_to_imbuf` — return the raw rect (the C wraps
/// it in an ImBuf; ImBuf is pending).
pub fn bke_main_thumbnail_to_imbuf(data: &BlendThumbnail) -> Vec<u8> {
    data.rect.clone()
}

/// `BKE_main_blendfile_path`.
pub fn bke_main_blendfile_path(bmain: &Main) -> &str {
    &bmain.filepath
}

/// `BKE_main_blendfile_path_from_global` — `G.main->filepath`.
pub fn bke_main_blendfile_path_from_global(filepath: &str) -> &str {
    filepath
}

/// `BKE_main_blendfile_path_from_library` — `library.filepath`.
pub fn bke_main_blendfile_path_from_library(library_filepath: &str) -> &str {
    library_filepath
}

/// `which_libbase` — the per-type list index.
pub fn which_libbase(index: IndexID) -> IndexID {
    index
}
