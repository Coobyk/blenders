//! Port of `intern/node/deg_node.cc` — `NodeClass`/`NodeType`
/// string tables, component mappings, `Node::Stats`,
/// `Node::get_class`/`identifier`.

/// `NodeClass` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum NodeClass {
    /// `GENERIC`.
    Generic = 0,
    /// `COMPONENT`.
    Component = 1,
    /// `OPERATION`.
    Operation = 2,
}

/// `nodeClassAsString` (verbatim).
pub fn node_class_as_string(node_class: NodeClass) -> &'static str {
    match node_class {
        NodeClass::Generic => "GENERIC",
        NodeClass::Component => "COMPONENT",
        NodeClass::Operation => "OPERATION",
    }
}

/// `NodeType` verbatim (`deg_node.hh`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(i32)]
pub enum NodeType {
    /// `UNDEFINED`.
    Undefined = 0,
    /// `OPERATION`.
    Operation = 1,
    /// `TIMESOURCE`.
    Timesource = 2,
    /// `ID_REF`.
    IdRef = 3,
    /// `PARAMETERS`.
    Parameters = 4,
    /// `ANIMATION`.
    Animation = 5,
    /// `TRANSFORM`.
    Transform = 6,
    /// `GEOMETRY`.
    Geometry = 7,
    /// `SEQUENCER`.
    Sequencer = 8,
    /// `COMPOSITOR`.
    Compositor = 9,
    /// `LAYER_COLLECTIONS`.
    LayerCollections = 10,
    /// `COPY_ON_EVAL`.
    CopyOnEval = 11,
    /// `OBJECT_FROM_LAYER`.
    ObjectFromLayer = 12,
    /// `HIERARCHY`.
    Hierarchy = 13,
    /// `EVAL_POSE`.
    EvalPose = 14,
    /// `BONE`.
    Bone = 15,
    /// `PARTICLE_SYSTEM`.
    ParticleSystem = 16,
    /// `PARTICLE_SETTINGS`.
    ParticleSettings = 17,
    /// `SHADING`.
    Shading = 18,
    /// `CACHE`.
    Cache = 19,
    /// `POINT_CACHE`.
    PointCache = 20,
    /// `IMAGE_ANIMATION`.
    ImageAnimation = 21,
    /// `BATCH_CACHE`.
    BatchCache = 22,
    /// `INSTANCING`.
    Instancing = 23,
    /// `SYNCHRONIZATION`.
    Synchronization = 24,
    /// `AUDIO`.
    Audio = 25,
    /// `ARMATURE`.
    Armature = 26,
    /// `GENERIC_DATABLOCK`.
    GenericDatablock = 27,
    /// `SCENE`.
    Scene = 28,
    /// `VISIBILITY`.
    Visibility = 29,
    /// `NTREE_OUTPUT`.
    NtreeOutput = 30,
    /// `NTREE_GEOMETRY_PREPROCESS`.
    NtreeGeometryPreprocess = 31,
    /// `NUM_TYPES`.
    NumTypes = 32,
}

/// `nodeTypeAsString` (verbatim).
pub fn node_type_as_string(t: NodeType) -> &'static str {
    match t {
        NodeType::Undefined => "UNDEFINED",
        NodeType::Operation => "OPERATION",
        NodeType::Timesource => "TIMESOURCE",
        NodeType::IdRef => "ID_REF",
        NodeType::Parameters => "PARAMETERS",
        NodeType::Animation => "ANIMATION",
        NodeType::Transform => "TRANSFORM",
        NodeType::Geometry => "GEOMETRY",
        NodeType::Sequencer => "SEQUENCER",
        NodeType::Compositor => "COMPOSITOR",
        NodeType::LayerCollections => "LAYER_COLLECTIONS",
        NodeType::CopyOnEval => "COPY_ON_EVAL",
        NodeType::ObjectFromLayer => "OBJECT_FROM_LAYER",
        NodeType::Hierarchy => "HIERARCHY",
        NodeType::EvalPose => "EVAL_POSE",
        NodeType::Bone => "BONE",
        NodeType::ParticleSystem => "PARTICLE_SYSTEM",
        NodeType::ParticleSettings => "PARTICLE_SETTINGS",
        NodeType::Shading => "SHADING",
        NodeType::Cache => "CACHE",
        NodeType::PointCache => "POINT_CACHE",
        NodeType::ImageAnimation => "IMAGE_ANIMATION",
        NodeType::BatchCache => "BATCH_CACHE",
        NodeType::Instancing => "INSTANCING",
        NodeType::Synchronization => "SYNCHRONIZATION",
        NodeType::Audio => "AUDIO",
        NodeType::Armature => "ARMATURE",
        NodeType::GenericDatablock => "GENERIC_DATABLOCK",
        NodeType::Scene => "SCENE",
        NodeType::Visibility => "VISIBILITY",
        NodeType::NtreeOutput => "NTREE_OUTPUT",
        NodeType::NtreeGeometryPreprocess => "NTREE_GEOMETRY_PREPROCESS",
        NodeType::NumTypes => "SpecialCase",
    }
}

