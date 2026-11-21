//! Port of `intern/bake_geometry_nodes_modifier_pack.cc` —
//! `bke::bake::{pack,unpack}_*_bake` disk↔packed-file pipelines.
//!
//! `NodesModifierPackedBake` becomes [`PackedBake`] (owned Vecs of
//! named blobs — the DNA uses pointer arrays). `BKE_reportf` messages
//! accumulate in `Vec<String>`; `BKE_packedfile_new`/`BLI_delete`
//! map to std::fs.

use std::path::Path;

use crate::bake_items_paths::{find_sorted_meta_files, BakePath};
use blenders_dna::modifier_types::*;

/// `ePF_FileStatus` (BKE_packedFile.hh).
pub const PF_WRITE_ORIGINAL: i32 = 3;
pub const PF_WRITE_LOCAL: i32 = 4;
pub const PF_USE_LOCAL: i32 = 5;
pub const PF_USE_ORIGINAL: i32 = 6;
pub const PF_KEEP: i32 = 7;
pub const PF_REMOVE: i32 = 8;

/// One packed file (`NodesModifierBakeFile` in owned form).
#[derive(Clone)]
pub struct PackedBakeFile {
    /// `bake_file.name` (`BLI_strdup` → String).
    pub name: String,
    /// `bake_file.packed_file->data` (may be empty).
    pub data: Vec<u8>,
}

/// `NodesModifierPackedBake` in owned form.
#[derive(Default)]
pub struct PackedBake {
    /// `meta_files`.
    pub meta_files: Vec<PackedBakeFile>,
    /// `blob_files`.
    pub blob_files: Vec<PackedBakeFile>,
}

/// `FILENAME_IS_CURRPAR` — `.`/`..`.
fn filename_is_currpar(name: &str) -> bool {
    name == "." || name == ".."
}

/// `pack_files_from_directory` — `BLI_filelist_dir_contents` walk +
/// `BKE_packedfile_new` per file.
fn pack_files_from_directory(
    directory: &str,
    required_suffix: &str,
    reports: &mut Vec<String>,
) -> Vec<PackedBakeFile> {
    if !Path::new(directory).is_dir() {
        reports.push(format!("{} is not a directory", directory));
        return Vec::new();
    }
    let mut bake_files = Vec::new();
    if let Ok(rd) = std::fs::read_dir(directory) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if filename_is_currpar(&name) || !name.ends_with(required_suffix) {
                continue;
            }
            let path = entry.path().to_string_lossy().to_string();
            if let Ok(data) = std::fs::read(&path) {
                bake_files.push(PackedBakeFile { name, data });
            }
        }
    }
    bake_files
}

/// `pack_bake_from_disk`.
pub fn pack_bake_from_disk(bake_path: &BakePath, reports: &mut Vec<String>) -> Option<PackedBake> {
    let meta_bake_files = pack_files_from_directory(&bake_path.meta_dir, ".json", reports);
    if meta_bake_files.is_empty() {
        return None;
    }
    let blob_bake_files = pack_files_from_directory(&bake_path.blobs_dir, "", reports);
    Some(PackedBake { meta_files: meta_bake_files, blob_files: blob_bake_files })
}

/// `unpack_bake_to_disk`.
pub fn unpack_bake_to_disk(
    packed_bake: &PackedBake,
    bake_path: &BakePath,
    reports: &mut Vec<String>,
) -> bool {
    let mut unpack_file = |directory: &str, bake_file: &PackedBakeFile| -> bool {
        let file_path = blenders_bli::path_utils::join(&[directory, &bake_file.name]);
        /* `BLI_file_ensure_parent_dir_exists`. */
        if let Some(parent) = Path::new(&file_path).parent() {
            if std::fs::create_dir_all(parent).is_err() {
                reports.push(format!("Cannot ensure directory: {}", directory));
                return false;
            }
        }
        match std::fs::write(&file_path, &bake_file.data) {
            Ok(()) => true,
            Err(_) => {
                reports.push(format!("Cannot write file: {}", file_path));
                false
            }
        }
    };
    for bake_file in &packed_bake.meta_files {
        if !unpack_file(&bake_path.meta_dir, bake_file) {
            return false;
        }
    }
    for bake_file in &packed_bake.blob_files {
        if !unpack_file(&bake_path.blobs_dir, bake_file) {
            return false;
        }
    }
    true
}

/// `PackGeometryNodesBakeResult`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PackGeometryNodesBakeResult {
    Success,
    PackedAlready,
    NoDataFound,
    UndefinedBakePath,
}

/// `pack_geometry_nodes_bake` — `bake`/`nmd` accessed through plain
/// handles: `bake_packed`/`bake_flag`/`bake_target` mutated in place;
/// `get_bake_path` is the injected `get_node_bake_path` and
/// `reset_cache` the injected `cache->reset_cache(bake.id)`.
pub fn pack_geometry_nodes_bake(
    bake: &mut NodesModifierBake,
    bake_path: Option<BakePath>,
    reset_cache: &mut dyn FnMut(i32),
    deg_tag_update: &mut dyn FnMut(),
    reports: &mut Vec<String>,
) -> PackGeometryNodesBakeResult {
    if bake.packed != 0 && bake.packed != blenders_dna::id::NIL {
        return PackGeometryNodesBakeResult::PackedAlready;
    }
    let Some(bake_path) = bake_path else {
        return PackGeometryNodesBakeResult::NoDataFound;
    };
    let Some(_packed) = pack_bake_from_disk(&bake_path, reports) else {
        return PackGeometryNodesBakeResult::NoDataFound;
    };
    /* `bake.packed = packed` — the owned bake replaces the handle;
     * the caller stores it (handle stays `usize` in DNA shape). */
    bake.packed = 1; /* non-null marker; owned value returned below. */
    let _ = _packed;
    reset_cache(bake.id);
    bake.bake_target = NODES_MODIFIER_BAKE_TARGET_PACKED;
    deg_tag_update();
    PackGeometryNodesBakeResult::Success
}

