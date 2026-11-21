//! Port of `DNA_anim_enums.h` — all F-curve / driver / NLA / keying enums.
#![allow(non_camel_case_types)]

/// `eFModifier_Types`.
pub type FModifierType = i16;
pub const FMODIFIER_TYPE_NULL: FModifierType = 0;
pub const FMODIFIER_TYPE_GENERATOR: FModifierType = 1;
pub const FMODIFIER_TYPE_FN_GENERATOR: FModifierType = 2;
pub const FMODIFIER_TYPE_ENVELOPE: FModifierType = 3;
pub const FMODIFIER_TYPE_CYCLES: FModifierType = 4;
pub const FMODIFIER_TYPE_NOISE: FModifierType = 5;
pub const FMODIFIER_TYPE_FILTER: FModifierType = 6;
pub const FMODIFIER_TYPE_PYTHON: FModifierType = 7;
pub const FMODIFIER_TYPE_LIMITS: FModifierType = 8;
pub const FMODIFIER_TYPE_STEPPED: FModifierType = 9;
pub const FMODIFIER_TYPE_SMOOTH: FModifierType = 10;
pub const FMODIFIER_NUM_TYPES: FModifierType = 11;

/// `eFModifier_Flags`.
pub const FMODIFIER_FLAG_DISABLED: i16 = 1 << 0;
pub const FMODIFIER_FLAG_EXPANDED: i16 = 1 << 1;
pub const FMODIFIER_FLAG_ACTIVE: i16 = 1 << 2;
pub const FMODIFIER_FLAG_MUTED: i16 = 1 << 3;
pub const FMODIFIER_FLAG_RANGERESTRICT: i16 = 1 << 4;
pub const FMODIFIER_FLAG_USEINFLUENCE: i16 = 1 << 5;

/// `eFMod_Generator_Modes`.
pub type FModGeneratorModes = i32;
pub const FCM_GENERATOR_POLYNOMIAL: FModGeneratorModes = 0;
pub const FCM_GENERATOR_POLYNOMIAL_FACTORISED: FModGeneratorModes = 1;

/// `eFMod_Generator_Flags`.
pub const FCM_GENERATOR_ADDITIVE: i32 = 1 << 0;

/// `eFMod_Generator_Functions`.
pub type FModGeneratorFunctions = i32;
pub const FCM_GENERATOR_FN_SIN: FModGeneratorFunctions = 0;
pub const FCM_GENERATOR_FN_COS: FModGeneratorFunctions = 1;
pub const FCM_GENERATOR_FN_TAN: FModGeneratorFunctions = 2;
pub const FCM_GENERATOR_FN_SQRT: FModGeneratorFunctions = 3;
pub const FCM_GENERATOR_FN_LN: FModGeneratorFunctions = 4;
pub const FCM_GENERATOR_FN_SINC: FModGeneratorFunctions = 5;

/// `eFMod_Cycling_Modes`.
pub type FModCyclingModes = i16;
pub const FCM_EXTRAPOLATE_NONE: FModCyclingModes = 0;
pub const FCM_EXTRAPOLATE_CYCLIC: FModCyclingModes = 1;
pub const FCM_EXTRAPOLATE_CYCLIC_OFFSET: FModCyclingModes = 2;
pub const FCM_EXTRAPOLATE_MIRROR: FModCyclingModes = 3;

/// `eFMod_Limit_Flags`.
pub const FCM_LIMIT_XMIN: i32 = 1 << 0;
pub const FCM_LIMIT_XMAX: i32 = 1 << 1;
pub const FCM_LIMIT_YMIN: i32 = 1 << 2;
pub const FCM_LIMIT_YMAX: i32 = 1 << 3;

/// `eFMod_Noise_Modifications`.
pub type FModNoiseModifications = i16;
pub const FCM_NOISE_MODIF_REPLACE: FModNoiseModifications = 0;
pub const FCM_NOISE_MODIF_ADD: FModNoiseModifications = 1;
pub const FCM_NOISE_MODIF_SUBTRACT: FModNoiseModifications = 2;
pub const FCM_NOISE_MODIF_MULTIPLY: FModNoiseModifications = 3;

