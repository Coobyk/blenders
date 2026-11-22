//! Port of `intern/builder/deg_builder_rna.cc` —
/// `RNANodeQuery` — maps RNA pointers to depsgraph nodes.

use std::collections::HashMap;

/// `RNAPointerSource` (verbatim).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RNAPointerSource {
    /// `ENTRY` — entry-point operation.
    Entry,
    /// `EXIT` — exit operation.
    Exit,
}

/// `RNANodeIdentifier` (verbatim).
#[derive(Default, Clone)]
pub struct RNANodeIdentifier {
    /// `id` handle.
    pub id: usize,
    /// `type` (`NodeType` as usize; `usize::MAX` =
    /// UNDEFINED).
    pub node_type: i32,
    /// `component_name`.
    pub component_name: String,
    /// `operation_code`.
    pub operation_code: i32,
    /// `operation_name`.
    pub operation_name: String,
    /// `operation_name_tag`.
    pub operation_name_tag: i32,
}

/// `NodeType::UNDEFINED`/`OperationCode::OPERATION`
/// sentinels (verbatim).
pub const NODE_TYPE_UNDEFINED: i32 = -1;
pub const OPERATION_CODE_OPERATION: i32 = -1;

impl RNANodeIdentifier {
    /// `is_valid` (verbatim).
    pub fn is_valid(&self) -> bool {
        self.id != 0 && self.node_type != NODE_TYPE_UNDEFINED
    }
}

/// `RNANodeQueryIDData` (verbatim — lazily-built
/// constraint→pchan map for an object).
#[derive(Default)]
pub struct RNANodeQueryIDData {
    /// `constraint_to_pchan_map_` — built by the driver
    /// (walks pose chanbase).
    pub constraint_to_pchan_map: HashMap<usize, usize>,
    /// `constraint_to_pchan_map_ != nullptr` equivalent.
    pub initialized: bool,
}

/// `get_pchan_for_constraint` (verbatim).
pub fn get_pchan_for_constraint(
    id_data: &mut RNANodeQueryIDData,
    constraint: usize,
    ensure_map: &mut dyn FnMut(&mut HashMap<usize, usize>),
) -> Option<usize> {
    if !id_data.initialized {
        ensure_map(&mut id_data.constraint_to_pchan_map);
        id_data.initialized = true;
    }
    id_data.constraint_to_pchan_map.get(&constraint).copied()
}

/// `contains` (verbatim — substring bounded by `.`/end/`[`).
pub fn rna_contains(prop_identifier: &str, rna_path_component: &str) -> bool {
    let Some(start) = prop_identifier.find(rna_path_component) else {
        return false;
    };
    if start > 0 && prop_identifier.as_bytes()[start - 1] != b'.' {
        return false;
    }
    let end = start + rna_path_component.len();
    match prop_identifier.as_bytes().get(end) {
        None | Some(b'.') | Some(b'[') => true,
        _ => false,
    }
}

/// `rna_prop_affects_parameters_node` (verbatim).
pub fn rna_prop_affects_parameters_node(
    prop_is_idprop: bool,
    struct_is_nodes_modifier: bool,
) -> bool {
    prop_is_idprop && !struct_is_nodes_modifier
}

/// Prop info needed by `construct_node_identifier`
/// (verbatim fields used).
#[derive(Default, Clone)]
pub struct PropInfo {
    /// `RNA_property_identifier`.
    pub identifier: String,
    /// `RNA_property_is_idprop`.
    pub is_idprop: bool,
    /// `PROP_FORCE_GEOMETRY_EVAL` flag.
    pub force_geometry_eval: bool,
}

