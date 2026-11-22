//! Port of `intern/depsgraph_tag.cc` —
/// ID → component/operation tag mapping + DEG tag API.

use crate::node::NodeType;
use crate::node_operation::OperationCode;
use blenders_dna::id::*;

/// `eObjectType` verbatim (`DNA_object_types.h`) — local
/// copies so deg does not depend on bke.
pub const OB_EMPTY: i16 = 0;
/// `OB_CURVES_LEGACY`.
pub const OB_CURVES_LEGACY: i16 = 2;
/// `OB_SURF`.
pub const OB_SURF: i16 = 3;
/// `OB_FONT`.
pub const OB_FONT: i16 = 4;
/// `OB_MBALL`.
pub const OB_MBALL: i16 = 5;
/// `OB_MESH`.
pub const OB_MESH: i16 = 1;
/// `OB_CAMERA`.
pub const OB_CAMERA: i16 = 11;
/// `OB_LATTICE`.
pub const OB_LATTICE: i16 = 22;
/// `OB_ARMATURE`.
pub const OB_ARMATURE: i16 = 25;
/// `OB_CURVES`.
pub const OB_CURVES: i16 = 27;
/// `OB_POINTCLOUD`.
pub const OB_POINTCLOUD: i16 = 28;
/// `OB_VOLUME`.
pub const OB_VOLUME: i16 = 29;
/// `OB_GREASE_PENCIL`.
pub const OB_GREASE_PENCIL: i16 = 30;

/// `is_selectable_data_id_type` (verbatim).
pub fn is_selectable_data_id_type(id_type: u16) -> bool {
    matches!(
        id_type,
        x if x == gs(b"ME") || x == gs(b"CU") || x == gs(b"MB") || x == gs(b"LT")
            || x == gs(b"GD") || x == gs(b"CV") || x == gs(b"PT") || x == gs(b"VO")
    )
}

/// `geometry_tag_to_component` (verbatim — object-type
/// switch injected via `ob_type`/`has_modifiers` since the
/// object struct is usize-keyed here).
pub fn geometry_tag_to_component(
    id_type: u16,
    ob_type: i16,
    has_modifiers: bool,
) -> NodeType {
    if id_type == gs(b"OB") {
        match ob_type {
            // Empties with modifiers have evaluated geometry.
            OB_EMPTY if has_modifiers => {
                return NodeType::Geometry;
            }
            OB_MESH
            | OB_SURF
            | OB_FONT
            | OB_LATTICE
            | OB_MBALL
            | OB_CURVES
            | OB_POINTCLOUD
            | OB_VOLUME
            | OB_GREASE_PENCIL
            | OB_CURVES_LEGACY => return NodeType::Geometry,
            OB_ARMATURE => return NodeType::EvalPose,
            _ => {}
        }
    } else if id_type == gs(b"ME")
        || id_type == gs(b"CU") || id_type == gs(b"LT") || id_type == gs(b"MB")
        || id_type == gs(b"CV") || id_type == gs(b"PT") || id_type == gs(b"VO")
        || id_type == gs(b"GR") || id_type == gs(b"GD") || id_type == gs(b"GP")
    {
        return NodeType::Geometry;
    } else if id_type == gs(b"PA") {
        return NodeType::Undefined;
    } else if id_type == gs(b"LP") || id_type == gs(b"PA") || id_type == gs(b"MS") {
        return NodeType::Parameters;
    }
    NodeType::Undefined
}

/// `depsgraph_select_tag_to_component_opcode` (verbatim).
pub fn depsgraph_select_tag_to_component_opcode(id_type: u16) -> (NodeType, OperationCode) {
    if id_type == gs(b"SC") {
        (NodeType::LayerCollections, OperationCode::ViewLayerEval)
    } else if id_type == gs(b"OB") {
        (NodeType::ObjectFromLayer, OperationCode::ObjectFromLayerEntry)
    } else if is_selectable_data_id_type(id_type) {
        (NodeType::BatchCache, OperationCode::GeometrySelectUpdate)
    } else {
        (NodeType::CopyOnEval, OperationCode::CopyOnEval)
    }
}

