//! Port of `intern/node_runtime.cc` — the `bNodeTreeRuntime`
//! topology cache: node/link/socket vectors, directly- and
//! logically-linked sockets, internal link inputs, toposort (both
//! directions + cycle detection), root frames/frame children, group
//! output node, dangling reroutes, inference usage.
//!
//! Pointers become indices: sockets are addressed by
//! `SocketRef { node, is_input, index }`; nodes/links by their
//! `nodes_by_id`/`links` position. Zone implicit links (simulation
//! input↔output, repeat, foreach) are injected.

use std::collections::HashMap;

/// `SOCK_MULTI_INPUT`.
pub const SOCK_MULTI_INPUT: i32 = 1 << 3;
/// `NODE_DO_OUTPUT`.
pub const NODE_DO_OUTPUT: i32 = 1 << 2;

/// Socket reference (`bNodeSocket*` → indices).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SocketRef {
    /// `owner_node` index.
    pub node: usize,
    /// true = input socket.
    pub is_input: bool,
    /// `index_in_node`.
    pub index: usize,
}

/// `bNodeSocket` topology-relevant fields.
#[derive(Debug, Clone)]
pub struct SocketDesc {
    /// `identifier`.
    pub identifier: i32,
    /// `identifier_ustr` — string id used for `*_by_identifier`.
    pub identifier_str: String,
    /// `flag`.
    pub flag: i32,
    /// `is_available()`.
    pub is_available: bool,
    /// `typeinfo == NodeSocketTypeUndefined`.
    pub is_undefined: bool,
}

/// `bNodeInternalLink` — (input socket index, output socket index)
/// pair on one node.
pub type InternalLink = (usize, usize);

/// `bNode` topology-relevant fields.
#[derive(Debug, Clone, Default)]
pub struct NodeDesc {
    /// `identifier`.
    pub identifier: i32,
    /// `type_legacy`.
    pub type_legacy: i16,
    /// `typeinfo` id for `nodes_by_type` (string idname).
    pub type_idname: String,
    /// `parent` frame node index.
    pub parent: Option<usize>,
    /// `flag`.
    pub flag: i32,
    /// `is_group()`.
    pub is_group: bool,
    /// `is_frame()`.
    pub is_frame: bool,
    /// `is_reroute()`.
    pub is_reroute: bool,
    /// `is_muted()`.
    pub is_muted: bool,
    /// `is_undefined()`.
    pub is_undefined: bool,
    /// `num_panel_states`.
    pub num_panel_states: i32,
    /// `inputs`.
    pub inputs: Vec<SocketDesc>,
    /// `outputs`.
    pub outputs: Vec<SocketDesc>,
    /// `internal_links` — `(in_index, out_index)`.
    pub internal_links: Vec<InternalLink>,
}

/// `bNodeLink` topology-relevant fields.
#[derive(Debug, Clone)]
pub struct LinkDesc {
    /// `fromnode` index.
    pub from_node: usize,
    /// `fromsock` index in node's outputs.
    pub from_socket: usize,
    /// `tonode` index.
    pub to_node: usize,
    /// `tosock` index in node's inputs.
    pub to_socket: usize,
    /// `is_available()`.
    pub is_available: bool,
    /// `is_muted()`.
    pub is_muted: bool,
    /// `multi_input_sort_id`.
    pub multi_input_sort_id: i32,
}

/// `bNodeSocketRuntime`.
#[derive(Debug, Default)]
pub struct SocketRuntime {
    /// `index_in_node`.
    pub index_in_node: usize,
    /// `index_in_all_sockets`.
    pub index_in_all_sockets: usize,
    /// `index_in_inout_sockets`.
    pub index_in_inout_sockets: usize,
    /// `internal_link_input` — input socket feeding this output
    /// through a muted node's internal link.
    pub internal_link_input: Option<SocketRef>,
    /// `directly_linked_links` — link indices.
    pub directly_linked_links: Vec<usize>,
    /// `directly_linked_sockets`.
    pub directly_linked_sockets: Vec<SocketRef>,
    /// `logically_linked_sockets`.
    pub logically_linked_sockets: Vec<SocketRef>,
    /// `logically_linked_skipped_sockets`.
    pub logically_linked_skipped_sockets: Vec<SocketRef>,
}