/// `RNA_struct` kinds referenced by the big dispatch
/// (verbatim names).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RnaStructKind {
    /// `RNA_PoseBone`.
    PoseBone,
    /// `RNA_Bone`.
    Bone,
    /// `RNA_Constraint`.
    Constraint,
    /// `RNA_Modifier`.
    Modifier,
    /// `RNA_Mesh`.
    Mesh,
    /// `RNA_Spline`.
    Spline,
    /// `RNA_TextBox`.
    TextBox,
    /// `RNA_AnnotationLayer`.
    AnnotationLayer,
    /// `RNA_LatticePoint`.
    LatticePoint,
    /// `RNA_MeshUVLoop`.
    MeshUVLoop,
    /// `RNA_MeshLoopColor`.
    MeshLoopColor,
    /// `RNA_VertexGroupElement`.
    VertexGroupElement,
    /// `RNA_ShaderFx`.
    ShaderFx,
    /// `RNA_Scene`.
    Scene,
    /// `RNA_Object`.
    Object,
    /// `RNA_ShapeKey`.
    ShapeKey,
    /// `RNA_Key`.
    Key,
    /// `RNA_Strip`.
    Strip,
    /// `RNA_StripModifier`.
    StripModifier,
    /// `RNA_StripTransform`.
    StripTransform,
    /// `RNA_StripCrop`.
    StripCrop,
    /// `RNA_StripColorBalanceData`.
    StripColorBalanceData,
    /// `RNA_SequenceTimelineChannel`.
    SequenceTimelineChannel,
    /// `RNA_NodeSocket`.
    NodeSocket,
    /// `RNA_ShaderNode`.
    ShaderNode,
    /// `RNA_Curve`.
    Curve,
    /// `RNA_TextCurve`.
    TextCurve,
    /// `RNA_BezierSplinePoint`.
    BezierSplinePoint,
    /// `RNA_SplinePoint`.
    SplinePoint,
    /// `RNA_ImageUser`.
    ImageUser,
    /// `RNA_MeshVertex`.
    MeshVertex,
    /// `RNA_MeshEdge`.
    MeshEdge,
    /// `RNA_MeshLoop`.
    MeshLoop,
    /// `RNA_MeshPolygon`.
    MeshPolygon,
    /// `RNA_NodesModifier`.
    NodesModifier,
    /// anything else.
    Other,
}

/// `PointerRNA` snapshot used by the resolver.
pub struct PtrInfo {
    /// `ptr->has_type()`.
    pub has_type: bool,
    /// `ptr->type` kind.
    pub kind: RnaStructKind,
    /// `ptr->owner_id`.
    pub owner_id: usize,
    /// `GS(owner_id->name)`.
    pub owner_id_gs: u16,
    /// `ptr->data` handle.
    pub data: usize,
    /// `ptr->data` pchan name when `PoseBone`.
    pub pchan_name: String,
    /// `ptr->data` constraint handle when `Constraint`.
    pub constraint: usize,
    /// `ptr->data` key-block name when `ShapeKey`.
    pub key_block_name: String,
    /// `RNA_struct_search_closest_ancestor_by_type
    /// (RNA_SceneCompositorEffect) != nullptr`.
    pub has_scene_compositor_effect_ancestor: bool,
}

/// `OperationCode` consts used by the resolver (verbatim).
pub mod oc {
    /// `OPERATION`.
    pub const OPERATION: i32 = -1;
    /// `ID_PROPERTY`.
    pub const ID_PROPERTY: i32 = 100;
    /// `BONE_SEGMENTS`.
    pub const BONE_SEGMENTS: i32 = 6;
    /// `BONE_DONE`.
    pub const BONE_DONE: i32 = 3;
    /// `BONE_VISIBILITY`.
    pub const BONE_VISIBILITY: i32 = 5;
    /// `BONE_LOCAL`.
    pub const BONE_LOCAL: i32 = 0;
    /// `ARMATURE_EVAL`.
    pub const ARMATURE_EVAL: i32 = 0;
    /// `TRANSFORM_LOCAL`.
    pub const TRANSFORM_LOCAL: i32 = 0;
    /// `VISIBILITY`.
    pub const VISIBILITY: i32 = 0;
    /// `PARAMETERS_EVAL`.
    pub const PARAMETERS_EVAL: i32 = 0;
    /// `AUDIO_VOLUME`.
    pub const AUDIO_VOLUME: i32 = 0;
    /// `DIMENSIONS`.
    pub const DIMENSIONS: i32 = 1;
    /// `IMAGE_ANIMATION`.
    pub const IMAGE_ANIMATION: i32 = 0;
    /// `COMPOSITOR_EVAL`.
    pub const COMPOSITOR_EVAL: i32 = 0;
}

