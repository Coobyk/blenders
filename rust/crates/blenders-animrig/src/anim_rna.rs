//! Port of `intern/anim_rna.cc` — RNA value/path
//! helpers for keyframing.

/// `pose_bone_path_prefix` verbatim.
pub const POSE_BONE_PATH_PREFIX: &str = "pose.bones[";
/// `MAXBONENAME` verbatim.
pub const MAXBONENAME: usize = 64;
/// `eRotationModes` extra verbatim.
pub const ROT_MODE_AXISANGLE: i16 = 1;
pub const ROT_MODE_EUL_ORDER_MIN: i16 = 4;
pub const ROT_MODE_EUL_ORDER_MAX: i16 = 12;

/// `get_pose_bone_rna_path` (verbatim — escaped name).
pub fn get_pose_bone_rna_path(bone_name: &str) -> String {
    let escaped: String = bone_name.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{}\"{}\"]", POSE_BONE_PATH_PREFIX, escaped)
}

/// `pose_bone_name_from_rna_path` (verbatim — needs ≥3
/// elems: pose/bones/"name").
pub fn pose_bone_name_from_rna_path(
    first_is_pose: bool,
    second_is_bones: bool,
    third_lookup_key: Option<&str>,
) -> Option<String> {
    if !first_is_pose || !second_is_bones {
        return None;
    }
    let name = third_lookup_key?;
    if name.len() >= MAXBONENAME {
        return None;
    }
    Some(name.to_string())
}

/// `get_rotation_mode_path` (verbatim — prop name per
/// mode).
pub fn get_rotation_mode_path(rotation_mode: i16) -> &'static str {
    match rotation_mode {
        x if x == crate::pose::ROT_MODE_QUAT => "rotation_quaternion",
        x if x == ROT_MODE_AXISANGLE => "rotation_axis_angle",
        _ => "rotation_euler",
    }
}

/// `get_rotation_mode_from_path` suffix check (verbatim —
/// startswith rotation_ + endswith quaternion/euler).
pub fn get_rotation_mode_from_path(last_property_name: &str) -> Option<i16> {
    if !last_property_name.starts_with("rotation_") {
        return None;
    }
    if last_property_name.ends_with("quaternion") {
        return Some(crate::pose::ROT_MODE_QUAT);
    }
    if last_property_name.ends_with("euler") {
        return Some(crate::pose::ROT_MODE_EUL);
    }
    None
}
