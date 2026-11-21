//! Port of `intern/node_tree_dot_export.cc` — node tree → Graphviz DOT.
//!
//! Sockets/links reach us pre-flattened (`DotNode`/`DotLink`) because DNA
//! node/socket lists are `usize` handles; the graph construction and
//! attribute rules match the C++ verbatim.

use blenders_bli::dot_export::{DirectedGraph, NodePort};

/// `bNodeTreeToDotOptions` — virtuals ported as plain data: socket names and
/// optional font colors resolved by the caller per socket.
#[derive(Default)]
pub struct BNodeTreeToDotOptions;

/// `socket_name(socket)` default — `socket.identifier`.
/// `socket_font_color(socket)` default — `"#999999"` when unavailable.
pub struct DotSocket {
    pub name: String,
    /// `is_available()` — unavailable sockets render `#999999`.
    pub available: bool,
}

/// One `bNode` for export: label + input/output socket rows.
pub struct DotNode {
    /// `node->label_or_name()`.
    pub name: String,
    pub inputs: Vec<DotSocket>,
    pub outputs: Vec<DotSocket>,
}

/// One `bNodeLink`.
pub struct DotLink {
    pub from_node: usize,
    pub from_socket: usize,
    pub to_node: usize,
    pub to_socket: usize,
    /// `link.is_used()`.
    pub used: bool,
}

fn font_color(sock: &DotSocket) -> Option<&'static str> {
    if !sock.available {
        Some("#999999")
    } else {
        None
    }
}

fn port_label(prefix: &str, i: usize, sock: &DotSocket) -> String {
    match font_color(sock) {
        Some(c) => format!("<{p}{i}> <FONT COLOR=\"{c}\">{n}</FONT>", p = prefix, i = i, c = c, n = sock.name),
        None => format!("<{p}{i}> {n}", p = prefix, i = i, n = sock.name),
    }
}

/// `node_tree_to_dot` — record-labelled digraph, `rankdir=LR`, unused links greyed.
pub fn node_tree_to_dot(nodes: &[DotNode], links: &[DotLink]) -> String {
    let mut digraph = DirectedGraph::new();
    digraph.graph.attributes.set("rankdir", "LR");

    let mut node_ids = Vec::with_capacity(nodes.len());
    for node in nodes {
        let mut label = String::from("{");
        for (i, s) in node.inputs.iter().enumerate() {
            if i > 0 {
                label.push('|');
            }
            label.push_str(&port_label("i", i, s));
        }
        label.push_str("} | ");
        label.push_str(&node.name);
        label.push_str(" | {");
        for (i, s) in node.outputs.iter().enumerate() {
            if i > 0 {
                label.push('|');
            }
            label.push_str(&port_label("o", i, s));
        }
        label.push('}');
        let id = digraph.graph.new_node(&label);
        digraph.node_mut(id).attributes.set("shape", "record");
        node_ids.push(id);
    }

    for link in links {
        let from = NodePort {
            node: node_ids[link.from_node],
            port: format!("o{}", link.from_socket),
        };
        let to = NodePort {
            node: node_ids[link.to_node],
            port: format!("i{}", link.to_socket),
        };
        let e = digraph.new_edge(from, to);
        if !link.used {
            digraph.edge_mut(e).attributes.set("color", "#999999");
        }
    }
    digraph.to_dot_string()
}
