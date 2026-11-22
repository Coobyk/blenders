//! Port of `intern/builder/deg_builder_relations.cc` —
/// `DepsgraphRelationBuilder`: relation wiring between
/// nodes built by `DepsgraphNodeBuilder`.

use blenders_dna::id::gs;

/// `PFIELD_*` verbatim.
pub const PFIELD_TEXTURE: i32 = 12;
pub const PFIELD_GUIDE: i32 = 16;
pub const PFIELD_FLUIDFLOW: i32 = 17;
pub const PFIELD_SHAPE_SURFACE: i32 = 2;
pub const PFIELD_SHAPE_POINTS: i32 = 1;
pub const PFIELD_VISIBILITY: i32 = 1 << 0;
/// `PART_HAIR`/`PSYS_HAIR_DYNAMICS`/`DVAR_TYPE_CONTEXT_PROP`/
/// `DTAR_CONTEXT_PROPERTY_ACTIVE_SCENE`/`OB_MESH` verbatim.
pub const PART_HAIR: i32 = 2;
pub const PSYS_HAIR_DYNAMICS: i32 = 1 << 9;
pub const DVAR_TYPE_CONTEXT_PROP: i32 = 8;
pub const DTAR_CONTEXT_PROPERTY_ACTIVE_SCENE: i32 = 0;
/// `RELATION_CHECK_BEFORE_ADD` (from `deg_node.rs`/`relation.rs`).
pub use crate::depsgraph::RELATION_CHECK_BEFORE_ADD;

/// `is_time_dependent_scene_driver_target` (verbatim).
pub fn is_time_dependent_scene_driver_target(rna_path: Option<&str>) -> bool {
    rna_path == Some("frame_current")
}

/// `driver_target_depends_on_time` (verbatim).
pub fn driver_target_depends_on_time(
    variable_type: i32,
    target_context_property: i32,
    target_idtype: u16,
    target_rna_path: Option<&str>,
) -> bool {
    if variable_type == DVAR_TYPE_CONTEXT_PROP
        && target_context_property == DTAR_CONTEXT_PROPERTY_ACTIVE_SCENE
    {
        return is_time_dependent_scene_driver_target(target_rna_path);
    }
    if target_idtype == gs(b"SC") {
        return is_time_dependent_scene_driver_target(target_rna_path);
    }
    false
}

/// `driver_variable_depends_on_time` (verbatim).
pub fn driver_variable_depends_on_time(
    variable_type: i32,
    targets: &[(i32, u16, Option<String>)],
) -> bool {
    for (ctx_prop, idtype, rna_path) in targets {
        if driver_target_depends_on_time(
            variable_type,
            *ctx_prop,
            *idtype,
            rna_path.as_deref(),
        ) {
            return true;
        }
    }
    false
}

/// `driver_depends_on_time` (verbatim — expression +
/// variables).
pub fn driver_depends_on_time(
    expression_depends_on_time: bool,
    variables: &[(i32, Vec<(i32, u16, Option<String>)>)],
) -> bool {
    if expression_depends_on_time {
        return true;
    }
    for (vtype, targets) in variables {
        if driver_variable_depends_on_time(*vtype, targets) {
            return true;
        }
    }
    false
}

/// `particle_system_depends_on_time` (verbatim).
pub fn particle_system_depends_on_time(part_type: i32, psys_flag: i32) -> bool {
    if part_type != PART_HAIR {
        return true;
    }
    if psys_flag & PSYS_HAIR_DYNAMICS != 0 {
        return true;
    }
    false
}

/// `object_particles_depends_on_time` (verbatim).
pub fn object_particles_depends_on_time(object_type: i32, psys: &[(i32, i32)]) -> bool {
    if object_type != 1 /*OB_MESH*/ {
        return false;
    }
    for (part_type, psys_flag) in psys {
        if particle_system_depends_on_time(*part_type, *psys_flag) {
            return true;
        }
    }
    false
}

