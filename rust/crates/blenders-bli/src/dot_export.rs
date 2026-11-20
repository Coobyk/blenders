//! Port of `dot_export.cc` — Graphviz DOT export (`blender::dot_export`).
//! Nodes/clusters are indices into the graph's arenas.

use std::collections::{HashMap, HashSet};

/// `Attributes` — ordered key/value attributes.
#[derive(Default)]
pub struct Attributes {
    attributes: Vec<(String, String)>,
}

impl Attributes {
    /// `attributes.set(key, value)`.
    pub fn set(&mut self, key: &str, value: &str) {
        if let Some(kv) = self.attributes.iter_mut().find(|(k, _)| k == key) {
            kv.1 = value.to_string();
        } else {
            self.attributes.push((key.to_string(), value.to_string()));
        }
    }
    /// `export__as_bracket_list`.
    pub fn export_as_bracket_list(&self, ss: &mut String) {
        ss.push('[');
        for (key, value) in &self.attributes {
            if value.starts_with('<') {
                ss.push_str(&format!("{key}={value}, "));
            } else {
                ss.push_str(&format!("{key}=\""));
                for c in value.chars() {
                    if c == '"' {
                        ss.push('\\');
                    }
                    ss.push(c);
                }
                ss.push_str("\", ");
            }
        }
        ss.push(']');
    }
}

/// `NodePort` — node + optional named port.
#[derive(Clone, Default)]
pub struct NodePort {
    pub node: usize,
    pub port: String,
}

/// `Node`.
pub struct Node {
    pub attributes: Attributes,
    cluster: Option<usize>,
    id: usize,
}

impl Node {
    /// `NodePort::to_dot_string` — `id:port`.
    pub fn port_id(port: &NodePort) -> String {
        if port.port.is_empty() {
            format!("node_{}", port.node)
        } else {
            format!("node_{}:{}", port.node, port.port)
        }
    }
    /// `export__as_id`.
    fn id_str(&self) -> String {
        format!("node_{}", self.id)
    }
    /// `export__as_declaration`.
    fn export_as_declaration(&self, ss: &mut String) {
        ss.push_str(&self.id_str());
        ss.push(' ');
        self.attributes.export_as_bracket_list(ss);
        ss.push('\n');
    }
    /// `node.parent_cluster()`.
    pub fn parent_cluster(&self) -> Option<usize> {
        self.cluster
    }
}

/// `Cluster`.
pub struct Cluster {
    pub attributes: Attributes,
    parent: Option<usize>,
    nodes: HashSet<usize>,
    children: HashSet<usize>,
    id: usize,
}

impl Cluster {
    fn name(&self) -> String {
        format!("cluster_{}", self.id)
    }
    /// `Cluster::contains` — is `node`'s chain inside this cluster.
    pub fn contains(&self, graph: &Graph, node_id: usize) -> bool {
        let mut current = graph.nodes[node_id].parent_cluster();
        while let Some(c) = current {
            if c == self.id {
                return true;
            }
            current = graph.clusters[c].parent;
        }
        false
    }
    /// `Cluster::export__declare_nodes_and_clusters`.
    fn export_declare(&self, graph: &Graph, ss: &mut String) {
        ss.push_str(&format!("subgraph {} {{\n", self.name()));
        ss.push_str("graph ");
        self.attributes.export_as_bracket_list(ss);
        ss.push_str("\n\n");
        for &n in &self.nodes {
            graph.nodes[n].export_as_declaration(ss);
        }
        for &c in &self.children {
            graph.clusters[c].export_declare(graph, ss);
        }
        ss.push_str("}\n");
    }
    /// `Cluster::set_random_cluster_bgcolors` (recursive).
    pub fn set_random_cluster_bgcolors(graph: &mut Graph, cluster: usize) {
        let hue = rand_f32();
        graph.clusters[cluster].attributes.set(
            "bgcolor",
            &color_attr_from_hsv(hue, 0.3, 0.8),
        );
        let children: Vec<usize> = graph.clusters[cluster].children.iter().copied().collect();
        for c in children {
            Self::set_random_cluster_bgcolors(graph, c);
        }
    }
}

fn rand_f32() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (t % 1000) as f32 / 1000.0
}

/// `DirectedEdge` / `UndirectedEdge`.
pub struct Edge {
    pub a: NodePort,
    pub b: NodePort,
    pub attributes: Attributes,
}

