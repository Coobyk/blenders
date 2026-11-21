//! Port of `DNA_grease_pencil_modifier_types.h` — GP legacy modifier data.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::armature_types::ARM_DEF_VGROUP;
use crate::brush_enums::GPPAINT_MODE_BOTH;
use crate::id::NIL;
use crate::modifier_enums::{
    GP_TEX_CONSTANT_LENGTH, MOD_SHRINKWRAP_NEAREST_SURFACE,
    MOD_SHRINKWRAP_PROJECT_ALLOW_POS_DIR,
};
use blenders_bli::listbase::ListBase;
/// `GpencilModifierMode`.
pub type Gpencilmodifiermode = i32;
pub const eGpencilModifierMode_Realtime: Gpencilmodifiermode = 1 << 0;
pub const eGpencilModifierMode_Render: Gpencilmodifiermode = 1 << 1;
pub const eGpencilModifierMode_Editmode: Gpencilmodifiermode = 1 << 2;
pub const eGpencilModifierMode_Expanded_DEPRECATED: Gpencilmodifiermode = 1 << 3;
pub const eGpencilModifierMode_Virtual: Gpencilmodifiermode = 1 << 4;

/// `GpencilModifierFlag`.
pub type Gpencilmodifierflag = i16;
pub const eGpencilModifierFlag_OverrideLibrary_Local: Gpencilmodifierflag = 1 << 0;

/// `eGpencilModifierSpace`.
pub type Gpencilmodifierspace = i32;
pub const GP_SPACE_LOCAL: Gpencilmodifierspace = 0;
pub const GP_SPACE_WORLD: Gpencilmodifierspace = 1;

/// `GpencilModifierType`.
pub type Gpencilmodifiertype = i32;
pub const eGpencilModifierType_None: Gpencilmodifiertype = 0;
pub const eGpencilModifierType_Noise: Gpencilmodifiertype = 1;
pub const eGpencilModifierType_Subdiv: Gpencilmodifiertype = 2;
pub const eGpencilModifierType_Thick: Gpencilmodifiertype = 3;
pub const eGpencilModifierType_Tint: Gpencilmodifiertype = 4;
pub const eGpencilModifierType_Array: Gpencilmodifiertype = 5;
pub const eGpencilModifierType_Build: Gpencilmodifiertype = 6;
pub const eGpencilModifierType_Opacity: Gpencilmodifiertype = 7;
pub const eGpencilModifierType_Color: Gpencilmodifiertype = 8;
pub const eGpencilModifierType_Lattice: Gpencilmodifiertype = 9;
pub const eGpencilModifierType_Simplify: Gpencilmodifiertype = 10;
pub const eGpencilModifierType_Smooth: Gpencilmodifiertype = 11;
pub const eGpencilModifierType_Hook: Gpencilmodifiertype = 12;
pub const eGpencilModifierType_Offset: Gpencilmodifiertype = 13;
pub const eGpencilModifierType_Mirror: Gpencilmodifiertype = 14;
pub const eGpencilModifierType_Armature: Gpencilmodifiertype = 15;
pub const eGpencilModifierType_Time: Gpencilmodifiertype = 16;
pub const eGpencilModifierType_Multiply: Gpencilmodifiertype = 17;
pub const eGpencilModifierType_Texture: Gpencilmodifiertype = 18;
pub const eGpencilModifierType_Lineart: Gpencilmodifiertype = 19;
pub const eGpencilModifierType_Length: Gpencilmodifiertype = 20;
pub const eGpencilModifierType_WeightProximity: Gpencilmodifiertype = 21;
pub const eGpencilModifierType_Dash: Gpencilmodifiertype = 22;
pub const eGpencilModifierType_WeightAngle: Gpencilmodifiertype = 23;
pub const eGpencilModifierType_Shrinkwrap: Gpencilmodifiertype = 24;
pub const eGpencilModifierType_Envelope: Gpencilmodifiertype = 25;
pub const eGpencilModifierType_Outline: Gpencilmodifiertype = 26;
pub const NUM_GREASEPENCIL_MODIFIER_TYPES: Gpencilmodifiertype = eGpencilModifierType_Outline + 1;

/// `eSubdivGpencil_Flag`.
pub type SubdivgpencilFlag = i32;
pub const GP_SUBDIV_INVERT_LAYER: SubdivgpencilFlag = 1 << 1;
pub const GP_SUBDIV_INVERT_PASS: SubdivgpencilFlag = 1 << 2;
pub const GP_SUBDIV_INVERT_LAYERPASS: SubdivgpencilFlag = 1 << 3;
pub const GP_SUBDIV_INVERT_MATERIAL: SubdivgpencilFlag = 1 << 4;

/// `eSubdivGpencil_Type`.
pub type SubdivgpencilType = i16;
pub const GP_SUBDIV_CATMULL: SubdivgpencilType = 0;
pub const GP_SUBDIV_SIMPLE: SubdivgpencilType = 1;

/// `eThickGpencil_Flag`.
pub type ThickgpencilFlag = i32;
pub const GP_THICK_INVERT_LAYER: ThickgpencilFlag = 1 << 0;
pub const GP_THICK_INVERT_PASS: ThickgpencilFlag = 1 << 1;
pub const GP_THICK_INVERT_VGROUP: ThickgpencilFlag = 1 << 2;
pub const GP_THICK_CUSTOM_CURVE: ThickgpencilFlag = 1 << 3;
pub const GP_THICK_NORMALIZE: ThickgpencilFlag = 1 << 4;
pub const GP_THICK_INVERT_LAYERPASS: ThickgpencilFlag = 1 << 5;
pub const GP_THICK_INVERT_MATERIAL: ThickgpencilFlag = 1 << 6;
pub const GP_THICK_WEIGHT_FACTOR: ThickgpencilFlag = 1 << 7;

/// `eTimeGpencil_Flag`.
pub type TimegpencilFlag = i32;
pub const GP_TIME_INVERT_LAYER: TimegpencilFlag = 1 << 0;
pub const GP_TIME_KEEP_LOOP: TimegpencilFlag = 1 << 1;
pub const GP_TIME_INVERT_LAYERPASS: TimegpencilFlag = 1 << 2;
pub const GP_TIME_CUSTOM_RANGE: TimegpencilFlag = 1 << 3;

/// `eTimeGpencil_Mode`.
pub type TimegpencilMode = i32;
pub const GP_TIME_MODE_NORMAL: TimegpencilMode = 0;
pub const GP_TIME_MODE_REVERSE: TimegpencilMode = 1;
pub const GP_TIME_MODE_FIX: TimegpencilMode = 2;
pub const GP_TIME_MODE_PINGPONG: TimegpencilMode = 3;
pub const GP_TIME_MODE_CHAIN: TimegpencilMode = 4;

