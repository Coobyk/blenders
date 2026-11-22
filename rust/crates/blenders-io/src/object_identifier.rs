/// Port of `common/intern/object_identifier.cc`
/// — stable Object identity for export.

/// `ObjectIdentifier` (verbatim — union of
/// real object / dupli context / graph root).
#[derive(Clone, PartialEq)]
pub enum ObjectIdentifier {
    /// `for_real_object`.
    RealObject(usize),
    /// `for_hierarchy_context`.
    HierarchyContext(usize),
    /// `for_duplicated_object` (parent chain).
    DuplicatedObject(usize, usize),
    /// `for_graph_root`.
    GraphRoot,
}
impl ObjectIdentifier {
    /// `is_root` (verbatim).
    pub fn is_root(&self) -> bool {
        matches!(self, ObjectIdentifier::GraphRoot)
    }
}
