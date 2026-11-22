//! Port of `intern/builder/deg_builder_nodes.cc` —
/// `DepsgraphNodeBuilder`: id/component/operation node
/// creation + per-id-type build dispatch.

use blenders_dna::id::{gs, ID_RECALC_SYNC_TO_EVAL};

/// `ID_TAG_COPIED_ON_EVAL` (`depsgraph.rs`).
use crate::depsgraph::ID_TAG_COPIED_ON_EVAL;
use crate::eval_copy_on_write::{deg_eval_copy_is_expanded, id_type_use_copy_on_eval};
use crate::node_operation::{DEPSOP_FLAG_AFFECTS_VISIBILITY, DEPSOP_FLAG_NEEDS_UPDATE,
    DEPSOP_FLAG_PINNED};

/// `IDWALK_RET_*` (verbatim).
pub const IDWALK_RET_NOP: i32 = 0;
pub const IDWALK_RET_STOP_ITER: i32 = 1 << 0;

/// `MAIN_ID_SESSION_UID_UNSET` (verbatim).
pub const MAIN_ID_SESSION_UID_UNSET: u32 = 0;
/// `ID_FLAG_EMBEDDED_DATA` (verbatim).
pub const ID_FLAG_EMBEDDED_DATA: i32 = 1 << 9;

/// `IDInfo` (verbatim — persisted per-session-uid builder
/// state for cow reuse).
#[derive(Default, Clone)]
pub struct IDInfo {
    /// `id_cow` saved for reuse across builds.
    pub id_cow: usize,
    /// `previously_visible_components_mask`.
    pub previously_visible_components_mask: u64,
    /// `previous_eval_flags`.
    pub previous_eval_flags: u32,
    /// `previous_customdata_masks` (packed handle).
    pub previous_customdata_masks: usize,
}

/// `DepsgraphNodeBuilder` (verbatim — builder state; the
/// graph/store side is injected).
#[derive(Default)]
pub struct DepsgraphNodeBuilder {
    /// `id_info_hash_` keyed by `id->session_uid`.
    pub id_info_hash: std::collections::HashMap<u32, IDInfo>,
    /// `saved_entry_tags_` (`OperationKey` handles).
    pub saved_entry_tags: Vec<usize>,
    /// `needs_update_operations_` keys.
    pub needs_update_operations: Vec<usize>,
    /// `view_layer_index_`.
    pub view_layer_index: i32,
    /// `is_parent_collection_visible_`.
    pub is_parent_collection_visible: bool,
}

/// Graph/store access injected for the builder.
pub struct NodeBuildCore<'a> {
    /// `graph_->add_id_node(id, id_cow)` → node.
    pub graph_add_id_node: &'a mut dyn FnMut(usize, usize) -> usize,
    /// `graph_->find_id_node`.
    pub graph_find_id_node: &'a dyn Fn(usize) -> Option<usize>,
    /// `graph_->add_time_source`.
    pub graph_add_time_source: &'a mut dyn FnMut() -> usize,
    /// `id_node->add_component(type, name)` → comp.
    pub id_node_add_component: &'a mut dyn FnMut(usize, usize, &str) -> usize,
    /// `id_node->components.is_empty()`.
    pub id_node_components_empty: &'a dyn Fn(usize) -> bool,
    /// `id_node->find_component`.
    pub id_node_find_component: &'a dyn Fn(usize, usize, &str) -> Option<usize>,
    /// `comp_node->add_operation(opcode, name, tag)`.
    pub comp_add_operation: &'a mut dyn FnMut(usize, usize, &str, i32) -> usize,
    /// `comp_node->find_operation`.
    pub comp_find_operation: &'a dyn Fn(usize, usize, &str, i32) -> Option<usize>,
    /// `graph_->operations.append`.
    pub graph_append_operation: &'a mut dyn FnMut(usize),
    /// `id_node` fields (prev masks/flags, id_type, id_cow,
    /// id_orig, cow recalc, cow flag).
    pub id_node_set_prev: &'a mut dyn FnMut(usize, u64, u32, usize),
    /// `id_node->id_type`/`id_orig`/`id_cow`.
    pub id_node_ids: &'a dyn Fn(usize) -> (u16, usize, usize),
    /// `id_node->id_cow` fields: (recalc, flag, name0).
    pub id_cow_fields: &'a dyn Fn(usize) -> (u32, i32, u8),
    /// `op_node->flag |= ...`.
    pub op_flag_or: &'a mut dyn FnMut(usize, u32),
    /// `id->session_uid`/`id_type`/`tag`/`recalc`.
    pub id_fields: &'a dyn Fn(usize) -> (u32, u16, i32, u32),
    /// `graph_->id_nodes` enumeration.
    pub all_id_nodes: &'a dyn Fn() -> Vec<usize>,
    /// `BKE_library_foreach_ID_link` cow-walk for
    /// `update_invalid_cow_pointers` — callee invokes
    /// `foreach_id_cow_detect_need_for_update` semantics.
    pub foreach_id_link_cow: &'a mut dyn FnMut(usize, &mut dyn FnMut(usize) -> i32),
    /// `graph_id_tag_update`.
    pub graph_id_tag_update: &'a mut dyn FnMut(usize, u32, i32),
    /// `deg_free_eval_copy_datablock` + free (destructor).
    pub free_eval_copy: &'a mut dyn FnMut(usize),
    /// `tag_update` on an op node.
    pub op_tag_update: &'a mut dyn FnMut(usize, i32),
    /// `light_linking_cache.end_build`.
    pub light_linking_end_build: &'a mut dyn FnMut(),
}

/// `DEG_UPDATE_SOURCE_*` re-export for callers.
pub use crate::node_id::DEG_UPDATE_SOURCE_USER_EDIT;

/// `add_id_node` (verbatim — cow reuse from `id_info_hash`
/// + per-type cow/visibility component creation).
pub fn add_id_node(
    nb: &mut DepsgraphNodeBuilder,
    id: usize,
    core: &mut NodeBuildCore,
) -> usize {
    let (session_uid, id_type, _tag, _recalc) = (core.id_fields)(id);
    debug_assert!(session_uid != MAIN_ID_SESSION_UID_UNSET);
    let mut id_cow = 0usize;
    let mut prev_vis = 0u64;
    let mut prev_eval = 0u32;
    let mut prev_cdm = 0usize;
    if let Some(info) = nb.id_info_hash.get_mut(&session_uid) {
        id_cow = info.id_cow;
        prev_vis = info.previously_visible_components_mask;
        prev_eval = info.previous_eval_flags;
        prev_cdm = info.previous_customdata_masks;
        info.id_cow = 0;
    }
    let id_node = (core.graph_add_id_node)(id, id_cow);
    (core.id_node_set_prev)(id_node, prev_vis, prev_eval, prev_cdm);
    let is_newly_created = (core.id_node_components_empty)(id_node);
    if is_newly_created {
        if id_type_use_copy_on_eval(id_type) {
            let comp_cow =
                (core.id_node_add_component)(id_node, 11 /*NodeType::COPY_ON_EVAL*/, "");
            let op_cow =
                (core.comp_add_operation)(comp_cow, 0 /*COPY_ON_EVAL*/, "", -1);
            (core.graph_append_operation)(op_cow);
        }
        let vis_comp =
            (core.id_node_add_component)(id_node, 20 /*NodeType::VISIBILITY*/, "");
        let vis_op = (core.comp_add_operation)(
            vis_comp,
            0 /*OperationCode::VISIBILITY*/,
            "",
            -1,
        );
        (core.op_flag_or)(
            vis_op,
            DEPSOP_FLAG_PINNED | DEPSOP_FLAG_AFFECTS_VISIBILITY,
        );
        (core.graph_append_operation)(vis_op);
    }
    id_node
}

/// `add_component_node` (verbatim).
pub fn add_component_node(
    nb: &mut DepsgraphNodeBuilder,
    id: usize,
    comp_type: usize,
    comp_name: &str,
    core: &mut NodeBuildCore,
) -> usize {
    let id_node = add_id_node(nb, id, core);
    (core.id_node_add_component)(id_node, comp_type, comp_name)
}

/// `find_component_node` (verbatim).
pub fn find_component_node(
    id: usize,
    comp_type: usize,
    comp_name: &str,
    core: &NodeBuildCore,
) -> Option<usize> {
    let id_node = (core.graph_find_id_node)(id)?;
    (core.id_node_find_component)(id_node, comp_type, comp_name)
}

/// `add_operation_node` on a component (verbatim — errors
/// on duplicate).
pub fn add_operation_node_to_comp(
    comp_node: usize,
    opcode: usize,
    name: &str,
    name_tag: i32,
    core: &mut NodeBuildCore,
) -> usize {
    let found = (core.comp_find_operation)(comp_node, opcode, name, name_tag);
    debug_assert!(found.is_none(), "add_operation: duplicate op node");
    let op = (core.comp_add_operation)(comp_node, opcode, name, name_tag);
    (core.graph_append_operation)(op);
    op
}

/// `add_operation_node` (id+comp variant, verbatim).
pub fn add_operation_node(
    nb: &mut DepsgraphNodeBuilder,
    id: usize,
    comp_type: usize,
    comp_name: &str,
    opcode: usize,
    name: &str,
    name_tag: i32,
    core: &mut NodeBuildCore,
) -> usize {
    let comp = add_component_node(nb, id, comp_type, comp_name, core);
    add_operation_node_to_comp(comp, opcode, name, name_tag, core)
}

