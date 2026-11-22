//! Port of `intern/depsgraph_query_foreach.cc` — foreach
/// traversal queries over the operation graph.

use crate::node::NodeType;
use crate::node_operation::OperationCode;
use std::collections::{HashSet, VecDeque};

/// `DEG_FOREACH_COMPONENT_IGNORE_TRANSFORM_SOLVERS`.
pub const DEG_FOREACH_COMPONENT_IGNORE_TRANSFORM_SOLVERS: i32 = 1 << 0;
/// `DEG_OB_COMP_ANY` sentinel (all components).
pub const DEG_OB_COMP_ANY: i32 = -1;

/// Graph accessors for foreach traversal (index arena).
pub struct ForeachCtx<'a> {
    /// `op_node->opcode`.
    pub op_opcode: &'a dyn Fn(usize) -> OperationCode,
    /// `op_node->outlinks` → to-node ids.
    pub outlinks: &'a dyn Fn(usize) -> Vec<usize>,
    /// `op_node->inlinks` → (from-node id, is_operation).
    pub inlinks: &'a dyn Fn(usize) -> Vec<(usize, bool)>,
    /// `op_node->owner` (component index).
    pub op_owner: &'a dyn Fn(usize) -> usize,
    /// `component->owner` (id_node index).
    pub comp_owner: &'a dyn Fn(usize) -> usize,
    /// `component->type`.
    pub comp_type: &'a dyn Fn(usize) -> NodeType,
    /// `component->name`.
    pub comp_name: &'a dyn Fn(usize) -> String,
    /// `component->operations`.
    pub comp_operations: &'a dyn Fn(usize) -> Vec<usize>,
    /// `id_node->id_orig`.
    pub id_orig: &'a dyn Fn(usize) -> usize,
    /// `id_node->components`.
    pub id_components: &'a dyn Fn(usize) -> Vec<usize>,
    /// `nodeTypeToObjectComponent` for a comp type.
    pub comp_to_object_component: &'a dyn Fn(NodeType) -> i32,
}

/// `deg_foreach_needs_visit` (verbatim).
pub fn deg_foreach_needs_visit(op_node: usize, flags: i32, ctx: &ForeachCtx) -> bool {
    if flags & DEG_FOREACH_COMPONENT_IGNORE_TRANSFORM_SOLVERS != 0
        && (ctx.op_opcode)(op_node) == OperationCode::RigidbodySim
    {
        return false;
    }
    true
}

/// `deg_foreach_dependent_operation(graph, id_node, ...)`
/// (verbatim — the scheduling/queue walk; callback returns
/// void here as in the C overload used by ID traversals).
pub fn deg_foreach_dependent_operation(
    target_id_node: Option<usize>,
    source_component_type: i32,
    flags: i32,
    ctx: &ForeachCtx,
    callback: &mut dyn FnMut(usize),
) {
    let Some(target_id_node) = target_id_node else {
        return;
    };
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut scheduled: HashSet<usize> = HashSet::new();
    for comp_node in (ctx.id_components)(target_id_node) {
        let ty = (ctx.comp_type)(comp_node);
        if ty == NodeType::Visibility {
            continue;
        }
        if source_component_type != DEG_OB_COMP_ANY
            && (ctx.comp_to_object_component)(ty) != source_component_type
        {
            continue;
        }
        for op_node in (ctx.comp_operations)(comp_node) {
            if !deg_foreach_needs_visit(op_node, flags, ctx) {
                continue;
            }
            queue.push_back(op_node);
            scheduled.insert(op_node);
        }
    }
    while let Some(mut op_node) = queue.pop_front() {
        loop {
            callback(op_node);
            let out = (ctx.outlinks)(op_node);
            if out.len() == 1 {
                let to_node = out[0];
                if !scheduled.contains(&to_node)
                    && deg_foreach_needs_visit(to_node, flags, ctx)
                {
                    scheduled.insert(to_node);
                    op_node = to_node;
                    continue;
                }
                break;
            }
            for to_node in out {
                if !scheduled.contains(&to_node)
                    && deg_foreach_needs_visit(to_node, flags, ctx)
                {
                    queue.push_front(to_node);
                    scheduled.insert(to_node);
                }
            }
            break;
        }
    }
}

/// `deg_foreach_dependent_operation(component, ...)` overload
/// (verbatim — early-out when callback returns false).
pub fn deg_foreach_dependent_operation_from_component(
    start_component: usize,
    ctx: &ForeachCtx,
    callback: &mut dyn FnMut(usize) -> bool,
) {
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut was_queued: HashSet<usize> = HashSet::new();
    for op_node in (ctx.comp_operations)(start_component) {
        queue.push_back(op_node);
        was_queued.insert(op_node);
    }
    while let Some(op_node) = queue.pop_front() {
        if !callback(op_node) {
            continue;
        }
        for to_node in (ctx.outlinks)(op_node) {
            if !was_queued.contains(&to_node) {
                queue.push_back(to_node);
                was_queued.insert(to_node);
            }
        }
    }
}