/// `eTimeGpencil_Seg_Mode`.
pub type TimegpencilSegMode = i32;
pub const GP_TIME_SEG_MODE_NORMAL: TimegpencilSegMode = 0;
pub const GP_TIME_SEG_MODE_REVERSE: TimegpencilSegMode = 1;
pub const GP_TIME_SEG_MODE_PINGPONG: TimegpencilSegMode = 2;

/// `eModifyColorGpencil_Flag`.
pub type ModifycolorgpencilFlag = i8;
pub const GP_MODIFY_COLOR_BOTH: ModifycolorgpencilFlag = 0;
pub const GP_MODIFY_COLOR_STROKE: ModifycolorgpencilFlag = 1;
pub const GP_MODIFY_COLOR_FILL: ModifycolorgpencilFlag = 2;
pub const GP_MODIFY_COLOR_HARDNESS: ModifycolorgpencilFlag = 3;

/// `eOpacityModesGpencil_Flag`.
pub type OpacitymodesgpencilFlag = i32;
pub const GP_OPACITY_MODE_MATERIAL: OpacitymodesgpencilFlag = 0;
pub const GP_OPACITY_MODE_STRENGTH: OpacitymodesgpencilFlag = 1;

/// `eColorGpencil_Flag`.
pub type ColorgpencilFlag = i32;
pub const GP_COLOR_INVERT_LAYER: ColorgpencilFlag = 1 << 1;
pub const GP_COLOR_INVERT_PASS: ColorgpencilFlag = 1 << 2;
pub const GP_COLOR_INVERT_LAYERPASS: ColorgpencilFlag = 1 << 3;
pub const GP_COLOR_INVERT_MATERIAL: ColorgpencilFlag = 1 << 4;
pub const GP_COLOR_CUSTOM_CURVE: ColorgpencilFlag = 1 << 5;

/// `eOpacityGpencil_Flag`.
pub type OpacitygpencilFlag = i32;
pub const GP_OPACITY_INVERT_LAYER: OpacitygpencilFlag = 1 << 0;
pub const GP_OPACITY_INVERT_PASS: OpacitygpencilFlag = 1 << 1;
pub const GP_OPACITY_INVERT_VGROUP: OpacitygpencilFlag = 1 << 2;
pub const GP_OPACITY_INVERT_LAYERPASS: OpacitygpencilFlag = 1 << 4;
pub const GP_OPACITY_INVERT_MATERIAL: OpacitygpencilFlag = 1 << 5;
pub const GP_OPACITY_CUSTOM_CURVE: OpacitygpencilFlag = 1 << 6;
pub const GP_OPACITY_NORMALIZE: OpacitygpencilFlag = 1 << 7;
pub const GP_OPACITY_WEIGHT_FACTOR: OpacitygpencilFlag = 1 << 8;

/// `eOutlineGpencil_Flag`.
pub type OutlinegpencilFlag = i32;
pub const GP_OUTLINE_INVERT_LAYER: OutlinegpencilFlag = 1 << 0;
pub const GP_OUTLINE_INVERT_PASS: OutlinegpencilFlag = 1 << 1;
pub const GP_OUTLINE_INVERT_LAYERPASS: OutlinegpencilFlag = 1 << 2;
pub const GP_OUTLINE_INVERT_MATERIAL: OutlinegpencilFlag = 1 << 3;
pub const GP_OUTLINE_KEEP_SHAPE: OutlinegpencilFlag = 1 << 4;

/// `eBuildGpencil_Mode`.
pub type BuildgpencilMode = i16;
pub const GP_BUILD_MODE_SEQUENTIAL: BuildgpencilMode = 0;
pub const GP_BUILD_MODE_CONCURRENT: BuildgpencilMode = 1;
pub const GP_BUILD_MODE_ADDITIVE: BuildgpencilMode = 2;

/// `eBuildGpencil_Transition`.
pub type BuildgpencilTransition = i16;
pub const GP_BUILD_TRANSITION_GROW: BuildgpencilTransition = 0;
pub const GP_BUILD_TRANSITION_SHRINK: BuildgpencilTransition = 1;
pub const GP_BUILD_TRANSITION_VANISH: BuildgpencilTransition = 2;

/// `eBuildGpencil_TimeAlignment`.
pub type BuildgpencilTimealignment = i16;
pub const GP_BUILD_TIMEALIGN_START: BuildgpencilTimealignment = 0;
pub const GP_BUILD_TIMEALIGN_END: BuildgpencilTimealignment = 1;

/// `eBuildGpencil_TimeMode`.
pub type BuildgpencilTimemode = i16;
pub const GP_BUILD_TIMEMODE_FRAMES: BuildgpencilTimemode = 0;
pub const GP_BUILD_TIMEMODE_PERCENTAGE: BuildgpencilTimemode = 1;
pub const GP_BUILD_TIMEMODE_DRAWSPEED: BuildgpencilTimemode = 2;

/// `eBuildGpencil_Flag`.
pub type BuildgpencilFlag = i16;
pub const GP_BUILD_INVERT_LAYER: BuildgpencilFlag = 1 << 0;
pub const GP_BUILD_INVERT_PASS: BuildgpencilFlag = 1 << 1;
pub const GP_BUILD_RESTRICT_TIME: BuildgpencilFlag = 1 << 2;
pub const GP_BUILD_INVERT_LAYERPASS: BuildgpencilFlag = 1 << 3;
pub const GP_BUILD_USE_FADING: BuildgpencilFlag = 1 << 4;

/// `eLatticeGpencil_Flag`.
pub type LatticegpencilFlag = i32;
pub const GP_LATTICE_INVERT_LAYER: LatticegpencilFlag = 1 << 0;
pub const GP_LATTICE_INVERT_PASS: LatticegpencilFlag = 1 << 1;
pub const GP_LATTICE_INVERT_VGROUP: LatticegpencilFlag = 1 << 2;
pub const GP_LATTICE_INVERT_LAYERPASS: LatticegpencilFlag = 1 << 3;
pub const GP_LATTICE_INVERT_MATERIAL: LatticegpencilFlag = 1 << 4;

/// `eDashGpencil_Flag`.
pub type DashgpencilFlag = i32;
pub const GP_DASH_INVERT_LAYER: DashgpencilFlag = 1 << 0;
pub const GP_DASH_INVERT_PASS: DashgpencilFlag = 1 << 1;
pub const GP_DASH_INVERT_LAYERPASS: DashgpencilFlag = 1 << 2;
pub const GP_DASH_INVERT_MATERIAL: DashgpencilFlag = 1 << 3;
pub const GP_DASH_USE_CYCLIC: DashgpencilFlag = 1 << 7;