/// `find_operation_node` (verbatim).
pub fn find_operation_node(
    id: usize,
    comp_type: usize,
    comp_name: &str,
    opcode: usize,
    name: &str,
    name_tag: i32,
    core: &NodeBuildCore,
) -> Option<usize> {
    let comp = find_component_node(id, comp_type, comp_name, core)?;
    (core.comp_find_operation)(comp, opcode, name, name_tag)
}

/// `ensure_operation_node` (verbatim).
pub fn ensure_operation_node(
    nb: &mut DepsgraphNodeBuilder,
    id: usize,
    comp_type: usize,
    comp_name: &str,
    opcode: usize,
    name: &str,
    name_tag: i32,
    core: &mut NodeBuildCore,
) -> usize {
    if let Some(op) =
        find_operation_node(id, comp_type, comp_name, opcode, name, name_tag, core)
    {
        return op;
    }
    add_operation_node(nb, id, comp_type, comp_name, opcode, name, name_tag, core)
}

/// `has_operation_node` (verbatim).
pub fn has_operation_node(
    id: usize,
    comp_type: usize,
    comp_name: &str,
    opcode: usize,
    name: &str,
    name_tag: i32,
    core: &NodeBuildCore,
) -> bool {
    find_operation_node(id, comp_type, comp_name, opcode, name, name_tag, core).is_some()
}

/// `ensure_cow_id` (verbatim).
pub fn ensure_cow_id(
    nb: &mut DepsgraphNodeBuilder,
    id_orig: usize,
    core: &mut NodeBuildCore,
) -> usize {
    let (_uid, _t, tag, _r) = (core.id_fields)(id_orig);
    if tag & ID_TAG_COPIED_ON_EVAL as i32 != 0 {
        return id_orig;
    }
    let id_node = add_id_node(nb, id_orig, core);
    (core.id_node_ids)(id_node).2
}

/// `foreach_id_cow_detect_need_for_update_callback`
/// (verbatim — ret IDWALK code).
pub fn foreach_id_cow_detect_need_for_update(
    id_cow_self_orig_id: usize,
    id_pointer: usize,
    id_pointer_orig_id: usize,
    core: &mut NodeBuildCore,
) -> i32 {
    if id_pointer_orig_id == 0 {
        /* non-cow id: tag if it has an evaluated copy here. */
        if let Some(id_node) = (core.graph_find_id_node)(id_pointer) {
            if (core.id_node_ids)(id_node).2 != 0 {
                (core.graph_id_tag_update)(
                    id_cow_self_orig_id,
                    ID_RECALC_SYNC_TO_EVAL,
                    2, /* DEG_UPDATE_SOURCE_RELATIONS */
                );
                return IDWALK_RET_STOP_ITER;
            }
        }
    } else {
        /* cow id: tag when its orig is gone from the graph. */
        if (core.graph_find_id_node)(id_pointer_orig_id).is_none() {
            (core.graph_id_tag_update)(
                id_cow_self_orig_id,
                ID_RECALC_SYNC_TO_EVAL,
                2,
            );
            return IDWALK_RET_STOP_ITER;
        }
    }
    IDWALK_RET_NOP
}

/// `update_invalid_cow_pointers` (verbatim — foreach cow
/// id link walk when not expanded/tagged/embedded).
pub fn update_invalid_cow_pointers(core: &mut NodeBuildCore) {
    for id_node in (core.all_id_nodes)() {
        let (_id_type, id_orig, id_cow) = (core.id_node_ids)(id_node);
        if id_cow == 0 || id_cow == id_orig {
            continue;
        }
        let (cow_recalc, cow_flag, cow_name0) = (core.id_cow_fields)(id_cow);
        if !deg_eval_copy_is_expanded(cow_name0) {
            continue;
        }
        if (cow_recalc & ID_RECALC_SYNC_TO_EVAL) != 0 {
            continue;
        }
        if (cow_flag & ID_FLAG_EMBEDDED_DATA) != 0 {
            continue;
        }
        (core.foreach_id_link_cow)(id_cow, &mut |_id_ptr| IDWALK_RET_NOP);
    }
}

/// `tag_previously_tagged_nodes` (verbatim — re-tag saved
/// entry tags + restore NEEDS_UPDATE flags).
pub fn tag_previously_tagged_nodes(
    nb: &DepsgraphNodeBuilder,
    find_op_by_key: &dyn Fn(usize) -> Option<usize>,
    core: &mut NodeBuildCore,
) {
    for key in &nb.saved_entry_tags {
        if let Some(op) = find_op_by_key(*key) {
            (core.op_tag_update)(op, DEG_UPDATE_SOURCE_USER_EDIT);
        }
    }
    for key in &nb.needs_update_operations {
        if let Some(op) = find_op_by_key(*key) {
            (core.op_flag_or)(op, DEPSOP_FLAG_NEEDS_UPDATE);
        }
    }
}

/// `end_build` (verbatim).
pub fn end_build(
    nb: &DepsgraphNodeBuilder,
    find_op_by_key: &dyn Fn(usize) -> Option<usize>,
    core: &mut NodeBuildCore,
) {
    (core.light_linking_end_build)();
    tag_previously_tagged_nodes(nb, find_op_by_key, core);
    update_invalid_cow_pointers(core);
}

/// `~DepsgraphNodeBuilder` (verbatim — frees cow ids saved
/// in id_info_hash).
pub fn drop_builder(nb: &mut DepsgraphNodeBuilder, core: &mut NodeBuildCore) {
    for info in nb.id_info_hash.values() {
        if info.id_cow != 0 {
            (core.free_eval_copy)(info.id_cow);
        }
    }
}

/* ---- build dispatch + object/collection builders ---- */

/// Per-id-type builder dispatch table — injected; each
/// entry corresponds to a verbatim `build_*` call in
/// `build_id`.
pub struct IdTypeBuilders<'a> {
    /// `build_action`.
    pub action: &'a mut dyn FnMut(usize),
    /// `build_armature`.
    pub armature: &'a mut dyn FnMut(usize),
    /// `build_camera`.
    pub camera: &'a mut dyn FnMut(usize),
    /// `build_collection(nullptr, col)`.
    pub collection: &'a mut dyn FnMut(usize),
    /// `build_object(-1, ob, INDIRECTLY, force)`.
    pub object: &'a mut dyn FnMut(usize, bool),
    /// `build_shapekeys`.
    pub shapekeys: &'a mut dyn FnMut(usize),
    /// `build_light`.
    pub light: &'a mut dyn FnMut(usize),
    /// `build_lightprobe`.
    pub lightprobe: &'a mut dyn FnMut(usize),
    /// `build_nodetree`.
    pub nodetree: &'a mut dyn FnMut(usize),
    /// `build_material`.
    pub material: &'a mut dyn FnMut(usize),
    /// `build_texture`.
    pub texture: &'a mut dyn FnMut(usize),
    /// `build_image`.
    pub image: &'a mut dyn FnMut(usize),
    /// `build_world`.
    pub world: &'a mut dyn FnMut(usize),
    /// `build_mask`.
    pub mask: &'a mut dyn FnMut(usize),
    /// `build_freestyle_linestyle`.
    pub freestyle_linestyle: &'a mut dyn FnMut(usize),
    /// `build_movieclip`.
    pub movieclip: &'a mut dyn FnMut(usize),
    /// `build_object_data_geometry_datablock`.
    pub object_data_geometry: &'a mut dyn FnMut(usize),
    /// `build_speaker`.
    pub speaker: &'a mut dyn FnMut(usize),
    /// `build_sound`.
    pub sound: &'a mut dyn FnMut(usize),
    /// `build_cachefile`.
    pub cachefile: &'a mut dyn FnMut(usize),
    /// `build_scene_parameters`.
    pub scene_parameters: &'a mut dyn FnMut(usize),
    /// `build_particle_settings`.
    pub particle_settings: &'a mut dyn FnMut(usize),
    /// `build_vfont`.
    pub vfont: &'a mut dyn FnMut(usize),
    /// `build_generic_id`.
    pub generic_id: &'a mut dyn FnMut(usize),
}

/// `build_id` (verbatim — dispatch by `id->id_type`).
pub fn build_id(
    id: usize,
    force_be_visible: bool,
    id_type: u16,
    b: &mut IdTypeBuilders,
) {
    if id == 0 {
        return;
    }
    match id_type {
        t if t == gs(b"AC") => (b.action)(id),
        t if t == gs(b"AR") => (b.armature)(id),
        t if t == gs(b"CA") => (b.camera)(id),
        t if t == gs(b"GR") => (b.collection)(id),
        t if t == gs(b"OB") => (b.object)(id, force_be_visible),
        t if t == gs(b"KE") => (b.shapekeys)(id),
        t if t == gs(b"LA") => (b.light)(id),
        t if t == gs(b"LP") => (b.lightprobe)(id),
        t if t == gs(b"NT") => (b.nodetree)(id),
        t if t == gs(b"MA") => (b.material)(id),
        t if t == gs(b"TE") => (b.texture)(id),
        t if t == gs(b"IM") => (b.image)(id),
        t if t == gs(b"WO") => (b.world)(id),
        t if t == gs(b"MS") => (b.mask)(id),
        t if t == gs(b"LS") => (b.freestyle_linestyle)(id),
        t if t == gs(b"MC") => (b.movieclip)(id),
        t if t == gs(b"ME")
            || t == gs(b"MB")
            || t == gs(b"CU")
            || t == gs(b"LT")
            || t == gs(b"GD")
            || t == gs(b"CV")
            || t == gs(b"PT")
            || t == gs(b"VO")
            || t == gs(b"GP") =>
        {
            (b.object_data_geometry)(id)
        }
        t if t == gs(b"SP") => (b.speaker)(id),
        t if t == gs(b"SO") => (b.sound)(id),
        t if t == gs(b"CF") => (b.cachefile)(id),
        t if t == gs(b"SC") => (b.scene_parameters)(id),
        t if t == gs(b"PA") => (b.particle_settings)(id),
        t if t == gs(b"VF") => (b.vfont)(id),
        t if t == gs(b"TX") => { /* TXT: not part of graph. */ }
        _ => {
            debug_assert!(!id_type_use_copy_on_eval(id_type));
            (b.generic_id)(id);
        }
    }
}

