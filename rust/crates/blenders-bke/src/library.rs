//! Port of `intern/library.cc` — `Library` ID handling:
//! `BKE_library_filepath_set`, `search_filepath_abs`,
//! `add_archive_library` / `ensure_archive_library`,
//! `BKE_library_main_rebuild_hierarchy`, `main_cleanup_parent_archives`.
//!
//! `Main`, ID relations, namemap and `BKE_id_new` are injected;
//! flag math and the hierarchy algorithm are ported verbatim.

use std::collections::{HashMap, HashSet};

use blenders_dna::id::{Library, NIL};

/// `LibraryRuntime` (BKE_library.hh).
#[derive(Clone, Default)]
pub struct LibraryRuntime {
    /// `filepath_abs[FILE_MAX]` → owned string.
    pub filepath_abs: String,
    /// `parent`.
    pub parent: usize,
    /// `archived_libraries`.
    pub archived_libraries: Vec<usize>,
    /// `temp_index`.
    pub temp_index: i32,
    /// `tag`.
    pub tag: u16,
    /// `versionfile`.
    pub versionfile: i32,
    /// `subversionfile`.
    pub subversionfile: i32,
}

/// `LIBRARY_FLAG_IS_ARCHIVE` (id_enums) — re-declared as u16.
pub const LIBRARY_FLAG_IS_ARCHIVE: u16 = 1 << 0;
/// `LIBRARY_TAG_RESYNC_REQUIRED`.
pub const LIBRARY_TAG_RESYNC_REQUIRED: u16 = 1 << 0;
/// `LIBRARY_ASSET_EDITABLE`.
pub const LIBRARY_ASSET_EDITABLE: u16 = 1 << 1;
/// `LIBRARY_IS_ASSET_EDIT_FILE`.
pub const LIBRARY_IS_ASSET_EDIT_FILE: u16 = 1 << 3;
/// `LIBRARY_FLAG_MASK` copy mask from `add_archive_library`.
pub const LIBRARY_ARCHIVE_COPY_TAG: u16 =
    LIBRARY_TAG_RESYNC_REQUIRED | LIBRARY_ASSET_EDITABLE | LIBRARY_IS_ASSET_EDIT_FILE;

/// `BKE_library_filepath_set`.
pub fn bke_library_filepath_set(
    lib: &mut Library,
    runtime: &mut LibraryRuntime,
    filepath: &str,
    blendfile_path: &dyn Fn() -> String,
) {
    if lib.filepath != filepath.as_bytes() {
        let mut buf = [0u8; 1024];
        let n = filepath.len().min(buf.len() - 1);
        buf[..n].copy_from_slice(&filepath.as_bytes()[..n]);
        lib.filepath = buf;
    }
    runtime.filepath_abs = filepath.to_string();
    if filepath.starts_with("//") {
        runtime.filepath_abs =
            blendfile_path() + &runtime.filepath_abs[2..];
    }
}

/// `bke::library::search_filepath_abs` — handles keyed by index.
pub fn library_search_filepath_abs(
    libraries: &[(usize, u16 /* flag */, &LibraryRuntime)],
    filepath_abs: &str,
) -> Option<usize> {
    for &(handle, flag, rt) in libraries {
        if flag & LIBRARY_FLAG_IS_ARCHIVE != 0 {
            continue;
        }
        if rt.filepath_abs == filepath_abs {
            return Some(handle);
        }
    }
    None
}

/// `add_archive_library` — caller supplies `new_lib` (the
/// `BKE_id_new` result); this fills its archive fields.
pub fn add_archive_library(
    reference_handle: usize,
    reference: &Library,
    reference_runtime: &LibraryRuntime,
    archive: &mut Library,
    archive_runtime: &mut LibraryRuntime,
    main_version: i32,
    main_subversion: i32,
) {
    debug_assert!(reference.flag & LIBRARY_FLAG_IS_ARCHIVE == 0);
    archive.archive_parent_library = reference_handle;
    let copy_flag = !LIBRARY_FLAG_IS_ARCHIVE;
    archive.flag = (reference.flag & copy_flag) | LIBRARY_FLAG_IS_ARCHIVE;
    archive_runtime.parent = reference_runtime.parent;
    archive_runtime.tag = reference_runtime.tag & LIBRARY_ARCHIVE_COPY_TAG;
    archive_runtime.versionfile = main_version;
    archive_runtime.subversionfile = main_subversion;
}