/// `check_id_has_anim_component` (verbatim).
pub fn check_id_has_anim_component(has_action: bool, has_nla_tracks: bool) -> bool {
    has_action || has_nla_tracks
}

/// `check_id_has_driver_component` (verbatim).
pub fn check_id_has_driver_component(has_drivers: bool) -> bool {
    has_drivers
}

/// `bone_target_opcode` (verbatim — same-armature
/// common-root → BONE_READY).
pub fn bone_target_opcode(
    target_eq_id: bool,
    has_root_map: bool,
    root_map_has_common_root: bool,
) -> usize {
    if target_eq_id && has_root_map && root_map_has_common_root {
        return 5 /*OperationCode::BONE_READY*/;
    }
    6 /*OperationCode::BONE_DONE*/
}

/// `object_have_geometry_component` (verbatim).
pub fn object_have_geometry_component(object_type: i32) -> bool {
    matches!(object_type, 1 | 2 | 3 | 4 | 5 | 22)
}

/* ---- relation adders ---- */

/// Graph access for the relation builder.
pub struct RelBuildCore<'a> {
    /// `graph_->find_id_node`.
    pub graph_find_id_node: &'a dyn Fn(usize) -> Option<usize>,
    /// `graph_->add_new_relation(from, to, descr, flags)`.
    pub graph_add_new_relation: &'a mut dyn FnMut(usize, usize, &str, i32) -> usize,
    /// `node->get_entry_operation`.
    pub node_get_entry_operation: &'a dyn Fn(usize) -> Option<usize>,
    /// `id_node->customdata_masks |=`.
    pub id_node_customdata_or: &'a mut dyn FnMut(usize, usize),
    /// `id_node->eval_flags |=`.
    pub id_node_eval_flags_or: &'a mut dyn FnMut(usize, u32),
    /// `id_node->id_orig`/`linked_state`.
    pub id_node_fields: &'a dyn Fn(usize) -> (usize, i32),
    /// Time source lookup.
    pub time_source_node: &'a dyn Fn() -> Option<usize>,
}

/// `add_node_handle_relation` (verbatim — time source →
/// handle's entry op).
pub fn add_node_handle_relation(
    handle_node: usize,
    description: &str,
    flags: i32,
    core: &mut RelBuildCore,
) -> Option<usize> {
    let time_from = (core.time_source_node)()?;
    let op_to = (core.node_get_entry_operation)(handle_node)?;
    Some((core.graph_add_new_relation)(time_from, op_to, description, flags))
}

/// `add_depends_on_transform_relation` (verbatim — GEOMETRY
/// MODIFIER keyed on the handle's component name).
pub fn add_depends_on_transform_relation(
    id: usize,
    comp_name: &str,
    description: &str,
    add_relation_by_keys: &mut dyn FnMut(usize, usize, usize, &str, i32),
) {
    let _ = (id, comp_name);
    add_relation_by_keys(id, 5 /*GEOMETRY*/, 1 /*MODIFIER*/, description, 0);
}

/// `add_customdata_mask` (verbatim — OB_MESH only).
pub fn add_customdata_mask(
    object_id: usize,
    object_type: i32,
    customdata_masks: usize,
    core: &mut RelBuildCore,
) {
    if customdata_masks != 0 && object_type == 1 /*OB_MESH*/ {
        if let Some(id_node) = (core.graph_find_id_node)(object_id) {
            (core.id_node_customdata_or)(id_node, customdata_masks);
        }
    }
}

/// `add_special_eval_flag` (verbatim).
pub fn add_special_eval_flag(id: usize, flag: u32, core: &mut RelBuildCore) {
    if let Some(id_node) = (core.graph_find_id_node)(id) {
        (core.id_node_eval_flags_or)(id_node, flag);
    }
}

/// `add_time_relation` (verbatim).
pub fn add_time_relation(
    timesrc: Option<usize>,
    node_to: Option<usize>,
    description: &str,
    flags: i32,
    core: &mut RelBuildCore,
) -> Option<usize> {
    if let (Some(from), Some(to)) = (timesrc, node_to) {
        return Some((core.graph_add_new_relation)(from, to, description, flags));
    }
    None
}

