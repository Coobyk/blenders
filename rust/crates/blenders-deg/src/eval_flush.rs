//! Port of `intern/eval/deg_eval_flush.cc` —
/// `deg_graph_flush_updates`/`clear_tags`.

use std::collections::VecDeque;

/// `ID_STATE_*` verbatim.
pub const ID_STATE_NONE: u32 = 0;
/// `ID_STATE_MODIFIED`.
pub const ID_STATE_MODIFIED: u32 = 1;
/// `COMPONENT_STATE_*`.
pub const COMPONENT_STATE_NONE: u32 = 0;
/// `COMPONENT_STATE_SCHEDULED`.
pub const COMPONENT_STATE_SCHEDULED: u32 = 1;
/// `COMPONENT_STATE_DONE`.
pub const COMPONENT_STATE_DONE: u32 = 2;

/// `RELATION_FLAG_NO_FLUSH`.
pub const RELATION_FLAG_NO_FLUSH: i32 = 1 << 3;
/// `RELATION_FLAG_FLUSH_USER_EDIT_ONLY`.
pub const RELATION_FLAG_FLUSH_USER_EDIT_ONLY: i32 = 1 << 4;

use crate::node::NodeType;
use crate::node_operation::{OperationCode, DEPSOP_FLAG_FLUSH, DEPSOP_FLAG_NEEDS_UPDATE,
                            DEPSOP_FLAG_USER_MODIFIED};

/// Per-op-node flush state (index arena).
pub struct FlushState<'a> {
    /// `op_node->flag`.
    pub op_flag: &'a mut dyn FnMut(usize) -> u32,
    /// Set `op_node->flag |= f`.
    pub op_flag_or: &'a mut dyn FnMut(usize, u32),
    /// `op_node->scheduled` get.
    pub op_scheduled_get: &'a dyn Fn(usize) -> bool,
    /// `op_node->scheduled` set.
    pub op_scheduled_set: &'a mut dyn FnMut(usize, bool),
    /// `op_node->opcode`.
    pub op_opcode: &'a dyn Fn(usize) -> OperationCode,
    /// `op_node->outlinks` edge ids.
    pub outlinks: &'a dyn Fn(usize) -> Vec<usize>,
    /// Edge `(to_node, flag)`.
    pub edge: &'a dyn Fn(usize) -> (usize, i32),
    /// `op_node->owner` (component) / `component->owner` (id).
    pub op_owner: &'a dyn Fn(usize) -> usize,
    /// `component->owner` (IDNode).
    pub comp_owner: &'a dyn Fn(usize) -> usize,
    /// `component->type`.
    pub comp_type: &'a dyn Fn(usize) -> NodeType,
    /// `component->operations`.
    pub comp_operations: &'a dyn Fn(usize) -> Vec<usize>,
    /// `component->custom_flags` get.
    pub comp_flags_get: &'a dyn Fn(usize) -> u32,
    /// `component->custom_flags` set.
    pub comp_flags_set: &'a mut dyn FnMut(usize, u32),
    /// `component->is_user_modified = true`.
    pub comp_set_user_modified: &'a mut dyn FnMut(usize),
    /// `id_node->custom_flags` get.
    pub id_flags_get: &'a dyn Fn(usize) -> u32,
    /// `id_node->custom_flags` set.
    pub id_flags_set: &'a mut dyn FnMut(usize, u32),
    /// `id_node->is_user_modified = true`.
    pub id_set_user_modified: &'a mut dyn FnMut(usize),
    /// `id_node->find_component(BONE→EVAL_POSE)`.
    pub id_find_component: &'a dyn Fn(usize, NodeType) -> Option<usize>,
    /// `component->get_entry_operation`.
    pub comp_entry_op: &'a mut dyn FnMut(usize) -> Option<usize>,
}

/// `flush_prepare` (verbatim — resets `scheduled` and
/// `custom_flags` on every op/component/id node).
pub fn flush_prepare(
    operations: &[usize],
    id_components: &dyn Fn() -> Vec<(usize, Vec<usize>)>,
    st: &mut FlushState,
) {
    for &node in operations {
        (st.op_scheduled_set)(node, false);
    }
    for (id_node, comps) in id_components() {
        (st.id_flags_set)(id_node, ID_STATE_NONE);
        for comp in comps {
            (st.comp_flags_set)(comp, COMPONENT_STATE_NONE);
        }
    }
}

/// `flush_schedule_entrypoints` (verbatim).
pub fn flush_schedule_entrypoints(
    entry_tags: &[usize],
    queue: &mut VecDeque<usize>,
    st: &mut FlushState,
) {
    for &op_node in entry_tags {
        queue.push_back(op_node);
        (st.op_scheduled_set)(op_node, true);
    }
}

/// `flush_handle_id_node` (verbatim).
pub fn flush_handle_id_node(id_node: usize, st: &mut FlushState) {
    (st.id_flags_set)(id_node, ID_STATE_MODIFIED);
}

