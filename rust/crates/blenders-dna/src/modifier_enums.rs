//! Port of `DNA_modifier_enums.h` — mechanically converted enum constants.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
/// `eArrayGpencil_Flag`.
pub type ArraygpencilFlag = i32;
pub const GP_ARRAY_INVERT_LAYER: ArraygpencilFlag = 1 << 2;
pub const GP_ARRAY_INVERT_PASS: ArraygpencilFlag = 1 << 3;
pub const GP_ARRAY_INVERT_LAYERPASS: ArraygpencilFlag = 1 << 5;
pub const GP_ARRAY_INVERT_MATERIAL: ArraygpencilFlag = 1 << 6;
pub const GP_ARRAY_USE_OFFSET: ArraygpencilFlag = 1 << 7;
pub const GP_ARRAY_USE_RELATIVE: ArraygpencilFlag = 1 << 8;
pub const GP_ARRAY_USE_OB_OFFSET: ArraygpencilFlag = 1 << 9;
pub const GP_ARRAY_UNIFORM_RANDOM_SCALE: ArraygpencilFlag = 1 << 10;

/// `eTextureGpencil_Fit`.
pub type TexturegpencilFit = i16;
pub const GP_TEX_FIT_STROKE: TexturegpencilFit = 0;
pub const GP_TEX_CONSTANT_LENGTH: TexturegpencilFit = 1;

/// `eNoiseGpencil_Flag`.
pub type NoisegpencilFlag = i32;
pub const GP_NOISE_USE_RANDOM: NoisegpencilFlag = 1 << 0;
pub const GP_NOISE_MOD_LOCATION: NoisegpencilFlag = 1 << 1;
pub const GP_NOISE_MOD_STRENGTH: NoisegpencilFlag = 1 << 2;
pub const GP_NOISE_MOD_THICKNESS: NoisegpencilFlag = 1 << 3;
pub const GP_NOISE_FULL_STROKE: NoisegpencilFlag = 1 << 4;
pub const GP_NOISE_CUSTOM_CURVE: NoisegpencilFlag = 1 << 5;
pub const GP_NOISE_INVERT_LAYER: NoisegpencilFlag = 1 << 6;
pub const GP_NOISE_INVERT_PASS: NoisegpencilFlag = 1 << 7;
pub const GP_NOISE_INVERT_VGROUP: NoisegpencilFlag = 1 << 8;
pub const GP_NOISE_MOD_UV: NoisegpencilFlag = 1 << 9;
pub const GP_NOISE_INVERT_LAYERPASS: NoisegpencilFlag = 1 << 10;
pub const GP_NOISE_INVERT_MATERIAL: NoisegpencilFlag = 1 << 11;

/// `eNoiseRandomGpencil_Mode`.
pub type NoiserandomgpencilMode = i16;
pub const GP_NOISE_RANDOM_STEP: NoiserandomgpencilMode = 0;
pub const GP_NOISE_RANDOM_KEYFRAME: NoiserandomgpencilMode = 1;

/// `eLengthGpencil_Flag`.
pub type LengthgpencilFlag = i32;
pub const GP_LENGTH_INVERT_LAYER: LengthgpencilFlag = 1 << 0;
pub const GP_LENGTH_INVERT_PASS: LengthgpencilFlag = 1 << 1;
pub const GP_LENGTH_INVERT_LAYERPASS: LengthgpencilFlag = 1 << 2;
pub const GP_LENGTH_INVERT_MATERIAL: LengthgpencilFlag = 1 << 3;
pub const GP_LENGTH_USE_CURVATURE: LengthgpencilFlag = 1 << 4;
pub const GP_LENGTH_INVERT_CURVATURE: LengthgpencilFlag = 1 << 5;
pub const GP_LENGTH_USE_RANDOM: LengthgpencilFlag = 1 << 6;

/// `eLengthGpencil_Type`.
pub type LengthgpencilType = i32;
pub const GP_LENGTH_RELATIVE: LengthgpencilType = 0;
pub const GP_LENGTH_ABSOLUTE: LengthgpencilType = 1;

/// `eShrinkwrap_Type`.
pub type ShrinkwrapType = i16;
pub const MOD_SHRINKWRAP_NEAREST_SURFACE: ShrinkwrapType = 0;
pub const MOD_SHRINKWRAP_PROJECT: ShrinkwrapType = 1;
pub const MOD_SHRINKWRAP_NEAREST_VERTEX: ShrinkwrapType = 2;
pub const MOD_SHRINKWRAP_TARGET_PROJECT: ShrinkwrapType = 3;