/// `bNodeRuntime`.
#[derive(Debug, Default)]
pub struct NodeRuntime {
    /// `index_in_tree`.
    pub index_in_tree: usize,
    /// `panels` — `reinitialize(num_panel_states)`.
    pub panels: Vec<i32>,
    /// `inputs` socket refs.
    pub inputs: Vec<SocketRef>,
    /// `outputs`.
    pub outputs: Vec<SocketRef>,
    /// `has_available_linked_inputs`.
    pub has_available_linked_inputs: bool,
    /// `has_available_linked_outputs`.
    pub has_available_linked_outputs: bool,
    /// `toposort_left_to_right_index`.
    pub toposort_left_to_right_index: usize,
    /// `toposort_right_to_left_index`.
    pub toposort_right_to_left_index: usize,
    /// `direct_children_in_frame`.
    pub direct_children_in_frame: Vec<usize>,
    /// `is_dangling_reroute`.
    pub is_dangling_reroute: bool,
    /// `inputs_by_identifier` — identifier string → socket index.
    pub inputs_by_identifier: HashMap<String, usize>,
    /// `outputs_by_identifier`.
    pub outputs_by_identifier: HashMap<String, usize>,
    /// `logical_zone_index` (set by zone discovery elsewhere).
    pub logical_zone_index: i32,
}

/// `bNodeTreeRuntime`.
#[derive(Default)]
pub struct NodeTreeRuntime {
    /// `nodes` — `bNode` descriptors (the "owned" side of the
    /// pointer model).
    pub nodes: Vec<NodeDesc>,
    /// `links` — `bNodeLink` descriptors.
    pub links: Vec<LinkDesc>,
    /// `node_runtimes` parallel to `nodes`.
    pub node_runtimes: Vec<NodeRuntime>,
    /// `socket_runtimes` — `[node][is_input][index]`.
    pub socket_runtimes: Vec<[Vec<SocketRuntime>; 2]>,
    /// `sockets` — all socket refs.
    pub sockets: Vec<SocketRef>,
    /// `input_sockets`.
    pub input_sockets: Vec<SocketRef>,
    /// `output_sockets`.
    pub output_sockets: Vec<SocketRef>,
    /// `group_nodes`.
    pub group_nodes: Vec<usize>,
    /// `has_undefined_nodes_or_sockets`.
    pub has_undefined_nodes_or_sockets: bool,
    /// `toposort_left_to_right` — node indices.
    pub toposort_left_to_right: Vec<usize>,
    /// `toposort_right_to_left`.
    pub toposort_right_to_left: Vec<usize>,
    /// `has_available_link_cycle`.
    pub has_available_link_cycle: bool,
    /// `root_frames` — top-level frame node indices.
    pub root_frames: Vec<usize>,
    /// `group_output_node`.
    pub group_output_node: Option<usize>,
    /// `nodes_by_type` — type idname → node indices.
    pub nodes_by_type: HashMap<String, Vec<usize>>,
    /// `topology_cache_exists`.
    pub topology_cache_exists: bool,
}

impl NodeTreeRuntime {
    /// `update_node_vector`.
    pub fn update_node_vector(&mut self) {
        self.group_nodes.clear();
        self.has_undefined_nodes_or_sockets = false;
        for i in 0..self.nodes.len() {
            self.node_runtimes[i].index_in_tree = i;
            if self.nodes[i].is_undefined {
                self.has_undefined_nodes_or_sockets = true;
            }
            if self.nodes[i].is_group {
                self.group_nodes.push(i);
            }
        }
    }