/// `eFMod_Stepped_Flags`.
pub const FCM_STEPPED_NO_BEFORE: i32 = 1 << 0;
pub const FCM_STEPPED_NO_AFTER: i32 = 1 << 1;

/// `eDriverTarget_Options`.
pub type DriverTargetOptions = i16;
pub const DTAR_OPTION_USE_FALLBACK: DriverTargetOptions = 1 << 0;

/// `eDriverTarget_Flag`.
pub type DriverTargetFlag = i16;
pub const DTAR_FLAG_STRUCT_REF: DriverTargetFlag = 1 << 0;
pub const DTAR_FLAG_ID_OB_ONLY: DriverTargetFlag = 1 << 1;
pub const DTAR_FLAG_LOCALSPACE: DriverTargetFlag = 1 << 2;
pub const DTAR_FLAG_LOCAL_CONSTS: DriverTargetFlag = 1 << 3;
pub const DTAR_FLAG_INVALID: DriverTargetFlag = 1 << 4;
pub const DTAR_FLAG_FALLBACK_USED: DriverTargetFlag = 1 << 5;

/// `eDriverTarget_TransformChannels`.
pub type DriverTargetTransformChannels = i16;
pub const DTAR_TRANSCHAN_LOCX: DriverTargetTransformChannels = 0;
pub const DTAR_TRANSCHAN_LOCY: DriverTargetTransformChannels = 1;
pub const DTAR_TRANSCHAN_LOCZ: DriverTargetTransformChannels = 2;
pub const DTAR_TRANSCHAN_ROTX: DriverTargetTransformChannels = 3;
pub const DTAR_TRANSCHAN_ROTY: DriverTargetTransformChannels = 4;
pub const DTAR_TRANSCHAN_ROTZ: DriverTargetTransformChannels = 5;
pub const DTAR_TRANSCHAN_SCALEX: DriverTargetTransformChannels = 6;
pub const DTAR_TRANSCHAN_SCALEY: DriverTargetTransformChannels = 7;
pub const DTAR_TRANSCHAN_SCALEZ: DriverTargetTransformChannels = 8;
pub const DTAR_TRANSCHAN_SCALE_AVG: DriverTargetTransformChannels = 9;
pub const DTAR_TRANSCHAN_ROTW: DriverTargetTransformChannels = 10;
pub const MAX_DTAR_TRANSCHAN_TYPES: DriverTargetTransformChannels = 11;

/// `eDriverTarget_RotationMode`.
pub type DriverTargetRotationMode = i8;
pub const DTAR_ROTMODE_AUTO: DriverTargetRotationMode = 0;
pub const DTAR_ROTMODE_EULER_XYZ: DriverTargetRotationMode = 1;
pub const DTAR_ROTMODE_EULER_XZY: DriverTargetRotationMode = 2;
pub const DTAR_ROTMODE_EULER_YXZ: DriverTargetRotationMode = 3;
pub const DTAR_ROTMODE_EULER_YZX: DriverTargetRotationMode = 4;
pub const DTAR_ROTMODE_EULER_ZXY: DriverTargetRotationMode = 5;
pub const DTAR_ROTMODE_EULER_ZYX: DriverTargetRotationMode = 6;
pub const DTAR_ROTMODE_QUATERNION: DriverTargetRotationMode = 7;
pub const DTAR_ROTMODE_SWING_TWIST_X: DriverTargetRotationMode = 8;
pub const DTAR_ROTMODE_SWING_TWIST_Y: DriverTargetRotationMode = 9;
pub const DTAR_ROTMODE_SWING_TWIST_Z: DriverTargetRotationMode = 10;
pub const DTAR_ROTMODE_EULER_MIN: DriverTargetRotationMode = DTAR_ROTMODE_EULER_XYZ;
pub const DTAR_ROTMODE_EULER_MAX: DriverTargetRotationMode = DTAR_ROTMODE_EULER_ZYX;

