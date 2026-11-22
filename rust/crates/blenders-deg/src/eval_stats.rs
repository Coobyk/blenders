//! Port of `intern/eval/deg_eval_stats.cc` —
/// `deg_eval_stats_aggregate`.

/// Per-node timing stats (`stats.current_time`).
#[derive(Clone, Copy, Default)]
pub struct DegStats {
    /// `current_time`.
    pub current_time: f64,
    /// Accumulated across evaluations.
    pub total_time: f64,
}

impl DegStats {
    /// `stats.reset_current`.
    pub fn reset_current(&mut self) {
        self.current_time = 0.0;
    }
}

/// `deg_eval_stats_aggregate` — reset ID/component stats,
/// then accumulate each operation's time into its component
/// and ID (verbatim two-pass shape; node lists injected).
pub fn eval_stats_aggregate(
    id_node_stats: &mut dyn FnMut(&mut dyn FnMut(&mut DegStats)),
    operations: &[f64],
    op_to_id_comp: &mut dyn FnMut(usize, &mut dyn FnMut(&mut DegStats, &mut DegStats)),
) {
    /* Reset current evaluation stats for ID and component nodes. */
    id_node_stats(&mut |s| s.reset_current());
    /* Accumulate operation timings to components and IDs. */
    for (i, &t) in operations.iter().enumerate() {
        op_to_id_comp(i, &mut |id_stats, comp_stats| {
            id_stats.current_time += t;
            comp_stats.current_time += t;
        });
    }
}
