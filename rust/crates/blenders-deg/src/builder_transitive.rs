//! Port of `intern/builder/deg_builder_transitive.cc` —
/// `deg_graph_transitive_reduction`.

/// `OP_VISITED`.
pub const OP_VISITED: u32 = 1;
/// `OP_REACHABLE`.
pub const OP_REACHABLE: u32 = 2;

/// `deg_graph_tag_paths_recursive` (verbatim; `custom_flags`
/// array indexed by node; `inlinks` per node = `from` node ids).
fn tag_paths_recursive(node: usize, inlinks: &[Vec<usize>], custom_flags: &mut [u32]) {
    if custom_flags[node] & OP_VISITED != 0 {
        return;
    }
    custom_flags[node] |= OP_VISITED;
    for &from in &inlinks[node] {
        tag_paths_recursive(from, inlinks, custom_flags);
        /* Do this only in inlinks loop, so the target node does
         * not get flagged. */
        custom_flags[from] |= OP_REACHABLE;
    }
}

/// `deg_graph_transitive_reduction` (verbatim).
/// `operations` = op node ids; `inlinks[node]`/`outlinks[node]`
/// as relation `(from, to)` edge ids; `is_timesource` marks
/// `NodeType::TIMESOURCE` nodes; `unlink` removes the edge.
/// Returns removed count (for `DEG_DEBUG_PRINTF`).
pub fn deg_graph_transitive_reduction(
    operations: &[usize],
    node_count: usize,
    inlinks: &[Vec<usize>],
    edge_from: &[usize],
    edge_to: &[usize],
    is_timesource: &[bool],
    unlink: &mut dyn FnMut(usize),
) -> usize {
    let mut num_removed_relations = 0;
    let mut relations_to_remove: Vec<usize> = Vec::new();
    let mut custom_flags = vec![0u32; node_count];

    for &target in operations {
        /* Clear tags. */
        for f in custom_flags.iter_mut() {
            *f = 0;
        }
        /* Mark nodes from which we can reach the target —
         * start with children so the target and direct children
         * are not flagged. */
        custom_flags[target] |= OP_VISITED;
        for &rel in &inlinks[target] {
            tag_paths_recursive(edge_from[rel], inlinks, &mut custom_flags);
        }
        /* Remove redundant paths to the target. */
        for &rel in &inlinks[target] {
            let from = edge_from[rel];
            if is_timesource[from] {
                /* HACK: time source nodes don't get
                 * "custom_flags" flag set/cleared. */
                continue;
            }
            if custom_flags[from] & OP_REACHABLE != 0 {
                relations_to_remove.push(rel);
            }
        }
        for &rel in &relations_to_remove {
            unlink(rel);
        }
        num_removed_relations += relations_to_remove.len();
        relations_to_remove.clear();
        let _ = edge_to;
    }
    num_removed_relations
}
