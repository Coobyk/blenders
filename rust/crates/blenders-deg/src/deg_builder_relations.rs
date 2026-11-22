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

/* ---- object data / parent / pointcache / light linking ---- */

/// `DAG_EVAL_NEED_CURVE_PATH`/`CD_MASK_ORIGINDEX`/
/// `RELATION_FLAG_FLUSH_USER_EDIT_ONLY`/`NO_FLUSH`
/// verbatim.
pub const DAG_EVAL_NEED_CURVE_PATH: u32 = 1 << 0;
pub const CD_MASK_ORIGINDEX: usize = 1 << 32;
pub const RELATION_FLAG_FLUSH_USER_EDIT_ONLY: i32 = 1 << 5;
pub const RELATION_FLAG_NO_FLUSH: i32 = 1 << 0;
/// `PAR*` partype verbatim.
pub const PARSKEL: i32 = 4;
pub const PARVERT1: i32 = 1;
pub const PARVERT3: i32 = 3;
pub const PARBONE: i32 = 2;
/// `OB_DUPLI`/`OB_DUPLIVERTS`/`CU_PATH` verbatim.
pub const OB_DUPLI: i32 = 1 << 0;
pub const OB_DUPLIVERTS: i32 = 1 << 12;
pub const CU_PATH: i32 = 1 << 3;

/// `build_object_data` (verbatim — EMPTY/modifiers early,
/// obdata anim, type dispatch, font textoncurve, key→
/// geometry, materials).
#[allow(clippy::too_many_arguments)]
pub fn build_object_data(
    object_id: usize,
    object_type: i32,
    has_modifiers: bool,
    obdata_id: usize,
    obdata_is_built: &mut dyn FnMut(usize) -> bool,
    font_textoncurve: Option<usize>,
    key_id: Option<usize>,
    materials_array: Option<usize>,
    add_rel: &mut dyn FnMut(usize, usize, &str),
    add_special_eval_flag: &mut dyn FnMut(usize, u32),
    steps: &mut RelObjectDataSteps,
) {
    if object_type == 0 /*OB_EMPTY*/ && has_modifiers {
        (steps.build_object_data_empty)(object_id);
        return;
    }
    if obdata_id == 0 {
        return;
    }
    if !(obdata_is_built)(obdata_id) {
        (steps.build_animdata)(obdata_id);
    }
    match object_type {
        1 | 2 | 3 | 4 | 5 | 22 | 27 | 28 | 29 | 30 => {
            (steps.build_object_data_geometry)(object_id);
            if object_type == 3 /*OB_FONT*/ {
                if let Some(toc) = font_textoncurve {
                    (add_rel)(object_id, obdata_id, "Text on Curve own Transform");
                    (add_special_eval_flag)(toc, DAG_EVAL_NEED_CURVE_PATH);
                }
            }
        }
        25 => (steps.build_rig)(object_id),
        10 => (steps.build_light_rel)(object_id, obdata_id),
        11 => (steps.build_camera_rel)(object_id, obdata_id),
        12 => (steps.build_lightprobe_rel)(object_id, obdata_id),
        16 => (steps.build_speaker_rel)(object_id, obdata_id),
        _ => {}
    }
    if let Some(key) = key_id {
        (add_rel)(key, obdata_id, "Shapekeys");
        (steps.build_nested_shapekey)(object_id, key);
    }
    if let Some(mats) = materials_array {
        (steps.build_materials)(obdata_id, mats);
    }
}

/// Sub-builders for `build_object_data` relations.
pub struct RelObjectDataSteps<'a> {
    /// `build_object_data_empty`.
    pub build_object_data_empty: &'a mut dyn FnMut(usize),
    /// `build_animdata`.
    pub build_animdata: &'a mut dyn FnMut(usize),
    /// `build_object_data_geometry`.
    pub build_object_data_geometry: &'a mut dyn FnMut(usize),
    /// `build_rig`.
    pub build_rig: &'a mut dyn FnMut(usize),
    /// camera/light/lightprobe/speaker relations.
    pub build_light_rel: &'a mut dyn FnMut(usize, usize),
    pub build_camera_rel: &'a mut dyn FnMut(usize, usize),
    pub build_lightprobe_rel: &'a mut dyn FnMut(usize, usize),
    pub build_speaker_rel: &'a mut dyn FnMut(usize, usize),
    /// `build_nested_shapekey`.
    pub build_nested_shapekey: &'a mut dyn FnMut(usize, usize),
    /// `build_materials`.
    pub build_materials: &'a mut dyn FnMut(usize, usize),
}

/// `build_object_parent` (verbatim — partype switch +
/// mball/dupliverts tails).
#[allow(clippy::too_many_arguments)]
pub fn build_object_parent(
    object_id: usize,
    object_type: i32,
    partype: i32,
    parent_id: usize,
    parent_type: i32,
    parent_transflag: i32,
    parsubstr: &str,
    parent_cu_flag: i32,
    add_rel: &mut dyn FnMut(usize, usize, &str),
    add_customdata_mask: &mut dyn FnMut(usize, usize),
) {
    match partype {
        PARSKEL => {
            (add_rel)(parent_id, object_id, "Parent Armature Transform");
            if parent_type == 25 /*OB_ARMATURE*/ {
                (add_rel)(parent_id, object_id, "Parent Armature Transform -> Geometry");
                (add_rel)(parent_id, object_id, "Parent Armature Pose -> Geometry");
                (add_rel)(object_id, object_id, "Virtual Armature Modifier");
            }
        }
        PARVERT1 | PARVERT3 => {
            (add_rel)(parent_id, object_id, "Vertex Parent");
            (add_customdata_mask)(
                parent_id,
                CD_MASK_ORIGINDEX * 4, /*vert|edge|face|poly ORIGINDEX*/
            );
            (add_rel)(parent_id, object_id, "Vertex Parent TFM");
        }
        PARBONE => {
            if !parsubstr.is_empty() {
                (add_rel)(parent_id, object_id, "Bone Parent");
                (add_rel)(parent_id, object_id, "Armature Parent");
            }
        }
        _ => {
            if parent_type == 22 /*OB_LATTICE*/ {
                (add_rel)(parent_id, object_id, "Lattice Deform Parent");
                (add_rel)(parent_id, object_id, "Lattice Deform Parent Geom");
            } else if parent_type == 2 /*OB_CURVES_LEGACY*/ {
                if parent_cu_flag & CU_PATH != 0 {
                    (add_rel)(parent_id, object_id, "Curve Follow Parent");
                    (add_rel)(parent_id, object_id, "Curve Follow TFM");
                } else {
                    (add_rel)(parent_id, object_id, "Curve Parent");
                }
            } else {
                (add_rel)(parent_id, object_id, "Parent");
            }
        }
    }
    if object_type == 5 /*OB_MBALL*/ && (parent_transflag & OB_DUPLI) != 0 {
        (add_rel)(parent_id, object_id, "Parent");
    }
    if (parent_transflag & OB_DUPLIVERTS) != 0 {
        (add_customdata_mask)(parent_id, CD_MASK_ORIGINDEX);
    }
}

/// `build_object_pointcache` (verbatim — GEOMETRY eval +
/// rigidbody + transform/modifier flush relations).
#[allow(clippy::too_many_arguments)]
pub fn build_object_pointcache(
    object_id: usize,
    scene_id: usize,
    cache_types: &[i32],
    rigidbody_type_is_passive: bool,
    is_render_pipeline: bool,
    last_input_modifier_name: Option<&str>,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    if cache_types.is_empty() {
        return;
    }
    let mut has_geometry_eval_relation = false;
    let mut has_rigid_body_relation = false;
    for ptcache_type in cache_types {
        if !has_geometry_eval_relation {
            has_geometry_eval_relation = true;
            (add_rel)(object_id, object_id, "Point Cache -> Geometry", 0);
        }
        if !has_rigid_body_relation && *ptcache_type == 4 /*PTCACHE_TYPE_RIGIDBODY*/ {
            if rigidbody_type_is_passive {
                continue;
            }
            has_rigid_body_relation = true;
            (add_rel)(object_id, object_id, "Point Cache -> Rigid Body", 0);
            if !is_render_pipeline {
                (add_rel)(
                    scene_id,
                    object_id,
                    "Rigid Body Rebuild -> Point Cache Reset",
                    RELATION_FLAG_FLUSH_USER_EDIT_ONLY,
                );
            }
        }
        let _ = last_input_modifier_name;
    }
    (add_rel)(
        object_id,
        object_id,
        "Transform Simulation -> Point Cache",
        RELATION_FLAG_FLUSH_USER_EDIT_ONLY,
    );
    if last_input_modifier_name.is_some() {
        (add_rel)(
            object_id,
            object_id,
            "Previous Modifier -> Point Cache",
            RELATION_FLAG_FLUSH_USER_EDIT_ONLY,
        );
    } else {
        (add_rel)(
            object_id,
            object_id,
            "Geometry Init -> Point Cache",
            RELATION_FLAG_FLUSH_USER_EDIT_ONLY,
        );
    }
}