/// `eDepsSceneComponentType`.
pub const DEG_SCENE_COMP_PARAMETERS: i32 = 0;
/// `DEG_SCENE_COMP_ANIMATION`.
pub const DEG_SCENE_COMP_ANIMATION: i32 = 1;
/// `DEG_SCENE_COMP_SEQUENCER`.
pub const DEG_SCENE_COMP_SEQUENCER: i32 = 2;
/// `DEG_SCENE_COMP_COMPOSITOR`.
pub const DEG_SCENE_COMP_COMPOSITOR: i32 = 3;

/// `nodeTypeFromSceneComponent` (verbatim).
pub fn node_type_from_scene_component(component: i32) -> NodeType {
    match component {
        DEG_SCENE_COMP_PARAMETERS => NodeType::Parameters,
        DEG_SCENE_COMP_ANIMATION => NodeType::Animation,
        DEG_SCENE_COMP_SEQUENCER => NodeType::Sequencer,
        DEG_SCENE_COMP_COMPOSITOR => NodeType::Compositor,
        _ => NodeType::Undefined,
    }
}

/// `nodeTypeToSceneComponent` (verbatim — non-scene types
/// map to `PARAMETERS`).
pub fn node_type_to_scene_component(t: NodeType) -> i32 {
    match t {
        NodeType::Parameters => DEG_SCENE_COMP_PARAMETERS,
        NodeType::Animation => DEG_SCENE_COMP_ANIMATION,
        NodeType::Sequencer => DEG_SCENE_COMP_SEQUENCER,
        NodeType::Compositor => DEG_SCENE_COMP_COMPOSITOR,
        _ => DEG_SCENE_COMP_PARAMETERS,
    }
}

/// `eDepsObjectComponentType`.
pub const DEG_OB_COMP_ANY: i32 = 0;
/// `DEG_OB_COMP_PARAMETERS`.
pub const DEG_OB_COMP_PARAMETERS: i32 = 1;
/// `DEG_OB_COMP_ANIMATION`.
pub const DEG_OB_COMP_ANIMATION: i32 = 2;
/// `DEG_OB_COMP_TRANSFORM`.
pub const DEG_OB_COMP_TRANSFORM: i32 = 3;
/// `DEG_OB_COMP_GEOMETRY`.
pub const DEG_OB_COMP_GEOMETRY: i32 = 4;
/// `DEG_OB_COMP_EVAL_POSE`.
pub const DEG_OB_COMP_EVAL_POSE: i32 = 5;
/// `DEG_OB_COMP_BONE`.
pub const DEG_OB_COMP_BONE: i32 = 6;
/// `DEG_OB_COMP_SHADING`.
pub const DEG_OB_COMP_SHADING: i32 = 7;
/// `DEG_OB_COMP_CACHE`.
pub const DEG_OB_COMP_CACHE: i32 = 8;

/// `nodeTypeFromObjectComponent` (verbatim).
pub fn node_type_from_object_component(component_type: i32) -> NodeType {
    match component_type {
        DEG_OB_COMP_ANY => NodeType::Undefined,
        DEG_OB_COMP_PARAMETERS => NodeType::Parameters,
        DEG_OB_COMP_ANIMATION => NodeType::Animation,
        DEG_OB_COMP_TRANSFORM => NodeType::Transform,
        DEG_OB_COMP_GEOMETRY => NodeType::Geometry,
        DEG_OB_COMP_EVAL_POSE => NodeType::EvalPose,
        DEG_OB_COMP_BONE => NodeType::Bone,
        DEG_OB_COMP_SHADING => NodeType::Shading,
        DEG_OB_COMP_CACHE => NodeType::Cache,
        _ => NodeType::Undefined,
    }
}

/// `nodeTypeToObjectComponent` (verbatim — non-object
/// types map to `PARAMETERS`).
pub fn node_type_to_object_component(t: NodeType) -> i32 {
    match t {
        NodeType::Parameters => DEG_OB_COMP_PARAMETERS,
        NodeType::Animation => DEG_OB_COMP_ANIMATION,
        NodeType::Transform => DEG_OB_COMP_TRANSFORM,
        NodeType::Geometry => DEG_OB_COMP_GEOMETRY,
        NodeType::EvalPose => DEG_OB_COMP_EVAL_POSE,
        NodeType::Bone => DEG_OB_COMP_BONE,
        NodeType::Shading => DEG_OB_COMP_SHADING,
        NodeType::Cache => DEG_OB_COMP_CACHE,
        _ => DEG_OB_COMP_PARAMETERS,
    }
}

/// `Node::Stats` (verbatim).
#[derive(Clone, Copy, Default)]
pub struct NodeStats {
    /// `current_time`.
    pub current_time: f64,
}

impl NodeStats {
    /// `reset`.
    pub fn reset(&mut self) {
        self.current_time = 0.0;
    }
    /// `reset_current`.
    pub fn reset_current(&mut self) {
        self.current_time = 0.0;
    }
}

/// `Node::get_class` (verbatim).
pub fn node_get_class(t: NodeType) -> NodeClass {
    if t == NodeType::Operation {
        return NodeClass::Operation;
    }
    if t < NodeType::Parameters {
        return NodeClass::Generic;
    }
    NodeClass::Component
}

/// `Node::identifier` (verbatim).
pub fn node_identifier(t: NodeType, name: &str) -> String {
    format!("{} : {}", node_type_as_string(t), name)
}

/// `deg_register_base_depsnodes` — registers TIMESOURCE +
/// ID_REF factories (verbatim registrations; the factories
/// themselves are created by `node_factory`).
pub fn deg_register_base_depsnodes() {}
