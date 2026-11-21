//! Port of `DNA_anim_types.h` — `AnimData`, `FCurve`, `ChannelDriver`,
//! `NlaStrip`/`NlaTrack`, `KeyingSet`, `AnimOverride` + related f-mod structs.
#![allow(non_camel_case_types)]
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

pub use crate::anim_enums::{
    AnimDataFlag, DriverFlags, DriverTargetContextProperty, DriverTargetFlag,
    DriverTargetOptions, DriverTargetRotationMode, DriverTargetTransformChannels, DriverTypes,
    DriverVarFlags, DriverVarTypes, FCurveColoring, FCurveExtend, FCurveFlags, FCurveSmoothing,
    FModCyclingModes, FModGeneratorFunctions, FModGeneratorModes, FModNoiseModifications,
    InsertKeyFlags, KsSettings, KspGrouping, NlaStripBlendMode, NlaStripExtrapolateMode,
    NlaStripFlag, NlaStripType, NlaTrackFlag,
};
pub type FModifierTypes = i16;
pub type FModifierFlags = i16;
pub type FModGeneratorFlags = i32;
pub type FModLimitFlags = i32;
pub type FModSteppedFlags = i32;
pub type KspSettings = i16;

/// `BezTriple` (DNA_curve_types.h) — placeholder until curve types port.
#[derive(Clone, Copy, Default)]
pub struct BezTriple(pub [f32; 24]);

/// `FModifier`.
#[derive(Clone)]
pub struct FModifier {
    pub next: usize,
    pub prev: usize,
    /// `FCurve *curve`.
    pub curve: usize,
    /// `void *data` → type-specific payload ref.
    pub data: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub type_: FModifierTypes,
    pub flag: FModifierFlags,
    pub ui_expand_flag: i16,
    pub influence: f32,
    pub sfra: f32,
    pub efra: f32,
    pub blendin: f32,
    pub blendout: f32,
}
impl Default for FModifier {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            curve: NIL,
            data: NIL,
            name: [0; 64],
            type_: 0,
            flag: 0,
            ui_expand_flag: 0,
            influence: 0.0,
            sfra: 0.0,
            efra: 0.0,
            blendin: 0.0,
            blendout: 0.0,
        }
    }
}

/// `FMod_Generator`.
#[derive(Clone, Default)]
pub struct FModGenerator {
    /// `float *coefficients` → Vec.
    pub coefficients: Vec<f32>,
    pub arraysize: u32,
    pub poly_order: i32,
    pub mode: FModGeneratorModes,
    pub flag: FModGeneratorFlags,
}

/// `FMod_FunctionGenerator`.
#[derive(Clone, Default)]
pub struct FModFunctionGenerator {
    pub amplitude: f32,
    pub phase_multiplier: f32,
    pub phase_offset: f32,
    pub value_offset: f32,
    pub type_: FModGeneratorFunctions,
    pub flag: FModGeneratorFlags,
}

/// `FCM_EnvelopeData`.
#[derive(Clone, Copy, Default)]
pub struct FcmEnvelopeData {
    pub min: f32,
    pub max: f32,
    pub time: f32,
    pub f1: i16,
    pub f2: i16,
}

/// `FMod_Envelope`.
#[derive(Clone, Default)]
pub struct FModEnvelope {
    /// `FCM_EnvelopeData *data` → Vec.
    pub data: Vec<FcmEnvelopeData>,
    pub totvert: i32,
    pub midval: f32,
    pub min: f32,
    pub max: f32,
}

/// `FMod_Cycles`.
#[derive(Clone, Default)]
pub struct FModCycles {
    pub before_mode: FModCyclingModes,
    pub after_mode: FModCyclingModes,
    pub before_cycles: i16,
    pub after_cycles: i16,
}

/// `FMod_Limits`.
#[derive(Clone, Default)]
pub struct FModLimits {
    pub rect: crate::vec_types::Rctf,
    pub flag: FModLimitFlags,
}

