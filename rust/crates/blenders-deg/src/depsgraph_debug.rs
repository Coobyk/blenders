//! Port of `intern/depsgraph_debug.cc` — debug flags,
/// consistency check, stats, eval-time print helpers.

/// `DEG_debug_flags_set` / `_get` (verbatim — arbitrary
/// flags stored per-graph; semantics chosen by callers).
pub fn deg_debug_flags_set(flags: &mut i32, new_flags: i32) {
    *flags = new_flags;
}
pub fn deg_debug_flags_get(flags: i32) -> i32 {
    flags
}

/// `DEG_debug_name_set` / `_get` (verbatim).
pub fn deg_debug_name_set<'a>(name: &mut &'a str, new_name: &'a str) {
    *name = new_name;
}
pub fn deg_debug_name_get<'a>(name: &'a str) -> &'a str {
    name
}

/// `depsgraph_name_for_logging` (verbatim — falls back to
/// "graph").
pub fn depsgraph_name_for_logging<'a>(name: &'a str) -> &'a str {
    if name.is_empty() { "graph" } else { name }
}

/// `DEG_debug_compare` (verbatim — currently only compares
/// operation counts).
pub fn deg_debug_compare(ops1_len: usize, ops2_len: usize) -> bool {
    ops1_len == ops2_len
}

/// Hooks for `deg_debug_consistency_check` — the graph
/// node stores aren't unified yet, so links are walked via
/// indices.
pub struct DebugCheckCtx<'a> {
    /// All operation-node handles.
    pub operations: &'a dyn Fn() -> Vec<usize>,
    /// outlink/inlink relation handles of a node.
    pub outlinks: &'a dyn Fn(usize) -> Vec<usize>,
    /// inlink relation handles.
    pub inlinks: &'a dyn Fn(usize) -> Vec<usize>,
    /// Relation endpoints (from, to).
    pub rel_ends: &'a dyn Fn(usize) -> (usize, usize),
    /// `node->type == OPERATION`.
    pub is_operation_node: &'a dyn Fn(usize) -> bool,
    /// `node->num_links_pending` get/set.
    pub num_links_pending: &'a mut dyn FnMut(usize, Option<usize>) -> usize,
    /// `node->custom_flags` get/set.
    pub custom_flags: &'a mut dyn FnMut(usize, Option<u32>) -> u32,
    /// `node->identifier()` for diagnostics.
    pub identifier: &'a dyn Fn(usize) -> String,
}

/// `DEG_debug_consistency_check` (verbatim — bidirectional
/// link counts + valency).
pub fn deg_debug_consistency_check(ctx: &mut DebugCheckCtx) -> bool {
    let ops = (ctx.operations)();
    /* Validate links exist in both directions. */
    for node in &ops {
        for rel in (ctx.outlinks)(*node) {
            let counter1 = (ctx.outlinks)(*node).iter().filter(|r| **r == rel).count();
            let to = (ctx.rel_ends)(rel).1;
            let counter2 = (ctx.inlinks)(to).iter().filter(|r| **r == rel).count();
            if counter1 != counter2 {
                return false;
            }
        }
    }
    for node in &ops {
        for rel in (ctx.inlinks)(*node) {
            let counter1 = (ctx.inlinks)(*node).iter().filter(|r| **r == rel).count();
            let from = (ctx.rel_ends)(rel).0;
            let counter2 = (ctx.outlinks)(from).iter().filter(|r| **r == rel).count();
            if counter1 != counter2 {
                return false;
            }
        }
    }
    /* Validate valency in both directions. */
    for node in &ops {
        (ctx.num_links_pending)(*node, Some(0));
        (ctx.custom_flags)(*node, Some(0));
    }
    for node in &ops {
        if (ctx.custom_flags)(*node, None) != 0 {
            /* Node twice in operations. */
            return false;
        }
        for rel in (ctx.outlinks)(*node) {
            let to = (ctx.rel_ends)(rel).1;
            if (ctx.is_operation_node)(to) {
                let n = (ctx.num_links_pending)(to, None);
                (ctx.num_links_pending)(to, Some(n + 1));
            }
        }
        (ctx.custom_flags)(*node, Some(1));
    }
    for node in &ops {
        let num_links_pending = (ctx.inlinks)(*node)
            .iter()
            .filter(|rel| (ctx.is_operation_node)((ctx.rel_ends)(**rel).0))
            .count();
        if (ctx.num_links_pending)(*node, None) != num_links_pending {
            return false;
        }
    }
    true
}

/// `DEG_stats_simple` (verbatim — outer nodes, operations,
/// relations counts).
pub fn deg_stats_simple(
    id_components: &dyn Fn() -> Vec<(usize, usize)>,
    comp_operations: &dyn Fn(usize) -> Vec<usize>,
    op_inlinks: &dyn Fn(usize) -> usize,
    time_source_inlinks: usize,
    tot_operations: usize,
) -> (usize, usize, usize) {
    let mut tot_outer = 0usize;
    let mut tot_rels = 0usize;
    let mut seen_ids = std::collections::HashSet::new();
    for (id, comp) in id_components() {
        if seen_ids.insert(id) {
            tot_outer += 1;
        }
        tot_outer += 1;
        for op in comp_operations(comp) {
            tot_rels += op_inlinks(op);
        }
    }
    tot_rels += time_source_inlinks;
    (tot_outer, tot_operations, tot_rels)
}