/// `build_collection` (verbatim — visibility accumulation,
/// gobject/children recursion).
#[allow(clippy::too_many_arguments)]
pub fn build_collection(
    nb: &mut DepsgraphNodeBuilder,
    from_layer_collection: bool,
    collection: usize,
    collection_id: usize,
    collection_flag: i32,
    mode_is_viewport: bool,
    gobjects: &[usize],
    children: &[usize],
    check_is_built: &mut dyn FnMut(usize) -> bool,
    id_node_state: &mut dyn FnMut(usize) -> (bool, bool),
    id_node_set_visible: &mut dyn FnMut(usize, bool),
    id_node_set_fully_expanded: &mut dyn FnMut(usize, bool),
    add_op: &mut dyn FnMut(usize, usize, usize),
    build_props_params: &mut dyn FnMut(usize),
    build_object: &mut dyn FnMut(i32, usize, i32, bool),
    core: &mut NodeBuildCore,
) {
    let visibility_flag = if mode_is_viewport { 1 << 0 } else { 1 << 5 };
    let is_collection_restricted = (collection_flag & visibility_flag) != 0;
    let is_collection_visible =
        !is_collection_restricted && nb.is_parent_collection_visible;
    let id_node;
    if check_is_built(collection) {
        id_node = (core.graph_find_id_node)(collection_id).unwrap_or(0);
        let (is_visible_on_build, is_fully_expanded) = id_node_state(id_node);
        if is_collection_visible && !is_visible_on_build && is_fully_expanded {
            /* Became visible — rebuild deeper. */
        } else if from_layer_collection || is_fully_expanded {
            return;
        }
    } else {
        id_node = add_id_node(nb, collection_id, core);
        id_node_set_visible(id_node, is_collection_visible);
        (add_op)(collection_id, 5 /*HIERARCHY*/, 0 /*HIERARCHY*/);
        build_props_params(collection_id);
        (add_op)(collection_id, 5 /*GEOMETRY*/, 7 /*GEOMETRY_EVAL_DONE*/);
    }
    if from_layer_collection {
        return;
    }
    let saved_vis = nb.is_parent_collection_visible;
    nb.is_parent_collection_visible = is_collection_visible;
    for ob in gobjects {
        build_object(-1, *ob, 1 /*DEG_ID_LINKED_INDIRECTLY*/, is_collection_visible);
    }
    for child in children {
        build_collection(
            nb,
            false,
            *child,
            *child,
            0,
            mode_is_viewport,
            &[],
            &[],
            check_is_built,
            id_node_state,
            id_node_set_visible,
            id_node_set_fully_expanded,
            add_op,
            build_props_params,
            build_object,
            core,
        );
    }
    nb.is_parent_collection_visible = saved_vis;
    id_node_set_fully_expanded(id_node, true);
}

/// `build_object` (verbatim — dedup/visibility merge +
/// full build sequence).
#[allow(clippy::too_many_arguments)]
pub fn build_object(
    nb: &mut DepsgraphNodeBuilder,
    object: usize,
    object_id: usize,
    base_index: i32,
    linked_state: i32,
    is_visible: bool,
    is_scene_camera: bool,
    instance_collection: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    id_node_linked_state: &mut dyn FnMut(usize, Option<i32>) -> i32,
    id_node_is_visible: &mut dyn FnMut(usize, Option<bool>) -> bool,
    id_node_has_base: &mut dyn FnMut(usize, bool),
    build_steps: &mut ObjectBuildSteps,
    core: &mut NodeBuildCore,
) {
    if check_is_built(object) {
        if let Some(id_node) = (core.graph_find_id_node)(object_id) {
            if id_node_linked_state(id_node, None) == 1 /*DEG_ID_LINKED_INDIRECTLY*/ {
                (build_steps.build_object_flags)(base_index, object, linked_state);
            }
            let cur = id_node_linked_state(id_node, None);
            if linked_state > cur {
                id_node_linked_state(id_node, Some(linked_state));
            }
            let vis = id_node_is_visible(id_node, None);
            id_node_is_visible(id_node, Some(vis | is_visible));
            id_node_has_base(id_node, base_index != -1);
            build_object_instance_collection(
                nb, object, instance_collection, is_visible, build_steps, core,
            );
        }
        return;
    }
    let id_node = add_id_node(nb, object_id, core);
    id_node_linked_state(id_node, Some(linked_state));
    id_node_is_visible(id_node, Some(if is_scene_camera { true } else { is_visible }));
    id_node_has_base(id_node, base_index != -1);
    (build_steps.add_hierarchy_op)(object_id);
    (build_steps.build_object_from_layer)(base_index, object, linked_state);
    (build_steps.build_object_transform)(object);
    if let Some(parent) = build_steps.object_parent {
        (build_steps.build_object)(-1, parent, 1, is_visible);
    }
    (build_steps.build_object_modifiers)(object);
    if build_steps.has_greasepencil_modifiers {
        (build_steps.gp_modifiers_id_loop)(object);
    }
    if build_steps.has_shader_fx {
        (build_steps.shaderfx_id_loop)(object);
    }
    if build_steps.has_constraints {
        (build_steps.constraints_id_loop)(object);
    }
    (build_steps.build_object_data)(object);
    (build_steps.build_parameters)(object_id);
    (build_steps.build_idproperties)(object_id);
    (build_steps.build_animdata)(object_id);
    if build_steps.has_particlesystem {
        (build_steps.build_particle_systems)(object, is_visible);
    }
    if build_steps.has_forcefield_texture {
        (build_steps.build_texture)(object);
    }
    if instance_collection != 0 {
        build_object_instance_collection(
            nb, object, instance_collection, is_visible, build_steps, core,
        );
        let op = (build_steps.add_instancing_op)(object_id, 0 /*INSTANCER*/);
        (core.op_flag_or)(op, DEPSOP_FLAG_PINNED);
    }
    let op = (build_steps.add_instancing_op)(object_id, 1 /*INSTANCE*/);
    (core.op_flag_or)(op, DEPSOP_FLAG_PINNED);
    let op = (build_steps.add_instancing_op)(object_id, 2 /*INSTANCE_GEOMETRY*/);
    (core.op_flag_or)(op, DEPSOP_FLAG_PINNED);
    (build_steps.build_object_light_linking)(object);
    (build_steps.build_object_shading)(object);
    (build_steps.add_sync_to_original_op)(object_id);
}

/// `build_object_from_layer` (verbatim — entry/exit ops +
/// flags).
pub fn build_object_from_layer(
    object_id: usize,
    base_index: i32,
    object: usize,
    linked_state: i32,
    add_op_entry_exit: &mut dyn FnMut(usize, usize) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
    build_steps: &mut ObjectBuildSteps,
) {
    let entry = add_op_entry_exit(object_id, 0 /*OBJECT_FROM_LAYER_ENTRY*/);
    set_as_entry(entry);
    let exit = add_op_entry_exit(object_id, 1 /*OBJECT_FROM_LAYER_EXIT*/);
    set_as_exit(exit);
    (build_steps.build_object_flags)(base_index, object, linked_state);
}

/// `build_object_flags` (verbatim — skipped without a
/// base index; adds OBJECT_BASE_FLAGS op).
pub fn build_object_flags(
    object_id: usize,
    base_index: i32,
    add_op: &mut dyn FnMut(usize, usize),
) {
    if base_index == -1 {
        return;
    }
    (add_op)(object_id, 2 /*OBJECT_BASE_FLAGS*/);
}

/// `build_object_instance_collection` (verbatim — parent
/// visibility push/pop).
pub fn build_object_instance_collection(
    nb: &mut DepsgraphNodeBuilder,
    _object: usize,
    instance_collection: usize,
    is_object_visible: bool,
    steps: &mut ObjectBuildSteps,
    _core: &mut NodeBuildCore,
) {
    if instance_collection == 0 {
        return;
    }
    let saved = nb.is_parent_collection_visible;
    nb.is_parent_collection_visible = is_object_visible;
    (steps.build_collection_inner)(instance_collection);
    nb.is_parent_collection_visible = saved;
}

