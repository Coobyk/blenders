/// Port of `alembic/intern/abstract_hierarchy_iterator.cc`
/// — shared DFS walk.
/// `AbstractHierarchyIterator` (verbatim —
/// visit each context).
pub struct AbstractHierarchyIterator {
    /// `contexts`.
    pub contexts: Vec<usize>,
}
