//! Port of `intern/eval/deg_eval_runtime_backup_object.cc` —
/// `ObjectRuntimeBackup` init/restore.

use crate::runtime_backup::ModifierDataBackup;
use crate::depsgraph_light_linking::LightLinkingRuntime;
use crate::depsgraph_tag::{OB_CURVES, OB_CURVES_LEGACY, OB_FONT, OB_GREASE_PENCIL,
                           OB_LATTICE, OB_MESH, OB_POINTCLOUD, OB_VOLUME};
use blenders_dna::id::ID_RECALC_GEOMETRY;
use std::collections::HashMap;

/// Field-level handle accessors injected for the object
/// (kept verbatim; Object/ObjectRuntime are foreign
/// DNA/runtime types keyed by usize).
pub struct ObjectBackupHooks<'a> {
    /// `object->runtime` snapshot — opaque verbatim copy.
    pub object_runtime_clone: &'a dyn Fn(usize) -> usize,
    /// `BKE_object_runtime_reset(object)` then
    /// `runtime->bounds_eval = bounds` restore.
    pub object_runtime_reset_keep_bounds: &'a mut dyn FnMut(usize),
    /// `object->runtime->data_orig`.
    pub runtime_data_orig: &'a dyn Fn(usize) -> usize,
    /// `object->runtime->data_eval`.
    pub runtime_data_eval: &'a dyn Fn(usize) -> usize,
    /// `object->data = id`.
    pub set_object_data: &'a mut dyn FnMut(usize, usize),
    /// `object->id.recalc`.
    pub id_recalc: &'a dyn Fn(usize) -> u32,
    /// `object->type`.
    pub ob_type: &'a dyn Fn(usize) -> i16,
    /// `object->light_linking->runtime` (if light_linking).
    pub light_linking_runtime: &'a dyn Fn(usize) -> Option<LightLinkingRuntime>,
    /// `BKE_light_linking_ensure` + assign.
    pub light_linking_restore: &'a mut dyn FnMut(usize, LightLinkingRuntime),
    /// `object->base_flag`/`base_local_view_bits` get/set.
    pub base_flag: &'a mut dyn FnMut(usize, Option<i32>) -> i32,
    /// `base_local_view_bits` get/set.
    pub base_local_view_bits: &'a mut dyn FnMut(usize, Option<u32>) -> u32,
    /// Modifiers: (persistent_uid, ModifierDataBackup).
    pub modifiers: &'a dyn Fn(usize) -> Vec<(i32, ModifierDataBackup)>,
    /// `modifier_data.runtime = backup.runtime` restore.
    pub modifier_restore_runtime: &'a mut dyn FnMut(usize, i32, usize),
    /// `BKE_modifier_get_info(type)->free_runtime_data(runtime)`.
    pub modifier_free_runtime: &'a mut dyn FnMut(i32, usize),
    /// Pose channels: (session_uid, runtime-copy handle).
    pub pose_channels: &'a dyn Fn(usize) -> Vec<(u64, usize)>,
    /// `pchan.runtime = backup` restore.
    pub pchan_restore_runtime: &'a mut dyn FnMut(usize, u64, usize),
    /// `BKE_pose_channel_runtime_free` for leftover backups.
    pub pchan_runtime_free: &'a mut dyn FnMut(usize),
    /// `BKE_pose_channel_runtime_reset` after backup.
    pub pchan_runtime_reset: &'a mut dyn FnMut(usize, u64),
    /// `BKE_object_free_derived_caches`.
    pub free_derived_caches: &'a mut dyn FnMut(usize),
    /// Mesh-eval edit_mesh pointer fix-up
    /// (`mesh_eval->runtime->edit_mesh = mesh_orig->runtime->edit_mesh`).
    pub fix_mesh_edit_mesh: &'a mut dyn FnMut(usize, usize),
    /// `object->data` read (for the GS compare).
    pub object_data: &'a dyn Fn(usize) -> usize,
    /// `GS(id->name)` for an ID.
    pub id_gs: &'a dyn Fn(usize) -> u16,
    /// Subsurf runtime dangling fix
    /// (`mesh->runtime->subsurf_runtime_data == backup.runtime`).
    pub fix_subsurf_runtime: &'a mut dyn FnMut(usize, usize),
}

