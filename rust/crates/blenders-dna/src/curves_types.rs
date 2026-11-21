//! Port of `DNA_curves_types.h` — new hair/curves geometry.

use crate::attribute_types::AttributeStorage;
use crate::customdata_types::CustomData;
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

/// `CurveType` (int8).
pub type CurveType = i8;
pub const CURVE_TYPE_CATMULL_ROM: CurveType = 0;
pub const CURVE_TYPE_POLY: CurveType = 1;
pub const CURVE_TYPE_BEZIER: CurveType = 2;
pub const CURVE_TYPE_NURBS: CurveType = 3;
pub const CURVE_TYPES_NUM: i32 = 4;

/// `HandleType` (int8).
pub type HandleType = i8;
pub const BEZIER_HANDLE_FREE: HandleType = 0;
pub const BEZIER_HANDLE_AUTO: HandleType = 1;
pub const BEZIER_HANDLE_VECTOR: HandleType = 2;
pub const BEZIER_HANDLE_ALIGN: HandleType = 3;
pub const BEZIER_HANDLES_NUM: i32 = 4;

/// `KnotsMode` (int8).
pub type KnotsMode = i8;
pub const NURBS_KNOT_MODE_NORMAL: KnotsMode = 0;
pub const NURBS_KNOT_MODE_ENDPOINT: KnotsMode = 1;
pub const NURBS_KNOT_MODE_BEZIER: KnotsMode = 2;
pub const NURBS_KNOT_MODE_ENDPOINT_BEZIER: KnotsMode = 3;
pub const NURBS_KNOT_MODE_CUSTOM: KnotsMode = 4;

/// `NormalMode` (int8).
pub type NormalMode = i8;
pub const NORMAL_MODE_MINIMUM_TWIST: NormalMode = 0;
pub const NORMAL_MODE_Z_UP: NormalMode = 1;
pub const NORMAL_MODE_FREE: NormalMode = 2;

/// `eCurves_Flag`.
pub type CurvesFlag = i32;
pub const HA_DS_EXPAND: CurvesFlag = 1 << 0;
pub const CV_SCULPT_COLLISION_ENABLED: CurvesFlag = 1 << 1;

/// `eCurvesSymmetryType` (char).
pub type CurvesSymmetryType = i8;
pub const CURVES_SYMMETRY_X: CurvesSymmetryType = 1 << 0;
pub const CURVES_SYMMETRY_Y: CurvesSymmetryType = 1 << 1;
pub const CURVES_SYMMETRY_Z: CurvesSymmetryType = 1 << 2;

/// `CurvesGeometry`.
#[derive(Clone)]
pub struct CurvesGeometry {
    /// `int *curve_offsets` → Vec.
    pub curve_offsets: Vec<i32>,
    pub attribute_storage: AttributeStorage,
    pub point_data: CustomData,
    pub curve_data_legacy: CustomData,
    pub point_num: i32,
    pub curve_num: i32,
    /// `ListBaseT<bDeformGroup> vertex_group_names`.
    pub vertex_group_names: ListBase,
    pub vertex_group_active_index: i32,
    pub attributes_active_index: i32,
    /// `bke::CurvesGeometryRuntime *runtime`.
    pub runtime: usize,
    /// `float *custom_knots` → Vec.
    pub custom_knots: Vec<f32>,
    pub custom_knot_num: i32,
}
impl Default for CurvesGeometry {
    fn default() -> Self {
        Self {
            curve_offsets: Vec::new(),
            attribute_storage: AttributeStorage::default(),
            point_data: CustomData::default(),
            curve_data_legacy: CustomData::default(),
            point_num: 0,
            curve_num: 0,
            vertex_group_names: ListBase::default(),
            vertex_group_active_index: 0,
            attributes_active_index: -1,
            runtime: NIL,
            custom_knots: Vec::new(),
            custom_knot_num: 0,
        }
    }
}

/// `Curves` — hair curves ID block.
#[derive(Clone)]
pub struct Curves {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub geometry: CurvesGeometry,
    pub flag: CurvesFlag,
    pub attributes_active_index_legacy: i32,
    /// `Material **mat` → Vec.
    pub mat: Vec<usize>,
    pub totcol: i16,
    pub symmetry: CurvesSymmetryType,
    pub selection_domain: i8,
    /// `Object *surface`.
    pub surface: usize,
    /// `char *surface_uv_map`.
    pub surface_uv_map: Option<String>,
    pub surface_collision_distance: f32,
    /// `draw::CurvesBatchCache *batch_cache`.
    pub batch_cache: usize,
}
impl Default for Curves {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            geometry: CurvesGeometry::default(),
            flag: 0,
            attributes_active_index_legacy: 0,
            mat: Vec::new(),
            totcol: 0,
            symmetry: 0,
            selection_domain: 0,
            surface: NIL,
            surface_uv_map: None,
            surface_collision_distance: 0.005,
            batch_cache: NIL,
        }
    }
}

/// `CURVES_MATERIAL_NR`.
pub const CURVES_MATERIAL_NR: i32 = 1;