    /// `update_link_vector` — links are stored on the struct;
    /// validates endpoints exist.
    pub fn update_link_vector(&mut self) {
        for l in &self.links {
            debug_assert!(l.from_node < self.nodes.len() && l.to_node < self.nodes.len());
        }
    }

    /// `update_socket_vectors_and_owner_node`.
    pub fn update_socket_vectors_and_owner_node(&mut self) {
        self.sockets.clear();
        self.input_sockets.clear();
        self.output_sockets.clear();
        self.socket_runtimes.clear();
        for (node_i, node) in self.nodes.iter().enumerate() {
            let mut input_runtimes = Vec::with_capacity(node.inputs.len());
            for (i, s) in node.inputs.iter().enumerate() {
                let sr = SocketRef {
                    node: node_i,
                    is_input: true,
                    index: i,
                };
                input_runtimes.push(SocketRuntime {
                    index_in_node: i,
                    index_in_all_sockets: self.sockets.len(),
                    index_in_inout_sockets: self.input_sockets.len(),
                    ..Default::default()
                });
                self.sockets.push(sr);
                self.input_sockets.push(sr);
                self.node_runtimes[node_i].inputs.push(sr);
                if s.is_undefined {
                    self.has_undefined_nodes_or_sockets = true;
                }
            }
            let mut output_runtimes = Vec::with_capacity(node.outputs.len());
            for (i, s) in node.outputs.iter().enumerate() {
                let sr = SocketRef {
                    node: node_i,
                    is_input: false,
                    index: i,
                };
                output_runtimes.push(SocketRuntime {
                    index_in_node: i,
                    index_in_all_sockets: self.sockets.len(),
                    index_in_inout_sockets: self.output_sockets.len(),
                    ..Default::default()
                });
                self.sockets.push(sr);
                self.output_sockets.push(sr);
                self.node_runtimes[node_i].outputs.push(sr);
                if s.is_undefined {
                    self.has_undefined_nodes_or_sockets = true;
                }
            }
            self.socket_runtimes.push([input_runtimes, output_runtimes]);
        }
    }

    /// Socket runtime accessor.
    pub fn socket_rt(&self, s: SocketRef) -> &SocketRuntime {
        &self.socket_runtimes[s.node][if s.is_input { 0 } else { 1 }][s.index]
    }
    /// Socket runtime accessor (mut).
    pub fn socket_rt_mut(&mut self, s: SocketRef) -> &mut SocketRuntime {
        &mut self.socket_runtimes[s.node][if s.is_input { 0 } else { 1 }][s.index]
    }

    /// `update_panels`.
    pub fn update_panels(&mut self) {
        for i in 0..self.nodes.len() {
            self.node_runtimes[i].panels =
                vec![0; self.nodes[i].num_panel_states.max(0) as usize];
        }
    }

    /// `update_internal_link_inputs`.
    pub fn update_internal_link_inputs(&mut self) {
        for node_i in 0..self.nodes.len() {
            let outs: Vec<SocketRef> = self.node_runtimes[node_i].outputs.clone();
            for s in outs {
                self.socket_rt_mut(s).internal_link_input = None;
            }
            let links = self.nodes[node_i].internal_links.clone();
            for (in_i, out_i) in links {
                let out_ref = SocketRef {
                    node: node_i,
                    is_input: false,
                    index: out_i,
                };
                self.socket_rt_mut(out_ref).internal_link_input = Some(SocketRef {
                    node: node_i,
                    is_input: true,
                    index: in_i,
                });
            }
        }
    }