/// `eMirrorGpencil_Flag`.
pub type MirrorgpencilFlag = i32;
pub const GP_MIRROR_INVERT_LAYER: MirrorgpencilFlag = 1 << 0;
pub const GP_MIRROR_INVERT_PASS: MirrorgpencilFlag = 1 << 1;
pub const GP_MIRROR_CLIPPING: MirrorgpencilFlag = 1 << 2;
pub const GP_MIRROR_AXIS_X: MirrorgpencilFlag = 1 << 3;
pub const GP_MIRROR_AXIS_Y: MirrorgpencilFlag = 1 << 4;
pub const GP_MIRROR_AXIS_Z: MirrorgpencilFlag = 1 << 5;
pub const GP_MIRROR_INVERT_LAYERPASS: MirrorgpencilFlag = 1 << 6;
pub const GP_MIRROR_INVERT_MATERIAL: MirrorgpencilFlag = 1 << 7;

/// `eHookGpencil_Flag`.
pub type HookgpencilFlag = i32;
pub const GP_HOOK_INVERT_LAYER: HookgpencilFlag = 1 << 0;
pub const GP_HOOK_INVERT_PASS: HookgpencilFlag = 1 << 1;
pub const GP_HOOK_INVERT_VGROUP: HookgpencilFlag = 1 << 2;
pub const GP_HOOK_UNIFORM_SPACE: HookgpencilFlag = 1 << 3;
pub const GP_HOOK_INVERT_LAYERPASS: HookgpencilFlag = 1 << 4;
pub const GP_HOOK_INVERT_MATERIAL: HookgpencilFlag = 1 << 5;

/// `eHookGpencil_Falloff`.
pub type HookgpencilFalloff = i8;
pub const eGPHook_Falloff_None: HookgpencilFalloff = 0;
pub const eGPHook_Falloff_Curve: HookgpencilFalloff = 1;
pub const eGPHook_Falloff_Sharp: HookgpencilFalloff = 2;
pub const eGPHook_Falloff_Smooth: HookgpencilFalloff = 3;
pub const eGPHook_Falloff_Root: HookgpencilFalloff = 4;
pub const eGPHook_Falloff_Linear: HookgpencilFalloff = 5;
pub const eGPHook_Falloff_Const: HookgpencilFalloff = 6;
pub const eGPHook_Falloff_Sphere: HookgpencilFalloff = 7;
pub const eGPHook_Falloff_InvSquare: HookgpencilFalloff = 8;

/// `eSimplifyGpencil_Flag`.
pub type SimplifygpencilFlag = i32;
pub const GP_SIMPLIFY_INVERT_LAYER: SimplifygpencilFlag = 1 << 0;
pub const GP_SIMPLIFY_INVERT_PASS: SimplifygpencilFlag = 1 << 1;
pub const GP_SIMPLIFY_INVERT_LAYERPASS: SimplifygpencilFlag = 1 << 2;
pub const GP_SIMPLIFY_INVERT_MATERIAL: SimplifygpencilFlag = 1 << 3;

/// `eSimplifyGpencil_Mode`.
pub type SimplifygpencilMode = i16;
pub const GP_SIMPLIFY_FIXED: SimplifygpencilMode = 0;
pub const GP_SIMPLIFY_ADAPTIVE: SimplifygpencilMode = 1;
pub const GP_SIMPLIFY_SAMPLE: SimplifygpencilMode = 2;
pub const GP_SIMPLIFY_MERGE: SimplifygpencilMode = 3;

/// `eOffsetGpencil_Mode`.
pub type OffsetgpencilMode = i32;
pub const GP_OFFSET_RANDOM: OffsetgpencilMode = 0;
pub const GP_OFFSET_LAYER: OffsetgpencilMode = 1;
pub const GP_OFFSET_MATERIAL: OffsetgpencilMode = 2;
pub const GP_OFFSET_STROKE: OffsetgpencilMode = 3;

/// `eOffsetGpencil_Flag`.
pub type OffsetgpencilFlag = i32;
pub const GP_OFFSET_INVERT_LAYER: OffsetgpencilFlag = 1 << 0;
pub const GP_OFFSET_INVERT_PASS: OffsetgpencilFlag = 1 << 1;
pub const GP_OFFSET_INVERT_VGROUP: OffsetgpencilFlag = 1 << 2;
pub const GP_OFFSET_INVERT_LAYERPASS: OffsetgpencilFlag = 1 << 3;
pub const GP_OFFSET_INVERT_MATERIAL: OffsetgpencilFlag = 1 << 4;
pub const GP_OFFSET_UNIFORM_RANDOM_SCALE: OffsetgpencilFlag = 1 << 5;

/// `eSmoothGpencil_Flag`.
pub type SmoothgpencilFlag = i32;
pub const GP_SMOOTH_MOD_LOCATION: SmoothgpencilFlag = 1 << 0;
pub const GP_SMOOTH_MOD_STRENGTH: SmoothgpencilFlag = 1 << 1;
pub const GP_SMOOTH_MOD_THICKNESS: SmoothgpencilFlag = 1 << 2;
pub const GP_SMOOTH_INVERT_LAYER: SmoothgpencilFlag = 1 << 3;
pub const GP_SMOOTH_INVERT_PASS: SmoothgpencilFlag = 1 << 4;
pub const GP_SMOOTH_INVERT_VGROUP: SmoothgpencilFlag = 1 << 5;
pub const GP_SMOOTH_MOD_UV: SmoothgpencilFlag = 1 << 6;
pub const GP_SMOOTH_INVERT_LAYERPASS: SmoothgpencilFlag = 1 << 7;
pub const GP_SMOOTH_INVERT_MATERIAL: SmoothgpencilFlag = 1 << 4;
pub const GP_SMOOTH_CUSTOM_CURVE: SmoothgpencilFlag = 1 << 8;
pub const GP_SMOOTH_KEEP_SHAPE: SmoothgpencilFlag = 1 << 9;

/// `eMultiplyGpencil_Flag`.
pub type MultiplygpencilFlag = i32;
pub const GP_MULTIPLY_ENABLE_FADING: MultiplygpencilFlag = 1 << 2;

/// `eTintGpencil_Type`.
pub type TintgpencilType = i32;
pub const GP_TINT_UNIFORM: TintgpencilType = 0;
pub const GP_TINT_GRADIENT: TintgpencilType = 1;

/// `eTintGpencil_Flag`.
pub type TintgpencilFlag = i32;
pub const GP_TINT_INVERT_LAYER: TintgpencilFlag = 1 << 0;
pub const GP_TINT_INVERT_PASS: TintgpencilFlag = 1 << 1;
pub const GP_TINT_INVERT_VGROUP: TintgpencilFlag = 1 << 2;
pub const GP_TINT_INVERT_LAYERPASS: TintgpencilFlag = 1 << 4;
pub const GP_TINT_INVERT_MATERIAL: TintgpencilFlag = 1 << 5;
pub const GP_TINT_CUSTOM_CURVE: TintgpencilFlag = 1 << 6;
pub const GP_TINT_WEIGHT_FACTOR: TintgpencilFlag = 1 << 7;