/// `eDriverTarget_ContextProperty`.
pub type DriverTargetContextProperty = i32;
pub const DTAR_CONTEXT_PROPERTY_ACTIVE_SCENE: DriverTargetContextProperty = 0;
pub const DTAR_CONTEXT_PROPERTY_ACTIVE_VIEW_LAYER: DriverTargetContextProperty = 1;

pub const MAX_DRIVER_TARGETS: usize = 8;

/// `eDriverVar_Types`.
pub type DriverVarTypes = i8;
pub const DVAR_TYPE_SINGLE_PROP: DriverVarTypes = 0;
pub const DVAR_TYPE_ROT_DIFF: DriverVarTypes = 1;
pub const DVAR_TYPE_LOC_DIFF: DriverVarTypes = 2;
pub const DVAR_TYPE_TRANSFORM_CHAN: DriverVarTypes = 3;
pub const DVAR_TYPE_CONTEXT_PROP: DriverVarTypes = 4;
pub const MAX_DVAR_TYPES: DriverVarTypes = 5;

/// `eDriverVar_Flags`.
pub type DriverVarFlags = i16;
pub const DVAR_FLAG_ERROR: DriverVarFlags = 1 << 0;
pub const DVAR_FLAG_INVALID_NAME: DriverVarFlags = 1 << 1;
pub const DVAR_FLAG_INVALID_START_NUM: DriverVarFlags = 1 << 2;
pub const DVAR_FLAG_INVALID_START_CHAR: DriverVarFlags = 1 << 3;
pub const DVAR_FLAG_INVALID_HAS_SPACE: DriverVarFlags = 1 << 4;
pub const DVAR_FLAG_INVALID_HAS_DOT: DriverVarFlags = 1 << 5;
pub const DVAR_FLAG_INVALID_HAS_SPECIAL: DriverVarFlags = 1 << 6;
pub const DVAR_FLAG_INVALID_PY_KEYWORD: DriverVarFlags = 1 << 7;
pub const DVAR_FLAG_INVALID_EMPTY: DriverVarFlags = 1 << 8;
pub const DVAR_ALL_INVALID_FLAGS: DriverVarFlags = DVAR_FLAG_INVALID_NAME
    | DVAR_FLAG_INVALID_START_NUM
    | DVAR_FLAG_INVALID_START_CHAR
    | DVAR_FLAG_INVALID_HAS_SPACE
    | DVAR_FLAG_INVALID_HAS_DOT
    | DVAR_FLAG_INVALID_HAS_SPECIAL
    | DVAR_FLAG_INVALID_PY_KEYWORD
    | DVAR_FLAG_INVALID_EMPTY;

/// `eDriver_Types`.
pub type DriverTypes = i32;
pub const DRIVER_TYPE_AVERAGE: DriverTypes = 0;
pub const DRIVER_TYPE_PYTHON: DriverTypes = 1;
pub const DRIVER_TYPE_SUM: DriverTypes = 2;
pub const DRIVER_TYPE_MIN: DriverTypes = 3;
pub const DRIVER_TYPE_MAX: DriverTypes = 4;

/// `eDriver_Flags`.
pub type DriverFlags = i32;
pub const DRIVER_FLAG_INVALID: DriverFlags = 1 << 0;
pub const DRIVER_FLAG_DEPRECATED: DriverFlags = 1 << 1;
pub const DRIVER_FLAG_RECOMPILE: DriverFlags = 1 << 3;
pub const DRIVER_FLAG_RENAMEVAR: DriverFlags = 1 << 4;
pub const DRIVER_FLAG_PYTHON_BLOCKED: DriverFlags = 1 << 5;
pub const DRIVER_FLAG_USE_SELF: DriverFlags = 1 << 6;

pub const FCURVE_ACTIVE_KEYFRAME_NONE: i32 = -1;

