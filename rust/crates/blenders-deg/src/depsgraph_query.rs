//! Port of `intern/depsgraph_query.cc` — DEG query API.

use crate::depsgraph::{Depsgraph, ID_TAG_COPIED_ON_EVAL};
use crate::node::NodeType;
use crate::node_id::IdNode;
use crate::node_operation::DEPSOP_FLAG_NEEDS_UPDATE;
use crate::node_operation::OperationCode;
use blenders_dna::id::gs;

/// `ID_TAG_LOCALIZED` (`DNA_ID.h`).
pub const ID_TAG_LOCALIZED: u32 = 1 << 11;

/// `deg::get_original_id` (verbatim — `id->orig_id`
/// resolved via the `orig_id` hook).
pub fn get_original_id(id: usize, orig_id: &dyn Fn(usize) -> usize) -> usize {
    let orig = orig_id(id);
    if orig == 0 {
        return id;
    }
    orig
}

/// `deg::get_evaluated_id` (verbatim).
pub fn get_evaluated_id<'a>(
    graph: &'a Depsgraph,
    id: usize,
    orig_id: &dyn Fn(usize) -> usize,
) -> usize {
    if id == 0 {
        return 0;
    }
    match graph.find_id_node(get_original_id(id, orig_id)) {
        Some(idx) => graph.id_nodes[idx].id_cow,
        None => id,
    }
}

/// `DEG_get_input_scene` (verbatim).
pub fn deg_get_input_scene(graph: &Depsgraph) -> usize {
    graph.scene
}

/// `DEG_get_input_view_layer` (verbatim).
pub fn deg_get_input_view_layer(graph: &Depsgraph) -> usize {
    graph.view_layer
}

/// `DEG_get_bmain` (verbatim).
pub fn deg_get_bmain(graph: &Depsgraph) -> usize {
    graph.bmain
}

/// `DEG_get_ctime` (verbatim).
pub fn deg_get_ctime(graph: &Depsgraph) -> f32 {
    graph.ctime
}

/// `DEG_get_mode` (verbatim).
pub fn deg_get_mode(graph: &Depsgraph) -> i32 {
    graph.mode
}

/// `DEG_id_type_updated` (verbatim — `idcode_to_index` as arg).
pub fn deg_id_type_updated(graph: &Depsgraph, id_type_index: usize) -> bool {
    graph.id_type_updated.get(id_type_index).copied().unwrap_or(0) != 0
}

/// `DEG_id_type_any_updated` (verbatim).
pub fn deg_id_type_any_updated(graph: &Depsgraph) -> bool {
    graph.id_type_updated.iter().any(|&b| b != 0)
}

/// `DEG_id_is_user_modified` (verbatim).
pub fn deg_id_is_user_modified(
    graph: &Depsgraph,
    id: usize,
    orig_id: &dyn Fn(usize) -> usize,
) -> bool {
    match graph.find_id_node(get_original_id(id, orig_id)) {
        Some(idx) => graph.id_nodes[idx].is_user_modified,
        None => false,
    }
}

/// `DEG_scene_component_is_user_modified` (verbatim —
/// `nodeTypeFromSceneComponent` on `DEG_SCENE_COMP_*`).
pub fn deg_scene_component_is_user_modified(
    graph: &Depsgraph,
    scene_id: usize,
    component_type: i32,
    orig_id: &dyn Fn(usize) -> usize,
    comp_is_user_modified: &dyn Fn(usize, usize) -> bool,
) -> bool {
    let Some(id_idx) = graph.find_id_node(get_original_id(scene_id, orig_id)) else {
        return false;
    };
    let node_type = crate::node::node_type_from_scene_component(component_type);
    match graph.id_nodes[id_idx].find_component(node_type as usize, "") {
        Some(comp_idx) => comp_is_user_modified(id_idx, comp_idx),
        None => false,
    }
}

/// `DEG_id_type_any_exists` (verbatim).
pub fn deg_id_type_any_exists(graph: &Depsgraph, id_type_index: usize) -> bool {
    graph.id_type_exist.get(id_type_index).copied().unwrap_or(0) != 0
}