/// `bke::library::ensure_archive_library` — find or create the
/// archive holding `id_name`. `contain_name(lib_handle)` =
/// `BKE_main_namemap_contain_name`; `create()` = `add_archive_library`
/// + push onto `reference_runtime.archived_libraries`.
/// Returns `(archive_handle, is_new)`.
pub fn ensure_archive_library(
    reference_runtime: &mut LibraryRuntime,
    id_name: &str,
    libraries: &dyn Fn(usize) -> u16,
    contain_name: &dyn Fn(usize, &str) -> bool,
    create: &mut dyn FnMut() -> usize,
) -> (usize, bool) {
    let mut archive_handle = NIL;
    for &lib in &reference_runtime.archived_libraries.clone() {
        let flag = libraries(lib);
        debug_assert!(flag & LIBRARY_FLAG_IS_ARCHIVE != 0);
        if contain_name(lib, id_name) {
            continue;
        }
        archive_handle = lib;
        break;
    }
    if archive_handle == NIL {
        archive_handle = create();
        reference_runtime.archived_libraries.push(archive_handle);
        return (archive_handle, true);
    }
    (archive_handle, false)
}

/// `rebuild_hierarchy_best_parent_find` — recursive best-parent
/// search. `linked_ids_of(lib)` → id handles linking `lib`;
/// `from_libs(id)` → non-packed source libraries via relations;
/// `libs`/`runtimes` are handle-keyed maps.
pub fn rebuild_hierarchy_best_parent_find(
    directly_used_libs: &mut HashSet<usize>,
    libs_in_hierarchy: &mut HashSet<usize>,
    lib: usize,
    runtimes: &mut HashMap<usize, LibraryRuntime>,
    linked_ids_of: &dyn Fn(usize) -> Vec<usize>,
    from_libs_of: &dyn Fn(usize) -> Vec<usize>,
    is_packed: &dyn Fn(usize) -> bool,
) {
    debug_assert!(!directly_used_libs.contains(&lib));
    if libs_in_hierarchy.contains(&lib) {
        return;
    }
    libs_in_hierarchy.insert(lib);

    let mut best_parent_lib: Option<usize> = None;
    'outer: for id_iter in linked_ids_of(lib) {
        if is_packed(id_iter) {
            continue;
        }
        for from_lib in from_libs_of(id_iter) {
            if is_packed(from_lib) {
                continue;
            }
            if from_lib == lib {
                continue;
            }
            if directly_used_libs.contains(&from_lib) {
                best_parent_lib = Some(from_lib);
                break 'outer;
            }
            let from_has_parent = runtimes
                .get(&from_lib)
                .is_some_and(|r| r.parent != NIL);
            if !from_has_parent {
                rebuild_hierarchy_best_parent_find(
                    directly_used_libs,
                    libs_in_hierarchy,
                    from_lib,
                    runtimes,
                    linked_ids_of,
                    from_libs_of,
                    is_packed,
                );
            }
            let from_idx = runtimes.get(&from_lib).map(|r| r.temp_index).unwrap_or(0);
            let best_idx = best_parent_lib
                .and_then(|b| runtimes.get(&b).map(|r| r.temp_index))
                .unwrap_or(i32::MAX);
            if best_parent_lib.is_none() || best_idx > from_idx {
                best_parent_lib = Some(from_lib);
                if from_idx == 0 {
                    if !directly_used_libs.contains(&from_lib) {
                        if let Some(rt) = runtimes.get_mut(&from_lib) {
                            rt.temp_index = 1;
                        }
                    }
                    break 'outer;
                }
            }
        }
    }

    match best_parent_lib {
        Some(best) => {
            let best_idx = runtimes.get(&best).map(|r| r.temp_index).unwrap_or(0);
            if let Some(rt) = runtimes.get_mut(&lib) {
                rt.parent = best;
                rt.temp_index = best_idx + 1;
            }
        }
        None => {
            if let Some(rt) = runtimes.get_mut(&lib) {
                rt.parent = NIL;
                rt.temp_index = 0;
            }
            directly_used_libs.insert(lib);
        }
    }
    libs_in_hierarchy.remove(&lib);
}

