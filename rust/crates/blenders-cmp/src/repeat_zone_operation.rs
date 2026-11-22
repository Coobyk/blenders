//! Port of `intern/repeat_zone_operation.cc` —
//! N-iteration zone (input/output state copy).

/// `RepeatZoneOperation` verbatim fields.
#[derive(Default)]
pub struct RepeatZoneOperation {
    /// `repeat_index` (current iteration).
    pub repeat_index: i32,
    /// `repeat_count`.
    pub repeat_count: i32,
    /// `carry_items` (item, in-sock, out-sock).
    pub carry_items: Vec<(usize, usize, usize)>,
}
/// `execute` loop structure (verbatim — iterate
/// iterations_ times, feed outputs back).
pub fn repeat_zone_iterations(op: &RepeatZoneOperation) -> std::ops::Range<i32> {
    0..op.repeat_count
}