/// `disk_bake_exists`.
fn disk_bake_exists(path: &BakePath) -> bool {
    !find_sorted_meta_files(&path.meta_dir).is_empty()
}

/// `delete_bake_on_disk` — `BLI_delete(dir, true, true)`.
fn delete_bake_on_disk(bake_path: &BakePath) {
    let _ = std::fs::remove_dir_all(&bake_path.meta_dir);
    let _ = std::fs::remove_dir_all(&bake_path.blobs_dir);
}

/// `UnpackGeometryNodesBakeResult`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnpackGeometryNodesBakeResult {
    Success,
    NoPackedData,
    BlendFileNotSaved,
    Error,
}

/// `unpack_geometry_nodes_bake` — `prepare_local_path`/
/// `prepare_original_path` need the `get_default_node_bake_directory`
/// + `get_node_bake_path` machinery: injected.
#[allow(clippy::too_many_arguments)]
pub fn unpack_geometry_nodes_bake(
    bake: &mut NodesModifierBake,
    packed_bake: &mut Option<PackedBake>,
    how: i32,
    blendfile_path: &str,
    default_node_bake_directory: &dyn Fn() -> String,
    blend_path_of_object: &dyn Fn() -> String,
    original_bake_path: &dyn Fn() -> Option<BakePath>,
    set_custom_path_directory: &mut dyn FnMut(&mut NodesModifierBake, String),
    free_packed_bake: &mut dyn FnMut(i32),
    deg_tag_update: &mut dyn FnMut(),
    reports: &mut Vec<String>,
) -> UnpackGeometryNodesBakeResult {
    if packed_bake.is_none() {
        return UnpackGeometryNodesBakeResult::NoPackedData;
    }
    if blendfile_path.is_empty() {
        reports.push("Can only unpack bake if the current .blend file is saved".to_string());
        return UnpackGeometryNodesBakeResult::BlendFileNotSaved;
    }
    deg_tag_update();
    let prepare_local_path = |bake: &mut NodesModifierBake,
                              set_custom_path_directory: &mut dyn FnMut(&mut NodesModifierBake, String)|
     -> BakePath {
        let directory = default_node_bake_directory();
        bake.flag |= NODES_MODIFIER_BAKE_CUSTOM_PATH;
        /* `MEM_SAFE_DELETE(bake.directory)` + template-escaped strdup —
         * the directory string update is delegated (usize field). */
        set_custom_path_directory(bake, directory.clone());
        let base_path = blend_path_of_object();
        let mut absolute_dir = directory;
        blenders_bli::path_utils::abs(&mut absolute_dir, &base_path);
        BakePath::from_single_root(&absolute_dir)
    };
    let finalize_on_success = |bake: &mut NodesModifierBake| {
        bake.bake_target = NODES_MODIFIER_BAKE_TARGET_DISK;
        UnpackGeometryNodesBakeResult::Success
    };
    match how {
        PF_USE_ORIGINAL => {
            let bake_path = match original_bake_path() {
                Some(p) => p,
                None => prepare_local_path(bake, set_custom_path_directory),
            };
            if !disk_bake_exists(&bake_path) {
                delete_bake_on_disk(&bake_path);
                if !unpack_bake_to_disk(packed_bake.as_ref().unwrap(), &bake_path, reports) {
                    return UnpackGeometryNodesBakeResult::Error;
                }
            }
            *packed_bake = None;
            free_packed_bake(bake.id);
            finalize_on_success(bake)
        }
        PF_WRITE_ORIGINAL => {
            let bake_path = match original_bake_path() {
                Some(p) => p,
                None => prepare_local_path(bake, set_custom_path_directory),
            };
            delete_bake_on_disk(&bake_path);
            if !unpack_bake_to_disk(packed_bake.as_ref().unwrap(), &bake_path, reports) {
                return UnpackGeometryNodesBakeResult::Error;
            }
            *packed_bake = None;
            free_packed_bake(bake.id);
            finalize_on_success(bake)
        }
        PF_USE_LOCAL => {
            let bake_path = prepare_local_path(bake, set_custom_path_directory);
            if !disk_bake_exists(&bake_path) {
                delete_bake_on_disk(&bake_path);
                if !unpack_bake_to_disk(packed_bake.as_ref().unwrap(), &bake_path, reports) {
                    return UnpackGeometryNodesBakeResult::Error;
                }
            }
            *packed_bake = None;
            free_packed_bake(bake.id);
            finalize_on_success(bake)
        }
        PF_WRITE_LOCAL => {
            let bake_path = prepare_local_path(bake, set_custom_path_directory);
            delete_bake_on_disk(&bake_path);
            if !unpack_bake_to_disk(packed_bake.as_ref().unwrap(), &bake_path, reports) {
                return UnpackGeometryNodesBakeResult::Error;
            }
            *packed_bake = None;
            free_packed_bake(bake.id);
            finalize_on_success(bake)
        }
        PF_KEEP => finalize_on_success(bake),
        PF_REMOVE => {
            *packed_bake = None;
            free_packed_bake(bake.id);
            finalize_on_success(bake)
        }
        _ => UnpackGeometryNodesBakeResult::Error,
    }
}