/// `ObjectRuntimeBackup` (verbatim).
pub struct ObjectRuntimeBackup {
    /// `runtime` (opaque evaluated `ObjectRuntime` copy).
    pub runtime: usize,
    /// `light_linking_runtime`.
    pub light_linking_runtime: Option<LightLinkingRuntime>,
    /// `base_flag`.
    pub base_flag: i32,
    /// `base_local_view_bits`.
    pub base_local_view_bits: u32,
    /// `modifier_runtime_data` keyed by `persistent_uid`.
    pub modifier_runtime_data: HashMap<i32, ModifierDataBackup>,
    /// `pose_channel_runtime_data` keyed by `session_uid`.
    pub pose_channel_runtime_data: HashMap<u64, usize>,
}

impl ObjectRuntimeBackup {
    /// `ObjectRuntimeBackup(depsgraph)` (verbatim).
    pub fn new() -> Self {
        Self {
            runtime: 0,
            light_linking_runtime: None,
            base_flag: 0,
            base_local_view_bits: 0,
            modifier_runtime_data: HashMap::new(),
            pose_channel_runtime_data: HashMap::new(),
        }
    }

    /// `init_from_object` (verbatim).
    pub fn init_from_object(&mut self, object: usize, h: &mut ObjectBackupHooks) {
        /* Store evaluated runtime; ensure it's not freed. */
        self.runtime = (h.object_runtime_clone)(object);
        self.light_linking_runtime = (h.light_linking_runtime)(object);
        (h.object_runtime_reset_keep_bounds)(object);
        (h.set_object_data)(object, (h.runtime_data_orig)(object));
        self.base_flag = (h.base_flag)(object, None);
        self.base_local_view_bits = (h.base_local_view_bits)(object, None);
        /* `backup_modifier_runtime_data` (verbatim). */
        for (uid, backup) in (h.modifiers)(object) {
            self.modifier_runtime_data.insert(uid, backup);
        }
        /* `backup_pose_channel_runtime_data` (verbatim). */
        for (uid, rt) in (h.pose_channels)(object) {
            self.pose_channel_runtime_data.insert(uid, rt);
            (h.pchan_runtime_reset)(object, uid);
        }
    }

    /// `restore_to_object` (verbatim).
    pub fn restore_to_object(&mut self, object: usize, h: &mut ObjectBackupHooks) {
        let data_orig = (h.runtime_data_orig)(object);
        let data_eval = (h.runtime_data_eval)(object);
        let ob_type = (h.ob_type)(object);
        if matches!(ob_type, OB_MESH | OB_LATTICE | OB_CURVES_LEGACY | OB_FONT)
            && data_eval != 0
        {
            if (h.id_recalc)(object) & ID_RECALC_GEOMETRY != 0 {
                /* Geometry tagged for update: restore orig +
                 * free invalidated caches. */
                (h.set_object_data)(object, data_orig);
                (h.free_derived_caches)(object);
            } else {
                /* Override data pointer with evaluated version
                 * when types match. */
                if (h.id_gs)((h.object_data)(object)) == (h.id_gs)(data_eval) {
                    (h.set_object_data)(object, data_eval);
                }
                if ob_type == OB_MESH {
                    (h.fix_mesh_edit_mesh)(data_eval, data_orig);
                }
            }
        } else if matches!(ob_type, OB_CURVES | OB_POINTCLOUD | OB_VOLUME | OB_GREASE_PENCIL)
        {
            if (h.id_recalc)(object) & ID_RECALC_GEOMETRY != 0 {
                (h.set_object_data)(object, data_orig);
                (h.free_derived_caches)(object);
            } else {
                (h.set_object_data)(object, data_eval);
            }
        }
        if let Some(rt) = self.light_linking_runtime {
            (h.light_linking_restore)(object, rt);
        }
        (h.base_flag)(object, Some(self.base_flag));
        (h.base_local_view_bits)(object, Some(self.base_local_view_bits));
        /* `restore_modifier_runtime_data` (verbatim). */
        for (uid, _) in (h.modifiers)(object) {
            if let Some(backup) = self.modifier_runtime_data.remove(&uid) {
                (h.modifier_restore_runtime)(object, uid, backup.runtime);
            }
        }
        for (_, backup) in self.modifier_runtime_data.drain() {
            (h.modifier_free_runtime)(backup.r#type, backup.runtime);
            (h.fix_subsurf_runtime)(object, backup.runtime);
        }
        /* `restore_pose_channel_runtime_data` (verbatim). */
        for (uid, _) in (h.pose_channels)(object) {
            if let Some(rt) = self.pose_channel_runtime_data.remove(&uid) {
                (h.pchan_restore_runtime)(object, uid, rt);
            }
        }
        for (_, rt) in self.pose_channel_runtime_data.drain() {
            (h.pchan_runtime_free)(rt);
        }
    }
}
