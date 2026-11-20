//! Port of `astar.cc` — `BLI_AStarGraph`/`BLI_AStarSolution` A* search.

use std::collections::BinaryHeap;

/// `BLI_AStarGNLink`.
#[derive(Clone, Copy)]
pub struct AStarGNLink<T> {
    pub nodes: [i32; 2],
    pub cost: f32,
    pub custom_data: T,
}

/// `BLI_AStarGNode`.
pub struct AStarGNode<N> {
    pub neighbor_links: Vec<usize>, /* link indices into graph.links */
    pub custom_data: N,
}

/// `BLI_AStarGraph`.
pub struct AStarGraph<N, T> {
    pub node_num: usize,
    pub nodes: Vec<AStarGNode<N>>,
    pub links: Vec<AStarGNLink<T>>,
    pub custom_data: Option<usize>, /* opaque */
}

/// `BLI_AStarSolution`.
pub struct AStarSolution {
    pub steps: i32,
    pub prev_nodes: Vec<i32>,
    pub prev_links: Vec<i32>, /* link indices, -1 none */
    pub done_nodes: Vec<u32>, /* bitmap */
    pub g_costs: Vec<f32>,
    pub g_steps: Vec<i32>,
    pub custom_data: Option<usize>,
}

/// `astar_f_cost` callback type.
pub type AStarFCost<N, T> = fn(&AStarGraph<N, T>, &AStarSolution, Option<&AStarGNLink<T>>, i32, i32, i32) -> f32;

/// `BLI_astar_node_init`.
pub fn astar_node_init<N: Default, T>(graph: &mut AStarGraph<N, T>, node_index: usize, custom_data: N) {
    graph.nodes[node_index].custom_data = custom_data;
}

/// `BLI_astar_node_link_add`.
pub fn astar_node_link_add<N, T>(
    graph: &mut AStarGraph<N, T>,
    node1: usize,
    node2: usize,
    cost: f32,
    custom_data: T,
) {
    let link = AStarGNLink {
        nodes: [node1 as i32, node2 as i32],
        cost,
        custom_data,
    };
    let idx = graph.links.len();
    graph.links.push(link);
    graph.nodes[node1].neighbor_links.push(idx);
    graph.nodes[node2].neighbor_links.push(idx);
}

/// `BLI_astar_node_link_other_node`.
pub fn astar_node_link_other_node<T>(lnk: &AStarGNLink<T>, idx: i32) -> i32 {
    if lnk.nodes[0] == idx {
        lnk.nodes[1]
    } else {
        lnk.nodes[0]
    }
}

/// `BLI_astar_graph_init`.
pub fn astar_graph_init<N: Default, T>(node_num: usize, custom_data: Option<usize>) -> AStarGraph<N, T> {
    AStarGraph {
        node_num,
        nodes: (0..node_num)
            .map(|_| AStarGNode {
                neighbor_links: Vec::new(),
                custom_data: N::default(),
            })
            .collect(),
        links: Vec::new(),
        custom_data,
    }
}

/// `BLI_astar_solution_init`.
pub fn astar_solution_init(node_num: usize, custom_data: Option<usize>) -> AStarSolution {
    AStarSolution {
        steps: 0,
        prev_nodes: vec![0; node_num],
        prev_links: vec![0; node_num],
        done_nodes: crate::bitmap::bitmap_new(node_num),
        g_costs: vec![0.0; node_num],
        g_steps: vec![0; node_num],
        custom_data,
    }
}

struct HeapEntry(f32, i32);
impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for HeapEntry {}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // min-heap
        other
            .0
            .partial_cmp(&self.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(other.1.cmp(&self.1))
    }
}

/// `BLI_astar_graph_solve`.
pub fn astar_graph_solve<N, T>(
    graph: &AStarGraph<N, T>,
    node_index_src: i32,
    node_index_dst: i32,
    f_cost_cb: AStarFCost<N, T>,
    r_solution: &mut AStarSolution,
    max_steps: i32,
) -> bool {
    use crate::bitmap;
    let done_nodes = &mut r_solution.done_nodes;
    r_solution.steps = 0;
    r_solution.prev_nodes[node_index_src as usize] = -1;
    bitmap::bitmap_set_all(done_nodes, false, graph.node_num);
    r_solution.g_costs.fill(f32::MAX);
    r_solution.g_costs[node_index_src as usize] = 0.0;
    r_solution.g_steps[node_index_src as usize] = 0;
    if node_index_src == node_index_dst {
        return true;
    }
    let mut todo_nodes: BinaryHeap<HeapEntry> = BinaryHeap::new();
    let f = f_cost_cb(graph, r_solution, None, -1, node_index_src, node_index_dst);
    todo_nodes.push(HeapEntry(f, node_index_src));
    while let Some(HeapEntry(_, node_curr_idx)) = todo_nodes.pop() {
        if bitmap::bitmap_get(done_nodes, node_curr_idx as usize) {
            continue;
        }
        if max_steps != 0 && r_solution.g_steps[node_curr_idx as usize] > max_steps {
            continue;
        }
        if node_curr_idx == node_index_dst {
            r_solution.steps = r_solution.g_steps[node_curr_idx as usize] + 1;
            return true;
        }
        bitmap::bitmap_set(done_nodes, node_curr_idx as usize);
        let node_curr = &graph.nodes[node_curr_idx as usize];
        for &link_idx in &node_curr.neighbor_links {
            let link = &graph.links[link_idx];
            let node_next_idx = astar_node_link_other_node(link, node_curr_idx);
            if !bitmap::bitmap_get(done_nodes, node_next_idx as usize) {
                let g_cst = r_solution.g_costs[node_curr_idx as usize] + link.cost;
                if g_cst < r_solution.g_costs[node_next_idx as usize] {
                    r_solution.prev_nodes[node_next_idx as usize] = node_curr_idx;
                    r_solution.prev_links[node_next_idx as usize] = link_idx as i32;
                    r_solution.g_costs[node_next_idx as usize] = g_cst;
                    r_solution.g_steps[node_next_idx as usize] =
                        r_solution.g_steps[node_curr_idx as usize] + 1;
                    let f = f_cost_cb(
                        graph,
                        r_solution,
                        Some(link),
                        node_curr_idx,
                        node_next_idx,
                        node_index_dst,
                    );
                    todo_nodes.push(HeapEntry(f, node_next_idx));
                }
            }
        }
    }
    false
}