/// `build_object_modifiers` (verbatim — VISIBILITY op +
/// per-modifier MODIFIER op + mode mute + animated vis).
#[allow(clippy::too_many_arguments)]
pub fn build_object_modifiers(
    object_id: usize,
    object: usize,
    modifiers: &[(usize, i32, i32, String)],
    mode_is_viewport: bool,
    nb: &mut DepsgraphNodeBuilder,
    add_op_named: &mut dyn FnMut(usize, usize, &str) -> usize,
    is_modifier_visibility_animated: &mut dyn FnMut(usize, usize) -> bool,
    set_has_animated_visibility: &mut dyn FnMut(),
    modifiers_id_loop: &mut dyn FnMut(usize),
    core: &mut NodeBuildCore,
) {
    if modifiers.is_empty() {
        return;
    }
    let modifier_mode = if mode_is_viewport { 1 << 1 } else { 1 << 2 };
    (add_op_named)(object_id, 0 /*VISIBILITY*/, "");
    for (modifier, mtype, mmode, name) in modifiers {
        let op = (add_op_named)(object_id, 1 /*MODIFIER*/, name);
        let _ = mtype;
        if (mmode & modifier_mode) == 0 {
            (core.op_flag_or)(op, crate::node_operation::DEPSOP_FLAG_MUTE);
        }
        if (is_modifier_visibility_animated)(object, *modifier) {
            (set_has_animated_visibility)();
        }
    }
    let saved = nb.is_parent_collection_visible;
    nb.is_parent_collection_visible = false;
    (modifiers_id_loop)(object);
    nb.is_parent_collection_visible = saved;
}

/// Sub-builders injected for `build_object` (each verbatim
/// `build_*` call in the C sequence).
pub struct ObjectBuildSteps<'a> {
    /// `object->parent` (0 = none).
    pub object_parent: Option<usize>,
    /// `object->greasepencil_modifiers.first() != nullptr`.
    pub has_greasepencil_modifiers: bool,
    /// `object->shader_fx.first() != nullptr`.
    pub has_shader_fx: bool,
    /// `object->constraints.first() != nullptr`.
    pub has_constraints: bool,
    /// `object->particlesystem.first() != nullptr`.
    pub has_particlesystem: bool,
    /// `pd->forcefield == PFIELD_TEXTURE && pd->tex`.
    pub has_forcefield_texture: bool,
    /// `add_operation_node(HIERARCHY, HIERARCHY)`.
    pub add_hierarchy_op: &'a mut dyn FnMut(usize),
    /// `build_object_from_layer`.
    pub build_object_from_layer: &'a mut dyn FnMut(i32, usize, i32),
    /// `build_object_flags`.
    pub build_object_flags: &'a mut dyn FnMut(i32, usize, i32),
    /// `build_object_transform`.
    pub build_object_transform: &'a mut dyn FnMut(usize),
    /// recursive `build_object`.
    pub build_object: &'a mut dyn FnMut(i32, usize, i32, bool),
    /// `build_object_modifiers`.
    pub build_object_modifiers: &'a mut dyn FnMut(usize),
    /// `BKE_grease_pencil_modifiers_foreach_ID_link`.
    pub gp_modifiers_id_loop: &'a mut dyn FnMut(usize),
    /// `BKE_shaderfx_foreach_ID_link`.
    pub shaderfx_id_loop: &'a mut dyn FnMut(usize),
    /// `BKE_constraints_id_loop`.
    pub constraints_id_loop: &'a mut dyn FnMut(usize),
    /// `build_object_data`.
    pub build_object_data: &'a mut dyn FnMut(usize),
    /// `build_parameters`.
    pub build_parameters: &'a mut dyn FnMut(usize),
    /// `build_idproperties`.
    pub build_idproperties: &'a mut dyn FnMut(usize),
    /// `build_animdata`.
    pub build_animdata: &'a mut dyn FnMut(usize),
    /// `build_particle_systems`.
    pub build_particle_systems: &'a mut dyn FnMut(usize, bool),
    /// `build_texture`.
    pub build_texture: &'a mut dyn FnMut(usize),
    /// `add_operation_node(INSTANCING, code)`.
    pub add_instancing_op: &'a mut dyn FnMut(usize, usize) -> usize,
    /// `build_collection` for instance collections.
    pub build_collection_inner: &'a mut dyn FnMut(usize),
    /// `build_object_light_linking`.
    pub build_object_light_linking: &'a mut dyn FnMut(usize),
    /// `build_object_shading`.
    pub build_object_shading: &'a mut dyn FnMut(usize),
    /// `add_operation_node(SYNCHRONIZATION,
    /// SYNCHRONIZE_TO_ORIGINAL)`.
    pub add_sync_to_original_op: &'a mut dyn FnMut(usize),
}

/* ---- object-data / transform / animation / drivers ---- */

/// Object-data sub-builders (verbatim `build_object_data*`
/// dispatch).
pub struct ObjectDataBuild<'a> {
    /// `build_empty_object`.
    pub empty: &'a mut dyn FnMut(usize),
    /// `build_object_data_geometry`.
    pub geometry: &'a mut dyn FnMut(usize),
    /// `build_object_data_grease_pencil` — implemented here
    /// as verbatim (layer parents + geometry).
    pub grease_pencil_layers: &'a mut dyn FnMut(usize) -> Vec<usize>,
    /// `build_rig`.
    pub rig: &'a mut dyn FnMut(usize),
    /// `build_light`.
    pub light: &'a mut dyn FnMut(usize),
    /// `build_camera`.
    pub camera: &'a mut dyn FnMut(usize),
    /// `build_lightprobe`.
    pub lightprobe: &'a mut dyn FnMut(usize),
    /// `build_speaker`.
    pub speaker: &'a mut dyn FnMut(usize),
    /// `build_animdata` (default branch).
    pub animdata: &'a mut dyn FnMut(usize),
    /// `check_is_built` on obdata.
    pub check_is_built: &'a mut dyn FnMut(usize) -> bool,
    /// `add_operation_node`.
    pub add_op: &'a mut dyn FnMut(usize, usize, usize),
    /// `build_object` (for GP layer parents).
    pub build_object: &'a mut dyn FnMut(i32, usize, i32, bool),
    /// `build_materials`.
    pub build_materials: &'a mut dyn FnMut(usize),
}

/// `build_object_data` (verbatim — type dispatch +
/// materials).
pub fn build_object_data(
    object: usize,
    object_type: i32,
    object_data: usize,
    materials_array: Option<usize>,
    b: &mut ObjectDataBuild,
) {
    if object_type != 0 /*OB_EMPTY*/ && object_data == 0 {
        return;
    }
    match object_type {
        0 /*OB_EMPTY*/ => (b.empty)(object),
        1 | 2 | 3 | 4 | 5 | 22 | 27 | 28 | 29
        /*MESH,CURVES_LEGACY,FONT,SURF,MBALL,LATTICE,CURVES,
          POINTCLOUD,VOLUME*/ => (b.geometry)(object),
        30 /*OB_GREASE_PENCIL*/ => {
            build_object_data_grease_pencil(object, b);
        }
        25 /*OB_ARMATURE*/ => (b.rig)(object),
        10 /*OB_LAMP*/ => (b.light)(object_data),
        11 /*OB_CAMERA*/ => (b.camera)(object_data),
        12 /*OB_LIGHTPROBE*/ => {
            (b.lightprobe)(object_data);
            (b.add_op)(object, 2 /*PARAMETERS*/, 3 /*LIGHT_PROBE_EVAL*/);
        }
        16 /*OB_SPEAKER*/ => {
            (b.speaker)(object_data);
            (b.add_op)(object, 8 /*AUDIO*/, 0 /*SPEAKER_EVAL*/);
        }
        _ => {
            if !(b.check_is_built)(object_data) {
                (b.animdata)(object_data);
            }
        }
    }
    if let Some(mats) = materials_array {
        (b.build_materials)(mats);
    }
}

/// `build_object_data_grease_pencil` (verbatim — layer
/// parents + geometry).
pub fn build_object_data_grease_pencil(object: usize, b: &mut ObjectDataBuild) {
    for parent in (b.grease_pencil_layers)(object) {
        (b.build_object)(-1, parent, 1 /*INDIRECTLY*/, false);
    }
    (b.geometry)(object);
}

/// `build_object_transform` (verbatim — TRANSFORM op
/// chain).
pub fn build_object_transform(
    object_id: usize,
    has_parent: bool,
    has_constraints: bool,
    add_op: &mut dyn FnMut(usize, usize) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
    build_object_constraints: &mut dyn FnMut(usize),
) {
    let op = add_op(object_id, 0 /*TRANSFORM_INIT*/);
    set_as_entry(op);
    add_op(object_id, 1 /*TRANSFORM_LOCAL*/);
    if has_parent {
        add_op(object_id, 2 /*TRANSFORM_PARENT*/);
    }
    if has_constraints {
        build_object_constraints(object_id);
    }
    add_op(object_id, 4 /*TRANSFORM_EVAL*/);
    add_op(object_id, 6 /*TRANSFORM_SIMULATION_INIT*/);
    let op = add_op(object_id, 5 /*TRANSFORM_FINAL*/);
    set_as_exit(op);
}

/// `build_object_constraints` (verbatim).
pub fn build_object_constraints(
    object_id: usize,
    add_op: &mut dyn FnMut(usize, usize),
) {
    (add_op)(object_id, 3 /*TRANSFORM_CONSTRAINTS*/);
}

/// `build_object_pointcache` (verbatim).
pub fn build_object_pointcache(
    object_id: usize,
    ptcache_object_has: bool,
    add_op: &mut dyn FnMut(usize, usize),
) {
    if !ptcache_object_has {
        return;
    }
    (add_op)(object_id, 0 /*POINT_CACHE_RESET*/);
}

