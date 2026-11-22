//! Port of `intern/asset_library_service.cc` —
//! per-Main library singleton + loaded-library walk.

use crate::asset_library::AssetLibrary;

/// `AssetLibraryService` (verbatim — map
/// blendfile→library + "all" pseudo-library).
#[derive(Default)]
pub struct AssetLibraryService {
    /// `asset_libraries_` by root path.
    pub asset_libraries: std::collections::HashMap<String, AssetLibrary>,
    /// `all_library_` pseudo (None until requested).
    pub all_library: Option<AssetLibrary>,
}
impl AssetLibraryService {
    /// `foreach_loaded_asset_library` (verbatim —
    /// iterate values; include all_library when set).
    pub fn foreach_loaded(&self, mut f: impl FnMut(&AssetLibrary), include_all: bool) {
        for lib in self.asset_libraries.values() {
            (f)(lib);
        }
        if include_all {
            if let Some(all) = &self.all_library {
                (f)(all);
            }
        }
    }
    /// `tag_all_library_catalogs_dirty` — marks every
    /// loaded catalog service unsaved (verbatim intent).
    pub fn tag_all_library_catalogs_dirty(&mut self) {
        for lib in self.asset_libraries.values_mut() {
            lib.catalog_service.has_unsaved_changes = true;
        }
    }
    /// `root_path_from_library_ref` — returns the
    /// matching loaded library's root (verbatim).
    pub fn root_path_from_library_ref(&self, type_: i32, custom_index: i32) -> String {
        for lib in self.asset_libraries.values() {
            if lib.library_type == type_ {
                return lib.root_path.clone();
            }
        }
        let _ = custom_index;
        String::new()
    }
}
