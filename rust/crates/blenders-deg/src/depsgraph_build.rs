//! Port of `intern/depsgraph_build.cc` — external
/// build/relation API (`DEG_add_*_relation`, pipeline
/// dispatch, relations update).

use crate::depsgraph::Depsgraph;
use crate::node::NodeType;
use crate::node_operation::OperationCode;
use crate::depsgraph_tag::{geometry_tag_to_component, OB_CAMERA};
use blenders_dna::id::{gs, ID_RECALC_BASE_FLAGS, ID_RECALC_HIERARCHY};

/// `deg_build_scene_component_type` (verbatim).
pub fn deg_build_scene_component_type(component: i32) -> NodeType {
    use crate::node::{DEG_SCENE_COMP_ANIMATION, DEG_SCENE_COMP_COMPOSITOR,
                      DEG_SCENE_COMP_PARAMETERS, DEG_SCENE_COMP_SEQUENCER};
    match component {
        DEG_SCENE_COMP_PARAMETERS => NodeType::Parameters,
        DEG_SCENE_COMP_ANIMATION => NodeType::Animation,
        DEG_SCENE_COMP_SEQUENCER => NodeType::Sequencer,
        DEG_SCENE_COMP_COMPOSITOR => NodeType::Compositor,
        _ => NodeType::Undefined,
    }
}

/// Builder-side handle: the relation builder calls collapse
/// to one hook per key kind (`add_node_handle_relation`,
/// `add_relation`, `add_special_eval_flag`,
/// `add_customdata_mask`, `add_depends_on_transform_relation`).
pub struct NodeHandleCtx<'a> {
    /// `builder->add_node_handle_relation(ComponentKey|
    /// OperationKey|TimeSourceKey, handle, description,
    /// flags)`. Key = `(id, node_type, opcode?, name?)`.
    pub add_component_relation: &'a mut dyn FnMut(usize, usize, &str),
    /// Operation-keyed add.
    pub add_operation_relation: &'a mut dyn FnMut(usize, OperationCode, &str, i32),
    /// `builder->add_relation(comp_key, comp_key, name)`
    /// → relation flag out.
    pub add_relation: &'a mut dyn FnMut(usize, usize, usize, &str) -> Option<usize>,
    /// `rel->flag |= f`.
    pub relation_flag_or: &'a mut dyn FnMut(usize, i32),
    /// `builder->add_special_eval_flag(id, flag)`.
    pub add_special_eval_flag: &'a mut dyn FnMut(usize, u32),
    /// `builder->add_customdata_mask(object, masks)`.
    pub add_customdata_mask: &'a mut dyn FnMut(usize, u64, u64, u64, u64, u64),
    /// `builder->add_depends_on_transform_relation`.
    pub add_depends_on_transform_relation: &'a mut dyn FnMut(&str),
    /// TimeSourceKey relation.
    pub add_time_source_relation: &'a mut dyn FnMut(&str),
    /// `handle->node->owner->owner->id_orig`.
    pub handle_id: &'a dyn Fn() -> usize,
    /// Scene camera + time-marker cameras for
    /// `DEG_add_scene_camera_relation`.
    pub scene_cameras: &'a dyn Fn(usize) -> Vec<(usize, i16, usize)>,
    /// Collection objects (recursive, incl. instance
    /// collections of empties).
    pub collection_objects: &'a dyn Fn(usize) -> Vec<(usize, i16, usize)>,
}

/// `DEG_add_scene_relation` (verbatim).
pub fn deg_add_scene_relation(
    scene_id: usize,
    component: i32,
    description: &str,
    ctx: &mut NodeHandleCtx,
) {
    let ty = deg_build_scene_component_type(component);
    (ctx.add_component_relation)(scene_id, ty as usize, description);
}

/// `add_camera_parameters_relation` (verbatim).
pub fn add_camera_parameters_relation(
    camera_id: usize,
    description: &str,
    ctx: &mut NodeHandleCtx,
) {
    (ctx.add_component_relation)(camera_id, NodeType::Parameters as usize, description);
}

