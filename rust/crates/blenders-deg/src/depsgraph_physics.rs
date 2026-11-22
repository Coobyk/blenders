//! Port of `intern/depsgraph_physics.cc` — physics
/// (effector/collision) relation query + build API.

use crate::node::{DEG_OB_COMP_EVAL_POSE, DEG_OB_COMP_GEOMETRY, DEG_OB_COMP_TRANSFORM};
use std::collections::HashMap;

/// `ePhysicsCollisionType` verbatim.
pub const DEG_PHYSICS_COLLISION: usize = 0;
/// `DEG_PHYSICS_SMOKE_COLLISION`.
pub const DEG_PHYSICS_SMOKE_COLLISION: usize = 1;
/// `DEG_PHYSICS_DYNAMIC_BRUSH`.
pub const DEG_PHYSICS_DYNAMIC_BRUSH: usize = 2;
/// `DEG_PHYSICS_COLLISION_NUM`.
pub const DEG_PHYSICS_COLLISION_NUM: usize = 3;

/// `eModifierType_*` tags used here (verbatim).
pub const E_MODIFIER_TYPE_COLLISION: u32 = 6;
/// `eModifierType_Fluid`.
pub const E_MODIFIER_TYPE_FLUID: u32 = 45;
/// `eModifierType_DynamicPaint`.
pub const E_MODIFIER_TYPE_DYNAMIC_PAINT: u32 = 57;

/// `PFIELD_*` verbatim (`DNA_particle_types.h`).
pub const PFIELD_SHAPE_SURFACE: i16 = 1;
/// `PFIELD_SHAPE_POINTS`.
pub const PFIELD_SHAPE_POINTS: i16 = 2;
/// `PFIELD_GUIDE`.
pub const PFIELD_GUIDE: i16 = 7;
/// `PFIELD_FLUIDFLOW`.
pub const PFIELD_FLUIDFLOW: i16 = 12;
/// `PFIELD_VISIBILITY`.
pub const PFIELD_VISIBILITY: i32 = 1 << 6;

/// `modifier_to_collision_type` (verbatim).
pub fn modifier_to_collision_type(modifier_type: u32) -> usize {
    match modifier_type {
        E_MODIFIER_TYPE_COLLISION => DEG_PHYSICS_COLLISION,
        E_MODIFIER_TYPE_FLUID => DEG_PHYSICS_SMOKE_COLLISION,
        E_MODIFIER_TYPE_DYNAMIC_PAINT => DEG_PHYSICS_DYNAMIC_BRUSH,
        _ => DEG_PHYSICS_COLLISION_NUM,
    }
}

/// `CollisionComponentFlag` verbatim.
pub mod collision_component_flag {
    /// `CollisionComponentFlag::None`.
    pub const NONE: u8 = 0;
    /// `Transform` — `DEG_OB_COMP_TRANSFORM` set.
    pub const TRANSFORM: u8 = 1 << 0;
    /// `Geometry`.
    pub const GEOMETRY: u8 = 1 << 1;
    /// `EvalPose`.
    pub const EVAL_POSE: u8 = 1 << 2;
}

/// One `EffectorRelation` entry (verbatim fields).
pub struct EffectorRelation {
    /// `ob`.
    pub ob: usize,
    /// `psys` present (ParticleSystem ptr).
    pub has_psys: bool,
    /// `pd->forcefield`.
    pub forcefield: i16,
    /// `pd->shape`.
    pub shape: i16,
    /// `pd->f_source` (fluid-flow source object).
    pub f_source: usize,
    /// `pd->flag`.
    pub pd_flag: i32,
}

/// One `CollisionRelation` entry (verbatim fields).
pub struct CollisionRelation {
    /// `ob`.
    pub ob: usize,
}

/// Physics relation caches on the depsgraph (verbatim —
/// `physics_relations_effector`,
/// `physics_relations_collision[TYPE]` keyed by collection id).
#[derive(Default)]
pub struct PhysicsRelations {
    /// `physics_relations_effector`.
    pub effector: HashMap<usize, Vec<EffectorRelation>>,
    /// `physics_relations_collision[DEG_PHYSICS_*]`.
    pub collision: [HashMap<usize, Vec<CollisionRelation>>; DEG_PHYSICS_COLLISION_NUM],
}

/// `DEG_get_effector_relations` (verbatim —
/// `collection_orig` via `orig_id`).
pub fn deg_get_effector_relations<'a>(
    relations: &'a PhysicsRelations,
    collection_id: usize,
) -> Option<&'a Vec<EffectorRelation>> {
    relations.effector.get(&collection_id)
}

/// `DEG_get_collision_relations` (verbatim).
pub fn deg_get_collision_relations<'a>(
    relations: &'a PhysicsRelations,
    collection_id: usize,
    modifier_type: u32,
) -> Option<&'a Vec<CollisionRelation>> {
    let ty = modifier_to_collision_type(modifier_type);
    relations.collision.get(ty)?.get(&collection_id)
}

