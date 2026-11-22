//! Port of `intern/action_legacy.cc` — legacy
//! fcurve/channel-group traversal helpers.

/// `fcurves_all` traversal (verbatim — layers→strips→
/// keyframe strips→channelbags→fcurves).
pub fn fcurves_all(
    layer_handles: &[usize],
    strip_handles_of: &dyn Fn(usize) -> Vec<usize>,
    strip_is_keyframe: &dyn Fn(usize) -> bool,
    channelbag_handles_of: &dyn Fn(usize) -> Vec<usize>,
    fcurve_handles_of: &dyn Fn(usize) -> Vec<usize>,
) -> Vec<usize> {
    let mut all_fcurves = Vec::new();
    for &layer in layer_handles {
        for strip in (strip_handles_of)(layer) {
            if !(strip_is_keyframe)(strip) {
                continue;
            }
            for bag in (channelbag_handles_of)(strip) {
                all_fcurves.extend((fcurve_handles_of)(bag));
            }
        }
    }
    all_fcurves
}

/// `channel_groups_all` (verbatim — same walk, groups).
pub fn channel_groups_all(
    layer_handles: &[usize],
    strip_handles_of: &dyn Fn(usize) -> Vec<usize>,
    strip_is_keyframe: &dyn Fn(usize) -> bool,
    channelbag_handles_of: &dyn Fn(usize) -> Vec<usize>,
    group_handles_of: &dyn Fn(usize) -> Vec<usize>,
) -> Vec<usize> {
    let mut all_groups = Vec::new();
    for &layer in layer_handles {
        for strip in (strip_handles_of)(layer) {
            if !(strip_is_keyframe)(strip) {
                continue;
            }
            for bag in (channelbag_handles_of)(strip) {
                all_groups.extend((group_handles_of)(bag));
            }
        }
    }
    all_groups
}

/// `assigned_action_has_keyframes` (verbatim — adt &&
/// action && has_keyframes(slot)).
pub fn assigned_action_has_keyframes(adt_present: bool, action_present: bool, has_keyframes: bool) -> bool {
    if !adt_present || !action_present {
        return false;
    }
    has_keyframes
}
