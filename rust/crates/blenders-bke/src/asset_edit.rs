//! Port of `intern/asset_edit.cc` — `BKE_asset_edit_*` (asset browser
//! "Edit"/"Save" operations).
//!
//! `BLO` write/link (`lib_write_ctx`, `asset_link_id` internals) is
//! PORT_PENDING — callers inject `write_file`/`link_id` closures.

use blenders_bli::{fileops_c, path_utils, string, storage::is_file};
use blenders_dna::asset_types::{ASSET_LIBRARY_CUSTOM, ASSET_LIBRARY_ESSENTIALS};
use blenders_dna::userdef_types::bUserAssetLibrary;

/// `BLENDER_ASSET_FILE_SUFFIX` (BKE_asset_edit.hh).
pub const BLENDER_ASSET_FILE_SUFFIX: &str = ".blend";

/// [u8;N] → trimmed &str (C string semantics).
fn cstr(buf: &[u8]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

/// `asset_name_in_library` — `library_weak_reference->library_id_name + 2`
/// for linked assets, else the ID name. `weak_ref_id_name` = the weak
/// ref name (None when no weak ref / not linked); `id_name` = `BKE_id_name`.
pub fn asset_name_in_library(is_linked: bool, weak_ref_id_name: Option<&str>, id_name: &str) -> String {
    if is_linked {
        if let Some(w) = weak_ref_id_name {
            return w.get(2..).unwrap_or(w).to_string();
        }
    }
    id_name.to_string()
}

/// `asset_name_in_library_set` — writes `name` into the weak-ref's
/// `library_id_name` ("XX<name>") or `BKE_id_name_set` equivalent.
pub fn asset_name_in_library_set(
    is_linked: bool,
    weak_ref_id_name: &mut Option<[u8; 256]>,
    name: &str,
) {
    if is_linked && weak_ref_id_name.is_some() {
        let buf = weak_ref_id_name.as_mut().unwrap();
        let mut out = [0u8; 256];
        out[0] = b'X';
        out[1] = b'X';
        let n = name.as_bytes();
        let copy = n.len().min(253);
        out[2..2 + copy].copy_from_slice(&n[..copy]);
        *buf = out;
    }
}

/// `asset_root_path_for_save` — `<lib>/Saved/<TypeName_plural>`.
/// `id_type_name_plural` = `BKE_idtype_get_info_from_idcode(id_type)->name_plural`.
pub fn asset_root_path_for_save(user_library: &bUserAssetLibrary, id_type_name_plural: &str) -> String {
    let mut libpath = cstr(&user_library.resolved_dirpath);
    path_utils::slash_native(&mut libpath);
    path_utils::normalize(&mut libpath);
    path_utils::slash_rstrip(&mut libpath);
    let mut name = id_type_name_plural.to_string();
    if let Some(c) = name.chars().next() {
        name.replace_range(..1, &string::toupper_ascii(c).to_string());
    }
    format!("{}/Saved/{}", libpath, name)
}

/// `asset_blendfile_path_for_save` — unique `<root>/<name>[_N].blend`.
/// Returns Err on directory-create failure.
pub fn asset_blendfile_path_for_save(
    user_library: &bUserAssetLibrary,
    base_name: &str,
    id_type_name_plural: &str,
) -> Result<String, &'static str> {
    let root_path = asset_root_path_for_save(user_library, id_type_name_plural);
    if !fileops_c::dir_create_recursive_pub(&root_path) {
        return Err("Failed to create asset library directory to save asset");
    }
    let mut base_name_filesafe = base_name.chars().take(255).collect::<String>();
    path_utils::make_safe_filename(&mut base_name_filesafe);
    let filepath = format!("{}/{}{}", root_path, base_name_filesafe, BLENDER_ASSET_FILE_SUFFIX);
    if !is_file(&filepath) {
        return Ok(filepath);
    }
    for i in 1i32.. {
        let filepath = format!(
            "{}/{}_{}{}",
            root_path, base_name_filesafe, i, BLENDER_ASSET_FILE_SUFFIX
        );
        if !is_file(&filepath) {
            return Ok(filepath);
        }
    }
    unreachable!()
}

/// `asset_weak_reference_for_user_library` — `ASSET_LIBRARY_CUSTOM`
/// weak ref keyed by library name; `filepath` is returned for the caller
/// to build `relative_asset_identifier` (C resolves it lazily).
pub fn asset_weak_reference_for_user_library(
    user_library: &bUserAssetLibrary,
    _idcode: i16,
    idname: &str,
    filepath: &str,
) -> crate::asset_weak_reference::AssetWeakReference {
    let _ = filepath;
    crate::asset_weak_reference::AssetWeakReference {
        asset_library_type: ASSET_LIBRARY_CUSTOM as i16,
        asset_library_identifier: Some(cstr(&user_library.name)),
        relative_asset_identifier: Some(idname.to_string()),
    }
}

/// `asset_weak_reference_for_online_essentials` /
/// `asset_weak_reference_for_essentials` — essentials library weak ref.
pub fn asset_weak_reference_for_essentials(_idcode: i16, idname: &str) -> crate::asset_weak_reference::AssetWeakReference {
    crate::asset_weak_reference::AssetWeakReference {
        asset_library_type: ASSET_LIBRARY_ESSENTIALS as i16,
        asset_library_identifier: None,
        relative_asset_identifier: Some(idname.to_string()),
    }
}
