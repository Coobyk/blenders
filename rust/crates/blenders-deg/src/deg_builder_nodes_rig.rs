//! Port of `intern/builder/deg_builder_nodes_rig.cc` —
/// pose/rig node building.

/// `POSE_RECALC`/`POSE_CONSTRAINTS_NEED_UPDATE_FLAGS`
/// (`DNA_action_types.h` verbatim).
pub const POSE_RECALC: i32 = 1 << 1;
pub const POSE_CONSTRAINTS_NEED_UPDATE_FLAGS: i32 = 1 << 0;
/// `CONSTRAINT_TYPE_KINEMATIC`/`_SPLINEIK` (verbatim).
pub const CONSTRAINT_TYPE_KINEMATIC: i32 = 1;
pub const CONSTRAINT_TYPE_SPLINEIK: i32 = 10;
/// `DEG_ID_LINKED_INDIRECTLY` (verbatim).
pub const DEG_ID_LINKED_INDIRECTLY: i32 = 1;

/// Per-pchan row for `build_rig`.
pub struct PchanRow {
    /// `pchan.name`.
    pub name: String,
    /// `pchan.prop` handle (0 = none).
    pub prop: usize,
    /// `pchan.system_properties` handle.
    pub system_properties: usize,
    /// `pchan.constraints.first() != nullptr`.
    pub has_constraints: bool,
    /// `pchan.constraints` — (constraint-type, con handle)
    /// pairs for IK dispatch.
    pub constraints: Vec<(i32, usize)>,
    /// `pchan.custom` handle.
    pub custom: usize,
    /// `check_pchan_has_bbone` result (precomputed).
    pub has_bbone: bool,
}

/// Injected calls for rig building.
pub struct RigNodeBuildHooks<'a> {
    /// `BKE_constraints_id_loop` over a pchan's constraints
    /// (`constraint_walk` callback inside).
    pub constraints_id_loop: &'a mut dyn FnMut(usize),
    /// `add_operation_node` (id, node_type, opcode;
    /// or id, node_type, name, opcode) → op handle.
    pub add_operation_node: &'a mut dyn FnMut(usize, usize, &str, usize) -> usize,
    /// `op_node->set_as_entry`/`set_as_exit`.
    pub set_as_entry: &'a mut dyn FnMut(usize),
    /// `set_as_exit`.
    pub set_as_exit: &'a mut dyn FnMut(usize),
    /// `has_operation_node`.
    pub has_operation_node: &'a dyn Fn(usize, usize, &str, usize) -> bool,
    /// `BKE_armature_ik_solver_find_root`.
    pub ik_solver_find_root: &'a dyn Fn(usize, usize) -> Option<usize>,
    /// `BKE_armature_splineik_solver_find_root`.
    pub splineik_solver_find_root: &'a dyn Fn(usize, usize) -> Option<usize>,
    /// `BLI_findindex` for a pchan in `chanbase`.
    pub pchan_index_of: &'a dyn Fn(usize) -> i32,
    /// `build_armature`.
    pub build_armature: &'a mut dyn FnMut(usize),
    /// `BKE_pose_rebuild` / `BKE_pose_ensure_bone_indices`.
    pub pose_rebuild: &'a mut dyn FnMut(usize, usize),
    /// `BKE_pose_ensure_bone_indices`.
    pub pose_ensure_bone_indices: &'a mut dyn FnMut(usize),
    /// `BKE_pose_channels_hash_ensure`.
    pub pose_channels_hash_ensure: &'a mut dyn FnMut(usize),
    /// `BKE_pose_update_constraint_flags`.
    pub pose_update_constraint_flags: &'a mut dyn FnMut(usize),
    /// `build_idproperties`.
    pub build_idproperties: &'a mut dyn FnMut(usize),
    /// `build_object` (base_index, object, linked, vis).
    pub build_object: &'a mut dyn FnMut(i32, usize, i32, bool),
}

/// `build_pose_constraints` (verbatim — id-loop +
/// BONE_CONSTRAINTS op).
pub fn build_pose_constraints(
    object_id: usize,
    pchan_index: usize,
    pchan_name: &str,
    pchan_constraints_list: usize,
    h: &mut RigNodeBuildHooks,
) {
    (h.constraints_id_loop)(pchan_constraints_list);
    (h.add_operation_node)(object_id, 13 /*NodeType::BONE*/, pchan_name, 4 /*BONE_CONSTRAINTS*/);
}

