//! Port of `intern/object_update.cc` — the depsgraph-driven object
//! eval orchestration: transform chain, per-type data update dispatch,
//! sync-to-original, batch-cache dirty tags, base flags, shading.
//!
//! The per-type heavy lifters (mesh_data_update, displist, particles,
//! constraints solver, light linking, DEG accessors) are injected via
//! `ObjectUpdateHooks`; matrix math and flag/dispatch logic are ported.

use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec3;

/// `eObjectType` (DNA_object_enums.h).
pub const OB_EMPTY: i16 = 0;
pub const OB_MESH: i16 = 1;
pub const OB_CURVES_LEGACY: i16 = 2;
pub const OB_SURF: i16 = 3;
pub const OB_FONT: i16 = 4;
pub const OB_MBALL: i16 = 5;
pub const OB_LATTICE: i16 = 9;
pub const OB_ARMATURE: i16 = 8;
pub const OB_CURVES: i16 = 11;
pub const OB_POINTCLOUD: i16 = 12;
pub const OB_VOLUME: i16 = 13;
pub const OB_GREASE_PENCIL: i16 = 14;

/// `eObjectTransFlag` bits used here.
pub const OB_NEG_SCALE: u32 = 1 << 13;
pub const OB_DUPLIPARTS: u32 = 1 << 14;

/// `eObjectMode` bit.
pub const OB_MODE_EDIT: u32 = 0x01;
pub const OB_MODE_PARTICLE_EDIT: u32 = 0x08;

/// `PARTYPE` mask / `PARSKEL` (DNA_object_types.h).
pub const PARTYPE: i32 = 15;
pub const PARSKEL: i32 = 0;

/// `BASE_*` flags (DNA_layer_types.h).
pub const BASE_ENABLED_RENDER: u32 = 1 << 9;
pub const BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT: u32 = 1 << 10;
pub const BASE_SELECTED: u32 = 1 << 1;
pub const BASE_SELECTABLE: u32 = 1 << 15;
pub const BASE_FROM_SET: u32 = 1 << 4;

/// `eMeshBatchDirtyMode` values this file passes.
pub const BKE_MESH_BATCH_DIRTY_ALL: i32 = 0;
pub const BKE_MESH_BATCH_DIRTY_SELECT: i32 = 1;
pub const BKE_LATTICE_BATCH_DIRTY_ALL: i32 = 0;
pub const BKE_CURVE_BATCH_DIRTY_ALL: i32 = 0;
pub const BKE_CURVES_BATCH_DIRTY_ALL: i32 = 0;
pub const BKE_POINTCLOUD_BATCH_DIRTY_ALL: i32 = 0;
pub const BKE_VOLUME_BATCH_DIRTY_ALL: i32 = 0;
pub const BKE_GREASEPENCIL_BATCH_DIRTY_ALL: i32 = 0;
pub const BKE_CURVE_BATCH_DIRTY_SELECT: i32 = 1;
pub const BKE_LATTICE_BATCH_DIRTY_SELECT: i32 = 1;

/// `GS(id->name)` two-letter codes → `ID_*` (DNA_ID.h).
pub const ID_ME: &str = "ME";
pub const ID_CU_LEGACY: &str = "CU";
pub const ID_LT: &str = "LT";

/// `PTCACHE_RESET_DEPSGRAPH`.
pub const PTCACHE_RESET_DEPSGRAPH: i32 = 1;

/// `DAG_EVAL_RENDER` (vs `DAG_EVAL_VIEWPORT`).
pub const DAG_EVAL_RENDER: i32 = 1;

