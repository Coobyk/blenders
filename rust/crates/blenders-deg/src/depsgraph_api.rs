//! Port of `intern/depsgraph.cc` tail — DEG_graph_*
/// lifecycle + active/update-count API.

use crate::depsgraph::Depsgraph;
use crate::registry::{register_graph, unregister_graph};

/// `DEG_graph_new` (verbatim — constructor + registry).
pub fn deg_graph_new(
    bmain: usize,
    scene: usize,
    view_layer: usize,
    mode: i32,
    graph_id: usize,
) -> Depsgraph {
    let g = Depsgraph {
        bmain,
        scene,
        view_layer,
        mode,
        is_active: true,
        use_visibility_optimization: true,
        ..Default::default()
    };
    register_graph(bmain, graph_id);
    g
}

/// `DEG_graph_replace_owners` (verbatim — re-registers on
/// bmain change).
pub fn deg_graph_replace_owners(
    graph: &mut Depsgraph,
    graph_id: usize,
    bmain: usize,
    scene: usize,
    view_layer: usize,
) {
    let do_update_register = graph.bmain != bmain;
    if do_update_register && graph.bmain != 0 {
        unregister_graph(graph.bmain, graph_id);
    }
    graph.bmain = bmain;
    graph.scene = scene;
    graph.view_layer = view_layer;
    if do_update_register {
        register_graph(bmain, graph_id);
    }
}

/// `DEG_graph_free` (verbatim — the drop of the graph
/// itself is handled by Rust ownership).
pub fn deg_graph_free(graph: &Depsgraph, graph_id: usize) {
    unregister_graph(graph.bmain, graph_id);
}

/// `DEG_is_evaluating` (verbatim).
pub fn deg_is_evaluating(graph: &Depsgraph) -> bool {
    graph.is_evaluating
}

/// `DEG_is_active` (verbatim).
pub fn deg_is_active(graph: Option<&Depsgraph>) -> bool {
    graph.map_or(false, |g| g.is_active)
}

/// `DEG_make_active` (verbatim).
pub fn deg_make_active(graph: &mut Depsgraph) {
    graph.is_active = true;
}

/// `DEG_make_inactive` (verbatim).
pub fn deg_make_inactive(graph: &mut Depsgraph) {
    graph.is_active = false;
}

/// `DEG_disable_visibility_optimization` (verbatim).
pub fn deg_disable_visibility_optimization(graph: &mut Depsgraph) {
    graph.use_visibility_optimization = false;
}

/// `DEG_get_update_count` (verbatim).
pub fn deg_get_update_count(graph: &Depsgraph) -> u64 {
    graph.update_count
}
