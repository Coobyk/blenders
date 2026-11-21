//! Port of `intern/viewer_path.cc` — `ViewerPath` element list:
//! init/clear/copy/equal/hash/new/copy/elem-free, foreach-id and
//! id-remap.
//!
//! Intrusive linked list → `Vec<ViewerPathElem>` enum over the 8
//! DNA element types; `ui_name` → `String`; blend read/write and the
//! ID-walk callback are injected/`PORT_PENDING`.

use blenders_dna::viewer_path_types::*;

/// `VIEWER_PATH_EQUAL_FLAG_IGNORE_ITERATION`.
pub const VIEWER_PATH_EQUAL_FLAG_IGNORE_ITERATION: i32 = 1 << 0;
/// `VIEWER_PATH_EQUAL_FLAG_CONSIDER_UI_NAME`.
pub const VIEWER_PATH_EQUAL_FLAG_CONSIDER_UI_NAME: i32 = 1 << 1;
pub type ViewerPathEqualFlag = i32;

/// `blenlib_detail::default_hash_factors` (BLI_hash.hh).
const DEFAULT_HASH_FACTORS: [u64; 5] =
    [19349669, 83492791, 3632623, 8789800933, 7235126189];

/// `get_default_hash` — `hash(v) ^ Σ factors[i] * args[i]` for
/// integer-like args (pointer args use `session_uid`, already a
/// `u64`).
fn get_default_hash(v: u64, args: &[u64]) -> u64 {
    let mut h = v;
    for (i, a) in args.iter().enumerate() {
        h ^= DEFAULT_HASH_FACTORS[i.min(4)].wrapping_mul(*a);
    }
    h
}

/// Owned element — one variant per `ViewerPathElemType`.
#[derive(Clone, Debug)]
pub enum ViewerPathElem {
    /// `IDViewerPathElem` — `id` is an ID handle (`session_uid` for
    /// hashing is looked up via the injected `id_session_uid` when
    /// hashing a path containing IDs).
    Id { id: usize, ui_name: String },
    /// `ModifierViewerPathElem`.
    Modifier { modifier_uid: i32, ui_name: String },
    /// `GroupNodeViewerPathElem`.
    GroupNode { node_id: i32, ui_name: String },
    /// `SimulationZoneViewerPathElem`.
    SimulationZone { sim_output_node_id: i32, ui_name: String },
    /// `ViewerNodeViewerPathElem`.
    ViewerNode { node_id: i32, ui_name: String },
    /// `RepeatZoneViewerPathElem`.
    RepeatZone { repeat_output_node_id: i32, iteration: i32, ui_name: String },
    /// `ForeachGeometryElementZoneViewerPathElem`.
    ForeachGeometryElementZone { zone_output_node_id: i32, index: i32, ui_name: String },
    /// `EvaluateClosureNodeViewerPathElem`.
    EvaluateClosure {
        evaluate_node_id: i32,
        source_output_node_id: i32,
        source_node_tree: usize,
        ui_name: String,
    },
}

impl ViewerPathElem {
    /// `ViewerPathElem.type`.
    pub fn type_(&self) -> ViewerPathElemType {
        match self {
            Self::Id { .. } => VIEWER_PATH_ELEM_TYPE_ID,
            Self::Modifier { .. } => VIEWER_PATH_ELEM_TYPE_MODIFIER,
            Self::GroupNode { .. } => VIEWER_PATH_ELEM_TYPE_GROUP_NODE,
            Self::SimulationZone { .. } => VIEWER_PATH_ELEM_TYPE_SIMULATION_ZONE,
            Self::ViewerNode { .. } => VIEWER_PATH_ELEM_TYPE_VIEWER_NODE,
            Self::RepeatZone { .. } => VIEWER_PATH_ELEM_TYPE_REPEAT_ZONE,
            Self::ForeachGeometryElementZone { .. } => {
                VIEWER_PATH_ELEM_TYPE_FOREACH_GEOMETRY_ELEMENT_ZONE
            }
            Self::EvaluateClosure { .. } => VIEWER_PATH_ELEM_TYPE_EVALUATE_CLOSURE,
        }
    }
    /// `ViewerPathElem.ui_name`.
    pub fn ui_name(&self) -> &str {
        match self {
            Self::Id { ui_name, .. }
            | Self::Modifier { ui_name, .. }
            | Self::GroupNode { ui_name, .. }
            | Self::SimulationZone { ui_name, .. }
            | Self::ViewerNode { ui_name, .. }
            | Self::RepeatZone { ui_name, .. }
            | Self::ForeachGeometryElementZone { ui_name, .. }
            | Self::EvaluateClosure { ui_name, .. } => ui_name,
        }
    }
    pub fn ui_name_mut(&mut self) -> &mut String {
        match self {
            Self::Id { ui_name, .. }
            | Self::Modifier { ui_name, .. }
            | Self::GroupNode { ui_name, .. }
            | Self::SimulationZone { ui_name, .. }
            | Self::ViewerNode { ui_name, .. }
            | Self::RepeatZone { ui_name, .. }
            | Self::ForeachGeometryElementZone { ui_name, .. }
            | Self::EvaluateClosure { ui_name, .. } => ui_name,
        }
    }
}