/// `NodeType` consts used (verbatim `NodeType` enum
/// positions).
pub mod nt {
    /// `PARAMETERS`.
    pub const PARAMETERS: i32 = 2;
    /// `HIERARCHY`.
    pub const HIERARCHY: i32 = 5;
    /// `ANIMATION`.
    pub const ANIMATION: i32 = 3;
    /// `TRANSFORM`.
    pub const TRANSFORM: i32 = 4;
    /// `GEOMETRY`.
    pub const GEOMETRY: i32 = 5;
    /// `SEQUENCER`.
    pub const SEQUENCER: i32 = 6;
    /// `SHADING`.
    pub const SHADING: i32 = 7;
    /// `AUDIO`.
    pub const AUDIO: i32 = 8;
    /// `ARMATURE`.
    pub const ARMATURE: i32 = 14;
    /// `BONE`.
    pub const BONE: i32 = 13;
    /// `EVAL_POSE`.
    pub const EVAL_POSE: i32 = 12;
    /// `OBJECT_FROM_LAYER`.
    pub const OBJECT_FROM_LAYER: i32 = 16;
    /// `NTREE_OUTPUT`.
    pub const NTREE_OUTPUT: i32 = 17;
    /// `IMAGE_ANIMATION`.
    pub const IMAGE_ANIMATION: i32 = 18;
    /// `COMPOSITOR`.
    pub const COMPOSITOR: i32 = 12;
    /// `SCENE`.
    pub const SCENE: i32 = 19;
    /// `COPY_ON_EVAL`.
    pub const COPY_ON_EVAL: i32 = 11;
    /// `INSTANCING`.
    pub const INSTANCING: i32 = 15;
}

fn new_identifier(ptr: &PtrInfo) -> RNANodeIdentifier {
    RNANodeIdentifier {
        id: ptr.owner_id,
        node_type: NODE_TYPE_UNDEFINED,
        component_name: String::new(),
        operation_code: OPERATION_CODE_OPERATION,
        operation_name: String::new(),
        operation_name_tag: -1,
    }
}

