//! Port of `DNA_linestyle_types.h` — Freestyle line styles + modifiers.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;
use crate::texture_types::TEX_PR_TEXTURE;
/// `eLineStyleModifier_Type`.
pub type LinestylemodifierType = i32;
pub const LS_MODIFIER_ALONG_STROKE: LinestylemodifierType = 1;
pub const LS_MODIFIER_DISTANCE_FROM_CAMERA: LinestylemodifierType = 2;
pub const LS_MODIFIER_DISTANCE_FROM_OBJECT: LinestylemodifierType = 3;
pub const LS_MODIFIER_MATERIAL: LinestylemodifierType = 4;
pub const LS_MODIFIER_SAMPLING: LinestylemodifierType = 5;
pub const LS_MODIFIER_BEZIER_CURVE: LinestylemodifierType = 6;
pub const LS_MODIFIER_SINUS_DISPLACEMENT: LinestylemodifierType = 7;
pub const LS_MODIFIER_SPATIAL_NOISE: LinestylemodifierType = 8;
pub const LS_MODIFIER_PERLIN_NOISE_1D: LinestylemodifierType = 9;
pub const LS_MODIFIER_PERLIN_NOISE_2D: LinestylemodifierType = 10;
pub const LS_MODIFIER_BACKBONE_STRETCHER: LinestylemodifierType = 11;
pub const LS_MODIFIER_TIP_REMOVER: LinestylemodifierType = 12;
pub const LS_MODIFIER_CALLIGRAPHY: LinestylemodifierType = 13;
pub const LS_MODIFIER_POLYGONIZATION: LinestylemodifierType = 14;
pub const LS_MODIFIER_GUIDING_LINES: LinestylemodifierType = 15;
pub const LS_MODIFIER_BLUEPRINT: LinestylemodifierType = 16;
pub const LS_MODIFIER_2D_OFFSET: LinestylemodifierType = 17;
pub const LS_MODIFIER_2D_TRANSFORM: LinestylemodifierType = 18;
pub const LS_MODIFIER_TANGENT: LinestylemodifierType = 19;
pub const LS_MODIFIER_NOISE: LinestylemodifierType = 20;
pub const LS_MODIFIER_CREASE_ANGLE: LinestylemodifierType = 21;
pub const LS_MODIFIER_SIMPLIFICATION: LinestylemodifierType = 22;
pub const LS_MODIFIER_CURVATURE_3D: LinestylemodifierType = 23;
pub const LS_MODIFIER_NUM: LinestylemodifierType = 24;

/// `eLineStyleModifier_Flag`.
pub type LinestylemodifierFlag = i32;
pub const LS_MODIFIER_ENABLED: LinestylemodifierFlag = 1;
pub const LS_MODIFIER_EXPANDED: LinestylemodifierFlag = 2;

/// `eLineStyleColorModifier_Flag`.
pub type LinestylecolormodifierFlag = i32;
pub const LS_MODIFIER_USE_RAMP: LinestylecolormodifierFlag = 1;

/// `eLineStyleAlphaThicknessModifier_Flag`.
pub type LinestylealphathicknessmodifierFlag = i32;
pub const LS_MODIFIER_USE_CURVE: LinestylealphathicknessmodifierFlag = 1;
pub const LS_MODIFIER_INVERT: LinestylealphathicknessmodifierFlag = 2;

/// `eLineStyleThicknessNoise_Flag`.
pub type LinestylethicknessnoiseFlag = i32;
pub const LS_THICKNESS_ASYMMETRIC: LinestylethicknessnoiseFlag = 1;

/// `eLineStyleBlend`.
pub type Linestyleblend = i32;
pub const LS_VALUE_BLEND: Linestyleblend = 0;
pub const LS_VALUE_ADD: Linestyleblend = 1;
pub const LS_VALUE_MULT: Linestyleblend = 2;
pub const LS_VALUE_SUB: Linestyleblend = 3;
pub const LS_VALUE_DIV: Linestyleblend = 4;
pub const LS_VALUE_DIFF: Linestyleblend = 5;
pub const LS_VALUE_MIN: Linestyleblend = 6;
pub const LS_VALUE_MAX: Linestyleblend = 7;

