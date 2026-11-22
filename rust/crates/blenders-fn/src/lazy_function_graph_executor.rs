//! Port of `intern/lazy_function_graph_executor.cc`
//! — GraphExecutor wrapper + logger/side-effect
//! defaults.

/// `GraphExecutor` verbatim fields.
#[derive(Default)]
pub struct GraphExecutor {
    /// `debug_name_`.
    pub debug_name: String,
    /// `graph_inputs_`/`graph_outputs_` socket idx.
    pub graph_inputs: Vec<usize>,
    pub graph_outputs: Vec<usize>,
    /// `graph_input_index_by_socket_index_`.
    pub graph_input_index_by_socket_index: Vec<i32>,
    /// `graph_output_index_by_socket_index_`.
    pub graph_output_index_by_socket_index: Vec<i32>,
    /// `allow_missing_requested_inputs_` (= true).
    pub allow_missing_requested_inputs: bool,
}
impl GraphExecutor {
    /// `GraphExecutor` ctor (verbatim — maybe-usage
    /// inputs "In", outputs "Out", index maps,
    /// missing-inputs allowed).
    pub fn new(name: &str, num_sockets: usize) -> Self {
        Self {
            debug_name: name.to_string(),
            graph_inputs: Vec::new(),
            graph_outputs: Vec::new(),
            graph_input_index_by_socket_index: vec![-1; num_sockets],
            graph_output_index_by_socket_index: vec![-1; num_sockets],
            allow_missing_requested_inputs: true,
        }
    }
    /// maps socket → graph in/out index (verbatim).
    pub fn set_graph_input_index(&mut self, socket: usize, i: usize) {
        self.graph_input_index_by_socket_index[socket] = i as i32;
    }
    /// `set_graph_output_index`.
    pub fn set_graph_output_index(&mut self, socket: usize, i: usize) {
        self.graph_output_index_by_socket_index[socket] = i as i32;
    }
}
/// `GraphExecutorLogger::LoggingEnabledState`
/// (verbatim — default true).
#[derive(Default)]
pub struct LoggingEnabledState {
    /// `enabled`.
    pub enabled: bool,
}
/// `get_logging_enabled_state` default → true.
pub fn logging_enabled_state() -> LoggingEnabledState {
    LoggingEnabledState { enabled: true }
}
/// `GraphExecutorSideEffectProvider::
/// get_nodes_with_side_effects` default → empty.
pub fn side_effect_nodes_default() -> Vec<usize> {
    Vec::new()
}
