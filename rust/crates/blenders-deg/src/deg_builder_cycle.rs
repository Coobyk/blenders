//! Port of `intern/builder/deg_builder_cycle.cc` —
/// iterative DFS cycle detector marking sacrificial
/// relations `RELATION_FLAG_CYCLIC`.

/// `eCyclicCheckVisitedState` (verbatim).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum CyclicCheckVisitedState {
    /// `NODE_NOT_VISITED`.
    NotVisited = 0,
    /// `NODE_VISITED`.
    Visited = 1,
    /// `NODE_IN_STACK`.
    InStack = 2,
}

/// `StackEntry` (verbatim — parent is the stack index of
/// the entry it was pushed from).
#[derive(Clone)]
pub struct StackEntry {
    /// `node` handle.
    pub node: usize,
    /// `from` — parent entry index (`usize::MAX` = null).
    pub from: usize,
    /// `via_relation` handle.
    pub via_relation: usize,
}

/// `CyclesSolverState` (verbatim — owns traversal stack +
/// cycle count).
pub struct CyclesSolverState {
    /// `traversal_stack`.
    pub traversal_stack: Vec<StackEntry>,
    /// `num_cycles`.
    pub num_cycles: usize,
}

/// `check_relation_can_murder` (verbatim — god-mode
/// relations are sacred).
pub fn check_relation_can_murder(rel_flag: u32) -> bool {
    const RELATION_FLAG_GODMODE: u32 = 1 << 4;
    rel_flag & RELATION_FLAG_GODMODE == 0
}

/// Graph accessors — concrete node stores aren't unified,
/// so traversal goes through handles.
pub struct CycleCtx<'a> {
    /// `graph->operations`.
    pub operations: &'a dyn Fn() -> Vec<usize>,
    /// inlink/outlink handles of a node.
    pub inlinks: &'a dyn Fn(usize) -> Vec<usize>,
    /// `node->outlinks`.
    pub outlinks: &'a dyn Fn(usize) -> Vec<usize>,
    /// relation endpoints (from,to) + flag.
    pub rel_ends_flag: &'a dyn Fn(usize) -> (usize, usize, u32),
    /// `node->type == OPERATION`.
    pub is_operation_node: &'a dyn Fn(usize) -> bool,
    /// `node->custom_flags` get/set.
    pub custom_flags: &'a mut dyn FnMut(usize, Option<u32>) -> u32,
    /// `relation->flag |= RELATION_FLAG_CYCLIC`.
    pub mark_cyclic: &'a mut dyn FnMut(usize),
    /// `relation->name` for diagnostics.
    pub rel_name: &'a dyn Fn(usize) -> String,
    /// `node->full_identifier()` for diagnostics.
    pub full_identifier: &'a dyn Fn(usize) -> String,
}

/// `set/get_node_visited_state` (verbatim — bits 0-1).
fn set_visited(ctx: &mut CycleCtx, node: usize, state: CyclicCheckVisitedState) {
    let f = (ctx.custom_flags)(node, None);
    (ctx.custom_flags)(node, Some((f & !0x3) | state as u32));
}
fn get_visited(ctx: &mut CycleCtx, node: usize) -> CyclicCheckVisitedState {
    match (ctx.custom_flags)(node, None) & 0x3 {
        1 => CyclicCheckVisitedState::Visited,
        2 => CyclicCheckVisitedState::InStack,
        _ => CyclicCheckVisitedState::NotVisited,
    }
}
/// `set/get_node_num_visited_children` (verbatim — bits 2+).
fn set_num_visited(ctx: &mut CycleCtx, node: usize, n: u32) {
    let f = (ctx.custom_flags)(node, None);
    (ctx.custom_flags)(node, Some((f & 0x3) | (n << 2)));
}
fn get_num_visited(ctx: &mut CycleCtx, node: usize) -> u32 {
    (ctx.custom_flags)(node, None) >> 2
}

/// `schedule_node_to_stack` (verbatim).
fn schedule_node_to_stack(state: &mut CyclesSolverState, ctx: &mut CycleCtx, node: usize) {
    state.traversal_stack.push(StackEntry {
        node,
        from: usize::MAX,
        via_relation: usize::MAX,
    });
    set_visited(ctx, node, CyclicCheckVisitedState::InStack);
}