/// `eTextureGpencil_Flag`.
pub type TexturegpencilFlag = i32;
pub const GP_TEX_INVERT_LAYER: TexturegpencilFlag = 1 << 0;
pub const GP_TEX_INVERT_PASS: TexturegpencilFlag = 1 << 1;
pub const GP_TEX_INVERT_VGROUP: TexturegpencilFlag = 1 << 2;
pub const GP_TEX_INVERT_LAYERPASS: TexturegpencilFlag = 1 << 3;
pub const GP_TEX_INVERT_MATERIAL: TexturegpencilFlag = 1 << 4;

/// `eTextureGpencil_Mode`.
pub type TexturegpencilMode = i16;
pub const STROKE: TexturegpencilMode = 0;
pub const FILL: TexturegpencilMode = 1;
pub const STROKE_AND_FILL: TexturegpencilMode = 2;

/// `eWeightGpencil_Flag`.
pub type WeightgpencilFlag = i32;
pub const GP_WEIGHT_INVERT_LAYER: WeightgpencilFlag = 1 << 0;
pub const GP_WEIGHT_INVERT_PASS: WeightgpencilFlag = 1 << 1;
pub const GP_WEIGHT_INVERT_VGROUP: WeightgpencilFlag = 1 << 2;
pub const GP_WEIGHT_INVERT_LAYERPASS: WeightgpencilFlag = 1 << 3;
pub const GP_WEIGHT_INVERT_MATERIAL: WeightgpencilFlag = 1 << 4;
pub const GP_WEIGHT_MULTIPLY_DATA: WeightgpencilFlag = 1 << 5;
pub const GP_WEIGHT_INVERT_OUTPUT: WeightgpencilFlag = 1 << 6;

/// `eLineArtGPencilModifierFlags`.
pub type Lineartgpencilmodifierflags = i32;
pub const MOD_LINEART_BINARY_WEIGHTS: Lineartgpencilmodifierflags = 1 << 2;
pub const MOD_LINEART_IS_BAKED: Lineartgpencilmodifierflags = 1 << 3;
pub const MOD_LINEART_USE_CACHE: Lineartgpencilmodifierflags = 1 << 4;
pub const MOD_LINEART_OFFSET_TOWARDS_CUSTOM_CAMERA: Lineartgpencilmodifierflags = 1 << 5;
pub const MOD_LINEART_INVERT_COLLECTION: Lineartgpencilmodifierflags = 1 << 6;
pub const MOD_LINEART_INVERT_SILHOUETTE_FILTER: Lineartgpencilmodifierflags = 1 << 7;

/// `eShrinkwrapGpencil_Flag`.
pub type ShrinkwrapgpencilFlag = i32;
pub const GP_SHRINKWRAP_INVERT_LAYER: ShrinkwrapgpencilFlag = 1 << 0;
pub const GP_SHRINKWRAP_INVERT_PASS: ShrinkwrapgpencilFlag = 1 << 1;
pub const GP_SHRINKWRAP_INVERT_LAYERPASS: ShrinkwrapgpencilFlag = 1 << 3;
pub const GP_SHRINKWRAP_INVERT_MATERIAL: ShrinkwrapgpencilFlag = 1 << 4;
pub const GP_SHRINKWRAP_INVERT_VGROUP: ShrinkwrapgpencilFlag = 1 << 6;

/// `eEnvelopeGpencil_Flag`.
pub type EnvelopegpencilFlag = i32;
pub const GP_ENVELOPE_INVERT_LAYER: EnvelopegpencilFlag = 1 << 0;
pub const GP_ENVELOPE_INVERT_PASS: EnvelopegpencilFlag = 1 << 1;
pub const GP_ENVELOPE_INVERT_VGROUP: EnvelopegpencilFlag = 1 << 2;
pub const GP_ENVELOPE_INVERT_LAYERPASS: EnvelopegpencilFlag = 1 << 3;
pub const GP_ENVELOPE_INVERT_MATERIAL: EnvelopegpencilFlag = 1 << 4;

/// `eEnvelopeGpencil_Mode`.
pub type EnvelopegpencilMode = i32;
pub const GP_ENVELOPE_DEFORM: EnvelopegpencilMode = 0;
pub const GP_ENVELOPE_SEGMENTS: EnvelopegpencilMode = 1;
pub const GP_ENVELOPE_FILLS: EnvelopegpencilMode = 2;

/// `GpencilModifierData`.
#[derive(Clone)]
pub struct GpencilModifierData {
    /// `GpencilModifierData *next`
    pub next: usize,
    /// `GpencilModifierData *prev`
    pub prev: usize,
    /// `GpencilModifierType type`
    pub type_: Gpencilmodifiertype,
    /// `GpencilModifierMode mode`
    pub mode: Gpencilmodifiermode,
    /// `GpencilModifierFlag flag`
    pub flag: Gpencilmodifierflag,
    /// `short ui_expand_flag`
    pub ui_expand_flag: i16,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `char *error`
    pub error: usize,
}
impl Default for GpencilModifierData {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: eGpencilModifierType_None,
            mode: Default::default(),
            flag: Default::default(),
            ui_expand_flag: 0,
            name: [0; 64],
            error: NIL,
        }
    }
}

// Flag bits from `DNA_modifier_enums.h` used by gpencil modifier defaults.
pub const GP_NOISE_FULL_STROKE: i32 = 1 << 4;
pub const GP_NOISE_USE_RANDOM: i32 = 1 << 5;
pub const GP_ARRAY_USE_RELATIVE: i32 = 1 << 8;
pub const GP_LENGTH_USE_CURVATURE: i32 = 1 << 4;

/// `NoiseGpencilModifierData`.
#[derive(Clone)]
pub struct NoiseGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `int flag`
    pub flag: i32,
    /// `float factor`
    pub factor: f32,
    /// `float factor_strength`
    pub factor_strength: f32,
    /// `float factor_thickness`
    pub factor_thickness: f32,
    /// `float factor_uvs`
    pub factor_uvs: f32,
    /// `float noise_scale`
    pub noise_scale: f32,
    /// `float noise_offset`
    pub noise_offset: f32,
    /// `short noise_mode`
    pub noise_mode: i16,
    /// `int step`
    pub step: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `int seed`
    pub seed: i32,
    /// `CurveMapping *curve_intensity`
    pub curve_intensity: usize,
}
impl Default for NoiseGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: GP_NOISE_FULL_STROKE | GP_NOISE_USE_RANDOM,
            factor: 0.5,
            factor_strength: 0.0,
            factor_thickness: 0.0,
            factor_uvs: 0.0,
            noise_scale: 0.0,
            noise_offset: 0.0,
            noise_mode: 0,
            step: 4,
            layer_pass: 0,
            seed: 1,
            curve_intensity: NIL,
        }
    }
}