/// `Graph` base.
pub struct Graph {
    pub attributes: Attributes,
    nodes: Vec<Node>,
    clusters: Vec<Cluster>,
    edges: Vec<Edge>,
    top_level_nodes: HashSet<usize>,
    top_level_clusters: HashSet<usize>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            attributes: Attributes::default(),
            nodes: Vec::new(),
            clusters: Vec::new(),
            edges: Vec::new(),
            top_level_nodes: HashSet::new(),
            top_level_clusters: HashSet::new(),
        }
    }
    /// `graph.new_node(label)` → node index.
    pub fn new_node(&mut self, label: &str) -> usize {
        let id = self.nodes.len();
        let mut attributes = Attributes::default();
        attributes.set("label", label);
        self.nodes.push(Node {
            attributes,
            cluster: None,
            id,
        });
        self.top_level_nodes.insert(id);
        id
    }
    /// `graph.new_cluster(label)` → cluster index.
    pub fn new_cluster(&mut self, label: &str) -> usize {
        let id = self.clusters.len();
        let mut attributes = Attributes::default();
        attributes.set("label", label);
        self.clusters.push(Cluster {
            attributes,
            parent: None,
            nodes: HashSet::new(),
            children: HashSet::new(),
            id,
        });
        self.top_level_clusters.insert(id);
        id
    }
    /// `Node::set_parent_cluster`.
    pub fn node_set_parent_cluster(&mut self, node: usize, cluster: Option<usize>) {
        if self.nodes[node].cluster == cluster {
            return;
        }
        match (self.nodes[node].cluster, cluster) {
            (None, Some(c)) => {
                self.top_level_nodes.remove(&node);
                self.clusters[c].nodes.insert(node);
            }
            (Some(p), None) => {
                self.clusters[p].nodes.remove(&node);
                self.top_level_nodes.insert(node);
            }
            (Some(p), Some(c)) => {
                self.clusters[p].nodes.remove(&node);
                self.clusters[c].nodes.insert(node);
            }
            (None, None) => {}
        }
        self.nodes[node].cluster = cluster;
    }
    /// `Cluster::set_parent_cluster`.
    pub fn cluster_set_parent_cluster(&mut self, cluster: usize, new_parent: Option<usize>) {
        if self.clusters[cluster].parent == new_parent {
            return;
        }
        match (self.clusters[cluster].parent, new_parent) {
            (None, Some(p)) => {
                self.top_level_clusters.remove(&cluster);
                self.clusters[p].children.insert(cluster);
            }
            (Some(p), None) => {
                self.clusters[p].children.remove(&cluster);
                self.top_level_clusters.insert(cluster);
            }
            (Some(p), Some(np)) => {
                self.clusters[p].children.remove(&cluster);
                self.clusters[np].children.insert(cluster);
            }
            (None, None) => {}
        }
        self.clusters[cluster].parent = new_parent;
    }
    /// `Graph::set_random_cluster_bgcolors`.
    pub fn set_random_cluster_bgcolors(&mut self) {
        let tops: Vec<usize> = self.top_level_clusters.iter().copied().collect();
        for c in tops {
            Cluster::set_random_cluster_bgcolors(self, c);
        }
    }
    fn export_declare_nodes_and_clusters(&self, ss: &mut String) {
        ss.push_str("graph ");
        self.attributes.export_as_bracket_list(ss);
        ss.push_str("\n\n");
        for &n in &self.top_level_nodes {
            self.nodes[n].export_as_declaration(ss);
        }
        for &c in &self.top_level_clusters {
            self.clusters[c].export_declare(self, ss);
        }
    }
}

/// `DirectedGraph`.
pub struct DirectedGraph {
    pub graph: Graph,
}
/// `UndirectedGraph`.
pub struct UndirectedGraph {
    pub graph: Graph,
}

impl DirectedGraph {
    pub fn new() -> Self {
        DirectedGraph { graph: Graph::new() }
    }
    /// `digraph.new_edge(from, to)` → edge index.
    pub fn new_edge(&mut self, from: NodePort, to: NodePort) -> usize {
        self.graph.edges.push(Edge {
            a: from,
            b: to,
            attributes: Attributes::default(),
        });
        self.graph.edges.len() - 1
    }
    /// `DirectedGraph::to_dot_string`.
    pub fn to_dot_string(&self) -> String {
        let mut ss = String::from("digraph {\n");
        self.graph.export_declare_nodes_and_clusters(&mut ss);
        ss.push('\n');
        for e in &self.graph.edges {
            ss.push_str(&Node::port_id(&e.a));
            ss.push_str(" -> ");
            ss.push_str(&Node::port_id(&e.b));
            ss.push(' ');
            e.attributes.export_as_bracket_list(&mut ss);
            ss.push('\n');
        }
        ss.push_str("}\n");
        ss
    }
}

impl UndirectedGraph {
    pub fn new() -> Self {
        UndirectedGraph { graph: Graph::new() }
    }
    /// `undigraph.new_edge(a, b)`.
    pub fn new_edge(&mut self, a: NodePort, b: NodePort) -> usize {
        self.graph.edges.push(Edge {
            a,
            b,
            attributes: Attributes::default(),
        });
        self.graph.edges.len() - 1
    }
    /// `UndirectedGraph::to_dot_string`.
    pub fn to_dot_string(&self) -> String {
        let mut ss = String::from("graph {\n");
        self.graph.export_declare_nodes_and_clusters(&mut ss);
        ss.push('\n');
        for e in &self.graph.edges {
            ss.push_str(&Node::port_id(&e.a));
            ss.push_str(" -- ");
            ss.push_str(&Node::port_id(&e.b));
            ss.push(' ');
            e.attributes.export_as_bracket_list(&mut ss);
            ss.push('\n');
        }
        ss.push_str("}\n");
        ss
    }
}

/// `color_attr_from_hsv` — `"<hue> <sat> <val>"` string.
pub fn color_attr_from_hsv(h: f32, s: f32, v: f32) -> String {
    format!("{h:.4} {s:.4} {v:.4}")
}