/// `ViewerPath` — `path` ListBase → Vec.
#[derive(Clone, Default)]
pub struct ViewerPath {
    pub path: Vec<ViewerPathElem>,
}

/// `BKE_viewer_path_init`.
pub fn bke_viewer_path_init(viewer_path: &mut ViewerPath) {
    viewer_path.path.clear();
}

/// `BKE_viewer_path_clear` — frees every element (drops here).
pub fn bke_viewer_path_clear(viewer_path: &mut ViewerPath) {
    viewer_path.path.clear();
}

/// `BKE_viewer_path_copy`.
pub fn bke_viewer_path_copy(dst: &mut ViewerPath, src: &ViewerPath) {
    dst.path = src.path.clone();
}

/// `BKE_viewer_path_equal`.
pub fn bke_viewer_path_equal(a: &ViewerPath, b: &ViewerPath, flag: ViewerPathEqualFlag) -> bool {
    if a.path.len() != b.path.len() {
        return false;
    }
    a.path
        .iter()
        .zip(b.path.iter())
        .all(|(ea, eb)| bke_viewer_path_elem_equal(ea, eb, flag))
}

/// `BKE_viewer_path_hash` — `id_session_uid` resolves an ID handle
/// to its `session_uid` (0 for null) like `typed_elem.id ?
/// typed_elem.id->session_uid : 0`.
pub fn bke_viewer_path_hash(
    viewer_path: &ViewerPath,
    id_session_uid: &dyn Fn(usize) -> u64,
) -> u64 {
    let mut hash = 0u64;
    for elem in &viewer_path.path {
        let elem_hash = bke_viewer_path_elem_hash(elem, id_session_uid);
        hash = get_default_hash(hash, &[elem_hash]);
    }
    hash
}

/// `BKE_viewer_path_foreach_id` — yield each referenced ID handle
/// (`IDWALK_CB_DIRECT_WEAK_LINK` elems only: `Id`, `EvaluateClosure`).
pub fn bke_viewer_path_foreach_id(viewer_path: &ViewerPath, cb: &mut dyn FnMut(usize)) {
    for elem in &viewer_path.path {
        match elem {
            ViewerPathElem::Id { id, .. } => cb(*id),
            ViewerPathElem::EvaluateClosure {
                source_node_tree, ..
            } => cb(*source_node_tree),
            _ => {}
        }
    }
}

/// `BKE_viewer_path_id_remap` — remap `Id` handles (`mappings.apply`).
pub fn bke_viewer_path_id_remap(
    viewer_path: &mut ViewerPath,
    remap: &dyn Fn(usize) -> usize,
) {
    for elem in &mut viewer_path.path {
        if let ViewerPathElem::Id { id, .. } = elem {
            *id = remap(*id);
        }
    }
}

