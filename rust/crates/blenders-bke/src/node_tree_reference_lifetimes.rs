//! Port of `intern/node_tree_reference_lifetimes.cc` (part) —
//! `ReferenceSetInfo`/`ReferenceSetType`, `can_contain_reference`/
//! `can_contain_referenced_data`, `or_into_each_other(_masked)`,
//! `RelationsInNode` construction for zone/reroute nodes,
//! `find_reference_sets`, `set_initial_data_and_reference_bits`,
//! `get_references_coming_from_outside_zone`.
//! The propagation passes over `BitGroupVector` are modeled with
//! `Vec<Vec<bool>>` bit vectors; tree data injected via
//! [`RlTreeModel`]. `PORT_PENDING`: `pass_left_to_right`,
//! `pass_right_to_left`, `get_tree_relations`,
//! `analyse_reference_lifetimes` (need full zone/link iteration).

/// `SOCK_*` socket datatypes used.
pub const SOCK_GEOMETRY: i32 = 1;
/// See above.
pub const SOCK_BUNDLE: i32 = 2;
/// See above.
pub const SOCK_CLOSURE: i32 = 3;
/// See above.
pub const SOCK_DYNAMIC: i32 = 4;
/// See above.
pub const SOCK_CUSTOM: i32 = -1;

/// `socket_may_have_reference` — `may_be_field` injected.
pub fn socket_may_have_reference(may_be_field: bool, socket_type: i32) -> bool {
    may_be_field
        || socket_type == SOCK_BUNDLE
        || socket_type == SOCK_CLOSURE
        || socket_type == SOCK_GEOMETRY
        || socket_type == SOCK_DYNAMIC
}

/// `or_into_each_other_masked` — OR `mask`-selected bits of `a` and `b`
/// into each other; returns whether anything changed.
pub fn or_into_each_other_masked(a: &mut [bool], b: &mut [bool], mask: &[bool]) -> bool {
    let mut changed = false;
    for i in 0..a.len() {
        if !mask[i] {
            continue;
        }
        let new_a = a[i] || b[i];
        let new_b = b[i] || a[i];
        if new_a != a[i] || new_b != b[i] {
            changed = true;
        }
        a[i] = new_a;
        b[i] = new_b;
    }
    changed
}

/// `or_into_each_other` — OR all bits of `a` and `b` into each other.
pub fn or_into_each_other(a: &mut [bool], b: &mut [bool]) -> bool {
    let mut changed = false;
    for i in 0..a.len() {
        let new_a = a[i] || b[i];
        let new_b = b[i] || a[i];
        if new_a != a[i] || new_b != b[i] {
            changed = true;
        }
        a[i] = new_a;
        b[i] = new_b;
    }
    changed
}

/// `can_contain_reference` — `socket_type_supports_fields` injected.
pub fn can_contain_reference(supports_fields: bool, socket_type: i32) -> bool {
    supports_fields
        || socket_type == SOCK_GEOMETRY
        || socket_type == SOCK_BUNDLE
        || socket_type == SOCK_CLOSURE
        || socket_type == SOCK_DYNAMIC
}

/// `can_contain_referenced_data`.
pub fn can_contain_referenced_data(socket_type: i32) -> bool {
    socket_type == SOCK_GEOMETRY
        || socket_type == SOCK_BUNDLE
        || socket_type == SOCK_CLOSURE
        || socket_type == SOCK_DYNAMIC
}

/// `rl::SocketRelation` — (dst, src) index pair within a node.
#[derive(Clone, Copy, Debug)]
pub struct SocketRelation {
    /// destination socket index.
    pub dst: i32,
    /// source socket index.
    pub src: i32,
}

/// `rl::RelationsInNode` — propagation relations of one node.
#[derive(Clone, Default)]
pub struct RelationsInNode {
    /// `use_relations` (field-use inside node: {user, data}).
    pub use_relations: Vec<SocketRelation>,
    /// `available_relations` ({reference_out, data_out}).
    pub available_relations: Vec<SocketRelation>,
    /// `data_propagations` (input→output data passthrough).
    pub data_propagations: Vec<SocketRelation>,
    /// `reference_propagations` (input→output reference passthrough).
    pub reference_propagations: Vec<SocketRelation>,
}

/// Socket view used by the relations builder.
#[derive(Clone, Copy, Default)]
pub struct RlSocketView {
    /// `index` within node.
    pub index: i32,
    /// `index_in_tree`.
    pub index_in_tree: usize,
    /// `type` (SOCK_*).
    pub socket_type: i32,
    /// `may_be_field`.
    pub may_be_field: bool,
    /// `is_available`.
    pub is_available: bool,
    /// `is_directly_linked`.
    pub is_directly_linked: bool,
    /// input or output.
    pub is_input: bool,
}