/// `depsgraph_base_flags_tag_to_component_opcode` (verbatim).
pub fn depsgraph_base_flags_tag_to_component_opcode(
    id_type: u16,
) -> Option<(NodeType, OperationCode)> {
    if id_type == gs(b"SC") {
        Some((NodeType::LayerCollections, OperationCode::ViewLayerEval))
    } else if id_type == gs(b"OB") {
        Some((NodeType::ObjectFromLayer, OperationCode::ObjectBaseFlags))
    } else {
        None
    }
}

/// `psysTagToOperationCode` (verbatim).
pub fn psys_tag_to_operation_code(tag: u32) -> OperationCode {
    if tag == ID_RECALC_PSYS_RESET {
        return OperationCode::ParticleSettingsReset;
    }
    OperationCode::Operation
}

/// `depsgraph_tag_to_component_opcode` (verbatim).
pub fn depsgraph_tag_to_component_opcode(
    id_type: u16,
    ob_type: i16,
    has_modifiers: bool,
    tag: u32,
) -> (NodeType, OperationCode) {
    let mut component_type = NodeType::Undefined;
    let mut operation_code = OperationCode::Operation;
    if tag == 0 {
        return (NodeType::IdRef, OperationCode::Operation);
    }
    match tag {
        ID_RECALC_TRANSFORM => component_type = NodeType::Transform,
        ID_RECALC_GEOMETRY => {
            let r = geometry_tag_to_component(id_type, ob_type, has_modifiers);
            if r != NodeType::Undefined {
                component_type = r;
            }
        }
        ID_RECALC_ANIMATION => component_type = NodeType::Animation,
        ID_RECALC_PSYS_REDO | ID_RECALC_PSYS_RESET | ID_RECALC_PSYS_CHILD
        | ID_RECALC_PSYS_PHYS => {
            if id_type == gs(b"PA") {
                component_type = NodeType::ParticleSettings;
                operation_code = psys_tag_to_operation_code(tag);
            } else {
                component_type = NodeType::ParticleSystem;
            }
        }
        ID_RECALC_SYNC_TO_EVAL => component_type = NodeType::CopyOnEval,
        ID_RECALC_SHADING => component_type = NodeType::Shading,
        ID_RECALC_SELECT => {
            let (c, o) = depsgraph_select_tag_to_component_opcode(id_type);
            component_type = c;
            operation_code = o;
        }
        ID_RECALC_BASE_FLAGS => {
            if let Some((c, o)) = depsgraph_base_flags_tag_to_component_opcode(id_type) {
                component_type = c;
                operation_code = o;
            }
        }
        ID_RECALC_POINT_CACHE => component_type = NodeType::PointCache,
        ID_RECALC_EDITORS => {}
        ID_RECALC_SEQUENCER_STRIPS => component_type = NodeType::Sequencer,
        ID_RECALC_FRAME_CHANGE | ID_RECALC_AUDIO_FPS | ID_RECALC_AUDIO_VOLUME
        | ID_RECALC_AUDIO_MUTE | ID_RECALC_AUDIO_LISTENER | ID_RECALC_AUDIO => {
            component_type = NodeType::Audio
        }
        ID_RECALC_PARAMETERS | ID_RECALC_SOURCE => component_type = NodeType::Parameters,
        ID_RECALC_TAG_FOR_UNDO => {}
        ID_RECALC_NTREE_OUTPUT => {
            component_type = NodeType::NtreeOutput;
            operation_code = OperationCode::NtreeOutput;
        }
        ID_RECALC_HIERARCHY => {
            component_type = NodeType::Hierarchy;
            operation_code = OperationCode::Hierarchy;
        }
        ID_RECALC_COMPOSITOR => {
            component_type = NodeType::Compositor;
            operation_code = OperationCode::CompositorEval;
        }
        /* GEOMETRY_ALL_MODES/ALL/PSYS_ALL: `BLI_assert_msg(0)`.
         * PROVISION_*: silently ignored. */
        _ => {}
    }
    (component_type, operation_code)
}