/// `BKE_viewer_path_elem_new` — factory per type.
pub fn bke_viewer_path_elem_new(type_: ViewerPathElemType) -> ViewerPathElem {
    match type_ {
        VIEWER_PATH_ELEM_TYPE_ID => ViewerPathElem::Id {
            id: 0,
            ui_name: String::new(),
        },
        VIEWER_PATH_ELEM_TYPE_MODIFIER => ViewerPathElem::Modifier {
            modifier_uid: 0,
            ui_name: String::new(),
        },
        VIEWER_PATH_ELEM_TYPE_GROUP_NODE => ViewerPathElem::GroupNode {
            node_id: 0,
            ui_name: String::new(),
        },
        VIEWER_PATH_ELEM_TYPE_SIMULATION_ZONE => ViewerPathElem::SimulationZone {
            sim_output_node_id: 0,
            ui_name: String::new(),
        },
        VIEWER_PATH_ELEM_TYPE_VIEWER_NODE => ViewerPathElem::ViewerNode {
            node_id: 0,
            ui_name: String::new(),
        },
        VIEWER_PATH_ELEM_TYPE_REPEAT_ZONE => ViewerPathElem::RepeatZone {
            repeat_output_node_id: 0,
            iteration: 0,
            ui_name: String::new(),
        },
        VIEWER_PATH_ELEM_TYPE_FOREACH_GEOMETRY_ELEMENT_ZONE => {
            ViewerPathElem::ForeachGeometryElementZone {
                zone_output_node_id: 0,
                index: 0,
                ui_name: String::new(),
            }
        }
        VIEWER_PATH_ELEM_TYPE_EVALUATE_CLOSURE => ViewerPathElem::EvaluateClosure {
            evaluate_node_id: 0,
            source_output_node_id: 0,
            source_node_tree: 0,
            ui_name: String::new(),
        },
        _ => bke_viewer_path_elem_new(VIEWER_PATH_ELEM_TYPE_ID),
    }
}

/// `BKE_viewer_path_elem_new_id`.
pub fn bke_viewer_path_elem_new_id() -> ViewerPathElem {
    bke_viewer_path_elem_new(VIEWER_PATH_ELEM_TYPE_ID)
}

/// `BKE_viewer_path_elem_new_modifier`.
pub fn bke_viewer_path_elem_new_modifier() -> ViewerPathElem {
    bke_viewer_path_elem_new(VIEWER_PATH_ELEM_TYPE_MODIFIER)
}

/// `BKE_viewer_path_elem_new_group_node`.
pub fn bke_viewer_path_elem_new_group_node() -> ViewerPathElem {
    bke_viewer_path_elem_new(VIEWER_PATH_ELEM_TYPE_GROUP_NODE)
}

/// `BKE_viewer_path_elem_new_simulation_zone`.
pub fn bke_viewer_path_elem_new_simulation_zone() -> ViewerPathElem {
    bke_viewer_path_elem_new(VIEWER_PATH_ELEM_TYPE_SIMULATION_ZONE)
}

/// `BKE_viewer_path_elem_new_viewer_node`.
pub fn bke_viewer_path_elem_new_viewer_node() -> ViewerPathElem {
    bke_viewer_path_elem_new(VIEWER_PATH_ELEM_TYPE_VIEWER_NODE)
}

/// `BKE_viewer_path_elem_new_repeat_zone`.
pub fn bke_viewer_path_elem_new_repeat_zone() -> ViewerPathElem {
    bke_viewer_path_elem_new(VIEWER_PATH_ELEM_TYPE_REPEAT_ZONE)
}

/// `BKE_viewer_path_elem_new_foreach_geometry_element_zone`.
pub fn bke_viewer_path_elem_new_foreach_geometry_element_zone() -> ViewerPathElem {
    bke_viewer_path_elem_new(VIEWER_PATH_ELEM_TYPE_FOREACH_GEOMETRY_ELEMENT_ZONE)
}

/// `BKE_viewer_path_elem_new_evaluate_closure`.
pub fn bke_viewer_path_elem_new_evaluate_closure() -> ViewerPathElem {
    bke_viewer_path_elem_new(VIEWER_PATH_ELEM_TYPE_EVALUATE_CLOSURE)
}

/// `BKE_viewer_path_elem_copy` — `Clone` covers the `ui_name`
/// strdup + per-type fields.
pub fn bke_viewer_path_elem_copy(src: &ViewerPathElem) -> ViewerPathElem {
    src.clone()
}

