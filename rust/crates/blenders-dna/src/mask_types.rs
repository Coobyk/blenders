//! Port of `DNA_mask_types.h` — `Mask`, `MaskLayer`, `MaskSpline`,
//! `MaskSplinePoint`, `MaskParent`, `MaskLayerShape`.
#![allow(non_camel_case_types)]
use crate::curve_types::BezTriple;
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

/// `MaskParentType`.
pub const MASK_PARENT_POINT_TRACK: i32 = 0;
pub const MASK_PARENT_PLANE_TRACK: i32 = 1;

/// `MaskSplineFlag`.
pub const MASK_SPLINE_SELECT: i16 = 1 << 0;
pub const MASK_SPLINE_CYCLIC: i16 = 1 << 1;
pub const MASK_SPLINE_NOFILL: i16 = 1 << 2;
pub const MASK_SPLINE_NOINTERSECT: i16 = 1 << 3;

/// `MaskSplineInterp`.
pub const MASK_SPLINE_INTERP_LINEAR: i8 = 1;
pub const MASK_SPLINE_INTERP_EASE: i8 = 2;

/// `MaskSplineOffset`.
pub const MASK_SPLINE_OFFSET_EVEN: i8 = 0;
pub const MASK_SPLINE_OFFSET_SMOOTH: i8 = 1;

/// `MaskLayerVisibility`.
pub const MASK_HIDE_VIEW: i8 = 1 << 0;
pub const MASK_HIDE_SELECT: i8 = 1 << 1;
pub const MASK_HIDE_RENDER: i8 = 1 << 2;

/// `MaskDrawFlag`.
pub const MASK_DRAWFLAG_SMOOTH_DEPRECATED: i8 = 1 << 0;
pub const MASK_DRAWFLAG_OVERLAY: i8 = 1 << 1;
pub const MASK_DRAWFLAG_SPLINE: i8 = 1 << 2;

/// `MaskDrawType`.
pub const MASK_DT_OUTLINE: i8 = 0;
pub const MASK_DT_DASH: i8 = 1;
pub const MASK_DT_BLACK: i8 = 2;
pub const MASK_DT_WHITE: i8 = 3;

/// `MaskOverlayMode`.
pub const MASK_OVERLAY_ALPHACHANNEL: i8 = 0;
pub const MASK_OVERLAY_COMBINED: i8 = 1;

/// `MaskLayerBlend`.
pub const MASK_BLEND_ADD: i8 = 0;
pub const MASK_BLEND_SUBTRACT: i8 = 1;
pub const MASK_BLEND_LIGHTEN: i8 = 2;
pub const MASK_BLEND_DARKEN: i8 = 3;
pub const MASK_BLEND_MUL: i8 = 4;
pub const MASK_BLEND_REPLACE: i8 = 5;
pub const MASK_BLEND_DIFFERENCE: i8 = 6;
pub const MASK_BLEND_MERGE_ADD: i8 = 7;
pub const MASK_BLEND_MERGE_SUBTRACT: i8 = 8;

/// `MaskLayerBlendFlag`.
pub const MASK_BLENDFLAG_INVERT: i8 = 1 << 0;

/// `MaskLayerFlag`.
pub const MASK_LAYERFLAG_LOCKED: i8 = 1 << 4;
pub const MASK_LAYERFLAG_SELECT: i8 = 1 << 5;
pub const MASK_LAYERFLAG_FILL_DISCRETE: i8 = 1 << 6;
pub const MASK_LAYERFLAG_FILL_OVERLAP: i8 = 1 << 7;

/// `MaskLayerShapeFlag`.
pub const MASK_SHAPE_SELECT: i8 = 1 << 0;

/// `MaskAnimFlag`.
pub const MASK_ANIMF_EXPAND: i32 = 1 << 4;

/// `MaskLayerFillSolverType`.
pub const MASK_FILL_SOLVER_SWEEP_LINE: i8 = 0;
pub const MASK_FILL_SOLVER_CDT: i8 = 1;

/// `Mask_Runtime`.
#[derive(Clone, Copy, Default)]
pub struct MaskRuntime {
    pub last_update: u64,
}

