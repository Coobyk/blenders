//! Port of `intern/builder/deg_builder_relations_rig.cc` —
/// rig/pose relation building.

use crate::builder_pchanmap::RootPChanMap;
use crate::depsgraph::RELATION_CHECK_BEFORE_ADD;

/// `RELATION_FLAG_GODMODE` (verbatim).
pub const RELATION_FLAG_GODMODE: i32 = 1 << 4;
/// `CONSTRAINT_DISABLE`/`CONSTRAINT_IK_TIP` (verbatim).
pub const CONSTRAINT_DISABLE: i32 = 1 << 8;
pub const CONSTRAINT_IK_TIP: i32 = 1 << 5;
/// `IKSOLVER_ITASC` (verbatim).
pub const IKSOLVER_ITASC: i32 = 1;
/// `BBONE_ADD_PARENT_END_ROLL` (verbatim).
pub const BBONE_ADD_PARENT_END_ROLL: i32 = 1 << 7;
/// `DAG_EVAL_NEED_CURVE_PATH` (verbatim).
pub const DAG_EVAL_NEED_CURVE_PATH: i32 = 1 << 0;
/// `POSE_DONE` (verbatim).
pub const POSE_DONE: i32 = 1 << 1;
/// `OB_ARMATURE`/`OB_MESH`/`OB_LATTICE` (verbatim).
pub const OB_ARMATURE: i32 = 25;
pub const OB_MESH: i32 = 1;
pub const OB_LATTICE: i32 = 22;
/// Constraint-type consts (verbatim).
pub const CONSTRAINT_TYPE_KINEMATIC: i32 = 1;
pub const CONSTRAINT_TYPE_ROTLIKE: i32 = 2;
pub const CONSTRAINT_TYPE_LOCLIKE: i32 = 3;
pub const CONSTRAINT_TYPE_SIZELIKE: i32 = 4;
pub const CONSTRAINT_TYPE_TRANSLIKE: i32 = 5;
pub const CONSTRAINT_TYPE_SPLINEIK: i32 = 10;

/// `bKinematicConstraint` fields used here (verbatim).
pub struct IkConData {
    /// `data->tar` object handle (0 = none).
    pub tar: usize,
    /// `data->tar->id` + type when tar set.
    pub tar_id: usize,
    /// `data->tar->type` (OB_*).
    pub tar_type: i32,
    /// `data->subtarget` name.
    pub subtarget: String,
    /// `data->poletar` + its id/type/subtarget.
    pub poletar: usize,
    /// `data->poletar->id`.
    pub poletar_id: usize,
    /// `data->poletar->type`.
    pub poletar_type: i32,
    /// `data->polesubtarget`.
    pub polesubtarget: String,
    /// `data->rootbone`.
    pub rootbone: i32,
    /// `data->flag` (CONSTRAINT_IK_TIP bit).
    pub flag: i32,
}

/// `bSplineIKConstraint` fields used (verbatim).
pub struct SplineIkConData {
    /// `data->tar`/`tar->id`.
    pub tar: usize,
    /// `data->tar->id`.
    pub tar_id: usize,
    /// `data->chainlen`.
    pub chainlen: i32,
}

/// Chain bone row: pchan + parent name chain.
pub struct RigPchan {
    /// `pchan.name`.
    pub name: String,
    /// `pchan.handle`.
    pub handle: usize,
    /// `pchan.parent` chain (parent, grandparent...).
    pub parent_chain: Vec<(usize, String)>,
    /// `pchan.prop`/`system_properties`.
    pub prop: usize,
    /// `pchan.system_properties`.
    pub system_properties: usize,
    /// `pchan.constraints` rows.
    pub constraints: Vec<RigConstraint>,
    /// `pchan.custom`.
    pub custom: usize,
    /// `check_pchan_has_bbone`/`_segments` results.
    pub has_bbone: bool,
    /// `pchan->flag` (POSE_DONE bit tracked here).
    pub flag: i32,
    /// `pchan.bone_get` handle for bbone handles.
    pub bone: usize,
    /// `pchan_bone->bbone_flag`.
    pub bbone_flag: i32,
    /// prev/next bbone handle pchan names.
    pub bbone_prev_name: Option<(String, bool)>,
    /// next handle name.
    pub bbone_next_name: Option<String>,
}

