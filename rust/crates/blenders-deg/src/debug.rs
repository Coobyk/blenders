//! Port of `intern/debug/deg_debug.cc` — `DepsgraphDebug`
/// timing + terminal colors.

/// `G_DEBUG_DEPSGRAPH_*` verbatim (`BKE_global.hh` bits).
pub const G_DEBUG_DEPSGRAPH_PRETTY: i32 = 1 << 9;
/// `G_DEBUG_DEPSGRAPH_TIME`.
pub const G_DEBUG_DEPSGRAPH_TIME: i32 = 1 << 10;

/// `DepsgraphDebug` (verbatim).
pub struct DepsgraphDebug {
    /// `flags` (`G.debug`).
    pub flags: i32,
    /// `name`.
    pub name: String,
    /// `graph_evaluation_start_time_`.
    pub graph_evaluation_start_time: f64,
    /// `graph_evaluation_total_time_`.
    pub graph_evaluation_total_time: f64,
}

impl DepsgraphDebug {
    /// `do_time_debug` (verbatim).
    pub fn do_time_debug(&self) -> bool {
        (self.flags & G_DEBUG_DEPSGRAPH_TIME) != 0
    }
    /// `begin_graph_evaluation` — `current_time` =
    /// `BLI_time_now_seconds()`.
    pub fn begin_graph_evaluation(&mut self, current_time: f64) {
        self.graph_evaluation_start_time = current_time;
    }
    /// `end_graph_evaluation` (verbatim print).
    pub fn end_graph_evaluation(&mut self, end_time: f64) {
        self.graph_evaluation_total_time = end_time - self.graph_evaluation_start_time;
        if !self.do_time_debug() {
            return;
        }
        if self.name.is_empty() {
            println!("Depsgraph updated in {} seconds.", self.graph_evaluation_total_time);
        } else {
            println!(
                "Depsgraph [{}] updated in {} seconds.",
                self.name, self.graph_evaluation_total_time
            );
        }
    }
    /// `total_evaluation_time`.
    pub fn total_evaluation_time(&self) -> f64 {
        self.graph_evaluation_total_time
    }
}

/// `terminal_do_color` (verbatim).
pub fn terminal_do_color(g_debug: i32) -> bool {
    (g_debug & G_DEBUG_DEPSGRAPH_PRETTY) != 0
}

/// `TRUECOLOR_ANSI_COLOR_FORMAT`/`FINISH`.
pub const TRUECOLOR_ANSI_COLOR_FORMAT: &str = "\u{1b}[38;2;{};{};{}m";
/// finish.
pub const TRUECOLOR_ANSI_COLOR_FINISH: &str = "\u{1b}[0m";

/// `color_for_pointer` (verbatim — hash → rgb).
pub fn color_for_pointer(pointer: usize, g_debug: i32, hash_to_rgb: &dyn Fn(usize) -> (i32, i32, i32)) -> String {
    if !terminal_do_color(g_debug) {
        return String::new();
    }
    let (r, g, b) = hash_to_rgb(pointer);
    format!("\u{1b}[38;2;{};{};{}m", r, g, b)
}

/// `color_end` (verbatim).
pub fn color_end(g_debug: i32) -> String {
    if !terminal_do_color(g_debug) {
        return String::new();
    }
    TRUECOLOR_ANSI_COLOR_FINISH.to_string()
}
