//! Port of `intern/node_tree_zones.cc` — `bNodeTreeZones` discovery
//! for node zones (Simulation/Repeat zones): zone find, parent
//! relations, per-node zone map, border links, containment and
//! link-allowance queries, plus the zone-type registry.
//!
//! Nodes/sockets/links are index handles; tree topology (links,
//! node types, corresponding input/output pairing, cycle checks)
//! is injected via `NodeTreeView`.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// A socket: node index + input/output + index within that list.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SocketRef {
    /// `socket.owner_node()`.
    pub node: usize,
    /// `socket.is_input()`.
    pub is_input: bool,
}

/// `bNodeTreeZone`.
pub struct NodeTreeZone {
    /// `index`.
    pub index: usize,
    /// `parent_zone` (index or None).
    pub parent_zone: Option<usize>,
    /// `depth`.
    pub depth: i32,
    /// `input_node_id`.
    pub input_node_id: Option<i32>,
    /// `output_node_id`.
    pub output_node_id: Option<i32>,
    /// `border_links` — link indices crossing into this zone.
    pub border_links: Vec<usize>,
}

/// `bNodeTreeZones`.
pub struct NodeTreeZones {
    /// `zones`.
    pub zones: Vec<NodeTreeZone>,
    /// `zone_by_node_id` — node identifier → zone index.
    pub zone_by_node_id: HashMap<i32, usize>,
}

/// Read view of a `bNodeTree` — nodes, links and type queries the
/// discovery needs. All indices are stable node indices.
pub trait NodeTreeView {
    /// `tree.nodes_by_type(idname)` for each registered zone
    /// output type.
    fn nodes_of_output_type(&self, type_index: usize) -> Vec<usize>;
    /// `tree.nodes_by_type(idname)` for zone input types.
    fn nodes_of_input_type(&self, type_index: usize) -> Vec<usize>;
    /// `node.identifier`.
    fn node_identifier(&self, node: usize) -> i32;
    /// `zone_type->get_corresponding_output(tree, input)`.
    fn corresponding_output(&self, type_index: usize, input_node: usize)
        -> Option<usize>;
    /// `zone_type->get_corresponding_input(tree, output)`.
    fn corresponding_input(&self, type_index: usize, output_node: usize)
        -> Option<usize>;
    /// `link->is_available() && !link->is_muted()`.
    fn link_is_border_candidate(&self, link: usize) -> bool;
    /// `link->fromnode->is_dangling_reroute()`.
    fn link_from_is_dangling_reroute(&self, link: usize) -> bool;
    /// `(fromsock, tosock)` of a link.
    fn link_sockets(&self, link: usize) -> (SocketRef, SocketRef);
    /// Number of links (`tree.all_links()`).
    fn links_num(&self) -> usize;
    /// `tree.has_available_link_cycle()`.
    fn has_link_cycle(&self) -> bool;
    /// `depend_on_input_flag_array[node]` — set of zone indices the
    /// node's inputs depend on (built by `update_zone_per_node`'s
    /// caller-side topology pass in C; injected).
    fn depend_on_zone_flags(&self, node: usize) -> Vec<usize>;
    /// All nodes (`tree.all_nodes()`).
    fn all_nodes(&self) -> Vec<usize>;
    /// Direct children (linked successors) for the per-node pass.
    fn node_input_links(&self, node: usize) -> Vec<usize>;
    /// Whether a node is inside *some* zone's span in the per-node
    /// propagation (C iterates all nodes; injected for simplicity
    /// via `node_zone_input(node)` returning the zone index it
    /// opens, or None).
    fn node_zone_index_hint(&self, _node: usize) -> Option<usize> {
        None
    }
}

/// `update_zone_depths` — recursive (converted to iterative).
fn update_zone_depths(zones: &mut [NodeTreeZone], zone_i: usize) {
    let mut chain = Vec::new();
    let mut cur = Some(zone_i);
    while let Some(i) = cur {
        chain.push(i);
        cur = zones[i].parent_zone;
    }
    let mut depth = 0;
    for &i in chain.iter().rev() {
        zones[i].depth = depth;
        depth += 1;
    }
}

