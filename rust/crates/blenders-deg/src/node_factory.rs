//! Port of `intern/node/deg_node_factory.cc` — global type
/// registry `register_node_typeinfo`/`type_get_factory`.

use std::collections::HashMap;
use std::sync::Mutex;

/// `NodeType` id (registry index).
pub type NodeType = usize;

/// `DepsNodeFactory` — boxed factory for a node type.
pub type DepsNodeFactory = Box<dyn Send + Sync>;

static NODE_TYPEINFO_REGISTRY: Mutex<Option<HashMap<NodeType, DepsNodeFactory>>> =
    Mutex::new(None);

/// `register_node_typeinfo`.
pub fn register_node_typeinfo(node_type: NodeType, factory: DepsNodeFactory) {
    NODE_TYPEINFO_REGISTRY
        .lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(node_type, factory);
}

/// `type_get_factory` — `None` when not registered.
pub fn type_get_factory(node_type: NodeType) -> bool {
    NODE_TYPEINFO_REGISTRY
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|m| m.contains_key(&node_type))
}