/// `FMod_Noise`.
#[derive(Clone, Default)]
pub struct FModNoise {
    pub size: f32,
    pub strength: f32,
    pub phase: f32,
    pub offset: f32,
    pub roughness: f32,
    pub lacunarity: f32,
    pub depth: i16,
    pub modification: FModNoiseModifications,
    pub legacy_noise: i8,
}

/// `FMod_Stepped`.
#[derive(Clone, Default)]
pub struct FModStepped {
    pub step_size: f32,
    pub offset: f32,
    pub start_frame: f32,
    pub end_frame: f32,
    pub flag: FModSteppedFlags,
}

/// `FMod_Smooth`.
#[derive(Clone, Copy, Default)]
pub struct FModSmooth {
    pub sigma: f32,
    pub filter_width: i32,
}

/// `DriverTarget`.
#[derive(Clone)]
pub struct DriverTarget {
    /// `ID *id`.
    pub id: usize,
    /// `char *rna_path`.
    pub rna_path: Option<String>,
    /// `char pchan_name[64]`.
    pub pchan_name: [u8; 64],
    pub trans_chan: DriverTargetTransformChannels,
    pub rotation_mode: DriverTargetRotationMode,
    pub flag: DriverTargetFlag,
    pub options: DriverTargetOptions,
    pub idtype: i32,
    pub context_property: DriverTargetContextProperty,
    pub fallback_value: f32,
}
impl Default for DriverTarget {
    fn default() -> Self {
        Self {
            id: NIL,
            rna_path: None,
            pchan_name: [0; 64],
            trans_chan: 0,
            rotation_mode: 0,
            flag: 0,
            options: 0,
            idtype: 0,
            context_property: 0,
            fallback_value: 0.0,
        }
    }
}

/// `DriverVar`.
#[derive(Clone)]
pub struct DriverVar {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `DriverTarget targets[8]`.
    pub targets: [DriverTarget; 8],
    pub num_targets: i8,
    pub type_: DriverVarTypes,
    pub flag: DriverVarFlags,
    pub curval: f32,
}
impl Default for DriverVar {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            targets: Default::default(),
            num_targets: 0,
            type_: 0,
            flag: 0,
            curval: 0.0,
        }
    }
}

/// `ChannelDriver`.
#[derive(Clone)]
pub struct ChannelDriver {
    /// `ListBaseT<DriverVar>`.
    pub variables: ListBase,
    /// `char expression[256]`.
    pub expression: [u8; 256],
    /// `void *expr_comp`, `ExprPyLike_Parsed *expr_simple` — opaque.
    pub expr_comp: usize,
    pub expr_simple: usize,
    pub curval: f32,
    pub influence: f32,
    pub type_: DriverTypes,
    pub flag: DriverFlags,
}
impl Default for ChannelDriver {
    fn default() -> Self {
        Self {
            variables: ListBase::new(),
            expression: [0; 256],
            expr_comp: NIL,
            expr_simple: NIL,
            curval: 0.0,
            influence: 0.0,
            type_: 0,
            flag: 0,
        }
    }
}

/// `FPoint`.
#[derive(Clone, Copy, Default)]
pub struct FPoint {
    pub vec: [f32; 2],
    pub flag: i32,
}