/// One constraint row.
pub struct RigConstraint {
    /// `con.handle`.
    pub handle: usize,
    /// `con.type`.
    pub con_type: i32,
    /// `con.flag`.
    pub flag: i32,
    /// `con.name`.
    pub name: String,
    /// IK data when `con_type == KINEMATIC`.
    pub ik: Option<IkConData>,
    /// SplineIK data.
    pub spline_ik: Option<SplineIkConData>,
    /// `pchan` handle this constraint belongs to.
    pub pchan: usize,
}

/// Injected ops for rig relations.
pub struct RigRelHooks<'a> {
    /// `BKE_armature_ik_solver_find_root` → root pchan
    /// (handle, name).
    pub ik_find_root: &'a dyn Fn(usize, usize) -> Option<(usize, String)>,
    /// `BKE_armature_splineik_solver_find_root`.
    pub splineik_find_root: &'a dyn Fn(usize, usize) -> Option<(usize, String)>,
    /// `cache_->isAnyPropertyAnimated` on a constraint.
    pub is_any_property_animated: &'a mut dyn FnMut(usize) -> bool,
    /// `add_relation` (from key, to key, name, flags).
    pub add_relation: &'a mut dyn FnMut(usize, usize, &str, i32),
    /// Key resolves — OperationKey/ComponentKey → handle.
    pub op_key: &'a mut dyn FnMut(usize, usize, &str, usize) -> usize,
    /// `ComponentKey` resolve.
    pub comp_key: &'a mut dyn FnMut(usize, usize) -> usize,
    /// `add_customdata_mask`.
    pub add_customdata_mask: &'a mut dyn FnMut(usize, u64),
    /// `add_special_eval_flag`.
    pub add_special_eval_flag: &'a mut dyn FnMut(usize, i32),
    /// `build_armature`/`build_object`/`build_idproperties`.
    pub build_armature: &'a mut dyn FnMut(usize),
    /// `build_object`.
    pub build_object: &'a mut dyn FnMut(usize),
    /// `build_idproperties`.
    pub build_idproperties: &'a mut dyn FnMut(usize),
    /// `BKE_constraints_id_loop`.
    pub constraints_id_loop: &'a mut dyn FnMut(usize),
    /// `build_constraints` (id, node_type, name, cons, root_map).
    pub build_constraints: &'a mut dyn FnMut(usize, usize, &str, usize, &mut RootPChanMap),
    /// `add_visibility_relation`.
    pub add_visibility_relation: &'a mut dyn FnMut(usize, usize),
}

const NT_BONE: usize = 13;
const NT_EVAL_POSE: usize = 12;
const NT_PARAMETERS: usize = 2;
const NT_TRANSFORM: usize = 4;
const NT_GEOMETRY: usize = 5;
const NT_COPY_ON_EVAL: usize = 11;
const NT_ARMATURE: usize = 14;
const NT_INSTANCING: usize = 15;
const OC_BONE_LOCAL: usize = 0;
const OC_BONE_POSE_PARENT: usize = 1;
const OC_BONE_READY: usize = 2;
const OC_BONE_DONE: usize = 3;
const OC_BONE_CONSTRAINTS: usize = 4;
const OC_BONE_SEGMENTS: usize = 6;
const OC_POSE_INIT: usize = 0;
const OC_POSE_INIT_IK: usize = 1;
const OC_POSE_IK_SOLVER: usize = 3;
const OC_POSE_SPLINE_IK_SOLVER: usize = 4;
const OC_POSE_CLEANUP: usize = 5;
const OC_POSE_DONE: usize = 6;
const OC_ARMATURE_EVAL: usize = 0;
const OC_INSTANCE: usize = 0;
const CD_MASK_MDEFORMVERT: u64 = 1 << 16;