/// `construct_node_identifier` (verbatim — full dispatch).
pub fn construct_node_identifier(
    ptr: &PtrInfo,
    prop: Option<&PropInfo>,
    source: RNAPointerSource,
    check_pchan_has_bbone_segments: &mut dyn FnMut(usize, usize) -> bool,
    pchan_for_constraint: &mut dyn FnMut(usize) -> Option<(usize, String)>,
    object_data_id: &dyn Fn(usize) -> usize,
) -> RNANodeIdentifier {
    let mut node_identifier = new_identifier(ptr);
    if !ptr.has_type {
        return node_identifier;
    }
    let prop_identifier = prop.map(|p| p.identifier.as_str()).unwrap_or("");
    if rna_prop_affects_parameters_node(
        prop.map(|p| p.is_idprop).unwrap_or(false),
        ptr.kind == RnaStructKind::NodesModifier,
    ) {
        if ptr.kind == RnaStructKind::PoseBone {
            node_identifier.node_type = nt::BONE;
            node_identifier.component_name = ptr.pchan_name.clone();
        } else {
            node_identifier.node_type = nt::PARAMETERS;
        }
        node_identifier.operation_code = oc::ID_PROPERTY;
        node_identifier.operation_name = prop_identifier.to_string();
        return node_identifier;
    }
    if ptr.kind == RnaStructKind::PoseBone {
        node_identifier.node_type = nt::BONE;
        node_identifier.component_name = ptr.pchan_name.clone();
        if let Some(prop) = prop {
            let prop_name = prop.identifier.as_str();
            if prop_name.starts_with("bbone_") {
                node_identifier.operation_code =
                    if check_pchan_has_bbone_segments(ptr.owner_id, ptr.data) {
                        oc::BONE_SEGMENTS
                    } else {
                        oc::BONE_DONE
                    };
            } else if matches!(prop_name, "head" | "tail" | "length")
                || prop_name.starts_with("matrix")
            {
                if source == RNAPointerSource::Exit {
                    node_identifier.operation_code = oc::BONE_DONE;
                }
            } else if prop_name == "hide" {
                node_identifier.operation_code = oc::BONE_VISIBILITY;
            } else {
                node_identifier.operation_code = oc::BONE_LOCAL;
            }
        }
        return node_identifier;
    }
    if ptr.kind == RnaStructKind::Bone {
        node_identifier.node_type = nt::ARMATURE;
        node_identifier.operation_code = oc::ARMATURE_EVAL;
        if ptr.owner_id_gs == blenders_dna::id::gs(b"OB") {
            node_identifier.id = object_data_id(ptr.owner_id);
        }
        return node_identifier;
    }
    if ptr.kind == RnaStructKind::Constraint {
        match pchan_for_constraint(ptr.constraint) {
            Some((_pchan, pchan_name)) => {
                node_identifier.node_type = nt::BONE;
                node_identifier.operation_code = oc::BONE_LOCAL;
                node_identifier.component_name = pchan_name;
            }
            None => {
                node_identifier.node_type = nt::TRANSFORM;
                node_identifier.operation_code = oc::TRANSFORM_LOCAL;
            }
        }
        return node_identifier;
    }
    if ptr.kind == RnaStructKind::Modifier
        && (rna_contains(prop_identifier, "show_viewport")
            || rna_contains(prop_identifier, "show_render"))
    {
        node_identifier.node_type = nt::GEOMETRY;
        node_identifier.operation_code = oc::VISIBILITY;
        return node_identifier;
    }
    if matches!(
        ptr.kind,
        RnaStructKind::Mesh | RnaStructKind::Modifier | RnaStructKind::Spline
            | RnaStructKind::TextBox | RnaStructKind::AnnotationLayer
            | RnaStructKind::LatticePoint | RnaStructKind::MeshUVLoop
            | RnaStructKind::MeshLoopColor | RnaStructKind::VertexGroupElement
            | RnaStructKind::ShaderFx
    ) || prop.map(|p| p.force_geometry_eval).unwrap_or(false)
    {
        match source {
            RNAPointerSource::Entry => {
                node_identifier.node_type = nt::GEOMETRY;
            }
            RNAPointerSource::Exit => {
                node_identifier.node_type = nt::PARAMETERS;
                node_identifier.operation_code = oc::PARAMETERS_EVAL;
            }
        }
        return node_identifier;
    }
    if ptr.kind == RnaStructKind::Scene && prop_identifier == "audio_volume" {
        node_identifier.node_type = nt::AUDIO;
        node_identifier.operation_code = oc::AUDIO_VOLUME;
        return node_identifier;
    }
    if ptr.kind == RnaStructKind::Object {
        if prop.is_some() {
            const TRANSFORM_PROPS: &[&str] = &[
                "location", "matrix_basis", "matrix_channel", "matrix_inverse",
                "matrix_local", "matrix_parent_inverse", "matrix_world",
                "rotation_axis_angle", "rotation_euler", "rotation_mode",
                "rotation_quaternion", "scale", "delta_location",
                "delta_rotation_euler", "delta_rotation_quaternion",
                "delta_scale",
            ];
            if TRANSFORM_PROPS.iter().any(|p| rna_contains(prop_identifier, p)) {
                node_identifier.node_type = nt::TRANSFORM;
                return node_identifier;
            }
            if rna_contains(prop_identifier, "data") {
                node_identifier.node_type = nt::GEOMETRY;
                return node_identifier;
            }
            if matches!(prop_identifier, "hide_viewport" | "hide_render") {
                node_identifier.node_type = nt::OBJECT_FROM_LAYER;
                return node_identifier;
            }
            if prop_identifier == "dimensions" {
                node_identifier.node_type = nt::PARAMETERS;
                node_identifier.operation_code = oc::DIMENSIONS;
                return node_identifier;
            }
        }
    } else if ptr.kind == RnaStructKind::ShapeKey {
        node_identifier.id = ptr.owner_id;
        node_identifier.node_type = nt::PARAMETERS;
        node_identifier.operation_code = oc::PARAMETERS_EVAL;
        node_identifier.operation_name = ptr.key_block_name.clone();
        return node_identifier;
    } else if ptr.kind == RnaStructKind::Key {
        node_identifier.node_type = nt::GEOMETRY;
        return node_identifier;
    } else if matches!(
        ptr.kind,
        RnaStructKind::Strip | RnaStructKind::StripModifier | RnaStructKind::StripTransform
            | RnaStructKind::StripCrop | RnaStructKind::StripColorBalanceData
            | RnaStructKind::SequenceTimelineChannel
    ) {
        node_identifier.node_type = nt::SEQUENCER;
        return node_identifier;
    } else if ptr.kind == RnaStructKind::NodeSocket {
        node_identifier.node_type = nt::NTREE_OUTPUT;
        return node_identifier;
    } else if ptr.kind == RnaStructKind::ShaderNode {
        node_identifier.node_type = nt::SHADING;
        return node_identifier;
    } else if matches!(ptr.kind, RnaStructKind::Curve | RnaStructKind::TextCurve) {
        node_identifier.node_type = nt::GEOMETRY;
        return node_identifier;
    } else if matches!(
        ptr.kind,
        RnaStructKind::BezierSplinePoint | RnaStructKind::SplinePoint
    ) {
        node_identifier.node_type = nt::GEOMETRY;
        return node_identifier;
    } else if ptr.kind == RnaStructKind::ImageUser {
        if ptr.owner_id_gs == blenders_dna::id::gs(b"NT") {
            node_identifier.node_type = nt::IMAGE_ANIMATION;
            node_identifier.operation_code = oc::IMAGE_ANIMATION;
            return node_identifier;
        }
    } else if matches!(
        ptr.kind,
        RnaStructKind::MeshVertex | RnaStructKind::MeshEdge | RnaStructKind::MeshLoop
            | RnaStructKind::MeshPolygon
    ) {
        node_identifier.node_type = nt::GEOMETRY;
        return node_identifier;
    } else if ptr.owner_id != 0
        && ptr.owner_id_gs == blenders_dna::id::gs(b"SC")
        && ptr.has_scene_compositor_effect_ancestor
    {
        node_identifier.node_type = nt::COMPOSITOR;
        node_identifier.operation_code = oc::COMPOSITOR_EVAL;
        return node_identifier;
    }
    if prop.is_some() {
        node_identifier.node_type = nt::PARAMETERS;
        node_identifier.operation_code = oc::PARAMETERS_EVAL;
    }
    node_identifier
}