/// `deg_recalc_flags_for_legacy_zero` (verbatim).
pub fn deg_recalc_flags_for_legacy_zero() -> u32 {
    let provision_all = ID_RECALC_PROVISION_28 | ID_RECALC_PROVISION_29
        | ID_RECALC_PROVISION_30 | ID_RECALC_PROVISION_31;
    ID_RECALC_ALL
        & !(ID_RECALC_PSYS_ALL | ID_RECALC_ANIMATION | ID_RECALC_FRAME_CHANGE
            | ID_RECALC_SOURCE | ID_RECALC_EDITORS | provision_all)
}

/// `deg_recalc_flags_effective` (verbatim — `graph->is_active`
/// supplied as a bool since the graph is caller-managed).
pub fn deg_recalc_flags_effective(graph_is_active: Option<bool>, flags: u32) -> u32 {
    if let Some(active) = graph_is_active {
        if !active {
            return 0;
        }
    }
    if flags == 0 {
        return deg_recalc_flags_for_legacy_zero();
    }
    flags
}

/// `DEG_update_tag_as_string` (verbatim).
pub fn deg_update_tag_as_string(flag: u32) -> Option<&'static str> {
    Some(match flag {
        ID_RECALC_TRANSFORM => "TRANSFORM",
        ID_RECALC_GEOMETRY => "GEOMETRY",
        ID_RECALC_GEOMETRY_ALL_MODES => "GEOMETRY_ALL_MODES",
        ID_RECALC_ANIMATION => "ANIMATION",
        ID_RECALC_PSYS_REDO => "PSYS_REDO",
        ID_RECALC_PSYS_RESET => "PSYS_RESET",
        ID_RECALC_PSYS_CHILD => "PSYS_CHILD",
        ID_RECALC_PSYS_PHYS => "PSYS_PHYS",
        ID_RECALC_PSYS_ALL => "PSYS_ALL",
        ID_RECALC_SYNC_TO_EVAL => "COPY_ON_EVAL",
        ID_RECALC_SHADING => "SHADING",
        ID_RECALC_SELECT => "SELECT",
        ID_RECALC_BASE_FLAGS => "BASE_FLAGS",
        ID_RECALC_POINT_CACHE => "POINT_CACHE",
        ID_RECALC_EDITORS => "EDITORS",
        ID_RECALC_SEQUENCER_STRIPS => "SEQUENCER_STRIPS",
        ID_RECALC_FRAME_CHANGE => "FRAME_CHANGE",
        ID_RECALC_AUDIO_FPS => "AUDIO_FPS",
        ID_RECALC_AUDIO_VOLUME => "AUDIO_VOLUME",
        ID_RECALC_AUDIO_MUTE => "AUDIO_MUTE",
        ID_RECALC_AUDIO_LISTENER => "AUDIO_LISTENER",
        ID_RECALC_AUDIO => "AUDIO",
        ID_RECALC_PARAMETERS => "PARAMETERS",
        ID_RECALC_SOURCE => "SOURCE",
        ID_RECALC_ALL => "ALL",
        ID_RECALC_TAG_FOR_UNDO => "TAG_FOR_UNDO",
        ID_RECALC_NTREE_OUTPUT => "ID_RECALC_NTREE_OUTPUT",
        ID_RECALC_HIERARCHY => "ID_RECALC_HIERARCHY",
        ID_RECALC_COMPOSITOR => "ID_RECALC_COMPOSITOR",
        ID_RECALC_PROVISION_28 | ID_RECALC_PROVISION_29 | ID_RECALC_PROVISION_30
        | ID_RECALC_PROVISION_31 => return None,
        _ => return None,
    })
}

