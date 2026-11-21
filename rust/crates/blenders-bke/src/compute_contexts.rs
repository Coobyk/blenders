//! Port of `intern/compute_contexts.cc` — the per-subsystem
//! `ComputeContext` types + `ComputeContextCache`.
//!
//! Polymorphic contexts → an arena `Vec<ComputeContextKind>`; parent
//! links are arena indices (`usize::MAX` = none). `ComputeContextHash`
//! comes from `blenders_bli::compute_context`; `::from(parent, tag, …)`
//! hashes `parent.hash ‖ tag ‖ args` (the C feeds XXH the same bytes).

use std::collections::HashMap;

use blenders_bli::compute_context::ComputeContextHash;

pub const NIL_CTX: usize = usize::MAX;

/// `nodes::ClosureSourceLocation` (NOD_geometry_nodes_closure_location.hh).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ClosureSourceLocation {
    /// `const bNodeTree *tree` — handle.
    pub tree: usize,
    /// `closure_output_node_id`.
    pub closure_output_node_id: i32,
}

fn hash_from(parent_hash: ComputeContextHash, tag: &str, args: &[&[u8]]) -> ComputeContextHash {
    let mut data = Vec::with_capacity(64);
    data.extend_from_slice(&parent_hash.v1.to_le_bytes());
    data.extend_from_slice(&parent_hash.v2.to_le_bytes());
    data.extend_from_slice(tag.as_bytes());
    for a in args {
        data.extend_from_slice(a);
    }
    ComputeContextHash::from_bytes(&data)
}

/// The `ComputeContext` subclasses, flattened to a tagged struct.
#[derive(Clone)]
pub enum ComputeContextKind {
    /// `DataBlockComputeContext` (`orig_session_uid_`, `id_`).
    DataBlock {
        orig_session_uid: u32,
        /// `const ID *id_` — handle (`usize::MAX` = null).
        id: usize,
    },
    /// `GeometryNodesModifierComputeContext`.
    GeometryNodesModifier {
        modifier_uid: i32,
        /// `const NodesModifierData *nmd_` — handle.
        nmd: usize,
    },
    /// `SceneCompositorEffectComputeContext` (`effect_.name`).
    SceneCompositorEffect { effect_name: String },
    /// `NodeComputeContext` / `GroupNodeComputeContext` (same hash tag
    /// "NODE" — GroupNodeComputeContext reuses the base).
    Node {
        node_id: i32,
        /// `const bNodeTree *tree_` — handle.
        tree: usize,
    },
    /// `SimulationZoneComputeContext`.
    SimulationZone { output_node_id: i32 },
    /// `RepeatZoneComputeContext`.
    RepeatZone { output_node_id: i32, iteration: i32 },
    /// `ForeachGeometryElementZoneComputeContext`.
    ForeachGeometryElementZone { output_node_id: i32, index: i32 },
    /// `EvaluateClosureComputeContext` (a Node context + location).
    EvaluateClosure {
        node_id: i32,
        tree: usize,
        closure_source_location: Option<ClosureSourceLocation>,
    },
    /// `ClosureToListComputeContext`.
    ClosureToList { node_id: i32, list_index: i32 },
    /// `OperatorComputeContext` (`tree_` optional).
    Operator { tree: usize },
    /// `ShaderComputeContext`.
    Shader { tree: usize },
}

/// Arena entry: parent link + kind + cached hash.
pub struct ComputeContextEntry {
    /// `parent_` — arena index or `NIL_CTX`.
    pub parent: usize,
    pub kind: ComputeContextKind,
    pub hash: ComputeContextHash,
}

