//! Port of `task_graph.cc` — `TaskGraph`/`TaskNode` (tbb::flow::graph equivalent).
//! Serial scheduling like the non-TBB build; `push_work` runs depth-first.

/// `TaskGraphNodeRunFunction`.
pub type TaskGraphNodeRun = Box<dyn FnOnce(&mut usize) + Send>;

/// `TaskNode`.
pub struct TaskNode {
    pub successors: Vec<usize>,
    pub run_func: Option<TaskGraphNodeRun>,
    /// Opaque user data slot (replaces `void *task_data` + free_func).
    pub task_data: usize,
}

impl TaskNode {
    /// `run_serial` — run node then all successors depth-first.
    fn run_serial(&mut self, nodes: &mut [Option<TaskNode>], self_idx: usize) {
        if let Some(f) = self.run_func.take() {
            let mut data = self.task_data;
            f(&mut data);
            self.task_data = data;
        }
        let succ = self.successors.clone();
        for s in succ {
            if let Some(mut node) = nodes[s].take() {
                node.run_serial(nodes, s);
                nodes[s] = Some(node);
            }
        }
    }
}

/// `TaskGraph`.
pub struct TaskGraph {
    pub nodes: Vec<Option<TaskNode>>,
}

/// `BLI_task_graph_create`.
pub fn task_graph_create() -> TaskGraph {
    TaskGraph { nodes: Vec::new() }
}

/// `BLI_task_graph_node_create` — returns node index.
pub fn task_graph_node_create(
    graph: &mut TaskGraph,
    run: TaskGraphNodeRun,
    user_data: usize,
) -> usize {
    graph.nodes.push(Some(TaskNode {
        successors: Vec::new(),
        run_func: Some(run),
        task_data: user_data,
    }));
    graph.nodes.len() - 1
}

/// `BLI_task_graph_node_push_work` — run the node (serial path).
pub fn task_graph_node_push_work(graph: &mut TaskGraph, node_idx: usize) -> bool {
    if let Some(mut node) = graph.nodes[node_idx].take() {
        node.run_serial(&mut graph.nodes, node_idx);
        graph.nodes[node_idx] = Some(node);
    }
    true
}

/// `BLI_task_graph_edge_create` — `from` runs before `to`.
pub fn task_graph_edge_create(graph: &mut TaskGraph, from: usize, to: usize) {
    if let Some(n) = graph.nodes[from].as_mut() {
        n.successors.push(to);
    }
}

/// `BLI_task_graph_work_and_wait` — serial: nothing pending after push_work.
pub fn task_graph_work_and_wait(_graph: &mut TaskGraph) {}