/// `eShrinkwrap_Mode`.
pub type ShrinkwrapMode = i8;
pub const MOD_SHRINKWRAP_ON_SURFACE: ShrinkwrapMode = 0;
pub const MOD_SHRINKWRAP_INSIDE: ShrinkwrapMode = 1;
pub const MOD_SHRINKWRAP_OUTSIDE: ShrinkwrapMode = 2;
pub const MOD_SHRINKWRAP_OUTSIDE_SURFACE: ShrinkwrapMode = 3;
pub const MOD_SHRINKWRAP_ABOVE_SURFACE: ShrinkwrapMode = 4;

/// `eShrinkwrap_Opts`.
pub type ShrinkwrapOpts = u8;
pub const MOD_SHRINKWRAP_PROJECT_ALLOW_POS_DIR: ShrinkwrapOpts = 1 << 0;
pub const MOD_SHRINKWRAP_PROJECT_ALLOW_NEG_DIR: ShrinkwrapOpts = 1 << 1;
pub const MOD_SHRINKWRAP_CULL_TARGET_FRONTFACE: ShrinkwrapOpts = 1 << 3;
pub const MOD_SHRINKWRAP_CULL_TARGET_BACKFACE: ShrinkwrapOpts = 1 << 4;
pub const MOD_SHRINKWRAP_KEEP_ABOVE_SURFACE: ShrinkwrapOpts = 1 << 5;
pub const MOD_SHRINKWRAP_INVERT_VGROUP: ShrinkwrapOpts = 1 << 6;
pub const MOD_SHRINKWRAP_INVERT_CULL_TARGET: ShrinkwrapOpts = 1 << 7;

/// `eShrinkwrap_ProjAxis`.
pub type ShrinkwrapProjaxis = i8;
pub const MOD_SHRINKWRAP_PROJECT_OVER_NORMAL: ShrinkwrapProjaxis = 0;
pub const MOD_SHRINKWRAP_PROJECT_OVER_X_AXIS: ShrinkwrapProjaxis = 1 << 0;
pub const MOD_SHRINKWRAP_PROJECT_OVER_Y_AXIS: ShrinkwrapProjaxis = 1 << 1;
pub const MOD_SHRINKWRAP_PROJECT_OVER_Z_AXIS: ShrinkwrapProjaxis = 1 << 2;

/// `eDTType`.
pub type Dttype = i32;
pub const DT_TYPE_MDEFORMVERT: Dttype = 1 << 0;
pub const DT_TYPE_SKIN: Dttype = 1 << 2;
pub const DT_TYPE_BWEIGHT_VERT: Dttype = 1 << 3;
pub const DT_TYPE_SHARP_EDGE: Dttype = 1 << 8;
pub const DT_TYPE_SEAM: Dttype = 1 << 9;
pub const DT_TYPE_CREASE: Dttype = 1 << 10;
pub const DT_TYPE_BWEIGHT_EDGE: Dttype = 1 << 11;
pub const DT_TYPE_FREESTYLE_EDGE: Dttype = 1 << 12;
pub const DT_TYPE_MPROPCOL_VERT: Dttype = 1 << 16;
pub const DT_TYPE_LNOR: Dttype = 1 << 17;
pub const DT_TYPE_UV: Dttype = 1 << 24;
pub const DT_TYPE_SHARP_FACE: Dttype = 1 << 25;
pub const DT_TYPE_FREESTYLE_FACE: Dttype = 1 << 26;
pub const DT_TYPE_MLOOPCOL_VERT: Dttype = 1 << 27;
pub const DT_TYPE_MPROPCOL_LOOP: Dttype = 1 << 28;
pub const DT_TYPE_MLOOPCOL_LOOP: Dttype = 1 << 29;
pub const DT_TYPE_VCOL_ALL: Dttype = (1 << 16) | (1 << 27) | (1 << 28) | (1 << 29);
pub const DT_TYPE_VERT_ALL: Dttype = DT_TYPE_MDEFORMVERT | DT_TYPE_SKIN | DT_TYPE_BWEIGHT_VERT |
                     DT_TYPE_MPROPCOL_VERT | DT_TYPE_MLOOPCOL_VERT;