/// `ComputeContextCache` — one map per `for_*` accessor, keyed like the
/// C `Map<std::pair<parent,key>, ComputeContext*>`.
#[derive(Default)]
pub struct ComputeContextCache {
    /// `contexts_` arena (all context objects).
    pub contexts: Vec<ComputeContextEntry>,
    data_block_contexts_cache: HashMap<(usize, u32), usize>,
    geometry_nodes_modifier_contexts_cache: HashMap<(usize, i32), usize>,
    scene_compositor_effect_contexts_cache: HashMap<(usize, String), usize>,
    group_node_contexts_cache: HashMap<(usize, i32), usize>,
    simulation_zone_contexts_cache: HashMap<(usize, i32), usize>,
    repeat_zone_contexts_cache: HashMap<(usize, i32, i32), usize>,
    foreach_geometry_element_zone_contexts_cache: HashMap<(usize, i32, i32), usize>,
    evaluate_closure_contexts_cache: HashMap<(usize, i32), usize>,
    operator_contexts_cache: HashMap<usize, usize>,
    shader_contexts_cache: HashMap<usize, usize>,
}

impl ComputeContextCache {
    fn parent_hash(&self, parent: usize) -> ComputeContextHash {
        if parent == NIL_CTX {
            ComputeContextHash::default()
        } else {
            self.contexts[parent].hash
        }
    }
    fn push(&mut self, parent: usize, kind: ComputeContextKind) -> usize {
        let ph = self.parent_hash(parent);
        let hash = match &kind {
            ComputeContextKind::DataBlock { orig_session_uid, .. } => {
                hash_from(ph, "DATABLOCK", &[&orig_session_uid.to_le_bytes()])
            }
            ComputeContextKind::GeometryNodesModifier { modifier_uid, .. } => {
                hash_from(ph, "MODIFIER", &[&modifier_uid.to_le_bytes()])
            }
            ComputeContextKind::SceneCompositorEffect { effect_name } => {
                hash_from(ph, "SCENE_COMPOSITOR_EFFECT", &[effect_name.as_bytes()])
            }
            ComputeContextKind::Node { node_id, .. } => {
                hash_from(ph, "NODE", &[&node_id.to_le_bytes()])
            }
            ComputeContextKind::SimulationZone { output_node_id } => {
                hash_from(ph, "SIM_ZONE", &[&output_node_id.to_le_bytes()])
            }
            ComputeContextKind::RepeatZone { output_node_id, iteration } => hash_from(
                ph,
                "REPEAT_ZONE",
                &[&output_node_id.to_le_bytes(), &iteration.to_le_bytes()],
            ),
            ComputeContextKind::ForeachGeometryElementZone { output_node_id, index } => hash_from(
                ph,
                "FOREACH_GEOMETRY_ELEMENT",
                &[&output_node_id.to_le_bytes(), &index.to_le_bytes()],
            ),
            ComputeContextKind::EvaluateClosure { node_id, .. } => {
                hash_from(ph, "NODE", &[&node_id.to_le_bytes()])
            }
            ComputeContextKind::ClosureToList { node_id, list_index } => hash_from(
                ph,
                "CLOSURE_TO_LIST",
                &[&node_id.to_le_bytes(), &list_index.to_le_bytes()],
            ),
            ComputeContextKind::Operator { .. } => hash_from(ph, "OPERATOR", &[]),
            ComputeContextKind::Shader { .. } => hash_from(ph, "SHADER", &[]),
        };
        self.contexts.push(ComputeContextEntry { parent, kind, hash });
        self.contexts.len() - 1
    }

    /// `for_data_block(parent, orig_session_uid, id)`.
    pub fn for_data_block(&mut self, parent: usize, orig_session_uid: u32, id: usize) -> usize {
        if let Some(&i) = self.data_block_contexts_cache.get(&(parent, orig_session_uid)) {
            return i;
        }
        let i = self.push(
            parent,
            ComputeContextKind::DataBlock { orig_session_uid, id },
        );
        self.data_block_contexts_cache.insert((parent, orig_session_uid), i);
        i
    }