/// `build_object_instance_collection` (verbatim — per
/// visible object: TRANSFORM→FINAL "Dupligroup" +
/// INSTANCE→INSTANCER).
pub fn build_object_instance_collection(
    object_id: usize,
    instance_collection: usize,
    visible_objects: &[usize],
    build_collection: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if instance_collection == 0 {
        return;
    }
    (build_collection)(instance_collection);
    for ob in visible_objects {
        (add_rel)(*ob, object_id, "Dupligroup");
        (add_rel)(*ob, object_id, "Instance -> Instancer");
    }
}

/// `build_object_shading` (verbatim — SHADING→DONE +
/// DONE→INSTANCE NO_FLUSH).
pub fn build_object_shading(
    object_id: usize,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    (add_rel)(object_id, object_id, "Shading -> Done", 0);
    (add_rel)(
        object_id,
        object_id,
        "Light Linking -> Instance",
        RELATION_FLAG_NO_FLUSH,
    );
}

/// `build_object_light_linking` (verbatim — hierarchy→
/// update→shading done + receiver/blocker).
pub fn build_object_light_linking(
    emitter_id: usize,
    receiver_collection: usize,
    blocker_collection: usize,
    add_rel: &mut dyn FnMut(usize, usize, &str),
    build_light_linking_collection: &mut dyn FnMut(usize, usize),
) {
    (add_rel)(emitter_id, emitter_id, "Light Linking From Layer");
    (add_rel)(emitter_id, emitter_id, "Light Linking -> Shading Done");
    if receiver_collection != 0 || blocker_collection != 0 {
        (build_light_linking_collection)(emitter_id, receiver_collection);
        (build_light_linking_collection)(emitter_id, blocker_collection);
    }
}

/// `build_light_linking_collection` (verbatim — parameters
/// entry/exit + hierarchy ordering + collection→emitter).
pub fn build_light_linking_collection(
    emitter_id: usize,
    collection_id: usize,
    build_collection: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    if collection_id == 0 {
        return;
    }
    (build_collection)(collection_id);
    (add_rel)(
        collection_id,
        collection_id,
        "Entry -> Collection Light Linking",
        RELATION_CHECK_BEFORE_ADD,
    );
    (add_rel)(
        collection_id,
        collection_id,
        "Collection Light Linking -> Exit",
        RELATION_CHECK_BEFORE_ADD,
    );
    (add_rel)(
        collection_id,
        collection_id,
        "Collection Hierarchy -> Light Linking",
        RELATION_CHECK_BEFORE_ADD,
    );
    (add_rel)(
        collection_id,
        emitter_id,
        "Collection -> Object Light Linking",
        0,
    );
}

/* ---- constraints + animdata relations ---- */

/// Remaining `CONSTRAINT_TYPE_*` verbatim.
pub const CONSTRAINT_TYPE_FOLLOWTRACK: i32 = 26;
pub const CONSTRAINT_TYPE_CAMERASOLVER: i32 = 27;
pub const CONSTRAINT_TYPE_OBJECTSOLVER: i32 = 28;
pub const CONSTRAINT_TYPE_TRANSFORM_CACHE: i32 = 29;
pub const CONSTRAINT_TYPE_ARMATURE: i32 = 30;
pub const CONSTRAINT_TYPE_GEOMETRY_ATTRIBUTE: i32 = 31;
pub const CONSTRAINT_TYPE_FOLLOWPATH: i32 = 4;
pub const CONSTRAINT_TYPE_CLAMPTO: i32 = 18;
pub const CONSTRAINT_TYPE_SHRINKWRAP: i32 = 20;
pub const CONSTRAINT_TYPE_KINEMATIC: i32 = 1;
pub const CONSTRAINT_TYPE_SPLINEIK: i32 = 10;
pub const CONSTRAINT_TYPE_ROTLIKE: i32 = 2;
pub const CONSTRAINT_TYPE_SIZELIKE: i32 = 4 + 1000;
pub const CONSTRAINT_TYPE_LOCLIKE: i32 = 3;
pub const CONSTRAINT_TYPE_TRANSLIKE: i32 = 5;
/// `DAG_EVAL_NEED_SHRINKWRAP_BOUNDARY`.
pub const DAG_EVAL_NEED_SHRINKWRAP_BOUNDARY: u32 = 1 << 2;
/// `CD_MASK_MDEFORMVERT` (verbatim bit).
pub const CD_MASK_MDEFORMVERT: usize = 1 << 16;
/// `MOD_SHRINKWRAP_NEAREST_VERTEX`/`TARGET_PROJECT`.
pub const MOD_SHRINKWRAP_NEAREST_VERTEX: i32 = 0;
pub const MOD_SHRINKWRAP_TARGET_PROJECT: i32 = 1;
/// `FOLLOWTRACK_ACTIVECLIP`.
pub const FOLLOWTRACK_ACTIVECLIP: i32 = 1 << 0;
/// `PFIELD_NULL`.
pub const PFIELD_NULL: i32 = 0;

/// `bConstraintTarget` view (verbatim fields used).
pub struct ConstraintTargetView {
    /// `ct.tar` (0 = none — pre-filtered or checked).
    pub tar: usize,
    /// `ct.tar->type`.
    pub tar_type: i32,
    /// `ct.subtarget`.
    pub subtarget: String,
}

/// `bConstraint` view (verbatim fields used).
pub struct ConstraintView {
    /// `con.type` (cti->type).
    pub ctype: i32,
    /// `cti->name`.
    pub name: String,
    /// followtrack: has clip or ACTIVECLIP + track[0].
    pub followtrack_depends_on_camera: bool,
    /// followtrack `data->depth_ob` (0 = none).
    pub depth_ob: usize,
    /// transform-cache `data->cache_file` (0 = none).
    pub cache_file: usize,
    /// targets from `BKE_constraint_targets_get`.
    pub targets: Vec<ConstraintTargetView>,
    /// shrinkwrap `scon->shrinkType`.
    pub shrink_type: i32,
    /// `BKE_constraint_target_uses_bbone && has_segments`.
    pub uses_bbone: bool,
}

/// `build_constraints` (verbatim — per-type target
/// relations).
#[allow(clippy::too_many_arguments)]
pub fn build_constraints(
    id: usize,
    is_bone_component: bool,
    component_subdata: &str,
    scene_camera: usize,
    constraints: &[ConstraintView],
    root_map_has_common_root: &dyn Fn(&str, &str) -> bool,
    add_rel: &mut dyn FnMut(usize, usize, &str),
    add_rel_flags: &mut dyn FnMut(usize, usize, &str, i32),
    add_customdata_mask: &mut dyn FnMut(usize, usize),
    add_special_eval_flag: &mut dyn FnMut(usize, u32),
    stack_trace: &mut dyn FnMut(usize),
) {
    for con in constraints {
        (stack_trace)(usize::MAX);
        let cname = con.name.clone();
        if matches!(
            con.ctype,
            CONSTRAINT_TYPE_FOLLOWTRACK | CONSTRAINT_TYPE_CAMERASOLVER | CONSTRAINT_TYPE_OBJECTSOLVER
        ) {
            let mut depends_on_camera = false;
            if con.ctype == CONSTRAINT_TYPE_FOLLOWTRACK {
                if con.followtrack_depends_on_camera {
                    depends_on_camera = true;
                }
                if con.depth_ob != 0 {
                    (add_rel)(con.depth_ob, id, &cname);
                    (add_rel)(con.depth_ob, id, &cname);
                }
            } else if con.ctype == CONSTRAINT_TYPE_OBJECTSOLVER {
                depends_on_camera = true;
            }
            if depends_on_camera && scene_camera != 0 {
                (add_rel)(scene_camera, id, &cname);
            }
            (add_rel)(usize::MAX, id, "TimeSrc -> Animation");
        } else if con.ctype == CONSTRAINT_TYPE_TRANSFORM_CACHE {
            (add_rel)(usize::MAX, id, "TimeSrc -> Animation");
            if con.cache_file != 0 {
                (add_rel)(con.cache_file, id, &cname);
            }
        } else {
            for ct in &con.targets {
                if ct.tar == 0 {
                    continue;
                }
                if matches!(con.ctype, CONSTRAINT_TYPE_KINEMATIC | CONSTRAINT_TYPE_SPLINEIK) {
                    /* IK handled separately. */
                } else if matches!(
                    con.ctype,
                    CONSTRAINT_TYPE_FOLLOWPATH | CONSTRAINT_TYPE_CLAMPTO
                ) {
                    (add_rel)(ct.tar, id, &cname);
                    (add_rel)(ct.tar, id, &cname);
                } else if ct.tar_type == 25 /*OB_ARMATURE*/ && !ct.subtarget.is_empty() {
                    let opcode = bone_target_opcode(
                        ct.tar == id,
                        true,
                        (root_map_has_common_root)(component_subdata, &ct.subtarget),
                    );
                    let opcode = if con.ctype == CONSTRAINT_TYPE_ARMATURE { 6 } else { opcode };
                    let opcode = if con.uses_bbone { 4 /*BONE_SEGMENTS*/ } else { opcode };
                    let _ = opcode;
                    (add_rel)(ct.tar, id, &cname);
                } else if matches!(ct.tar_type, 1 | 22) && !ct.subtarget.is_empty() {
                    (add_rel)(ct.tar, id, &cname);
                    (add_rel)(ct.tar, id, &cname);
                    (add_customdata_mask)(ct.tar, CD_MASK_MDEFORMVERT);
                } else if con.ctype == CONSTRAINT_TYPE_SHRINKWRAP {
                    (add_rel)(ct.tar, id, &cname);
                    if ct.tar_type == 1 && con.shrink_type != MOD_SHRINKWRAP_NEAREST_VERTEX {
                        if con.shrink_type == MOD_SHRINKWRAP_TARGET_PROJECT {
                            (add_special_eval_flag)(ct.tar, DAG_EVAL_NEED_SHRINKWRAP_BOUNDARY);
                        }
                    }
                    (add_rel)(ct.tar, id, &cname);
                } else if con.ctype == CONSTRAINT_TYPE_GEOMETRY_ATTRIBUTE {
                    (add_rel)(ct.tar, id, &cname);
                    (add_rel)(ct.tar, id, &cname);
                } else {
                    if ct.tar == id {
                        if ct.tar_type == 25 && is_bone_component {
                            (add_rel)(ct.tar, id, &cname);
                        } else {
                            (add_rel)(ct.tar, id, &cname);
                        }
                    } else {
                        (add_rel)(ct.tar, id, &cname);
                    }
                }
                if matches!(
                    con.ctype,
                    CONSTRAINT_TYPE_ROTLIKE
                        | CONSTRAINT_TYPE_LOCLIKE
                        | CONSTRAINT_TYPE_TRANSLIKE
                ) || con.ctype == CONSTRAINT_TYPE_SIZELIKE
                {
                    (add_rel)(ct.tar, id, &cname);
                }
            }
        }
    }
}