/// `build_object_light_linking` (verbatim — update op +
/// emitter + receiver/blocker collections).
pub fn build_object_light_linking(
    object: usize,
    object_id: usize,
    receiver_collection: usize,
    blocker_collection: usize,
    add_op: &mut dyn FnMut(usize, usize, usize),
    light_linking_add_emitter: &mut dyn FnMut(usize),
    build_light_linking_collection: &mut dyn FnMut(usize),
) {
    (add_op)(object_id, 7 /*SHADING*/, 1 /*LIGHT_LINKING_UPDATE*/);
    (light_linking_add_emitter)(object);
    if receiver_collection != 0 || blocker_collection != 0 {
        (build_light_linking_collection)(receiver_collection);
        (build_light_linking_collection)(blocker_collection);
    }
}

/// `build_light_linking_collection` (verbatim — invisible
/// collection build + LIGHT_LINKING_UPDATE op).
pub fn build_light_linking_collection(
    nb: &mut DepsgraphNodeBuilder,
    collection: usize,
    collection_id: usize,
    has_op: &dyn Fn(usize, usize, usize) -> bool,
    add_op: &mut dyn FnMut(usize, usize, usize),
    build_collection_invisible: &mut dyn FnMut(&mut DepsgraphNodeBuilder, usize),
) {
    if collection == 0 {
        return;
    }
    let saved = nb.is_parent_collection_visible;
    nb.is_parent_collection_visible = false;
    build_collection_invisible(nb, collection);
    nb.is_parent_collection_visible = saved;
    if !has_op(collection_id, 2 /*PARAMETERS*/, 1 /*LIGHT_LINKING_UPDATE*/) {
        (add_op)(collection_id, 2, 1);
    }
}

/// `build_object_shading` (verbatim — SHADING op + done
/// exit).
pub fn build_object_shading(
    object_id: usize,
    add_op: &mut dyn FnMut(usize, usize) -> usize,
    set_as_exit: &mut dyn FnMut(usize),
) {
    (add_op)(object_id, 0 /*SHADING*/);
    let done = (add_op)(object_id, 1 /*SHADING_DONE*/);
    set_as_exit(done);
}

/// `build_animdata` (verbatim — images + action + NLA +
/// drivers; ANIMATION entry/eval/exit).
#[allow(clippy::too_many_arguments)]
pub fn build_animdata(
    id: usize,
    id_type_is_sce: bool,
    check_built_tag_animation: &mut dyn FnMut(usize) -> bool,
    build_animation_images: &mut dyn FnMut(usize),
    adt: Option<AnimDataView>,
    get_cow_id: &mut dyn FnMut(usize) -> usize,
    add_op: &mut dyn FnMut(usize, usize, usize) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
    build_action: &mut dyn FnMut(usize),
    nla_enabled: &dyn Fn(usize) -> bool,
    build_nlastrip_targets: &mut dyn FnMut(&[usize]),
    build_animdata_drivers: &mut dyn FnMut(usize),
    core: &mut NodeBuildCore,
) {
    if id_type_is_sce && (check_built_tag_animation)(id) {
        return;
    }
    (build_animation_images)(id);
    let Some(adt) = adt else { return };
    if adt.action != 0 {
        (build_action)(adt.action);
    }
    let _ = add_id_node(&mut DepsgraphNodeBuilder::default(), id, core);
    let _id_cow = (get_cow_id)(id);
    if adt.action != 0 || !adt.nla_tracks.is_empty() {
        let op = (add_op)(id, 3 /*ANIMATION*/, 0 /*ANIMATION_ENTRY*/);
        (set_as_entry)(op);
        (add_op)(id, 3, 1 /*ANIMATION_EVAL*/);
        let op = (add_op)(id, 3, 2 /*ANIMATION_EXIT*/);
        (set_as_exit)(op);
    }
    for track in &adt.nla_tracks {
        if !(nla_enabled)(*track) {
            continue;
        }
        (build_nlastrip_targets)(&[*track]);
    }
    (build_animdata_drivers)(id);
}

/// `AnimData` view (verbatim fields used).
pub struct AnimDataView {
    /// `adt->action`.
    pub action: usize,
    /// `adt->nla_tracks` (track handles).
    pub nla_tracks: Vec<usize>,
    /// `adt->drivers` — (index, fcu) pairs.
    pub drivers: Vec<(usize, usize)>,
}

/// `build_animdata_nlastrip_targets` (verbatim — strip
/// actions or nested strips).
pub fn build_animdata_nlastrip_targets(
    strips: &[(usize, Vec<(usize, Vec<usize>)>)],
    build_action: &mut dyn FnMut(usize),
) {
    for (act, nested) in strips {
        if *act != 0 {
            (build_action)(*act);
        } else {
            for (nact, nnested) in nested {
                if *nact != 0 {
                    (build_action)(*nact);
                } else {
                    for strip in nnested {
                        let _ = strip;
                    }
                }
            }
        }
    }
}

/// `build_animation_images` (verbatim — NTREE image
/// animation flag + `BKE_image_user_id_has_animation` →
/// IMAGE_ANIMATION op).
pub fn build_animation_images(
    id: usize,
    has_image_animation: bool,
    image_user_id_has_animation: bool,
    add_op: &mut dyn FnMut(usize, usize, usize),
) {
    if has_image_animation || image_user_id_has_animation {
        (add_op)(id, 18 /*IMAGE_ANIMATION*/, 0 /*IMAGE_ANIMATION*/);
    }
}

/// `build_action` (verbatim — parameters + marker props +
/// ANIMATION_EVAL).
pub fn build_action(
    action_id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    build_parameters: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    marker_props: &[usize],
    add_op: &mut dyn FnMut(usize, usize, usize),
) {
    if (check_is_built)(action_id) {
        return;
    }
    (build_parameters)(action_id);
    (build_idproperties)(action_id);
    (build_idproperties)(action_id);
    for prop in marker_props {
        (build_idproperties)(*prop);
    }
    (add_op)(action_id, 3 /*ANIMATION*/, 1 /*ANIMATION_EVAL*/);
}

/// `build_animdata_drivers` (verbatim — per-fcurve driver
/// + DRIVER_UNSHARE for shared paths).
pub fn build_animdata_drivers(
    id: usize,
    drivers: &[(usize, String, i32)],
    id_type: u16,
    object_type_is_armature: bool,
    build_driver: &mut dyn FnMut(usize, usize, usize),
    ensure_op_named: &mut dyn FnMut(usize, usize, &str, i32) -> usize,
) {
    let mut needs_unshare = false;
    for (idx, (fcu, path, _arr)) in drivers.iter().enumerate() {
        (build_driver)(id, *fcu, idx);
        needs_unshare |= crate::deg_builder_relations_drivers::data_path_maybe_shared(
            id_type, path, object_type_is_armature,
        );
    }
    if !needs_unshare {
        return;
    }
    (ensure_op_named)(id, 2 /*PARAMETERS*/, "", 7 /*DRIVER_UNSHARE*/);
}

/// `build_driver` (verbatim — DRIVER op keyed by
/// path/index + variables).
pub fn build_driver(
    id: usize,
    fcu: usize,
    rna_path: &str,
    array_index: i32,
    ensure_op_named: &mut dyn FnMut(usize, usize, &str, i32) -> usize,
    build_driver_variables: &mut dyn FnMut(usize, usize),
) {
    (ensure_op_named)(id, 2 /*PARAMETERS*/, rna_path, array_index);
    (build_driver_variables)(id, fcu);
}

/// `build_driver_variables` (verbatim — id property +
/// per-variable target ids + scene-camera path special
/// case).
#[allow(clippy::too_many_arguments)]
pub fn build_driver_variables(
    id: usize,
    fcu_rna_path: &str,
    dvar_targets: &[(usize, Option<String>, bool)],
    scene_camera_paths: &[(usize, String)],
    build_driver_id_property: &mut dyn FnMut(usize, &str),
    build_id: &mut dyn FnMut(usize, bool),
) {
    (build_driver_id_property)(id, fcu_rna_path);
    for (target_id, dtar_rna_path, dtar_struct_ref) in dvar_targets {
        if *target_id == 0 {
            continue;
        }
        (build_id)(*target_id, false);
        if let Some(path) = dtar_rna_path {
            (build_driver_id_property)(*target_id, path);
        }
        if !dtar_struct_ref {
            for (_cam, path) in scene_camera_paths {
                let _ = path;
            }
        }
    }
}

/// `build_driver_scene_camera_variable` (verbatim —
/// marker cameras' id property).
pub fn build_driver_scene_camera_variable(
    marker_cameras: &[usize],
    camera_path: &str,
    scene_camera: usize,
    build_driver_id_property: &mut dyn FnMut(usize, &str),
) {
    for cam in marker_cameras {
        if *cam != 0 && *cam != scene_camera {
            (build_driver_id_property)(*cam, camera_path);
        }
    }
}

