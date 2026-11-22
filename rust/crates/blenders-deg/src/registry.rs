//! Port of `intern/depsgraph_registry.cc` — `GraphRegistry`
/// (`Main` → ordered graph set).

use std::collections::HashMap;
use std::sync::Mutex;

/// `GraphRegistry` — `Main*` → insertion-ordered `VectorSet` of
/// depsgraphs. Indices are `Main`/graph arena handles.
#[derive(Default)]
pub struct GraphRegistry {
    /// `map` — insertion order preserved via `Vec`.
    pub map: HashMap<usize, Vec<usize>>,
}

static GRAPH_REGISTRY: Mutex<Option<GraphRegistry>> = Mutex::new(None);

fn with_registry<R>(f: impl FnOnce(&mut GraphRegistry) -> R) -> R {
    let mut g = GRAPH_REGISTRY.lock().unwrap();
    f(g.get_or_insert_with(GraphRegistry::default))
}

/// `register_graph` (verbatim — `add_new` dedups).
pub fn register_graph(bmain: usize, depsgraph: usize) {
    with_registry(|reg| {
        let graphs = reg.map.entry(bmain).or_default();
        if !graphs.contains(&depsgraph) {
            graphs.push(depsgraph);
        }
    });
}

/// `unregister_graph` (verbatim — removes the `bmain` entry
/// when its set goes empty).
pub fn unregister_graph(bmain: usize, depsgraph: usize) {
    with_registry(|reg| {
        if let Some(graphs) = reg.map.get_mut(&bmain) {
            graphs.retain(|&g| g != depsgraph);
            if graphs.is_empty() {
                reg.map.remove(&bmain);
            }
        }
    });
}

/// `get_all_registered_graphs` (verbatim).
pub fn get_all_registered_graphs(bmain: usize) -> Vec<usize> {
    with_registry(|reg| reg.map.get(&bmain).cloned().unwrap_or_default())
}