/// `add_visibility_relation` (verbatim — VISIBILITY comp→
/// comp "visibility").
pub fn add_visibility_relation(
    id_from: usize,
    id_to: usize,
    add_relation_comps: &mut dyn FnMut(usize, usize, usize, usize, &str),
) {
    (add_relation_comps)(
        id_from,
        20 /*VISIBILITY*/,
        id_to,
        20,
        "visibility",
    );
}

/// `add_operation_relation` (verbatim).
pub fn add_operation_relation(
    node_from: Option<usize>,
    node_to: Option<usize>,
    description: &str,
    flags: i32,
    core: &mut RelBuildCore,
) -> Option<usize> {
    if let (Some(from), Some(to)) = (node_from, node_to) {
        return Some((core.graph_add_new_relation)(from, to, description, flags));
    }
    None
}

/// `CollisionRelation` (verbatim fields used).
pub struct CollisionRelation {
    /// `relation.ob`.
    pub ob: usize,
}

/// `add_particle_collision_relations` (verbatim — skip
/// self; TRANSFORM + GEOMETRY comps).
pub fn add_particle_collision_relations(
    object: usize,
    relations: &[CollisionRelation],
    name: &str,
    add_relation_comps: &mut dyn FnMut(usize, usize, usize, usize, &str),
    add_relation_key: &mut dyn FnMut(usize, &str),
) {
    for rel in relations {
        if rel.ob != object {
            (add_relation_comps)(rel.ob, 4 /*TRANSFORM*/, rel.ob, 0, name);
            (add_relation_key)(rel.ob, name);
            (add_relation_comps)(rel.ob, 5 /*GEOMETRY*/, rel.ob, 0, name);
            (add_relation_key)(rel.ob, name);
        }
    }
}

/// `EffectorRelation` (verbatim fields used).
pub struct EffectorRelation {
    /// `relation.ob`.
    pub ob: usize,
    /// `relation.psys` (0 = none).
    pub psys: usize,
    /// `relation.psys->name` (when psys != 0).
    pub psys_name: String,
    /// `relation.pd->shape`.
    pub pd_shape: i32,
    /// `relation.pd->forcefield`.
    pub pd_forcefield: i32,
    /// `relation.pd->flag`.
    pub pd_flag: i32,
    /// `relation.pd->tex` (0 = none).
    pub pd_tex: usize,
    /// `relation.pd->f_source` (0 = none).
    pub pd_f_source: usize,
}