/// `GEO_NODE_*` tags.
pub const GEO_NODE_SIMULATION_INPUT: i32 = 10001;
/// See above.
pub const GEO_NODE_SIMULATION_OUTPUT: i32 = 10002;
/// See above.
pub const GEO_NODE_BAKE: i32 = 10005;
/// See above.
pub const GEO_NODE_REPEAT_INPUT: i32 = 10003;
/// See above.
pub const GEO_NODE_REPEAT_OUTPUT: i32 = 10004;
/// See above.
pub const NODE_REROUTE: i32 = 20001;
/// See above.
pub const NODE_CLOSURE_OUTPUT: i32 = 30001;

/// `prepare_relations_by_node` — build `RelationsInNode` for the
/// zone-boundary and reroute node types; `decl_relations` supplies
/// declaration-derived relations for other node types.
pub fn prepare_node_relations(
    type_tag: i32,
    input_sockets: &[RlSocketView],
    output_sockets: &[RlSocketView],
    decl_relations: Option<RelationsInNode>,
) -> Option<RelationsInNode> {
    match type_tag {
        GEO_NODE_SIMULATION_INPUT | GEO_NODE_SIMULATION_OUTPUT | GEO_NODE_BAKE => {
            let mut relations = RelationsInNode::default();
            let mut prev_geometry_index: i32 = -1;
            for (i, socket) in input_sockets.iter().enumerate() {
                if socket.socket_type == SOCK_GEOMETRY {
                    prev_geometry_index = i as i32;
                    continue;
                }
                if prev_geometry_index == -1 {
                    continue;
                }
                if socket_may_have_reference(socket.may_be_field, socket.socket_type) {
                    relations.use_relations.push(SocketRelation {
                        dst: i as i32,
                        src: prev_geometry_index,
                    });
                }
            }
            let mut prev_geometry_index: i32 = -1;
            for (i, socket) in output_sockets.iter().enumerate() {
                if socket.socket_type == SOCK_GEOMETRY {
                    prev_geometry_index = i as i32;
                }
                if prev_geometry_index == -1 {
                    continue;
                }
                if socket_may_have_reference(socket.may_be_field, socket.socket_type) {
                    relations.available_relations.push(SocketRelation {
                        dst: i as i32,
                        src: prev_geometry_index,
                    });
                }
            }
            Some(relations)
        }
        GEO_NODE_REPEAT_INPUT | GEO_NODE_REPEAT_OUTPUT => {
            let mut relations = RelationsInNode::default();
            for socket in output_sockets {
                if can_contain_referenced_data(socket.socket_type) {
                    for other in output_sockets {
                        if socket_may_have_reference(other.may_be_field, other.socket_type) {
                            relations.available_relations.push(SocketRelation {
                                dst: other.index,
                                src: socket.index,
                            });
                        }
                    }
                }
            }
            for socket in input_sockets {
                if can_contain_referenced_data(socket.socket_type) {
                    for other in input_sockets {
                        if socket_may_have_reference(other.may_be_field, other.socket_type) {
                            relations.use_relations.push(SocketRelation {
                                dst: other.index,
                                src: socket.index,
                            });
                        }
                    }
                }
            }
            if type_tag == GEO_NODE_REPEAT_INPUT {
                let input_items_start = 1usize;
                let output_items_start = 1usize;
                let items_num = output_sockets.len().saturating_sub(1 + output_items_start);
                for i in 0..items_num {
                    let input_index = input_items_start + i;
                    let output_index = output_items_start + i;
                    let input_socket = input_sockets[input_index];
                    if can_contain_referenced_data(input_socket.socket_type) {
                        relations.data_propagations.push(SocketRelation {
                            dst: input_index as i32,
                            src: output_index as i32,
                        });
                    } else if socket_may_have_reference(
                        input_socket.may_be_field,
                        input_socket.socket_type,
                    ) {
                        relations.reference_propagations.push(SocketRelation {
                            dst: input_index as i32,
                            src: output_index as i32,
                        });
                    }
                }
            }
            Some(relations)
        }
        NODE_REROUTE => Some(RelationsInNode {
            data_propagations: vec![SocketRelation { dst: 0, src: 0 }],
            reference_propagations: vec![SocketRelation { dst: 0, src: 0 }],
            ..Default::default()
        }),
        _ => decl_relations,
    }
}

/// `ReferenceSetType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReferenceSetType {
    /// `GroupOutputData`.
    GroupOutputData,
    /// `GroupInputReferenceSet`.
    GroupInputReferenceSet,
    /// `LocalReferenceSet`.
    LocalReferenceSet,
    /// `ClosureInputReferenceSet`.
    ClosureInputReferenceSet,
    /// `ClosureOutputData`.
    ClosureOutputData,
}

/// `ReferenceSetInfo`.
#[derive(Clone)]
pub struct ReferenceSetInfo {
    /// `type`.
    pub r#type: ReferenceSetType,
    /// `index` (interface index) or socket tree-index for local/closure.
    pub index: usize,
    /// `socket` tree-index when type is a socket-bound set.
    pub socket: Option<usize>,
    /// `potential_data_origins` socket tree-indices.
    pub potential_data_origins: Vec<usize>,
}