/// `FCurve`.
#[derive(Clone)]
pub struct FCurve {
    pub next: usize,
    pub prev: usize,
    /// `bActionGroup *grp`.
    pub grp: usize,
    /// `ChannelDriver *driver`.
    pub driver: usize,
    /// `ListBaseT<FModifier>`.
    pub modifiers: ListBase,
    /// `BezTriple *bezt` → Vec.
    pub bezt: Vec<BezTriple>,
    /// `FPoint *fpt` → Vec.
    pub fpt: Vec<FPoint>,
    pub totvert: u32,
    pub active_keyframe_index: i32,
    pub flag: FCurveFlags,
    pub extend: FCurveExtend,
    pub auto_smoothing: FCurveSmoothing,
    pub local_view_bits: u16,
    pub array_index: i32,
    /// `char *rna_path_ptr`.
    pub rna_path_ptr: Option<String>,
    pub color_mode: FCurveColoring,
    pub color: [f32; 3],
    pub prev_norm_factor: f32,
    pub prev_offset: f32,
    /// `bke::FCurveRuntime *runtime`.
    pub runtime: usize,
}
impl Default for FCurve {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            grp: NIL,
            driver: NIL,
            modifiers: ListBase::new(),
            bezt: Vec::new(),
            fpt: Vec::new(),
            totvert: 0,
            active_keyframe_index: 0,
            flag: 0,
            extend: 0,
            auto_smoothing: 0,
            local_view_bits: 0,
            array_index: 0,
            rna_path_ptr: None,
            color_mode: 0,
            color: [0.0; 3],
            prev_norm_factor: 0.0,
            prev_offset: 0.0,
            runtime: NIL,
        }
    }
}
impl FCurve {
    /// `rna_path()` → path string.
    pub fn rna_path(&self) -> &str {
        self.rna_path_ptr.as_deref().unwrap_or("")
    }
    /// `rna_path_set`.
    pub fn rna_path_set(&mut self, path: &str) {
        self.rna_path_ptr = Some(path.to_string());
    }
}

/// `NlaStrip`.
#[derive(Clone)]
pub struct NlaStrip {
    pub next: usize,
    pub prev: usize,
    /// `ListBaseT<NlaStrip>` children.
    pub strips: ListBase,
    /// `bAction *act`.
    pub act: usize,
    pub action_slot_handle: i32,
    /// `char last_slot_identifier[258]`.
    pub last_slot_identifier: [u8; 258],
    /// `ListBaseT<FCurve>`.
    pub fcurves: ListBase,
    /// `ListBaseT<FModifier>`.
    pub modifiers: ListBase,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub influence: f32,
    pub strip_time: f32,
    pub start: f32,
    pub end: f32,
    pub actstart: f32,
    pub actend: f32,
    pub repeat: f32,
    pub scale: f32,
    pub blendin: f32,
    pub blendout: f32,
    pub blendmode: NlaStripBlendMode,
    pub extendmode: NlaStripExtrapolateMode,
    pub type_: NlaStripType,
    pub flag: NlaStripFlag,
    /// `NlaStrip *orig_strip`.
    pub orig_strip: usize,
    /// `NlaStripRuntime *runtime`.
    pub runtime: usize,
}
impl Default for NlaStrip {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            strips: ListBase::new(),
            act: NIL,
            action_slot_handle: 0,
            last_slot_identifier: [0; 258],
            fcurves: ListBase::new(),
            modifiers: ListBase::new(),
            name: [0; 64],
            influence: 0.0,
            strip_time: 0.0,
            start: 0.0,
            end: 0.0,
            actstart: 0.0,
            actend: 0.0,
            repeat: 0.0,
            scale: 0.0,
            blendin: 0.0,
            blendout: 0.0,
            blendmode: 0,
            extendmode: 0,
            type_: 0,
            flag: 0,
            orig_strip: NIL,
            runtime: NIL,
        }
    }
}

/// `NlaTrack`.
#[derive(Clone)]
pub struct NlaTrack {
    pub next: usize,
    pub prev: usize,
    /// `ListBaseT<NlaStrip>`.
    pub strips: ListBase,
    pub flag: NlaTrackFlag,
    pub index: i32,
    /// `char name[64]`.
    pub name: [u8; 64],
}
impl Default for NlaTrack {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            strips: ListBase::new(),
            flag: 0,
            index: 0,
            name: [0; 64],
        }
    }
}

