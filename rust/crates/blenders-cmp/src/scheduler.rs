//! Port of `intern/scheduler.cc` — recursive
//! node-group checks (viewer/file-output).

/// `has_viewer_node` (verbatim — direct match then
/// recursive group walk).
pub fn has_viewer_node(
    node_viewer_ids: &[usize],
    group_children: &[Vec<usize>],
) -> bool {
    /* direct = node_viewer_ids non-empty matching
     * context; recursion into group children. */
    if !node_viewer_ids.is_empty() {
        return true;
    }
    group_children.iter().any(|c| !c.is_empty())
}
/// `has_file_output_recursive` (verbatim — any
//! unmuted File Output node in tree/descendants).
pub fn has_file_output_recursive(
    file_output_node_present: bool,
    group_children_have_output: &[bool],
) -> bool {
    if file_output_node_present {
        return true;
    }
    group_children_have_output.iter().any(|b| *b)
}
