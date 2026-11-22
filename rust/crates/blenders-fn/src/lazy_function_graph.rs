//! Port of `intern/lazy_function_graph.cc` —
//! lazy-function graph (nodes, sockets, links) as
//! index arenas.

/// `Socket` verbatim fields (foreign ptrs → indices).
#[derive(Clone)]
pub struct Socket {
    /// `index_in_node_`/`index_in_graph_`.
    pub index_in_node: usize,
    pub index_in_graph: usize,
    /// `is_input_`.
    pub is_input: bool,
    /// `node_` owner index.
    pub node: usize,
    /// `type_` (CPPType → type index).
    pub type_: usize,
    /// `origin_` (for input sockets).
    pub origin: Option<usize>,
    /// `targets_` (for output sockets).
    pub targets: Vec<usize>,
    /// `default_value_` handle.
    pub default_value: Option<usize>,
}
/// `Node` verbatim fields.
#[derive(Clone)]
pub struct Node {
    /// `is_function_` vs interface.
    pub is_function: bool,
    /// `fn_` (LazyFunction handle).
    pub fn_: usize,
    /// `inputs_`/`outputs_` socket indices.
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    /// `socket_names_` (interface nodes).
    pub socket_names: Vec<String>,
    /// `index_in_graph_`.
    pub index_in_graph: usize,
}
impl Node {
    /// `is_function`/`is_interface` (verbatim).
    pub fn is_function(&self) -> bool {
        self.is_function
    }
    pub fn is_interface(&self) -> bool {
        !self.is_function
    }
    /// `name` (verbatim — fn name else "Interface").
    pub fn name(&self, fn_names: &[String]) -> String {
        if self.is_function {
            return fn_names.get(self.fn_).cloned().unwrap_or_default();
        }
        "Interface".to_string()
    }
}
/// `Graph` verbatim.
#[derive(Default)]
pub struct Graph {
    /// `name_`.
    pub name: String,
    /// `nodes_` arena.
    pub nodes: Vec<Node>,
    /// `sockets` arena (all input+output sockets).
    pub sockets: Vec<Socket>,
    /// `graph_input_node_`/`graph_output_node_` indices.
    pub graph_input_node: usize,
    pub graph_output_node: usize,
    /// `socket_num_`.
    pub socket_num: usize,
}
impl Graph {
    /// `Graph(name)` (verbatim — input+output
    /// interface nodes 0 and 1).
    pub fn new(name: &str) -> Self {
        let mut g = Graph {
            name: name.to_string(),
            nodes: vec![
                Node { is_function: false, ..Default::default() },
                Node { is_function: false, ..Default::default() },
            ],
            ..Default::default()
        };
        g.graph_input_node = 0;
        g.graph_output_node = 1;
        g.nodes[0].index_in_graph = 0;
        g.nodes[1].index_in_graph = 1;
        g
    }
    /// `add_function` (verbatim — node + input/output
    /// sockets).
    pub fn add_function(&mut self, fn_: usize, inputs: &[usize], outputs: &[usize]) -> usize {
        let node_i = self.nodes.len();
        let mut node = Node {
            is_function: true,
            fn_,
            index_in_graph: node_i,
            ..Default::default()
        };
        for (i, &t) in inputs.iter().enumerate() {
            let si = self.sockets.len();
            self.sockets.push(Socket {
                index_in_node: i,
                is_input: true,
                node: node_i,
                type_: t,
                ..Socket { index_in_graph: 0, index_in_node: i, is_input: true, node: node_i, type_: t, origin: None, targets: Vec::new(), default_value: None, index_in_graph: 0 }
            });
            node.inputs.push(si);
        }
        for (i, &t) in outputs.iter().enumerate() {
            let si = self.sockets.len();
            self.sockets.push(Socket {
                index_in_node: i,
                is_input: false,
                node: node_i,
                type_: t,
                ..Socket { index_in_graph: 0, index_in_node: i, is_input: false, node: node_i, type_: t, origin: None, targets: Vec::new(), default_value: None, index_in_graph: 0 }
            });
            node.outputs.push(si);
        }
        self.nodes.push(node);
        node_i
    }
    /// `add_input` (verbatim — graph input =
    /// output socket on node 0).
    pub fn add_input(&mut self, type_: usize, name: String) -> usize {
        let si = self.sockets.len();
        let index_in_node = self.nodes[self.graph_input_node].outputs.len();
        self.sockets.push(Socket {
            index_in_node,
            is_input: false,
            node: self.graph_input_node,
            type_,
            origin: None,
            targets: Vec::new(),
            default_value: None,
            index_in_graph: 0,
        });
        self.nodes[self.graph_input_node].outputs.push(si);
        self.nodes[self.graph_input_node].socket_names.push(name);
        si
    }
    /// `add_output` (verbatim — graph output =
    /// input socket on node 1).
    pub fn add_output(&mut self, type_: usize, name: String) -> usize {
        let si = self.sockets.len();
        let index_in_node = self.nodes[self.graph_output_node].inputs.len();
        self.sockets.push(Socket {
            index_in_node,
            is_input: true,
            node: self.graph_output_node,
            type_,
            origin: None,
            targets: Vec::new(),
            default_value: None,
            index_in_graph: 0,
        });
        self.nodes[self.graph_output_node].inputs.push(si);
        self.nodes[self.graph_output_node].socket_names.push(name);
        si
    }
    /// `add_link` (verbatim — origin + targets).
    pub fn add_link(&mut self, from: usize, to: usize) {
        debug_assert!(self.sockets[to].origin.is_none());
        debug_assert_eq!(self.sockets[from].type_, self.sockets[to].type_);
        self.sockets[to].origin = Some(from);
        self.sockets[from].targets.push(to);
    }
    /// `clear_origin` (verbatim — unlink both sides).
    pub fn clear_origin(&mut self, socket: usize) {
        if let Some(origin) = self.sockets[socket].origin.take() {
            let origin_targets = &mut self.sockets[origin].targets;
            if let Some(pos) = origin_targets.iter().position(|&t| t == socket) {
                origin_targets.remove(pos);
            }
        }
    }
    /// `update_node_indices` (verbatim).
    pub fn update_node_indices(&mut self) {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            node.index_in_graph = i;
        }
    }
    /// `update_socket_indices` (verbatim — sequential
    /// counter over all sockets).
    pub fn update_socket_indices(&mut self) {
        let mut counter = 0;
        for node in &self.nodes {
            for &s in node.inputs.iter().chain(node.outputs.iter()) {
                self.sockets[s].index_in_graph = counter;
                counter += 1;
            }
        }
        self.socket_num = counter;
    }
    /// `node_indices_are_valid` (verbatim).
    pub fn node_indices_are_valid(&self) -> bool {
        self.nodes.iter().enumerate().all(|(i, n)| n.index_in_graph == i)
    }
}
