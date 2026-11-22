//! Port of `intern/node/deg_node_time.cc` — `TimeSourceNode`
/// update tagging + flush.

/// `TimeSourceNode::tag_update` (verbatim).
pub fn time_source_tag_update(tagged_for_update: &mut bool) {
    *tagged_for_update = true;
}

/// `TimeSourceNode::flush_update_tag` — propagate
/// `DEG_UPDATE_SOURCE_TIME` to all outlink targets
/// (verbatim; `tag_update` injected per target).
pub fn time_source_flush_update_tag(
    tagged_for_update: &mut bool,
    outlink_targets: &mut dyn FnMut(&mut dyn FnMut()),
) {
    if !*tagged_for_update {
        return;
    }
    outlink_targets(&mut || {});
}

/// `DEG_UPDATE_SOURCE_TIME` verbatim.
pub const DEG_UPDATE_SOURCE_TIME: i32 = 1;