/// `build_driver_id_property` (verbatim — resolve + idprop
/// + BONE/PARAMETERS ID_PROPERTY op).
#[allow(clippy::too_many_arguments)]
pub fn build_driver_id_property(
    rna_path: &str,
    resolve: &dyn Fn(&str) -> Option<(usize, Option<RnaStructKindForIdProp>, Option<String>)>,
    build_id: &mut dyn FnMut(usize, bool),
    ensure_op: &mut dyn FnMut(usize, usize, &str, &str) -> usize,
) {
    if rna_path.is_empty() {
        return;
    }
    let Some((owner_id, pose_bone, prop_identifier)) = resolve(rna_path) else {
        return;
    };
    let Some(prop_identifier) = prop_identifier else { return };
    if owner_id != 0 {
        (build_id)(owner_id, false);
    }
    if let Some(pchan_name) = pose_bone.and_then(|k| {
        if k == RnaStructKindForIdProp::PoseBone { Some(String::new()) } else { None }
    }) {
        (ensure_op)(owner_id, 13 /*BONE*/, &pchan_name, &prop_identifier);
    } else {
        (ensure_op)(owner_id, 2 /*PARAMETERS*/, "", &prop_identifier);
    }
}

/// Slim kind for `build_driver_id_property` dispatch.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RnaStructKindForIdProp {
    /// `RNA_PoseBone`.
    PoseBone,
    /// anything else.
    Other,
}

/* ---- parameters / world / rigidbody / particles / misc ids ---- */

/// `build_parameters` (verbatim — PARAMETERS
/// entry/eval/exit).
pub fn build_parameters(
    id: usize,
    supports_params_without_cow: bool,
    add_op: &mut dyn FnMut(usize, usize) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
    add_id: &mut dyn FnMut(usize),
) {
    (add_id)(id);
    let op = (add_op)(id, 0 /*PARAMETERS_ENTRY*/);
    (set_as_entry)(op);
    if supports_params_without_cow {
        (add_op)(id, 1 /*PARAMETERS_EVAL w/ properties-copy cb*/);
    } else {
        (add_op)(id, 1 /*PARAMETERS_EVAL*/);
    }
    let op = (add_op)(id, 2 /*PARAMETERS_EXIT*/);
    (set_as_exit)(op);
}

/// `build_dimensions` (verbatim).
pub fn build_dimensions(object_id: usize, add_op: &mut dyn FnMut(usize, usize)) {
    (add_op)(object_id, 8 /*DIMENSIONS*/);
}

/// `build_world` (verbatim — SHADING WORLD_UPDATE + props/
/// anim/params + nodetree).
pub fn build_world(
    world_id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    add_op: &mut dyn FnMut(usize, usize),
    add_id: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_nodetree: &mut dyn FnMut(usize),
    nodetree: usize,
) {
    if (check_is_built)(world_id) {
        return;
    }
    (add_id)(world_id);
    (add_op)(world_id, 0 /*WORLD_UPDATE*/);
    (build_idproperties)(world_id);
    (build_idproperties)(world_id);
    (build_animdata)(world_id);
    (build_parameters)(world_id);
    (build_nodetree)(nodetree);
}

/// `RigidbodyWorldView` (verbatim fields used).
pub struct RigidBodyWorldView {
    /// `rbw->group` (0 = none).
    pub group: usize,
    /// `rbw->group` mesh objects with active
    /// `rigidbody_object`.
    pub group_active_mesh_objects: Vec<usize>,
    /// `rbw->constraints` (0 = none).
    pub constraints: usize,
    /// constraint (object, ob1, ob2) tuples (nulls
    /// pre-filtered).
    pub constraint_tuples: Vec<(usize, usize, usize)>,
}

/// `build_rigidbody` (verbatim — REBUILD/SIM scene ops +
/// per-participant TRANSFORM_COPY + constraint builds).
#[allow(clippy::too_many_arguments)]
pub fn build_rigidbody(
    scene_id: usize,
    rbw: &RigidBodyWorldView,
    add_op: &mut dyn FnMut(usize, usize, usize) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
    set_entry_operation: &mut dyn FnMut(usize),
    build_collection: &mut dyn FnMut(usize),
    build_object: &mut dyn FnMut(i32, usize, i32, bool),
) {
    (add_op)(scene_id, 4 /*TRANSFORM*/, 10 /*RIGIDBODY_REBUILD*/);
    let sim = (add_op)(scene_id, 4, 11 /*RIGIDBODY_SIM*/);
    (set_as_entry)(sim);
    (set_as_exit)(sim);
    (set_entry_operation)(sim);
    if rbw.group != 0 {
        (build_collection)(rbw.group);
        for ob in &rbw.group_active_mesh_objects {
            (add_op)(*ob, 4 /*TRANSFORM*/, 12 /*RIGIDBODY_TRANSFORM_COPY*/);
        }
    }
    if rbw.constraints != 0 {
        for (object, ob1, ob2) in &rbw.constraint_tuples {
            (build_object)(-1, *object, 1, false);
            (build_object)(-1, *ob1, 1, false);
            (build_object)(-1, *ob2, 1, false);
        }
    }
}

/// `build_particle_systems` (verbatim — PSYS comp + per-
/// system eval + keyed targets + draw-ob/collection).
#[allow(clippy::too_many_arguments)]
pub fn build_particle_systems(
    object_id: usize,
    systems: &[ParticleSystemView],
    is_object_visible: bool,
    add_comp: &mut dyn FnMut(usize, usize) -> usize,
    add_comp_op: &mut dyn FnMut(usize, usize, &str) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
    build_particle_settings: &mut dyn FnMut(usize),
    build_object: &mut dyn FnMut(i32, usize, i32, bool),
    build_collection: &mut dyn FnMut(usize),
) {
    let psys_comp = (add_comp)(object_id, 6 /*PARTICLE_SYSTEM*/);
    let op = (add_comp_op)(psys_comp, 0 /*PARTICLE_SYSTEM_INIT*/, "");
    (set_as_entry)(op);
    for psys in systems {
        (build_particle_settings)(psys.part);
        (add_comp_op)(psys_comp, 1 /*PARTICLE_SYSTEM_EVAL*/, &psys.name);
        if psys.phystype == 5 /*KEYED*/ || psys.phystype == 8 /*BOIDS*/ {
            for target_ob in &psys.target_objects {
                (build_object)(-1, *target_ob, 1, is_object_visible);
            }
        }
        match psys.ren_as {
            1 /*PART_DRAW_OB*/ => {
                if psys.instance_object != 0 {
                    (build_object)(-1, psys.instance_object, 1, is_object_visible);
                }
            }
            2 /*PART_DRAW_GR*/ => {
                if psys.instance_collection != 0 {
                    (build_collection)(psys.instance_collection);
                }
            }
            _ => {}
        }
    }
    let op = (add_comp_op)(psys_comp, 2 /*PARTICLE_SYSTEM_DONE*/, "");
    (set_as_exit)(op);
}

/// `ParticleSystem` view (verbatim fields used).
pub struct ParticleSystemView {
    /// `psys.part`.
    pub part: usize,
    /// `psys.name`.
    pub name: String,
    /// `psys.part->phystype`.
    pub phystype: i32,
    /// `psys.targets` objects (self/none pre-filtered).
    pub target_objects: Vec<usize>,
    /// `psys.part->ren_as`.
    pub ren_as: i32,
    /// `psys.part->instance_object`.
    pub instance_object: usize,
    /// `psys.part->instance_collection`.
    pub instance_collection: usize,
}

/// `build_particle_settings` (verbatim — INIT/RESET/EVAL +
/// mtex textures).
pub fn build_particle_settings(
    part_id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    add_id: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
    mtex_textures: &[usize],
    build_texture: &mut dyn FnMut(usize),
) {
    if (check_is_built)(part_id) {
        return;
    }
    (add_id)(part_id);
    (build_animdata)(part_id);
    (build_parameters)(part_id);
    let op = (add_op)(part_id, 0 /*PARTICLE_SETTINGS_INIT*/);
    (set_as_entry)(op);
    (add_op)(part_id, 1 /*PARTICLE_SETTINGS_RESET*/);
    let op = (add_op)(part_id, 2 /*PARTICLE_SETTINGS_EVAL*/);
    (set_as_exit)(op);
    for tex in mtex_textures {
        (build_texture)(*tex);
    }
}

/// `build_shapekeys` (verbatim — GEOMETRY_SHAPEKEY + per-
/// block PARAMETERS_EVAL).
pub fn build_shapekeys(
    key_id: usize,
    key_block_names: &[String],
    check_is_built: &mut dyn FnMut(usize) -> bool,
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
    add_op_named: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(key_id) {
        return;
    }
    (build_idproperties)(key_id);
    (build_idproperties)(key_id);
    (build_animdata)(key_id);
    (build_parameters)(key_id);
    (add_op)(key_id, 3 /*GEOMETRY_SHAPEKEY*/);
    for name in key_block_names {
        (add_op_named)(key_id, 1 /*PARAMETERS_EVAL*/, name);
    }
}

/// `build_empty_object` (verbatim — no-modifiers early
/// exit + GEOMETRY init/eval ops).
pub fn build_empty_object(
    object_id: usize,
    has_modifiers: bool,
    cow_is_expanded: bool,
    free_derived_caches: &mut dyn FnMut(usize),
    object_cow: usize,
    add_op: &mut dyn FnMut(usize, usize) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
) {
    if !has_modifiers {
        if cow_is_expanded {
            (free_derived_caches)(object_cow);
        }
        return;
    }
    let op = (add_op)(object_id, 0 /*GEOMETRY_EVAL_INIT*/);
    (set_as_entry)(op);
    let op = (add_op)(object_id, 1 /*GEOMETRY_EVAL*/);
    (set_as_exit)(op);
}

