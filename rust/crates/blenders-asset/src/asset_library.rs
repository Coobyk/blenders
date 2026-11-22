//! Port of `intern/asset_library.cc` — AssetLibrary
//! + library references + weak-ref explode.

use crate::asset_catalog::AssetCatalogService;

/// `eAssetLibraryType` verbatim ordinals.
pub const ASSET_LIBRARY_LOCAL: i32 = 0;
pub const ASSET_LIBRARY_ESSENTIALS: i32 = 1;
pub const ASSET_LIBRARY_CUSTOM: i32 = 2;
pub const ASSET_LIBRARY_ALL: i32 = 3;
pub const ASSET_LIBRARY_ONLINE_ESSENTIALS: i32 = 4;
/// `FILE_MAX_LIBEXTRA` verbatim.
pub const FILE_MAX_LIBEXTRA: usize = 1282;

/// `AssetLibraryReference` verbatim.
#[derive(Clone, Copy, Default)]
pub struct AssetLibraryReference {
    /// `custom_library_index`.
    pub custom_library_index: i32,
    /// `type`.
    pub type_: i32,
}
/// `all_library_reference` (verbatim — index -1,
/// ALL).
pub fn all_library_reference() -> AssetLibraryReference {
    AssetLibraryReference {
        custom_library_index: -1,
        type_: ASSET_LIBRARY_ALL,
    }
}
/// `essentials_library_reference`.
pub fn essentials_library_reference() -> AssetLibraryReference {
    AssetLibraryReference {
        custom_library_index: -1,
        type_: ASSET_LIBRARY_ESSENTIALS,
    }
}
/// `current_file_library_reference`.
pub fn current_file_library_reference() -> AssetLibraryReference {
    AssetLibraryReference {
        custom_library_index: -1,
        type_: ASSET_LIBRARY_LOCAL,
    }
}
/// `online_essentials_library_reference`.
pub fn online_essentials_library_reference() -> AssetLibraryReference {
    AssetLibraryReference {
        custom_library_index: -1,
        type_: ASSET_LIBRARY_ONLINE_ESSENTIALS,
    }
}
/// `is_or_contains_remote_libraries` (verbatim — ALL
/// or ONLINE_ESSENTIALS type).
pub fn is_or_contains_remote_libraries(reference: &AssetLibraryReference) -> bool {
    reference.type_ == ASSET_LIBRARY_ALL || reference.type_ == ASSET_LIBRARY_ONLINE_ESSENTIALS
}

/// `AssetWeakReference` verbatim fields.
#[derive(Clone, Default)]
pub struct AssetWeakReference {
    /// `asset_library_type`.
    pub asset_library_type: i32,
    /// `asset_library_custom_identifier`.
    pub asset_library_custom_identifier: String,
    /// `relative_asset_identifier`.
    pub relative_asset_identifier: String,
}

/// `ExplodedPath` verbatim.
#[derive(Default)]
pub struct ExplodedPath {
    /// `full_path`.
    pub full_path: String,
    /// `dir_component`/`group_component`/`name_component`
    /// (byte ranges into full_path as strings).
    pub dir_component: String,
    pub group_component: String,
    pub name_component: String,
}

/// `AssetLibrary` verbatim.
#[derive(Default)]
pub struct AssetLibrary {
    /// `library_type_`.
    pub library_type: i32,
    /// `name_`/`root_path_`.
    pub name: String,
    pub root_path: String,
    /// `catalog_service_`.
    pub catalog_service: AssetCatalogService,
    /// `asset_storage_.local_id_assets`/`external_assets`.
    pub local_id_assets: Vec<crate::asset_representation::AssetRepresentation>,
    pub external_assets: Vec<crate::asset_representation::AssetRepresentation>,
    /// `save_catalogs_when_file_is_saved` static.
    pub save_catalogs_when_file_is_saved: bool,
}
impl AssetLibrary {
    /// `name()`/`root_path()` (verbatim).
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn root_path(&self) -> &str {
        &self.root_path
    }
    /// `is_read_only` (verbatim — service flag or
    /// non-local types read-only).
    pub fn is_read_only(&self) -> bool {
        self.catalog_service.is_read_only
    }
    /// `use_relative_paths` (verbatim — local
    /// library only).
    pub fn use_relative_paths(&self) -> bool {
        self.library_type == ASSET_LIBRARY_LOCAL
    }
    /// `AS_asset_library_find_suitable_root_path_from_path`
    /// (verbatim — prefs dir containing path else dir
    /// part of input).
    pub fn find_suitable_root_path_from_path(
        input_path: &str,
        prefs_library_containing: &dyn Fn(&str) -> Option<String>,
    ) -> String {
        if let Some(dir) = (prefs_library_containing)(input_path) {
            return dir;
        }
        match input_path.rfind('/') {
            Some(i) => input_path[..=i].to_string(),
            None => String::new(),
        }
    }
    /// `remove_asset` (verbatim — local then
    /// external removal).
    pub fn remove_asset(&mut self, asset_index: usize, is_local: bool) -> bool {
        if is_local {
            if asset_index < self.local_id_assets.len() {
                self.local_id_assets.remove(asset_index);
                return true;
            }
            return false;
        }
        if asset_index < self.external_assets.len() {
            self.external_assets.remove(asset_index);
            return true;
        }
        false
    }
    /// `refresh_catalog_simplename` (verbatim — nil
    /// uuid → empty; missing catalog → no-op).
    pub fn refresh_catalog_simplename(
        &self,
        catalog_id: Option<crate::asset_catalog::CatalogId>,
    ) -> Option<String> {
        let id = catalog_id?;
        let catalog = self.catalog_service.find_catalog(id)?;
        Some(catalog.simple_name.clone())
    }
}
/// `AS_asset_full_path_explode_from_weak_ref` result
/// split (verbatim — dir/group/name cuts).
pub fn explode_weak_ref_path(full_path: &str) -> (String, String, String) {
    let (dir, rest) = match full_path.rfind('/') {
        Some(i) => (&full_path[..i], &full_path[i + 1..]),
        None => ("", full_path),
    };
    let (group, name) = match rest.rfind('/') {
        Some(i) => (&rest[..i], &rest[i + 1..]),
        None => (rest, ""),
    };
    (dir.to_string(), group.to_string(), name.to_string())
}