/// `eFCurve_Flags`.
pub type FCurveFlags = i16;
pub const FCURVE_VISIBLE: FCurveFlags = 1 << 0;
pub const FCURVE_SELECTED: FCurveFlags = 1 << 1;
pub const FCURVE_ACTIVE: FCurveFlags = 1 << 2;
pub const FCURVE_PROTECTED: FCurveFlags = 1 << 3;
pub const FCURVE_MUTED: FCurveFlags = 1 << 4;
pub const FCURVE_AUTO_HANDLES: FCurveFlags = 1 << 5;
pub const FCURVE_MOD_OFF: FCurveFlags = 1 << 6;
pub const FCURVE_DISABLED: FCurveFlags = 1 << 10;
pub const FCURVE_INT_VALUES: FCurveFlags = 1 << 11;
pub const FCURVE_DISCRETE_VALUES: FCurveFlags = 1 << 12;
pub const FCURVE_TAGGED: FCurveFlags = 1 << 15;

/// `eFCurve_Extend`.
pub type FCurveExtend = i16;
pub const FCURVE_EXTRAPOLATE_CONSTANT: FCurveExtend = 0;
pub const FCURVE_EXTRAPOLATE_LINEAR: FCurveExtend = 1;

/// `eFCurve_Coloring`.
pub type FCurveColoring = i32;
pub const FCURVE_COLOR_AUTO_RAINBOW: FCurveColoring = 0;
pub const FCURVE_COLOR_AUTO_RGB: FCurveColoring = 1;
pub const FCURVE_COLOR_CUSTOM: FCurveColoring = 2;
pub const FCURVE_COLOR_AUTO_YRGB: FCurveColoring = 3;

/// `eFCurve_Smoothing`.
pub type FCurveSmoothing = i8;
pub const FCURVE_SMOOTH_NONE: FCurveSmoothing = 0;
pub const FCURVE_SMOOTH_CONT_ACCEL: FCurveSmoothing = 1;

/// `eNlaStrip_Blend_Mode`.
pub type NlaStripBlendMode = i16;
pub const NLASTRIP_MODE_REPLACE: NlaStripBlendMode = 0;
pub const NLASTRIP_MODE_ADD: NlaStripBlendMode = 1;
pub const NLASTRIP_MODE_SUBTRACT: NlaStripBlendMode = 2;
pub const NLASTRIP_MODE_MULTIPLY: NlaStripBlendMode = 3;
pub const NLASTRIP_MODE_COMBINE: NlaStripBlendMode = 4;

/// `eNlaStrip_Extrapolate_Mode`.
pub type NlaStripExtrapolateMode = i16;
pub const NLASTRIP_EXTEND_HOLD: NlaStripExtrapolateMode = 0;
pub const NLASTRIP_EXTEND_HOLD_FORWARD: NlaStripExtrapolateMode = 1;
pub const NLASTRIP_EXTEND_NOTHING: NlaStripExtrapolateMode = 2;

/// `eNlaStrip_Flag`.
pub type NlaStripFlag = u32;
pub const NLASTRIP_FLAG_ACTIVE: NlaStripFlag = 1 << 0;
pub const NLASTRIP_FLAG_SELECT: NlaStripFlag = 1 << 1;
pub const NLASTRIP_FLAG_TWEAKUSER: NlaStripFlag = 1 << 4;
pub const NLASTRIP_FLAG_USR_INFLUENCE: NlaStripFlag = 1 << 5;
pub const NLASTRIP_FLAG_USR_TIME: NlaStripFlag = 1 << 6;
pub const NLASTRIP_FLAG_USR_TIME_CYCLIC: NlaStripFlag = 1 << 7;
pub const NLASTRIP_FLAG_SYNC_LENGTH: NlaStripFlag = 1 << 9;
pub const NLASTRIP_FLAG_AUTO_BLENDS: NlaStripFlag = 1 << 10;
pub const NLASTRIP_FLAG_REVERSE: NlaStripFlag = 1 << 11;
pub const NLASTRIP_FLAG_MUTED: NlaStripFlag = 1 << 12;
pub const NLASTRIP_FLAG_INVALID_LOCATION: NlaStripFlag = 1 << 28;
pub const NLASTRIP_FLAG_NO_TIME_MAP: NlaStripFlag = 1 << 29;
pub const NLASTRIP_FLAG_TEMP_META: NlaStripFlag = 1 << 30;
pub const NLASTRIP_FLAG_EDIT_TOUCHED: NlaStripFlag = 1 << 31;