/// `find_reference_sets` — build the reference sets for the tree.
/// `supports_fields` answers `socket_type_supports_fields` per
/// `eNodeSocketDatatype`; `group_input_sockets` supplies the output
/// socket tree-index per interface input per group input node.
#[allow(clippy::too_many_arguments)]
pub fn find_reference_sets(
    interface_input_types: &[i32],
    interface_output_types: &[i32],
    supports_fields: &dyn Fn(i32) -> bool,
    group_input_sockets: &dyn Fn(usize) -> Vec<usize>, /* input_i -> sockets */
    local_available: &[(usize, usize)], /* (reference_socket, data_socket) */
    closure_inputs: &[(usize, i32)],    /* (socket, socket_type) */
    closure_outputs: &[(usize, i32)],   /* (socket, socket_type) */
    closure_data_origin_sockets: &[usize],
) -> (Vec<ReferenceSetInfo>, Vec<usize>) {
    let mut reference_sets: Vec<ReferenceSetInfo> = Vec::new();
    let mut group_output_reference_sets: Vec<usize> = Vec::new();

    for (input_i, &st) in interface_input_types.iter().enumerate() {
        if can_contain_reference(supports_fields(st), st) {
            reference_sets.push(ReferenceSetInfo {
                r#type: ReferenceSetType::GroupInputReferenceSet,
                index: input_i,
                socket: None,
                potential_data_origins: Vec::new(),
            });
        }
    }
    for (output_i, &st) in interface_output_types.iter().enumerate() {
        if can_contain_referenced_data(st) {
            group_output_reference_sets.push(reference_sets.len());
            reference_sets.push(ReferenceSetInfo {
                r#type: ReferenceSetType::GroupOutputData,
                index: output_i,
                socket: None,
                potential_data_origins: Vec::new(),
            });
        }
    }
    for (input_i, &st) in interface_input_types.iter().enumerate() {
        if can_contain_referenced_data(st) {
            for socket in group_input_sockets(input_i) {
                for source in &mut reference_sets {
                    source.potential_data_origins.push(socket);
                }
            }
        }
    }
    for &(reference_socket, data_socket) in local_available {
        reference_sets.push(ReferenceSetInfo {
            r#type: ReferenceSetType::LocalReferenceSet,
            index: 0,
            socket: Some(reference_socket),
            potential_data_origins: vec![data_socket],
        });
    }

    let old_count = reference_sets.len();
    for &(socket, st) in closure_inputs {
        if can_contain_reference(supports_fields(st), st) {
            reference_sets.push(ReferenceSetInfo {
                r#type: ReferenceSetType::ClosureInputReferenceSet,
                index: 0,
                socket: Some(socket),
                potential_data_origins: Vec::new(),
            });
        }
    }
    for &(socket, st) in closure_outputs {
        if can_contain_referenced_data(st) {
            reference_sets.push(ReferenceSetInfo {
                r#type: ReferenceSetType::ClosureOutputData,
                index: 0,
                socket: Some(socket),
                potential_data_origins: Vec::new(),
            });
        }
    }
    let new_range = old_count..reference_sets.len();
    for &socket in closure_data_origin_sockets {
        for set_i in new_range.clone() {
            reference_sets[set_i].potential_data_origins.push(socket);
        }
    }

    (reference_sets, group_output_reference_sets)
}

/// `set_initial_data_and_reference_bits`.
/// `potential_data_by_socket`/`potential_reference_by_socket` are
/// `sockets × sets` bit matrices (row per socket).
pub fn set_initial_data_and_reference_bits(
    reference_sets: &[ReferenceSetInfo],
    group_input_sockets: &dyn Fn(usize) -> Vec<usize>,
    potential_data_by_socket: &mut [Vec<bool>],
    potential_reference_by_socket: &mut [Vec<bool>],
) {
    for (set_i, set) in reference_sets.iter().enumerate() {
        for &socket in &set.potential_data_origins {
            potential_data_by_socket[socket][set_i] = true;
        }
        match set.r#type {
            ReferenceSetType::LocalReferenceSet
            | ReferenceSetType::ClosureInputReferenceSet => {
                potential_reference_by_socket[set.socket.unwrap()][set_i] = true;
            }
            ReferenceSetType::GroupInputReferenceSet => {
                for socket in group_input_sockets(set.index) {
                    potential_reference_by_socket[socket][set_i] = true;
                }
            }
            ReferenceSetType::GroupOutputData | ReferenceSetType::ClosureOutputData => {}
        }
    }
}

/// `get_references_coming_from_outside_zone` — union of bits on the
/// zone's input-node inputs + border-link sources.
pub fn get_references_coming_from_outside_zone(
    zone_input_sockets: &[usize],
    border_link_from_sockets: &[usize],
    sets_num: usize,
    sources: &[&[Vec<bool>]],
) -> Vec<bool> {
    let mut found = vec![false; sets_num];
    for &src in zone_input_sockets {
        for source in sources {
            for (i, &b) in source[src].iter().enumerate() {
                if b {
                    found[i] = true;
                }
            }
        }
    }
    for &src in border_link_from_sockets {
        for source in sources {
            for (i, &b) in source[src].iter().enumerate() {
                if b {
                    found[i] = true;
                }
            }
        }
    }
    found
}