/// `DEG_add_scene_camera_relation` (verbatim — camera
/// enumeration injected via `scene_cameras`: `(object_id,
/// ob_type, camera_data_id)` including `scene->camera` and
/// non-scene marker cameras).
pub fn deg_add_scene_camera_relation(
    scene_id: usize,
    component: i32,
    description: &str,
    ctx: &mut NodeHandleCtx,
) {
    for (object, ob_type, camera_data) in (ctx.scene_cameras)(scene_id) {
        deg_add_object_relation(object, component, description, ctx);
        if ob_type == OB_CAMERA {
            add_camera_parameters_relation(camera_data, description, ctx);
        }
    }
}

/// `DEG_add_object_relation` (verbatim — `component` is a
/// `DEG_OB_COMP_*`; `nodeTypeFromObjectComponent` resolves it).
pub fn deg_add_object_relation(
    object_id: usize,
    component: i32,
    description: &str,
    ctx: &mut NodeHandleCtx,
) {
    let ty = crate::node::node_type_from_object_component(component);
    (ctx.add_component_relation)(object_id, ty as usize, description);
}

/// `DEG_object_has_geometry_component` (verbatim).
pub fn deg_object_has_geometry_component(ob_id_type: u16, ob_type: i16, has_modifiers: bool) -> bool {
    geometry_tag_to_component(ob_id_type, ob_type, has_modifiers) == NodeType::Geometry
}

/// `DEG_add_collection_geometry_relation` (verbatim).
pub fn deg_add_collection_geometry_relation(
    collection_id: usize,
    description: &str,
    ctx: &mut NodeHandleCtx,
) {
    (ctx.add_operation_relation)(
        collection_id,
        OperationCode::GeometryEvalDone,
        description,
        0,
    );
}

/// `DEG_add_collection_geometry_customdata_mask` (verbatim —
/// recursive empty-instance expansion folded into
/// `collection_objects` which yields `(ob_id, ob_type,
/// instance_collection_id)`).
pub fn deg_add_collection_geometry_customdata_mask(
    collection_id: usize,
    masks: (u64, u64, u64, u64, u64),
    ctx: &mut NodeHandleCtx,
) {
    let mut stack = vec![collection_id];
    while let Some(col) = stack.pop() {
        for (ob, ob_type, instance_collection) in (ctx.collection_objects)(col) {
            (ctx.add_customdata_mask)(ob, masks.0, masks.1, masks.2, masks.3, masks.4);
            if ob_type == crate::depsgraph_tag::OB_EMPTY && instance_collection != 0 {
                stack.push(instance_collection);
            }
        }
    }
}

/// `DEG_add_node_tree_output_relation` (verbatim — adds the
/// preprocess relation with `RELATION_FLAG_NO_FLUSH`).
pub fn deg_add_node_tree_output_relation(
    node_tree_id: usize,
    description: &str,
    ctx: &mut NodeHandleCtx,
) {
    (ctx.add_operation_relation)(node_tree_id, OperationCode::NtreeOutput, description, 0);
    (ctx.add_operation_relation)(
        node_tree_id,
        OperationCode::NtreeGeometryPreprocess,
        description,
        crate::eval_flush::RELATION_FLAG_NO_FLUSH,
    );
}

/// `DEG_add_object_cache_relation` (verbatim).
pub fn deg_add_object_cache_relation(
    cache_file_id: usize,
    component: i32,
    description: &str,
    ctx: &mut NodeHandleCtx,
) {
    let ty = crate::node::node_type_from_object_component(component);
    (ctx.add_component_relation)(cache_file_id, ty as usize, description);
}

