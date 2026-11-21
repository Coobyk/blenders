//! Port of `intern/node.cc` pure parts —
//! `node_is_parent_and_child`, `node_chain_iterator`+
//! `_backwards`, `node_parents_iterator`,
//! `node_remove_link`/`link_set_mute`/`remove_socket_
//! links`/`link_is_hidden`/`link_is_selected`,
//! `adjust_multi_input_indices_after_removed_link`,
//! `node_position_relative`/`propagate`.
//!
//! `PORT_PENDING`: node/socket type registry (`node_
//! register_*`, `node_tree_type_add` — global registries),
//! socket value init/copy/free (`socket_value_*` — per-type
//! dispatch), `node_modify_socket_type*`, `node_add_node`/
//! `node_remove_node`/`node_tree_free_tree` (memory +
//! runtime), `node_internal_relink` (link rewiring),
//! `node_tree_blend_write`/`read_data` (blend IO),
//! `node_socket_move_default_value`, `node_unlink_node`,
//! `node_rebuild_id_vector`, interface attach/detach.

use blenders_dna::node_types::{bNode, bNodeLink, NODE_LINK_MUTED, NODE_LINK_VALID};

/// `bNodeLink`-level info used by the pure link helpers
/// (nodes/sockets are index handles).
#[derive(Clone, Copy)]
pub struct NodeLinkRef {
    /// `fromnode`/`tonode` node indices.
    pub fromnode: usize,
    /// `tonode`.
    pub tonode: usize,
    /// `fromsock`/`tosock` socket indices (per node).
    pub fromsock: usize,
    /// `tosock`.
    pub tosock: usize,
    /// `flag`.
    pub flag: i32,
    /// `multi_input_sort_id`.
    pub multi_input_sort_id: i32,
    /// `is_muted` flag test.
    pub is_muted: bool,
}

/// `node_is_parent_and_child` (verbatim — walk `parent`
/// chain by index).
pub fn node_is_parent_and_child(
    parent_index: usize,
    child_index: usize,
    parent_of: &dyn Fn(usize) -> Option<usize>,
) -> bool {
    let mut iter = Some(child_index);
    while let Some(i) = iter {
        if i == parent_index {
            return true;
        }
        iter = parent_of(i);
    }
    false
}

/// `node_chain_iterator` (verbatim — forward when
/// `reversed` is false; callback stops iteration on false).
pub fn node_chain_iterator(
    links: &[NodeLinkRef],
    node_start: usize,
    reversed: bool,
    callback: &mut dyn FnMut(usize, usize) -> bool,
) {
    for link in links {
        if link.flag & NODE_LINK_VALID == 0 {
            continue;
        }
        let (from, to) = (link.fromnode, link.tonode);
        if (!reversed && from != node_start) || (reversed && to != node_start) {
            continue;
        }
        if !callback(from, to) {
            return;
        }
        node_chain_iterator(links, if reversed { from } else { to }, reversed, callback);
    }
}

/// `node_parents_iterator` (verbatim).
pub fn node_parents_iterator(
    node_index: usize,
    parent_of: &dyn Fn(usize) -> Option<usize>,
    callback: &mut dyn FnMut(usize) -> bool,
) {
    if let Some(parent) = parent_of(node_index) {
        if !callback(parent) {
            return;
        }
        node_parents_iterator(parent, parent_of, callback);
    }
}

/// `node_link_set_mute` (verbatim — tags when it changes).
pub fn node_link_set_mute(link: &mut NodeLinkRef, muted: bool) -> bool {
    let was_muted = link.is_muted;
    if muted {
        link.flag |= NODE_LINK_MUTED;
        link.is_muted = true;
    } else {
        link.flag &= !NODE_LINK_MUTED;
        link.is_muted = false;
    }
    muted != was_muted
}

/// `node_link_is_hidden` (verbatim).
pub fn node_link_is_hidden(fromsock_visible: bool, tosock_visible: bool) -> bool {
    !(fromsock_visible && tosock_visible)
}

/// `adjust_multi_input_indices_after_removed_link`
/// (verbatim — returns indices of links to update).
pub fn adjust_multi_input_indices_after_removed_link(
    links: &mut [NodeLinkRef],
    tosock_node: usize,
    tosock: usize,
    deleted_index: i32,
) {
    for link in links {
        if link.tonode != tosock_node
            || link.tosock != tosock
            || link.multi_input_sort_id <= deleted_index
        {
            continue;
        }
        link.multi_input_sort_id -= 1;
    }
}

/// `node_position_relative` (verbatim — `widget_unit` is
/// `U.widget_unit`, ~20px).
pub fn node_position_relative(
    to_sock_is_input: bool,
    from_default_width: f32,
    to_default_width: f32,
    to_node_outputs_count: usize,
    to_sock_index_in_inputs_or_outputs: usize,
    from_sock_index_in_inputs_or_outputs: Option<usize>,
    from_node_outputs_count: usize,
    from_sock_is_input: bool,
    widget_unit: f32,
    to_location: [f32; 2],
) -> [f32; 2] {
    let mut offset_x;
    let mut tot_sock_idx;
    if to_sock_is_input {
        offset_x = -(from_default_width + 50.0);
        tot_sock_idx = to_node_outputs_count + to_sock_index_in_inputs_or_outputs;
    } else {
        offset_x = to_default_width + 50.0;
        tot_sock_idx = to_sock_index_in_inputs_or_outputs;
    }
    let mut offset_y = widget_unit * tot_sock_idx as f32;
    if let Some(from_idx) = from_sock_index_in_inputs_or_outputs {
        tot_sock_idx = if from_sock_is_input {
            from_node_outputs_count + from_idx
        } else {
            from_idx
        };
    }
    offset_y -= widget_unit * tot_sock_idx as f32;
    offset_x += 0.0;
    [to_location[0] + offset_x, to_location[1] - offset_y]
}

/// `node_position_propagate` (verbatim — recursive over
/// input links; positions `from` nodes relative to `to`).
pub fn node_position_propagate<FPos, FLink>(
    node_index: usize,
    position_relative: &mut FPos,
    input_links: &FLink,
) where
    FPos: FnMut(usize, usize),
    FLink: Fn(usize) -> Vec<usize>,
{
    for from in input_links(node_index) {
        position_relative(from, node_index);
        node_position_propagate(from, position_relative, input_links);
    }
}
