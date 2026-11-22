//! Port of `intern/depsgraph_relation.cc` — `Relation::unlink`.

/// `Relation::unlink` — removes `relation` from `from->outlinks`
/// and `to->inlinks` (verbatim `remove_first_occurrence_and_reorder`
/// semantics: swap-remove).
pub fn relation_unlink<T>(from_outlinks: &mut Vec<T>, to_inlinks: &mut Vec<T>, relation: T)
where
    T: PartialEq,
{
    if let Some(i) = from_outlinks.iter().position(|r| *r == relation) {
        from_outlinks.swap_remove(i);
    }
    if let Some(i) = to_inlinks.iter().position(|r| *r == relation) {
        to_inlinks.swap_remove(i);
    }
}