/// `DEG_stringify_recalc_flags` (from `depsgraph.cc` — joins
/// the per-bit names).
pub fn deg_stringify_recalc_flags(flags: u32) -> String {
    let mut parts = Vec::new();
    let mut f = flags;
    while f != 0 {
        let bit = 1u32 << f.trailing_zeros();
        f &= !bit;
        if let Some(name) = deg_update_tag_as_string(bit) {
            parts.push(name);
        }
    }
    parts.join(" | ")
}

/// `update_source_as_string` (verbatim).
pub fn update_source_as_string(source: i32) -> &'static str {
    match source {
        crate::node_id::DEG_UPDATE_SOURCE_TIME => "TIME",
        crate::node_id::DEG_UPDATE_SOURCE_USER_EDIT => "USER_EDIT",
        crate::node_id::DEG_UPDATE_SOURCE_RELATIONS => "RELATIONS",
        crate::node_id::DEG_UPDATE_SOURCE_VISIBILITY => "VISIBILITY",
        crate::node_id::DEG_UPDATE_SOURCE_SIDE_EFFECT_REQUEST => "SIDE_EFFECT_REQUEST",
        _ => "UNKNOWN",
    }
}

/// Tag-driver state: graph lookups + ID-side bookkeeping
/// injected so the verbatim control flow stays here.
pub struct IdTagHooks<'a> {
    /// `id_type` (GS of `id->name`).
    pub id_type: &'a dyn Fn(usize) -> u16,
    /// Object type for `ID_OB` ids (else 0).
    pub ob_type: &'a dyn Fn(usize) -> i16,
    /// `!BLI_listbase_is_empty(&object->modifiers)` for `ID_OB`.
    pub ob_has_modifiers: &'a dyn Fn(usize) -> bool,
    /// `graph->find_id_node(id)` → id_node index.
    pub find_id_node: &'a dyn Fn(usize) -> Option<usize>,
    /// `id_node->find_component(type)` → comp index.
    pub find_component: &'a dyn Fn(usize, NodeType) -> Option<usize>,
    /// `component->find_operation(opcode)` → op index.
    pub find_operation: &'a dyn Fn(usize, OperationCode) -> Option<usize>,
    /// `component->tag_update` / `id_node->tag_update` /
    /// `op->tag_update` — one entry-tag hook per level.
    pub tag_component_update: &'a mut dyn FnMut(usize, i32),
    /// `id_node->tag_update`.
    pub tag_id_node_update: &'a mut dyn FnMut(usize, i32),
    /// `op->tag_update`.
    pub tag_operation_update: &'a mut dyn FnMut(usize, i32),
    /// `id_node->id_cow->recalc |= flags`.
    pub id_cow_recalc_or: &'a mut dyn FnMut(usize, u32),
    /// `id_node->id_cow->recalc` read.
    pub id_cow_recalc_get: &'a dyn Fn(usize) -> u32,
    /// `id_node->is_cow_explicitly_tagged = true`.
    pub set_cow_explicitly_tagged: &'a mut dyn FnMut(usize),
    /// `component->need_tag_cow_before_update(recalc)`.
    pub need_tag_cow: &'a dyn Fn(usize, u32) -> bool,
    /// `id->recalc |= flags` on the original ID.
    pub id_orig_recalc_or: &'a mut dyn FnMut(usize, u32),
    /// `id->recalc_after_undo_push |= flags`.
    pub id_recalc_after_undo_or: &'a mut dyn FnMut(usize, u32),
    /// `factory->id_recalc_tag()` for a component type.
    pub factory_id_recalc_tag: &'a dyn Fn(NodeType) -> u32,
    /// `BKE_idtype_idcode_to_index` + `id_type_updated[i]=1`.
    pub id_type_tag: &'a mut dyn FnMut(u16),
    /// `deg_eval_copy_is_needed(id_type)`.
    pub eval_copy_is_needed: &'a dyn Fn(u16) -> bool,
    /// `deg_editors_id_update(update_ctx, id)` for EDITORS.
    pub editors_id_update: &'a mut dyn FnMut(usize),
    /// `BKE_animdata_from_id(id) != nullptr`.
    pub animdata_from_id: &'a dyn Fn(usize) -> bool,
    /// Legacy-compat extra tag on data/key IDs
    /// (`graph_id_tag_update` recursion).
    pub legacy_data_ids: &'a dyn Fn(usize) -> Vec<usize>,
    /// `id_node->components` (for tag_zero iteration).
    pub id_components: &'a dyn Fn(usize) -> Vec<usize>,
    /// `component->type`.
    pub comp_type: &'a dyn Fn(usize) -> NodeType,
    /// `deg_eval_copy_is_expanded(id_cow)`.
    pub eval_copy_is_expanded: &'a dyn Fn(usize) -> bool,
    /// `id_node->visible_components_mask`.
    pub visible_components_mask: &'a dyn Fn(usize) -> u32,
    /// `id_node->previously_visible_components_mask` get/set.
    pub prev_visible_mask: &'a mut dyn FnMut(usize, Option<u32>) -> u32,
    /// `id->recalc &= ~ID_RECALC_ALL` (+ embedded ntree).
    pub clear_id_recalc: &'a mut dyn FnMut(usize),
    /// `id_node->id_cow_recalc_backup |= recalc`.
    pub cow_recalc_backup_or: &'a mut dyn FnMut(usize, u32),
    /// `id_node->id_cow_recalc_backup` read/clear.
    pub cow_recalc_backup_take: &'a mut dyn FnMut(usize) -> u32,
    /// `id_node->is_user_modified`/`is_cow_explicitly_tagged`
    ////`component->is_user_modified` reset.
    pub clear_user_modified: &'a mut dyn FnMut(usize),
    /// `node_tree_from_id` for `id_tag_update_ntree_special`
    /// — returns extra IDs to tag (embedded trees).
    pub ntree_extra_ids: &'a dyn Fn(usize) -> Vec<usize>,
    /// `set_id_update_count` (image runtime counter).
    pub set_id_update_count: &'a mut dyn FnMut(usize),
    /// `BKE_lib_override_id_tag_on_deg_tag_from_user`.
    pub lib_override_tag_on_user: &'a mut dyn FnMut(usize),
    /// `deg_editors_scene_update(update_ctx, updated)`.
    pub editors_scene_update: &'a mut dyn FnMut(bool),
    /// Whether graph is active / evaluating.
    pub graph_is_active: bool,
    /// `graph->is_evaluating`.
    pub graph_is_evaluating: bool,
}