/// `schedule_leaf_nodes` (verbatim — nodes with no
/// operation inlinks).
fn schedule_leaf_nodes(state: &mut CyclesSolverState, ctx: &mut CycleCtx) {
    for node in (ctx.operations)() {
        let has_inlinks = (ctx.inlinks)(node)
            .iter()
            .any(|rel| (ctx.is_operation_node)((ctx.rel_ends_flag)(*rel).0));
        (ctx.custom_flags)(node, Some(0));
        if !has_inlinks {
            schedule_node_to_stack(state, ctx, node);
        } else {
            set_visited(ctx, node, CyclicCheckVisitedState::NotVisited);
        }
    }
}

/// `schedule_non_checked_node` (verbatim).
fn schedule_non_checked_node(state: &mut CyclesSolverState, ctx: &mut CycleCtx) -> bool {
    for node in (ctx.operations)() {
        if get_visited(ctx, node) == CyclicCheckVisitedState::NotVisited {
            schedule_node_to_stack(state, ctx, node);
            return true;
        }
    }
    false
}

/// `select_relation_to_murder` (verbatim — walks the cycle
/// for a killable relation when the found one is god-mode).
fn select_relation_to_murder(
    state: &CyclesSolverState,
    ctx: &mut CycleCtx,
    relation: usize,
    cycle_start_entry: usize,
) -> usize {
    let (_from, to_node, rel_flag) = (ctx.rel_ends_flag)(relation);
    if check_relation_can_murder(rel_flag) {
        return relation;
    }
    let mut current = cycle_start_entry;
    while state.traversal_stack[current].node != to_node {
        let via = state.traversal_stack[current].via_relation;
        if via != usize::MAX && check_relation_can_murder((ctx.rel_ends_flag)(via).2) {
            return via;
        }
        current = state.traversal_stack[current].from;
    }
    relation
}

/// `solve_cycles` (verbatim — iterative DFS; marks
/// back-edge sacrificial relation `RELATION_FLAG_CYCLIC`).
fn solve_cycles(state: &mut CyclesSolverState, ctx: &mut CycleCtx) {
    while let Some(entry_idx) = state.traversal_stack.len().checked_sub(1) {
        let node = state.traversal_stack[entry_idx].node;
        let mut all_child_traversed = true;
        let outlinks = (ctx.outlinks)(node);
        let num_visited = get_num_visited(ctx, node) as usize;
        for i in num_visited..outlinks.len() {
            let rel = outlinks[i];
            let (_from, to, _flag) = (ctx.rel_ends_flag)(rel);
            if !(ctx.is_operation_node)(to) {
                continue;
            }
            match get_visited(ctx, to) {
                CyclicCheckVisitedState::InStack => {
                    /* Cycle detected — murder a relation. */
                    let sacrificial = select_relation_to_murder(state, ctx, rel, entry_idx);
                    (ctx.mark_cyclic)(sacrificial);
                    state.num_cycles += 1;
                }
                CyclicCheckVisitedState::NotVisited => {
                    state.traversal_stack.push(StackEntry {
                        node: to,
                        from: entry_idx,
                        via_relation: rel,
                    });
                    /* Verbatim: upstream marks the *parent* `node`
                     * here (already in-stack), not `to`. */
                    set_visited(ctx, node, CyclicCheckVisitedState::InStack);
                    all_child_traversed = false;
                    set_num_visited(ctx, node, i as u32);
                    break;
                }
                CyclicCheckVisitedState::Visited => {}
            }
        }
        if all_child_traversed {
            set_visited(ctx, node, CyclicCheckVisitedState::Visited);
            state.traversal_stack.pop();
        }
    }
}

/// `deg_graph_detect_cycles` (verbatim — leaf-driven pass
/// then closed-loop passes).
pub fn deg_graph_detect_cycles(ctx: &mut CycleCtx) -> usize {
    let mut state = CyclesSolverState {
        traversal_stack: Vec::new(),
        num_cycles: 0,
    };
    schedule_leaf_nodes(&mut state, ctx);
    solve_cycles(&mut state, ctx);
    while schedule_non_checked_node(&mut state, ctx) {
        solve_cycles(&mut state, ctx);
    }
    state.num_cycles
}
