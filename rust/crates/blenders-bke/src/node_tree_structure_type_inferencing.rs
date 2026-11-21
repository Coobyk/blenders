//! Port of `intern/node_tree_structure_type_inferencing.cc` —
//! `DataRequirement` merge lattice, `StructureType` propagation
//! (right-to-left requirements, left-to-right status, zone
//! input/output pairing) as a fixed-point iteration.
//! The node tree is injected via [`TreeModel`].

/// `nodes::StructureType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StructureType {
    /// `Dynamic`.
    Dynamic,
    /// `Single`.
    Single,
    /// `Field`.
    Field,
    /// `Grid`.
    Grid,
    /// `List`.
    List,
    /// `Auto` (interface declaration only).
    Auto,
}

/// `NodeSocketInterfaceStructureType` values for interface items.
pub const NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO: i32 = 0;
/// Single.
pub const NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_SINGLE: i32 = 1;
/// Field.
pub const NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_DYNAMIC: i32 = 2;

/// `DataRequirement` enum (verbatim).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataRequirement {
    /// None.
    None,
    /// Field.
    Field,
    /// Single.
    Single,
    /// Grid.
    Grid,
    /// List.
    List,
    /// Invalid merge.
    Invalid,
}

/// `merge` — DataRequirement lattice join.
pub fn merge(a: DataRequirement, b: DataRequirement) -> DataRequirement {
    if a == b {
        return a;
    }
    if a == DataRequirement::None {
        return b;
    }
    if b == DataRequirement::None {
        return a;
    }
    if (a == DataRequirement::Field && b == DataRequirement::Single)
        || (a == DataRequirement::Single && b == DataRequirement::Field)
    {
        return DataRequirement::Single;
    }
    DataRequirement::Invalid
}

/// `data_requirement_to_auto_structure_type`.
pub fn data_requirement_to_auto_structure_type(r: DataRequirement) -> StructureType {
    match r {
        DataRequirement::None => StructureType::Dynamic,
        DataRequirement::Field => StructureType::Field,
        DataRequirement::Single => StructureType::Single,
        DataRequirement::Grid => StructureType::Grid,
        DataRequirement::List => StructureType::List,
        DataRequirement::Invalid => StructureType::Dynamic,
    }
}

/// `left_to_right_merge` — StructureType join for L2R propagation.
pub fn left_to_right_merge(a: StructureType, b: StructureType) -> StructureType {
    if a == b {
        return a;
    }
    if a == StructureType::Dynamic || b == StructureType::Dynamic {
        return StructureType::Dynamic;
    }
    use StructureType::*;
    match (a, b) {
        (Field, Grid) | (Grid, Field) => Grid,
        (Single, Field) | (Field, Single) => Field,
        (Single, Grid) | (Grid, Single) => Grid,
        (Single, List) | (List, Single) => List,
        (Field, List) | (List, Field) => List,
        _ => a,
    }
}

/// `ZoneInOutChange` flags.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ZoneInOutChange(pub u8);

/// `ZoneInOutChange::None`.
pub const ZONE_CHANGE_NONE: ZoneInOutChange = ZoneInOutChange(0);
/// `ZoneInOutChange::In`.
pub const ZONE_CHANGE_IN: ZoneInOutChange = ZoneInOutChange(1 << 1);
/// `ZoneInOutChange::Out`.
pub const ZONE_CHANGE_OUT: ZoneInOutChange = ZoneInOutChange(1 << 2);

impl ZoneInOutChange {
    /// Bit-or.
    pub fn with(self, other: ZoneInOutChange) -> ZoneInOutChange {
        ZoneInOutChange(self.0 | other.0)
    }
    /// Flag test.
    pub fn has(self, other: ZoneInOutChange) -> bool {
        self.0 & other.0 != 0
    }
}

/// Which socket kinds a node type supports.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct SupportedTypes {
    /// single supported.
    pub single: bool,
    /// field supported.
    pub field: bool,
    /// grid supported.
    pub grid: bool,
    /// list supported.
    pub list: bool,
}

/// `ensure_supported_structure_type(socket_type, type, supported)`.
/// `supported` per-socket-type allowed set; drops to Dynamic/fallbacks.
pub fn ensure_supported_structure_type(
    st: StructureType,
    supported: &SupportedTypes,
) -> StructureType {
    let ok = match st {
        StructureType::Single => supported.single,
        StructureType::Field => supported.field,
        StructureType::Grid => supported.grid,
        StructureType::List => supported.list,
        StructureType::Dynamic | StructureType::Auto => true,
    };
    if ok {
        st
    } else {
        StructureType::Dynamic
    }
}

