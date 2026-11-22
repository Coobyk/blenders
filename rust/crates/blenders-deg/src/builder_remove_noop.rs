//! Port of `intern/builder/deg_builder_remove_noop.cc` —
/// `deg_graph_remove_unused_noops`.

use std::collections::VecDeque;

/// `OperationFlag::DEPSOP_FLAG_PINNED`.
pub const DEPSOP_FLAG_PINNED: u32 = 1;

/// `is_unused_noop` (verbatim — `is_noop` + empty outlinks,
/// not pinned).
pub fn is_unused_noop(flag: u32, is_noop: bool, outlinks_empty: bool) -> bool {
    if flag & DEPSOP_FLAG_PINNED != 0 {
        return false;
    }
    is_noop && outlinks_empty
}

/// `is_removable_relation` (verbatim — non-op endpoints are
/// removable; op→op relations only when both ends share
/// `owner` (same component), so cross-ID/component visibility
/// flushing stays intact).
pub fn is_removable_relation(from_is_operation: bool, to_is_operation: bool, same_owner: bool) -> bool {
    if !from_is_operation || !to_is_operation {
        return true;
    }
    same_owner
}

/// `deg_graph_remove_unused_noops` (verbatim BFS).
/// `op_flags`/`op_is_noop` per node; `inlinks`/`outlinks` as
/// edge `(from, to)` id lists; `edge_from`/`edge_to` maps;
/// `from_is_operation`/`to_is_operation`/`same_owner` per edge;
/// `unlink` removes an edge.
#[allow(clippy::too_many_arguments)]
pub fn deg_graph_remove_unused_noops(
    operations: &[usize],
    op_flags: &[u32],
    op_is_noop: &[bool],
    inlinks: &[Vec<usize>],
    outlinks: &[Vec<usize>],
    edge_from: &[usize],
    edge_to: &[usize],
    edge_removable: &dyn Fn(usize) -> bool,
    unlink: &mut dyn FnMut(usize),
) {
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut removed = vec![false; op_flags.len()];

    for &node in operations {
        if op_flags[node] & DEPSOP_FLAG_PINNED == 0
            && op_is_noop[node]
            && outlinks[node].is_empty()
        {
            queue.push_back(node);
        }
    }

    while let Some(to_remove) = queue.pop_front() {
        if removed[to_remove] {
            continue;
        }
        removed[to_remove] = true;
        for &rel_in in &inlinks[to_remove] {
            if !edge_removable(rel_in) {
                continue;
            }
            let dependency = edge_from[rel_in];
            unlink(rel_in);
            if op_flags[dependency] & DEPSOP_FLAG_PINNED == 0
                && op_is_noop[dependency]
                && outlinks[dependency].is_empty()
            {
                queue.push_back(dependency);
            }
            let _ = edge_to;
        }
    }
}
