//! Port of `intern/pose.cc` — apply-pose fcurve
//! filtering + bone selection.

/// `FCURVE_*`/`AGRP_MUTED`/`POSE_SELECTED` verbatim.
pub const FCURVE_MUTED: i32 = 1 << 1;
pub const FCURVE_DISABLED: i32 = 1 << 2;
pub const FCURVE_PROTECTED: i32 = 1 << 0;
pub const AGRP_MUTED: i32 = 1 << 1;
pub const POSE_SELECTED: i32 = 1 << 0;
/// `ROT_MODE_*` verbatim.
pub const ROT_MODE_QUAT: i16 = 0;
pub const ROT_MODE_EUL: i16 = 4;

/// `pose_apply_restore_fcurves` (verbatim — clear
/// DISABLED).
pub fn pose_apply_restore_fcurves(fcu_flags: &mut [i32]) {
    for flag in fcu_flags.iter_mut() {
        *flag &= !FCURVE_DISABLED;
    }
}

/// `pose_apply_disable_fcurves_for_unselected_bones`
/// (verbatim — disable unselected, track modified).
pub fn pose_apply_disable_fcurves_for_unselected(
    fcurves: &mut [(String, i32)],
    selected_bone_names: &std::collections::HashSet<String>,
) -> Vec<usize> {
    let mut modified = Vec::new();
    for (i, (bone_name, flag)) in fcurves.iter_mut().enumerate() {
        if !selected_bone_names.contains(bone_name) {
            if (*flag & FCURVE_DISABLED) == 0 {
                modified.push(i);
            }
            *flag |= FCURVE_DISABLED;
        }
    }
    modified
}

/// `is_fcurve_evaluatable` (verbatim — not muted/disabled,
/// group not muted, not empty).
pub fn is_fcurve_evaluatable(fcu_flag: i32, grp_muted: bool, fcurve_is_empty: bool) -> bool {
    if (fcu_flag & (FCURVE_MUTED | FCURVE_DISABLED)) != 0 {
        return false;
    }
    if grp_muted {
        return false;
    }
    if fcurve_is_empty {
        return false;
    }
    true
}

/// `rotation_mode_is_euler` (verbatim — ≥ROT_MODE_EUL).
pub fn rotation_mode_is_euler(rotation_mode: i16) -> bool {
    rotation_mode >= ROT_MODE_EUL
}

/// `animsys_quaternion_evaluate_fcurves` (verbatim —
/// unit quat init + per-component eval + normalize
/// when <4 keyed).
pub fn animsys_quaternion_evaluate_fcurves(
    quat_fcurves: &[(i32, f32)],
    r_quaternion: &mut [f32; 4],
) {
    *r_quaternion = [1.0, 0.0, 0.0, 0.0];
    for &(array_index, value) in quat_fcurves {
        r_quaternion[array_index as usize] = value;
    }
    if quat_fcurves.len() < 4 {
        let len = (r_quaternion[0] * r_quaternion[0]
            + r_quaternion[1] * r_quaternion[1]
            + r_quaternion[2] * r_quaternion[2]
            + r_quaternion[3] * r_quaternion[3])
            .sqrt();
        if len != 0.0 {
            for c in r_quaternion.iter_mut() {
                *c /= len;
            }
        }
    }
}

/// `any_bone_selected` (verbatim).
pub fn any_bone_selected(objects_pose_channel_flags: &[Vec<i32>]) -> bool {
    for flags in objects_pose_channel_flags {
        for &flag in flags {
            if (flag & POSE_SELECTED) != 0 {
                return true;
            }
        }
    }
    false
}
