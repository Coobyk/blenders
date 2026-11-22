//! Port of `intern/zone_tree_operation.cc` —
//! zone operation base w/ identifier prefixing.

/// `get_external_input_identifier` (verbatim —
/// build "external" prefixed identifier).
pub fn external_input_identifier(socket_identifier: i32) -> String {
    format!("external_{}", socket_identifier)
}
/// `get_external_output_identifier` (verbatim).
pub fn external_output_identifier(socket_identifier: i32) -> String {
    format!("external_{}", socket_identifier)
}