/// `DEG_add_bone_relation` (verbatim — the bone name rides
/// in the component key; key = `(id, type, bone_name)`).
pub fn deg_add_bone_relation(
    object_id: usize,
    bone_name: &str,
    component: i32,
    description: &str,
    ctx: &mut NodeHandleCtx,
) {
    let ty = crate::node::node_type_from_object_component(component);
    (ctx.add_component_relation)(object_id, ty as usize, &format!("{description} [{bone_name}]"));
}

/// `DEG_add_vfont_relation` (verbatim).
pub fn deg_add_vfont_relation(vfont_id: usize, description: &str, ctx: &mut NodeHandleCtx) {
    (ctx.add_operation_relation)(vfont_id, OperationCode::ParametersExit, description, 0);
}

/// `DEG_add_sound_relation` (verbatim).
pub fn deg_add_sound_relation(sound_id: usize, description: &str, ctx: &mut NodeHandleCtx) {
    (ctx.add_operation_relation)(sound_id, OperationCode::SoundEval, description, 0);
}

/// `DEG_add_object_pointcache_relation` (verbatim — adds
/// the component relation + a `FLUSH_USER_EDIT_ONLY` point-
/// cache relation to the handle's owner id).
pub fn deg_add_object_pointcache_relation(
    object_id: usize,
    component: i32,
    description: &str,
    ctx: &mut NodeHandleCtx,
) {
    let ty = crate::node::node_type_from_object_component(component);
    (ctx.add_component_relation)(object_id, ty as usize, description);
    let id = (ctx.handle_id)();
    if let Some(rel) = (ctx.add_relation)(
        object_id,
        ty as usize,
        id,
        "Point Cache",
    ) {
        (ctx.relation_flag_or)(rel, crate::eval_flush::RELATION_FLAG_FLUSH_USER_EDIT_ONLY);
    }
}

/// `DEG_add_generic_id_relation` (verbatim).
pub fn deg_add_generic_id_relation(id: usize, description: &str, ctx: &mut NodeHandleCtx) {
    (ctx.add_operation_relation)(id, OperationCode::GenericDatablockUpdate, description, 0);
}

/// `DEG_add_depends_on_transform_relation` (verbatim).
pub fn deg_add_depends_on_transform_relation(description: &str, ctx: &mut NodeHandleCtx) {
    (ctx.add_depends_on_transform_relation)(description);
}

/// `DEG_add_time_source_relation` (verbatim).
pub fn deg_add_time_source_relation(description: &str, ctx: &mut NodeHandleCtx) {
    (ctx.add_time_source_relation)(description);
}

/// `DEG_add_special_eval_flag` (verbatim).
pub fn deg_add_special_eval_flag(id: usize, flag: u32, ctx: &mut NodeHandleCtx) {
    (ctx.add_special_eval_flag)(id, flag);
}

/// `DEG_add_customdata_mask` (verbatim).
pub fn deg_add_customdata_mask(
    object_id: usize,
    masks: (u64, u64, u64, u64, u64),
    ctx: &mut NodeHandleCtx,
) {
    (ctx.add_customdata_mask)(object_id, masks.0, masks.1, masks.2, masks.3, masks.4);
}

/// `DEG_graph_tag_relations_update` (verbatim —
/// `graph_id_tag_update` injected via `tag_update`).
pub fn deg_graph_tag_relations_update(
    graph: &mut Depsgraph,
    tag_update: &mut dyn FnMut(usize, u32, i32),
) {
    graph.need_update_relations = true;
    /* New bases might appear: tag scene BASE_FLAGS|HIERARCHY. */
    if graph.find_id_node(graph.scene).is_some() {
        tag_update(
            graph.scene,
            ID_RECALC_BASE_FLAGS | ID_RECALC_HIERARCHY,
            crate::node_id::DEG_UPDATE_SOURCE_RELATIONS,
        );
    }
}

/// `DEG_graph_relations_update` (verbatim — build injected).
pub fn deg_graph_relations_update(graph: &Depsgraph, build_from_view_layer: &mut dyn FnMut()) {
    if !graph.need_update_relations {
        return;
    }
    build_from_view_layer();
}