/// `eLineStyleMaterialAttr`.
pub type Linestylematerialattr = i32;
pub const LS_MODIFIER_MATERIAL_DIFF: Linestylematerialattr = 1;
pub const LS_MODIFIER_MATERIAL_DIFF_R: Linestylematerialattr = 2;
pub const LS_MODIFIER_MATERIAL_DIFF_G: Linestylematerialattr = 3;
pub const LS_MODIFIER_MATERIAL_DIFF_B: Linestylematerialattr = 4;
pub const LS_MODIFIER_MATERIAL_SPEC: Linestylematerialattr = 5;
pub const LS_MODIFIER_MATERIAL_SPEC_R: Linestylematerialattr = 6;
pub const LS_MODIFIER_MATERIAL_SPEC_G: Linestylematerialattr = 7;
pub const LS_MODIFIER_MATERIAL_SPEC_B: Linestylematerialattr = 8;
pub const LS_MODIFIER_MATERIAL_SPEC_HARD: Linestylematerialattr = 9;
pub const LS_MODIFIER_MATERIAL_ALPHA: Linestylematerialattr = 10;
pub const LS_MODIFIER_MATERIAL_LINE: Linestylematerialattr = 11;
pub const LS_MODIFIER_MATERIAL_LINE_R: Linestylematerialattr = 12;
pub const LS_MODIFIER_MATERIAL_LINE_G: Linestylematerialattr = 13;
pub const LS_MODIFIER_MATERIAL_LINE_B: Linestylematerialattr = 14;
pub const LS_MODIFIER_MATERIAL_LINE_A: Linestylematerialattr = 15;

/// `eLineStyleGeomSpatialNoise_Flag`.
pub type LinestylegeomspatialnoiseFlag = i32;
pub const LS_MODIFIER_SPATIAL_NOISE_SMOOTH: LinestylegeomspatialnoiseFlag = 1;
pub const LS_MODIFIER_SPATIAL_NOISE_PURERANDOM: LinestylegeomspatialnoiseFlag = 2;

/// `eLineStyleGeomBlueprint_Flag`.
pub type LinestylegeomblueprintFlag = i32;
pub const LS_MODIFIER_BLUEPRINT_CIRCLES: LinestylegeomblueprintFlag = 1;
pub const LS_MODIFIER_BLUEPRINT_ELLIPSES: LinestylegeomblueprintFlag = 2;
pub const LS_MODIFIER_BLUEPRINT_SQUARES: LinestylegeomblueprintFlag = 4;

/// `eLineStyleGeom2DTransform_Pivot`.
pub type Linestylegeom2dtransformPivot = i32;
pub const LS_MODIFIER_2D_TRANSFORM_PIVOT_CENTER: Linestylegeom2dtransformPivot = 1;
pub const LS_MODIFIER_2D_TRANSFORM_PIVOT_START: Linestylegeom2dtransformPivot = 2;
pub const LS_MODIFIER_2D_TRANSFORM_PIVOT_END: Linestylegeom2dtransformPivot = 3;
pub const LS_MODIFIER_2D_TRANSFORM_PIVOT_PARAM: Linestylegeom2dtransformPivot = 4;
pub const LS_MODIFIER_2D_TRANSFORM_PIVOT_ABSOLUTE: Linestylegeom2dtransformPivot = 5;

/// `eLineStyle_Panel`.
pub type LinestylePanel = i32;
pub const LS_PANEL_STROKES: LinestylePanel = 1;
pub const LS_PANEL_COLOR: LinestylePanel = 2;
pub const LS_PANEL_ALPHA: LinestylePanel = 3;
pub const LS_PANEL_THICKNESS: LinestylePanel = 4;
pub const LS_PANEL_GEOMETRY: LinestylePanel = 5;
pub const LS_PANEL_TEXTURE: LinestylePanel = 6;
pub const LS_PANEL_MISC: LinestylePanel = 7;

/// `eLineStyle_Flag`.
pub type LinestyleFlag = i32;
pub const LS_DS_EXPAND: LinestyleFlag = 1 << 0;
pub const LS_SAME_OBJECT: LinestyleFlag = 1 << 1;
pub const LS_DASHED_LINE: LinestyleFlag = 1 << 2;
pub const LS_MATERIAL_BOUNDARY: LinestyleFlag = 1 << 3;
pub const LS_MIN_2D_LENGTH: LinestyleFlag = 1 << 4;
pub const LS_MAX_2D_LENGTH: LinestyleFlag = 1 << 5;
pub const LS_NO_CHAINING: LinestyleFlag = 1 << 6;
pub const LS_MIN_2D_ANGLE: LinestyleFlag = 1 << 7;
pub const LS_MAX_2D_ANGLE: LinestyleFlag = 1 << 8;
pub const LS_SPLIT_LENGTH: LinestyleFlag = 1 << 9;
pub const LS_SPLIT_PATTERN: LinestyleFlag = 1 << 10;
pub const LS_NO_SORTING: LinestyleFlag = 1 << 11;
pub const LS_REVERSE_ORDER: LinestyleFlag = 1 << 12;
pub const LS_TEXTURE: LinestyleFlag = 1 << 13;
pub const LS_CHAIN_COUNT: LinestyleFlag = 1 << 14;