/// Eval-time view of `Object` for this module.
pub struct ObjectEvalView {
    pub type_: i16,
    pub flag: u32,
    pub transflag: u32,
    pub mode: u32,
    /// `parent` ID handle (`0`/`NIL` = none).
    pub parent: usize,
    /// `parentinv` matrix (identity when unset).
    pub parentinv: Mat4,
    /// `constraints` list non-empty?
    pub has_constraints: bool,
    /// `particlesystem` non-empty?
    pub has_particles: bool,
    /// `instance_collection` handle.
    pub instance_collection: usize,
    /// `data` (ID handle: Mesh/Curve/Lattice/...).
    pub data: usize,
    /// `runtime->data_orig` handle.
    pub data_orig: usize,
    pub is_data_eval_owned: bool,
    /// `contained_geometry_types` bitmask (written by data update).
    pub contained_geometry_types: u16,
    /// `base_flag` / `base_local_view_bits` synced to orig.
    pub base_flag: u32,
    pub base_local_view_bits: u32,
}

impl Default for ObjectEvalView {
    fn default() -> Self {
        Self {
            type_: 0,
            flag: 0,
            transflag: 0,
            mode: 0,
            parent: 0,
            parentinv: Mat4::IDENTITY,
            has_constraints: false,
            has_particles: false,
            instance_collection: 0,
            data: 0,
            data_orig: 0,
            is_data_eval_owned: false,
            contained_geometry_types: 0,
            base_flag: 0,
            base_local_view_bits: 0,
        }
    }
}

impl ObjectEvalView {
    /// `Object::object_to_world` is `runtime->object_to_world` —
    /// tracked separately by the caller.
    pub fn has_parent(&self) -> bool {
        self.parent != 0
    }
}

/// Negative-determinant check for `is_negative_m4` — scale sign.
pub fn is_negative_m4(m: &Mat4) -> bool {
    /* det3 of upper-left 3x3. */
    let a = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    a < 0.0
}

/// `invert_m4_m4_safe` — inverse, or zero matrix on failure.
pub fn invert_m4_m4_safe(m: &Mat4) -> Mat4 {
    m.inverted().unwrap_or(Mat4::ZERO)
}

/// Hooks for systems this file orchestrates but which are ported
/// elsewhere or still pending.
pub struct ObjectUpdateHooks<'a> {
    /// `BKE_object_to_mat4` → local object matrix.
    pub object_to_mat4: &'a dyn Fn(usize) -> Mat4,
    /// `BKE_object_get_parent_matrix(ob, par)` → parent effect matrix.
    pub get_parent_matrix: &'a dyn Fn(usize, usize) -> Mat4,
    /// `DEG_get_update_count`.
    pub deg_update_count: &'a dyn Fn() -> u64,
    /// `DEG_get_mode` → `DAG_EVAL_*`.
    pub deg_mode: &'a dyn Fn() -> i32,
    /// `DEG_is_active`.
    pub deg_is_active: &'a dyn Fn() -> bool,
    /// `BKE_object_free_derived_caches`.
    pub free_derived_caches: &'a dyn Fn(usize),
    /// `BKE_constraints_make_evalob` + `BKE_constraints_solve` +
    /// `BKE_constraints_clear_evalob` collapsed into one call.
    pub solve_constraints: &'a dyn Fn(usize, f32),
    /// `BKE_scene_ctime_get`.
    pub scene_ctime: &'a dyn Fn() -> f32,
    /// Per-type data-update dispatch (mesh/lattice/curves/...); the
    /// `ob.data` handle and customdata mask semantics live inside.
    pub data_update: &'a dyn Fn(usize /*ob_handle*/, i16 /*ob_type*/, bool /*for_render*/),
    /// Particle system update (per psys handle).
    pub particle_system_update: &'a dyn Fn(usize /*psys*/, bool /*use_render_params*/),
    /// `GeometrySet` component-type gather → bitmask.
    pub gather_geometry_types: &'a dyn Fn(usize /*ob*/) -> u16,
    /// `BKE_object_evaluated_geometry_bounds` → orig bounds update.
    pub sync_eval_bounds: &'a dyn Fn(usize /*ob_orig*/),
    /// Per-type batch-cache dirty tag (`ob.data` handle, mode).
    pub batch_dirty_tag: &'a dyn Fn(usize /*data_handle*/, i16 /*ob_type*/, i32 /*mode*/),
    /// `deg::light_linking::eval_runtime_data`.
    pub eval_light_linking: &'a dyn Fn(usize),
    /// `BKE_ptcache_object_reset`.
    pub ptcache_reset: &'a dyn Fn(usize /*object*/, i32 /*reset_mode*/),
    /// `BKE_base_eval_flags` — recomputes `base.flag`; returns it.
    pub base_eval_flags: &'a dyn Fn(usize /*base*/) -> u32,
}

