//! Port of `intern/node_operation.cc` — node-backed
//! operation (declare per socket + link map).

/// `NodeOperation` verbatim fields.
#[derive(Default)]
pub struct NodeOperation {
    /// `node` handle.
    pub node: usize,
    /// `output_sockets_to_output_identifiers_map_`.
    pub output_sockets_to_output_identifiers: std::collections::HashMap<usize, String>,
    /// `input_sockets` declared per socket id.
    pub input_identifiers: std::collections::HashMap<usize, String>,
}
/// `get_output_identifier_from_output_socket`
/// (verbatim — socket identifier string).
pub fn output_identifier_from_output_socket(socket_identifier: i32) -> String {
    socket_identifier.to_string()
}
/// `get_input_identifier_from_input_socket`.
pub fn input_identifier_from_input_socket(socket_identifier: i32) -> String {
    socket_identifier.to_string()
}