/// `eLineStyle_Chaining`.
pub type LinestyleChaining = i32;
pub const LS_CHAINING_PLAIN: LinestyleChaining = 1;
pub const LS_CHAINING_SKETCHY: LinestyleChaining = 2;

/// `eLineStyle_Caps`.
pub type LinestyleCaps = i32;
pub const LS_CAPS_BUTT: LinestyleCaps = 1;
pub const LS_CAPS_ROUND: LinestyleCaps = 2;
pub const LS_CAPS_SQUARE: LinestyleCaps = 3;

/// `eLineStyle_ThicknessPosition`.
pub type LinestyleThicknessposition = i32;
pub const LS_THICKNESS_CENTER: LinestyleThicknessposition = 1;
pub const LS_THICKNESS_INSIDE: LinestyleThicknessposition = 2;
pub const LS_THICKNESS_OUTSIDE: LinestyleThicknessposition = 3;
pub const LS_THICKNESS_RELATIVE: LinestyleThicknessposition = 4;

/// `eLineStyle_SortKey`.
pub type LinestyleSortkey = i32;
pub const LS_SORT_KEY_DISTANCE_FROM_CAMERA: LinestyleSortkey = 1;
pub const LS_SORT_KEY_2D_LENGTH: LinestyleSortkey = 2;
pub const LS_SORT_KEY_PROJECTED_X: LinestyleSortkey = 3;
pub const LS_SORT_KEY_PROJECTED_Y: LinestyleSortkey = 4;

/// `eLineStyle_IntegrationType`.
pub type LinestyleIntegrationtype = i32;
pub const LS_INTEGRATION_MEAN: LinestyleIntegrationtype = 1;
pub const LS_INTEGRATION_MIN: LinestyleIntegrationtype = 2;
pub const LS_INTEGRATION_MAX: LinestyleIntegrationtype = 3;
pub const LS_INTEGRATION_FIRST: LinestyleIntegrationtype = 4;
pub const LS_INTEGRATION_LAST: LinestyleIntegrationtype = 5;

/// `LineStyleModifier`.
#[derive(Clone)]
pub struct LineStyleModifier {
    /// `LineStyleModifier *next`
    pub next: usize,
    /// `LineStyleModifier *prev`
    pub prev: usize,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `eLineStyleModifier_Type type`
    pub type_: LinestylemodifierType,
    /// `float influence`
    pub influence: f32,
    /// `eLineStyleModifier_Flag flags`
    pub flags: LinestylemodifierFlag,
    /// `eLineStyleBlend blend`
    pub blend: Linestyleblend,
}
impl Default for LineStyleModifier {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            type_: Default::default(),
            influence: 0.0,
            flags: Default::default(),
            blend: LS_VALUE_BLEND,
        }
    }
}

/// `LineStyleColorModifier_AlongStroke`.
#[derive(Clone)]
pub struct LineStyleColorModifier_AlongStroke {
    pub modifier: LineStyleModifier,
    /// `ColorBand *color_ramp`
    pub color_ramp: usize,
}
impl Default for LineStyleColorModifier_AlongStroke {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            color_ramp: NIL,
        }
    }
}

/// `LineStyleAlphaModifier_AlongStroke`.
#[derive(Clone)]
pub struct LineStyleAlphaModifier_AlongStroke {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
}
impl Default for LineStyleAlphaModifier_AlongStroke {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
        }
    }
}

/// `LineStyleThicknessModifier_AlongStroke`.
#[derive(Clone)]
pub struct LineStyleThicknessModifier_AlongStroke {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float value_min`
    pub value_min: f32,
    /// `float value_max`
    pub value_max: f32,
}
impl Default for LineStyleThicknessModifier_AlongStroke {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            value_min: 0.0,
            value_max: 0.0,
        }
    }
}

/// `LineStyleColorModifier_DistanceFromCamera`.
#[derive(Clone)]
pub struct LineStyleColorModifier_DistanceFromCamera {
    pub modifier: LineStyleModifier,
    /// `ColorBand *color_ramp`
    pub color_ramp: usize,
    /// `float range_min`
    pub range_min: f32,
    /// `float range_max`
    pub range_max: f32,
}
impl Default for LineStyleColorModifier_DistanceFromCamera {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            color_ramp: NIL,
            range_min: 0.0,
            range_max: 0.0,
        }
    }
}

/// `LineStyleAlphaModifier_DistanceFromCamera`.
#[derive(Clone)]
pub struct LineStyleAlphaModifier_DistanceFromCamera {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float range_min`
    pub range_min: f32,
    /// `float range_max`
    pub range_max: f32,
}
impl Default for LineStyleAlphaModifier_DistanceFromCamera {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            range_min: 0.0,
            range_max: 0.0,
        }
    }
}

