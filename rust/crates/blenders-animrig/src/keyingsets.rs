//! Port of `intern/keyingsets.cc` — KeyingSet registry,
//! find/select, source pointers, path array loop.

/// `KSP_GROUP_*`/`KSP_FLAG_WHOLE_ARRAY` verbatim.
pub const KSP_GROUP_NONE: i32 = 0;
pub const KSP_GROUP_KSNAME: i32 = 1;
pub const KSP_GROUP_NAMED: i32 = 2;
pub const KSP_FLAG_WHOLE_ARRAY: i32 = 1;
/// `ANIM_KS_AVAILABLE_ID` verbatim.
pub const ANIM_KS_AVAILABLE_ID: &str = "Available";

/// `keyingset_find_id` (verbatim — paths contains id).
pub fn keyingset_find_id(path_ids: &[usize], id: usize) -> bool {
    path_ids.contains(&id)
}

/// `builtin_keyingset_get_named` (verbatim — idname match).
pub fn builtin_keyingset_get_named<'a>(
    keyingsets: &'a [(String, usize)],
    name: &str,
) -> Option<usize> {
    if name.is_empty() {
        return None;
    }
    keyingsets
        .iter()
        .find(|(idname, _)| idname == name)
        .map(|(_, h)| *h)
}

/// `scene_get_active_keyingset` (verbatim — positive →
/// scene list idx-1, negative → builtin -idx-1).
pub fn scene_get_active_keyingset_index(active_keyingset: i32) -> (bool, usize) {
    if active_keyingset > 0 {
        (true, (active_keyingset - 1) as usize)
    } else {
        (false, ((-active_keyingset) - 1) as usize)
    }
}

/// `get_keyingset_for_autokeying` (verbatim — active ks /
/// Available / transformKS).
pub fn get_keyingset_for_autokeying(
    onlykeyingset_active: bool,
    insertavailable: bool,
    transform_ks_name: &str,
) -> String {
    if onlykeyingset_active {
        "<active>".to_string()
    } else if insertavailable {
        ANIM_KS_AVAILABLE_ID.to_string()
    } else {
        transform_ks_name.to_string()
    }
}

/// `insert_key_to_keying_set_path` groupname +
/// array range (verbatim — groupmode, whole-array).
pub fn keyingset_path_groupname(
    groupmode: i32,
    ks_name: &str,
    path_group: &str,
) -> Option<String> {
    match groupmode {
        KSP_GROUP_NONE => None,
        KSP_GROUP_KSNAME => Some(ks_name.to_string()),
        _ => Some(path_group.to_string()),
    }
}

/// Array range: `array_length=array_index`; whole-array →
/// [0, rna_array_length); always ≥1 step.
pub fn keyingset_path_array_range(
    array_index: i32,
    whole_array: bool,
    rna_array_length: i32,
) -> (i32, i32) {
    let mut array_index = array_index;
    let mut array_length = array_index;
    if whole_array {
        array_length = rna_array_length;
        array_index = 0;
    }
    if array_length == array_index {
        array_length += 1;
    }
    (array_index, array_length)
}