/// `SubdivGpencilModifierData`.
#[derive(Clone)]
pub struct SubdivGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eSubdivGpencil_Flag flag`
    pub flag: SubdivgpencilFlag,
    /// `int level`
    pub level: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `eSubdivGpencil_Type type`
    pub type_: SubdivgpencilType,
}
impl Default for SubdivGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            level: 1,
            layer_pass: 0,
            type_: GP_SUBDIV_CATMULL,
        }
    }
}

/// `ThickGpencilModifierData`.
#[derive(Clone)]
pub struct ThickGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eThickGpencil_Flag flag`
    pub flag: ThickgpencilFlag,
    /// `float thickness_fac`
    pub thickness_fac: f32,
    /// `int thickness`
    pub thickness: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `CurveMapping *curve_thickness`
    pub curve_thickness: usize,
}
impl Default for ThickGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            thickness_fac: 1.0,
            thickness: 30,
            layer_pass: 0,
            curve_thickness: NIL,
        }
    }
}

/// `TimeGpencilModifierSegment`.
#[derive(Clone)]
pub struct TimeGpencilModifierSegment {
    /// `char name[64]`
    pub name: [u8; 64],
    /// `TimeGpencilModifierData *gpmd`
    pub gpmd: usize,
    /// `int seg_start`
    pub seg_start: i32,
    /// `int seg_end`
    pub seg_end: i32,
    /// `eTimeGpencil_Seg_Mode seg_mode`
    pub seg_mode: TimegpencilSegMode,
    /// `int seg_repeat`
    pub seg_repeat: i32,
}
impl Default for TimeGpencilModifierSegment {
    fn default() -> Self {
        Self {
            name: [0; 64],
            gpmd: NIL,
            seg_start: 1,
            seg_end: 2,
            seg_mode: GP_TIME_SEG_MODE_NORMAL,
            seg_repeat: 1,
        }
    }
}

/// `TimeGpencilModifierData`.
#[derive(Clone)]
pub struct TimeGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `eTimeGpencil_Flag flag`
    pub flag: TimegpencilFlag,
    /// `int offset`
    pub offset: i32,
    /// `float frame_scale`
    pub frame_scale: f32,
    /// `eTimeGpencil_Mode mode`
    pub mode: TimegpencilMode,
    /// `int sfra`
    pub sfra: i32,
    /// `int efra`
    pub efra: i32,
    /// `TimeGpencilModifierSegment *segments`
    pub segments: usize,
    /// `int segments_len`
    pub segments_len: i32,
    /// `int segment_active_index`
    pub segment_active_index: i32,
}
impl Default for TimeGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            layer_pass: 0,
            flag: GP_TIME_KEEP_LOOP,
            offset: 1,
            frame_scale: 1.0,
            mode: GP_TIME_MODE_NORMAL,
            sfra: 1,
            efra: 250,
            segments: NIL,
            segments_len: 1,
            segment_active_index: 0,
        }
    }
}

/// `ColorGpencilModifierData`.
#[derive(Clone)]
pub struct ColorGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eColorGpencil_Flag flag`
    pub flag: ColorgpencilFlag,
    /// `float hsv[3]`
    pub hsv: [f32; 3],
    /// `eModifyColorGpencil_Flag modify_color`
    pub modify_color: ModifycolorgpencilFlag,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `CurveMapping *curve_intensity`
    pub curve_intensity: usize,
}
impl Default for ColorGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            hsv: [0.0; 3],
            modify_color: GP_MODIFY_COLOR_BOTH,
            layer_pass: 0,
            curve_intensity: NIL,
        }
    }
}

/// `OpacityGpencilModifierData`.
#[derive(Clone)]
pub struct OpacityGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eOpacityGpencil_Flag flag`
    pub flag: OpacitygpencilFlag,
    /// `float factor`
    pub factor: f32,
    /// `eModifyColorGpencil_Flag modify_color`
    pub modify_color: ModifycolorgpencilFlag,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `float hardness`
    pub hardness: f32,
    /// `CurveMapping *curve_intensity`
    pub curve_intensity: usize,
}
impl Default for OpacityGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            factor: 1.0,
            modify_color: GP_MODIFY_COLOR_BOTH,
            layer_pass: 0,
            hardness: 1.0,
            curve_intensity: NIL,
        }
    }
}

/// `OutlineGpencilModifierData`.
#[derive(Clone)]
pub struct OutlineGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Object *object`
    pub object: usize,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eOutlineGpencil_Flag flag`
    pub flag: OutlinegpencilFlag,
    /// `int thickness`
    pub thickness: i32,
    /// `float sample_length`
    pub sample_length: f32,
    /// `int subdiv`
    pub subdiv: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `Material *outline_material`
    pub outline_material: usize,
}
impl Default for OutlineGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            material: NIL,
            layername: [0; 64],
            pass_index: 0,
            flag: GP_OUTLINE_KEEP_SHAPE,
            thickness: 1,
            sample_length: 0.0,
            subdiv: 3,
            layer_pass: 0,
            outline_material: NIL,
        }
    }
}

