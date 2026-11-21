//! Port of `intern/anim_data.cc` — `AnimData` management plus the
//! RNA-path rename/remove fix algorithms. ID/FCurve/bAction
//! access is injected via handles and closures.

/// `AnimData` (relevant fields verbatim).
#[derive(Clone, Default)]
pub struct AnimData {
    /// `action` handle.
    pub action: usize,
    /// `tmpact` handle.
    pub tmpact: usize,
    /// `binding` handle.
    pub binding: usize,
    /// `act_influence` (1.0 default on ensure).
    pub act_influence: f32,
    /// `act_blendmode`.
    pub act_blendmode: i16,
    /// `actslot` handle.
    pub actslot: i64,
    /// `drivers` list (FCurve handles).
    pub drivers: Vec<usize>,
    /// `nla_tracks` list.
    pub nla_tracks: Vec<usize>,
    /// `overrides`.
    pub overrides: Vec<usize>,
    /// `flag`.
    pub flag: i32,
    /// `recalc`.
    pub recalc: i32,
    /// `slot_handle`.
    pub slot_handle: i64,
}

impl AnimData {
    /// `BKE_animdata_ensure_id` — new adt gets `act_influence = 1.0`.
    pub fn new_ensured() -> Self {
        Self {
            act_influence: 1.0,
            ..Default::default()
        }
    }
}

/// `ADT_RECALC_ANIM` and friends (verbatim values).
pub const ADT_RECALC_ANIM: i32 = 1 << 0;
/// `ADT_RECALC_DRIVERS`.
pub const ADT_RECALC_DRIVERS: i32 = 1 << 1;
/// `ADT_RECALC_ALL`.
pub const ADT_RECALC_ALL: i32 = ADT_RECALC_ANIM | ADT_RECALC_DRIVERS;
/// `ADT_NLA_EDIT_ON`.
pub const ADT_NLA_EDIT_ON: i32 = 1 << 0;
/// `ADT_NLA_EVAL_IGNORING_STRIP`.
pub const ADT_NLA_EVAL_IGNORING_STRIP: i32 = 1 << 1;
/// `ADT_FLAG_ACTIVE_SLOT_NO_AUTO`.
pub const ADT_FLAG_ACTIVE_SLOT_NO_AUTO: i32 = 1 << 2;
/// `ADT_UI_ACTIVE`.
pub const ADT_UI_ACTIVE: i32 = 1 << 3;
/// `ADT_RECURSE`.
pub const ADT_RECURSE: i32 = 1 << 4;

/// `BKE_animdata_action_editable` — action is editable when it is
/// local (not a library / not linked).
pub fn bke_animdata_action_editable(action_is_lib: bool, action_is_override: bool) -> bool {
    !(action_is_lib || action_is_override)
}

/// `BKE_animdata_id_is_animated` — ID has drivers or fcurves or nla.
/// `has_action_fcurves` injected.
pub fn bke_animdata_id_is_animated(
    adt: &AnimData,
    has_action_fcurves: bool,
) -> bool {
    has_action_fcurves || !adt.drivers.is_empty() || !adt.nla_tracks.is_empty()
}

/// `prop_is_animated` — fcurve `rna_path == path` and array_index
/// matches (-1 = any).
pub fn prop_is_animated(fcurve_rna_path: &str, fcurve_array_index: i32,
                        rna_path: &str, array_index: i32) -> bool {
    fcurve_rna_path == rna_path && (array_index < 0 || fcurve_array_index == array_index)
}

/// `animpath_matches_basepath`.
pub fn animpath_matches_basepath(path: &str, basepath: &str) -> bool {
    path.starts_with(basepath)
}

/// `animpath_update_basepath` — returns the new path.
pub fn animpath_update_basepath(
    rna_path: &str,
    old_basepath: &str,
    new_basepath: &str,
) -> String {
    debug_assert!(animpath_matches_basepath(rna_path, old_basepath));
    if old_basepath == new_basepath {
        return rna_path.to_string();
    }
    format!("{}{}", new_basepath, &rna_path[old_basepath.len()..])
}