pub const DT_TYPE_EDGE_ALL: Dttype = DT_TYPE_SHARP_EDGE | DT_TYPE_SEAM | DT_TYPE_CREASE | DT_TYPE_BWEIGHT_EDGE |
                     DT_TYPE_FREESTYLE_EDGE;
pub const DT_TYPE_LOOP_ALL: Dttype = DT_TYPE_LNOR | DT_TYPE_UV | DT_TYPE_MPROPCOL_LOOP | DT_TYPE_MLOOPCOL_LOOP;
pub const DT_TYPE_POLY_ALL: Dttype = DT_TYPE_UV | DT_TYPE_SHARP_FACE | DT_TYPE_FREESTYLE_FACE;

/// `eDTMultilayerIndex`.
pub type Dtmultilayerindex = i32;
pub const DT_MULTILAYER_INDEX_INVALID: Dtmultilayerindex = -1;
pub const DT_MULTILAYER_INDEX_MDEFORMVERT: Dtmultilayerindex = 0;
pub const DT_MULTILAYER_INDEX_VCOL_LOOP: Dtmultilayerindex = 2;
pub const DT_MULTILAYER_INDEX_UV: Dtmultilayerindex = 3;
pub const DT_MULTILAYER_INDEX_VCOL_VERT: Dtmultilayerindex = 4;
pub const DT_MULTILAYER_INDEX_MAX: Dtmultilayerindex = 5;

/// `eDTLayersSrc`.
pub type Dtlayerssrc = i32;
pub const DT_LAYERS_ACTIVE_SRC: Dtlayerssrc = -1;
pub const DT_LAYERS_ALL_SRC: Dtlayerssrc = -2;
pub const DT_LAYERS_VGROUP_SRC: Dtlayerssrc = 1 << 8;
pub const DT_LAYERS_VGROUP_SRC_BONE_SELECT: Dtlayerssrc = -(DT_LAYERS_VGROUP_SRC | 1);
pub const DT_LAYERS_VGROUP_SRC_BONE_DEFORM: Dtlayerssrc = -(DT_LAYERS_VGROUP_SRC | 2);

/// `eDTLayersDst`.
pub type Dtlayersdst = i32;
pub const DT_LAYERS_ACTIVE_DST: Dtlayersdst = -1;
pub const DT_LAYERS_NAME_DST: Dtlayersdst = -2;
pub const DT_LAYERS_INDEX_DST: Dtlayersdst = -3;

/// `eMRemapMode`.
pub type Mremapmode = i32;
pub const MREMAP_USE_VERT: Mremapmode = 1 << 4;
pub const MREMAP_USE_EDGE: Mremapmode = 1 << 5;
pub const MREMAP_USE_LOOP: Mremapmode = 1 << 6;
pub const MREMAP_USE_POLY: Mremapmode = 1 << 7;
pub const MREMAP_USE_NEAREST: Mremapmode = 1 << 8;
pub const MREMAP_USE_NORPROJ: Mremapmode = 1 << 9;
pub const MREMAP_USE_INTERP: Mremapmode = 1 << 10;
pub const MREMAP_USE_NORMAL: Mremapmode = 1 << 11;
pub const MREMAP_MODE_VERT: Mremapmode = 1 << 24;
pub const MREMAP_MODE_VERT_NEAREST: Mremapmode = MREMAP_MODE_VERT | MREMAP_USE_VERT | MREMAP_USE_NEAREST;
pub const MREMAP_MODE_VERT_EDGE_NEAREST: Mremapmode = MREMAP_MODE_VERT | MREMAP_USE_EDGE | MREMAP_USE_NEAREST;
pub const MREMAP_MODE_VERT_EDGEINTERP_NEAREST: Mremapmode = MREMAP_MODE_VERT | MREMAP_USE_EDGE | MREMAP_USE_NEAREST |
                                        MREMAP_USE_INTERP;
pub const MREMAP_MODE_VERT_FACE_NEAREST: Mremapmode = MREMAP_MODE_VERT | MREMAP_USE_POLY | MREMAP_USE_NEAREST;
pub const MREMAP_MODE_VERT_POLYINTERP_NEAREST: Mremapmode = MREMAP_MODE_VERT | MREMAP_USE_POLY | MREMAP_USE_NEAREST |
                                        MREMAP_USE_INTERP;