/// `DEG_get_eval_flags_for_id` (verbatim).
pub fn deg_get_eval_flags_for_id(
    graph: &Depsgraph,
    id: usize,
    orig_id: &dyn Fn(usize) -> usize,
) -> u32 {
    match graph.find_id_node(get_original_id(id, orig_id)) {
        Some(idx) => graph.id_nodes[idx].eval_flags,
        None => 0,
    }
}

/// `DEG_get_customdata_mask_for_object` (verbatim).
pub fn deg_get_customdata_mask_for_object(
    graph: &Depsgraph,
    ob_id: usize,
    orig_id: &dyn Fn(usize) -> usize,
) -> Option<(u64, u64, u64, u64, u64)> {
    let idx = graph.find_id_node(get_original_id(ob_id, orig_id))?;
    let m = &graph.id_nodes[idx].customdata_masks;
    Some((m.vert_mask, m.edge_mask, m.face_mask, m.loop_mask, m.poly_mask))
}

/// `DEG_get_evaluated_scene` (verbatim).
pub fn deg_get_evaluated_scene(graph: &Depsgraph) -> usize {
    graph.scene_cow
}

/// `DEG_get_evaluated_id` (verbatim).
pub fn deg_get_evaluated_id(
    graph: &Depsgraph,
    id: usize,
    orig_id: &dyn Fn(usize) -> usize,
) -> usize {
    get_evaluated_id(graph, id, orig_id)
}

/// `DEG_get_original_id` (verbatim).
pub fn deg_get_original_id(id: usize, orig_id: &dyn Fn(usize) -> usize) -> usize {
    get_original_id(id, orig_id)
}

/// `DEG_is_original_id` (verbatim — `id->tag` via hook).
pub fn deg_is_original_id(id_tag: u32) -> bool {
    if id_tag & (ID_TAG_COPIED_ON_EVAL | ID_TAG_LOCALIZED) != 0 {
        return false;
    }
    true
}

/// `DEG_is_evaluated_id` (verbatim).
pub fn deg_is_evaluated_id(id_tag: u32) -> bool {
    !deg_is_original_id(id_tag)
}

/// `DEG_is_fully_evaluated` (verbatim).
pub fn deg_is_fully_evaluated(graph: &Depsgraph) -> bool {
    if graph.need_update_relations {
        return false;
    }
    if !graph.entry_tags.is_empty() {
        return false;
    }
    true
}

/// `DEG_relations_need_update` (verbatim).
pub fn deg_relations_need_update(graph: &Depsgraph) -> bool {
    graph.need_update_relations
}

/// `DEG_id_is_fully_evaluated` (verbatim — per-op flag
/// read injected).
pub fn deg_id_is_fully_evaluated(
    graph: &Depsgraph,
    id_eval: usize,
    orig_id: &dyn Fn(usize) -> usize,
    comp_operations: &dyn Fn(usize) -> Vec<usize>,
    op_flag: &dyn Fn(usize) -> u32,
) -> bool {
    let id_orig = get_original_id(id_eval, orig_id);
    let Some(idx) = graph.find_id_node(id_orig) else {
        return false;
    };
    for &comp in graph.id_nodes[idx].components.values() {
        for op in comp_operations(comp) {
            if op_flag(op) & DEPSOP_FLAG_NEEDS_UPDATE != 0 {
                return false;
            }
        }
    }
    true
}

/// `operation_needs_update` (verbatim).
pub fn operation_needs_update(
    graph: Option<&Depsgraph>,
    id: usize,
    component_type: NodeType,
    opcode: OperationCode,
    orig_id: &dyn Fn(usize) -> usize,
    comp_find_operation: &dyn Fn(usize, OperationCode) -> Option<usize>,
    op_flag: &dyn Fn(usize) -> u32,
) -> bool {
    let Some(graph) = graph else { return false };
    let id_orig = get_original_id(id, orig_id);
    if id_orig == 0 {
        return false;
    }
    let Some(idx) = graph.find_id_node(id_orig) else {
        return false;
    };
    let id_node: &IdNode = &graph.id_nodes[idx];
    let Some(comp_idx) = id_node.find_component(component_type as usize, "")
    else {
        return false;
    };
    let Some(op_idx) = comp_find_operation(comp_idx, opcode) else {
        return false;
    };
    op_flag(op_idx) & DEPSOP_FLAG_NEEDS_UPDATE != 0
}