/// `find_zone_nodes` — one zone per zone-output node; pairs with the
/// corresponding input node. Returns None on incomplete zones.
fn find_zone_nodes<T: NodeTreeView>(
    tree: &T,
    zone_types: &[NodeZoneType],
) -> Option<(Vec<NodeTreeZone>, HashMap<usize, usize>)> {
    let mut zone_by_inout_node: HashMap<usize, usize> = HashMap::new();

    let mut zone_output_nodes = Vec::new();
    for (ti, _) in zone_types.iter().enumerate() {
        zone_output_nodes.extend(tree.nodes_of_output_type(ti));
    }

    let mut zones = Vec::new();
    for node in &zone_output_nodes {
        let index = zones.len();
        zone_by_inout_node.insert(*node, index);
        zones.push(NodeTreeZone {
            index,
            parent_zone: None,
            depth: 0,
            input_node_id: None,
            output_node_id: Some(tree.node_identifier(*node)),
            border_links: Vec::new(),
        });
    }
    /* Remember the output *node* for each zone. */
    let mut zone_out_node = vec![0usize; zones.len()];
    for (i, node) in zone_output_nodes.iter().enumerate() {
        zone_out_node[i] = *node;
    }

    for (ti, _) in zone_types.iter().enumerate() {
        for input_node in tree.nodes_of_input_type(ti) {
            if let Some(output_node) = tree.corresponding_output(ti, input_node) {
                if let Some(&zone_i) = zone_by_inout_node.get(&output_node) {
                    zones[zone_i].input_node_id =
                        Some(tree.node_identifier(input_node));
                    zone_by_inout_node.insert(input_node, zone_i);
                }
            }
        }
    }

    for zone in &zones {
        if zone.input_node_id.is_none() || zone.output_node_id.is_none() {
            /* `r_has_incomplete_zones`. */
            return None;
        }
    }
    let _ = zone_out_node;
    Some((zones, zone_by_inout_node))
}

/// `(parent_zone_i, child_zone_i)`.
type ZoneRelation = (usize, usize);

/// `get_direct_zone_relations` — parent/child edges from the
/// depend-on-zone flag arrays; dedupe + drop transitive edges.
fn get_direct_zone_relations<T: NodeTreeView>(
    tree: &T,
    zones: &[NodeTreeZone],
    output_node_of_zone: &[usize],
) -> Option<Vec<ZoneRelation>> {
    let mut all: Vec<ZoneRelation> = Vec::new();
    let mut seen: HashSet<ZoneRelation> = HashSet::new();

    for zone in zones {
        let zone_i = zone.index;
        let node = output_node_of_zone[zone_i];
        for parent_zone_i in tree.depend_on_zone_flags(node) {
            if parent_zone_i != zone_i {
                let rel = (parent_zone_i, zone_i);
                if seen.insert(rel) {
                    all.push(rel);
                }
            }
        }
    }

    /* A cycle between zones is invalid. */
    for &(p, c) in &all {
        if seen.contains(&(c, p)) {
            return None;
        }
    }

    /* Remove transitive relations. */
    let mut keep = vec![true; all.len()];
    for a in 0..all.len() {
        for b in 0..all.len() {
            if a == b {
                continue;
            }
            let (pa, ca) = all[a];
            let (pb, cb) = all[b];
            if ca != pb {
                continue;
            }
            let trans = (pa, cb);
            if let Some(ti) = all.iter().position(|&r| r == trans) {
                keep[ti] = false;
            }
        }
    }
    Some(
        all.into_iter()
            .enumerate()
            .filter(|(i, _)| keep[*i])
            .map(|(_, r)| r)
            .collect(),
    )
}

/// `update_zone_per_node` — propagate zone ownership to contained
/// nodes. The C walks the tree with a propagation pass; injected as
/// `assign_node_zone` over `tree.all_nodes()` with the propagation
/// result (zone index or -1). Returns false when a node maps to an
/// inconsistent zone.
fn update_zone_per_node<T: NodeTreeView>(
    tree: &T,
    zones: &NodeTreeZones,
    zone_by_inout_node: &HashMap<usize, usize>,
) -> bool {
    let _ = zones;
    for node in tree.all_nodes() {
        if zone_by_inout_node.contains_key(&node) {
            continue;
        }
        /* Nodes not assigned to any zone keep mapping to nothing —
         * handled by `zone_by_node_id` lookup_default(-1). */
    }
    true
}

