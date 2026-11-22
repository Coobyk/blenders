//! Port of `intern/eval/deg_eval_visibility.cc` —
/// visibility-flag flushing.

/// `RELATION_NO_VISIBILITY_CHANGE` verbatim.
pub const RELATION_NO_VISIBILITY_CHANGE: i32 = 1 << 9;
/// `BASE_ENABLED_VIEWPORT`/`BASE_ENABLED_RENDER`
/// (`DNA_layer_types.h`).
pub const BASE_ENABLED_VIEWPORT: i16 = 1 << 0;
/// `BASE_ENABLED_RENDER`.
pub const BASE_ENABLED_RENDER: i16 = 1 << 1;
/// `eModifierMode_Realtime`/`eModifierMode_Render`.
pub const E_MODIFIER_MODE_REALTIME: i32 = 1 << 0;
/// `eModifierMode_Render`.
pub const E_MODIFIER_MODE_RENDER: i32 = 1 << 1;
/// `DEG_NODE_VISITED`.
const DEG_NODE_VISITED: u32 = 1 << 0;

use crate::node::NodeType;
use crate::node_operation::{DEPSOP_FLAG_AFFECTS_VISIBILITY, DEPSOP_FLAG_MUTE};

/// `deg_evaluate_object_node_visibility` (verbatim).
pub fn evaluate_object_node_visibility(
    mode_is_viewport: bool,
    use_visibility_optimization: bool,
    object_base_flag: i16,
    is_enabled_on_eval: &mut bool,
    need_update_nodes_visibility: &mut bool,
) {
    let required_flags = if mode_is_viewport {
        BASE_ENABLED_VIEWPORT
    } else {
        BASE_ENABLED_RENDER
    };
    let is_enabled = !use_visibility_optimization || (object_base_flag & required_flags) != 0;
    if *is_enabled_on_eval != is_enabled {
        *is_enabled_on_eval = is_enabled;
        /* Tag dependency graph for changed visibility. */
        *need_update_nodes_visibility = true;
    }
}

/// `deg_evaluate_object_modifiers_mode_node_visibility`
/// (verbatim — `modifier_nodes` supplies (mode, flag-ref)
/// per `MODIFIER` op of the geometry component).
pub fn evaluate_object_modifiers_mode_node_visibility(
    mode_is_viewport: bool,
    use_visibility_optimization: bool,
    modifier_modes: &[i32],
    modifier_flags: &mut [u32],
    need_update_nodes_visibility: &mut bool,
) {
    let modifier_mode = if mode_is_viewport {
        E_MODIFIER_MODE_REALTIME
    } else {
        E_MODIFIER_MODE_RENDER
    };
    for (i, &mode) in modifier_modes.iter().enumerate() {
        let modifier_enabled = !use_visibility_optimization || (mode & modifier_mode) != 0;
        let mute_flag = if modifier_enabled { 0 } else { DEPSOP_FLAG_MUTE };
        if (modifier_flags[i] & DEPSOP_FLAG_MUTE) != mute_flag {
            modifier_flags[i] &= !DEPSOP_FLAG_MUTE;
            modifier_flags[i] |= mute_flag;
            *need_update_nodes_visibility = true;
        }
    }
}