/// Socket info needed by the inferencing.
#[derive(Clone)]
pub struct SocketModel {
    /// `index_in_tree` — unique socket index over all sockets.
    pub index_in_tree: usize,
    /// `index_in_all_inputs` — index over all input sockets.
    pub index_in_all_inputs: usize,
    /// is input (vs output).
    pub is_input: bool,
    /// `is_available`.
    pub available: bool,
    /// declaration structure type (None = no declaration).
    pub decl_structure_type: Option<StructureType>,
    /// supported types for `ensure_supported_structure_type`.
    pub supported: SupportedTypes,
    /// directly linked input socket tree-indices (for outputs).
    pub linked_inputs: Vec<usize>,
    /// directly linked output socket tree-index (for inputs).
    pub linked_output: Option<usize>,
    /// default_input_is_field (for unconnected inputs).
    pub default_input_is_field: bool,
}

/// Node info needed by the inferencing.
#[derive(Clone)]
pub struct NodeModel {
    /// `identifier`.
    pub identifier: i32,
    /// type tag (NODE_*).
    pub type_tag: i32,
    /// input socket indices (in socket array order).
    pub inputs: Vec<usize>,
    /// output socket indices.
    pub outputs: Vec<usize>,
    /// zone partner node identifier (simulation/repeat input→output).
    pub output_node_id: Option<i32>,
}

/// Zone type tags used in propagation.
pub const GEO_NODE_SIMULATION_INPUT: i32 = 10001;
/// See above.
pub const GEO_NODE_SIMULATION_OUTPUT: i32 = 10002;
/// See above.
pub const GEO_NODE_REPEAT_INPUT: i32 = 10003;
/// See above.
pub const GEO_NODE_REPEAT_OUTPUT: i32 = 10004;

/// The tree data the algorithm reads.
#[derive(Clone, Default)]
pub struct TreeModel {
    /// all sockets by `index_in_tree`.
    pub sockets: Vec<SocketModel>,
    /// all nodes.
    pub nodes: Vec<NodeModel>,
    /// interface input structure types (NodeSocketInterfaceStructureType).
    pub interface_inputs: Vec<i32>,
    /// interface output structure types.
    pub interface_outputs: Vec<i32>,
    /// group input node indices.
    pub group_input_nodes: Vec<usize>,
    /// group output node index.
    pub group_output_node: Option<usize>,
}

impl TreeModel {
    fn node_by_id(&self, id: i32) -> Option<&NodeModel> {
        self.nodes.iter().find(|n| n.identifier == id)
    }
    fn nodes_of_type(&self, t: i32) -> impl Iterator<Item = &NodeModel> {
        self.nodes.iter().filter(move |n| n.type_tag == t)
    }
    /// input→all_inputs index map lookup.
    pub fn input_req_index(&self, socket_tree_index: usize) -> usize {
        self.sockets[socket_tree_index].index_in_all_inputs
    }
}

/// `init_input_requirements`.
pub fn init_input_requirements(
    tree: &TreeModel,
    is_auto: &[bool],
    input_requirements: &mut [DataRequirement],
) {
    for node in &tree.nodes {
        for &si in &node.inputs {
            let socket = &tree.sockets[si];
            let req = &mut input_requirements[socket.index_in_all_inputs];
            if is_auto[socket.index_in_tree] {
                *req = DataRequirement::None;
                continue;
            }
            match socket.decl_structure_type {
                None | Some(StructureType::Dynamic) | Some(StructureType::Auto) => {
                    *req = DataRequirement::None;
                }
                Some(StructureType::Single) => *req = DataRequirement::Single,
                Some(StructureType::Grid) => *req = DataRequirement::Grid,
                Some(StructureType::Field) => *req = DataRequirement::Field,
                Some(StructureType::List) => *req = DataRequirement::List,
            }
        }
    }
}

/// `calc_output_socket_requirement`.
pub fn calc_output_socket_requirement(
    tree: &TreeModel,
    output_socket: usize,
    input_requirements: &[DataRequirement],
) -> DataRequirement {
    let mut requirement = DataRequirement::None;
    let out = &tree.sockets[output_socket];
    if !out.available {
        return requirement;
    }
    for &linked in &out.linked_inputs {
        let s = &tree.sockets[linked];
        if !s.available {
            continue;
        }
        requirement = merge(requirement, input_requirements[s.index_in_all_inputs]);
    }
    requirement
}

