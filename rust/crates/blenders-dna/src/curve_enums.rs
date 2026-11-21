//! Port of `DNA_curve_enums.h`.
#![allow(non_camel_case_types)]

pub const MAXTEXTBOX: usize = 256;

/// `eCurveTexspaceFlag`.
pub const CU_TEXSPACE_FLAG_AUTO: i8 = 1 << 0;
pub const CU_TEXSPACE_FLAG_AUTO_EVALUATED: i8 = 1 << 1;

/// `eCurveFlag`.
pub type CurveFlag = i32;
pub const CU_3D: CurveFlag = 1 << 0;
pub const CU_FRONT: CurveFlag = 1 << 1;
pub const CU_BACK: CurveFlag = 1 << 2;
pub const CU_PATH: CurveFlag = 1 << 3;
pub const CU_FOLLOW: CurveFlag = 1 << 4;
pub const CU_PATH_CLAMP: CurveFlag = 1 << 5;
pub const CU_DEFORM_BOUNDS_OFF: CurveFlag = 1 << 6;
pub const CU_STRETCH: CurveFlag = 1 << 7;
pub const CU_FAST: CurveFlag = 1 << 9;
pub const CU_DS_EXPAND: CurveFlag = 1 << 11;
pub const CU_PATH_RADIUS: CurveFlag = 1 << 12;
pub const CU_FILL_CAPS: CurveFlag = 1 << 14;
pub const CU_MAP_TAPER: CurveFlag = 1 << 15;

/// `eCurveTwistMode`.
pub type CurveTwistMode = i16;
pub const CU_TWIST_Z_UP: CurveTwistMode = 0;
pub const CU_TWIST_MINIMUM: CurveTwistMode = 3;
pub const CU_TWIST_TANGENT: CurveTwistMode = 4;

/// `eCurveBevfacMapping`.
pub type CurveBevfacMapping = i8;
pub const CU_BEVFAC_MAP_RESOLU: CurveBevfacMapping = 0;
pub const CU_BEVFAC_MAP_SEGMENT: CurveBevfacMapping = 1;
pub const CU_BEVFAC_MAP_SPLINE: CurveBevfacMapping = 2;

/// `eCurveSpaceMode`.
pub type CurveSpaceMode = i8;
pub const CU_ALIGN_X_LEFT: CurveSpaceMode = 0;
pub const CU_ALIGN_X_MIDDLE: CurveSpaceMode = 1;
pub const CU_ALIGN_X_RIGHT: CurveSpaceMode = 2;
pub const CU_ALIGN_X_JUSTIFY: CurveSpaceMode = 3;
pub const CU_ALIGN_X_FLUSH: CurveSpaceMode = 4;

/// `eCurveAlignY`.
pub type CurveAlignY = i8;
pub const CU_ALIGN_Y_TOP_BASELINE: CurveAlignY = 0;
pub const CU_ALIGN_Y_TOP: CurveAlignY = 1;
pub const CU_ALIGN_Y_CENTER: CurveAlignY = 2;
pub const CU_ALIGN_Y_BOTTOM_BASELINE: CurveAlignY = 3;
pub const CU_ALIGN_Y_BOTTOM: CurveAlignY = 4;

/// `eCurveBevelMode`.
pub type CurveBevelMode = i8;
pub const CU_BEV_MODE_ROUND: CurveBevelMode = 0;
pub const CU_BEV_MODE_OBJECT: CurveBevelMode = 1;
pub const CU_BEV_MODE_CURVE_PROFILE: CurveBevelMode = 2;

/// `eCurveTaperRadiusMode`.
pub type CurveTaperRadiusMode = i8;
pub const CU_TAPER_RADIUS_OVERRIDE: CurveTaperRadiusMode = 0;
pub const CU_TAPER_RADIUS_MULTIPLY: CurveTaperRadiusMode = 1;
pub const CU_TAPER_RADIUS_ADD: CurveTaperRadiusMode = 2;

/// `CurveFillSolverType`.
pub type CurveFillSolverType = i8;
pub const CU_FILL_SOLVER_SWEEP_LINE: CurveFillSolverType = 0;
pub const CU_FILL_SOLVER_CDT: CurveFillSolverType = 1;

/// `CurveFillRuleType`.
pub type CurveFillRuleType = i8;
pub const CU_FILL_RULE_EVEN_ODD: CurveFillRuleType = 0;
pub const CU_FILL_RULE_NONZERO: CurveFillRuleType = 1;

/// `eCurveOverflow`.
pub type CurveOverflow = i8;
pub const CU_OVERFLOW_NONE: CurveOverflow = 0;
pub const CU_OVERFLOW_SCALE: CurveOverflow = 1;
pub const CU_OVERFLOW_TRUNCATE: CurveOverflow = 2;

/// `eNurbFlag`.
pub const CU_SMOOTH: i16 = 1 << 0;