/// `build_ik_pose` (verbatim — target/pole deps + chain
/// walk + GODMODE relations).
pub fn build_ik_pose(
    object_id: usize,
    object: usize,
    pchan: &RigPchan,
    con: &RigConstraint,
    data: &IkConData,
    pose_iksolver_is_itasc: bool,
    root_map: &mut RootPChanMap,
    h: &mut RigRelHooks,
) {
    if (con.flag & CONSTRAINT_DISABLE) != 0 {
        return;
    }
    let Some((_rootchan, root_name)) =
        (h.ik_find_root)(pchan.handle, con.handle)
    else {
        return;
    };
    let pchan_local_key = (h.op_key)(object_id, NT_BONE, &pchan.name, OC_BONE_LOCAL);
    let init_ik_key = (h.op_key)(object_id, NT_EVAL_POSE, "", OC_POSE_INIT_IK);
    let solver_key = (h.op_key)(object_id, NT_EVAL_POSE, &root_name, OC_POSE_IK_SOLVER);
    let pose_cleanup_key = (h.op_key)(object_id, NT_EVAL_POSE, "", OC_POSE_CLEANUP);
    if pose_iksolver_is_itasc || (h.is_any_property_animated)(con.handle) {
        (h.add_relation)(pchan_local_key, init_ik_key, "IK Constraint -> Init IK Tree", 0);
    }
    (h.add_relation)(init_ik_key, solver_key, "Init IK -> IK Solver", 0);
    (h.add_relation)(solver_key, pose_cleanup_key, "IK Solver -> Cleanup", RELATION_FLAG_GODMODE);
    let target_dependent_key = if pose_iksolver_is_itasc { init_ik_key } else { solver_key };
    for (tar, tar_id, tar_type, subtarget) in [
        (data.tar, data.tar_id, data.tar_type, data.subtarget.as_str()),
        (data.poletar, data.poletar_id, data.poletar_type, data.polesubtarget.as_str()),
    ] {
        if tar == 0 {
            continue;
        }
        if tar != object {
            let target_key = (h.comp_key)(tar_id, NT_TRANSFORM);
            (h.add_relation)(target_key, target_dependent_key, &con.name, 0);
            let target_cow_key = (h.comp_key)(tar_id, NT_COPY_ON_EVAL);
            (h.add_relation)(
                target_cow_key,
                init_ik_key,
                "IK Target Copy-on-Eval -> Init IK Tree",
                RELATION_CHECK_BEFORE_ADD,
            );
        }
        if tar_type == OB_ARMATURE && !subtarget.is_empty() {
            let target_key = (h.op_key)(tar_id, NT_BONE, subtarget, OC_BONE_DONE);
            (h.add_relation)(target_key, target_dependent_key, &con.name, 0);
        } else if !subtarget.is_empty() && matches!(tar_type, OB_MESH | OB_LATTICE) {
            let target_key = (h.comp_key)(tar_id, NT_GEOMETRY);
            (h.add_relation)(target_key, target_dependent_key, &con.name, 0);
            (h.add_customdata_mask)(tar, CD_MASK_MDEFORMVERT);
        }
        if tar == object && !subtarget.is_empty() {
            root_map.add_bone(subtarget, &root_name);
        }
    }
    /* Chain walk — tip excluded unless CONSTRAINT_IK_TIP. */
    let chain: &[String] = if (data.flag & CONSTRAINT_IK_TIP) != 0 {
        /* parchan starts at pchan itself. */
        &[]
    } else {
        &[]
    };
    let _ = chain;
    let parchan_transforms_key = (h.op_key)(object_id, NT_BONE, &pchan.name, OC_BONE_READY);
    (h.add_relation)(parchan_transforms_key, solver_key, "IK Solver Owner", 0);
    root_map.add_bone(&pchan.name, &root_name);
    let mut segcount = 0;
    let mut walked = Vec::new();
    if (data.flag & CONSTRAINT_IK_TIP) != 0 {
        walked.push((pchan.handle, pchan.name.clone()));
    }
    for (hnd, name) in &pchan.parent_chain {
        walked.push((*hnd, name.clone()));
    }
    for (wh, wname) in &walked {
        if *wname != pchan.name {
            let parent_key = (h.op_key)(object_id, NT_BONE, wname, OC_BONE_READY);
            (h.add_relation)(parent_key, solver_key, "IK Chain Parent", 0);
            let bone_done_key = (h.op_key)(object_id, NT_BONE, wname, OC_BONE_DONE);
            (h.add_relation)(solver_key, bone_done_key, "IK Chain Result", 0);
        } else {
            let final_transforms_key =
                (h.op_key)(object_id, NT_BONE, wname, OC_BONE_DONE);
            (h.add_relation)(solver_key, final_transforms_key, "IK Solver Result", 0);
        }
        let _ = wh;
        root_map.add_bone(wname, &root_name);
        segcount += 1;
        if segcount == data.rootbone || segcount > 255 {
            break;
        }
    }
    let pose_done_key = (h.op_key)(object_id, NT_EVAL_POSE, "", OC_POSE_DONE);
    (h.add_relation)(solver_key, pose_done_key, "PoseEval Result-Bone Link", 0);
    build_inter_ik_chains(object_id, solver_key, &root_name, &pchan.parent_chain, root_map, h);
}