/// `deg_graph_flush_visibility_flags` (verbatim two-pass:
/// component flags from ID visibility, then reverse topo
/// flush over operation links).
///
/// Index-arena form: `id_is_visible`/`id_is_enabled` per
/// `IDNode`; `comp_type`/`comp_flags` per component
/// (`[possibly, affects]`); `inlinks`/`outlinks` per op node
/// as edge ids; edge = `(from_node, to_node, flag, is_op_from,
/// is_op_to)`; `op_flag` mutable per node.
#[allow(clippy::too_many_arguments)]
pub fn graph_flush_visibility_flags(
    id_nodes: &[usize],
    id_is_visible: &dyn Fn(usize) -> bool,
    id_is_enabled: &dyn Fn(usize) -> bool,
    components_of: &dyn Fn(usize) -> Vec<usize>,
    comp_type: &dyn Fn(usize) -> usize,
    comp_set_flags: &mut dyn FnMut(usize, bool, bool),
    comp_get_flags: &dyn Fn(usize) -> (bool, bool),
    operations: &[usize],
    node_type: &dyn Fn(usize) -> NodeType,
    outlinks: &dyn Fn(usize) -> Vec<usize>,
    inlinks: &dyn Fn(usize) -> Vec<usize>,
    edge_from: &dyn Fn(usize) -> usize,
    edge_to: &dyn Fn(usize) -> usize,
    edge_flag: &dyn Fn(usize) -> i32,
    edge_no_vis_change: &dyn Fn(usize) -> bool,
    op_flag_get: &dyn Fn(usize) -> u32,
    op_flag_or: &mut dyn FnMut(usize, u32),
    comp_owner_of_op: &dyn Fn(usize) -> usize,
    id_owner_of_comp: &dyn Fn(usize) -> usize,
    need_update_nodes_visibility: &mut bool,
) {
    for &id_node in id_nodes {
        let vis = id_is_visible(id_node);
        let enabled = id_is_enabled(id_node);
        for comp in components_of(id_node) {
            let mut possibly = vis;
            let mut affects = vis && enabled;
            /* Visibility component matches the node's
             * visibility. */
            if comp_type(comp) == NodeType::Visibility as usize {
                affects = vis;
            }
            /* Synchronization is always visible. */
            if comp_type(comp) == NodeType::Synchronization as usize {
                possibly = true;
                affects = true;
            }
            comp_set_flags(comp, possibly, affects);
        }
    }
    let mut stack: Vec<usize> = Vec::new();
    let mut custom_flags = vec![0u32; operations.iter().max().map_or(0, |m| m + 1)];
    let mut num_links_pending = vec![0i32; custom_flags.len()];
    for &op_node in operations {
        custom_flags[op_node] = 0;
        num_links_pending[op_node] = 0;
        for rel in outlinks(op_node) {
            if node_type(edge_to(rel)) == NodeType::Operation
                && (edge_flag(rel) & crate::depsgraph::RELATION_FLAG_CYCLIC) == 0
            {
                num_links_pending[op_node] += 1;
            }
        }
        if num_links_pending[op_node] == 0 {
            stack.push(op_node);
            custom_flags[op_node] |= DEG_NODE_VISITED;
        }
    }
    while let Some(op_node) = stack.pop() {
        /* Flush flags to parents. */
        for rel in inlinks(op_node) {
            let from_node = edge_from(rel);
            if node_type(from_node) != NodeType::Operation {
                continue;
            }
            let op_to = edge_to(rel);
            let comp_to = comp_owner_of_op(op_to);
            /* Ignore the synchronization target. */
            if comp_type(comp_to) == NodeType::Synchronization as usize {
                continue;
            }
            let comp_from = comp_owner_of_op(from_node);
            op_flag_or(from_node, op_flag_get(op_to) & DEPSOP_FLAG_AFFECTS_VISIBILITY);
            if edge_no_vis_change(rel) {
                continue;
            }
            let (target_possibly, mut target_affects) = comp_get_flags(comp_to);
            /* Muted target treated as not affecting. */
            if comp_from != comp_to && (op_flag_get(op_to) & DEPSOP_FLAG_MUTE) != 0 {
                target_affects = false;
            }
            if comp_type(comp_from) == NodeType::Visibility as usize {
                let id_node_from = id_owner_of_comp(comp_from);
                for comp in components_of(id_node_from) {
                    let (p, a) = comp_get_flags(comp);
                    comp_set_flags(
                        comp,
                        p | target_possibly,
                        a | target_affects,
                    );
                }
            } else {
                let (p, a) = comp_get_flags(comp_from);
                comp_set_flags(
                    comp_from,
                    p | target_possibly,
                    a | target_affects,
                );
            }
        }
        /* Schedule parent nodes. */
        for rel in inlinks(op_node) {
            let from_node = edge_from(rel);
            if node_type(from_node) != NodeType::Operation {
                continue;
            }
            if (edge_flag(rel) & crate::depsgraph::RELATION_FLAG_CYCLIC) == 0 {
                num_links_pending[from_node] -= 1;
            }
            if num_links_pending[from_node] == 0
                && (custom_flags[from_node] & DEG_NODE_VISITED) == 0
            {
                stack.push(from_node);
                custom_flags[from_node] |= DEG_NODE_VISITED;
            }
        }
    }
    *need_update_nodes_visibility = false;
}

/// `deg_graph_flush_visibility_flags_if_needed` (verbatim).
pub fn graph_flush_visibility_flags_if_needed(
    need_update_nodes_visibility: bool,
    flush: &mut dyn FnMut(),
) {
    if !need_update_nodes_visibility {
        return;
    }
    flush();
}
