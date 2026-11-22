//! Port of `intern/action_iterators.cc` — foreach
//! fcurve/action-slot-use iterators.

/// `foreach_fcurve_in_action` walk (verbatim —
/// layers→keyframe strips→channelbags→fcurves).
pub fn foreach_fcurve_in_action(
    layers: &[usize],
    strips_of: &dyn Fn(usize) -> Vec<usize>,
    strip_is_keyframe: &dyn Fn(usize) -> bool,
    bags_of: &dyn Fn(usize) -> Vec<usize>,
    bag_fcurves: &dyn Fn(usize) -> Vec<usize>,
    callback: &mut dyn FnMut(usize),
) {
    for &layer in layers {
        for strip in (strips_of)(layer) {
            if !(strip_is_keyframe)(strip) {
                continue;
            }
            for bag in (bags_of)(strip) {
                for fcu in (bag_fcurves)(bag) {
                    (callback)(fcu);
                }
            }
        }
    }
}

/// `foreach_fcurve_in_action_slot`/`_editable` (verbatim —
/// slot filter; `_editable` skips FCURVE_PROTECTED).
pub fn foreach_fcurve_in_action_slot(
    layers: &[usize],
    strips_of: &dyn Fn(usize) -> Vec<usize>,
    strip_is_keyframe: &dyn Fn(usize) -> bool,
    bags_of: &dyn Fn(usize) -> Vec<usize>,
    bag_slot_handle: &dyn Fn(usize) -> i64,
    bag_fcurves_and_protected: &dyn Fn(usize) -> Vec<(usize, bool)>,
    handle: i64,
    editable_only: bool,
    callback: &mut dyn FnMut(usize),
) {
    for &layer in layers {
        for strip in (strips_of)(layer) {
            if !(strip_is_keyframe)(strip) {
                continue;
            }
            for bag in (bags_of)(strip) {
                if (bag_slot_handle)(bag) != handle {
                    continue;
                }
                for (fcu, protected) in (bag_fcurves_and_protected)(bag) {
                    if editable_only && protected {
                        continue;
                    }
                    (callback)(fcu);
                }
            }
        }
    }
}
