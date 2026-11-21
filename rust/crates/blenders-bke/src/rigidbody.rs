//! Port of `intern/rigidbody.cc` — rigid-body world/object
//! management: flag consts, `world_init_runtime`,
//! `is_affected_by_simulation`, `check_sim_running`,
//! `sync_transforms` logic, `update_ob_array`,
//! `cache_reset`, substep data.
//!
//! `PORT_PENDING`: `RB_*` physics calls (Bullet —
//! `rb_dworld`/`rb_body`/`rb_shape`/`rb_constraint` are
//! injected traits); `rigidbody_validate_sim_*` (needs Mesh/
//! depsgraph-eval meshes); `calc_volume`/`center_of_mass`
//! (mesh tris → injected volumes callback); `collection_
//! validate`/`add_object_to_scene` (needs Main/Scene/CopyOnWrite).

/// `RBW_FLAG_MUTED`.
pub const RBW_FLAG_MUTED: u16 = 1 << 0;
/// `RBW_FLAG_USE_SPLIT_IMPULSE`.
pub const RBW_FLAG_USE_SPLIT_IMPULSE: u16 = 1 << 2;
/// `RBO_TYPE_ACTIVE`.
pub const RBO_TYPE_ACTIVE: i8 = 0;
/// `RBO_TYPE_PASSIVE`.
pub const RBO_TYPE_PASSIVE: i8 = 1;
/// `RBO_FLAG_KINEMATIC`.
pub const RBO_FLAG_KINEMATIC: i32 = 1 << 0;
/// `RBO_FLAG_NEEDS_VALIDATE`.
pub const RBO_FLAG_NEEDS_VALIDATE: i32 = 1 << 1;
/// `RBO_FLAG_NEEDS_RESHAPE`.
pub const RBO_FLAG_NEEDS_RESHAPE: i32 = 1 << 2;
/// `RBO_FLAG_USE_DEACTIVATION`.
pub const RBO_FLAG_USE_DEACTIVATION: i32 = 1 << 3;
/// `RBO_FLAG_START_DEACTIVATED`.
pub const RBO_FLAG_START_DEACTIVATED: i32 = 1 << 4;
/// `RBO_FLAG_DISABLED`.
pub const RBO_FLAG_DISABLED: i32 = 1 << 5;
/// `RBO_FLAG_USE_MARGIN`.
pub const RBO_FLAG_USE_MARGIN: i32 = 1 << 6;
/// `RBO_FLAG_USE_DEFORM`.
pub const RBO_FLAG_USE_DEFORM: i32 = 1 << 7;
/// `RB_SHAPE_COMPOUND` (shape enum — box/sphere/etc live in
/// `rigidbody_types`).
pub const RB_SHAPE_COMPOUND: i8 = 10;

/// `RigidBodyWorld` runtime state (verbatim fields, handles).
#[derive(Default)]
pub struct RigidBodyWorldRuntime {
    /// `flag`.
    pub flag: u16,
    /// `numbodies` + `objects` — cached object array from
    /// `update_ob_array`.
    pub numbodies: i32,
    /// `objects`.
    pub objects: Vec<usize>,
    /// `pointcache` handle.
    pub pointcache: usize,
    /// `pointcache_startframe` (cache->startframe).
    pub startframe: f32,
    /// `effector_weights` handle.
    pub effector_weights: usize,
    /// `group` (Collection handle).
    pub group: usize,
    /// `constraints` (Collection handle).
    pub constraints: usize,
    /// `k_pos`, `k_rot` — animated locations (substep data).
    pub k_pos: Vec<[f32; 3]>,
    /// `k_rot`.
    pub k_rot: Vec<[f32; 4]>,
}

/// `BKE_rigidbody_is_affected_by_simulation` (verbatim;
/// accessors injected).
pub fn rigidbody_is_affected_by_simulation(
    rbo_flag: i32,
    rbo_type: i8,
    rbo_present: bool,
    parent_has_compound: bool,
) -> bool {
    if !rbo_present
        || (rbo_flag & RBO_FLAG_KINEMATIC) != 0
        || rbo_type == RBO_TYPE_PASSIVE
        || parent_has_compound
    {
        return false;
    }
    true
}

/// `rigidbody_update_ob_array` — rebuild `objects` from
/// `group`, skipping compound children (verbatim; the
/// `FOREACH_COLLECTION_OBJECT_RECURSIVE` walk injected).
pub fn rigidbody_update_ob_array(
    rbw: &mut RigidBodyWorldRuntime,
    objects_in_group: &dyn Fn(&mut dyn FnMut(usize, bool)),
) {
    /* `objects_in_group(cb)` calls `cb(object_handle,
     * parent_is_compound_child)`. */
    let mut list = Vec::new();
    objects_in_group(&mut |ob, parent_is_compound_child| {
        let _ = ob;
        if !parent_is_compound_child {
            list.push(ob);
        }
    });
    rbw.numbodies = list.len() as i32;
    rbw.objects = list;
}

/// `BKE_rigidbody_check_sim_running` (verbatim).
pub fn rigidbody_check_sim_running(
    rbw_flag: u16,
    ctime: f32,
    pointcache_startframe: f32,
) -> bool {
    (rbw_flag & RBW_FLAG_MUTED) == 0 && ctime > pointcache_startframe
}

/// `rigidbody_update_sim_world` — adj_gravity =
/// scene gravity × `global_gravity * weight[0]` when
/// `PHYS_GLOBAL_GRAVITY` (verbatim).
pub fn rigidbody_update_sim_world(
    phys_flag_global_gravity: bool,
    gravity: [f32; 3],
    global_gravity: f32,
    weight0: f32,
) -> [f32; 3] {
    if phys_flag_global_gravity {
        [
            gravity[0] * global_gravity * weight0,
            gravity[1] * global_gravity * weight0,
            gravity[2] * global_gravity * weight0,
        ]
    } else {
        [0.0; 3]
    }
}

/// `BKE_rigidbody_sync_transforms` — return whether the
/// object's transform should come from the simulation
/// (verbatim predicate part; the actual `RB_body_set_*
/// transforms` / `obmat` writes are injected).
pub fn rigidbody_sync_transforms_check(
    is_affected: bool,
    sim_running: bool,
    base_selected_and_moving: bool,
) -> bool {
    is_affected && sim_running && !base_selected_and_moving
}

/// `rigidbody_create_substep_data` — animated pos/rot
/// vectors sized `numbodies` (verbatim `LinkData` rows).
pub fn rigidbody_create_substep_data(numbodies: i32) -> (Vec<[f32; 3]>, Vec<[f32; 4]>) {
    let n = numbodies.max(0) as usize;
    (vec![[0.0; 3]; n], vec![[0.0; 4]; n])
}

/// `BKE_rigidbody_cache_reset` — `PTCACHE_OUTDATED` via the
/// injected flag-mutator.
pub fn rigidbody_cache_reset(flag_mut: &mut dyn FnMut(u32)) {
    /* `rbw->shared->pointcache->flag |= PTCACHE_OUTDATED` */
    flag_mut(PTCACHE_OUTDATED);
}
/// `PTCACHE_OUTDATED` (DNA_pointcache_types verbatim).
pub const PTCACHE_OUTDATED: u32 = 1 << 4;
/// `PHYS_GLOBAL_GRAVITY` (DNA_scene_types verbatim).
pub const PHYS_GLOBAL_GRAVITY: i32 = 1 << 1;