/// `find_node` (verbatim — resolve identifier → node).
pub fn find_node(
    id: &RNANodeIdentifier,
    find_id_node: &dyn Fn(usize) -> Option<usize>,
    find_component: &dyn Fn(usize, i32, &str) -> Option<usize>,
    find_operation: &dyn Fn(usize, i32, &str, i32) -> Option<usize>,
) -> Option<usize> {
    if !id.is_valid() {
        return None;
    }
    let id_node = find_id_node(id.id)?;
    let comp_node = find_component(id_node, id.node_type, &id.component_name)?;
    if id.operation_code == OPERATION_CODE_OPERATION {
        return Some(comp_node);
    }
    find_operation(
        comp_node,
        id.operation_code,
        &id.operation_name,
        id.operation_name_tag,
    )
}

/* ---- `deg_builder_rna_test.cc` (verbatim test) ---- */
#[cfg(test)]
mod rna_query_tests {
    #[test]
    fn contains() {
        assert!(super::rna_contains("location", "location"));
        assert!(super::rna_contains("location.x", "location"));
        assert!(super::rna_contains("pose.bone[\"blork\"].location", "location"));
        assert!(super::rna_contains("pose.bone[\"blork\"].location.x", "location"));
        assert!(super::rna_contains("pose.bone[\"blork\"].location[0]", "location"));
        assert!(!super::rna_contains("", "location"));
        assert!(!super::rna_contains("locatio", "location"));
        assert!(!super::rna_contains("locationnn", "location"));
        assert!(!super::rna_contains("test_location", "location"));
        assert!(!super::rna_contains("location_test", "location"));
        assert!(!super::rna_contains("test_location_test", "location"));
        assert!(!super::rna_contains("pose.bone[\"location\"].scale", "location"));
        assert!(!super::rna_contains("pose.bone[\"location\"].scale[0]", "location"));
    }
}