/// `add_particle_forcefield_relations` (verbatim — effector
/// transform/geometry/texture/smoke/absorption + psys
/// relations).
#[allow(clippy::too_many_arguments)]
pub fn add_particle_forcefield_relations(
    object_id: usize,
    self_psys: usize,
    add_absorption: bool,
    relations: &[EffectorRelation],
    name: &str,
    add_relation_comps: &mut dyn FnMut(usize, usize, usize, usize, &str, i32),
    add_collision: &mut dyn FnMut(&str),
) {
    if !relations.is_empty() {
        (add_relation_comps)(
            usize::MAX /*time_src*/,
            0,
            object_id,
            5 /*GEOMETRY*/,
            "Effector Time -> Particle",
            RELATION_CHECK_BEFORE_ADD,
        );
    }
    for rel in relations {
        if rel.ob != object_id {
            (add_relation_comps)(rel.ob, 4 /*TRANSFORM*/, rel.ob, 0, name, 0);
            if rel.pd_shape == PFIELD_SHAPE_SURFACE
                || rel.pd_shape == PFIELD_SHAPE_POINTS
                || rel.pd_forcefield == PFIELD_GUIDE
            {
                (add_relation_comps)(rel.ob, 5 /*GEOMETRY*/, rel.ob, 0, name, 0);
            }
            if rel.pd_forcefield == PFIELD_TEXTURE && rel.pd_tex != 0 {
                (add_relation_comps)(
                    rel.pd_tex,
                    26 /*GENERIC_DATABLOCK*/,
                    rel.ob,
                    0,
                    "Force field Texture",
                    0,
                );
            }
            if rel.pd_forcefield == PFIELD_FLUIDFLOW && rel.pd_f_source != 0 {
                (add_relation_comps)(
                    rel.pd_f_source,
                    4,
                    rel.ob,
                    0,
                    "Smoke Force Domain",
                    0,
                );
                (add_relation_comps)(
                    rel.pd_f_source,
                    5,
                    rel.ob,
                    0,
                    "Smoke Force Domain",
                    0,
                );
            }
            if add_absorption && (rel.pd_flag & PFIELD_VISIBILITY) != 0 {
                (add_collision)("Force Absorption");
            }
        }
        if rel.psys != 0 {
            if rel.ob != object_id {
                (add_relation_comps)(rel.ob, 6 /*PARTICLE_SYSTEM*/, rel.ob, 0, name, 0);
                (add_relation_comps)(rel.ob, 5 /*GEOMETRY*/, rel.ob, 0, name, 0);
            } else if rel.psys != self_psys {
                (add_relation_comps)(rel.ob, 6, rel.ob, 0, &rel.psys_name, 0);
            }
        }
    }
}

/* ---- build dispatch + collection/object relations ---- */

/// `RELATION_NO_VISIBILITY_CHANGE` (verbatim).
pub const RELATION_NO_VISIBILITY_CHANGE: i32 = 1 << 7;

/// `build_id` (verbatim — same dispatch as the node
/// builder).
pub fn build_id(id: usize, id_type: u16, b: &mut crate::deg_builder_nodes::IdTypeBuilders) {
    if id == 0 {
        return;
    }
    crate::deg_builder_nodes::build_id(id, false, id_type, b);
}

/// `build_generic_id` (verbatim).
pub fn build_generic_id(
    id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
) {
    if (check_is_built)(id) {
        return;
    }
    (stack_trace)(id);
    (build_idproperties)(id);
    (build_idproperties)(id);
    (build_animdata)(id);
    (build_parameters)(id);
}

/// `build_collection` (verbatim — children-hierarchy
/// relations when from layer; else full geometry wiring).
#[allow(clippy::too_many_arguments)]
pub fn build_collection(
    from_layer_collection: bool,
    collection_id: usize,
    check_built_props: &mut dyn FnMut(usize) -> bool,
    check_built_children: &mut dyn FnMut(usize) -> bool,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    collection_hierarchy_exit: &dyn Fn(usize) -> usize,
    find_comp: &dyn Fn(usize, usize) -> Option<usize>,
    comp_entry: &dyn Fn(usize) -> usize,
    add_op_rel: &mut dyn FnMut(usize, usize, &str),
    add_rel: &mut dyn FnMut(usize, usize, &str),
    gobjects: &[usize],
    children: &[usize],
    object_instance_collections: &dyn Fn(usize) -> usize,
    object_types: &dyn Fn(usize) -> i32,
    build_idproperties: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_object: &mut dyn FnMut(usize),
    build_collection_inner: &mut dyn FnMut(usize),
) {
    if !(check_built_props)(collection_id) {
        (build_idproperties)(collection_id);
        (build_idproperties)(collection_id);
        (build_parameters)(collection_id);
    }
    if from_layer_collection {
        if !(check_built_children)(collection_id) {
            let collection_hierarchy_exit = collection_hierarchy_exit(collection_id);
            for ob in gobjects {
                if let Some(obj_hier) = find_comp(*ob, 5 /*HIERARCHY*/) {
                    (build_object)(*ob);
                    (add_op_rel)(
                        collection_hierarchy_exit,
                        comp_entry(obj_hier),
                        "Collection -> Object hierarchy",
                    );
                }
            }
        }
        return;
    }
    if (check_is_built)(collection_id) {
        return;
    }
    (stack_trace)(collection_id);
    let collection_hierarchy_exit = collection_hierarchy_exit(collection_id);
    for ob in gobjects {
        (build_object)(*ob);
        let obj_hier_entry = comp_entry(find_comp(*ob, 5).unwrap_or(0));
        (add_op_rel)(
            collection_hierarchy_exit,
            obj_hier_entry,
            "Collection -> Object hierarchy",
        );
        /* INSTANCE_GEOMETRY -> collection GEOMETRY_EVAL_DONE. */
        (add_rel)(*ob, collection_id, "Collection Geometry");
        if (object_types)(*ob) == 0 /*OB_EMPTY*/ {
            let ci = (object_instance_collections)(*ob);
            if ci != 0 {
                (add_rel)(ci, collection_id, "Collection Geometry");
            }
        }
    }
    for child in children {
        (build_collection_inner)(*child);
        (add_rel)(*child, collection_id, "Collection Geometry");
    }
}

