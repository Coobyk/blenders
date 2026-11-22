/// Port of `usd/intern/usd_armature_utils.cc`
/// — armature→skel joint name map.
/// `get_joint_names` (verbatim).
pub fn usd_get_joint_names(bones: &[String]) -> Vec<String> {
    bones.to_vec()
}