/// `eNlaStrip_Type`.
pub type NlaStripType = i16;
pub const NLASTRIP_TYPE_CLIP: NlaStripType = 0;
pub const NLASTRIP_TYPE_TRANSITION: NlaStripType = 1;
pub const NLASTRIP_TYPE_META: NlaStripType = 2;
pub const NLASTRIP_TYPE_SOUND: NlaStripType = 3;

/// `eNlaTrack_Flag`.
pub type NlaTrackFlag = i32;
pub const NLATRACK_ACTIVE: NlaTrackFlag = 1 << 0;
pub const NLATRACK_SELECTED: NlaTrackFlag = 1 << 1;
pub const NLATRACK_MUTED: NlaTrackFlag = 1 << 2;
pub const NLATRACK_SOLO: NlaTrackFlag = 1 << 3;
pub const NLATRACK_PROTECTED: NlaTrackFlag = 1 << 4;
pub const NLATRACK_DISABLED: NlaTrackFlag = 1 << 10;
pub const NLATRACK_TEMPORARILY_ADDED: NlaTrackFlag = 1 << 11;
pub const NLATRACK_OVERRIDELIBRARY_LOCAL: NlaTrackFlag = 1 << 16;

/// `eKS_Settings`.
pub type KsSettings = i16;
pub const KEYINGSET_ABSOLUTE: KsSettings = 1 << 1;

/// `eInsertKeyFlags`.
pub type InsertKeyFlags = i16;
pub const INSERTKEY_NOFLAGS: InsertKeyFlags = 0;
pub const INSERTKEY_NEEDED: InsertKeyFlags = 1 << 0;
pub const INSERTKEY_MATRIX: InsertKeyFlags = 1 << 1;
pub const INSERTKEY_FAST: InsertKeyFlags = 1 << 2;
pub const INSERTKEY_REPLACE: InsertKeyFlags = 1 << 4;
pub const INSERTKEY_NO_USERPREF: InsertKeyFlags = 1 << 6;
pub const INSERTKEY_OVERWRITE_FULL: InsertKeyFlags = 1 << 7;
pub const INSERTKEY_CYCLE_AWARE: InsertKeyFlags = 1 << 9;
pub const INSERTKEY_AVAILABLE: InsertKeyFlags = 1 << 10;

/// `eKSP_Settings`.
pub const KSP_FLAG_WHOLE_ARRAY: i16 = 1 << 0;

/// `eKSP_Grouping`.
pub type KspGrouping = i16;
pub const KSP_GROUP_NAMED: KspGrouping = 0;
pub const KSP_GROUP_NONE: KspGrouping = 1;
pub const KSP_GROUP_KSNAME: KspGrouping = 2;

/// `eAnimData_Flag`.
pub type AnimDataFlag = i32;
pub const ADT_NLA_SOLO_TRACK: AnimDataFlag = 1 << 0;
pub const ADT_NLA_EVAL_OFF: AnimDataFlag = 1 << 1;
pub const ADT_NLA_EDIT_ON: AnimDataFlag = 1 << 2;
pub const ADT_NLA_EDIT_NOMAP: AnimDataFlag = 1 << 3;
pub const ADT_NLA_SKEYS_COLLAPSED: AnimDataFlag = 1 << 4;
pub const ADT_NLA_EVAL_UPPER_TRACKS: AnimDataFlag = 1 << 5;
pub const ADT_DRIVERS_COLLAPSED: AnimDataFlag = 1 << 10;
pub const ADT_UI_SELECTED: AnimDataFlag = 1 << 14;
pub const ADT_UI_ACTIVE: AnimDataFlag = 1 << 15;
pub const ADT_CURVES_NOT_VISIBLE: AnimDataFlag = 1 << 16;
pub const ADT_CURVES_ALWAYS_VISIBLE: AnimDataFlag = 1 << 17;
pub const ADT_UI_EXPANDED: AnimDataFlag = 1 << 18;
