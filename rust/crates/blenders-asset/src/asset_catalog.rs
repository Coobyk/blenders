//! Port of `intern/asset_catalog.cc` —
//! AssetCatalogService + AssetCatalog + filter.

use crate::asset_catalog_path::AssetCatalogPath;

/// `CatalogFilePath` default name verbatim.
pub const DEFAULT_CATALOG_FILENAME: &str = "blender_assets.cats.txt";
/// `CatalogID` verbatim (128-bit uuid → u128).
pub type CatalogId = u128;
/// `MAX_NAME` verbatim.
pub const MAX_NAME: usize = 64;

/// `AssetCatalog` verbatim fields.
#[derive(Clone, Default)]
pub struct AssetCatalog {
    /// `catalog_id`.
    pub catalog_id: CatalogId,
    /// `path`.
    pub path: AssetCatalogPath,
    /// `simple_name`.
    pub simple_name: String,
    /// `flags.is_deleted`/`has_unsaved_changes`.
    pub is_deleted: bool,
    pub has_unsaved_changes: bool,
}
impl AssetCatalog {
    /// `AssetCatalog::from_path` (verbatim — clean path
    /// + uuid + sensible simple name).
    pub fn from_path(path: &AssetCatalogPath, catalog_id: CatalogId) -> AssetCatalog {
        let clean_path = path.cleanup();
        let simple_name = Self::sensible_simple_name_for_path(&clean_path);
        AssetCatalog {
            catalog_id,
            path: clean_path,
            simple_name,
            is_deleted: false,
            has_unsaved_changes: false,
        }
    }
    /// `simple_name_refresh` (verbatim).
    pub fn simple_name_refresh(&mut self) {
        self.simple_name = Self::sensible_simple_name_for_path(&self.path);
    }
    /// `sensible_simple_name_for_path` (verbatim —
    /// '/'→'-'; >63 chars → "..." + last 60).
    pub fn sensible_simple_name_for_path(path: &AssetCatalogPath) -> String {
        let name = path.path.replace(AssetCatalogPathSEPARATOR, "-");
        if name.len() < MAX_NAME - 1 {
            return name;
        }
        format!("...{}", &name[name.len() - 60..])
    }
}
/// separator as a literal for replace().
const AssetCatalogPathSEPARATOR: char = '/';