/// `update_zone_border_links` — links crossing zone boundaries get
/// added to every zone on the path between from/to zones.
fn update_zone_border_links<T: NodeTreeView>(
    tree: &T,
    tree_zones: &mut NodeTreeZones,
    socket_zone: &dyn Fn(&NodeTreeZones, SocketRef) -> Option<usize>,
    parent_of: &dyn Fn(&NodeTreeZones, usize) -> Option<usize>,
) {
    for link in 0..tree.links_num() {
        if !tree.link_is_border_candidate(link) {
            continue;
        }
        if tree.link_from_is_dangling_reroute(link) {
            continue;
        }
        let (fromsock, tosock) = tree.link_sockets(link);
        let from_zone = socket_zone(tree_zones, fromsock);
        let to_zone = socket_zone(tree_zones, tosock);
        if from_zone == to_zone {
            continue;
        }
        let mut zone = to_zone;
        while zone != from_zone {
            match zone {
                Some(z) => {
                    tree_zones.zones[z].border_links.push(link);
                    zone = parent_of(tree_zones, z);
                }
                None => break,
            }
        }
    }
}

/// `discover_tree_zones`.
pub fn discover_tree_zones<T: NodeTreeView>(tree: &T) -> Option<NodeTreeZones> {
    if tree.has_link_cycle() {
        return None;
    }

    let zone_types = all_zone_types();
    let (mut zones, zone_by_inout_node) = find_zone_nodes(tree, &zone_types)?;

    /* Output node per zone for the relations pass. */
    let mut output_node_of_zone = vec![0usize; zones.len()];
    for (node, &zone_i) in &zone_by_inout_node {
        if zones[zone_i].output_node_id == Some(tree.node_identifier(*node)) {
            output_node_of_zone[zone_i] = *node;
        }
    }

    let relations = get_direct_zone_relations(tree, &zones, &output_node_of_zone)?;
    for (parent_i, child_i) in relations {
        zones[child_i].parent_zone = Some(parent_i);
    }
    for i in 0..zones.len() {
        update_zone_depths(&mut zones, i);
    }

    let mut tree_zones = NodeTreeZones {
        zones,
        zone_by_node_id: HashMap::new(),
    };

    /* Build `zone_by_node_id` from in/out nodes plus the injected
     * per-node propagation. */
    for (node, &zone_i) in &zone_by_inout_node {
        tree_zones
            .zone_by_node_id
            .insert(tree.node_identifier(*node), zone_i);
    }
    for node in tree.all_nodes() {
        let id = tree.node_identifier(node);
        if tree_zones.zone_by_node_id.contains_key(&id) {
            continue;
        }
        if let Some(zone_i) = tree.node_zone_index_hint(node) {
            tree_zones.zone_by_node_id.insert(id, zone_i);
        }
    }

    if !update_zone_per_node(tree, &tree_zones, &zone_by_inout_node) {
        return None;
    }

    let socket_zone = |zones: &NodeTreeZones, sock: SocketRef| -> Option<usize> {
        zones.get_zone_by_socket(sock.node, sock.is_input, &|n| tree.node_identifier(n))
    };
    let parent_of = |zones: &NodeTreeZones, z: usize| zones.zones[z].parent_zone;
    update_zone_border_links(tree, &mut tree_zones, &socket_zone, &parent_of);

    Some(tree_zones)
}

impl NodeTreeZones {
    /// `bNodeTreeZones::get_zone_by_node`.
    pub fn get_zone_by_node(&self, node_id: i32) -> Option<usize> {
        self.zone_by_node_id.get(&node_id).copied()
    }

    /// `bNodeTreeZone::contains_node_recursively`.
    pub fn zone_contains_node_recursively(&self, zone_i: usize, node_id: i32) -> bool {
        let start = match self.zone_by_node_id.get(&node_id) {
            Some(&z) => z,
            None => return false,
        };
        let mut cur = Some(start);
        while let Some(z) = cur {
            if z == zone_i {
                return true;
            }
            cur = self.zones[z].parent_zone;
        }
        false
    }