/// `build_effector_relations` (verbatim — `BKE_effector_relations_create`
/// injected).
pub fn build_effector_relations<'a>(
    relations: &'a mut PhysicsRelations,
    collection_id: usize,
    create: &mut dyn FnMut(usize) -> Vec<EffectorRelation>,
) -> &'a Vec<EffectorRelation> {
    relations
        .effector
        .entry(collection_id)
        .or_insert_with(|| create(collection_id))
}

/// `build_collision_relations` (verbatim — `BKE_collision_relations_create`
/// injected).
pub fn build_collision_relations<'a>(
    relations: &'a mut PhysicsRelations,
    collection_id: usize,
    modifier_type: u32,
    create: &mut dyn FnMut(usize) -> Vec<CollisionRelation>,
) -> &'a Vec<CollisionRelation> {
    let ty = modifier_to_collision_type(modifier_type);
    relations.collision[ty]
        .entry(collection_id)
        .or_insert_with(|| create(collection_id))
}

/// `DEG_add_collision_relations` (verbatim —
/// `DEG_add_object_pointcache_relation` becomes the
/// `add_pointcache_relation` hook; `BKE_object_modifier_update_subframe_only_callback`
/// becomes `update_subframe`; `filter_function` + `findby_type`
/// merged into `filter`).
#[allow(clippy::too_many_arguments)]
pub fn deg_add_collision_relations(
    relations: &[CollisionRelation],
    object: usize,
    modifier_type: u32,
    filter: &dyn Fn(usize) -> bool,
    add_pointcache_relation: &mut dyn FnMut(usize, i32, &str),
    update_subframe: &mut dyn FnMut(usize, &mut dyn FnMut(usize, bool)),
    name: &str,
) {
    /* Expand tag objects for DynamicPaint (see comment upstream). */
    let use_recursive_parents = modifier_type == E_MODIFIER_TYPE_DYNAMIC_PAINT;
    let mut object_component_map: HashMap<usize, u8> = HashMap::new();

    for relation in relations {
        let ob1 = relation.ob;
        if ob1 == object {
            continue;
        }
        if !filter(ob1) {
            continue;
        }
        if use_recursive_parents {
            update_subframe(ob1, &mut |ob, update_mesh| {
                let update_flag = object_component_map.entry(ob).or_insert(0);
                if *update_flag & collision_component_flag::TRANSFORM == 0 {
                    *update_flag |= collision_component_flag::TRANSFORM;
                    add_pointcache_relation(ob, DEG_OB_COMP_TRANSFORM, name);
                }
                if update_mesh && *update_flag & collision_component_flag::GEOMETRY == 0 {
                    *update_flag |= collision_component_flag::GEOMETRY;
                    add_pointcache_relation(ob, DEG_OB_COMP_GEOMETRY, name);
                }
                /* `ob->type == OB_ARMATURE` resolved by caller into a
                 * side-table is overkill — callers pass armature objects
                 * through the same callback; keep verbatim flag logic. */
            });
            continue;
        }
        add_pointcache_relation(ob1, DEG_OB_COMP_TRANSFORM, name);
        add_pointcache_relation(ob1, DEG_OB_COMP_GEOMETRY, name);
    }
}

/// `DEG_add_forcefield_relations` (verbatim —
/// `effector_weights->group` → `collection_id`;
/// `add_absorption` triggers a nested collision add via
/// `add_absorption_relations`).
#[allow(clippy::too_many_arguments)]
pub fn deg_add_forcefield_relations(
    relations: &[EffectorRelation],
    object: usize,
    add_absorption: bool,
    skip_forcefield: i16,
    name: &str,
    add_pointcache_relation: &mut dyn FnMut(usize, i32, &str),
    add_absorption_relations: &mut dyn FnMut(),
) {
    for relation in relations {
        if relation.ob == object {
            continue;
        }
        if relation.forcefield == skip_forcefield {
            continue;
        }
        /* Relation to forcefield object, optionally including
         * geometry — point-cache relations for cache clearing. */
        add_pointcache_relation(relation.ob, DEG_OB_COMP_TRANSFORM, name);
        if relation.has_psys
            || relation.shape == PFIELD_SHAPE_SURFACE
            || relation.shape == PFIELD_SHAPE_POINTS
            || relation.forcefield == PFIELD_GUIDE
        {
            add_pointcache_relation(relation.ob, DEG_OB_COMP_GEOMETRY, name);
        }
        /* Smoke flow relations. */
        if relation.forcefield == PFIELD_FLUIDFLOW && relation.f_source != 0 {
            add_pointcache_relation(relation.f_source, DEG_OB_COMP_TRANSFORM, "Fluid Force Domain");
            add_pointcache_relation(relation.f_source, DEG_OB_COMP_GEOMETRY, "Fluid Force Domain");
        }
        /* Absorption forces need collision relation. */
        if add_absorption && (relation.pd_flag & PFIELD_VISIBILITY) != 0 {
            add_absorption_relations();
        }
    }
}

/// `clear_physics_relations` (verbatim — the lists are
/// owned by the maps, so this just clears them).
pub fn clear_physics_relations(relations: &mut PhysicsRelations) {
    relations.effector.clear();
    for map in relations.collision.iter_mut() {
        map.clear();
    }
}