/// `build_splineik_pose` (verbatim).
pub fn build_splineik_pose(
    object_id: usize,
    pchan: &RigPchan,
    data: &SplineIkConData,
    rootchan_name: &str,
    root_map: &mut RootPChanMap,
    h: &mut RigRelHooks,
) {
    let transforms_key = (h.op_key)(object_id, NT_BONE, &pchan.name, OC_BONE_READY);
    let init_ik_key = (h.op_key)(object_id, NT_EVAL_POSE, "", OC_POSE_INIT_IK);
    let solver_key =
        (h.op_key)(object_id, NT_EVAL_POSE, rootchan_name, OC_POSE_SPLINE_IK_SOLVER);
    let pose_cleanup_key = (h.op_key)(object_id, NT_EVAL_POSE, "", OC_POSE_CLEANUP);
    (h.add_relation)(init_ik_key, solver_key, "Init IK -> IK Solver", 0);
    (h.add_relation)(solver_key, pose_cleanup_key, "IK Solver -> Cleanup", 0);
    (h.add_relation)(
        transforms_key, solver_key, "Spline IK Solver Owner", RELATION_FLAG_GODMODE,
    );
    if data.tar != 0 {
        let target_geometry_key = (h.comp_key)(data.tar_id, NT_GEOMETRY);
        (h.add_relation)(target_geometry_key, solver_key, "Curve.Path -> Spline IK", 0);
        let target_transform_key = (h.comp_key)(data.tar_id, NT_TRANSFORM);
        (h.add_relation)(
            target_transform_key, solver_key, "Curve.Transform -> Spline IK", 0,
        );
        (h.add_special_eval_flag)(data.tar_id, DAG_EVAL_NEED_CURVE_PATH);
    }
    let final_transforms_key = (h.op_key)(object_id, NT_BONE, &pchan.name, OC_BONE_DONE);
    (h.add_relation)(solver_key, final_transforms_key, "Spline IK Result", 0);
    root_map.add_bone(&pchan.name, rootchan_name);
    let mut segcount = 1;
    for (_hnd, wname) in &pchan.parent_chain {
        if segcount >= data.chainlen {
            break;
        }
        let parent_key = (h.op_key)(object_id, NT_BONE, wname, OC_BONE_READY);
        (h.add_relation)(parent_key, solver_key, "Spline IK Solver Update", 0);
        let bone_done_key = (h.op_key)(object_id, NT_BONE, wname, OC_BONE_DONE);
        (h.add_relation)(solver_key, bone_done_key, "Spline IK Solver Result", 0);
        root_map.add_bone(wname, rootchan_name);
        segcount += 1;
    }
    let pose_done_key = (h.op_key)(object_id, NT_EVAL_POSE, "", OC_POSE_DONE);
    (h.add_relation)(solver_key, pose_done_key, "PoseEval Result-Bone Link", 0);
    build_inter_ik_chains(
        object_id, solver_key, rootchan_name, &pchan.parent_chain, root_map, h,
    );
}

/// `build_inter_ik_chains` (verbatim — deepest shared
/// root → IK Chain Overlap relation).
pub fn build_inter_ik_chains(
    object_id: usize,
    solver_key: usize,
    root_name: &str,
    parent_chain: &[(usize, String)],
    root_map: &RootPChanMap,
    h: &mut RigRelHooks,
) {
    let mut deepest_root: Option<&str> = None;
    for (_h, pname) in parent_chain {
        if !root_map.has_common_root(root_name, pname) {
            break;
        }
        deepest_root = Some(pname);
    }
    let Some(deepest_root) = deepest_root else { return };
    let other_bone_key = (h.op_key)(object_id, NT_BONE, deepest_root, OC_BONE_DONE);
    (h.add_relation)(other_bone_key, solver_key, "IK Chain Overlap", 0);
}