/// `store_group_input_structure_types` — derive interface input types.
pub fn store_group_input_structure_types(
    tree: &TreeModel,
    input_requirements: &[DataRequirement],
    derived_inputs: &mut Vec<StructureType>,
) {
    let mut interface_requirements =
        vec![DataRequirement::None; tree.interface_inputs.len()];
    for &node_i in &tree.group_input_nodes {
        let node = &tree.nodes[node_i];
        for i in 0..node.outputs.len().saturating_sub(1) {
            let output = node.outputs[i];
            interface_requirements[i] = merge(
                interface_requirements[i],
                calc_output_socket_requirement(tree, output, input_requirements),
            );
        }
    }
    for (i, &io_st) in tree.interface_inputs.iter().enumerate() {
        if io_st != NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO {
            derived_inputs[i] = match io_st {
                NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_SINGLE => StructureType::Single,
                _ => StructureType::Dynamic,
            };
            continue;
        }
        derived_inputs[i] =
            data_requirement_to_auto_structure_type(interface_requirements[i]);
    }
}

/// `store_auto_output_structure_types` — resolve auto outputs.
pub fn store_auto_output_structure_types(
    tree: &TreeModel,
    input_requirements: &[DataRequirement],
    is_auto: &[bool],
    structure_types: &mut [StructureType],
) {
    for (i, socket) in tree.sockets.iter().enumerate() {
        if !is_auto[i] || socket.is_input {
            continue;
        }
        let requirement =
            calc_output_socket_requirement(tree, i, input_requirements);
        structure_types[i] = ensure_supported_structure_type(
            data_requirement_to_auto_structure_type(requirement),
            &socket.supported,
        );
    }
}

/// `simulation_zone_requirements_propagate`.
pub fn simulation_zone_requirements_propagate(
    tree: &TreeModel,
    input_node: &NodeModel,
    output_node: &NodeModel,
    input_requirements: &mut [DataRequirement],
) -> ZoneInOutChange {
    let mut change = ZONE_CHANGE_NONE;
    for i in 0..output_node.outputs.len() {
        let in_in = input_node.inputs[i];
        let out_out = output_node.outputs[i];
        let in_out = output_node.inputs[i + 1];
        let new_value = merge(
            input_requirements[tree.input_req_index(in_in)],
            calc_output_socket_requirement(tree, out_out, input_requirements),
        );
        let ii = tree.input_req_index(in_in);
        if input_requirements[ii] != new_value {
            input_requirements[ii] = new_value;
            change = change.with(ZONE_CHANGE_IN);
        }
        let oi = tree.input_req_index(in_out);
        if input_requirements[oi] != new_value {
            input_requirements[oi] = new_value;
            change = change.with(ZONE_CHANGE_OUT);
        }
    }
    change
}

/// `repeat_zone_requirements_propagate`.
pub fn repeat_zone_requirements_propagate(
    tree: &TreeModel,
    input_node: &NodeModel,
    output_node: &NodeModel,
    input_requirements: &mut [DataRequirement],
) -> ZoneInOutChange {
    let mut change = ZONE_CHANGE_NONE;
    for i in 0..output_node.outputs.len() {
        let in_in = input_node.inputs[i + 1];
        let out_out = output_node.outputs[i];
        let in_out = output_node.inputs[i];
        let new_value = merge(
            input_requirements[tree.input_req_index(in_in)],
            calc_output_socket_requirement(tree, out_out, input_requirements),
        );
        let ii = tree.input_req_index(in_in);
        if input_requirements[ii] != new_value {
            input_requirements[ii] = new_value;
            change = change.with(ZONE_CHANGE_IN);
        }
        let oi = tree.input_req_index(in_out);
        if input_requirements[oi] != new_value {
            input_requirements[oi] = new_value;
            change = change.with(ZONE_CHANGE_OUT);
        }
    }
    change
}

/// `simulation_zone_status_propagate` — L2R across zone border.
pub fn simulation_zone_status_propagate(
    tree: &TreeModel,
    input_node: &NodeModel,
    output_node: &NodeModel,
    structure_types: &mut [StructureType],
) -> ZoneInOutChange {
    let mut change = ZONE_CHANGE_NONE;
    for i in 0..output_node.outputs.len() {
        let input = input_node.outputs[i + 1];
        let output = output_node.outputs[i];
        let new_value = left_to_right_merge(
            structure_types[tree.sockets[input].index_in_tree],
            structure_types[tree.sockets[output].index_in_tree],
        );
        let new_input = ensure_supported_structure_type(new_value, &tree.sockets[input].supported);
        let new_output =
            ensure_supported_structure_type(new_value, &tree.sockets[output].supported);
        let ti = tree.sockets[input].index_in_tree;
        if structure_types[ti] != new_input {
            structure_types[ti] = new_input;
            change = change.with(ZONE_CHANGE_IN);
        }
        let to = tree.sockets[output].index_in_tree;
        if structure_types[to] != new_output {
            structure_types[to] = new_output;
            change = change.with(ZONE_CHANGE_OUT);
        }
    }
    change
}