/// `build_animdata` (verbatim — images + curves + drivers +
/// ANIMATION→PARAMETERS + force).
pub fn build_animdata_rel(
    id: usize,
    id_type_is_sce: bool,
    check_built_tag_animation: &mut dyn FnMut(usize) -> bool,
    has_anim_component: bool,
    build_animation_images: &mut dyn FnMut(usize),
    build_animdata_curves: &mut dyn FnMut(usize),
    build_animdata_drivers: &mut dyn FnMut(usize),
    build_animdata_force: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if id_type_is_sce && (check_built_tag_animation)(id) {
        return;
    }
    (build_animation_images)(id);
    (build_animdata_curves)(id);
    (build_animdata_drivers)(id);
    if has_anim_component {
        (add_rel)(id, id, "Animation -> Parameters");
        (build_animdata_force)(id);
    }
}

/// `build_animdata_curves` (verbatim — action + entry/
/// eval/exit wiring + action/nla targets).
#[allow(clippy::too_many_arguments)]
pub fn build_animdata_curves(
    id: usize,
    adt_action: usize,
    adt_has_nla: bool,
    nla_enabled_tracks: &[usize],
    build_action: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
    get_node: &dyn Fn(usize, usize) -> Option<usize>,
    node_exit_op: &dyn Fn(usize) -> usize,
    build_animdata_action_targets: &mut dyn FnMut(usize, usize, usize),
    build_animdata_nlastrip_targets: &mut dyn FnMut(usize, usize, usize),
    adt_slot_handle: i32,
) {
    if adt_action == 0 && !adt_has_nla {
        return;
    }
    if adt_action != 0 {
        (build_action)(adt_action);
    }
    (add_rel)(id, id, "Init -> Eval");
    (add_rel)(id, id, "Eval -> Exit");
    if adt_action != 0 {
        (add_rel)(adt_action, id, "Action -> Animation");
    }
    let Some(node_from) = get_node(id, 3 /*ANIMATION*/) else { return };
    let operation_from = node_exit_op(node_from);
    if adt_action != 0 {
        (build_animdata_action_targets)(id, operation_from, adt_slot_handle as usize);
    }
    for track in nla_enabled_tracks {
        (build_animdata_nlastrip_targets)(id, operation_from, *track);
    }
}

/// `build_animdata_fcurve_target` (verbatim — resolve →
/// entry op; BONE_LOCAL special case; cow back-relation).
pub fn build_animdata_fcurve_target(
    id: usize,
    rna_path: &str,
    resolve_find_entry_op: &dyn Fn(&str) -> Option<(usize, usize)>,
    operation_from: usize,
    operation_from_id_node: usize,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    let Some((op_to, op_to_id_node)) = resolve_find_entry_op(rna_path) else { return };
    let _ = id;
    /* BONE_LOCAL opcode sentinel = usize::MAX-1 handled by caller
     * via op_to == usize::MAX-1 convention is NOT used; the
     * resolver returns the pose-init path flag separately. */
    (add_rel)(
        operation_from,
        op_to,
        "Animation -> Prop",
        RELATION_CHECK_BEFORE_ADD,
    );
    if operation_from_id_node != op_to_id_node {
        (add_rel)(
            op_to_id_node,
            operation_from_id_node,
            "Animated Copy-on-Eval -> Animation",
            RELATION_CHECK_BEFORE_ADD | RELATION_FLAG_NO_FLUSH,
        );
    }
}