/// `BKE_viewer_path_elem_equal`.
pub fn bke_viewer_path_elem_equal(
    a: &ViewerPathElem,
    b: &ViewerPathElem,
    flag: ViewerPathEqualFlag,
) -> bool {
    if a.type_() != b.type_() {
        return false;
    }
    if flag & VIEWER_PATH_EQUAL_FLAG_CONSIDER_UI_NAME != 0
        && a.ui_name() != b.ui_name()
    {
        return false;
    }
    let ignore_iteration = flag & VIEWER_PATH_EQUAL_FLAG_IGNORE_ITERATION != 0;
    match (a, b) {
        (ViewerPathElem::Id { id: a_id, .. }, ViewerPathElem::Id { id: b_id, .. }) => {
            a_id == b_id
        }
        (
            ViewerPathElem::Modifier { modifier_uid: a_uid, .. },
            ViewerPathElem::Modifier { modifier_uid: b_uid, .. },
        ) => a_uid == b_uid,
        (
            ViewerPathElem::GroupNode { node_id: a_id, .. },
            ViewerPathElem::GroupNode { node_id: b_id, .. },
        ) => a_id == b_id,
        (
            ViewerPathElem::SimulationZone {
                sim_output_node_id: a_id,
                ..
            },
            ViewerPathElem::SimulationZone {
                sim_output_node_id: b_id,
                ..
            },
        ) => a_id == b_id,
        (
            ViewerPathElem::ViewerNode { node_id: a_id, .. },
            ViewerPathElem::ViewerNode { node_id: b_id, .. },
        ) => a_id == b_id,
        (
            ViewerPathElem::RepeatZone {
                repeat_output_node_id: a_id,
                iteration: a_it,
                ..
            },
            ViewerPathElem::RepeatZone {
                repeat_output_node_id: b_id,
                iteration: b_it,
                ..
            },
        ) => a_id == b_id && (ignore_iteration || a_it == b_it),
        (
            ViewerPathElem::ForeachGeometryElementZone {
                zone_output_node_id: a_id,
                index: a_index,
                ..
            },
            ViewerPathElem::ForeachGeometryElementZone {
                zone_output_node_id: b_id,
                index: b_index,
                ..
            },
        ) => a_id == b_id && (ignore_iteration || a_index == b_index),
        (
            ViewerPathElem::EvaluateClosure {
                source_output_node_id: a_src,
                evaluate_node_id: a_eval,
                source_node_tree: a_tree,
                ..
            },
            ViewerPathElem::EvaluateClosure {
                source_output_node_id: b_src,
                evaluate_node_id: b_eval,
                source_node_tree: b_tree,
                ..
            },
        ) => a_src == b_src && a_eval == b_eval && a_tree == b_tree,
        _ => false,
    }
}

/// `BKE_viewer_path_elem_hash`.
pub fn bke_viewer_path_elem_hash(
    elem: &ViewerPathElem,
    id_session_uid: &dyn Fn(usize) -> u64,
) -> u64 {
    let t = elem.type_() as u64;
    match elem {
        ViewerPathElem::Id { id, .. } => {
            get_default_hash(t, &[id_session_uid(*id)])
        }
        ViewerPathElem::Modifier { modifier_uid, .. } => {
            get_default_hash(t, &[*modifier_uid as u64])
        }
        ViewerPathElem::GroupNode { node_id, .. }
        | ViewerPathElem::ViewerNode { node_id, .. } => {
            get_default_hash(t, &[*node_id as u64])
        }
        ViewerPathElem::SimulationZone {
            sim_output_node_id, ..
        } => get_default_hash(t, &[*sim_output_node_id as u64]),
        ViewerPathElem::RepeatZone {
            repeat_output_node_id,
            iteration,
            ..
        } => get_default_hash(t, &[*repeat_output_node_id as u64, *iteration as u64]),
        ViewerPathElem::ForeachGeometryElementZone {
            zone_output_node_id,
            index,
            ..
        } => get_default_hash(t, &[*zone_output_node_id as u64, *index as u64]),
        ViewerPathElem::EvaluateClosure {
            evaluate_node_id,
            source_output_node_id,
            source_node_tree,
            ..
        } => get_default_hash(
            t,
            &[
                *evaluate_node_id as u64,
                *source_output_node_id as u64,
                id_session_uid(*source_node_tree),
            ],
        ),
    }
}