/// `LineStyleThicknessModifier_DistanceFromCamera`.
#[derive(Clone)]
pub struct LineStyleThicknessModifier_DistanceFromCamera {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float range_min`
    pub range_min: f32,
    /// `float range_max`
    pub range_max: f32,
    /// `float value_min`
    pub value_min: f32,
    /// `float value_max`
    pub value_max: f32,
}
impl Default for LineStyleThicknessModifier_DistanceFromCamera {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            range_min: 0.0,
            range_max: 0.0,
            value_min: 0.0,
            value_max: 0.0,
        }
    }
}

/// `LineStyleColorModifier_DistanceFromObject`.
#[derive(Clone)]
pub struct LineStyleColorModifier_DistanceFromObject {
    pub modifier: LineStyleModifier,
    /// `Object *target`
    pub target: usize,
    /// `ColorBand *color_ramp`
    pub color_ramp: usize,
    /// `float range_min`
    pub range_min: f32,
    /// `float range_max`
    pub range_max: f32,
}
impl Default for LineStyleColorModifier_DistanceFromObject {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            target: NIL,
            color_ramp: NIL,
            range_min: 0.0,
            range_max: 0.0,
        }
    }
}

/// `LineStyleAlphaModifier_DistanceFromObject`.
#[derive(Clone)]
pub struct LineStyleAlphaModifier_DistanceFromObject {
    pub modifier: LineStyleModifier,
    /// `Object *target`
    pub target: usize,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float range_min`
    pub range_min: f32,
    /// `float range_max`
    pub range_max: f32,
}
impl Default for LineStyleAlphaModifier_DistanceFromObject {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            target: NIL,
            curve: NIL,
            flags: Default::default(),
            range_min: 0.0,
            range_max: 0.0,
        }
    }
}

/// `LineStyleThicknessModifier_DistanceFromObject`.
#[derive(Clone)]
pub struct LineStyleThicknessModifier_DistanceFromObject {
    pub modifier: LineStyleModifier,
    /// `Object *target`
    pub target: usize,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float range_min`
    pub range_min: f32,
    /// `float range_max`
    pub range_max: f32,
    /// `float value_min`
    pub value_min: f32,
    /// `float value_max`
    pub value_max: f32,
}
impl Default for LineStyleThicknessModifier_DistanceFromObject {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            target: NIL,
            curve: NIL,
            flags: Default::default(),
            range_min: 0.0,
            range_max: 0.0,
            value_min: 0.0,
            value_max: 0.0,
        }
    }
}

/// `LineStyleColorModifier_Curvature_3D`.
#[derive(Clone)]
pub struct LineStyleColorModifier_Curvature_3D {
    pub modifier: LineStyleModifier,
    /// `float min_curvature`
    pub min_curvature: f32,
    /// `float max_curvature`
    pub max_curvature: f32,
    /// `ColorBand *color_ramp`
    pub color_ramp: usize,
    /// `float range_min`
    pub range_min: f32,
    /// `float range_max`
    pub range_max: f32,
}
impl Default for LineStyleColorModifier_Curvature_3D {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            min_curvature: 0.0,
            max_curvature: 0.0,
            color_ramp: NIL,
            range_min: 0.0,
            range_max: 0.0,
        }
    }
}

/// `LineStyleAlphaModifier_Curvature_3D`.
#[derive(Clone)]
pub struct LineStyleAlphaModifier_Curvature_3D {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float min_curvature`
    pub min_curvature: f32,
    /// `float max_curvature`
    pub max_curvature: f32,
}
impl Default for LineStyleAlphaModifier_Curvature_3D {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            min_curvature: 0.0,
            max_curvature: 0.0,
        }
    }
}