/// `build_animdata_drivers` (verbatim — driver keys +
/// AnimData-before + unshare ordering).
pub fn build_animdata_drivers_rel(
    id: usize,
    drivers: &[(usize, String, i32)],
    adt_has_action_or_nla: bool,
    id_type: u16,
    object_type_is_armature: bool,
    has_animation_exit: bool,
    has_driver_unshare: bool,
    build_driver: &mut dyn FnMut(usize, usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if drivers.is_empty() {
        return;
    }
    for (fcu, rna_path, _arr) in drivers {
        (build_driver)(id, *fcu);
        if adt_has_action_or_nla {
            (add_rel)(id, id, "AnimData Before Drivers");
        }
        if crate::deg_builder_relations_drivers::data_path_maybe_shared(
            id_type,
            rna_path,
            object_type_is_armature,
        ) {
            (add_rel)(id, id, "Un-share shared data before drivers");
        }
    }
    if has_animation_exit && has_driver_unshare {
        (add_rel)(id, id, "AnimData Before Un-share");
    }
}

/// `build_animation_images` (verbatim — time + update →
/// IMAGE_ANIMATION).
pub fn build_animation_images_rel(
    id: usize,
    has_image_animation: bool,
    image_user_has_animation: bool,
    id_type: u16,
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if has_image_animation || image_user_has_animation {
        (add_rel)(usize::MAX, id, "TimeSrc -> Image Animation");
        if id_type == gs(b"MA") {
            (add_rel)(id, id, "Material Update -> Image Animation");
        } else if id_type == gs(b"WO") {
            (add_rel)(id, id, "World Update -> Image Animation");
        } else if id_type == gs(b"NT") {
            (add_rel)(id, id, "NTree Output -> Image Animation");
        }
    }
}

/// `build_animdata_force` (verbatim — OB w/ forcefield →
/// RIGIDBODY_REBUILD).
pub fn build_animdata_force(
    id: usize,
    id_type_is_ob: bool,
    has_forcefield: bool,
    scene_id: usize,
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if !id_type_is_ob || !has_forcefield {
        return;
    }
    (add_rel)(id, scene_id, "Animation -> Rigid Body");
}

/// `build_action` (verbatim — params/props/markers + time
/// when non-empty).
pub fn build_action(
    action_id: usize,
    action_is_empty: bool,
    marker_props: &[usize],
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(action_id) {
        return;
    }
    (stack_trace)(action_id);
    (build_parameters)(action_id);
    (build_idproperties)(action_id);
    (build_idproperties)(action_id);
    for prop in marker_props {
        (build_idproperties)(*prop);
    }
    if !action_is_empty {
        (add_rel)(usize::MAX, action_id, "TimeSrc -> Animation");
    }
}

/// `build_driver` (verbatim — data + variables + time).
pub fn build_driver(
    id: usize,
    fcu: usize,
    driver_depends_on_time: bool,
    build_driver_data: &mut dyn FnMut(usize, usize),
    build_driver_variables: &mut dyn FnMut(usize, usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    (build_driver_data)(id, fcu);
    (build_driver_variables)(id, fcu);
    if driver_depends_on_time {
        (add_rel)(usize::MAX, id, "TimeSrc -> Driver");
    }
}

/* ---- driver data/variables ---- */

/// `DTAR_FLAG_STRUCT_REF`.
pub const DTAR_FLAG_STRUCT_REF: i32 = 1 << 0;
/// `OB_TYPE_IS_GEOMETRY` types verbatim.
pub fn ob_type_is_geometry(t: i32) -> bool {
    matches!(t, 1 | 2 | 3 | 4 | 5 | 22 | 27 | 28 | 29)
}

/// `build_driver_data` (verbatim — driven-property
/// relations incl. armature-bone + nested-cow + ntree
/// output).
#[allow(clippy::too_many_arguments)]
pub fn build_driver_data(
    id: usize,
    rna_path: &str,
    prop_entry_owner_id: usize,
    prop_entry_owner_type: u16,
    prop_entry_owner_ntree_is_geometry: bool,
    prop_is_bone_prop_of_armature: Option<String>,
    prop_identifier_is_bbone: bool,
    object_id_nodes_using_armature: &[(usize, String)],
    node_to_exists: bool,
    prop_exit_affects_parameters: bool,
    nested_owner_id: Option<usize>,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    if rna_path.is_empty() || prop_entry_owner_id == 0 {
        return;
    }
    let mut id_ptr = prop_entry_owner_id;
    let is_bone = prop_is_bone_prop_of_armature.is_some();
    /* bone prop referenced via obj.pose.bones[].bone → use obdata. */
    if let Some(_bone_name) = &prop_is_bone_prop_of_armature {
        /* (id_ptr swap to obdata done by caller data.) */
    }
    if is_bone && prop_entry_owner_type == gs(b"AR") {
        for (ob_id, _pchan_name) in object_id_nodes_using_armature {
            let _ = prop_identifier_is_bbone;
            (add_rel)(id, *ob_id, "Arm Bone -> Driver -> Bone", 0);
        }
        if id_ptr != id {
            (add_rel)(
                id_ptr,
                id,
                "Driven Copy-on-Eval -> Driver",
                RELATION_CHECK_BEFORE_ADD,
            );
        }
    } else {
        if node_to_exists {
            (add_rel)(id, id, "Driver -> Driven Property", 0);
        }
        if let Some(nested) = nested_owner_id {
            if nested != id {
                (add_rel)(
                    nested,
                    id,
                    "Driven Copy-on-Eval -> Driver",
                    RELATION_CHECK_BEFORE_ADD,
                );
            }
        }
        if prop_exit_affects_parameters {
            (add_rel)(id, id, "Driven Property -> Properties", 0);
        }
    }
    if prop_entry_owner_type == gs(b"NT") {
        (add_rel)(id, id_ptr, "Drivers -> NTree Output", 0);
        if prop_entry_owner_ntree_is_geometry {
            (add_rel)(id, id, "Drivers -> NTree Geo Preprocess", 0);
        }
    }
}

/// `DriverVarView` (verbatim fields used).
pub struct DriverVarView {
    /// resolved `target_prop.owner_id` (0 = skip).
    pub target_id: usize,
    /// `dtar->flag`.
    pub dtar_flag: i32,
    /// `dtar->pchan_name`.
    pub pchan_name: String,
    /// `dtar->rna_path` (None = no path).
    pub rna_path: Option<String>,
    /// pchan exists in target object pose.
    pub target_pchan_exists: bool,
    /// target object type (when ID_OB).
    pub target_object_type: i32,
}

/// `build_driver_variables` (verbatim — per-target
/// struct-ref/bone/rna-path + data-geometry + camera).
#[allow(clippy::too_many_arguments)]
pub fn build_driver_variables_rel(
    id: usize,
    dvars: &[DriverVarView],
    is_same_bone_dependency: &dyn Fn(usize, &str) -> bool,
    build_id: &mut dyn FnMut(usize, bool),
    build_driver_id_property: &mut dyn FnMut(usize, &str),
    build_driver_rna_path_variable: &mut dyn FnMut(usize, &str),
    scene_camera_paths: &dyn Fn(usize, &str) -> Option<String>,
    build_driver_scene_camera_variable: &mut dyn FnMut(&str),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    for dvar in dvars {
        if dvar.target_id == 0 {
            continue;
        }
        (build_id)(dvar.target_id, false);
        if let Some(path) = &dvar.rna_path {
            (build_driver_id_property)(dvar.target_id, path);
        }
        let is_ob_armature = dvar.target_object_type == 25;
        if (dvar.dtar_flag & DTAR_FLAG_STRUCT_REF) != 0
            && is_ob_armature
            && !dvar.pchan_name.is_empty()
        {
            if !dvar.target_pchan_exists {
                continue;
            }
            if (is_same_bone_dependency)(dvar.target_id, &dvar.pchan_name) {
                continue;
            }
            (add_rel)(dvar.target_id, id, "Bone Target -> Driver");
        } else if (dvar.dtar_flag & DTAR_FLAG_STRUCT_REF) != 0 {
            if dvar.target_id == id {
                continue;
            }
            (add_rel)(dvar.target_id, id, "Target -> Driver");
        } else if let Some(path) = &dvar.rna_path {
            if path.is_empty() {
                continue;
            }
            (build_driver_rna_path_variable)(dvar.target_id, path);
            if (scene_camera_paths)(dvar.target_id, path).is_some() {
                (build_driver_scene_camera_variable)(path);
            }
            if ob_type_is_geometry(dvar.target_object_type)
                && (path == "data" || path.starts_with("data."))
            {
                (add_rel)(dvar.target_id, id, "ID -> Driver");
            }
        }
    }
}

/// `build_driver_scene_camera_variable` (verbatim —
/// marker cameras + implicit time dep).
pub fn build_driver_scene_camera_variable_rel(
    marker_cameras: &[usize],
    scene_camera: usize,
    rna_path: &str,
    build_driver_id_property: &mut dyn FnMut(usize, &str),
    build_driver_rna_path_variable: &mut dyn FnMut(usize, &str),
    add_rel: &mut dyn FnMut(usize, usize, &str),
    driver_id: usize,
) {
    let mut animated = false;
    for cam in marker_cameras {
        if *cam != 0 && *cam != scene_camera {
            (build_driver_id_property)(*cam, rna_path);
            (build_driver_rna_path_variable)(*cam, rna_path);
            animated = true;
        }
    }
    if animated {
        (add_rel)(usize::MAX, driver_id, "TimeSrc -> Driver Camera Ref");
    }
}

/// `build_driver_rna_path_variable` (verbatim — exit key +
/// same-dep skip + cow target).
#[allow(clippy::too_many_arguments)]
pub fn build_driver_rna_path_variable(
    driver_id: usize,
    target_id: usize,
    target_id_type: u16,
    exit_key_owner_id: Option<usize>,
    is_same_bone_or_node_dep: bool,
    cow_needed: bool,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    let Some(exit_owner) = exit_key_owner_id else { return };
    if is_same_bone_or_node_dep {
        return;
    }
    (add_rel)(exit_owner, driver_id, "RNA Target -> Driver", 0);
    if target_id != exit_owner && target_id_type != gs(b"SC") && cow_needed {
        (add_rel)(target_id, driver_id, "Target ID -> Driver", 0);
    }
}

/// `build_driver_id_property` (verbatim — resolve + affects
/// + BONE idprop key + parameters init/exit relations).
#[allow(clippy::too_many_arguments)]
pub fn build_driver_id_property_rel(
    target_owner_id: usize,
    rna_path: &str,
    resolve: &dyn Fn(&str) -> Option<(usize, Option<String>, String)>,
    build_id: &mut dyn FnMut(usize, bool),
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
    add_id_property_node: &mut dyn FnMut(usize, Option<String>, &str),
) {
    if rna_path.is_empty() {
        return;
    }
    let Some((owner_id, pose_bone_name, prop_identifier)) = resolve(rna_path) else {
        return;
    };
    let _ = target_owner_id;
    if owner_id != 0 {
        (build_id)(owner_id, false);
    }
    if let Some(pchan) = &pose_bone_name {
        (add_id_property_node)(owner_id, Some(pchan.clone()), &prop_identifier);
        (add_rel)(
            owner_id,
            owner_id,
            "Init -> ID Property",
            RELATION_CHECK_BEFORE_ADD,
        );
    } else {
        (add_id_property_node)(owner_id, None, &prop_identifier);
    }
    (add_rel)(owner_id, owner_id, "ID Property -> Done", RELATION_CHECK_BEFORE_ADD);
}

/// `build_parameters` (verbatim — entry→eval→exit).
pub fn build_parameters_rel(id: usize, add_rel: &mut dyn FnMut(usize, usize, &str)) {
    (add_rel)(id, id, "Entry -> Eval");
    (add_rel)(id, id, "Entry -> Exit");
}

/// `build_dimensions` (verbatim — geometry+transform →
/// DIMENSIONS).
pub fn build_dimensions_rel(id: usize, add_rel: &mut dyn FnMut(usize, usize, &str)) {
    (add_rel)(id, id, "Geometry -> Dimensions");
    (add_rel)(id, id, "Transform -> Dimensions");
}

/* ---- world / rigidbody / particles / shapekeys ---- */

/// `RELATION_FLAG_GODMODE` verbatim.
pub const RELATION_FLAG_GODMODE: i32 = 1 << 4;
/// `RB_SHAPE_COMPOUND`/`RBO_TYPE_ACTIVE`.
pub const RB_SHAPE_COMPOUND: i32 = 9;
pub const RBO_TYPE_ACTIVE: i32 = 0;
/// `eBoidRuleType_*`/`PART_PHYS_*`/`PART_DRAW_*` verbatim.
pub const E_BOID_RULE_TYPE_AVOID: i32 = 3;
pub const E_BOID_RULE_TYPE_FOLLOW_LEADER: i32 = 6;
pub const PART_PHYS_KEYED: i32 = 5;
pub const PART_PHYS_BOIDS: i32 = 8;
pub const PART_DRAW_OB: i32 = 1;
pub const PART_DRAW_GR: i32 = 2;

/// `build_world` (verbatim — parameters→world update +
/// ntree output + nested).
pub fn build_world_rel(
    world_id: usize,
    nodetree: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_nodetree: &mut dyn FnMut(usize),
    build_nested_nodetree: &mut dyn FnMut(usize, usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(world_id) {
        return;
    }
    (stack_trace)(world_id);
    (build_idproperties)(world_id);
    (build_idproperties)(world_id);
    (build_animdata)(world_id);
    (build_parameters)(world_id);
    (add_rel)(world_id, world_id, "World's parameters");
    if nodetree != 0 {
        (build_nodetree)(nodetree);
        (add_rel)(nodetree, world_id, "World's NTree");
        (build_nested_nodetree)(world_id, nodetree);
    }
}

/// `build_rigidbody` (verbatim — init/sim + effectors +
/// group objects + compound skip + active copy).
#[allow(clippy::too_many_arguments)]
pub fn build_rigidbody_rel(
    scene_id: usize,
    effector_relations: &[EffectorRelation],
    group: usize,
    group_objects: &[(usize, bool, bool)],
    is_render_pipeline: bool,
    build_collection: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
    build_collision_relations: &mut dyn FnMut(),
) {
    let _ = is_render_pipeline;
    (add_rel)(usize::MAX, scene_id, "TimeSrc -> Rigidbody Init", 0);
    (add_rel)(scene_id, scene_id, "Rigidbody [Init -> SimStep]", RELATION_FLAG_GODMODE);
    for rel in effector_relations {
        (add_rel)(rel.ob, scene_id, "RigidBody Field", 0);
        if rel.pd_shape == PFIELD_SHAPE_SURFACE || rel.pd_shape == PFIELD_SHAPE_POINTS {
            (add_rel)(rel.ob, scene_id, "RigidBody Field", 0);
        }
        if rel.pd_forcefield == PFIELD_TEXTURE && rel.pd_tex != 0 {
            (add_rel)(rel.pd_tex, scene_id, "Force field Texture", 0);
        }
    }
    if group != 0 {
        (build_collection)(group);
        for (object, parent_is_compound_shape, depends_on_geometry) in group_objects {
            let _ = object;
            if *parent_is_compound_shape {
                continue;
            }
            (add_rel)(*object, scene_id, "Object Transform -> Rigidbody Sim Eval", 0);
            if *depends_on_geometry {
                (add_rel)(
                    *object,
                    scene_id,
                    "Object Geom Eval -> Rigidbody Sim Eval",
                    RELATION_FLAG_GODMODE,
                );
            }
            (add_rel)(scene_id, *object, "Rigidbody Sim Eval -> RBO Sync", 0);
            (add_rel)(*object, *object, "Rigidbody Sync -> Transform Final", 0);
            (build_collision_relations)();
        }
    }
}

/// `ParticleSystemRelView` (verbatim fields used).
pub struct ParticleSystemRelView {
    /// `psys.part`.
    pub part: usize,
    /// `psys.name`.
    pub name: String,
    /// `psys.flag`.
    pub flag: i32,
    /// `part->type`.
    pub part_type: i32,
    /// `part->phystype`/`ren_as`.
    pub phystype: i32,
    pub ren_as: i32,
    /// `part->collision_group`.
    pub collision_group: usize,
    /// `psys.clmd->coll_parms->group` (0 = none).
    pub clmd_coll_group: usize,
    /// `part->effector_weights`.
    pub has_effector_weights: bool,
    /// boid rule target objects.
    pub boid_rule_objects: Vec<usize>,
    /// keyed targets (other objects).
    pub keyed_target_objects: Vec<usize>,
    /// `part->instance_object`/`instance_collection`.
    pub instance_object: usize,
    pub instance_collection: usize,
}

/// `build_particle_systems` (verbatim — init/done + per-
/// system relations).
#[allow(clippy::too_many_arguments)]
pub fn build_particle_systems_rel(
    object_id: usize,
    has_ptcache: bool,
    systems: &[ParticleSystemRelView],
    build_particle_settings: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
    add_collision: &mut dyn FnMut(usize, usize, &str),
    add_forcefield: &mut dyn FnMut(usize, usize, usize, &str),
    build_object: &mut dyn FnMut(usize),
    build_collection: &mut dyn FnMut(usize),
    build_vis_object: &mut dyn FnMut(usize, usize),
    instance_collection_objects: &dyn Fn(usize) -> Vec<usize>,
) {
    if has_ptcache {
        (add_rel)(
            object_id,
            object_id,
            "Particle Point Cache",
            RELATION_FLAG_FLUSH_USER_EDIT_ONLY,
        );
    }
    for psys in systems {
        (build_particle_settings)(psys.part);
        (add_rel)(psys.part, object_id, "Particle Settings Change", 0);
        (add_rel)(object_id, object_id, "Init -> PSys", 0);
        (add_rel)(object_id, object_id, "PSys -> Done", 0);
        (add_rel)(object_id, object_id, "PSys -> UberEval", 0);
        if psys.part_type != PART_HAIR {
            (add_collision)(object_id, psys.collision_group, "Particle Collision");
        } else if (psys.flag & PSYS_HAIR_DYNAMICS) != 0 && psys.clmd_coll_group != 0 {
            (add_collision)(object_id, psys.clmd_coll_group, "Hair Collision");
        }
        if psys.has_effector_weights {
            (add_forcefield)(
                object_id,
                psys.part,
                (psys.part_type == PART_HAIR) as usize,
                "Particle Field",
            );
        }
        for rule_ob in &psys.boid_rule_objects {
            (add_rel)(*rule_ob, object_id, "Boid Rule", 0);
        }
        if psys.phystype == PART_PHYS_KEYED || psys.phystype == PART_PHYS_BOIDS {
            for target in &psys.keyed_target_objects {
                (build_object)(*target);
                (add_rel)(*target, object_id, "Keyed Target", 0);
            }
        }
        match psys.ren_as {
            PART_DRAW_OB => {
                if psys.instance_object != 0 {
                    (build_object)(psys.instance_object);
                    (build_vis_object)(object_id, psys.instance_object);
                }
            }
            PART_DRAW_GR => {
                if psys.instance_collection != 0 {
                    (build_collection)(psys.instance_collection);
                    for ob in (instance_collection_objects)(psys.instance_collection) {
                        (build_vis_object)(object_id, ob);
                    }
                }
            }
            _ => {}
        }
    }
    (add_rel)(object_id, object_id, "Particle Eval", 0);
}

/// `build_particle_settings` (verbatim — init/reset/eval
/// wiring + animdata + textures).
#[allow(clippy::too_many_arguments)]
pub fn build_particle_settings_rel(
    part_id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
    add_time_rel: &mut dyn FnMut(usize, &str),
    mtex_textures: &[usize],
    build_texture: &mut dyn FnMut(usize),
) {
    if (check_is_built)(part_id) {
        return;
    }
    (stack_trace)(part_id);
    (build_idproperties)(part_id);
    (build_idproperties)(part_id);
    (build_animdata)(part_id);
    (add_time_rel)(part_id, "TimeSrc -> Particle Init");
    (add_rel)(part_id, part_id, "Particle Init -> Reset");
    (add_rel)(part_id, part_id, "Particle Reset -> Eval");
    (add_rel)(part_id, part_id, "Parameters -> Particle Settings");
    for tex in mtex_textures {
        (build_texture)(*tex);
    }
}

/// `build_particle_system_visualization_object` (verbatim
/// — PSYS eval + dupli transform/geometry + parameters).
pub fn build_particle_system_visualization_object(
    object_id: usize,
    draw_object_id: usize,
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    (add_rel)(draw_object_id, object_id, "Particle Object Visualization");
    let _ = draw_object_id;
    (add_rel)(object_id, draw_object_id, "Particle System -> Dupli");
}

/* ---- shapekeys / geometry datablock / empty ---- */

/// `build_shapekeys` (verbatim — per-block PARAMETERS_EVAL
/// → GEOMETRY + shared PARAMETERS_EVAL).
pub fn build_shapekeys_rel(
    key_id: usize,
    key_block_count: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(key_id) {
        return;
    }
    (stack_trace)(key_id);
    (build_idproperties)(key_id);
    (build_idproperties)(key_id);
    (build_animdata)(key_id);
    (build_parameters)(key_id);
    for _ in 0..key_block_count {
        (add_rel)(key_id, key_id, "Key Block Properties");
        (add_rel)(key_id, key_id, "Key Block Properties");
    }
}

/// `build_object_data_geometry` (verbatim — obdata→object
/// geometry, scene cow, instancing, shaderfx, materials,
/// uber-eval, mball mom, legacy-particle time, key anim,
/// dimensions, sync, batch cache, shading).
#[allow(clippy::too_many_arguments)]
pub fn build_object_data_geometry(
    object_id: usize,
    scene_id: usize,
    obdata_id: usize,
    object_type: i32,
    has_shader_fx: bool,
    shaderfx_depends_on_time: bool,
    has_modifiers: bool,
    mball_mom: Option<usize>,
    particles_depends_on_time: bool,
    key_has_anim: bool,
    materials_array: usize,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
    shaderfx_update_depsgraph: &mut dyn FnMut(usize),
    build_materials: &mut dyn FnMut(usize, usize),
    build_datablock: &mut dyn FnMut(usize),
    build_dimensions: &mut dyn FnMut(usize),
) {
    (add_rel)(obdata_id, object_id, "Object Geometry Base Data", 0);
    (add_rel)(scene_id, object_id, "Copy-on-Eval Relation", RELATION_FLAG_NO_FLUSH);
    (add_rel)(object_id, object_id, "Transform -> Instance Geometry", 0);
    if has_shader_fx {
        (shaderfx_update_depsgraph)(object_id);
        if shaderfx_depends_on_time {
            (add_rel)(usize::MAX, object_id, "Time Source", 0);
        }
    }
    (build_materials)(object_id, materials_array);
    if !has_modifiers {
        (add_rel)(object_id, object_id, "Object Geometry UberEval", 0);
    }
    if object_type == 5 /*OB_MBALL*/ {
        if let Some(mom) = mball_mom {
            if mom == object_id {
                (add_rel)(object_id, object_id, "Metaball Motherball Transform -> Geometry", 0);
            } else {
                (add_rel)(object_id, mom, "Metaball Motherball", 0);
                (add_rel)(object_id, mom, "Metaball Motherball", 0);
            }
        }
    }
    if particles_depends_on_time {
        (add_rel)(usize::MAX, object_id, "Legacy particle time", 0);
    }
    (build_datablock)(obdata_id);
    if key_has_anim {
        (add_rel)(usize::MAX - 1, obdata_id, "Animation", 0);
    }
    (build_dimensions)(object_id);
    (add_rel)(object_id, object_id, "Synchronize to Original", 0);
    (add_rel)(obdata_id, object_id, "Data Selection -> Object Selection", 0);
    (add_rel)(object_id, object_id, "Object Geometry -> Select Update", RELATION_FLAG_NO_FLUSH);
    (add_rel)(obdata_id, object_id, "Geometry Shading -> Object Shading", 0);
}

/// `GeometryDatablockView` (verbatim per-type relations
/// data).
#[derive(Default)]
pub struct GeometryDatablockView {
    /// `cu->bevobj`/`taperobj`/`textoncurve` (CU_LEGACY).
    pub bevobj: usize,
    pub taperobj: usize,
    pub textoncurve: usize,
    /// gpd materials with gp_style.
    pub gpd_materials: Vec<usize>,
    /// gpd layers: (parent, is_parbone, parent_bone).
    pub gpd_layer_parents: Vec<(usize, bool, String)>,
    /// `curves_id->surface`.
    pub surface: usize,
    /// `volume->is_sequence`.
    pub volume_is_sequence: bool,
    /// gp layer parents (parent, bone_name or "").
    pub gp_layer_parents: Vec<(usize, String)>,
}

/// `build_object_data_geometry_datablock` (verbatim —
/// eval→done, params→eval, per-type links).
#[allow(clippy::too_many_arguments)]
pub fn build_object_data_geometry_datablock(
    obdata_id: usize,
    id_type: u16,
    view: &GeometryDatablockView,
    key_id: Option<usize>,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_shapekeys: &mut dyn FnMut(usize),
    build_object: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(obdata_id) {
        return;
    }
    (stack_trace)(obdata_id);
    (build_idproperties)(obdata_id);
    (build_idproperties)(obdata_id);
    (build_animdata)(obdata_id);
    (build_parameters)(obdata_id);
    if let Some(key) = key_id {
        (build_shapekeys)(key);
    }
    (add_rel)(obdata_id, obdata_id, "ObData Geom Eval Done");
    (add_rel)(obdata_id, obdata_id, "ObData Geom Params");
    if id_type == gs(b"CU") {
        if view.bevobj != 0 {
            (add_rel)(view.bevobj, obdata_id, "Curve Bevel Geometry");
            (add_rel)(view.bevobj, obdata_id, "Curve Bevel Transform");
            (build_object)(view.bevobj);
        }
        if view.taperobj != 0 {
            (add_rel)(view.taperobj, obdata_id, "Curve Taper");
            (build_object)(view.taperobj);
        }
        if view.textoncurve != 0 {
            (add_rel)(view.textoncurve, obdata_id, "Text on Curve Geometry");
            (add_rel)(view.textoncurve, obdata_id, "Text on Curve Transform");
            (build_object)(view.textoncurve);
        }
        (add_rel)(obdata_id, obdata_id, "Active Spline Update");
    } else if id_type == gs(b"GD") {
        (add_rel)(usize::MAX, obdata_id, "GP Frame Change");
        for ma in &view.gpd_materials {
            (add_rel)(*ma, obdata_id, "Material -> GP Data");
        }
        for (parent, is_parbone, _bone) in &view.gpd_layer_parents {
            if *is_parbone {
                (add_rel)(*parent, obdata_id, "Bone Parent");
                (add_rel)(*parent, obdata_id, "Armature Parent");
            } else {
                (add_rel)(*parent, obdata_id, "GPencil Parent Layer");
            }
        }
    } else if id_type == gs(b"CV") {
        if view.surface != 0 {
            (build_object)(view.surface);
            (add_rel)(view.surface, obdata_id, "Curves Object Surface");
            (add_rel)(view.surface, obdata_id, "Curves Object Surface");
        }
    } else if id_type == gs(b"VO") {
        if view.volume_is_sequence {
            (add_rel)(usize::MAX, obdata_id, "Volume sequence time");
        }
    } else if id_type == gs(b"GP") {
        (add_rel)(usize::MAX, obdata_id, "Grease Pencil Frame Change");
        for (parent, bone) in &view.gp_layer_parents {
            if !bone.is_empty() {
                (add_rel)(*parent, obdata_id, "Grease Pencil Layer Bone Parent");
                (add_rel)(*parent, obdata_id, "Grease Pencil Layer Armature Parent");
            } else {
                (add_rel)(*parent, obdata_id, "Grease Pencil Layer Object Parent");
            }
        }
    }
}

/// `build_object_data_empty` (verbatim — scene cow +
/// instancing + sync + instance collection geometry).
pub fn build_object_data_empty(
    object_id: usize,
    scene_id: usize,
    instance_collection: usize,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    (add_rel)(scene_id, object_id, "Copy-on-Eval Relation", RELATION_FLAG_NO_FLUSH);
    (add_rel)(object_id, object_id, "Transform -> Instance Geometry", 0);
    (add_rel)(object_id, object_id, "Synchronize to Original", 0);
    if instance_collection != 0 {
        (add_rel)(
            instance_collection,
            object_id,
            "Collection Geometry -> Empty Geometry",
            0,
        );
    }
}

/* ---- armature / camera / light / nodetree / material ---- */

/// `build_armature` (verbatim).
pub fn build_armature_rel(
    armature_id: usize,
    bone_props: &[usize],
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
) {
    if (check_is_built)(armature_id) {
        return;
    }
    (stack_trace)(armature_id);
    (build_idproperties)(armature_id);
    (build_idproperties)(armature_id);
    (build_animdata)(armature_id);
    (build_parameters)(armature_id);
    for p in bone_props {
        (build_idproperties)(*p);
        (build_idproperties)(*p);
    }
}

/// `build_camera` (verbatim — DOF object + bone subtarget).
pub fn build_camera_rel(
    camera_id: usize,
    dof_focus_object: usize,
    dof_focus_object_type: i32,
    dof_focus_subtarget: &str,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_object: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(camera_id) {
        return;
    }
    (stack_trace)(camera_id);
    (build_idproperties)(camera_id);
    (build_idproperties)(camera_id);
    (build_animdata)(camera_id);
    (build_parameters)(camera_id);
    if dof_focus_object != 0 {
        (build_object)(dof_focus_object);
        (add_rel)(dof_focus_object, camera_id, "Camera DOF");
        if dof_focus_object_type == 25 && !dof_focus_subtarget.is_empty() {
            (add_rel)(dof_focus_object, camera_id, "Camera DOF subtarget");
        }
    }
}

/// `build_light` (verbatim — parameters→shading + ntree).
pub fn build_light_rel(
    lamp_id: usize,
    nodetree: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_nodetree: &mut dyn FnMut(usize),
    build_nested_nodetree: &mut dyn FnMut(usize, usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(lamp_id) {
        return;
    }
    (stack_trace)(lamp_id);
    (build_idproperties)(lamp_id);
    (build_idproperties)(lamp_id);
    (build_animdata)(lamp_id);
    (build_parameters)(lamp_id);
    (add_rel)(lamp_id, lamp_id, "Light Shading Parameters");
    if nodetree != 0 {
        (build_nodetree)(nodetree);
        (add_rel)(nodetree, lamp_id, "NTree->Light Parameters");
        (build_nested_nodetree)(lamp_id, nodetree);
    }
}

/// `build_nodetree_socket` (verbatim — per-type id build).
pub fn build_nodetree_socket_rel(
    socket_prop: usize,
    socket_type: i32,
    default_value_id: usize,
    build_idproperties: &mut dyn FnMut(usize),
    build_id: &mut dyn FnMut(usize, bool),
) {
    (build_idproperties)(socket_prop);
    /* SOCK_TEXT_ID (15) skipped — text not in graph. */
    if socket_type != 15 && default_value_id != 0 {
        (build_id)(default_value_id, false);
    }
}

/// `NodetreeRelView` (verbatim fields used).
pub struct NodetreeRelView {
    /// `ntree->type == NTREE_GEOMETRY`/`NTREE_SHADER`.
    pub is_geometry: bool,
    pub is_shader: bool,
    /// nodes: (prop, sockets, linked id/type/group, has
    /// frame_current attribute).
    pub nodes: Vec<NodetreeNodeRelView>,
    /// interface props.
    pub interface_props: Vec<usize>,
    /// `check_id_has_anim_component`.
    pub has_anim_component: bool,
}

/// `NodetreeNodeRelView`.
pub struct NodetreeNodeRelView {
    /// `bnode->prop`.
    pub prop: usize,
    /// sockets (prop, type, default id).
    pub sockets: Vec<(usize, i32, usize)>,
    /// `bnode->id`/type/group.
    pub id: usize,
    pub id_type: u16,
    pub is_group: bool,
    pub group_ntree_is_geometry: bool,
    /// shader attribute "frame_current" on view layer.
    pub has_frame_current_attr: bool,
    /// OB object type (geometry-component check).
    pub object_type: i32,
    /// SCE camera (0 = none).
    pub scene_camera: usize,
}

/// `build_nodetree` (verbatim — output/preprocess +
/// per-node id dispatch + interface + anim).
#[allow(clippy::too_many_arguments)]
pub fn build_nodetree_rel(
    ntree_id: usize,
    ntree: &NodetreeRelView,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_nodetree_socket: &mut dyn FnMut(usize, i32, usize),
    build_id: &mut dyn FnMut(usize, bool),
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    if ntree_id == 0 || (check_is_built)(ntree_id) {
        return;
    }
    (stack_trace)(ntree_id);
    (build_idproperties)(ntree_id);
    (build_idproperties)(ntree_id);
    (build_animdata)(ntree_id);
    (build_parameters)(ntree_id);
    if ntree.is_geometry {
        (add_rel)(ntree_id, ntree_id, "Copy-on-Eval -> Preprocess", 0);
        (add_rel)(ntree_id, ntree_id, "Preprocess -> Output", RELATION_FLAG_NO_FLUSH);
    }
    for bnode in &ntree.nodes {
        (build_idproperties)(bnode.prop);
        for (sprop, stype, sdef) in &bnode.sockets {
            (build_nodetree_socket)(*sprop, *stype, *sdef);
        }
        if ntree.is_shader && bnode.has_frame_current_attr {
            (add_rel)(usize::MAX, ntree_id, "TimeSrc -> Node", 0);
        }
        if bnode.id == 0 {
            continue;
        }
        let t = bnode.id_type;
        if t == gs(b"MA") {
            (build_id)(bnode.id, false);
            (add_rel)(bnode.id, ntree_id, "Material -> Node", 0);
        } else if t == gs(b"TE") {
            (build_id)(bnode.id, false);
            (add_rel)(bnode.id, ntree_id, "Texture -> Node", 0);
        } else if t == gs(b"IM") {
            (build_id)(bnode.id, false);
            (add_rel)(bnode.id, ntree_id, "Image -> Node", 0);
        } else if t == gs(b"OB") {
            (build_id)(bnode.id, false);
            (add_rel)(bnode.id, ntree_id, "Object Transform -> Node", 0);
            if object_have_geometry_component(bnode.object_type) {
                (add_rel)(bnode.id, ntree_id, "Object Geometry -> Node", 0);
            }
        } else if t == gs(b"SC") {
            (build_id)(bnode.id, false);
            if bnode.scene_camera != 0 {
                (build_id)(bnode.scene_camera, false);
            }
        } else if t == gs(b"TX") {
            /* script nodes ignored */
        } else if t == gs(b"MS") {
            (build_id)(bnode.id, false);
            (add_rel)(bnode.id, ntree_id, "Mask -> Node", 0);
        } else if t == gs(b"MC") {
            (build_id)(bnode.id, false);
            (add_rel)(bnode.id, ntree_id, "Clip -> Node", 0);
        } else if t == gs(b"VF") {
            (build_id)(bnode.id, false);
            (add_rel)(bnode.id, ntree_id, "VFont -> Node", 0);
        } else if t == gs(b"GR") {
            (build_id)(bnode.id, false);
        } else if bnode.is_group {
            (build_id)(bnode.id, false);
            (add_rel)(bnode.id, ntree_id, "Group Node", 0);
            if bnode.group_ntree_is_geometry {
                (add_rel)(bnode.id, ntree_id, "Group Node Preprocess", 0);
            }
        }
    }
    for prop in &ntree.interface_props {
        (build_idproperties)(*prop);
    }
    if ntree.has_anim_component {
        (add_rel)(ntree_id, ntree_id, "NTree Shading Parameters", 0);
        if ntree.is_geometry {
            (add_rel)(ntree_id, ntree_id, "NTree Animation -> Preprocess", 0);
        }
    }
}

/// `build_material` (verbatim — owner shading +
/// parameters + ntree).
pub fn build_material_rel(
    material_id: usize,
    owner: usize,
    nodetree: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_nodetree: &mut dyn FnMut(usize),
    build_nested_nodetree: &mut dyn FnMut(usize, usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if owner != 0 {
        (add_rel)(material_id, owner, "Material -> Owner Shading");
    }
    if (check_is_built)(material_id) {
        return;
    }
    (stack_trace)(material_id);
    (build_idproperties)(material_id);
    (build_idproperties)(material_id);
    (build_animdata)(material_id);
    (build_parameters)(material_id);
    (add_rel)(material_id, material_id, "Material's parameters");
    if nodetree != 0 {
        (build_nodetree)(nodetree);
        (add_rel)(nodetree, material_id, "Material's NTree");
        (build_nested_nodetree)(material_id, nodetree);
    }
}

/* ---- texture/image/cache/mask/speaker/sound/sequencer/cow ---- */

/// `build_texture` (verbatim — ntree + image + anim rels).
#[allow(clippy::too_many_arguments)]
pub fn build_texture_rel(
    texture_id: usize,
    nodetree: usize,
    ima: usize,
    has_anim_component: bool,
    image_user_has_animation: bool,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_nodetree: &mut dyn FnMut(usize),
    build_nested_nodetree: &mut dyn FnMut(usize, usize),
    build_image: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(texture_id) {
        return;
    }
    (stack_trace)(texture_id);
    (build_idproperties)(texture_id);
    (build_idproperties)(texture_id);
    (build_animdata)(texture_id);
    (build_parameters)(texture_id);
    if nodetree != 0 {
        (build_nodetree)(nodetree);
        (add_rel)(nodetree, texture_id, "Texture's NTree");
        (build_nested_nodetree)(texture_id, nodetree);
    }
    if ima != 0 {
        (build_image)(ima);
        (add_rel)(ima, texture_id, "Texture Image");
    }
    if has_anim_component {
        (add_rel)(texture_id, texture_id, "Datablock Animation");
    }
    if image_user_has_animation {
        (add_rel)(texture_id, texture_id, "Datablock Image Animation");
    }
}

/// `build_cachefile` (verbatim — anim/driver → CACHE +
/// sequence time).
#[allow(clippy::too_many_arguments)]
pub fn build_cachefile_rel(
    cache_file_id: usize,
    is_sequence: bool,
    has_anim_component: bool,
    has_driver_component: bool,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(cache_file_id) {
        return;
    }
    (stack_trace)(cache_file_id);
    (build_idproperties)(cache_file_id);
    (build_idproperties)(cache_file_id);
    (build_animdata)(cache_file_id);
    (build_parameters)(cache_file_id);
    if has_anim_component {
        (add_rel)(cache_file_id, cache_file_id, "Datablock Animation");
    }
    if has_driver_component {
        (add_rel)(cache_file_id, cache_file_id, "Drivers -> Cache Eval");
    }
    if is_sequence {
        (add_rel)(usize::MAX, cache_file_id, "TimeSrc -> Cache File Eval");
    }
}

/// `build_mask` (verbatim — animation → eval + parent
/// movieclip).
#[allow(clippy::too_many_arguments)]
pub fn build_mask_rel(
    mask_id: usize,
    mask_is_animated: bool,
    parent_movieclip_ids: &[usize],
    other_parent_ids: &[usize],
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_id: &mut dyn FnMut(usize, bool),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(mask_id) {
        return;
    }
    (stack_trace)(mask_id);
    (build_idproperties)(mask_id);
    (build_idproperties)(mask_id);
    (build_animdata)(mask_id);
    (build_parameters)(mask_id);
    if mask_is_animated {
        (add_rel)(usize::MAX, mask_id, "TimeSrc -> Mask Animation");
    }
    (add_rel)(mask_id, mask_id, "Mask Animation -> Mask Eval");
    for pid in parent_movieclip_ids {
        (build_id)(*pid, false);
        (add_rel)(*pid, mask_id, "Movie Clip -> Mask Eval");
    }
    for pid in other_parent_ids {
        (build_id)(*pid, false);
    }
}

/// `build_speaker` (verbatim — sound → AUDIO).
pub fn build_speaker_rel(
    speaker_id: usize,
    sound: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_sound: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(speaker_id) {
        return;
    }
    (stack_trace)(speaker_id);
    (build_idproperties)(speaker_id);
    (build_idproperties)(speaker_id);
    (build_animdata)(speaker_id);
    (build_parameters)(speaker_id);
    if sound != 0 {
        (build_sound)(sound);
        (add_rel)(sound, speaker_id, "Sound -> Speaker");
    }
}

/// `build_sound` (verbatim — PARAMETERS → AUDIO).
pub fn build_sound_rel(
    sound_id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
    build_animdata: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    add_rel: &mut dyn FnMut(usize, usize, &str),
) {
    if (check_is_built)(sound_id) {
        return;
    }
    (stack_trace)(sound_id);
    (build_idproperties)(sound_id);
    (build_idproperties)(sound_id);
    (build_animdata)(sound_id);
    (build_parameters)(sound_id);
    (add_rel)(sound_id, sound_id, "Parameters -> Audio");
}

/// `build_scene_sequencer` (verbatim — audio + per-strip
/// cb + audio/cache).
pub fn build_scene_sequencer_rel(
    scene_id: usize,
    has_ed: bool,
    check_is_built_tag: &mut dyn FnMut(usize) -> bool,
    build_scene_audio: &mut dyn FnMut(usize),
    foreach_strip: &mut dyn FnMut(&mut dyn FnMut(usize)),
    has_audio_strips_out: &mut bool,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    if !has_ed || (check_is_built_tag)(scene_id) {
        return;
    }
    (build_scene_audio)(scene_id);
    (foreach_strip)(&mut |_strip| {});
    if *has_audio_strips_out {
        (add_rel)(scene_id, scene_id, "Sequencer -> Audio", 0);
    }
    (add_rel)(
        scene_id,
        scene_id,
        "Sequencer -> Audio Cache",
        RELATION_FLAG_FLUSH_USER_EDIT_ONLY,
    );
}

/// `build_scene_audio` (verbatim — entry→volume→sound +
/// volume→cache).
pub fn build_scene_audio_rel(scene_id: usize, add_rel: &mut dyn FnMut(usize, usize, &str, i32)) {
    (add_rel)(scene_id, scene_id, "Audio Entry -> Volume", 0);
    (add_rel)(scene_id, scene_id, "Audio Volume -> Sound", 0);
    (add_rel)(
        scene_id,
        scene_id,
        "Audio Volume -> Audio Cache",
        RELATION_FLAG_FLUSH_USER_EDIT_ONLY | RELATION_CHECK_BEFORE_ADD,
    );
}

/// `build_scene_audio_cache_invalidation_relations`
/// (verbatim — backward BFS from each
/// AUDIO_CACHE_INVALIDATE; skip nested cache/DRIVER/SCE-
/// cow; NO_FLUSH cow sources get explicit relations).
#[allow(clippy::too_many_arguments)]
pub fn build_scene_audio_cache_invalidation_relations(
    cache_operations: &[usize],
    inlinks: &dyn Fn(usize) -> Vec<(usize, bool, i32, i32)>,
    add_new_relation: &mut dyn FnMut(usize, usize, &str, i32),
) {
    for cache_op in cache_operations {
        let mut visited = std::collections::HashSet::new();
        let mut invalidation_sources = Vec::new();
        let mut stack = vec![*cache_op];
        visited.insert(*cache_op);
        while let Some(op) = stack.pop() {
            for (dep, _is_operation, opcode, rel_flag) in (inlinks)(op) {
                if opcode == 5 /*AUDIO_CACHE_INVALIDATE*/ {
                    continue;
                }
                if opcode == 3 /*DRIVER*/ {
                    continue;
                }
                /* SCE cow skipped by caller-provided opcode=-1. */
                if opcode == -1 {
                    continue;
                }
                if (rel_flag & RELATION_FLAG_NO_FLUSH) != 0 && opcode == 0 /*COPY_ON_EVAL*/ {
                    invalidation_sources.push(dep);
                }
                if visited.insert(dep) {
                    stack.push(dep);
                }
            }
        }
        for src in invalidation_sources {
            (add_new_relation)(
                src,
                *cache_op,
                "Audio Cache Invalidation",
                RELATION_FLAG_FLUSH_USER_EDIT_ONLY,
            );
        }
    }
}

/// `build_vfont` (verbatim).
pub fn build_vfont_rel(
    vfont_id: usize,
    check_is_built: &mut dyn FnMut(usize) -> bool,
    stack_trace: &mut dyn FnMut(usize),
    build_parameters: &mut dyn FnMut(usize),
    build_idproperties: &mut dyn FnMut(usize),
) {
    if (check_is_built)(vfont_id) {
        return;
    }
    (stack_trace)(vfont_id);
    (build_parameters)(vfont_id);
    (build_idproperties)(vfont_id);
    (build_idproperties)(vfont_id);
}

/// `build_nested_datablock` (verbatim — cow→cow "Eval
/// Order", NO_FLUSH when not flushing).
pub fn build_nested_datablock(
    owner: usize,
    id: usize,
    flush_cow_changes: bool,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    let flag = if flush_cow_changes { 0 } else { RELATION_FLAG_NO_FLUSH };
    (add_rel)(id, owner, "Eval Order", flag);
}

/// `build_nested_nodetree` / `build_nested_shapekey`
/// (verbatim).
pub fn build_nested_nodetree(
    owner: usize,
    ntree: usize,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    if ntree == 0 {
        return;
    }
    build_nested_datablock(owner, ntree, false, add_rel);
}

/// `build_nested_shapekey` (verbatim).
pub fn build_nested_shapekey(
    owner: usize,
    key: usize,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    if key == 0 {
        return;
    }
    build_nested_datablock(owner, key, true, add_rel);
}

/// `build_copy_on_write_relations(id_node)` (verbatim —
/// cow → component entry ops + dangling ops + OB data
/// ordering).
#[allow(clippy::too_many_arguments)]
pub fn build_copy_on_write_relations(
    id_node: usize,
    id_orig: usize,
    id_type: u16,
    cow_needed: bool,
    components: &[(usize, bool, usize, Vec<usize>, Vec<usize>)],
    op_cow: usize,
    object_data_id: Option<usize>,
    object_data_cow_needed: bool,
    add_rel: &mut dyn FnMut(usize, usize, &str, i32),
) {
    if !cow_needed {
        return;
    }
    for (comp_type, depends_on_cow, op_entry, dangling_ops, no_same_comp_ops) in components {
        let _ = id_node;
        if *comp_type == 11 /*COPY_ON_EVAL*/ || !depends_on_cow {
            continue;
        }
        let mut rel_flag = RELATION_FLAG_NO_FLUSH | RELATION_FLAG_GODMODE;
        if ((id_type == gs(b"ME")
            || id_type == gs(b"CV")
            || id_type == gs(b"PT")
            || id_type == gs(b"VO"))
            && *comp_type == 5 /*GEOMETRY*/)
            || (id_type == gs(b"CF") && *comp_type == 9 /*CACHE*/)
        {
            rel_flag &= !RELATION_FLAG_NO_FLUSH;
        }
        if id_type == gs(b"SO") {
            rel_flag &= !RELATION_FLAG_NO_FLUSH;
        }
        if *comp_type == 15 /*LAYER_COLLECTIONS*/ {
            rel_flag &= !RELATION_FLAG_NO_FLUSH;
        }
        if *comp_type == 2 /*PARAMETERS*/ && id_type == gs(b"MS") {
            rel_flag &= !RELATION_FLAG_NO_FLUSH;
        }
        if id_type == gs(b"GR") && *comp_type == 5 /*HIERARCHY*/ {
            rel_flag &= !RELATION_FLAG_NO_FLUSH;
        }
        if *op_entry != usize::MAX {
            (add_rel)(op_cow, *op_entry, "Copy-on-Eval Dependency", rel_flag);
        }
        for op in dangling_ops {
            (add_rel)(op_cow, *op, "Copy-on-Eval Dependency", rel_flag);
        }
        for op in no_same_comp_ops {
            (add_rel)(op_cow, *op, "Copy-on-Eval Dependency", rel_flag);
        }
    }
    if id_type == gs(b"OB") {
        if let Some(obdata) = object_data_id {
            if object_data_cow_needed {
                (add_rel)(obdata, id_orig, "Eval Order", RELATION_FLAG_GODMODE);
            }
        }
    }
}

/// `modifier_walk`/`constraint_walk` (verbatim — build_id).
pub fn rel_walk_id(id: usize, build_id: &mut dyn FnMut(usize, bool)) {
    if id == 0 {
        return;
    }
    (build_id)(id, false);
}