    /// `for_geometry_nodes_modifier(parent, nmd | modifier_uid)`.
    pub fn for_geometry_nodes_modifier(
        &mut self,
        parent: usize,
        modifier_uid: i32,
        nmd: usize,
    ) -> usize {
        if let Some(&i) = self
            .geometry_nodes_modifier_contexts_cache
            .get(&(parent, modifier_uid))
        {
            return i;
        }
        let i = self.push(
            parent,
            ComputeContextKind::GeometryNodesModifier { modifier_uid, nmd },
        );
        self.geometry_nodes_modifier_contexts_cache
            .insert((parent, modifier_uid), i);
        i
    }

    /// `for_scene_compositor_effect`.
    pub fn for_scene_compositor_effect(&mut self, parent: usize, effect_name: &str) -> usize {
        if let Some(&i) = self
            .scene_compositor_effect_contexts_cache
            .get(&(parent, effect_name.to_string()))
        {
            return i;
        }
        let i = self.push(
            parent,
            ComputeContextKind::SceneCompositorEffect {
                effect_name: effect_name.to_string(),
            },
        );
        self.scene_compositor_effect_contexts_cache
            .insert((parent, effect_name.to_string()), i);
        i
    }

    /// `for_operator` (with or without `tree`).
    pub fn for_operator(&mut self, parent: usize, tree: usize) -> usize {
        if let Some(&i) = self.operator_contexts_cache.get(&parent) {
            return i;
        }
        let i = self.push(parent, ComputeContextKind::Operator { tree });
        self.operator_contexts_cache.insert(parent, i);
        i
    }

    /// `for_shader`.
    pub fn for_shader(&mut self, parent: usize, tree: usize) -> usize {
        if let Some(&i) = self.shader_contexts_cache.get(&parent) {
            return i;
        }
        let i = self.push(parent, ComputeContextKind::Shader { tree });
        self.shader_contexts_cache.insert(parent, i);
        i
    }

    /// `for_group_node` — `GroupNodeComputeContext` reuses the
    /// `NodeComputeContext` shape + "NODE" hash tag.
    pub fn for_group_node(&mut self, parent: usize, node_id: i32, tree: usize) -> usize {
        if let Some(&i) = self.group_node_contexts_cache.get(&(parent, node_id)) {
            return i;
        }
        let i = self.push(parent, ComputeContextKind::Node { node_id, tree });
        self.group_node_contexts_cache.insert((parent, node_id), i);
        i
    }

    /// `for_simulation_zone`.
    pub fn for_simulation_zone(&mut self, parent: usize, output_node_id: i32) -> usize {
        if let Some(&i) = self
            .simulation_zone_contexts_cache
            .get(&(parent, output_node_id))
        {
            return i;
        }
        let i = self.push(parent, ComputeContextKind::SimulationZone { output_node_id });
        self.simulation_zone_contexts_cache.insert((parent, output_node_id), i);
        i
    }

    /// `for_repeat_zone`.
    pub fn for_repeat_zone(&mut self, parent: usize, output_node_id: i32, iteration: i32) -> usize {
        if let Some(&i) = self
            .repeat_zone_contexts_cache
            .get(&(parent, output_node_id, iteration))
        {
            return i;
        }
        let i = self.push(
            parent,
            ComputeContextKind::RepeatZone { output_node_id, iteration },
        );
        self.repeat_zone_contexts_cache
            .insert((parent, output_node_id, iteration), i);
        i
    }

    /// `for_foreach_geometry_element_zone`.
    pub fn for_foreach_geometry_element_zone(
        &mut self,
        parent: usize,
        output_node_id: i32,
        index: i32,
    ) -> usize {
        if let Some(&i) = self
            .foreach_geometry_element_zone_contexts_cache
            .get(&(parent, output_node_id, index))
        {
            return i;
        }
        let i = self.push(
            parent,
            ComputeContextKind::ForeachGeometryElementZone { output_node_id, index },
        );
        self.foreach_geometry_element_zone_contexts_cache
            .insert((parent, output_node_id, index), i);
        i
    }