/// `LineStyleThicknessModifier_Curvature_3D`.
#[derive(Clone)]
pub struct LineStyleThicknessModifier_Curvature_3D {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float min_curvature`
    pub min_curvature: f32,
    /// `float max_curvature`
    pub max_curvature: f32,
    /// `float min_thickness`
    pub min_thickness: f32,
    /// `float max_thickness`
    pub max_thickness: f32,
}
impl Default for LineStyleThicknessModifier_Curvature_3D {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            min_curvature: 0.0,
            max_curvature: 0.0,
            min_thickness: 0.0,
            max_thickness: 0.0,
        }
    }
}

/// `LineStyleColorModifier_Noise`.
#[derive(Clone)]
pub struct LineStyleColorModifier_Noise {
    pub modifier: LineStyleModifier,
    /// `ColorBand *color_ramp`
    pub color_ramp: usize,
    /// `float period`
    pub period: f32,
    /// `float amplitude`
    pub amplitude: f32,
    /// `int seed`
    pub seed: i32,
}
impl Default for LineStyleColorModifier_Noise {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            color_ramp: NIL,
            period: 0.0,
            amplitude: 0.0,
            seed: 0,
        }
    }
}

/// `LineStyleAlphaModifier_Noise`.
#[derive(Clone)]
pub struct LineStyleAlphaModifier_Noise {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float period`
    pub period: f32,
    /// `float amplitude`
    pub amplitude: f32,
    /// `int seed`
    pub seed: i32,
}
impl Default for LineStyleAlphaModifier_Noise {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            period: 0.0,
            amplitude: 0.0,
            seed: 0,
        }
    }
}

/// `LineStyleThicknessModifier_Noise`.
#[derive(Clone)]
pub struct LineStyleThicknessModifier_Noise {
    pub modifier: LineStyleModifier,
    /// `float period`
    pub period: f32,
    /// `float amplitude`
    pub amplitude: f32,
    /// `eLineStyleThicknessNoise_Flag flags`
    pub flags: LinestylethicknessnoiseFlag,
    /// `int seed`
    pub seed: i32,
}
impl Default for LineStyleThicknessModifier_Noise {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            period: 0.0,
            amplitude: 0.0,
            flags: Default::default(),
            seed: 0,
        }
    }
}

/// `LineStyleColorModifier_CreaseAngle`.
#[derive(Clone)]
pub struct LineStyleColorModifier_CreaseAngle {
    pub modifier: LineStyleModifier,
    /// `ColorBand *color_ramp`
    pub color_ramp: usize,
    /// `float min_angle`
    pub min_angle: f32,
    /// `float max_angle`
    pub max_angle: f32,
}
impl Default for LineStyleColorModifier_CreaseAngle {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            color_ramp: NIL,
            min_angle: 0.0,
            max_angle: 0.0,
        }
    }
}

/// `LineStyleAlphaModifier_CreaseAngle`.
#[derive(Clone)]
pub struct LineStyleAlphaModifier_CreaseAngle {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float min_angle`
    pub min_angle: f32,
    /// `float max_angle`
    pub max_angle: f32,
}
impl Default for LineStyleAlphaModifier_CreaseAngle {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            min_angle: 0.0,
            max_angle: 0.0,
        }
    }
}

/// `LineStyleThicknessModifier_CreaseAngle`.
#[derive(Clone)]
pub struct LineStyleThicknessModifier_CreaseAngle {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float min_angle`
    pub min_angle: f32,
    /// `float max_angle`
    pub max_angle: f32,
    /// `float min_thickness`
    pub min_thickness: f32,
    /// `float max_thickness`
    pub max_thickness: f32,
}
impl Default for LineStyleThicknessModifier_CreaseAngle {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            min_angle: 0.0,
            max_angle: 0.0,
            min_thickness: 0.0,
            max_thickness: 0.0,
        }
    }
}

/// `LineStyleColorModifier_Tangent`.
#[derive(Clone)]
pub struct LineStyleColorModifier_Tangent {
    pub modifier: LineStyleModifier,
    /// `ColorBand *color_ramp`
    pub color_ramp: usize,
}
impl Default for LineStyleColorModifier_Tangent {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            color_ramp: NIL,
        }
    }
}

/// `LineStyleAlphaModifier_Tangent`.
#[derive(Clone)]
pub struct LineStyleAlphaModifier_Tangent {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
}
impl Default for LineStyleAlphaModifier_Tangent {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
        }
    }
}