/// `BKE_object_eval_reset`.
pub fn bke_object_eval_reset(hooks: &ObjectUpdateHooks, ob_eval: usize) {
    (hooks.free_derived_caches)(ob_eval);
}

/// `BKE_object_eval_local_transform` — local matrix only.
pub fn bke_object_eval_local_transform(
    hooks: &ObjectUpdateHooks,
    ob: usize,
    object_to_world: &mut Mat4,
) {
    *object_to_world = (hooks.object_to_mat4)(ob);
}

/// `BKE_object_eval_parent` — `totmat * parentinv * locmat`;
/// `partype`/`parent_display_origin` updated.
pub fn bke_object_eval_parent(
    hooks: &ObjectUpdateHooks,
    ob: usize,
    partype: i32,
    locmat: Mat4,
    parentinv: &Mat4,
    object_to_world: &mut Mat4,
    parent_display_origin: &mut Vec3,
) {
    /* get parent effect matrix */
    let totmat = (hooks.get_parent_matrix)(ob, /* par resolved inside */ 0);
    /* total */
    let tmat = totmat * *parentinv;
    *object_to_world = tmat * locmat;

    /* origin, for help line */
    if (partype & PARTYPE) == PARSKEL {
        /* `par->object_to_world().location()` — caller passes the
         * parent's world matrix via the same hook; simplified to
         * totmat translation when no separate parent matrix given. */
        *parent_display_origin = Vec3::new(totmat[3][0], totmat[3][1], totmat[3][2]);
    } else {
        *parent_display_origin = Vec3::new(totmat[3][0], totmat[3][1], totmat[3][2]);
    }
}

/// `BKE_object_eval_constraints`.
pub fn bke_object_eval_constraints(
    hooks: &ObjectUpdateHooks,
    ob: usize,
    ctime: f32,
) {
    (hooks.solve_constraints)(ob, ctime);
}

/// `BKE_object_eval_transform_final` — keep inverse current +
/// negative-scale flag. Returns updated `transflag`.
pub fn bke_object_eval_transform_final(
    hooks: &ObjectUpdateHooks,
    object_to_world: &Mat4,
    world_to_object: &mut Mat4,
    transflag: &mut u32,
) {
    *world_to_object = invert_m4_m4_safe(object_to_world);
    if is_negative_m4(object_to_world) {
        *transflag |= OB_NEG_SCALE;
    } else {
        *transflag &= !OB_NEG_SCALE;
    }
    let _ = (hooks.deg_update_count)();
}

/// `BKE_object_eval_transform_all` — mimics the new-depsgraph
/// transform chain.
pub fn bke_object_eval_transform_all(
    hooks: &ObjectUpdateHooks,
    ob: usize,
    view: &ObjectEvalView,
    partype: i32,
    object_to_world: &mut Mat4,
    world_to_object: &mut Mat4,
    transflag: &mut u32,
    parent_display_origin: &mut Vec3,
) {
    bke_object_eval_local_transform(hooks, ob, object_to_world);
    if view.has_parent() {
        let locmat = *object_to_world;
        bke_object_eval_parent(
            hooks,
            ob,
            partype,
            locmat,
            &view.parentinv,
            object_to_world,
            parent_display_origin,
        );
    }
    if view.has_constraints {
        bke_object_eval_constraints(hooks, ob, (hooks.scene_ctime)());
    }
    /* `BKE_object_eval_uber_transform` — no-op in C. */
    bke_object_eval_transform_final(hooks, object_to_world, world_to_object, transflag);
}