/// `ArrayGpencilModifierData`.
#[derive(Clone)]
pub struct ArrayGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Object *object`
    pub object: usize,
    /// `Material *material`
    pub material: usize,
    /// `int count`
    pub count: i32,
    /// `int flag`
    pub flag: i32,
    /// `float offset[3]`
    pub offset: [f32; 3],
    /// `float shift[3]`
    pub shift: [f32; 3],
    /// `float rnd_offset[3]`
    pub rnd_offset: [f32; 3],
    /// `float rnd_rot[3]`
    pub rnd_rot: [f32; 3],
    /// `float rnd_scale[3]`
    pub rnd_scale: [f32; 3],
    /// `int seed`
    pub seed: i32,
    /// `int pass_index`
    pub pass_index: i32,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `int mat_rpl`
    pub mat_rpl: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
}
impl Default for ArrayGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            material: NIL,
            count: 2,
            flag: GP_ARRAY_USE_RELATIVE,
            offset: [0.0; 3],
            shift: [0.0; 3],
            rnd_offset: [0.0; 3],
            rnd_rot: [0.0; 3],
            rnd_scale: [0.0; 3],
            seed: 1,
            pass_index: 0,
            layername: [0; 64],
            materialname: [0; 64],
            mat_rpl: 0,
            layer_pass: 0,
        }
    }
}

/// `BuildGpencilModifierData`.
#[derive(Clone)]
pub struct BuildGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `float start_frame`
    pub start_frame: f32,
    /// `float end_frame`
    pub end_frame: f32,
    /// `float start_delay`
    pub start_delay: f32,
    /// `float length`
    pub length: f32,
    /// `eBuildGpencil_Flag flag`
    pub flag: BuildgpencilFlag,
    /// `eBuildGpencil_Mode mode`
    pub mode: BuildgpencilMode,
    /// `eBuildGpencil_Transition transition`
    pub transition: BuildgpencilTransition,
    /// `eBuildGpencil_TimeAlignment time_alignment`
    pub time_alignment: BuildgpencilTimealignment,
    /// `float speed_fac`
    pub speed_fac: f32,
    /// `float speed_maxgap`
    pub speed_maxgap: f32,
    /// `eBuildGpencil_TimeMode time_mode`
    pub time_mode: BuildgpencilTimemode,
    /// `Object *object`
    pub object: usize,
    /// `float percentage_fac`
    pub percentage_fac: f32,
    /// `float fade_fac`
    pub fade_fac: f32,
    /// `char target_vgname[64]`
    pub target_vgname: [u8; 64],
    /// `float fade_opacity_strength`
    pub fade_opacity_strength: f32,
    /// `float fade_thickness_strength`
    pub fade_thickness_strength: f32,
}
impl Default for BuildGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            pass_index: 0,
            materialname: [0; 64],
            layer_pass: 0,
            start_frame: 1.0,
            end_frame: 125.0,
            start_delay: 0.0,
            length: 100.0,
            flag: Default::default(),
            mode: GP_BUILD_MODE_SEQUENTIAL,
            transition: GP_BUILD_TRANSITION_GROW,
            time_alignment: GP_BUILD_TIMEALIGN_START,
            speed_fac: 1.2,
            speed_maxgap: 0.5,
            time_mode: GP_BUILD_TIMEMODE_FRAMES,
            object: NIL,
            percentage_fac: 0.0,
            fade_fac: 0.0,
            target_vgname: [0; 64],
            fade_opacity_strength: 0.0,
            fade_thickness_strength: 0.0,
        }
    }
}

/// `LatticeGpencilModifierData`.
#[derive(Clone)]
pub struct LatticeGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Object *object`
    pub object: usize,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eLatticeGpencil_Flag flag`
    pub flag: LatticegpencilFlag,
    /// `float strength`
    pub strength: f32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `LatticeDeformData *cache_data`
    pub cache_data: usize,
}
impl Default for LatticeGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            strength: 1.0,
            layer_pass: 0,
            cache_data: NIL,
        }
    }
}

/// `LengthGpencilModifierData`.
#[derive(Clone)]
pub struct LengthGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `int flag`
    pub flag: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `float start_fac`
    pub start_fac: f32,
    /// `float end_fac`
    pub end_fac: f32,
    /// `float rand_start_fac`
    pub rand_start_fac: f32,
    /// `float rand_end_fac`
    pub rand_end_fac: f32,
    /// `float rand_offset`
    pub rand_offset: f32,
    /// `float overshoot_fac`
    pub overshoot_fac: f32,
    /// `int seed`
    pub seed: i32,
    /// `int step`
    pub step: i32,
    /// `int mode`
    pub mode: i32,
    /// `float point_density`
    pub point_density: f32,
    /// `float segment_influence`
    pub segment_influence: f32,
    /// `float max_angle`
    pub max_angle: f32,
}
impl Default for LengthGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            pass_index: 0,
            flag: GP_LENGTH_USE_CURVATURE,
            layer_pass: 0,
            start_fac: 0.1,
            end_fac: 0.1,
            rand_start_fac: 0.0,
            rand_end_fac: 0.0,
            rand_offset: 0.0,
            overshoot_fac: 0.1,
            seed: 0,
            step: 4,
            mode: 0,
            point_density: 30.0,
            segment_influence: 0.0,
            max_angle: 170.0_f32.to_radians(),
        }
    }
}

/// `DashGpencilModifierSegment`.
#[derive(Clone)]
pub struct DashGpencilModifierSegment {
    /// `char name[64]`
    pub name: [u8; 64],
    /// `DashGpencilModifierData *dmd`
    pub dmd: usize,
    /// `int dash`
    pub dash: i32,
    /// `int gap`
    pub gap: i32,
    /// `float radius`
    pub radius: f32,
    /// `float opacity`
    pub opacity: f32,
    /// `int mat_nr`
    pub mat_nr: i32,
    /// `int flag`
    pub flag: i32,
}
impl Default for DashGpencilModifierSegment {
    fn default() -> Self {
        Self {
            name: [0; 64],
            dmd: NIL,
            dash: 2,
            gap: 1,
            radius: 1.0,
            opacity: 1.0,
            mat_nr: -1,
            flag: 0,
        }
    }
}

/// `DashGpencilModifierData`.
#[derive(Clone)]
pub struct DashGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eDashGpencil_Flag flag`
    pub flag: DashgpencilFlag,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `int dash_offset`
    pub dash_offset: i32,
    /// `DashGpencilModifierSegment *segments`
    pub segments: usize,
    /// `int segments_len`
    pub segments_len: i32,
    /// `int segment_active_index`
    pub segment_active_index: i32,
}
impl Default for DashGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            layer_pass: 0,
            dash_offset: 0,
            segments: NIL,
            segments_len: 1,
            segment_active_index: 0,
        }
    }
}

/// `MirrorGpencilModifierData`.
#[derive(Clone)]
pub struct MirrorGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Object *object`
    pub object: usize,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eMirrorGpencil_Flag flag`
    pub flag: MirrorgpencilFlag,
    /// `int layer_pass`
    pub layer_pass: i32,
}
impl Default for MirrorGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            pass_index: 0,
            flag: GP_MIRROR_AXIS_X,
            layer_pass: 0,
        }
    }
}

/// `HookGpencilModifierData`.
#[derive(Clone)]
pub struct HookGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Object *object`
    pub object: usize,
    /// `Material *material`
    pub material: usize,
    /// `char subtarget[64]`
    pub subtarget: [u8; 64],
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `eHookGpencil_Flag flag`
    pub flag: HookgpencilFlag,
    /// `eHookGpencil_Falloff falloff_type`
    pub falloff_type: HookgpencilFalloff,
    /// `float parentinv[4][4]`
    pub parentinv: [[f32; 4]; 4],
    /// `float cent[3]`
    pub cent: [f32; 3],
    /// `float falloff`
    pub falloff: f32,
    /// `float force`
    pub force: f32,
    /// `CurveMapping *curfalloff`
    pub curfalloff: usize,
}
impl Default for HookGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            material: NIL,
            subtarget: [0; 64],
            layername: [0; 64],
            materialname: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            layer_pass: 0,
            flag: Default::default(),
            falloff_type: eGPHook_Falloff_Smooth,
            parentinv: [[0.0; 4]; 4],
            cent: [0.0; 3],
            falloff: 0.0,
            force: 0.5,
            curfalloff: NIL,
        }
    }
}

/// `SimplifyGpencilModifierData`.
#[derive(Clone)]
pub struct SimplifyGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eSimplifyGpencil_Flag flag`
    pub flag: SimplifygpencilFlag,
    /// `float factor`
    pub factor: f32,
    /// `eSimplifyGpencil_Mode mode`
    pub mode: SimplifygpencilMode,
    /// `short step`
    pub step: i16,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `float length`
    pub length: f32,
    /// `float sharp_threshold`
    pub sharp_threshold: f32,
    /// `float distance`
    pub distance: f32,
}
impl Default for SimplifyGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            factor: 0.0,
            mode: GP_SIMPLIFY_FIXED,
            step: 1,
            layer_pass: 0,
            length: 0.1,
            sharp_threshold: 0.0,
            distance: 0.1,
        }
    }
}