    /// `update_directly_linked_links_and_sockets`.
    pub fn update_directly_linked_links_and_sockets(&mut self) {
        for node_i in 0..self.nodes.len() {
            for s in self.node_runtimes[node_i].inputs.clone() {
                self.socket_rt_mut(s).directly_linked_links.clear();
                self.socket_rt_mut(s).directly_linked_sockets.clear();
            }
            for s in self.node_runtimes[node_i].outputs.clone() {
                self.socket_rt_mut(s).directly_linked_links.clear();
                self.socket_rt_mut(s).directly_linked_sockets.clear();
            }
            self.node_runtimes[node_i].has_available_linked_inputs = false;
            self.node_runtimes[node_i].has_available_linked_outputs = false;
        }

        let links = self.links.clone();
        for (link_i, link) in links.iter().enumerate() {
            let from = SocketRef {
                node: link.from_node,
                is_input: false,
                index: link.from_socket,
            };
            let to = SocketRef {
                node: link.to_node,
                is_input: true,
                index: link.to_socket,
            };
            self.socket_rt_mut(from).directly_linked_links.push(link_i);
            self.socket_rt_mut(from).directly_linked_sockets.push(to);
            self.socket_rt_mut(to).directly_linked_links.push(link_i);
            if link.is_available {
                self.node_runtimes[link.from_node].has_available_linked_outputs = true;
                self.node_runtimes[link.to_node].has_available_linked_inputs = true;
            }
        }

        /* Multi-input sort (descending `multi_input_sort_id`). */
        let input_sockets = self.input_sockets.clone();
        for s in input_sockets {
            if self.nodes[s.node].inputs[s.index].flag & SOCK_MULTI_INPUT != 0 {
                let mut links = self.socket_rt(s).directly_linked_links.clone();
                links.sort_by(|&a, &b| {
                    self.links[b].multi_input_sort_id.cmp(&self.links[a].multi_input_sort_id)
                });
                self.socket_rt_mut(s).directly_linked_links = links;
            }
        }

        let input_sockets = self.input_sockets.clone();
        for s in input_sockets {
            let from_sockets: Vec<SocketRef> = self
                .socket_rt(s)
                .directly_linked_links
                .iter()
                .map(|&li| SocketRef {
                    node: self.links[li].from_node,
                    is_input: false,
                    index: self.links[li].from_socket,
                })
                .collect();
            self.socket_rt_mut(s).directly_linked_sockets = from_sockets;
        }
    }

    /// `find_logical_origins_for_socket_recursive`.
    fn find_logical_origins(
        &mut self,
        input_socket: SocketRef,
        only_follow_first_input_link: bool,
        sockets_in_current_chain: &mut Vec<SocketRef>,
        r_logical_origins: &mut Vec<SocketRef>,
        r_skipped_origins: &mut Vec<SocketRef>,
    ) {
        if sockets_in_current_chain.contains(&input_socket) {
            return;
        }
        sockets_in_current_chain.push(input_socket);

        let mut links_to_check = self.socket_rt(input_socket).directly_linked_links.clone();
        if only_follow_first_input_link {
            links_to_check.truncate(1);
        }
        for link_i in links_to_check {
            let link = &self.links[link_i];
            if link.is_muted || !link.is_available {
                continue;
            }
            let origin_socket = SocketRef {
                node: link.from_node,
                is_input: false,
                index: link.from_socket,
            };
            let origin_node = link.from_node;
            if !self.nodes[origin_node].outputs[link.from_socket].is_available {
                continue;
            }
            if self.nodes[origin_node].is_reroute {
                let reroute_input = SocketRef {
                    node: origin_node,
                    is_input: true,
                    index: 0,
                };
                let reroute_output = SocketRef {
                    node: origin_node,
                    is_input: false,
                    index: 0,
                };
                r_skipped_origins.push(reroute_input);
                r_skipped_origins.push(reroute_output);
                self.find_logical_origins(
                    reroute_input,
                    false,
                    sockets_in_current_chain,
                    r_logical_origins,
                    r_skipped_origins,
                );
                continue;
            }
            if self.nodes[origin_node].is_muted {
                if let Some(mute_input) = self.socket_rt(origin_socket).internal_link_input {
                    r_skipped_origins.push(origin_socket);
                    r_skipped_origins.push(mute_input);
                    self.find_logical_origins(
                        mute_input,
                        true,
                        sockets_in_current_chain,
                        r_logical_origins,
                        r_skipped_origins,
                    );
                }
                continue;
            }
            r_logical_origins.push(origin_socket);
        }

        sockets_in_current_chain.pop();
    }