/// `build_ik_pose` (verbatim — POSE_IK_SOLVER at chain
/// root).
pub fn build_ik_pose(
    object_id: usize,
    pchan: usize,
    con: usize,
    h: &mut RigNodeBuildHooks,
) {
    let Some(rootchan) = (h.ik_solver_find_root)(pchan, con) else {
        return;
    };
    let root_name = (h.pchan_index_of)(rootchan);
    debug_assert!(root_name != -1);
    if (h.has_operation_node)(object_id, 12 /*EVAL_POSE*/, &root_name.to_string(), 3 /*POSE_IK_SOLVER*/) {
        return;
    }
    (h.add_operation_node)(object_id, 12, &root_name.to_string(), 3);
}

/// `build_splineik_pose` (verbatim).
pub fn build_splineik_pose(
    object_id: usize,
    pchan: usize,
    con: usize,
    h: &mut RigNodeBuildHooks,
) {
    let Some(rootchan) = (h.splineik_solver_find_root)(pchan, con) else {
        return;
    };
    let root_name = (h.pchan_index_of)(rootchan);
    debug_assert!(root_name != -1);
    if (h.has_operation_node)(object_id, 12, &root_name.to_string(), 4 /*POSE_SPLINE_IK_SOLVER*/) {
        return;
    }
    (h.add_operation_node)(object_id, 12, &root_name.to_string(), 4);
}

/// `build_rig` (verbatim — pose graph + per-bone ops).
pub fn build_rig(
    object_id: usize,
    object: usize,
    armature: usize,
    pose_flag: Option<i32>,
    pose: Option<usize>,
    pchans: &[PchanRow],
    h: &mut RigNodeBuildHooks,
) {
    (h.build_armature)(armature);
    if pose.is_none() || pose_flag.map_or(false, |f| f & POSE_RECALC != 0) {
        (h.pose_rebuild)(object, armature);
    } else {
        (h.pose_ensure_bone_indices)(object);
    }
    if let (Some(p), Some(f)) = (pose, pose_flag) {
        (h.pose_channels_hash_ensure)(p);
        if f & POSE_CONSTRAINTS_NEED_UPDATE_FLAGS != 0 {
            (h.pose_update_constraint_flags)(object);
        }
    }
    let op = (h.add_operation_node)(object_id, 12, "", 0 /*POSE_INIT*/);
    (h.set_as_entry)(op);
    (h.add_operation_node)(object_id, 12, "", 1 /*POSE_INIT_IK*/);
    (h.add_operation_node)(object_id, 12, "", 5 /*POSE_CLEANUP*/);
    let op = (h.add_operation_node)(object_id, 12, "", 6 /*POSE_DONE*/);
    (h.set_as_exit)(op);
    for (pchan_index, pchan) in pchans.iter().enumerate() {
        let op = (h.add_operation_node)(object_id, 13, &pchan.name, 0 /*BONE_LOCAL*/);
        (h.set_as_entry)(op);
        (h.add_operation_node)(object_id, 13, &pchan.name, 5 /*BONE_VISIBILITY*/);
        (h.add_operation_node)(object_id, 13, &pchan.name, 1 /*BONE_POSE_PARENT*/);
        (h.add_operation_node)(object_id, 13, &pchan.name, 2 /*BONE_READY*/);
        let op = (h.add_operation_node)(object_id, 13, &pchan.name, 3 /*BONE_DONE*/);
        if pchan.has_bbone {
            (h.add_operation_node)(object_id, 13, &pchan.name, 6 /*BONE_SEGMENTS*/);
        }
        (h.set_as_exit)(op);
        let mut add_idprops_operation = false;
        if pchan.prop != 0 {
            (h.build_idproperties)(pchan.prop);
            add_idprops_operation = true;
        }
        if pchan.system_properties != 0 {
            (h.build_idproperties)(pchan.system_properties);
            add_idprops_operation = true;
        }
        if add_idprops_operation {
            (h.add_operation_node)(
                object_id, 2 /*NodeType::PARAMETERS*/, &pchan.name, 0 /*PARAMETERS_EVAL*/,
            );
        }
        if pchan.has_constraints {
            build_pose_constraints(object_id, pchan_index, &pchan.name, 0, h);
        }
        for (con_type, con) in &pchan.constraints {
            match *con_type {
                CONSTRAINT_TYPE_KINEMATIC => {
                    build_ik_pose(object_id, pchan_index, *con, h);
                }
                CONSTRAINT_TYPE_SPLINEIK => {
                    build_splineik_pose(object_id, pchan_index, *con, h);
                }
                _ => {}
            }
        }
        if pchan.custom != 0 {
            (h.build_object)(-1, pchan.custom, DEG_ID_LINKED_INDIRECTLY, false);
        }
    }
}