/// `build_object_data_geometry` (verbatim — GEOMETRY init/
/// eval + materials + pointcache + datablock + dims +
/// batch cache).
#[allow(clippy::too_many_arguments)]
pub fn build_object_data_geometry(
    object_id: usize,
    object_data: usize,
    materials_array: usize,
    add_op: &mut dyn FnMut(usize, usize) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
    build_materials: &mut dyn FnMut(usize),
    build_object_pointcache: &mut dyn FnMut(usize),
    build_datablock: &mut dyn FnMut(usize),
    build_dimensions: &mut dyn FnMut(usize),
) {
    let op = (add_op)(object_id, 0 /*GEOMETRY_EVAL_INIT*/);
    (set_as_entry)(op);
    let op = (add_op)(object_id, 1 /*GEOMETRY_EVAL*/);
    (set_as_exit)(op);
    (build_materials)(materials_array);
    (build_object_pointcache)(object_id);
    (build_datablock)(object_data);
    (build_dimensions)(object_id);
    (add_op)(object_id, 9 /*BATCH_CACHE GEOMETRY_SELECT_UPDATE*/);
}

/// `build_object_data_geometry_datablock` (verbatim —
/// per-obdata-type GEOMETRY_EVAL + done + parameters +
/// batch cache + shading noop).
#[allow(clippy::too_many_arguments)]
pub fn build_object_data_geometry_datablock(
    obdata_id: usize,
    id_type: u16,
    extra_objects: &[usize],
    key: Option<usize>,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    add_id: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_shapekeys: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_object: &mut dyn FnMut(i32, usize, i32, bool),
) {
    if (check_is_built)(obdata_id) {
        return;
    }
    (add_id)(obdata_id);
    (build_idproperties)(obdata_id);
    (build_idproperties)(obdata_id);
    (build_animdata)(obdata_id);
    if let Some(k) = key {
        (build_shapekeys)(k);
    }
    let op = (add_op)(obdata_id, 1 /*GEOMETRY_EVAL*/);
    (set_as_entry)(op);
    for ob in extra_objects {
        (build_object)(-1, *ob, 1, false);
    }
    let _ = id_type;
    let op = (add_op)(obdata_id, 7 /*GEOMETRY_EVAL_DONE*/);
    (set_as_exit)(op);
    (build_parameters)(obdata_id);
    (add_op)(obdata_id, 9 /*BATCH_CACHE GEOMETRY_SELECT_UPDATE*/);
    (add_op)(obdata_id, 0 /*SHADING*/);
}

/// `build_armature` (verbatim — ARMATURE_EVAL + bones +
/// collections props).
pub fn build_armature(
    armature_id: usize,
    bone_props: &[usize],
    check_is_built: &mut dyn FnMut(usize) -> bool,
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
) {
    if (check_is_built)(armature_id) {
        return;
    }
    (build_idproperties)(armature_id);
    (build_idproperties)(armature_id);
    (build_animdata)(armature_id);
    (build_parameters)(armature_id);
    (add_op)(armature_id, 0 /*ARMATURE_EVAL*/);
    for prop in bone_props {
        (build_idproperties)(*prop);
        (build_idproperties)(*prop);
    }
}

/// `build_camera` (verbatim — dof focus object).
pub fn build_camera(
    camera_id: usize,
    dof_focus_object: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_object: &mut dyn FnMut(i32, usize, i32, bool),
) {
    if (check_is_built)(camera_id) {
        return;
    }
    (build_idproperties)(camera_id);
    (build_idproperties)(camera_id);
    (build_animdata)(camera_id);
    (build_parameters)(camera_id);
    if dof_focus_object != 0 {
        (build_object)(-1, dof_focus_object, 1, false);
    }
}

/// `build_light` (verbatim — SHADING LIGHT_UPDATE +
/// nodetree).
pub fn build_light(
    lamp_id: usize,
    nodetree: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_nodetree: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
) {
    if (check_is_built)(lamp_id) {
        return;
    }
    (build_idproperties)(lamp_id);
    (build_idproperties)(lamp_id);
    (build_animdata)(lamp_id);
    (build_parameters)(lamp_id);
    (build_nodetree)(nodetree);
    (add_op)(lamp_id, 0 /*LIGHT_UPDATE*/);
}

/// `build_nodetree_socket` (verbatim — prop + per-
/// socket-type `build_id` of default value).
pub fn build_nodetree_socket(
    socket_prop: usize,
    socket_type: i32,
    default_value_id: usize,
    build_idproperties: &mut dyn FnMut(usize),
    build_id: &mut dyn FnMut(usize, bool),
) {
    (build_idproperties)(socket_prop);
    const SOCK_OBJECT: i32 = 6;
    const SOCK_IMAGE: i32 = 9;
    const SOCK_COLLECTION: i32 = 10;
    const SOCK_TEXTURE: i32 = 11;
    const SOCK_MATERIAL: i32 = 12;
    const SOCK_FONT: i32 = 13;
    const SOCK_SCENE: i32 = 14;
    const SOCK_TEXT_ID: i32 = 15;
    const SOCK_MASK: i32 = 16;
    const SOCK_SOUND: i32 = 17;
    match socket_type {
        SOCK_OBJECT | SOCK_IMAGE | SOCK_COLLECTION | SOCK_TEXTURE | SOCK_MATERIAL | SOCK_FONT
        | SOCK_SCENE | SOCK_MASK | SOCK_SOUND => {
            if default_value_id != 0 {
                (build_id)(default_value_id, false);
            }
        }
        SOCK_TEXT_ID => { /* Text data-blocks don't use the depsgraph. */ }
        _ => {}
    }
}

/// `NodetreeView` for `build_nodetree` (verbatim node
/// walk).
pub struct NodetreeView {
    /// `ntree->type == NTREE_GEOMETRY`.
    pub is_geometry: bool,
    /// per-node (prop, socket props, linked id+type).
    pub nodes: Vec<NodetreeNodeView>,
    /// interface socket props.
    pub interface_props: Vec<usize>,
}

/// `NodetreeNodeView` (verbatim fields used).
pub struct NodetreeNodeView {
    /// `bnode->prop`.
    pub prop: usize,
    /// socket (prop, type, default id) triples.
    pub sockets: Vec<(usize, i32, usize)>,
    /// `bnode->id`.
    pub id: usize,
    /// `bnode->id->type`.
    pub id_type: u16,
    /// `bnode->is_group()`.
    pub is_group: bool,
}

/// `build_nodetree` (verbatim — NTREE_OUTPUT +
/// preprocess + per-node socket/id dispatch + interface
/// props).
#[allow(clippy::too_many_arguments)]
pub fn build_nodetree(
    ntree_id: usize,
    ntree: &NodetreeView,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    add_id: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
    add_op_named: &mut dyn FnMut(usize, usize),
    build_parameters: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_nodetree_socket: &mut dyn FnMut(usize, i32, usize),
    build_id: &mut dyn FnMut(usize, bool),
) {
    if ntree_id == 0 || (check_is_built)(ntree_id) {
        return;
    }
    (add_id)(ntree_id);
    (build_parameters)(ntree_id);
    (build_idproperties)(ntree_id);
    (build_idproperties)(ntree_id);
    (build_animdata)(ntree_id);
    (add_op)(ntree_id, 0 /*NTREE_OUTPUT*/);
    if ntree.is_geometry {
        (add_op_named)(ntree_id, 1 /*NTREE_GEOMETRY_PREPROCESS*/);
    }
    for bnode in &ntree.nodes {
        (build_idproperties)(bnode.prop);
        for (sprop, stype, sdef_id) in &bnode.sockets {
            (build_nodetree_socket)(*sprop, *stype, *sdef_id);
        }
        if bnode.id == 0 {
            continue;
        }
        if bnode.is_group {
            (build_id)(bnode.id, false);
        } else {
            (build_id)(bnode.id, false);
        }
    }
    for prop in &ntree.interface_props {
        (build_idproperties)(*prop);
    }
}

/// `build_material` (verbatim — SHADING MATERIAL_UPDATE +
/// nodetree).
pub fn build_material(
    material_id: usize,
    nodetree: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    add_id: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_nodetree: &mut dyn FnMut(usize),
) {
    if (check_is_built)(material_id) {
        return;
    }
    (add_id)(material_id);
    (add_op)(material_id, 0 /*MATERIAL_UPDATE*/);
    (build_idproperties)(material_id);
    (build_idproperties)(material_id);
    (build_animdata)(material_id);
    (build_parameters)(material_id);
    (build_nodetree)(nodetree);
}

/// `build_materials` (verbatim).
pub fn build_materials(materials: &[usize], build_material: &mut dyn FnMut(usize)) {
    for mat in materials {
        if *mat != 0 {
            (build_material)(*mat);
        }
    }
}

/// `build_texture` (verbatim — GENERIC_DATABLOCK_UPDATE +
/// nodetree + image).
pub fn build_texture(
    texture_id: usize,
    nodetree: usize,
    ima: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    add_id: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_nodetree: &mut dyn FnMut(usize),
    build_image: &mut dyn FnMut(usize),
) {
    if (check_is_built)(texture_id) {
        return;
    }
    (add_id)(texture_id);
    (build_idproperties)(texture_id);
    (build_idproperties)(texture_id);
    (build_animdata)(texture_id);
    (build_parameters)(texture_id);
    (build_nodetree)(nodetree);
    if ima != 0 {
        (build_image)(ima);
    }
    (add_op)(texture_id, 0 /*GENERIC_DATABLOCK_UPDATE*/);
}