/// `AssetCatalogService` (verbatim fields).
#[derive(Default)]
pub struct AssetCatalogService {
    /// `catalog_collection_->catalogs_`.
    pub catalogs: std::collections::HashMap<CatalogId, AssetCatalog>,
    /// `catalog_collection_->deleted_catalogs_`.
    pub deleted_catalogs: std::collections::HashMap<CatalogId, AssetCatalog>,
    /// `catalog_collection_->has_unsaved_changes_`.
    pub has_unsaved_changes: bool,
    /// `is_read_only_`.
    pub is_read_only: bool,
}
impl AssetCatalogService {
    /// `tag_has_unsaved_changes` (verbatim — catalog +
    /// collection flags).
    pub fn tag_has_unsaved_changes(&mut self, catalog_id: Option<CatalogId>) {
        if let Some(id) = catalog_id {
            if let Some(c) = self.catalogs.get_mut(&id) {
                c.has_unsaved_changes = true;
            }
        }
        self.has_unsaved_changes = true;
    }
    /// `untag_has_unsaved_changes` (verbatim — clear
    /// deleted, unflag all).
    pub fn untag_has_unsaved_changes(&mut self) {
        self.has_unsaved_changes = false;
        self.deleted_catalogs.clear();
        for c in self.catalogs.values_mut() {
            c.has_unsaved_changes = false;
        }
    }
    /// `is_empty` (verbatim).
    pub fn is_empty(&self) -> bool {
        self.catalogs.is_empty() && self.deleted_catalogs.is_empty()
    }
    /// `find_catalog` (verbatim).
    pub fn find_catalog(&self, catalog_id: CatalogId) -> Option<&AssetCatalog> {
        self.catalogs.get(&catalog_id)
    }
    /// `find_catalog_by_path` (verbatim — first match in
    /// ordered set → lowest simple_name here).
    pub fn find_catalog_by_path(&self, path: &AssetCatalogPath) -> Option<&AssetCatalog> {
        self.catalogs
            .values()
            .filter(|c| c.path == *path)
            .min_by(|a, b| a.simple_name.cmp(&b.simple_name))
    }
    /// `is_catalog_known` (verbatim).
    pub fn is_catalog_known(&self, catalog_id: CatalogId) -> bool {
        self.catalogs.contains_key(&catalog_id)
    }
    /// `delete_catalog_by_id_soft` (verbatim — mark
    /// deleted, move to deleted map).
    pub fn delete_catalog_by_id_soft(&mut self, catalog_id: CatalogId) {
        if let Some(mut cat) = self.catalogs.remove(&catalog_id) {
            cat.is_deleted = true;
            self.deleted_catalogs.insert(catalog_id, cat);
        }
    }
    /// `delete_catalog_by_id_hard` (verbatim — drop
    /// both maps).
    pub fn delete_catalog_by_id_hard(&mut self, catalog_id: CatalogId) {
        self.catalogs.remove(&catalog_id);
        self.deleted_catalogs.remove(&catalog_id);
    }
    /// `prune_catalogs_by_path` (verbatim — collect
    /// contained ids then soft-delete).
    pub fn prune_catalogs_by_path(&mut self, path: &AssetCatalogPath) {
        let to_delete: Vec<CatalogId> = self
            .catalogs
            .values()
            .filter(|c| c.path.is_contained_in(path))
            .map(|c| c.catalog_id)
            .collect();
        for id in to_delete {
            self.delete_catalog_by_id_soft(id);
        }
    }
    /// `prune_catalogs_by_id` (verbatim).
    pub fn prune_catalogs_by_id(&mut self, catalog_id: CatalogId) {
        if let Some(cat) = self.find_catalog(catalog_id) {
            let path = cat.path.clone();
            self.prune_catalogs_by_path(&path);
        }
    }
    /// `update_catalog_path` (verbatim — rebase all
    /// contained paths, tag changed).
    pub fn update_catalog_path(&mut self, catalog_id: CatalogId, new_path: &AssetCatalogPath) {
        let old_path = match self.find_catalog(catalog_id) {
            Some(c) => c.path.clone(),
            None => return,
        };
        let ids: Vec<CatalogId> = self.catalogs.keys().copied().collect();
        for id in ids {
            let rebased = {
                let cat = self.catalogs.get(&id).unwrap();
                cat.path.rebase(&old_path, new_path)
            };
            if rebased.is_empty() {
                continue;
            }
            let cat = self.catalogs.get_mut(&id).unwrap();
            cat.path = rebased;
            cat.simple_name_refresh();
            cat.has_unsaved_changes = true;
            self.has_unsaved_changes = true;
        }
    }
    /// `create_catalog` (verbatim — from_path + flag +
    /// insert).
    pub fn create_catalog(&mut self, catalog_path: &AssetCatalogPath, catalog_id: CatalogId) -> CatalogId {
        let mut catalog = AssetCatalog::from_path(catalog_path, catalog_id);
        catalog.has_unsaved_changes = true;
        self.catalogs.insert(catalog_id, catalog);
        catalog_id
    }
    /// `create_missing_catalogs` partial — parents of
    /// known paths inserted verbatim semantics.
    pub fn missing_parent_paths(&self) -> Vec<AssetCatalogPath> {
        let mut missing = Vec::new();
        for cat in self.catalogs.values() {
            let mut p = cat.path.parent();
            while !p.is_empty() {
                if !self.catalogs.values().any(|c| c.path == p) {
                    missing.push(p.clone());
                }
                p = p.parent();
            }
        }
        missing
    }
}