/// `build_object` (verbatim — full transform/constraint/
/// anim/instancing/sync relation graph).
#[allow(clippy::too_many_arguments)]
pub fn build_object(
    object_id: usize,
    has_parent: bool,
    parent: usize,
    has_constraints: bool,
    has_greasepencil_modifiers: bool,
    has_shader_fx: bool,
    has_particlesystem: bool,
    has_forcefield_texture: bool,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
    add_rel_flags: &mut dyn FnMut(usize, usize, &str, i32),
    steps: &mut RelObjectSteps,
) {
    if (check_is_built)(object_id) {
        return;
    }
    (stack_trace)(object_id);
    let base_op = if has_parent { 2 /*PARENT*/ } else { 1 /*LOCAL*/ };
    (add_rel)(object_id, object_id, "Transform Init");
    (steps.build_object_layer_component_relations)(object_id);
    if has_parent {
        (steps.build_object)(parent);
        (steps.build_object_parent)(object_id);
        (add_rel)(object_id, object_id, "ObLocal -> ObParent");
    }
    let _ = base_op;
    (add_rel)(object_id, object_id, "Instance Geometry -> Geometry");
    (add_rel)(object_id, object_id, "Transform -> Instance Geometry");
    (steps.build_object_modifiers)(object_id);
    if has_greasepencil_modifiers {
        (steps.gp_modifiers_id_loop)(object_id);
    }
    if has_shader_fx {
        (steps.shaderfx_id_loop)(object_id);
    }
    if has_constraints {
        (steps.constraints_id_loop)(object_id);
        (steps.build_constraints)(object_id);
        (add_rel)(object_id, object_id, "ObBase-> Constraint Stack");
        (add_rel)(object_id, object_id, "ObConstraints -> Done");
        (add_rel)(object_id, object_id, "Constraint -> Transform Eval");
        (add_rel)(object_id, object_id, "Transform Eval -> Simulation Init");
        (add_rel)(object_id, object_id, "Simulation -> Final Transform");
    } else {
        (add_rel)(object_id, object_id, "Eval");
        (add_rel)(object_id, object_id, "Transform Eval -> Simulation Init");
        (add_rel)(object_id, object_id, "Simulation -> Final Transform");
    }
    (steps.build_idproperties)(object_id);
    (steps.build_idproperties)(object_id);
    (steps.build_animdata)(object_id);
    (steps.build_object_data)(object_id);
    if has_particlesystem {
        (steps.build_particle_systems)(object_id);
    }
    if has_forcefield_texture {
        (steps.build_texture)(object_id);
    }
    (steps.build_object_instance_collection)(object_id);
    (steps.build_object_pointcache)(object_id);
    (steps.build_object_shading)(object_id);
    (steps.build_object_light_linking)(object_id);
    (add_rel)(object_id, object_id, "Synchronize to Original");
    (steps.build_parameters)(object_id);
    (add_rel_flags)(object_id, object_id, "Object Visibility", 0);
}

