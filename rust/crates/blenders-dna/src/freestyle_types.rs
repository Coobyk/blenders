//! Port of `DNA_freestyle_types.h` — Freestyle line rendering config.

use crate::id::NIL;
use blenders_bli::listbase::ListBase;

/// `eFreestyleConfig_Flags`.
pub type FreestyleConfigFlags = i32;
pub const FREESTYLE_SUGGESTIVE_CONTOURS_FLAG: FreestyleConfigFlags = 1 << 0;
pub const FREESTYLE_RIDGES_AND_VALLEYS_FLAG: FreestyleConfigFlags = 1 << 1;
pub const FREESTYLE_MATERIAL_BOUNDARIES_FLAG: FreestyleConfigFlags = 1 << 2;
pub const FREESTYLE_FACE_SMOOTHNESS_FLAG: FreestyleConfigFlags = 1 << 3;
pub const FREESTYLE_CULLING: FreestyleConfigFlags = 1 << 5;
pub const FREESTYLE_VIEW_MAP_CACHE: FreestyleConfigFlags = 1 << 6;
pub const FREESTYLE_AS_RENDER_PASS: FreestyleConfigFlags = 1 << 7;

/// `eFreestyleControl_Mode`.
pub type FreestyleControlMode = i32;
pub const FREESTYLE_CONTROL_SCRIPT_MODE: FreestyleControlMode = 1;
pub const FREESTYLE_CONTROL_EDITOR_MODE: FreestyleControlMode = 2;

/// `eFreestyleLineSet_Flags`.
pub type FreestyleLineSetFlags = i32;
pub const FREESTYLE_LINESET_CURRENT: FreestyleLineSetFlags = 1 << 0;
pub const FREESTYLE_LINESET_ENABLED: FreestyleLineSetFlags = 1 << 1;
pub const FREESTYLE_LINESET_FE_NOT: FreestyleLineSetFlags = 1 << 2;
pub const FREESTYLE_LINESET_FE_AND: FreestyleLineSetFlags = 1 << 3;
pub const FREESTYLE_LINESET_GR_NOT: FreestyleLineSetFlags = 1 << 4;
pub const FREESTYLE_LINESET_FM_NOT: FreestyleLineSetFlags = 1 << 5;
pub const FREESTYLE_LINESET_FM_BOTH: FreestyleLineSetFlags = 1 << 6;

/// `eFreestyleLineSet_Selection`.
pub type FreestyleLineSetSelection = i32;
pub const FREESTYLE_SEL_VISIBILITY: FreestyleLineSetSelection = 1 << 0;
pub const FREESTYLE_SEL_EDGE_TYPES: FreestyleLineSetSelection = 1 << 1;
pub const FREESTYLE_SEL_GROUP: FreestyleLineSetSelection = 1 << 2;
pub const FREESTYLE_SEL_IMAGE_BORDER: FreestyleLineSetSelection = 1 << 3;
pub const FREESTYLE_SEL_FACE_MARK: FreestyleLineSetSelection = 1 << 4;

/// `eFreestyleLineSet_EdgeTypes`.
pub type FreestyleLineSetEdgeTypes = i32;
pub const FREESTYLE_FE_SILHOUETTE: FreestyleLineSetEdgeTypes = 1 << 0;
pub const FREESTYLE_FE_BORDER: FreestyleLineSetEdgeTypes = 1 << 1;
pub const FREESTYLE_FE_CREASE: FreestyleLineSetEdgeTypes = 1 << 2;
pub const FREESTYLE_FE_RIDGE_VALLEY: FreestyleLineSetEdgeTypes = 1 << 3;
pub const FREESTYLE_FE_SUGGESTIVE_CONTOUR: FreestyleLineSetEdgeTypes = 1 << 5;
pub const FREESTYLE_FE_MATERIAL_BOUNDARY: FreestyleLineSetEdgeTypes = 1 << 6;
pub const FREESTYLE_FE_CONTOUR: FreestyleLineSetEdgeTypes = 1 << 7;
pub const FREESTYLE_FE_EXTERNAL_CONTOUR: FreestyleLineSetEdgeTypes = 1 << 8;
pub const FREESTYLE_FE_EDGE_MARK: FreestyleLineSetEdgeTypes = 1 << 9;

/// `eFreestyleLineSet_QI` (short).
pub type FreestyleLineSetQi = i16;
pub const FREESTYLE_QI_VISIBLE: FreestyleLineSetQi = 1;
pub const FREESTYLE_QI_HIDDEN: FreestyleLineSetQi = 2;
pub const FREESTYLE_QI_RANGE: FreestyleLineSetQi = 3;

/// `eFreestyleRaycastingAlgorithm`.
pub type FreestyleRaycastingAlgorithm = i32;
pub const FREESTYLE_ALGO_REGULAR: FreestyleRaycastingAlgorithm = 1;
pub const FREESTYLE_ALGO_FAST: FreestyleRaycastingAlgorithm = 2;
pub const FREESTYLE_ALGO_VERYFAST: FreestyleRaycastingAlgorithm = 3;
pub const FREESTYLE_ALGO_CULLED_ADAPTIVE_TRADITIONAL: FreestyleRaycastingAlgorithm = 4;
pub const FREESTYLE_ALGO_ADAPTIVE_TRADITIONAL: FreestyleRaycastingAlgorithm = 5;
pub const FREESTYLE_ALGO_CULLED_ADAPTIVE_CUMULATIVE: FreestyleRaycastingAlgorithm = 6;
pub const FREESTYLE_ALGO_ADAPTIVE_CUMULATIVE: FreestyleRaycastingAlgorithm = 7;

/// `FreestyleLineSet`.
#[derive(Clone)]
pub struct FreestyleLineSet {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub flags: FreestyleLineSetFlags,
    pub selection: FreestyleLineSetSelection,
    pub qi: FreestyleLineSetQi,
    pub qi_start: i32,
    pub qi_end: i32,
    pub edge_types: FreestyleLineSetEdgeTypes,
    pub exclude_edge_types: FreestyleLineSetEdgeTypes,
    /// `Collection *group`.
    pub group: usize,
    /// `FreestyleLineStyle *linestyle`.
    pub linestyle: usize,
}
impl Default for FreestyleLineSet {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            flags: 0,
            selection: 0,
            qi: 0,
            qi_start: 0,
            qi_end: 0,
            edge_types: 0,
            exclude_edge_types: 0,
            group: NIL,
            linestyle: NIL,
        }
    }
}

/// `FreestyleModuleConfig`.
#[derive(Clone)]
pub struct FreestyleModuleConfig {
    pub next: usize,
    pub prev: usize,
    /// `Text *script`.
    pub script: usize,
    pub is_displayed: i16,
}
impl Default for FreestyleModuleConfig {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            script: NIL,
            is_displayed: 0,
        }
    }
}

/// `FreestyleConfig`.
#[derive(Clone, Default)]
pub struct FreestyleConfig {
    /// `ListBaseT<FreestyleModuleConfig>`.
    pub modules: ListBase,
    pub mode: FreestyleControlMode,
    /// `DNA_DEPRECATED int raycasting_algorithm`.
    pub raycasting_algorithm: i32,
    pub flags: FreestyleConfigFlags,
    pub sphere_radius: f32,
    pub dkr_epsilon: f32,
    pub crease_angle: f32,
    /// `ListBaseT<FreestyleLineSet>`.
    pub linesets: ListBase,
}