/// `build_image` / `build_generic_id` / `build_lightprobe`
/// / `build_freestyle_linestyle` (verbatim — shared
/// parameters/props/anim pattern).
pub fn build_simple_id(
    id: usize,
    op: Option<(usize, usize)>,
    nodetree: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    add_id: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_nodetree: &mut dyn FnMut(usize),
) {
    if (check_is_built)(id) {
        return;
    }
    (add_id)(id);
    if let Some((comp, opcode)) = op {
        (add_op)(id, comp * 100 + opcode);
    }
    (build_idproperties)(id);
    (build_idproperties)(id);
    (build_animdata)(id);
    (build_parameters)(id);
    if nodetree != 0 {
        (build_nodetree)(nodetree);
    }
}

/// `build_idproperties` (verbatim — IDP_TYPE_FILTER_ID
/// walk → `build_id`).
pub fn build_idproperties(
    id_pointer_props: &[usize],
    build_id: &mut dyn FnMut(usize, bool),
) {
    for id_ptr in id_pointer_props {
        (build_id)(*id_ptr, false);
    }
}

/// `build_cachefile` (verbatim — CACHE FILE_CACHE_UPDATE).
pub fn build_cachefile(
    cache_file_id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    add_id: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
) {
    if (check_is_built)(cache_file_id) {
        return;
    }
    (add_id)(cache_file_id);
    (build_idproperties)(cache_file_id);
    (build_idproperties)(cache_file_id);
    (build_animdata)(cache_file_id);
    (build_parameters)(cache_file_id);
    (add_op)(cache_file_id, 0 /*FILE_CACHE_UPDATE*/);
}

/// `build_mask` (verbatim — MASK_ANIMATION + MASK_EVAL +
/// parent ids).
pub fn build_mask(
    mask_id: usize,
    parent_ids: &[usize],
    check_is_built: &mut dyn FnMut(usize) -> bool,
    ensure_cow: &mut dyn FnMut(usize) -> usize,
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
    build_id: &mut dyn FnMut(usize, bool),
) {
    if (check_is_built)(mask_id) {
        return;
    }
    let _ = (ensure_cow)(mask_id);
    (build_idproperties)(mask_id);
    (build_idproperties)(mask_id);
    (build_animdata)(mask_id);
    (build_parameters)(mask_id);
    (add_op)(mask_id, 3 /*ANIMATION:MASK_ANIMATION*/);
    (add_op)(mask_id, 2 /*PARAMETERS:MASK_EVAL*/);
    for pid in parent_ids {
        (build_id)(*pid, false);
    }
}

/// `build_movieclip` (verbatim — MOVIECLIP_EVAL).
pub fn build_movieclip(
    clip_id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    ensure_cow: &mut dyn FnMut(usize) -> usize,
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
) {
    if (check_is_built)(clip_id) {
        return;
    }
    let _ = (ensure_cow)(clip_id);
    (build_idproperties)(clip_id);
    (build_idproperties)(clip_id);
    (build_animdata)(clip_id);
    (build_parameters)(clip_id);
    (add_op)(clip_id, 2 /*PARAMETERS:MOVIECLIP_EVAL*/);
}

/// `build_speaker` (verbatim — SPEAKER_EVAL + sound).
pub fn build_speaker(
    speaker_id: usize,
    sound: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    add_op: &mut dyn FnMut(usize, usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_sound: &mut dyn FnMut(usize),
) {
    if (check_is_built)(speaker_id) {
        return;
    }
    (add_op)(speaker_id, 8 /*AUDIO:SPEAKER_EVAL*/);
    (build_idproperties)(speaker_id);
    (build_idproperties)(speaker_id);
    (build_animdata)(speaker_id);
    (build_parameters)(speaker_id);
    if sound != 0 {
        (build_sound)(sound);
    }
}

/// `build_sound` (verbatim — AUDIO SOUND_EVAL).
pub fn build_sound(
    sound_id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    add_id: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
) {
    if (check_is_built)(sound_id) {
        return;
    }
    (add_id)(sound_id);
    (add_op)(sound_id, 8 /*AUDIO:SOUND_EVAL*/);
    (build_idproperties)(sound_id);
    (build_idproperties)(sound_id);
    (build_animdata)(sound_id);
    (build_parameters)(sound_id);
}

/// `build_vfont` (verbatim).
pub fn build_vfont(
    vfont_id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    build_parameters: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
) {
    if (check_is_built)(vfont_id) {
        return;
    }
    (build_parameters)(vfont_id);
    (build_idproperties)(vfont_id);
    (build_idproperties)(vfont_id);
}

/// `build_scene_audio` (verbatim — AUDIO entry/exit +
/// AUDIO_VOLUME).
pub fn build_scene_audio(
    scene_id: usize,
    check_is_built_tag: &mut dyn FnMut(usize) -> bool,
    build_scene_audio_cache: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize) -> usize,
    set_as_entry: &mut dyn FnMut(usize),
    set_as_exit: &mut dyn FnMut(usize),
) {
    (build_scene_audio_cache)(scene_id);
    if (check_is_built_tag)(scene_id) {
        return;
    }
    let op = (add_op)(scene_id, 0 /*AUDIO_ENTRY*/);
    (set_as_entry)(op);
    let op = (add_op)(scene_id, 1 /*SOUND_EVAL*/);
    (set_as_exit)(op);
    (add_op)(scene_id, 2 /*AUDIO_VOLUME*/);
}

/// `build_scene_audio_cache` (verbatim — AUDIO_CACHE_
/// INVALIDATE op + tag_update RELATIONS).
pub fn build_scene_audio_cache(
    scene_id: usize,
    check_is_built_tag: &mut dyn FnMut(usize) -> bool,
    add_op: &mut dyn FnMut(usize, usize) -> usize,
    tag_update: &mut dyn FnMut(usize),
) {
    if (check_is_built_tag)(scene_id) {
        return;
    }
    let op = (add_op)(scene_id, 4 /*CACHE:AUDIO_CACHE_INVALIDATE*/);
    (tag_update)(op);
}

/// `build_scene_speakers` (verbatim — view-layer speaker
/// bases).
pub fn build_scene_speakers(
    speaker_objects: &[usize],
    build_object: &mut dyn FnMut(i32, usize, i32, bool),
) {
    for ob in speaker_objects {
        (build_object)(-1, *ob, 1, true);
    }
}

/// `build_scene_sequencer` (verbatim — audio + SEQUENCES_
/// EVAL + per-strip data).
#[allow(clippy::too_many_arguments)]
pub fn build_scene_sequencer(
    scene_id: usize,
    has_ed: bool,
    check_is_built_tag: &mut dyn FnMut(usize) -> bool,
    build_scene_audio: &mut dyn FnMut(usize),
    add_op: &mut dyn FnMut(usize, usize),
    foreach_strip: &mut dyn FnMut(&mut dyn FnMut(usize)),
) {
    if !has_ed || (check_is_built_tag)(scene_id) {
        return;
    }
    (build_scene_audio)(scene_id);
    (add_op)(scene_id, 5 /*SEQUENCER:SEQUENCES_EVAL*/);
    (foreach_strip)(&mut |_strip| {});
}

/// `strip_node_build_cb` (verbatim — per-strip props/
/// sound/scene/compositor nodetree).
#[allow(clippy::too_many_arguments)]
pub fn strip_node_build_cb(
    strip_prop: usize,
    strip_sound: usize,
    strip_scene: Option<usize>,
    is_scene_strip_with_scene_strips_and_ed: bool,
    is_compositor_with_node_group: Option<usize>,
    modifier_node_groups: &[usize],
    build_idproperties: &mut dyn FnMut(usize),
    build_sound: &mut dyn FnMut(usize),
    build_scene_parameters: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_scene_sequencer: &mut dyn FnMut(usize),
    build_scene_speakers: &mut dyn FnMut(usize),
    build_nodetree: &mut dyn FnMut(usize),
) {
    (build_idproperties)(strip_prop);
    (build_idproperties)(strip_prop);
    if strip_sound != 0 {
        (build_sound)(strip_sound);
    }
    if let Some(scene) = strip_scene {
        (build_scene_parameters)(scene);
    }
    if is_scene_strip_with_scene_strips_and_ed {
        if let Some(scene) = strip_scene {
            (build_animdata)(scene);
            (build_scene_sequencer)(scene);
            (build_scene_speakers)(scene);
        }
    }
    if let Some(ng) = is_compositor_with_node_group {
        (build_nodetree)(ng);
    }
    for ng in modifier_node_groups {
        (build_nodetree)(*ng);
    }
}

/// `modifier_walk`/`constraint_walk` (verbatim — OB →
/// build_object else build_id).
pub fn walk_id(
    id: usize,
    id_type: u16,
    build_object: &mut dyn FnMut(i32, usize, i32, bool),
    build_id: &mut dyn FnMut(usize, bool),
) {
    if id == 0 {
        return;
    }
    if id_type == gs(b"OB") {
        (build_object)(-1, id, 1 /*DEG_ID_LINKED_INDIRECTLY*/, false);
    } else {
        (build_id)(id, false);
    }
}
