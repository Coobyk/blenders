//! Port of `intern/pose_backup.cc` — `BKE_pose_backup_*` (snapshot of pose
//! channel state before pasting/applying poses).
//!
//! Foreign systems stay injected: animated-bone iteration comes from
//! `BKE_action_find_fcurves_with_bones` (action module pending), and pose
//! channel lookups from `BKE_pose_channel_find_name` (armature pending).

use std::collections::HashSet;

use crate::idprop::{idp_copy_property, idp_sync_group_values, IDProperty};

/// `PoseChannelBackup` — one channel's saved state.
pub struct PoseChannelBackup {
    /// `bPoseChannel *pchan` — caller handle.
    pub pchan: usize,
    /// `bPoseChannel olddata` — shallow copy at backup time (opaque bytes;
    /// `bPoseChannel` is in blenders_dna).
    pub olddata: blenders_dna::action_types::BPoseChannel,
    /// `IDProperty *oldprops` — arena index.
    pub oldprops: Option<usize>,
    /// `IDProperty *old_system_properties` — arena index.
    pub old_system_properties: Option<usize>,
    /// `const Object *owner`.
    pub owner: usize,
}

/// `PoseBackup`.
pub struct PoseBackup {
    pub is_bone_selection_relevant: bool,
    /// `ListBaseT<PoseChannelBackup> backups`.
    pub backups: Vec<PoseChannelBackup>,
}

/// `pose_backup_create` — one object.
///
/// * `foreach_animated_bone`: calls its argument once per bone name animated
///   by the action's best pose slot (`animrig::get_best_pose_slot_for_id` +
///   `BKE_action_find_fcurves_with_bones`).
/// * `find_pchan`: `BKE_pose_channel_find_name` — returns the channel handle
///   and a shallow copy of it.
/// * `idp_arena`: shared IDProperty arena for the prop copies.
pub fn pose_backup_create(
    owner: usize,
    selected_bone_names: &HashSet<String>,
    is_bone_selection_relevant: bool,
    foreach_animated_bone: &mut dyn FnMut(&mut dyn FnMut(&str)),
    find_pchan: &mut dyn FnMut(&str) -> Option<(usize, blenders_dna::action_types::BPoseChannel)>,
    get_pchan_props: &mut dyn FnMut(usize) -> (Option<usize>, Option<usize>),
    idp_arena: &mut Vec<IDProperty>,
    pose_backup: &mut PoseBackup,
) {
    let mut backed_up_bone_names: HashSet<String> = HashSet::new();
    foreach_animated_bone(&mut |bone_name: &str| {
        if backed_up_bone_names.contains(bone_name) {
            return;
        }
        let Some((pchan, olddata)) = find_pchan(bone_name) else {
            return;
        };
        if is_bone_selection_relevant && !selected_bone_names.contains(bone_name) {
            return;
        }
        let (prop, sys_props) = get_pchan_props(pchan);
        let chan_bak = PoseChannelBackup {
            pchan,
            olddata,
            owner,
            oldprops: prop.map(|i| idp_copy_property(&idp_arena[i].clone(), idp_arena, 0)),
            old_system_properties: sys_props.map(|i| idp_copy_property(&idp_arena[i].clone(), idp_arena, 0)),
        };
        pose_backup.backups.push(chan_bak);
        backed_up_bone_names.insert(bone_name.to_string());
    });
}

/// `armature_find_selected_pose_bones` — empty when every bone is selected.
/// `iter_pose_bones` yields `(handle, flag)` per pose channel per object;
/// `POSE_SELECTED` is the flag bit to test.
pub fn armature_find_selected_pose_bones(
    iter_pose_bones: &mut dyn FnMut(&mut dyn FnMut(usize, i32)),
    pose_selected_flag: i32,
) -> HashSet<usize> {
    let mut selected_bones = HashSet::new();
    let mut all_bones_selected = true;
    iter_pose_bones(&mut |pchan: usize, flag: i32| {
        if flag & pose_selected_flag != 0 {
            selected_bones.insert(pchan);
        } else {
            all_bones_selected = false;
        }
    });
    if all_bones_selected {
        return HashSet::new();
    }
    selected_bones
}

/// `BKE_pose_backup_is_selection_relevant`.
pub fn bke_pose_backup_is_selection_relevant(pose_backup: &PoseBackup) -> bool {
    pose_backup.is_bone_selection_relevant
}

/// `BKE_pose_backup_restore` — syncs the backed-up ID property values back
/// into the live channels. `resolve_props` maps a pchan handle to its
/// `(prop, system_properties)` group indices in `idp_arena`.
pub fn bke_pose_backup_restore(
    pose_backup: &PoseBackup,
    idp_arena: &mut Vec<IDProperty>,
    resolve_props: &mut dyn FnMut(usize) -> (Option<usize>, Option<usize>),
) {
    for chan_bak in &pose_backup.backups {
        let (prop, sys_props) = resolve_props(chan_bak.pchan);
        if let Some(oldprops) = chan_bak.oldprops {
            if let Some(dest) = prop {
                idp_sync_group_values(dest, oldprops, idp_arena);
            }
        }
        if let Some(old_sys) = chan_bak.old_system_properties {
            if let Some(dest) = sys_props {
                idp_sync_group_values(dest, old_sys, idp_arena);
            }
        }
    }
}