/// `rna_path_rename_fix` — replace `prefix + old_infix` head of
/// `old_path` with `prefix + new_infix`; `verify` closure checks
/// path validity (returns None when old path already resolves or
/// new path does not).
pub fn rna_path_rename_fix(
    prefix: &str,
    old_infix: &str,
    new_infix: &str,
    old_path: &str,
    verify_paths: bool,
    check_rna_path_is_valid: &mut dyn FnMut(&str) -> bool,
) -> Option<String> {
    let prefix_offset = old_path.find(prefix)?;
    let old_infix_offset = old_path.find(old_infix)?;
    if prefix_offset + prefix.len() != old_infix_offset {
        return None;
    }
    if verify_paths && check_rna_path_is_valid(old_path) {
        return None;
    }
    let postfix_offset = old_infix_offset + old_infix.len();
    let mut modified_path = String::new();
    modified_path.push_str(&old_path[..prefix_offset]);
    modified_path.push_str(prefix);
    modified_path.push_str(new_infix);
    modified_path.push_str(&old_path[postfix_offset..]);
    if verify_paths && !check_rna_path_is_valid(&modified_path) {
        return None;
    }
    Some(modified_path)
}

/// `BLI_str_unescape` minimal port: `\\x` → `x`.
fn str_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&n) = chars.peek() {
                out.push(n);
                chars.next();
                continue;
            }
        }
        out.push(c);
    }
    out
}

/// `infix_to_name` — infix is `["escaped_name"]`; strip brackets
/// and unescape.
pub fn infix_to_name(infix: &str) -> String {
    debug_assert!(infix.len() >= 4);
    str_unescape(&infix[2..infix.len() - 2])
}

/// `fcurves_path_rename_fix` — fix all curves; returns whether any
/// changed. `resolve_prop_type` returns `(prop_type, is_pose_bone,
/// pose_bone_name)` when the new path resolves.
pub fn fcurves_path_rename_fix(
    curves: &mut Vec<(String, Option<String>)>, /* (rna_path, group_name) */
    prefix: &str,
    old_infix: &str,
    new_infix: &str,
    verify_paths: bool,
    check_rna_path_is_valid: &mut dyn FnMut(&str) -> bool,
    resolve_for_autoflags: &mut dyn FnMut(&str) -> Option<(i32, Option<String>)>,
) -> bool {
    let mut is_changed = false;
    for (path, group_name) in curves.iter_mut() {
        if path.is_empty() {
            continue;
        }
        let Some(new_path) =
            rna_path_rename_fix(prefix, old_infix, new_infix, path, verify_paths,
                                check_rna_path_is_valid)
        else {
            continue;
        };
        *path = new_path;
        is_changed = true;
        let Some((_prop_type, pose_bone_name)) = resolve_for_autoflags(path) else {
            continue;
        };
        if let (Some(grp), Some(pname)) = (group_name.as_mut(), pose_bone_name) {
            let old_name = infix_to_name(old_infix);
            if old_name == *grp {
                *grp = pname;
            }
        }
    }
    is_changed
}

/// `fcurves_path_remove_from_listbase` — remove curves whose
/// `rna_path` starts with `prefix` ("[\"\"]"); returns removed count.
pub fn fcurves_path_remove_from_listbase(
    rna_paths: &mut Vec<String>,
    prefix: &str,
) -> usize {
    let before = rna_paths.len();
    rna_paths.retain(|p| !p.starts_with(prefix));
    before - rna_paths.len()
}

/// `BKE_animdata_driver_path_remove` — drops driver fcurves matching
/// `prefix`; returns changed.
pub fn bke_animdata_driver_path_remove(driver_paths: &mut Vec<String>, prefix: &str) -> bool {
    fcurves_path_remove_from_listbase(driver_paths, prefix) > 0
}
