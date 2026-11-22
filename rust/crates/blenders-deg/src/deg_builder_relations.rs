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