/// `depsgraph_id_tag_copy_on_write` (verbatim).
pub fn depsgraph_id_tag_copy_on_write(
    id_node: usize,
    id_type: u16,
    update_source: i32,
    h: &mut IdTagHooks,
) {
    match (h.find_component)(id_node, NodeType::CopyOnEval) {
        Some(cow_comp) => (h.tag_component_update)(cow_comp, update_source),
        None => {
            debug_assert!(!(h.eval_copy_is_needed)(id_type));
        }
    }
}

/// `depsgraph_tag_component` (verbatim).
pub fn depsgraph_tag_component(
    id_node: usize,
    id_type: u16,
    component_type: NodeType,
    operation_code: OperationCode,
    update_source: i32,
    h: &mut IdTagHooks,
) {
    let component_node = (h.find_component)(id_node, component_type);
    /* Animation component might not exist yet — tag COW
     * explicitly. */
    let Some(component_node) = component_node else {
        if component_type == NodeType::Animation {
            (h.set_cow_explicitly_tagged)(id_node);
            depsgraph_id_tag_copy_on_write(id_node, id_type, update_source, h);
        }
        return;
    };
    if operation_code == OperationCode::Operation {
        (h.tag_component_update)(component_node, update_source);
    } else if let Some(op) = (h.find_operation)(component_node, operation_code) {
        (h.tag_operation_update)(op, update_source);
    }
    /* If component depends on copy-on-evaluation, tag it too. */
    if (h.need_tag_cow)(component_node, (h.id_cow_recalc_get)(id_node)) {
        depsgraph_id_tag_copy_on_write(id_node, id_type, update_source, h);
    }
    if component_type == NodeType::CopyOnEval {
        (h.set_cow_explicitly_tagged)(id_node);
    }
}

