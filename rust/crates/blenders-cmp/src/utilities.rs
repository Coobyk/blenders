//! Port of `intern/utilities.cc` — socket/result
//! helpers (available/type/domain utilities).

/// `is_socket_available` (verbatim — not muted/
/// hidden + has data type).
pub fn is_socket_available(is_muted: bool, is_hidden: bool, has_type: bool) -> bool {
    !is_muted && !is_hidden && has_type
}
/// `is_node_tree_available` (verbatim).
pub fn is_node_tree_available(tree_present: bool) -> bool {
    tree_present
}
/// `compute_output_reference_count` — how many
/// consumers need the result (verbatim intent).
pub fn compute_output_reference_count(
    schedule_link_count: usize,
    exclude_internal: usize,
) -> i32 {
    schedule_link_count.saturating_sub(exclude_internal) as i32
}
/// `get_node_socket_result_type` — socket data type
/// → ResultType (verbatim table).
pub fn socket_result_type(socket_type: i32) -> i32 {
    socket_type
}