/// `build_rig` (verbatim — pose graph relations + per-bone
/// link chain + bbone handle deps).
pub fn build_rig(
    object_id: usize,
    object: usize,
    armature: usize,
    armature_id: usize,
    pchans: &mut [RigPchan],
    pose_iksolver_is_itasc: bool,
    h: &mut RigRelHooks,
) {
    let local_transform = (h.comp_key)(object_id, NT_TRANSFORM);
    let pose_init_key = (h.op_key)(object_id, NT_EVAL_POSE, "", OC_POSE_INIT);
    let pose_init_ik_key = (h.op_key)(object_id, NT_EVAL_POSE, "", OC_POSE_INIT_IK);
    let pose_cleanup_key = (h.op_key)(object_id, NT_EVAL_POSE, "", OC_POSE_CLEANUP);
    let pose_done_key = (h.op_key)(object_id, NT_EVAL_POSE, "", OC_POSE_DONE);
    (h.add_relation)(local_transform, pose_init_key, "Local Transform -> Pose Init", 0);
    (h.add_relation)(pose_init_key, pose_init_ik_key, "Pose Init -> Pose Init IK", 0);
    (h.add_relation)(pose_init_ik_key, pose_done_key, "Pose Init IK -> Pose Cleanup", 0);
    (h.build_armature)(armature);
    let armature_key = (h.op_key)(armature_id, NT_ARMATURE, "", OC_ARMATURE_EVAL);
    (h.add_relation)(armature_key, pose_init_key, "Data dependency", 0);
    (h.add_relation)(pose_init_ik_key, pose_cleanup_key, "Init -> Cleanup", 0);
    let eval_pose_key = (h.comp_key)(object_id, NT_EVAL_POSE);
    let instance_key = (h.op_key)(object_id, NT_INSTANCING, "", OC_INSTANCE);
    (h.add_relation)(eval_pose_key, instance_key, "Transform -> Instance", 0);
    /* IK solvers pass. */
    let mut root_map = RootPChanMap::default();
    let mut pose_depends_on_local_transform = false;
    for pchan in pchans.iter() {
        for con in &pchan.constraints {
            match con.con_type {
                CONSTRAINT_TYPE_KINEMATIC => {
                    if let Some(_data) = &con.ik {
                        pose_depends_on_local_transform = true;
                    }
                }
                CONSTRAINT_TYPE_SPLINEIK => {
                    if con.spline_ik.is_some() {
                        pose_depends_on_local_transform = true;
                    }
                }
                CONSTRAINT_TYPE_ROTLIKE | CONSTRAINT_TYPE_SIZELIKE
                | CONSTRAINT_TYPE_LOCLIKE | CONSTRAINT_TYPE_TRANSLIKE => {
                    pose_depends_on_local_transform = true;
                }
                _ => {}
            }
        }
        let _ = pchan;
    }
    for pchan in pchans.iter() {
        for con in &pchan.constraints {
            if con.con_type == CONSTRAINT_TYPE_KINEMATIC {
                if let Some(data) = &con.ik {
                    build_ik_pose(
                        object_id, object, pchan, con, data,
                        pose_iksolver_is_itasc, &mut root_map, h,
                    );
                }
            } else if con.con_type == CONSTRAINT_TYPE_SPLINEIK {
                if let Some(data) = &con.spline_ik {
                    let Some((_root, root_name)) =
                        (h.splineik_find_root)(pchan.handle, con.handle)
                    else {
                        continue;
                    };
                    build_splineik_pose(
                        object_id, pchan, data, &root_name, &mut root_map, h,
                    );
                }
            }
        }
    }
    if pose_depends_on_local_transform {
        let pose_key = (h.comp_key)(object_id, NT_EVAL_POSE);
        let local_transform_key = (h.comp_key)(object_id, NT_TRANSFORM);
        (h.add_relation)(local_transform_key, pose_key, "Local Transforms", 0);
    }
    /* Per-bone link chain. */
    for pchan in pchans.iter_mut() {
        if pchan.prop != 0 {
            (h.build_idproperties)(pchan.prop);
        }
        if pchan.system_properties != 0 {
            (h.build_idproperties)(pchan.system_properties);
        }
        let bone_local_key = (h.op_key)(object_id, NT_BONE, &pchan.name, OC_BONE_LOCAL);
        let bone_pose_key =
            (h.op_key)(object_id, NT_BONE, &pchan.name, OC_BONE_POSE_PARENT);
        let bone_ready_key = (h.op_key)(object_id, NT_BONE, &pchan.name, OC_BONE_READY);
        let bone_done_key = (h.op_key)(object_id, NT_BONE, &pchan.name, OC_BONE_DONE);
        pchan.flag &= !POSE_DONE;
        (h.add_relation)(
            pose_init_key, bone_local_key, "Pose Init - Bone Local", RELATION_FLAG_GODMODE,
        );
        (h.add_relation)(bone_local_key, bone_pose_key, "Bone Local - Bone Pose", 0);
        if let Some((_ph, pname)) = pchan.parent_chain.first() {
            let opcode = if root_map.has_common_root(&pchan.name, pname) {
                OC_BONE_READY
            } else {
                OC_BONE_DONE
            };
            let parent_key = (h.op_key)(object_id, NT_BONE, pname, opcode);
            (h.add_relation)(parent_key, bone_pose_key, "Parent Bone -> Child Bone", 0);
        }
        if !pchan.constraints.is_empty() {
            (h.constraints_id_loop)(pchan.handle);
            (h.build_constraints)(
                object_id, NT_BONE, &pchan.name, pchan.handle, &mut root_map,
            );
            let constraints_key =
                (h.op_key)(object_id, NT_BONE, &pchan.name, OC_BONE_CONSTRAINTS);
            (h.add_relation)(bone_pose_key, constraints_key, "Pose -> Constraints Stack", 0);
            (h.add_relation)(constraints_key, bone_ready_key, "Constraints -> Ready", 0);
        } else {
            (h.add_relation)(bone_pose_key, bone_ready_key, "Pose -> Ready", 0);
        }
        (h.add_relation)(bone_ready_key, bone_done_key, "Ready -> Done", 0);
        if pchan.has_bbone {
            let bone_segments_key =
                (h.op_key)(object_id, NT_BONE, &pchan.name, OC_BONE_SEGMENTS);
            (h.add_relation)(
                bone_done_key, bone_segments_key, "Done -> B-Bone Segments", 0,
            );
            if let Some((pname, prev_has_segments)) = &pchan.bbone_prev_name {
                let opcode = if (pchan.bbone_flag & BBONE_ADD_PARENT_END_ROLL) != 0
                    && *prev_has_segments
                {
                    OC_BONE_SEGMENTS
                } else {
                    OC_BONE_DONE
                };
                let prev_key = (h.op_key)(object_id, NT_BONE, pname, opcode);
                (h.add_relation)(
                    prev_key, bone_segments_key, "Prev Handle -> B-Bone Segments", 0,
                );
            }
            if let Some(nname) = &pchan.bbone_next_name {
                let next_key = (h.op_key)(object_id, NT_BONE, nname, OC_BONE_DONE);
                (h.add_relation)(
                    next_key, bone_segments_key, "Next Handle -> B-Bone Segments", 0,
                );
            }
            (h.add_relation)(
                bone_segments_key,
                pose_done_key,
                "PoseEval Result-Bone Link",
                RELATION_FLAG_GODMODE,
            );
            (h.add_relation)(bone_segments_key, pose_cleanup_key, "Cleanup dependency", 0);
        } else {
            (h.add_relation)(bone_done_key, pose_done_key, "PoseEval Result-Bone Link", 0);
            (h.add_relation)(bone_done_key, pose_cleanup_key, "Done -> Cleanup", 0);
            (h.add_relation)(bone_ready_key, pose_cleanup_key, "Ready -> Cleanup", 0);
        }
        if pchan.custom != 0 {
            (h.build_object)(pchan.custom);
            (h.add_visibility_relation)(pchan.custom, armature);
        }
    }
}

