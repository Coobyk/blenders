//! Port of `intern/depsgraph_writeback_sync.cc` —
/// `sync_writeback::add`.

/// `DEG_EVALUATE_SYNC_WRITEBACK_NO`.
pub const DEG_EVALUATE_SYNC_WRITEBACK_NO: i32 = 0;
/// `DEG_EVALUATE_SYNC_WRITEBACK_YES`.
pub const DEG_EVALUATE_SYNC_WRITEBACK_YES: i32 = 1;

/// `sync_writeback::add` (verbatim).
pub fn sync_writeback_add(
    is_active: bool,
    sync_writeback: i32,
    callbacks: &mut Vec<Box<dyn FnMut() + Send>>,
    f: Box<dyn FnMut() + Send>,
) {
    if !is_active || sync_writeback == DEG_EVALUATE_SYNC_WRITEBACK_NO {
        return;
    }
    callbacks.push(f);
}