/// `deg_foreach_dependent_ID_component` (verbatim —
/// folded `deg_foreach_dependent_component_callback` in).
pub fn deg_foreach_dependent_id_component(
    target_id_node: Option<usize>,
    source_component_type: i32,
    flags: i32,
    ctx: &ForeachCtx,
    callback: &mut dyn FnMut(usize, i32),
) {
    let mut visited: HashSet<usize> = HashSet::new();
    deg_foreach_dependent_operation(
        target_id_node,
        source_component_type,
        flags,
        ctx,
        &mut |op_node| {
            let comp_node = (ctx.op_owner)(op_node);
            let id_node = (ctx.comp_owner)(comp_node);
            if Some(id_node) != target_id_node && !visited.contains(&comp_node) {
                callback(
                    (ctx.id_orig)(id_node),
                    (ctx.comp_to_object_component)((ctx.comp_type)(comp_node)),
                );
                visited.insert(comp_node);
            }
        },
    );
}

/// `deg_foreach_dependent_component` (verbatim — starts at
/// a named component, reports (id_orig, object_component, name)).
pub fn deg_foreach_dependent_component(
    id_node: Option<usize>,
    start_component: Option<usize>,
    ctx: &ForeachCtx,
    callback: &mut dyn FnMut(usize, i32, &str) -> bool,
) {
    if id_node.is_none() {
        return;
    }
    let Some(start_component) = start_component else {
        return;
    };
    deg_foreach_dependent_operation_from_component(start_component, ctx, &mut |op_node| {
        let comp_node = (ctx.op_owner)(op_node);
        let ty = (ctx.comp_type)(comp_node);
        if ty == NodeType::Visibility {
            return true;
        }
        let id_node = (ctx.comp_owner)(comp_node);
        callback(
            (ctx.id_orig)(id_node),
            (ctx.comp_to_object_component)(ty),
            &(ctx.comp_name)(comp_node),
        )
    });
}

/// `deg_foreach_dependent_ID` (verbatim).
pub fn deg_foreach_dependent_id(
    target_id_node: Option<usize>,
    ctx: &ForeachCtx,
    callback: &mut dyn FnMut(usize),
) {
    let mut visited: HashSet<usize> = HashSet::new();
    deg_foreach_dependent_operation(target_id_node, DEG_OB_COMP_ANY, 0, ctx, &mut |op_node| {
        let comp_node = (ctx.op_owner)(op_node);
        let id_node = (ctx.comp_owner)(comp_node);
        if Some(id_node) != target_id_node && !visited.contains(&id_node) {
            callback((ctx.id_orig)(id_node));
            visited.insert(id_node);
        }
    });
}

/// `deg_foreach_ancestor_ID` (verbatim — incoming-link walk).
pub fn deg_foreach_ancestor_id(
    target_id_node: Option<usize>,
    ctx: &ForeachCtx,
    callback: &mut dyn FnMut(usize),
) {
    let Some(target_id_node) = target_id_node else {
        return;
    };
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut scheduled: HashSet<usize> = HashSet::new();
    for comp_node in (ctx.id_components)(target_id_node) {
        for op_node in (ctx.comp_operations)(comp_node) {
            queue.push_back(op_node);
            scheduled.insert(op_node);
        }
    }
    let mut visited: HashSet<usize> = HashSet::new();
    visited.insert(target_id_node);
    while let Some(mut op_node) = queue.pop_front() {
        loop {
            let comp_node = (ctx.op_owner)(op_node);
            let id_node = (ctx.comp_owner)(comp_node);
            if !visited.contains(&id_node) {
                callback((ctx.id_orig)(id_node));
                visited.insert(id_node);
            }
            let inl = (ctx.inlinks)(op_node);
            if inl.len() == 1 {
                let (from, is_op) = inl[0];
                if is_op && scheduled.insert(from) {
                    op_node = from;
                    continue;
                }
                break;
            }
            for (from, is_op) in inl {
                if is_op && scheduled.insert(from) {
                    queue.push_front(from);
                }
            }
            break;
        }
    }
}

/// `deg_foreach_id` (verbatim).
pub fn deg_foreach_id(id_origs: &[usize], callback: &mut dyn FnMut(usize)) {
    for &id in id_origs {
        callback(id);
    }
}

/// `deg_scene_component_depends_on_time` (verbatim —
/// backward BFS from component ops to the time-source node).
pub fn deg_scene_component_depends_on_time(
    time_source_node: Option<usize>,
    component_node: usize,
    ctx: &ForeachCtx,
) -> bool {
    let Some(time_source_node) = time_source_node else {
        return false;
    };
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut scheduled: HashSet<usize> = HashSet::new();
    queue.push_back(component_node);
    scheduled.insert(component_node);
    for op in (ctx.comp_operations)(component_node) {
        queue.push_back(op);
        scheduled.insert(op);
    }
    while let Some(cur) = queue.pop_front() {
        for (from_node, _) in (ctx.inlinks)(cur) {
            if from_node == time_source_node {
                return true;
            }
            if !scheduled.contains(&from_node) {
                queue.push_back(from_node);
                scheduled.insert(from_node);
            }
        }
    }
    false
}