/// `Mask`.
#[derive(Clone)]
pub struct Mask {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `ListBaseT<MaskLayer>`.
    pub masklayers: ListBase,
    pub masklay_act: i32,
    pub masklay_tot: i32,
    pub sfra: i32,
    pub efra: i32,
    pub flag: i32,
    pub runtime: MaskRuntime,
}
impl Default for Mask {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            masklayers: ListBase::new(),
            masklay_act: 0,
            masklay_tot: 0,
            sfra: 0,
            efra: 0,
            flag: 0,
            runtime: MaskRuntime::default(),
        }
    }
}

/// `MaskParent`.
#[derive(Clone)]
pub struct MaskParent {
    pub id_type: i32,
    pub type_: i32,
    /// `ID *id`.
    pub id: usize,
    /// `char parent[64]`, `sub_parent[64]`.
    pub parent: [u8; 64],
    pub sub_parent: [u8; 64],
    pub parent_orig: [f32; 2],
    /// `float parent_corners_orig[4][2]`.
    pub parent_corners_orig: [[f32; 2]; 4],
}
impl Default for MaskParent {
    fn default() -> Self {
        Self {
            id_type: crate::id_enums::ID_MC as i32,
            type_: MASK_PARENT_POINT_TRACK,
            id: NIL,
            parent: [0; 64],
            sub_parent: [0; 64],
            parent_orig: [0.0; 2],
            parent_corners_orig: [[0.0; 2]; 4],
        }
    }
}

/// `MaskSplinePointUW`.
#[derive(Clone, Copy, Default)]
pub struct MaskSplinePointUw {
    pub u: f32,
    pub w: f32,
    pub flag: i32,
}

/// `MaskSplinePoint`.
#[derive(Clone)]
pub struct MaskSplinePoint {
    pub bezt: BezTriple,
    pub tot_uw: i32,
    /// `MaskSplinePointUW *uw` → Vec.
    pub uw: Vec<MaskSplinePointUw>,
    pub parent: MaskParent,
}
impl Default for MaskSplinePoint {
    fn default() -> Self {
        Self {
            bezt: BezTriple::default(),
            tot_uw: 0,
            uw: Vec::new(),
            parent: MaskParent::default(),
        }
    }
}

/// `MaskSpline`.
#[derive(Clone)]
pub struct MaskSpline {
    pub next: usize,
    pub prev: usize,
    pub flag: i16,
    pub offset_mode: i8,
    pub weight_interp: i8,
    pub tot_point: i32,
    /// `MaskSplinePoint *points` → Vec.
    pub points: Vec<MaskSplinePoint>,
    pub parent: MaskParent,
    /// `MaskSplineRuntime *runtime`.
    pub runtime: usize,
}
impl Default for MaskSpline {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            flag: 0,
            offset_mode: MASK_SPLINE_OFFSET_EVEN,
            weight_interp: 0,
            tot_point: 0,
            points: Vec::new(),
            parent: MaskParent::default(),
            runtime: NIL,
        }
    }
}

/// `MaskLayerShape`.
#[derive(Clone)]
pub struct MaskLayerShape {
    pub next: usize,
    pub prev: usize,
    /// `float *data` — shape vertex coords (`MaskLayerShapeElem` array).
    pub data: Vec<f32>,
    pub tot_vert: i32,
    pub frame: i32,
    pub flag: i8,
}
impl Default for MaskLayerShape {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            data: Vec::new(),
            tot_vert: 0,
            frame: 0,
            flag: 0,
        }
    }
}

/// `MaskLayer`.
#[derive(Clone)]
pub struct MaskLayer {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `ListBaseT<MaskSpline>`.
    pub splines: ListBase,
    /// `ListBaseT<MaskLayerShape>`.
    pub splines_shapes: ListBase,
    /// `MaskSpline *act_spline`.
    pub act_spline: usize,
    /// `MaskSplinePoint *act_point`.
    pub act_point: usize,
    pub alpha: f32,
    pub blend: i8,
    pub blend_flag: i8,
    pub falloff: i8,
    pub fill_solver: i8,
    pub flag: i8,
    pub visibility_flag: i8,
}
impl Default for MaskLayer {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            splines: ListBase::new(),
            splines_shapes: ListBase::new(),
            act_spline: NIL,
            act_point: NIL,
            alpha: 0.0,
            blend: MASK_BLEND_ADD,
            blend_flag: 0,
            falloff: 0,
            fill_solver: MASK_FILL_SOLVER_CDT,
            flag: 0,
            visibility_flag: 0,
        }
    }
}
