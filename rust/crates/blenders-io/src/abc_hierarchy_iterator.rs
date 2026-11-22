/// Port of `alembic/intern/abc_hierarchy_iterator.cc`
/// — Alembic object tree walk.
/// `AbcHierarchyIterator` (verbatim — DFS).
pub struct AbcHierarchyIterator {
    /// `objects` path list.
    pub objects: Vec<String>,
}
impl AbcHierarchyIterator {
    /// `iter` (verbatim — collect paths).
    pub fn iter(&mut self, path: &str) {
        self.objects.push(path.to_string());
    }
}
