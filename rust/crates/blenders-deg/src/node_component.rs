//! Port of `intern/node/deg_node_component.cc` —
/// `ComponentNode` + `OperationIDKey` + component registry.

use crate::node_operation::{OperationCode, OperationNode};
use std::collections::HashMap;

/// `ComponentNode::OperationIDKey` (verbatim).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OperationIdKey {
    /// `opcode`.
    pub opcode: OperationCode,
    /// `name`.
    pub name: String,
    /// `name_tag`.
    pub name_tag: i32,
}

impl OperationIdKey {
    /// `identifier` (verbatim).
    pub fn identifier(&self) -> String {
        format!("OperationIDKey({}, {})", self.opcode as i32, self.name)
    }
}

/// `ComponentNode` (verbatim — operation ids into a
/// shared arena).
#[derive(Default)]
pub struct ComponentNode {
    /// `type` (`NodeType` index).
    pub node_type: usize,
    /// `name`.
    pub name: String,
    /// `entry_operation`.
    pub entry_operation: Option<usize>,
    /// `exit_operation`.
    pub exit_operation: Option<usize>,
    /// `possibly_affects_visible_id`.
    pub possibly_affects_visible_id: bool,
    /// `affects_visible_id`.
    pub affects_visible_id: bool,
    /// `is_user_modified`.
    pub is_user_modified: bool,
    /// `operations_map` (pre-finalize).
    pub operations_map: Option<HashMap<OperationIdKey, usize>>,
    /// `operations` (post-finalize flat list).
    pub operations: Vec<usize>,
}

impl ComponentNode {
    /// ctor (verbatim — `operations_map` allocated).
    pub fn new() -> Self {
        Self {
            operations_map: Some(HashMap::new()),
            ..Default::default()
        }
    }

    /// `identifier` (verbatim — `type_name` injected).
    pub fn identifier(&self, type_name: &str) -> String {
        let name_part = if self.name.is_empty() {
            String::new()
        } else {
            format!(" '{}'", self.name)
        };
        format!(
            "[{}]{} : (affects_visible_id: {})",
            type_name, name_part, self.affects_visible_id
        )
    }

    /// `find_operation` (verbatim — `ops` supplies
    /// `(opcode, name, name_tag)` per op id).
    pub fn find_operation(
        &self,
        key: &OperationIdKey,
        ops: &dyn Fn(usize) -> (OperationCode, String, i32),
    ) -> Option<usize> {
        if let Some(map) = &self.operations_map {
            return map.get(key).copied();
        }
        for &op in &self.operations {
            let (opcode, name, name_tag) = ops(op);
            if opcode == key.opcode && name_tag == key.name_tag && name == key.name {
                return Some(op);
            }
        }
        None
    }

    /// `has_operation` (verbatim).
    pub fn has_operation(
        &self,
        key: &OperationIdKey,
        ops: &dyn Fn(usize) -> (OperationCode, String, i32),
    ) -> bool {
        self.find_operation(key, ops).is_some()
    }

    /// `add_operation` (verbatim — `create_node`
    /// injects `OperationNode` arena alloc).
    pub fn add_operation(
        &mut self,
        opcode: OperationCode,
        name: &str,
        name_tag: i32,
        create_node: &mut dyn FnMut(&str) -> usize,
        set_fields: &mut dyn FnMut(usize, OperationCode, &str, i32),
    ) -> usize {
        let key = OperationIdKey {
            opcode,
            name: name.to_string(),
            name_tag,
        };
        let op_node = create_node(name);
        if let Some(map) = &mut self.operations_map {
            map.insert(key, op_node);
        }
        set_fields(op_node, opcode, name, name_tag);
        op_node
    }

    /// `set_entry_operation` (verbatim).
    pub fn set_entry_operation(&mut self, op_node: usize) {
        debug_assert!(self.entry_operation.is_none());
        self.entry_operation = Some(op_node);
    }

    /// `set_exit_operation` (verbatim).
    pub fn set_exit_operation(&mut self, op_node: usize) {
        debug_assert!(self.exit_operation.is_none());
        self.exit_operation = Some(op_node);
    }

    /// `clear_operations` (verbatim — ids dropped).
    pub fn clear_operations(&mut self) {
        if let Some(map) = &mut self.operations_map {
            map.clear();
        }
        self.operations.clear();
    }

    /// `tag_update` (verbatim — per-op tag).
    pub fn tag_update(&self, source: i32, tag: &mut dyn FnMut(usize, i32)) {
        for &op in &self.operations {
            tag(op, source);
        }
        if let Some(map) = &self.operations_map {
            for &op in map.values() {
                tag(op, source);
            }
        }
    }

    /// `get_entry_operation` (verbatim).
    pub fn get_entry_operation(&mut self) -> Option<usize> {
        if self.entry_operation.is_some() {
            return self.entry_operation;
        }
        if let Some(map) = &self.operations_map {
            if map.len() == 1 {
                let op = *map.values().next().unwrap();
                self.entry_operation = Some(op);
                return Some(op);
            }
        }
        if self.operations.len() == 1 {
            return Some(self.operations[0]);
        }
        None
    }

    /// `get_exit_operation` (verbatim).
    pub fn get_exit_operation(&mut self) -> Option<usize> {
        if self.exit_operation.is_some() {
            return self.exit_operation;
        }
        if let Some(map) = &self.operations_map {
            if map.len() == 1 {
                let op = *map.values().next().unwrap();
                self.exit_operation = Some(op);
                return Some(op);
            }
        }
        if self.operations.len() == 1 {
            return Some(self.operations[0]);
        }
        None
    }

    /// `finalize_build` (verbatim — map → flat list).
    pub fn finalize_build(&mut self) {
        if let Some(map) = self.operations_map.take() {
            self.operations.reserve(map.len());
            for op in map.into_values() {
                self.operations.push(op);
            }
        }
    }
}

/// `deg_register_component_depsnodes` — the verbatim
/// `DEG_COMPONENT_NODE_DEFINE` table (type, id_recalc_tag).
/// `ID_RECALC_*` consts are the `DNA_ID.h` recalc bits.
pub fn deg_component_node_table() -> &'static [(&'static str, &'static str, u64)] {
    &[
        ("Animation", "ANIMATION", 1 << 6),            /* ID_RECALC_ANIMATION */
        ("BatchCache", "BATCH_CACHE", 1 << 2),         /* ID_RECALC_SHADING */
        ("Bone", "BONE", 1 << 1),                      /* ID_RECALC_GEOMETRY */
        ("Cache", "CACHE", 0),
        ("CopyOnWrite", "COPY_ON_EVAL", 1 << 7),       /* ID_RECALC_SYNC_TO_EVAL */
        ("ImageAnimation", "IMAGE_ANIMATION", 0),
        ("Geometry", "GEOMETRY", 1 << 1),
        ("LayerCollections", "LAYER_COLLECTIONS", 0),
        ("Parameters", "PARAMETERS", 1 << 5),          /* ID_RECALC_PARAMETERS */
        ("Particles", "PARTICLE_SYSTEM", 1 << 1),
    ]
}
