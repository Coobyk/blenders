//! Port of `intern/action_bones.cc` — `BKE_action_find_fcurves_with_bones`.
//!
//! `fcurves_for_action_slot` stays caller-supplied (animrig slot API);
//! the `pose.bones["name"]` quoted-substr extraction is ported verbatim.

use blenders_bli::string::str_quoted_substr;

/// `FoundFCurveCallback` — (fcurve_handle, bone_name).
pub fn bke_action_find_fcurves_with_bones(
    fcurves: &[(usize, &str)], /* (FCurve*, rna_path) */
    mut callback: impl FnMut(usize, &str),
) {
    for &(fcu, rna_path) in fcurves {
        let Some(bone_name) = str_quoted_substr(rna_path, "pose.bones[") else {
            continue;
        };
        callback(fcu, &bone_name);
    }
}
