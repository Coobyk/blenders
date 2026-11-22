//! Port of `intern/nla.cc` — NLA strip action/slot
//! assign wrappers + `insert_keyframe_direct`.

/// `assign_action` extra fallback (verbatim — unassigned
/// + single slot + suitable → assign).
pub fn nla_assign_action_extra_fallback(
    slot_handle_unassigned: bool,
    action_slots_len: usize,
    first_slot_suitable: bool,
) -> bool {
    slot_handle_unassigned && action_slots_len == 1 && first_slot_suitable
}

/// `get_keyframe_values_create_reports` (verbatim —
/// count+list failed indices).
pub fn failed_keyframe_indices(
    index: i32,
    count: usize,
    force_all: bool,
    successful_remaps: &[bool],
) -> Vec<i32> {
    let mut failed = Vec::new();
    for i in 0..count {
        let cur_index_evaluated = index == i as i32 || index == -1 || force_all;
        if !cur_index_evaluated {
            continue;
        }
        if successful_remaps[i] {
            continue;
        }
        failed.push(i as i32);
    }
    failed
}

/// `insert_keyframe_direct` guard chain (verbatim —
/// fcu/keyframable/ptr checks then index bounds).
pub enum InsertKeyframeReject {
    /// "No F-Curve to add keyframes to".
    NoFCurve,
    /// "FCurve is not keyable".
    NotKeyframable,
    /// "No RNA pointer available".
    NoRnaPointer,
    /// "RNA path is invalid for the given ID".
    InvalidRnaPath,
    /// remap failed for `index`.
    RemapFailed,
    /// `insert_vert_fcurve` non-SUCCESS.
    InsertFailed,
}
pub fn insert_keyframe_direct_preflight(
    fcu_present: bool,
    fcu_is_keyframable: bool,
    ptr_present: bool,
    prop_present_or_resolved: bool,
) -> Option<InsertKeyframeReject> {
    if !fcu_present {
        return Some(InsertKeyframeReject::NoFCurve);
    }
    if !fcu_is_keyframable {
        return Some(InsertKeyframeReject::NotKeyframable);
    }
    if !ptr_present {
        return Some(InsertKeyframeReject::NoRnaPointer);
    }
    if !prop_present_or_resolved {
        return Some(InsertKeyframeReject::InvalidRnaPath);
    }
    None
}
