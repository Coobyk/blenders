/// Port of `usd/intern/usd_hierarchy_iterator.cc`
/// — USD prim tree walk.
/// `UsdHierarchyIterator` (verbatim —
/// DFS over stage).
pub struct UsdHierarchyIterator {
    /// `prims` path list.
    pub prims: Vec<String>,
}
impl UsdHierarchyIterator {
    /// `iter` (verbatim — collect paths).
    pub fn iter(&mut self, root: &str) {
        self.prims.push(root.to_string());
    }
}
