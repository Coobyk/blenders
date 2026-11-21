//! Port of `DNA_viewer_path_types.h`.
#![allow(non_camel_case_types)]
use crate::id::NIL;
use blenders_bli::listbase::ListBase;

/// `ViewerPathElemType`.
pub type ViewerPathElemType = i32;
pub const VIEWER_PATH_ELEM_TYPE_ID: ViewerPathElemType = 0;
pub const VIEWER_PATH_ELEM_TYPE_MODIFIER: ViewerPathElemType = 1;
pub const VIEWER_PATH_ELEM_TYPE_GROUP_NODE: ViewerPathElemType = 2;
pub const VIEWER_PATH_ELEM_TYPE_SIMULATION_ZONE: ViewerPathElemType = 3;
pub const VIEWER_PATH_ELEM_TYPE_VIEWER_NODE: ViewerPathElemType = 4;
pub const VIEWER_PATH_ELEM_TYPE_REPEAT_ZONE: ViewerPathElemType = 5;
pub const VIEWER_PATH_ELEM_TYPE_FOREACH_GEOMETRY_ELEMENT_ZONE: ViewerPathElemType = 6;
pub const VIEWER_PATH_ELEM_TYPE_EVALUATE_CLOSURE: ViewerPathElemType = 7;

/// `ViewerPathElem` — linked-list base. Subtypes share the header layout;
/// extra fields travel in `extra`.
#[derive(Clone)]
pub struct ViewerPathElem {
    pub next: usize,
    pub prev: usize,
    pub type_: ViewerPathElemType,
    /// `char *ui_name`.
    pub ui_name: Option<String>,
}
impl Default for ViewerPathElem {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: VIEWER_PATH_ELEM_TYPE_ID,
            ui_name: None,
        }
    }
}

/// `IDViewerPathElem`.
#[derive(Clone)]
pub struct IdViewerPathElem {
    pub base: ViewerPathElem,
    /// `ID *id`.
    pub id: usize,
}
impl Default for IdViewerPathElem {
    fn default() -> Self {
        Self {
            base: ViewerPathElem::default(),
            id: NIL,
        }
    }
}

/// `ModifierViewerPathElem`.
#[derive(Clone, Default)]
pub struct ModifierViewerPathElem {
    pub base: ViewerPathElem,
    /// `#ModifierData.persistent_uid`.
    pub modifier_uid: i32,
}

/// `GroupNodeViewerPathElem`.
#[derive(Clone, Default)]
pub struct GroupNodeViewerPathElem {
    pub base: ViewerPathElem,
    pub node_id: i32,
}

/// `SimulationZoneViewerPathElem`.
#[derive(Clone, Default)]
pub struct SimulationZoneViewerPathElem {
    pub base: ViewerPathElem,
    pub sim_output_node_id: i32,
}

/// `RepeatZoneViewerPathElem`.
#[derive(Clone, Default)]
pub struct RepeatZoneViewerPathElem {
    pub base: ViewerPathElem,
    pub repeat_output_node_id: i32,
    pub iteration: i32,
}

/// `ForeachGeometryElementZoneViewerPathElem`.
#[derive(Clone, Default)]
pub struct ForeachGeometryElementZoneViewerPathElem {
    pub base: ViewerPathElem,
    pub zone_output_node_id: i32,
    pub index: i32,
}

/// `ViewerNodeViewerPathElem`.
#[derive(Clone, Default)]
pub struct ViewerNodeViewerPathElem {
    pub base: ViewerPathElem,
    pub node_id: i32,
}

/// `EvaluateClosureNodeViewerPathElem`.
#[derive(Clone)]
pub struct EvaluateClosureNodeViewerPathElem {
    pub base: ViewerPathElem,
    /// Node that evaluates the closure.
    pub evaluate_node_id: i32,
    /// Output node of the evaluated closure zone.
    pub source_output_node_id: i32,
    /// `bNodeTree *source_node_tree`.
    pub source_node_tree: usize,
}
impl Default for EvaluateClosureNodeViewerPathElem {
    fn default() -> Self {
        Self {
            base: ViewerPathElem::default(),
            evaluate_node_id: 0,
            source_output_node_id: 0,
            source_node_tree: NIL,
        }
    }
}

/// `ViewerPath`.
#[derive(Clone, Default)]
pub struct ViewerPath {
    /// `ListBaseT<ViewerPathElem>`.
    pub path: ListBase,
}