/// `BKE_library_main_rebuild_hierarchy` — phase 1+2 (temp_index
/// sweep, directly-used detection, parent chains) before the
/// recursive best-parent pass, then a final pass assigning indices.
pub fn library_main_rebuild_hierarchy(
    libraries: &[usize],
    flags: &dyn Fn(usize) -> u16,
    archive_parent_of: &dyn Fn(usize) -> usize,
    runtimes: &mut HashMap<usize, LibraryRuntime>,
    linked_ids_of: &dyn Fn(usize) -> Vec<usize>,
    all_linked_ids: &dyn Fn() -> Vec<(usize, usize)>, /* (id, lib) */
    from_libs_of: &dyn Fn(usize) -> Vec<usize>,
    is_packed: &dyn Fn(usize) -> bool,
) {
    for &lib in libraries {
        let rt = runtimes.entry(lib).or_default();
        if flags(lib) & LIBRARY_FLAG_IS_ARCHIVE != 0 {
            rt.parent = archive_parent_of(lib);
            rt.temp_index = 1;
        } else {
            rt.temp_index = 0;
        }
    }

    /* Directly used: a linked id referenced by a non-linked id. */
    let mut directly_used_libs: HashSet<usize> = HashSet::new();
    for (id_iter, id_lib) in all_linked_ids() {
        if is_packed(id_iter) || directly_used_libs.contains(&id_lib) {
            continue;
        }
        /* `from_libs_of` yields only linked, non-packed sources;
         * an empty list means a local user → directly used. */
        let froms = from_libs_of(id_iter);
        if froms.is_empty() {
            directly_used_libs.insert(id_lib);
            if let Some(rt) = runtimes.get_mut(&id_lib) {
                rt.parent = NIL;
            }
        }
    }

    /* Resolve parent chains with known indices. */
    for &lib in libraries {
        if directly_used_libs.contains(&lib) {
            continue;
        }
        let mut chain: Vec<usize> = Vec::new();
        let mut cur = lib;
        loop {
            let (parent, idx) = runtimes.get(&cur).map(|r| (r.parent, r.temp_index)).unwrap_or((NIL, 0));
            if parent == NIL || idx != 0 || chain.contains(&cur) {
                break;
            }
            chain.push(cur);
            cur = parent;
        }
        if chain.is_empty() {
            continue;
        }
        let ancestor = *chain.last().unwrap();
        let mut parent_temp_index = runtimes
            .get(&ancestor)
            .map(|r| r.parent)
            .filter(|&p| p != NIL)
            .and_then(|p| runtimes.get(&p).map(|r| r.temp_index))
            .unwrap_or(0)
            + chain.len() as i32
            - 1;
        for &l in &chain {
            if let Some(rt) = runtimes.get_mut(&l) {
                rt.temp_index = parent_temp_index;
            }
            parent_temp_index -= 1;
        }
    }

    /* Best-parent pass for the rest. */
    let mut libs_in_hierarchy = HashSet::new();
    for &lib in libraries {
        if directly_used_libs.contains(&lib) {
            continue;
        }
        let has_parent = runtimes.get(&lib).is_some_and(|r| r.parent != NIL);
        let has_idx = runtimes.get(&lib).is_some_and(|r| r.temp_index > 0);
        if has_parent || has_idx {
            continue;
        }
        rebuild_hierarchy_best_parent_find(
            &mut directly_used_libs,
            &mut libs_in_hierarchy,
            lib,
            runtimes,
            linked_ids_of,
            from_libs_of,
            is_packed,
        );
    }
}

/// `bke::library::main_cleanup_parent_archives` — drop archive
/// libraries whose parent is gone or that hold no IDs.
/// `has_ids(lib)` counts packed IDs the archive owns.
pub fn library_main_cleanup_parent_archives(
    libraries: &mut Vec<usize>,
    flags: &dyn Fn(usize) -> u16,
    parent_of: &dyn Fn(usize) -> usize,
    has_ids: &dyn Fn(usize) -> bool,
    free_lib: &mut dyn FnMut(usize),
) {
    libraries.retain(|&lib| {
        if flags(lib) & LIBRARY_FLAG_IS_ARCHIVE == 0 {
            return true;
        }
        let parent = parent_of(lib);
        let keep = parent != NIL && has_ids(lib);
        if !keep {
            free_lib(lib);
        }
        keep
    });
}