/// `OffsetGpencilModifierData`.
#[derive(Clone)]
pub struct OffsetGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eOffsetGpencil_Flag flag`
    pub flag: OffsetgpencilFlag,
    /// `float loc[3]`
    pub loc: [f32; 3],
    /// `float rot[3]`
    pub rot: [f32; 3],
    /// `float scale[3]`
    pub scale: [f32; 3],
    /// `float rnd_offset[3]`
    pub rnd_offset: [f32; 3],
    /// `float rnd_rot[3]`
    pub rnd_rot: [f32; 3],
    /// `float rnd_scale[3]`
    pub rnd_scale: [f32; 3],
    /// `int seed`
    pub seed: i32,
    /// `eOffsetGpencil_Mode mode`
    pub mode: OffsetgpencilMode,
    /// `int stroke_step`
    pub stroke_step: i32,
    /// `int stroke_start_offset`
    pub stroke_start_offset: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
}
impl Default for OffsetGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            loc: [0.0; 3],
            rot: [0.0; 3],
            scale: [0.0; 3],
            rnd_offset: [0.0; 3],
            rnd_rot: [0.0; 3],
            rnd_scale: [0.0; 3],
            seed: 0,
            mode: GP_OFFSET_RANDOM,
            stroke_step: 1,
            stroke_start_offset: 0,
            layer_pass: 0,
        }
    }
}

/// `SmoothGpencilModifierData`.
#[derive(Clone)]
pub struct SmoothGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eSmoothGpencil_Flag flag`
    pub flag: SmoothgpencilFlag,
    /// `float factor`
    pub factor: f32,
    /// `int step`
    pub step: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `CurveMapping *curve_intensity`
    pub curve_intensity: usize,
}
impl Default for SmoothGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: GP_SMOOTH_MOD_LOCATION,
            factor: 1.0,
            step: 1,
            layer_pass: 0,
            curve_intensity: NIL,
        }
    }
}

/// `ArmatureGpencilModifierData`.
#[derive(Clone)]
pub struct ArmatureGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `short deformflag`
    pub deformflag: i16,
    /// `short multi`
    pub multi: i16,
    /// `Object *object`
    pub object: usize,
    /// `char vgname[64]`
    pub vgname: [u8; 64],
}
impl Default for ArmatureGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            deformflag: ARM_DEF_VGROUP,
            multi: 0,
            object: NIL,
            vgname: [0; 64],
        }
    }
}

/// `MultiplyGpencilModifierData`.
#[derive(Clone)]
pub struct MultiplyGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `int flag`
    pub flag: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `eMultiplyGpencil_Flag flags`
    pub flags: MultiplygpencilFlag,
    /// `int duplications`
    pub duplications: i32,
    /// `float distance`
    pub distance: f32,
    /// `float offset`
    pub offset: f32,
    /// `float fading_center`
    pub fading_center: f32,
    /// `float fading_thickness`
    pub fading_thickness: f32,
    /// `float fading_opacity`
    pub fading_opacity: f32,
}
impl Default for MultiplyGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            pass_index: 0,
            flag: 0,
            layer_pass: 0,
            flags: Default::default(),
            duplications: 3,
            distance: 0.1,
            offset: 0.0,
            fading_center: 0.5,
            fading_thickness: 0.5,
            fading_opacity: 0.5,
        }
    }
}

/// `TintGpencilModifierData`.
#[derive(Clone)]
pub struct TintGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Object *object`
    pub object: usize,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `eTintGpencil_Flag flag`
    pub flag: TintgpencilFlag,
    /// `int mode`
    pub mode: i32,
    /// `float factor`
    pub factor: f32,
    /// `float radius`
    pub radius: f32,
    /// `float rgb[3]`
    pub rgb: [f32; 3],
    /// `eTintGpencil_Type type`
    pub type_: TintgpencilType,
    /// `CurveMapping *curve_intensity`
    pub curve_intensity: usize,
    /// `ColorBand *colorband`
    pub colorband: usize,
}
impl Default for TintGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            layer_pass: 0,
            flag: Default::default(),
            mode: GPPAINT_MODE_BOTH,
            factor: 0.5,
            radius: 1.0,
            rgb: [0.0; 3],
            type_: GP_TINT_UNIFORM,
            curve_intensity: NIL,
            colorband: NIL,
        }
    }
}

/// `TextureGpencilModifierData`.
#[derive(Clone)]
pub struct TextureGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char materialname[64]`
    pub materialname: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eTextureGpencil_Flag flag`
    pub flag: TexturegpencilFlag,
    /// `float uv_offset`
    pub uv_offset: f32,
    /// `float uv_scale`
    pub uv_scale: f32,
    /// `float fill_rotation`
    pub fill_rotation: f32,
    /// `float fill_offset[2]`
    pub fill_offset: [f32; 2],
    /// `float fill_scale`
    pub fill_scale: f32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `short fit_method`
    pub fit_method: i16,
    /// `eTextureGpencil_Mode mode`
    pub mode: TexturegpencilMode,
    /// `float alignment_rotation`
    pub alignment_rotation: f32,
}
impl Default for TextureGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            materialname: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            uv_offset: 0.0,
            uv_scale: 1.0,
            fill_rotation: 0.0,
            fill_offset: [0.0; 2],
            fill_scale: 1.0,
            layer_pass: 0,
            fit_method: GP_TEX_CONSTANT_LENGTH,
            mode: STROKE,
            alignment_rotation: 0.0,
        }
    }
}

/// `WeightProxGpencilModifierData`.
#[derive(Clone)]
pub struct WeightProxGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `char target_vgname[64]`
    pub target_vgname: [u8; 64],
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eWeightGpencil_Flag flag`
    pub flag: WeightgpencilFlag,
    /// `float min_weight`
    pub min_weight: f32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `float dist_start`
    pub dist_start: f32,
    /// `float dist_end`
    pub dist_end: f32,
    /// `Object *object`
    pub object: usize,
}
impl Default for WeightProxGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            target_vgname: [0; 64],
            material: NIL,
            layername: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            min_weight: 0.0,
            layer_pass: 0,
            dist_start: 0.0,
            dist_end: 20.0,
            object: NIL,
        }
    }
}