/// `repeat_zone_status_propagate`.
pub fn repeat_zone_status_propagate(
    tree: &TreeModel,
    input_node: &NodeModel,
    output_node: &NodeModel,
    structure_types: &mut [StructureType],
) -> ZoneInOutChange {
    let mut change = ZONE_CHANGE_NONE;
    for i in 0..output_node.outputs.len() {
        let input_of_input = input_node.outputs[i + 1];
        let output_of_output = output_node.outputs[i];
        let new_value = left_to_right_merge(
            structure_types[tree.sockets[input_of_input].index_in_tree],
            structure_types[tree.sockets[output_of_output].index_in_tree],
        );
        let new_input = ensure_supported_structure_type(
            new_value,
            &tree.sockets[input_of_input].supported,
        );
        let new_output = ensure_supported_structure_type(
            new_value,
            &tree.sockets[output_of_output].supported,
        );
        let ti = tree.sockets[input_of_input].index_in_tree;
        if structure_types[ti] != new_input {
            structure_types[ti] = new_input;
            change = change.with(ZONE_CHANGE_IN);
        }
        let to = tree.sockets[output_of_output].index_in_tree;
        if structure_types[to] != new_output {
            structure_types[to] = new_output;
            change = change.with(ZONE_CHANGE_OUT);
        }
    }
    change
}

