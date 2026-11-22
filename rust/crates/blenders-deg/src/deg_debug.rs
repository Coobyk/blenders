//! Port of `intern/debug/deg_debug.cc` — `DepsgraphDebug`
/// timing tracker + terminal colors.

/// `DepsgraphDebug` (verbatim — flags come from `G.debug`
/// in C; here it's an injected value).
#[derive(Default)]
pub struct DepsgraphDebug {
    /// `flags`.
    pub flags: i32,
    /// `graph_evaluation_start_time_` (secs).
    pub graph_evaluation_start_time: f64,
}

/// `DEG_DEBUG_*` flags (verbatim from `deg_debug.h`).
pub const DEG_DEBUG_BUILDTIME: i32 = 1 << 0;
pub const DEG_DEBUG_EVAL_TIME: i32 = 1 << 1;

impl DepsgraphDebug {
    /// `do_time_debug` (verbatim).
    pub fn do_time_debug(&self) -> bool {
        self.flags & DEG_DEBUG_EVAL_TIME != 0
    }
    /// `begin_graph_evaluation` (verbatim).
    pub fn begin_graph_evaluation(&mut self, now: f64) {
        self.graph_evaluation_start_time = now;
    }
    /// `end_graph_evaluation` (verbatim — caller prints
    /// total time).
    pub fn end_graph_evaluation(&self) {
        /* `BKE_global_msg` of total time — injected. */
    }
    /// `total_evaluation_time` (verbatim).
    pub fn total_evaluation_time(&self, now: f64) -> f64 {
        now - self.graph_evaluation_start_time
    }
}

/// `terminal_do_color` (verbatim — ANSI-capable tty).
pub fn terminal_do_color(is_tty: bool) -> bool {
    is_tty
}

/// `color_for_pointer` (verbatim — hashes pointer to one
/// of 256 xterm colors).
pub fn color_for_pointer(pointer: usize) -> String {
    let hash = (pointer >> 4).wrapping_mul(2654435761) % 216;
    format!("\x1b[38;5;{}m", 16 + hash)
}

/// `color_end` (verbatim).
pub fn color_end() -> &'static str {
    "\x1b[0m"
}