/// `deg_graph_id_tag_legacy_compat` (verbatim — data/key id
/// lookups injected via `legacy_data_ids`).
pub fn deg_graph_id_tag_legacy_compat(
    id: usize,
    tag: u32,
    update_source: i32,
    h: &mut IdTagHooks,
    graph_tag_fn: &mut dyn FnMut(usize, u32, i32, &mut IdTagHooks),
) {
    if tag == ID_RECALC_GEOMETRY || tag == 0 {
        for data_id in (h.legacy_data_ids)(id) {
            graph_tag_fn(data_id, 0, update_source, h);
        }
    }
}

/// `graph_id_tag_update_single_flag` (verbatim).
pub fn graph_id_tag_update_single_flag(
    id: usize,
    id_node: Option<usize>,
    tag: u32,
    update_source: i32,
    h: &mut IdTagHooks,
    graph_tag_fn: &mut dyn FnMut(usize, u32, i32, &mut IdTagHooks),
) {
    if tag == ID_RECALC_EDITORS {
        if h.graph_is_active {
            (h.editors_id_update)(id);
        }
        return;
    }
    let id_type = (h.id_type)(id);
    let (component_type, operation_code) = depsgraph_tag_to_component_opcode(
        id_type,
        (h.ob_type)(id),
        (h.ob_has_modifiers)(id),
        tag,
    );
    if component_type == NodeType::Undefined {
        return;
    }
    let Some(id_node) = id_node else {
        return;
    };
    (h.id_cow_recalc_or)(id_node, (h.factory_id_recalc_tag)(component_type));
    /* Frame change is a time update even when reported as
     * user edit. */
    let operation_update_source = if tag == ID_RECALC_FRAME_CHANGE {
        crate::node_id::DEG_UPDATE_SOURCE_TIME
    } else {
        update_source
    };
    if component_type == NodeType::IdRef {
        (h.tag_id_node_update)(id_node, operation_update_source);
    } else {
        depsgraph_tag_component(
            id_node,
            id_type,
            component_type,
            operation_code,
            operation_update_source,
            h,
        );
    }
    deg_graph_id_tag_legacy_compat(id, tag, update_source, h, graph_tag_fn);
}

/// `deg_graph_node_tag_zero` (verbatim).
pub fn deg_graph_node_tag_zero(
    id: usize,
    id_node: Option<usize>,
    update_source: i32,
    h: &mut IdTagHooks,
    graph_tag_fn: &mut dyn FnMut(usize, u32, i32, &mut IdTagHooks),
) {
    let Some(id_node) = id_node else { return };
    (h.id_cow_recalc_or)(id_node, deg_recalc_flags_for_legacy_zero());
    for comp_node in (h.id_components)(id_node) {
        if (h.comp_type)(comp_node) == NodeType::Animation {
            continue;
        }
        if (h.comp_type)(comp_node) == NodeType::CopyOnEval {
            (h.set_cow_explicitly_tagged)(id_node);
        }
        (h.tag_component_update)(comp_node, update_source);
    }
    deg_graph_id_tag_legacy_compat(id, 0, update_source, h, graph_tag_fn);
}

/// `deg_graph_tag_parameters_if_needed` (verbatim).
pub fn deg_graph_tag_parameters_if_needed(
    id: usize,
    id_node: Option<usize>,
    flags: u32,
    update_source: i32,
    h: &mut IdTagHooks,
    graph_tag_fn: &mut dyn FnMut(usize, u32, i32, &mut IdTagHooks),
) {
    if flags == 0 || (flags & ID_RECALC_PARAMETERS) != 0 {
        return;
    }
    let clean_flags = flags
        & !(ID_RECALC_SYNC_TO_EVAL | ID_RECALC_SELECT | ID_RECALC_BASE_FLAGS
            | ID_RECALC_SHADING | ID_RECALC_FRAME_CHANGE);
    if clean_flags == 0 {
        return;
    }
    graph_id_tag_update_single_flag(
        id,
        id_node,
        ID_RECALC_PARAMETERS,
        update_source,
        h,
        graph_tag_fn,
    );
}