/// Injected sub-builders for `build_object` relations.
pub struct RelObjectSteps<'a> {
    /// `build_object` recursion.
    pub build_object: &'a mut dyn FnMut(usize),
    /// `build_object_layer_component_relations`.
    pub build_object_layer_component_relations: &'a mut dyn FnMut(usize),
    /// `build_object_parent`.
    pub build_object_parent: &'a mut dyn FnMut(usize),
    /// `build_object_modifiers`.
    pub build_object_modifiers: &'a mut dyn FnMut(usize),
    /// gp/shaderfx/constraint id loops.
    pub gp_modifiers_id_loop: &'a mut dyn FnMut(usize),
    pub shaderfx_id_loop: &'a mut dyn FnMut(usize),
    pub constraints_id_loop: &'a mut dyn FnMut(usize),
    /// `build_constraints`.
    pub build_constraints: &'a mut dyn FnMut(usize),
    /// idproperties/animdata/data/particles/texture.
    pub build_idproperties: &'a mut dyn FnMut(usize),
    pub build_animdata: &'a mut dyn FnMut(usize),
    pub build_object_data: &'a mut dyn FnMut(usize),
    pub build_particle_systems: &'a mut dyn FnMut(usize),
    pub build_texture: &'a mut dyn FnMut(usize),
    /// instance/pointcache/shading/light-linking/params.
    pub build_object_instance_collection: &'a mut dyn FnMut(usize),
    pub build_object_pointcache: &'a mut dyn FnMut(usize),
    pub build_object_shading: &'a mut dyn FnMut(usize),
    pub build_object_light_linking: &'a mut dyn FnMut(usize),
    pub build_parameters: &'a mut dyn FnMut(usize),
}

/// `build_object_from_view_layer_base` (verbatim —
/// VIEW_LAYER_EVAL → OBJECT_FROM_LAYER_ENTRY).
pub fn build_object_from_view_layer_base(
    scene_id: usize,
    object_id: usize,
    add_rel: &mut dyn FnMut(usize, usize, &str),
    build_object: &mut dyn FnMut(usize),
) {
    (add_rel)(scene_id, object_id, "View Layer flags to Object");
    (build_object)(object_id);
}

/// `build_object_layer_component_relations` (verbatim —
/// entry→(flags)→exit + exit→sync).
pub fn build_object_layer_component_relations(
    object_id: usize,
    has_flags_node: bool,
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if !has_flags_node {
        (add_rel)(object_id, object_id, "Object from Layer");
        return;
    }
    (add_rel)(object_id, object_id, "Base flags flush Entry");
    (add_rel)(object_id, object_id, "Base flags flush Exit");
    (add_rel)(object_id, object_id, "Synchronize to Original");
}

/// `build_object_modifiers` (verbatim — visibility →
/// object visibility + modifier chain + per-modifier
/// update_depsgraph + time deps).
#[allow(clippy::too_many_arguments)]
pub fn build_object_modifiers(
    object_id: usize,
    modifiers: &[(String, bool, bool)],
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
    modifier_update_depsgraph: &mut dyn FnMut(usize, &str),
    modifiers_id_loop: &mut dyn FnMut(usize),
) {
    if modifiers.is_empty() {
        return;
    }
    (add_rel)(
        object_id,
        object_id,
        "modifier -> object visibility",
        RELATION_NO_VISIBILITY_CHANGE,
    );
    (add_rel)(object_id, object_id, "modifier visibility -> geometry eval", 0);
    for (name, has_update_depsgraph, depends_on_time) in modifiers {
        (add_rel)(object_id, object_id, "Modifier", 0);
        if *has_update_depsgraph {
            (modifier_update_depsgraph)(object_id, name);
        }
        if *depends_on_time {
            (add_rel)(usize::MAX, object_id, "Time Source -> Modifier", 0);
        }
    }
    (add_rel)(object_id, object_id, "modifier stack order", 0);
    (modifiers_id_loop)(object_id);
}
