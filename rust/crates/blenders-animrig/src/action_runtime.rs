//! Port of `intern/action_runtime.cc` — slot user cache
//! rebuild.

/// `rebuild_slot_user_cache` (verbatim — clear all slot
//! users, visit animdata-capable IDs + embedded
//! node trees).
pub fn rebuild_slot_user_cache(
    actions: &[usize],
    clear_slot_users: &mut dyn FnMut(usize),
    animdata_ids: &[usize],
    embedded_id_of: &dyn Fn(usize) -> Option<usize>,
    foreach_action_slot_use: &mut dyn FnMut(usize, &mut dyn FnMut(usize, i64)),
    slot_for_handle: &dyn Fn(usize, i64) -> Option<usize>,
    slot_users_add: &mut dyn FnMut(usize, usize),
) {
    for &action in actions {
        (clear_slot_users)(action);
    }
    let mut visited_ids = std::collections::HashSet::new();
    for &id in animdata_ids {
        if !visited_ids.insert(id) {
            continue;
        }
        (foreach_action_slot_use)(id, &mut |action, slot_handle| {
            if let Some(slot) = (slot_for_handle)(action, slot_handle) {
                (slot_users_add)(slot, id);
            }
        });
        if let Some(embedded) = (embedded_id_of)(id) {
            if visited_ids.insert(embedded) {
                (foreach_action_slot_use)(embedded, &mut |action, slot_handle| {
                    if let Some(slot) = (slot_for_handle)(action, slot_handle) {
                        (slot_users_add)(slot, embedded);
                    }
                });
            }
        }
    }
}