/// `graph_id_tag_update` (verbatim — the per-graph entry
/// point called by `id_tag_update` for each graph).
pub fn graph_id_tag_update(
    id: usize,
    flags: u32,
    update_source: i32,
    h: &mut IdTagHooks,
) {
    if h.graph_is_evaluating {
        return;
    }
    (h.set_id_update_count)(id);
    let id_node = (h.find_id_node)(id);
    (h.id_type_tag)((h.id_type)(id));
    if flags == 0 {
        deg_graph_node_tag_zero(id, id_node, update_source, h, &mut graph_id_tag_update);
    }
    if let Some(id_node) = id_node {
        (h.id_cow_recalc_or)(id_node, flags);
    }
    if update_source == crate::node_id::DEG_UPDATE_SOURCE_USER_EDIT {
        (h.id_orig_recalc_or)(id, deg_recalc_flags_effective(Some(h.graph_is_active), flags));
    }
    let mut current_flag = flags;
    while current_flag != 0 {
        let tag = 1u32 << current_flag.trailing_zeros();
        current_flag &= !tag;
        graph_id_tag_update_single_flag(
            id,
            id_node,
            tag,
            update_source,
            h,
            &mut graph_id_tag_update,
        );
    }
    /* Nested node-tree data-blocks. */
    for extra in (h.ntree_extra_ids)(id) {
        graph_id_tag_update(extra, flags, update_source, h);
    }
    if update_source == crate::node_id::DEG_UPDATE_SOURCE_USER_EDIT
        && flags != ID_RECALC_SHADING
    {
        graph_id_tag_update_single_flag(
            id,
            id_node,
            ID_RECALC_POINT_CACHE,
            update_source,
            h,
            &mut graph_id_tag_update,
        );
    }
    deg_graph_tag_parameters_if_needed(
        id,
        id_node,
        flags,
        update_source,
        h,
        &mut graph_id_tag_update,
    );
}

/// `graph_tag_ids_for_visible_update` (verbatim —
/// `graph->need_tag_id_on_graph_visibility_*` flags and the
/// per-IDNode iteration injected).
pub fn graph_tag_ids_for_visible_update(
    id_nodes: &[usize],
    do_time: bool,
    h: &mut IdTagHooks,
) {
    for &id_node in id_nodes {
        let id = id_node; /* `id_node->id_orig` resolved via hooks. */
        let mask = (h.visible_components_mask)(id_node);
        if mask == 0 {
            continue;
        }
        let mut flags = 0u32;
        if !(h.eval_copy_is_expanded)(id_node) {
            flags |= ID_RECALC_SYNC_TO_EVAL;
            if do_time && (h.animdata_from_id)(id) {
                flags |= ID_RECALC_ANIMATION;
            }
        } else if mask == (h.prev_visible_mask)(id_node, None) {
            continue;
        }
        let id_type = (h.id_type)(id);
        if id_type == gs(b"OB") {
            flags |= ID_RECALC_TRANSFORM | ID_RECALC_GEOMETRY;
        }
        if (h.eval_copy_is_needed)(id_type) || flags != 0 {
            graph_id_tag_update(
                id,
                flags,
                crate::node_id::DEG_UPDATE_SOURCE_VISIBILITY,
                h,
            );
        }
        if id_type == gs(b"SC") {
            (h.tag_id_node_update)(id_node, crate::node_id::DEG_UPDATE_SOURCE_VISIBILITY);
        }
        (h.prev_visible_mask)(id_node, Some(mask));
    }
}