/// `eNurbType`.
pub type NurbType = i16;
pub const CU_POLY: NurbType = 0;
pub const CU_BEZIER: NurbType = 1;
pub const CU_NURBS: NurbType = 4;
pub const CU_TYPE: NurbType = CU_POLY | CU_BEZIER | CU_NURBS;
pub const CU_PRIMITIVE: NurbType = 0xF00;
pub const CU_PRIM_CURVE: NurbType = 0x100;
pub const CU_PRIM_CIRCLE: NurbType = 0x200;
pub const CU_PRIM_PATCH: NurbType = 0x300;
pub const CU_PRIM_TUBE: NurbType = 0x400;
pub const CU_PRIM_SPHERE: NurbType = 0x500;
pub const CU_PRIM_DONUT: NurbType = 0x600;
pub const CU_PRIM_PATH: NurbType = 0x700;

/// `eNurbKnotFlag`.
pub const CU_NURB_CYCLIC: i16 = 1 << 0;
pub const CU_NURB_ENDPOINT: i16 = 1 << 1;
pub const CU_NURB_BEZIER: i16 = 1 << 2;
pub const CU_NURB_CUSTOM: i16 = 1 << 3;

pub const CU_ACT_NONE: i32 = -1;

/// `eBezTriple_Flag`.
pub const BEZT_FLAG_SELECT: u8 = 1 << 0;
pub const BEZT_FLAG_TEMP_TAG: u8 = 1 << 1;
pub const BEZT_FLAG_IGNORE_TAG: u8 = 1 << 2;

/// `eBezTriple_Handle`.
pub type BezTripleHandle = u8;
pub const HD_FREE: BezTripleHandle = 0;
pub const HD_AUTO: BezTripleHandle = 1;
pub const HD_VECT: BezTripleHandle = 2;
pub const HD_ALIGN: BezTripleHandle = 3;
pub const HD_AUTO_ANIM: BezTripleHandle = 4;
pub const HD_ALIGN_DOUBLESIDE: BezTripleHandle = 5;

/// `eBezTriple_Auto_Type`.
pub type BezTripleAutoType = i8;
pub const HD_AUTOTYPE_NORMAL: BezTripleAutoType = 0;
pub const HD_AUTOTYPE_LOCKED_FINAL: BezTripleAutoType = 1;

/// `eBezTriple_Interpolation`.
pub type BezTripleInterpolation = i8;
pub const BEZT_IPO_CONST: BezTripleInterpolation = 0;
pub const BEZT_IPO_LIN: BezTripleInterpolation = 1;
pub const BEZT_IPO_BEZ: BezTripleInterpolation = 2;
pub const BEZT_IPO_BACK: BezTripleInterpolation = 3;
pub const BEZT_IPO_BOUNCE: BezTripleInterpolation = 4;
pub const BEZT_IPO_CIRC: BezTripleInterpolation = 5;
pub const BEZT_IPO_CUBIC: BezTripleInterpolation = 6;
pub const BEZT_IPO_ELASTIC: BezTripleInterpolation = 7;
pub const BEZT_IPO_EXPO: BezTripleInterpolation = 8;
pub const BEZT_IPO_QUAD: BezTripleInterpolation = 9;
pub const BEZT_IPO_QUART: BezTripleInterpolation = 10;
pub const BEZT_IPO_QUINT: BezTripleInterpolation = 11;
pub const BEZT_IPO_SINE: BezTripleInterpolation = 12;

/// `eBezTriple_Easing`.
pub type BezTripleEasing = i8;
pub const BEZT_IPO_EASE_AUTO: BezTripleEasing = 0;
pub const BEZT_IPO_EASE_IN: BezTripleEasing = 1;
pub const BEZT_IPO_EASE_OUT: BezTripleEasing = 2;
pub const BEZT_IPO_EASE_IN_OUT: BezTripleEasing = 3;

/// `eBezTriple_KeyframeType`.
pub type BezTripleKeyframeType = i8;
pub const BEZT_KEYTYPE_KEYFRAME: BezTripleKeyframeType = 0;
pub const BEZT_KEYTYPE_EXTREME: BezTripleKeyframeType = 1;
pub const BEZT_KEYTYPE_BREAKDOWN: BezTripleKeyframeType = 2;
pub const BEZT_KEYTYPE_JITTER: BezTripleKeyframeType = 3;
pub const BEZT_KEYTYPE_MOVEHOLD: BezTripleKeyframeType = 4;
pub const BEZT_KEYTYPE_GENERATED: BezTripleKeyframeType = 5;

/// `eCharInfoFlag`.
pub const CU_CHINFO_BOLD: i8 = 1 << 0;
pub const CU_CHINFO_ITALIC: i8 = 1 << 1;
pub const CU_CHINFO_UNDERLINE: i8 = 1 << 2;
pub const CU_CHINFO_UNUSED_3: i8 = 1 << 3;
pub const CU_CHINFO_SMALLCAPS: i8 = 1 << 4;
pub const CU_CHINFO_UNUSED_5: i8 = 1 << 5;
pub const CU_CHINFO_UNUSED_6: i8 = 1 << 6;
pub const CU_CHINFO_STYLE_ALL: i8 =
    CU_CHINFO_BOLD | CU_CHINFO_ITALIC | CU_CHINFO_UNDERLINE | CU_CHINFO_SMALLCAPS;

pub const KEY_CU_EASE: i32 = 3;
pub const SURF_SEEN: i32 = 1 << 2;