/// `BKE_object_handle_data_update` — per-type dispatch. The
/// heavy per-type work is injected via `data_update`; particles and
/// `contained_geometry_types` bookkeeping are here.
pub fn bke_object_handle_data_update(
    hooks: &ObjectUpdateHooks,
    ob: usize,
    view: &mut ObjectEvalView,
) {
    let for_render = (hooks.deg_mode)() == DAG_EVAL_RENDER;

    match view.type_ {
        OB_EMPTY | OB_MESH | OB_ARMATURE | OB_MBALL | OB_CURVES_LEGACY | OB_SURF
        | OB_FONT | OB_LATTICE | OB_CURVES | OB_POINTCLOUD | OB_VOLUME
        | OB_GREASE_PENCIL => {
            (hooks.data_update)(ob, view.type_, for_render);
        }
        _ => {}
    }

    /* particles: `psys_check_enabled` / `PART_DRAW_REND` / dupli flag
     * decisions live inside the injected particle hook; here we only
     * keep the edit-mode guard. */
    if view.mode & OB_MODE_EDIT == 0 && view.has_particles {
        view.transflag &= !OB_DUPLIPARTS;
        (hooks.particle_system_update)(ob, for_render);
    }

    /* Cache the contained geometry types of the geometry_set_eval. */
    view.contained_geometry_types = (hooks.gather_geometry_types)(ob);

    if (hooks.deg_is_active)() {
        (hooks.sync_eval_bounds)(ob);
    }
}

/// `BKE_object_eval_uber_data` — assert not armature; update + dirty
/// tag + timestamp.
pub fn bke_object_eval_uber_data(
    hooks: &ObjectUpdateHooks,
    ob: usize,
    view: &mut ObjectEvalView,
) {
    debug_assert!(view.type_ != OB_ARMATURE);
    bke_object_handle_data_update(hooks, ob, view);
    bke_object_batch_cache_dirty_tag(hooks, view);
    let _last_update = (hooks.deg_update_count)();
}

/// `BKE_object_batch_cache_dirty_tag` — per-data-type dirty tag
/// (`BKE_*_BATCH_DIRTY_ALL`). `OB_MBALL` tags the evaluated mesh —
/// resolved inside the injected hook.
pub fn bke_object_batch_cache_dirty_tag(hooks: &ObjectUpdateHooks, view: &ObjectEvalView) {
    let mode = match view.type_ {
        OB_MESH => BKE_MESH_BATCH_DIRTY_ALL,
        OB_LATTICE => BKE_LATTICE_BATCH_DIRTY_ALL,
        OB_CURVES_LEGACY | OB_SURF | OB_FONT => BKE_CURVE_BATCH_DIRTY_ALL,
        OB_MBALL => BKE_MESH_BATCH_DIRTY_ALL,
        OB_CURVES => BKE_CURVES_BATCH_DIRTY_ALL,
        OB_POINTCLOUD => BKE_POINTCLOUD_BATCH_DIRTY_ALL,
        OB_VOLUME => BKE_VOLUME_BATCH_DIRTY_ALL,
        OB_GREASE_PENCIL => BKE_GREASEPENCIL_BATCH_DIRTY_ALL,
        _ => return,
    };
    (hooks.batch_dirty_tag)(view.data, view.type_, mode);
}

/// `BKE_object_eval_ptcache_reset`.
pub fn bke_object_eval_ptcache_reset(hooks: &ObjectUpdateHooks, ob: usize) {
    (hooks.ptcache_reset)(ob, PTCACHE_RESET_DEPSGRAPH);
}