/// `KS_Path`.
#[derive(Clone)]
pub struct KsPath {
    pub next: usize,
    pub prev: usize,
    /// `ID *id`.
    pub id: usize,
    /// `char group[64]`.
    pub group: [u8; 64],
    pub idtype: i32,
    pub groupmode: KspGrouping,
    pub flag: KspSettings,
    /// `char *rna_path`.
    pub rna_path: Option<String>,
    pub array_index: i32,
    pub keyingflag: InsertKeyFlags,
    pub keyingoverride: InsertKeyFlags,
}
impl Default for KsPath {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            id: NIL,
            group: [0; 64],
            idtype: 0,
            groupmode: 0,
            flag: 0,
            rna_path: None,
            array_index: 0,
            keyingflag: 0,
            keyingoverride: 0,
        }
    }
}

/// `KeyingSet`.
#[derive(Clone)]
pub struct KeyingSet {
    pub next: usize,
    pub prev: usize,
    /// `ListBaseT<KS_Path>`.
    pub paths: ListBase,
    /// `char idname[64]`, `name[64]`, `description[1024]`, `typeinfo[64]`.
    pub idname: [u8; 64],
    pub name: [u8; 64],
    pub description: [u8; 1024],
    pub typeinfo: [u8; 64],
    pub active_path: i32,
    pub flag: KsSettings,
    pub keyingflag: InsertKeyFlags,
    pub keyingoverride: InsertKeyFlags,
}
impl Default for KeyingSet {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            paths: ListBase::new(),
            idname: [0; 64],
            name: [0; 64],
            description: [0; 1024],
            typeinfo: [0; 64],
            active_path: 0,
            flag: 0,
            keyingflag: 0,
            keyingoverride: 0,
        }
    }
}

/// `AnimOverride`.
#[derive(Clone)]
pub struct AnimOverride {
    pub next: usize,
    pub prev: usize,
    /// `char *rna_path`.
    pub rna_path: Option<String>,
    pub array_index: i32,
    pub value: f32,
}
impl Default for AnimOverride {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            rna_path: None,
            array_index: 0,
            value: 0.0,
        }
    }
}

/// `AnimData`.
#[derive(Clone)]
pub struct AnimData {
    /// `bAction *action`.
    pub action: usize,
    pub slot_handle: i32,
    /// `char last_slot_identifier[258]`.
    pub last_slot_identifier: [u8; 258],
    /// `bAction *tmpact`.
    pub tmpact: usize,
    pub tmp_slot_handle: i32,
    /// `char tmp_last_slot_identifier[258]`.
    pub tmp_last_slot_identifier: [u8; 258],
    /// `ListBaseT<NlaTrack>`.
    pub nla_tracks: ListBase,
    /// `NlaTrack *act_track`.
    pub act_track: usize,
    /// `NlaStrip *actstrip`.
    pub actstrip: usize,
    /// `ListBaseT<FCurve>` drivers.
    pub drivers: ListBase,
    /// `ListBaseT<AnimOverride>`.
    pub overrides: ListBase,
    /// `FCurve **driver_array` → Vec of refs.
    pub driver_array: Vec<usize>,
    pub flag: AnimDataFlag,
    pub act_blendmode: NlaStripBlendMode,
    pub act_extendmode: NlaStripExtrapolateMode,
    pub act_influence: f32,
}
impl Default for AnimData {
    fn default() -> Self {
        Self {
            action: NIL,
            slot_handle: 0,
            last_slot_identifier: [0; 258],
            tmpact: NIL,
            tmp_slot_handle: 0,
            tmp_last_slot_identifier: [0; 258],
            nla_tracks: ListBase::new(),
            act_track: NIL,
            actstrip: NIL,
            drivers: ListBase::new(),
            overrides: ListBase::new(),
            driver_array: Vec::new(),
            flag: 0,
            act_blendmode: 0,
            act_extendmode: 0,
            act_influence: 0.0,
        }
    }
}

/// `IdAdtTemplate` — ID + adt layout prefix.
#[derive(Clone)]
pub struct IdAdtTemplate {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
}
impl Default for IdAdtTemplate {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
        }
    }
}
