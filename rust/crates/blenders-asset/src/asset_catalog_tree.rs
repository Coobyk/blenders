//! Port of `intern/asset_catalog_tree.cc` — catalog
//! tree item hierarchy over paths.

/// `AssetCatalogTreeItem` verbatim fields.
#[derive(Default)]
pub struct AssetCatalogTreeItem {
    /// `name` (last path component).
    pub name: String,
    /// `catalog_path`.
    pub catalog_path: crate::asset_catalog_path::AssetCatalogPath,
    /// `children_` by simple name.
    pub children: std::collections::BTreeMap<String, AssetCatalogTreeItem>,
    /// `has_unsaved_changes`.
    pub has_unsaved_changes: bool,
}
impl AssetCatalogTreeItem {
    /// `has_children` (verbatim).
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
    /// `count_parents` (verbatim — depth via '/').
    pub fn count_parents(&self) -> i32 {
        self.catalog_path.path.matches('/').count() as i32
    }
    /// `foreach_item_recursive` (verbatim — self then
    /// children dfs).
    pub fn foreach_item(&self, callback: &mut dyn FnMut(&AssetCatalogTreeItem)) {
        (callback)(self);
        for child in self.children.values() {
            child.foreach_item(callback);
        }
    }
}