pub const MREMAP_MODE_VERT_POLYINTERP_VNORPROJ: Mremapmode = MREMAP_MODE_VERT | MREMAP_USE_POLY | MREMAP_USE_NORPROJ |
                                         MREMAP_USE_INTERP;
pub const MREMAP_MODE_EDGE: Mremapmode = 1 << 25;
pub const MREMAP_MODE_EDGE_VERT_NEAREST: Mremapmode = MREMAP_MODE_EDGE | MREMAP_USE_VERT | MREMAP_USE_NEAREST;
pub const MREMAP_MODE_EDGE_NEAREST: Mremapmode = MREMAP_MODE_EDGE | MREMAP_USE_EDGE | MREMAP_USE_NEAREST;
pub const MREMAP_MODE_EDGE_POLY_NEAREST: Mremapmode = MREMAP_MODE_EDGE | MREMAP_USE_POLY | MREMAP_USE_NEAREST;
pub const MREMAP_MODE_EDGE_EDGEINTERP_VNORPROJ: Mremapmode = MREMAP_MODE_EDGE | MREMAP_USE_VERT | MREMAP_USE_NORPROJ |
                                         MREMAP_USE_INTERP;
pub const MREMAP_MODE_LOOP: Mremapmode = 1 << 26;
pub const MREMAP_MODE_LOOP_NEAREST_LOOPNOR: Mremapmode = MREMAP_MODE_LOOP | MREMAP_USE_LOOP | MREMAP_USE_VERT |
                                     MREMAP_USE_NEAREST | MREMAP_USE_NORMAL;
pub const MREMAP_MODE_LOOP_NEAREST_POLYNOR: Mremapmode = MREMAP_MODE_LOOP | MREMAP_USE_POLY | MREMAP_USE_VERT |
                                     MREMAP_USE_NEAREST | MREMAP_USE_NORMAL;
pub const MREMAP_MODE_LOOP_POLY_NEAREST: Mremapmode = MREMAP_MODE_LOOP | MREMAP_USE_POLY | MREMAP_USE_NEAREST;
pub const MREMAP_MODE_LOOP_POLYINTERP_NEAREST: Mremapmode = MREMAP_MODE_LOOP | MREMAP_USE_POLY | MREMAP_USE_NEAREST |
                                        MREMAP_USE_INTERP;
pub const MREMAP_MODE_LOOP_POLYINTERP_LNORPROJ: Mremapmode = MREMAP_MODE_LOOP | MREMAP_USE_POLY | MREMAP_USE_NORPROJ |
                                         MREMAP_USE_INTERP;
pub const MREMAP_MODE_POLY: Mremapmode = 1 << 27;
pub const MREMAP_MODE_POLY_NEAREST: Mremapmode = MREMAP_MODE_POLY | MREMAP_USE_POLY | MREMAP_USE_NEAREST;
pub const MREMAP_MODE_POLY_NOR: Mremapmode = MREMAP_MODE_POLY | MREMAP_USE_POLY | MREMAP_USE_NORMAL;
pub const MREMAP_MODE_POLY_POLYINTERP_PNORPROJ: Mremapmode = MREMAP_MODE_POLY | MREMAP_USE_POLY | MREMAP_USE_NORPROJ |
                                         MREMAP_USE_INTERP;
pub const MREMAP_MODE_TOPOLOGY: Mremapmode = MREMAP_MODE_VERT | MREMAP_MODE_EDGE | MREMAP_MODE_LOOP | MREMAP_MODE_POLY;

/// `eCDTMixMode`.
pub type Cdtmixmode = i32;
pub const CDT_MIX_NOMIX: Cdtmixmode = -1;
pub const CDT_MIX_TRANSFER: Cdtmixmode = 0;
pub const CDT_MIX_REPLACE_ABOVE_THRESHOLD: Cdtmixmode = 1;
pub const CDT_MIX_REPLACE_BELOW_THRESHOLD: Cdtmixmode = 2;
pub const CDT_MIX_MIX: Cdtmixmode = 16;
pub const CDT_MIX_ADD: Cdtmixmode = 17;
pub const CDT_MIX_SUB: Cdtmixmode = 18;
pub const CDT_MIX_MUL: Cdtmixmode = 19;