/// `propagate_zone_data_requirements` — one node step (R2L).
pub fn propagate_zone_data_requirements(
    tree: &TreeModel,
    node_index: usize,
    input_requirements: &mut [DataRequirement],
) -> bool {
    let node = &tree.nodes[node_index];
    match node.type_tag {
        GEO_NODE_SIMULATION_INPUT | GEO_NODE_REPEAT_INPUT => {
            if let Some(out_id) = node.output_node_id {
                if let Some(output_node) = tree.node_by_id(out_id) {
                    let change = if node.type_tag == GEO_NODE_SIMULATION_INPUT {
                        simulation_zone_requirements_propagate(
                            tree,
                            node,
                            output_node,
                            input_requirements,
                        )
                    } else {
                        repeat_zone_requirements_propagate(
                            tree,
                            node,
                            output_node,
                            input_requirements,
                        )
                    };
                    return change.has(ZONE_CHANGE_OUT);
                }
            }
            false
        }
        GEO_NODE_SIMULATION_OUTPUT | GEO_NODE_REPEAT_OUTPUT => {
            let input_tag = if node.type_tag == GEO_NODE_SIMULATION_OUTPUT {
                GEO_NODE_SIMULATION_INPUT
            } else {
                GEO_NODE_REPEAT_INPUT
            };
            for input_node in tree.nodes_of_type(input_tag) {
                if input_node.output_node_id == Some(node.identifier) {
                    let change = if input_tag == GEO_NODE_SIMULATION_INPUT {
                        simulation_zone_requirements_propagate(
                            tree,
                            input_node,
                            node,
                            input_requirements,
                        )
                    } else {
                        repeat_zone_requirements_propagate(
                            tree,
                            input_node,
                            node,
                            input_requirements,
                        )
                    };
                    if change.has(ZONE_CHANGE_IN) {
                        return true;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

/// `propagate_zone_status` — one node step (L2R).
pub fn propagate_zone_status(
    tree: &TreeModel,
    node_index: usize,
    structure_types: &mut [StructureType],
) -> bool {
    let node = &tree.nodes[node_index];
    match node.type_tag {
        GEO_NODE_SIMULATION_INPUT | GEO_NODE_REPEAT_INPUT => {
            if let Some(out_id) = node.output_node_id {
                if let Some(output_node) = tree.node_by_id(out_id) {
                    let change = if node.type_tag == GEO_NODE_SIMULATION_INPUT {
                        simulation_zone_status_propagate(
                            tree,
                            node,
                            output_node,
                            structure_types,
                        )
                    } else {
                        repeat_zone_status_propagate(
                            tree,
                            node,
                            output_node,
                            structure_types,
                        )
                    };
                    return change.has(ZONE_CHANGE_OUT);
                }
            }
            false
        }
        GEO_NODE_SIMULATION_OUTPUT | GEO_NODE_REPEAT_OUTPUT => {
            let input_tag = if node.type_tag == GEO_NODE_SIMULATION_OUTPUT {
                GEO_NODE_SIMULATION_INPUT
            } else {
                GEO_NODE_REPEAT_INPUT
            };
            for input_node in tree.nodes_of_type(input_tag) {
                if input_node.output_node_id == Some(node.identifier) {
                    let change = if input_tag == GEO_NODE_SIMULATION_INPUT {
                        simulation_zone_status_propagate(
                            tree,
                            input_node,
                            node,
                            structure_types,
                        )
                    } else {
                        repeat_zone_status_propagate(
                            tree,
                            input_node,
                            node,
                            structure_types,
                        )
                    };
                    if change.has(ZONE_CHANGE_IN) {
                        return true;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

/// `get_unconnected_input_structure_type`.
pub fn get_unconnected_input_structure_type(default_input_is_field: bool) -> StructureType {
    if default_input_is_field {
        StructureType::Field
    } else {
        StructureType::Single
    }
}

/// `propagate_right_to_left` — one pass: inputs inherit the merged
/// requirement of the output driving them (per node), zones step.
pub fn propagate_right_to_left(
    tree: &TreeModel,
    is_auto: &[bool],
    input_requirements: &mut [DataRequirement],
) -> bool {
    let mut changed = false;
    /* Outputs pull requirements from linked inputs. */
    for node in 0..tree.nodes.len() {
        if propagate_zone_data_requirements(tree, node, input_requirements) {
            changed = true;
        }
    }
    /* For each non-auto output, push requirement into linked auto
       sockets' inferred interface — handled by callers re-running
       `calc_output_socket_requirement` to fixpoint. */
    let _ = is_auto;
    changed
}

/// `StructureTypeInferenceResult`.
#[derive(Default)]
pub struct StructureTypeInferenceResult {
    /// per-socket structure types (index_in_tree).
    pub structure_types: Vec<StructureType>,
    /// interface inputs.
    pub interface_inputs: Vec<StructureType>,
    /// interface outputs.
    pub interface_outputs: Vec<StructureType>,
}

/// `update_structure_type_interface` driver — R2L fixpoint for
/// requirements then L2R fixpoint for structure types.
pub fn update_structure_type_interface(
    tree: &TreeModel,
    is_auto: &[bool],
) -> StructureTypeInferenceResult {
    /* Right-to-left pass: input requirements fixpoint. */
    let mut input_requirements =
        vec![DataRequirement::None; tree.sockets.iter().filter(|s| s.is_input).count() + 1];
    init_input_requirements(tree, is_auto, &mut input_requirements);
    loop {
        let mut changed = false;
        for node in 0..tree.nodes.len() {
            if propagate_zone_data_requirements(tree, node, &mut input_requirements) {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    /* Initialize structure types: declared, or Dynamic for auto. */
    let mut structure_types: Vec<StructureType> = tree
        .sockets
        .iter()
        .map(|s| s.decl_structure_type.unwrap_or(StructureType::Dynamic))
        .collect();

    /* Left-to-right fixpoint: linked sockets merge; zone borders step. */
    loop {
        let mut changed = false;
        for socket in &tree.sockets {
            if !socket.is_input {
                continue;
            }
            let merged = match socket.linked_output {
                Some(out) => structure_types[tree.sockets[out].index_in_tree],
                None => get_unconnected_input_structure_type(socket.default_input_is_field),
            };
            let new_value = ensure_supported_structure_type(merged, &socket.supported);
            if structure_types[socket.index_in_tree] != new_value {
                structure_types[socket.index_in_tree] = new_value;
                changed = true;
            }
        }
        for node in 0..tree.nodes.len() {
            if propagate_zone_status(tree, node, &mut structure_types) {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    /* Store results. */
    store_auto_output_structure_types(
        tree,
        &input_requirements,
        is_auto,
        &mut structure_types,
    );
    let mut interface_inputs = vec![StructureType::Dynamic; tree.interface_inputs.len()];
    store_group_input_structure_types(tree, &input_requirements, &mut interface_inputs);
    let interface_outputs = tree
        .interface_outputs
        .iter()
        .map(|&t| match t {
            NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_SINGLE => StructureType::Single,
            _ => StructureType::Dynamic,
        })
        .collect();

    StructureTypeInferenceResult {
        structure_types,
        interface_inputs,
        interface_outputs,
    }
}