/// `LineStyleThicknessModifier_Tangent`.
#[derive(Clone)]
pub struct LineStyleThicknessModifier_Tangent {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float min_thickness`
    pub min_thickness: f32,
    /// `float max_thickness`
    pub max_thickness: f32,
}
impl Default for LineStyleThicknessModifier_Tangent {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            min_thickness: 0.0,
            max_thickness: 0.0,
        }
    }
}

/// `LineStyleColorModifier_Material`.
#[derive(Clone)]
pub struct LineStyleColorModifier_Material {
    pub modifier: LineStyleModifier,
    /// `ColorBand *color_ramp`
    pub color_ramp: usize,
    /// `eLineStyleColorModifier_Flag flags`
    pub flags: LinestylecolormodifierFlag,
    /// `eLineStyleMaterialAttr mat_attr`
    pub mat_attr: Linestylematerialattr,
}
impl Default for LineStyleColorModifier_Material {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            color_ramp: NIL,
            flags: Default::default(),
            mat_attr: Default::default(),
        }
    }
}

/// `LineStyleAlphaModifier_Material`.
#[derive(Clone)]
pub struct LineStyleAlphaModifier_Material {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `eLineStyleMaterialAttr mat_attr`
    pub mat_attr: Linestylematerialattr,
}
impl Default for LineStyleAlphaModifier_Material {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            mat_attr: Default::default(),
        }
    }
}

/// `LineStyleThicknessModifier_Material`.
#[derive(Clone)]
pub struct LineStyleThicknessModifier_Material {
    pub modifier: LineStyleModifier,
    /// `CurveMapping *curve`
    pub curve: usize,
    /// `eLineStyleAlphaThicknessModifier_Flag flags`
    pub flags: LinestylealphathicknessmodifierFlag,
    /// `float value_min`
    pub value_min: f32,
    /// `float value_max`
    pub value_max: f32,
    /// `eLineStyleMaterialAttr mat_attr`
    pub mat_attr: Linestylematerialattr,
}
impl Default for LineStyleThicknessModifier_Material {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve: NIL,
            flags: Default::default(),
            value_min: 0.0,
            value_max: 0.0,
            mat_attr: Default::default(),
        }
    }
}

/// `LineStyleGeometryModifier_Sampling`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_Sampling {
    pub modifier: LineStyleModifier,
    /// `float sampling`
    pub sampling: f32,
}
impl Default for LineStyleGeometryModifier_Sampling {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            sampling: 0.0,
        }
    }
}

/// `LineStyleGeometryModifier_BezierCurve`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_BezierCurve {
    pub modifier: LineStyleModifier,
    /// `float error`
    pub error: f32,
}
impl Default for LineStyleGeometryModifier_BezierCurve {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            error: 0.0,
        }
    }
}

/// `LineStyleGeometryModifier_SinusDisplacement`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_SinusDisplacement {
    pub modifier: LineStyleModifier,
    /// `float wavelength`
    pub wavelength: f32,
    /// `float amplitude`
    pub amplitude: f32,
    /// `float phase`
    pub phase: f32,
}
impl Default for LineStyleGeometryModifier_SinusDisplacement {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            wavelength: 0.0,
            amplitude: 0.0,
            phase: 0.0,
        }
    }
}

/// `LineStyleGeometryModifier_SpatialNoise`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_SpatialNoise {
    pub modifier: LineStyleModifier,
    /// `float amplitude`
    pub amplitude: f32,
    /// `float scale`
    pub scale: f32,
    /// `unsigned_int octaves`
    pub octaves: u32,
    /// `eLineStyleGeomSpatialNoise_Flag flags`
    pub flags: LinestylegeomspatialnoiseFlag,
}
impl Default for LineStyleGeometryModifier_SpatialNoise {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            amplitude: 0.0,
            scale: 0.0,
            octaves: 0,
            flags: Default::default(),
        }
    }
}

/// `LineStyleGeometryModifier_PerlinNoise1D`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_PerlinNoise1D {
    pub modifier: LineStyleModifier,
    /// `float frequency`
    pub frequency: f32,
    /// `float amplitude`
    pub amplitude: f32,
    /// `float angle`
    pub angle: f32,
    /// `unsigned_int octaves`
    pub octaves: u32,
    /// `int seed`
    pub seed: i32,
}
impl Default for LineStyleGeometryModifier_PerlinNoise1D {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            frequency: 0.0,
            amplitude: 0.0,
            angle: 0.0,
            octaves: 0,
            seed: 0,
        }
    }
}

/// `LineStyleGeometryModifier_PerlinNoise2D`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_PerlinNoise2D {
    pub modifier: LineStyleModifier,
    /// `float frequency`
    pub frequency: f32,
    /// `float amplitude`
    pub amplitude: f32,
    /// `float angle`
    pub angle: f32,
    /// `unsigned_int octaves`
    pub octaves: u32,
    /// `int seed`
    pub seed: i32,
}
impl Default for LineStyleGeometryModifier_PerlinNoise2D {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            frequency: 0.0,
            amplitude: 0.0,
            angle: 0.0,
            octaves: 0,
            seed: 0,
        }
    }
}

/// `LineStyleGeometryModifier_BackboneStretcher`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_BackboneStretcher {
    pub modifier: LineStyleModifier,
    /// `float backbone_length`
    pub backbone_length: f32,
}
impl Default for LineStyleGeometryModifier_BackboneStretcher {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            backbone_length: 0.0,
        }
    }
}

/// `LineStyleGeometryModifier_TipRemover`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_TipRemover {
    pub modifier: LineStyleModifier,
    /// `float tip_length`
    pub tip_length: f32,
}
impl Default for LineStyleGeometryModifier_TipRemover {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            tip_length: 0.0,
        }
    }
}

/// `LineStyleGeometryModifier_Polygonalization`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_Polygonalization {
    pub modifier: LineStyleModifier,
    /// `float error`
    pub error: f32,
}
impl Default for LineStyleGeometryModifier_Polygonalization {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            error: 0.0,
        }
    }
}

/// `LineStyleGeometryModifier_GuidingLines`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_GuidingLines {
    pub modifier: LineStyleModifier,
    /// `float offset`
    pub offset: f32,
}
impl Default for LineStyleGeometryModifier_GuidingLines {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            offset: 0.0,
        }
    }
}

/// `LineStyleGeometryModifier_Blueprint`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_Blueprint {
    pub modifier: LineStyleModifier,
    /// `eLineStyleGeomBlueprint_Flag flags`
    pub flags: LinestylegeomblueprintFlag,
    /// `unsigned_int rounds`
    pub rounds: u32,
    /// `float backbone_length`
    pub backbone_length: f32,
    /// `unsigned_int random_radius`
    pub random_radius: u32,
    /// `unsigned_int random_center`
    pub random_center: u32,
    /// `unsigned_int random_backbone`
    pub random_backbone: u32,
}
impl Default for LineStyleGeometryModifier_Blueprint {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            flags: Default::default(),
            rounds: 0,
            backbone_length: 0.0,
            random_radius: 0,
            random_center: 0,
            random_backbone: 0,
        }
    }
}

/// `LineStyleGeometryModifier_2DOffset`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_2DOffset {
    pub modifier: LineStyleModifier,
    /// `float start`
    pub start: f32,
    /// `float end`
    pub end: f32,
    /// `float x`
    pub x: f32,
    /// `float y`
    pub y: f32,
}
impl Default for LineStyleGeometryModifier_2DOffset {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            start: 0.0,
            end: 0.0,
            x: 0.0,
            y: 0.0,
        }
    }
}

/// `LineStyleGeometryModifier_2DTransform`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_2DTransform {
    pub modifier: LineStyleModifier,
    /// `eLineStyleGeom2DTransform_Pivot pivot`
    pub pivot: Linestylegeom2dtransformPivot,
    /// `float scale_x`
    pub scale_x: f32,
    /// `float scale_y`
    pub scale_y: f32,
    /// `float angle`
    pub angle: f32,
    /// `float pivot_u`
    pub pivot_u: f32,
    /// `float pivot_x`
    pub pivot_x: f32,
    /// `float pivot_y`
    pub pivot_y: f32,
}
impl Default for LineStyleGeometryModifier_2DTransform {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            pivot: Default::default(),
            scale_x: 0.0,
            scale_y: 0.0,
            angle: 0.0,
            pivot_u: 0.0,
            pivot_x: 0.0,
            pivot_y: 0.0,
        }
    }
}

/// `LineStyleGeometryModifier_Simplification`.
#[derive(Clone)]
pub struct LineStyleGeometryModifier_Simplification {
    pub modifier: LineStyleModifier,
    /// `float tolerance`
    pub tolerance: f32,
}
impl Default for LineStyleGeometryModifier_Simplification {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            tolerance: 0.0,
        }
    }
}

/// `LineStyleThicknessModifier_Calligraphy`.
#[derive(Clone)]
pub struct LineStyleThicknessModifier_Calligraphy {
    pub modifier: LineStyleModifier,
    /// `float min_thickness`
    pub min_thickness: f32,
    /// `float max_thickness`
    pub max_thickness: f32,
    /// `float orientation`
    pub orientation: f32,
}
impl Default for LineStyleThicknessModifier_Calligraphy {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            min_thickness: 0.0,
            max_thickness: 0.0,
            orientation: 0.0,
        }
    }
}

/// `FreestyleLineStyle`.
#[derive(Clone)]
pub struct FreestyleLineStyle {
    pub id: ID,
    /// `AnimData *adt`
    pub adt: usize,
    /// `float r`
    pub r: f32,
    /// `float g`
    pub g: f32,
    /// `float b`
    pub b: f32,
    /// `float alpha`
    pub alpha: f32,
    /// `float thickness`
    pub thickness: f32,
    /// `eLineStyle_ThicknessPosition thickness_position`
    pub thickness_position: LinestyleThicknessposition,
    /// `float thickness_ratio`
    pub thickness_ratio: f32,
    /// `eLineStyle_Flag flag`
    pub flag: LinestyleFlag,
    /// `eLineStyle_Caps caps`
    pub caps: LinestyleCaps,
    /// `eLineStyle_Chaining chaining`
    pub chaining: LinestyleChaining,
    /// `unsigned_int rounds`
    pub rounds: u32,
    /// `float split_length`
    pub split_length: f32,
    /// `float min_angle`
    pub min_angle: f32,
    /// `float max_angle`
    pub max_angle: f32,
    /// `float min_length`
    pub min_length: f32,
    /// `float max_length`
    pub max_length: f32,
    /// `unsigned_int chain_count`
    pub chain_count: u32,
    /// `unsigned_short split_dash1`
    pub split_dash1: u16,
    /// `unsigned_short split_gap1`
    pub split_gap1: u16,
    /// `unsigned_short split_dash2`
    pub split_dash2: u16,
    /// `unsigned_short split_gap2`
    pub split_gap2: u16,
    /// `unsigned_short split_dash3`
    pub split_dash3: u16,
    /// `unsigned_short split_gap3`
    pub split_gap3: u16,
    /// `eLineStyle_SortKey sort_key`
    pub sort_key: LinestyleSortkey,
    /// `eLineStyle_IntegrationType integration_type`
    pub integration_type: LinestyleIntegrationtype,
    /// `float texstep`
    pub texstep: f32,
    /// `short texact`
    pub texact: i16,
    /// `short pr_texture`
    pub pr_texture: i16,
    /// `short use_nodes`
    pub use_nodes: i16,
    /// `unsigned_short dash1`
    pub dash1: u16,
    /// `unsigned_short gap1`
    pub gap1: u16,
    /// `unsigned_short dash2`
    pub dash2: u16,
    /// `unsigned_short gap2`
    pub gap2: u16,
    /// `unsigned_short dash3`
    pub dash3: u16,
    /// `unsigned_short gap3`
    pub gap3: u16,
    /// `eLineStyle_Panel panel`
    pub panel: LinestylePanel,
    /// `MTex *mtex[]`
    pub mtex: [usize; 18],
    /// `bNodeTree *nodetree`
    pub nodetree: usize,
    /// `ListBaseT<LineStyleModifier>`
    pub color_modifiers: ListBase,
    /// `ListBaseT<LineStyleModifier>`
    pub alpha_modifiers: ListBase,
    /// `ListBaseT<LineStyleModifier>`
    pub thickness_modifiers: ListBase,
    /// `ListBaseT<LineStyleModifier>`
    pub geometry_modifiers: ListBase,
}
impl Default for FreestyleLineStyle {
    fn default() -> Self {
        Self {
            id: Default::default(),
            adt: NIL,
            r: 0.0,
            g: 0.0,
            b: 0.0,
            alpha: 1.0,
            thickness: 3.0,
            thickness_position: LS_THICKNESS_CENTER,
            thickness_ratio: 0.5,
            flag: LS_SAME_OBJECT | LS_NO_SORTING | LS_TEXTURE,
            caps: LS_CAPS_BUTT,
            chaining: LS_CHAINING_PLAIN,
            rounds: 3,
            split_length: 100.0,
            min_angle: (0.0f32).to_radians(),
            max_angle: (0.0f32).to_radians(),
            min_length: 0.0,
            max_length: 10000.0,
            chain_count: 10,
            split_dash1: 0,
            split_gap1: 0,
            split_dash2: 0,
            split_gap2: 0,
            split_dash3: 0,
            split_gap3: 0,
            sort_key: LS_SORT_KEY_DISTANCE_FROM_CAMERA,
            integration_type: LS_INTEGRATION_MEAN,
            texstep: 1.0,
            texact: 0,
            pr_texture: TEX_PR_TEXTURE,
            use_nodes: 0,
            dash1: 0,
            gap1: 0,
            dash2: 0,
            gap2: 0,
            dash3: 0,
            gap3: 0,
            panel: LS_PANEL_STROKES,
            mtex: [NIL; 18],
            nodetree: NIL,
            color_modifiers: ListBase::default(),
            alpha_modifiers: ListBase::default(),
            thickness_modifiers: ListBase::default(),
            geometry_modifiers: ListBase::default(),
        }
    }
}