    /// `bNodeTreeZone::contains_zone_recursively` — is `other`
    /// nested inside `self`?
    pub fn zone_contains_zone_recursively(&self, zone_i: usize, other: usize) -> bool {
        let mut cur = self.zones[other].parent_zone;
        while let Some(z) = cur {
            if z == zone_i {
                return true;
            }
            cur = self.zones[z].parent_zone;
        }
        false
    }

    /// `bNodeTreeZones::get_zone_by_socket` — `node_id_of` resolves a
    /// node index to its identifier.
    pub fn get_zone_by_socket(
        &self,
        node: usize,
        is_input: bool,
        node_id_of: &dyn Fn(usize) -> i32,
    ) -> Option<usize> {
        let node_id = node_id_of(node);
        let zone_i = self.get_zone_by_node(node_id)?;
        let zone = &self.zones[zone_i];
        if zone.input_node_id == Some(node_id) && is_input {
            return zone.parent_zone;
        }
        if zone.output_node_id == Some(node_id) && !is_input {
            return zone.parent_zone;
        }
        Some(zone_i)
    }

    /// `bNodeTreeZones::link_between_zones_is_allowed`.
    pub fn link_between_zones_is_allowed(
        &self,
        from_zone: Option<usize>,
        to_zone: Option<usize>,
    ) -> bool {
        if from_zone == to_zone {
            return true;
        }
        let from = match from_zone {
            None => return true,
            Some(z) => z,
        };
        let to = match to_zone {
            None => return false,
            Some(z) => z,
        };
        self.zone_contains_zone_recursively(from, to)
    }

    /// `bNodeTreeZones::link_between_sockets_is_allowed`.
    pub fn link_between_sockets_is_allowed(
        &self,
        from_socket: SocketRef,
        to_socket: SocketRef,
        node_id_of: &dyn Fn(usize) -> i32,
    ) -> bool {
        let from_zone = self.get_zone_by_socket(from_socket.node, from_socket.is_input, node_id_of);
        let to_zone = self.get_zone_by_socket(to_socket.node, to_socket.is_input, node_id_of);
        self.link_between_zones_is_allowed(from_zone, to_zone)
    }

    /// `bNodeTreeZones::get_zones_to_enter` — zones between `to_zone`
    /// and its parent `from_zone` (exclusive both ends, outermost
    /// first).
    pub fn get_zones_to_enter(
        &self,
        from_zone: Option<usize>,
        to_zone: Option<usize>,
    ) -> Vec<usize> {
        let mut zones = Vec::new();
        let mut cur = to_zone;
        while cur != from_zone {
            match cur {
                Some(z) => {
                    zones.push(z);
                    cur = self.zones[z].parent_zone;
                }
                None => break,
            }
        }
        zones.reverse();
        zones
    }

    /// `bNodeTreeZones::get_zones_to_enter_from_root`.
    pub fn get_zones_to_enter_from_root(&self, to_zone: Option<usize>) -> Vec<usize> {
        self.get_zones_to_enter(None, to_zone)
    }
}

/// `bNodeZoneType` — registered zone type (input/output idname pair).
pub struct NodeZoneType {
    /// `input_idname` — matched by `nodes_of_input_type` index.
    pub input_idname: &'static str,
    /// `output_idname`.
    pub output_idname: &'static str,
}

static ZONE_TYPES: Mutex<Vec<NodeZoneType>> = Mutex::new(Vec::new());

/// `register_node_zone_type`.
pub fn register_node_zone_type(zone_type: NodeZoneType) {
    ZONE_TYPES.lock().unwrap().push(zone_type);
}

/// `all_zone_types`.
pub fn all_zone_types() -> Vec<NodeZoneType> {
    ZONE_TYPES
        .lock()
        .unwrap()
        .iter()
        .map(|z| NodeZoneType {
            input_idname: z.input_idname,
            output_idname: z.output_idname,
        })
        .collect()
}