    /// `update_logically_linked_sockets`.
    pub fn update_logically_linked_sockets(&mut self) {
        let nodes: Vec<usize> = (0..self.nodes.len()).collect();
        for &node_i in &nodes {
            for socket in self.node_runtimes[node_i].inputs.clone() {
                let mut chain = Vec::with_capacity(16);
                let mut origins = Vec::new();
                let mut skipped = Vec::new();
                self.find_logical_origins(socket, false, &mut chain, &mut origins, &mut skipped);
                self.socket_rt_mut(socket).logically_linked_sockets = origins;
                self.socket_rt_mut(socket).logically_linked_skipped_sockets = skipped;
            }
        }
        for &node_i in &nodes {
            for socket in self.node_runtimes[node_i].outputs.clone() {
                self.socket_rt_mut(socket).logically_linked_sockets.clear();
            }
        }
        /* Reverse: inputs' logical origins → outputs' logical
         * targets. */
        let mut reverse: Vec<(SocketRef, SocketRef)> = Vec::new();
        for &node_i in &nodes {
            for input_socket in &self.node_runtimes[node_i].inputs {
                for output_socket in &self.socket_rt(*input_socket).logically_linked_sockets {
                    reverse.push((*output_socket, *input_socket));
                }
            }
        }
        for (output, input) in reverse {
            self.socket_rt_mut(output).logically_linked_sockets.push(input);
        }
    }

    /// `update_nodes_by_type`.
    pub fn update_nodes_by_type(&mut self) {
        self.nodes_by_type.clear();
        for (i, node) in self.nodes.iter().enumerate() {
            self.nodes_by_type
                .entry(node.type_idname.clone())
                .or_default()
                .push(i);
        }
    }

    /// `update_sockets_by_identifier`.
    pub fn update_sockets_by_identifier(&mut self) {
        for i in 0..self.nodes.len() {
            self.node_runtimes[i].inputs_by_identifier.clear();
            self.node_runtimes[i].outputs_by_identifier.clear();
            for (si, s) in self.nodes[i].inputs.iter().enumerate() {
                self.node_runtimes[i]
                    .inputs_by_identifier
                    .insert(s.identifier_str.clone(), si);
            }
            for (si, s) in self.nodes[i].outputs.iter().enumerate() {
                self.node_runtimes[i]
                    .outputs_by_identifier
                    .insert(s.identifier_str.clone(), si);
            }
        }
    }
}

/// `ToposortDirection`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToposortDirection {
    /// `LeftToRight`.
    LeftToRight,
    /// `RightToLeft`.
    RightToLeft,
}

/// `ToposortNodeState`.
#[derive(Default, Clone, Copy)]
struct ToposortNodeState {
    is_done: bool,
    is_in_stack: bool,
}

