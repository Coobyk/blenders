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
