//! Port of `intern/asset_catalog_collection.cc` —
//! owning catalog map + deep copy.

/// `AssetCatalog` — canonical type lives in
/// `asset_catalog`.
pub use crate::asset_catalog::AssetCatalog;
/// `CatalogID` alias matching `asset_catalog`.
pub type CatalogId = crate::asset_catalog::CatalogId;
/// `AssetCatalogCollection` verbatim.
#[derive(Default)]
pub struct AssetCatalogCollection {
    /// `catalogs_` by catalog_id.
    pub catalogs: std::collections::HashMap<CatalogId, AssetCatalog>,
    /// `deleted_catalogs_` by catalog_id.
    pub deleted_catalogs: std::collections::HashMap<CatalogId, AssetCatalog>,
    /// `has_unsaved_changes_`.
    pub has_unsaved_changes: bool,
}
impl AssetCatalogCollection {
    /// `is_empty` (verbatim).
    pub fn is_empty(&self) -> bool {
        self.catalogs.is_empty() && self.deleted_catalogs.is_empty()
    }
    /// `copy_catalog_map_into_existing` (verbatim —
    /// skip dups or call hook).
    pub fn copy_catalog_map_into_existing(
        source: &std::collections::HashMap<CatalogId, AssetCatalog>,
        dest: &mut std::collections::HashMap<CatalogId, AssetCatalog>,
        on_duplicate: Option<&dyn Fn(&mut AssetCatalog, &AssetCatalog)>,
    ) {
        for orig in source.values() {
            if dest.contains_key(&orig.catalog_id) {
                if let Some(cb) = on_duplicate {
                    let d = dest.get_mut(&orig.catalog_id).unwrap();
                    (cb)(d, orig);
                }
                continue;
            }
            dest.insert(orig.catalog_id, orig.clone());
        }
    }
    /// `add_catalogs_from_existing` (verbatim).
    pub fn add_catalogs_from_existing(
        &mut self,
        other: &AssetCatalogCollection,
        on_duplicate: Option<&dyn Fn(&mut AssetCatalog, &AssetCatalog)>,
    ) {
        Self::copy_catalog_map_into_existing(&other.catalogs, &mut self.catalogs, on_duplicate);
    }
    /// `deep_copy` (verbatim — copy flags + both maps).
    pub fn deep_copy(&self) -> AssetCatalogCollection {
        let mut copy = AssetCatalogCollection::default();
        copy.has_unsaved_changes = self.has_unsaved_changes;
        copy.catalogs = self.catalogs.clone();
        copy.deleted_catalogs = self.deleted_catalogs.clone();
        copy
    }
}