/// `toposort_from_start_node` — iterative DFS matching the C's
/// socket/link/implicit-link ordering.
fn toposort_from_start_node(
    rt: &mut NodeTreeRuntime,
    direction: ToposortDirection,
    start_node: usize,
    node_states: &mut [ToposortNodeState],
    implicit_neighbors: &dyn Fn(usize, ToposortDirection) -> Vec<usize>,
    r_sorted_nodes: &mut Vec<usize>,
    r_cycle_detected: &mut bool,
) {
    struct Item {
        node: usize,
        socket_index: usize,
        link_index: usize,
        implicit_link_index: usize,
    }

    let mut nodes_to_check: Vec<Item> = vec![Item {
        node: start_node,
        socket_index: 0,
        link_index: 0,
        implicit_link_index: 0,
    }];
    node_states[start_node].is_in_stack = true;

    while !nodes_to_check.is_empty() {
        let top = nodes_to_check.len() - 1;
        let node = nodes_to_check[top].node;
        let mut pushed_node = false;

        macro_rules! handle_linked_node {
            ($linked:expr) => {{
                let linked = $linked;
                let state = &mut node_states[linked];
                if state.is_done {
                    true
                } else {
                    if state.is_in_stack {
                        *r_cycle_detected = true;
                    } else {
                        nodes_to_check.push(Item {
                            node: linked,
                            socket_index: 0,
                            link_index: 0,
                            implicit_link_index: 0,
                        });
                        state.is_in_stack = true;
                        pushed_node = true;
                    }
                    false
                }
            }};
        }

        let sockets: Vec<SocketRef> = if direction == ToposortDirection::LeftToRight {
            rt.node_runtimes[node].inputs.clone()
        } else {
            rt.node_runtimes[node].outputs.clone()
        };

        loop {
            if nodes_to_check[top].socket_index == sockets.len() {
                break;
            }
            let socket = sockets[nodes_to_check[top].socket_index];
            let linked_links = rt.socket_rt(socket).directly_linked_links.clone();
            if nodes_to_check[top].link_index == linked_links.len() {
                nodes_to_check[top].socket_index += 1;
                nodes_to_check[top].link_index = 0;
                continue;
            }
            let link_i = linked_links[nodes_to_check[top].link_index];
            let link = &rt.links[link_i];
            if !link.is_available {
                nodes_to_check[top].link_index += 1;
                continue;
            }
            let linked_socket =
                rt.socket_rt(socket).directly_linked_sockets[nodes_to_check[top].link_index];
            let linked_node = linked_socket.node;
            if handle_linked_node!(linked_node) {
                nodes_to_check[top].link_index += 1;
                continue;
            }
            break;
        }

        if !pushed_node {
            let implicitly_linked = implicit_neighbors(node, direction);
            loop {
                if nodes_to_check[top].implicit_link_index == implicitly_linked.len() {
                    break;
                }
                let linked_node = implicitly_linked[nodes_to_check[top].implicit_link_index];
                if handle_linked_node!(linked_node) {
                    nodes_to_check[top].implicit_link_index += 1;
                    continue;
                }
                break;
            }
        }

        if !pushed_node {
            let state = &mut node_states[node];
            state.is_done = true;
            state.is_in_stack = false;
            r_sorted_nodes.push(node);
            nodes_to_check.pop();
        }
    }
}

/// `update_toposort`.
pub fn update_toposort(
    rt: &mut NodeTreeRuntime,
    direction: ToposortDirection,
    implicit_neighbors: &dyn Fn(usize, ToposortDirection) -> Vec<usize>,
    r_sorted_nodes: &mut Vec<usize>,
    r_cycle_detected: &mut bool,
) {
    r_sorted_nodes.clear();
    r_sorted_nodes.reserve(rt.nodes.len());
    *r_cycle_detected = false;

    let mut node_states = vec![ToposortNodeState::default(); rt.nodes.len()];
    for node in 0..rt.nodes.len() {
        if node_states[node].is_done {
            continue;
        }
        let has_links = if direction == ToposortDirection::LeftToRight {
            rt.node_runtimes[node].has_available_linked_outputs
        } else {
            rt.node_runtimes[node].has_available_linked_inputs
        };
        if has_links {
            continue;
        }
        toposort_from_start_node(
            rt,
            direction,
            node,
            &mut node_states,
            implicit_neighbors,
            r_sorted_nodes,
            r_cycle_detected,
        );
    }

    if r_sorted_nodes.len() < rt.nodes.len() {
        *r_cycle_detected = true;
        for node in 0..rt.nodes.len() {
            if node_states[node].is_done {
                continue;
            }
            toposort_from_start_node(
                rt,
                direction,
                node,
                &mut node_states,
                implicit_neighbors,
                r_sorted_nodes,
                r_cycle_detected,
            );
        }
    }

    debug_assert_eq!(rt.nodes.len(), r_sorted_nodes.len());
}