    /// `for_evaluate_closure`.
    pub fn for_evaluate_closure(
        &mut self,
        parent: usize,
        node_id: i32,
        tree: usize,
        closure_source_location: Option<ClosureSourceLocation>,
    ) -> usize {
        if let Some(&i) = self.evaluate_closure_contexts_cache.get(&(parent, node_id)) {
            return i;
        }
        let i = self.push(
            parent,
            ComputeContextKind::EvaluateClosure { node_id, tree, closure_source_location },
        );
        self.evaluate_closure_contexts_cache.insert((parent, node_id), i);
        i
    }
}

/// `EvaluateClosureComputeContext::is_recursive` — walks the parent
/// chain for a matching location.
pub fn evaluate_closure_is_recursive(cache: &ComputeContextCache, ctx: usize) -> bool {
    let ComputeContextKind::EvaluateClosure { closure_source_location, .. } =
        &cache.contexts[ctx].kind
    else {
        return false;
    };
    let Some(loc) = closure_source_location else { return false };
    let mut parent = cache.contexts[ctx].parent;
    while parent != NIL_CTX {
        if let ComputeContextKind::EvaluateClosure {
            closure_source_location: Some(ploc),
            ..
        } = &cache.contexts[parent].kind
        {
            if ploc.tree == loc.tree && ploc.closure_output_node_id == loc.closure_output_node_id {
                return true;
            }
        }
        parent = cache.contexts[parent].parent;
    }
    false
}

/// `print_current_in_line` — the per-kind label; `node_label` and the
/// modifier/ID names are injected text (PORT_PENDING accessors).
pub fn print_current_in_line(
    cache: &ComputeContextCache,
    ctx: usize,
    modifier_name: &dyn Fn(usize) -> String,
    id_name: &dyn Fn(usize) -> String,
    node_label: &dyn Fn(usize, i32) -> String,
    tree_id_name: &dyn Fn(usize) -> String,
) -> String {
    match &cache.contexts[ctx].kind {
        ComputeContextKind::DataBlock { orig_session_uid, id } => {
            let mut s = format!("DataBlock: {}", orig_session_uid);
            if *id != usize::MAX {
                s += &format!(" {}", id_name(*id));
            }
            s
        }
        ComputeContextKind::GeometryNodesModifier { nmd, .. } => {
            if *nmd != usize::MAX {
                format!("Modifier: {}", modifier_name(*nmd))
            } else {
                String::new()
            }
        }
        ComputeContextKind::SceneCompositorEffect { effect_name } => {
            format!("Scene Compositor Effect: {}", effect_name)
        }
        ComputeContextKind::Node { node_id, tree } => {
            if *tree != usize::MAX {
                let l = node_label(*tree, *node_id);
                if !l.is_empty() {
                    return format!("Node: {}", l);
                }
            }
            format!("Node ID: {}", node_id)
        }
        ComputeContextKind::SimulationZone { output_node_id } => {
            format!("Simulation Zone ID: {}", output_node_id)
        }
        ComputeContextKind::RepeatZone { output_node_id, iteration } => {
            format!("Repeat Zone ID: {}, Iteration: {}", output_node_id, iteration)
        }
        ComputeContextKind::ForeachGeometryElementZone { output_node_id, .. } => {
            format!("Foreach Geometry Element Zone ID: {}", output_node_id)
        }
        ComputeContextKind::EvaluateClosure { node_id, tree, .. } => {
            if *tree != usize::MAX {
                let l = node_label(*tree, *node_id);
                if !l.is_empty() {
                    return format!("Node: {}", l);
                }
            }
            format!("Node ID: {}", node_id)
        }
        ComputeContextKind::ClosureToList { node_id, list_index } => {
            format!("Closure to List ID: {}, List Index: {}", node_id, list_index)
        }
        ComputeContextKind::Operator { .. } => "Operator".to_string(),
        ComputeContextKind::Shader { tree } => {
            let mut s = "Shader ".to_string();
            if *tree != usize::MAX {
                s += &tree_id_name(*tree);
            }
            s
        }
    }
}