/// `flush_handle_component_node` (verbatim).
pub fn flush_handle_component_node(
    id_node: usize,
    comp_node: usize,
    queue: &mut VecDeque<usize>,
    st: &mut FlushState,
) {
    /* We only handle component once. */
    if (st.comp_flags_get)(comp_node) != COMPONENT_STATE_NONE {
        (st.comp_flags_set)(comp_node, COMPONENT_STATE_DONE);
        return;
    }
    (st.comp_flags_set)(comp_node, COMPONENT_STATE_DONE);
    /* Tag all ops as needing update (visibility op of the
     * geometry component excepted). */
    let is_geometry = (st.comp_type)(comp_node) == NodeType::Geometry;
    for op in (st.comp_operations)(comp_node) {
        if is_geometry && (st.op_opcode)(op) == OperationCode::Visibility {
            continue;
        }
        (st.op_flag_or)(op, DEPSOP_FLAG_NEEDS_UPDATE);
    }
    /* A bone change re-runs the pose component. */
    if (st.comp_type)(comp_node) == NodeType::Bone {
        if let Some(pose_comp) = (st.id_find_component)(id_node, NodeType::EvalPose) {
            if (st.comp_flags_get)(pose_comp) == COMPONENT_STATE_NONE {
                if let Some(entry) = (st.comp_entry_op)(pose_comp) {
                    queue.push_front(entry);
                }
                (st.comp_flags_set)(pose_comp, COMPONENT_STATE_SCHEDULED);
            }
        }
    }
}

/// `flush_schedule_children` (verbatim — returns the first
/// newly-scheduled child for the inner loop tail, pushes
/// the rest to the front of the queue).
pub fn flush_schedule_children(
    op_node: usize,
    queue: &mut VecDeque<usize>,
    st: &mut FlushState,
) -> Option<usize> {
    if (st.op_flag)(op_node) & DEPSOP_FLAG_USER_MODIFIED != 0 {
        let comp = (st.op_owner)(op_node);
        (st.comp_set_user_modified)(comp);
        (st.id_set_user_modified)((st.comp_owner)(comp));
    }
    let mut result = None;
    for rel in (st.outlinks)(op_node) {
        let (to_node, rel_flag) = (st.edge)(rel);
        /* Flush is forbidden, completely. */
        if rel_flag & RELATION_FLAG_NO_FLUSH != 0 {
            continue;
        }
        /* Relation only allows flushes on user changes. */
        if (rel_flag & RELATION_FLAG_FLUSH_USER_EDIT_ONLY) != 0
            && (st.op_flag)(op_node) & DEPSOP_FLAG_USER_MODIFIED == 0
        {
            continue;
        }
        /* Always flush flushable flags. */
        (st.op_flag_or)(to_node, (st.op_flag)(op_node) & DEPSOP_FLAG_FLUSH);
        let scheduled = (st.op_scheduled_get)(to_node);
        if scheduled {
            continue;
        }
        if result.is_some() {
            queue.push_front(to_node);
        } else {
            result = Some(to_node);
        }
        (st.op_scheduled_set)(to_node, true);
    }
    result
}

/// `deg_graph_flush_updates` driver (verbatim — the
/// `time_source->flush_update_tag`, `flush_editors_id_update`
/// and `invalidate_tagged_evaluated_data` steps injected).
pub fn deg_graph_flush_updates(
    entry_tags: &[usize],
    operations: &[usize],
    id_components: &dyn Fn() -> Vec<(usize, Vec<usize>)>,
    st: &mut FlushState,
    time_source_flush: &mut dyn FnMut(),
    editors_id_update: &mut dyn FnMut(),
) {
    time_source_flush();
    if entry_tags.is_empty() {
        return;
    }
    flush_prepare(operations, id_components, st);
    let mut queue: VecDeque<usize> = VecDeque::new();
    flush_schedule_entrypoints(entry_tags, &mut queue, st);
    loop {
        let mut op_node = queue.pop_front();
        if op_node.is_none() {
            break;
        }
        while let Some(op) = op_node.take() {
            (st.op_flag_or)(op, DEPSOP_FLAG_NEEDS_UPDATE);
            let comp_node = (st.op_owner)(op);
            let id_node = (st.comp_owner)(comp_node);
            flush_handle_id_node(id_node, st);
            flush_handle_component_node(id_node, comp_node, &mut queue, st);
            op_node = flush_schedule_children(op, &mut queue, st);
        }
    }
    editors_id_update();
    /* `invalidate_tagged_evaluated_data` — no-op without
     * `INVALIDATE_ON_FLUSH`. */
}

/// `deg_graph_clear_tags` (verbatim).
pub fn deg_graph_clear_tags(
    entry_tags: &mut std::collections::HashSet<usize>,
    time_source_tagged: &mut bool,
) {
    entry_tags.clear();
    *time_source_tagged = false;
}