/// `BKE_object_data_select_update` — `GS(id->name)` dispatch on the
/// DATA id (`ME`/`CU`/`LT` dirty-select; others no-op).
pub fn bke_object_data_select_update(
    hooks: &ObjectUpdateHooks,
    data_gs: &str,
    data_handle: usize,
) {
    let mode = match data_gs {
        ID_ME => BKE_MESH_BATCH_DIRTY_SELECT,
        ID_CU_LEGACY => BKE_CURVE_BATCH_DIRTY_SELECT,
        ID_LT => BKE_LATTICE_BATCH_DIRTY_SELECT,
        _ => return,
    };
    (hooks.batch_dirty_tag)(data_handle, 0, mode);
}

/// `BKE_object_select_update` — for an un-owned mesh eval, lock the
/// input mesh's `eval_mutex` (caller-side lock; hook-free flag).
pub fn bke_object_select_update(
    hooks: &ObjectUpdateHooks,
    view: &ObjectEvalView,
    data_gs: &str,
) {
    if view.type_ == OB_MESH && !view.is_data_eval_owned {
        /* `std::lock_guard lock{mesh_input->runtime->eval_mutex}` —
         * the caller synchronizes on the input mesh; we just route. */
        bke_object_data_select_update(hooks, data_gs, view.data_orig);
    } else {
        bke_object_data_select_update(hooks, data_gs, view.data);
    }
}

/// `BKE_object_eval_eval_base_flags` — copy base flags onto the
/// evaluated object and back to the orig base when appropriate.
///
/// `base` fields passed by value: returns the new `(base_flag,
/// base_orig_flag)` pair; `is_from_set` strips selectability.
#[allow(clippy::too_many_arguments)]
pub fn bke_object_eval_eval_base_flags(
    hooks: &ObjectUpdateHooks,
    view: &mut ObjectEvalView,
    base_handle: usize,
    is_from_set: bool,
    is_evaluated_view_layer: bool,
) -> u32 {
    /* Set base flags based on collection and object restriction. */
    let mut base_flag = (hooks.base_eval_flags)(base_handle);

    /* For render, compute base visibility again since
     * BKE_base_eval_flags assumed viewport visibility. */
    if (hooks.deg_mode)() == DAG_EVAL_RENDER {
        if base_flag & BASE_ENABLED_RENDER != 0 {
            base_flag |= BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT;
        } else {
            base_flag &= !BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT;
        }
    }

    /* Copy flags and settings from base. */
    view.base_flag = base_flag;
    if is_from_set {
        view.base_flag |= BASE_FROM_SET;
        view.base_flag &= !(BASE_SELECTED | BASE_SELECTABLE);
    }

    if view.mode == OB_MODE_PARTICLE_EDIT {
        /* `BKE_particle_batch_cache_dirty_tag(psys, ALL)` per psys —
         * routed through batch_dirty_tag with the object handle. */
        (hooks.batch_dirty_tag)(view.data, view.type_, 0);
    }

    /* Copy base flag back to the original view layer for editing. */
    if (hooks.deg_is_active)() && is_evaluated_view_layer {
        /* base_orig->flag = base->flag — caller writes it back with
         * the returned value. */
    }
    base_flag
}

/// `BKE_object_eval_light_linking`.
pub fn bke_object_eval_light_linking(hooks: &ObjectUpdateHooks, ob: usize) {
    (hooks.eval_light_linking)(ob);
}

/// `BKE_object_eval_shading` — stamps `last_update_shading`.
pub fn bke_object_eval_shading(hooks: &ObjectUpdateHooks) -> u64 {
    (hooks.deg_update_count)()
}

/// `BKE_object_sync_to_original` — copy flags/transforms/modifier
/// errors back to the original object.
///
/// Field copies are injected (`sync_orig_fields` handles base_flag,
/// local_view_bits, transforms, transflag/flag and per-modifier error
/// strings, and per-psys `imat` copies).
pub fn bke_object_sync_to_original(
    hooks: &ObjectUpdateHooks,
    view: &ObjectEvalView,
    sync_orig_fields: &dyn Fn(&ObjectEvalView),
) {
    if !(hooks.deg_is_active)() {
        return;
    }
    sync_orig_fields(view);
}