/// `WeightAngleGpencilModifierData`.
#[derive(Clone)]
pub struct WeightAngleGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `char target_vgname[64]`
    pub target_vgname: [u8; 64],
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eWeightGpencil_Flag flag`
    pub flag: WeightgpencilFlag,
    /// `float min_weight`
    pub min_weight: f32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `short axis`
    pub axis: i16,
    /// `short space`
    pub space: i16,
    /// `float angle`
    pub angle: f32,
}
impl Default for WeightAngleGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            target_vgname: [0; 64],
            material: NIL,
            layername: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            min_weight: 0.0,
            layer_pass: 0,
            axis: 1,
            space: 0,
            angle: 0.0,
        }
    }
}

/// `LineartGpencilModifierData`.
#[derive(Clone)]
pub struct LineartGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `char source_type`
    pub source_type: i8,
    /// `char use_multiple_levels`
    pub use_multiple_levels: i8,
    /// `short level_start`
    pub level_start: i16,
    /// `short level_end`
    pub level_end: i16,
    /// `Object *source_camera`
    pub source_camera: usize,
    /// `Object *light_contour_object`
    pub light_contour_object: usize,
    /// `Object *source_object`
    pub source_object: usize,
    /// `Collection *source_collection`
    pub source_collection: usize,
    /// `Material *target_material`
    pub target_material: usize,
    /// `char target_layer[64]`
    pub target_layer: [u8; 64],
    /// `char source_vertex_group[64]`
    pub source_vertex_group: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `float overscan`
    pub overscan: f32,
    /// `float shadow_camera_fov`
    pub shadow_camera_fov: f32,
    /// `float shadow_camera_size`
    pub shadow_camera_size: f32,
    /// `float shadow_camera_near`
    pub shadow_camera_near: f32,
    /// `float shadow_camera_far`
    pub shadow_camera_far: f32,
    /// `float opacity`
    pub opacity: f32,
    /// `short thickness`
    pub thickness: i16,
    /// `unsigned_char mask_switches`
    pub mask_switches: u8,
    /// `unsigned_char material_mask_bits`
    pub material_mask_bits: u8,
    /// `unsigned_char intersection_mask`
    pub intersection_mask: u8,
    /// `unsigned_char shadow_selection`
    pub shadow_selection: u8,
    /// `unsigned_char silhouette_selection`
    pub silhouette_selection: u8,
    /// `float crease_threshold`
    pub crease_threshold: f32,
    /// `float angle_splitting_threshold`
    pub angle_splitting_threshold: f32,
    /// `float chain_smooth_tolerance`
    pub chain_smooth_tolerance: f32,
    /// `float chaining_image_threshold`
    pub chaining_image_threshold: f32,
    /// `eLineArtGPencilModifierFlags flags`
    pub flags: Lineartgpencilmodifierflags,
    /// `float stroke_depth_offset`
    pub stroke_depth_offset: f32,
    /// `char level_start_override`
    pub level_start_override: i8,
    /// `char level_end_override`
    pub level_end_override: i8,
    /// `short edge_types_override`
    pub edge_types_override: i16,
    /// `char shadow_selection_override`
    pub shadow_selection_override: i8,
    /// `char shadow_use_silhouette_override`
    pub shadow_use_silhouette_override: i8,
    /// `LineartCache *cache`
    pub cache: usize,
    /// `LineartData *la_data_ptr`
    pub la_data_ptr: usize,
}
impl Default for LineartGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            source_type: 0,
            use_multiple_levels: 0,
            level_start: 0,
            level_end: 0,
            source_camera: NIL,
            light_contour_object: NIL,
            source_object: NIL,
            source_collection: NIL,
            target_material: NIL,
            target_layer: [0; 64],
            source_vertex_group: [0; 64],
            vgname: [0; 64],
            overscan: 0.1,
            shadow_camera_fov: 0.0,
            shadow_camera_size: 200.0,
            shadow_camera_near: 0.1,
            shadow_camera_far: 200.0,
            opacity: 1.0,
            thickness: 25,
            mask_switches: 0,
            material_mask_bits: 0,
            intersection_mask: 0,
            shadow_selection: 0,
            silhouette_selection: 0,
            crease_threshold: 140.0_f32.to_radians(),
            angle_splitting_threshold: 0.0,
            chain_smooth_tolerance: 0.0,
            chaining_image_threshold: 0.001,
            flags: Default::default(),
            stroke_depth_offset: 0.05,
            level_start_override: 0,
            level_end_override: 0,
            edge_types_override: 0,
            shadow_selection_override: 0,
            shadow_use_silhouette_override: 0,
            cache: NIL,
            la_data_ptr: NIL,
        }
    }
}

/// `ShrinkwrapGpencilModifierData`.
#[derive(Clone)]
pub struct ShrinkwrapGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Object *target`
    pub target: usize,
    /// `Object *aux_target`
    pub aux_target: usize,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eShrinkwrapGpencil_Flag flag`
    pub flag: ShrinkwrapgpencilFlag,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `float keep_dist`
    pub keep_dist: f32,
    /// `short shrink_type`
    pub shrink_type: i16,
    /// `char shrink_opts`
    pub shrink_opts: i8,
    /// `char shrink_mode`
    pub shrink_mode: i8,
    /// `float proj_limit`
    pub proj_limit: f32,
    /// `char proj_axis`
    pub proj_axis: i8,
    /// `char subsurf_levels`
    pub subsurf_levels: i8,
    /// `float smooth_factor`
    pub smooth_factor: f32,
    /// `int smooth_step`
    pub smooth_step: i32,
    /// `ShrinkwrapTreeData *cache_data`
    pub cache_data: usize,
}
impl Default for ShrinkwrapGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            target: NIL,
            aux_target: NIL,
            material: NIL,
            layername: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            layer_pass: 0,
            keep_dist: 0.05,
            shrink_type: MOD_SHRINKWRAP_NEAREST_SURFACE,
            shrink_opts: MOD_SHRINKWRAP_PROJECT_ALLOW_POS_DIR as i8,
            shrink_mode: 0,
            proj_limit: 0.0,
            proj_axis: 0,
            subsurf_levels: 0,
            smooth_factor: 0.05,
            smooth_step: 1,
            cache_data: NIL,
        }
    }
}

/// `EnvelopeGpencilModifierData`.
#[derive(Clone)]
pub struct EnvelopeGpencilModifierData {
    pub modifier: GpencilModifierData,
    /// `Material *material`
    pub material: usize,
    /// `char layername[64]`
    pub layername: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `int pass_index`
    pub pass_index: i32,
    /// `eEnvelopeGpencil_Flag flag`
    pub flag: EnvelopegpencilFlag,
    /// `eEnvelopeGpencil_Mode mode`
    pub mode: EnvelopegpencilMode,
    /// `int mat_nr`
    pub mat_nr: i32,
    /// `float thickness`
    pub thickness: f32,
    /// `float strength`
    pub strength: f32,
    /// `int skip`
    pub skip: i32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `int spread`
    pub spread: i32,
}
impl Default for EnvelopeGpencilModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            material: NIL,
            layername: [0; 64],
            vgname: [0; 64],
            pass_index: 0,
            flag: Default::default(),
            mode: GP_ENVELOPE_SEGMENTS,
            mat_nr: -1,
            thickness: 1.0,
            strength: 1.0,
            skip: 0,
            layer_pass: 0,
            spread: 10,
        }
    }
}