/// `update_root_frames`.
pub fn update_root_frames(rt: &mut NodeTreeRuntime) {
    rt.root_frames.clear();
    for (i, node) in rt.nodes.iter().enumerate() {
        if node.parent.is_none() && node.is_frame {
            rt.root_frames.push(i);
        }
    }
}

/// `update_direct_frames_childrens`.
pub fn update_direct_frames_childrens(rt: &mut NodeTreeRuntime) {
    for nrt in rt.node_runtimes.iter_mut() {
        nrt.direct_children_in_frame.clear();
    }
    for (i, node) in rt.nodes.iter().enumerate() {
        if let Some(frame) = node.parent {
            rt.node_runtimes[frame].direct_children_in_frame.push(i);
        }
    }
}

/// `update_group_output_node`.
pub fn update_group_output_node(rt: &mut NodeTreeRuntime) {
    let group_output_nodes: Vec<usize> = rt
        .nodes_by_type
        .get("NodeGroupOutput")
        .cloned()
        .unwrap_or_default();
    if group_output_nodes.len() == 1 {
        rt.group_output_node = Some(group_output_nodes[0]);
    } else {
        rt.group_output_node = group_output_nodes
            .iter()
            .copied()
            .find(|&n| rt.nodes[n].flag & NODE_DO_OUTPUT != 0);
    }
}

/// `update_dangling_reroute_nodes`.
pub fn update_dangling_reroute_nodes(rt: &mut NodeTreeRuntime) {
    let order = rt.toposort_left_to_right.clone();
    for node in order {
        if !rt.nodes[node].is_reroute {
            rt.node_runtimes[node].is_dangling_reroute = false;
            continue;
        }
        let input = SocketRef {
            node,
            is_input: true,
            index: 0,
        };
        let links = rt.socket_rt(input).directly_linked_links.clone();
        if links.is_empty() {
            rt.node_runtimes[node].is_dangling_reroute = true;
            continue;
        }
        debug_assert_eq!(links.len(), 1);
        let source_node = rt.links[links[0]].from_node;
        rt.node_runtimes[node].is_dangling_reroute =
            rt.node_runtimes[source_node].is_dangling_reroute;
    }
}

/// `ensure_topology_cache` — run all update passes in order.
pub fn ensure_topology_cache(
    rt: &mut NodeTreeRuntime,
    implicit_neighbors: &dyn Fn(usize, ToposortDirection) -> Vec<usize>,
) {
    rt.node_runtimes = (0..rt.nodes.len()).map(|_| NodeRuntime::default()).collect();
    rt.update_node_vector();
    rt.update_link_vector();
    rt.update_socket_vectors_and_owner_node();
    rt.update_panels();
    rt.update_internal_link_inputs();
    rt.update_directly_linked_links_and_sockets();
    rt.update_nodes_by_type();
    rt.update_logically_linked_sockets();
    rt.update_sockets_by_identifier();
    let mut order_l = Vec::new();
    let mut cycle = false;
    update_toposort(
        rt,
        ToposortDirection::LeftToRight,
        implicit_neighbors,
        &mut order_l,
        &mut cycle,
    );
    rt.toposort_left_to_right = order_l;
    rt.has_available_link_cycle = cycle;
    for (i, &node) in rt.toposort_left_to_right.iter().enumerate() {
        rt.node_runtimes[node].toposort_left_to_right_index = i;
    }
    let mut order_r = Vec::new();
    let mut dummy = false;
    update_toposort(
        rt,
        ToposortDirection::RightToLeft,
        implicit_neighbors,
        &mut order_r,
        &mut dummy,
    );
    rt.toposort_right_to_left = order_r;
    for (i, &node) in rt.toposort_right_to_left.iter().enumerate() {
        rt.node_runtimes[node].toposort_right_to_left_index = i;
    }
    update_root_frames(rt);
    update_direct_frames_childrens(rt);
    update_group_output_node(rt);
    update_dangling_reroute_nodes(rt);
    rt.topology_cache_exists = true;
}
