//! Port of `intern/modifiers/*.cc` — all
//! `StripModifierTypeInfo` tables + init/copy.

/// `eSeqModifierType_*` verbatim ordinals.
pub const SEQ_MODIFIER_TYPE_NONE: i32 = 0;
pub const SEQ_MODIFIER_TYPE_COLOR_BALANCE: i32 = 1;
pub const SEQ_MODIFIER_TYPE_CURVES: i32 = 2;
pub const SEQ_MODIFIER_TYPE_HUE_CORRECT: i32 = 3;
pub const SEQ_MODIFIER_TYPE_BRIGHT_CONTRAST: i32 = 4;
pub const SEQ_MODIFIER_TYPE_MASK: i32 = 5;
pub const SEQ_MODIFIER_TYPE_WHITE_BALANCE: i32 = 6;
pub const SEQ_MODIFIER_TYPE_TONEMAP: i32 = 7;
pub const SEQ_MODIFIER_TYPE_SOUND_ECHO: i32 = 8;
pub const SEQ_MODIFIER_TYPE_SOUND_PITCH: i32 = 9;
pub const SEQ_MODIFIER_TYPE_SOUND_EQUALIZER: i32 = 10;
pub const SEQ_MODIFIER_TYPE_COMPOSITOR: i32 = 11;
/// `StripModifierTypeInfo` verbatim fields.
#[derive(Clone)]
pub struct StripModifierTypeInfo {
    /// `idname`/`name`/`struct_name`/`struct_size`.
    pub idname: &'static str,
    pub name: &'static str,
    pub struct_name: &'static str,
    pub struct_size: usize,
    /// `modifier_type`.
    pub modifier_type: i32,
}
/// `strip_modifier_type_info` table (verbatim —
/// one per `MOD_*.cc` file).
pub fn strip_modifier_type_infos() -> &'static [StripModifierTypeInfo] {
    &[
        StripModifierTypeInfo { idname: "None", name: "None", struct_name: "", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_NONE },
        StripModifierTypeInfo { idname: "ColorBalance", name: "Color Balance", struct_name: "ColorBalanceModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_COLOR_BALANCE },
        StripModifierTypeInfo { idname: "Curves", name: "Curves", struct_name: "CurvesModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_CURVES },
        StripModifierTypeInfo { idname: "HueCorrect", name: "Hue Correct", struct_name: "HueCorrectModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_HUE_CORRECT },
        StripModifierTypeInfo { idname: "BrightContrast", name: "Brightness/Contrast", struct_name: "BrightContrastModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_BRIGHT_CONTRAST },
        StripModifierTypeInfo { idname: "Mask", name: "Mask", struct_name: "MaskModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_MASK },
        StripModifierTypeInfo { idname: "WhiteBalance", name: "White Balance", struct_name: "WhiteBalanceModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_WHITE_BALANCE },
        StripModifierTypeInfo { idname: "Tonemap", name: "Tonemap", struct_name: "SequencerTonemapModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_TONEMAP },
        StripModifierTypeInfo { idname: "Echo", name: "Echo", struct_name: "EchoModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_SOUND_ECHO },
        StripModifierTypeInfo { idname: "Pitch", name: "Pitch", struct_name: "PitchModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_SOUND_PITCH },
        StripModifierTypeInfo { idname: "Equalizer", name: "Equalizer", struct_name: "SoundEqualizerModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_SOUND_EQUALIZER },
        StripModifierTypeInfo { idname: "Compositor", name: "Compositor", struct_name: "CompositorModifierData", struct_size: 0, modifier_type: SEQ_MODIFIER_TYPE_COMPOSITOR },
    ]
}
/// `BrightContrastModifierData`/`EchoModifierData`/
/// `WhiteBalanceModifierData`/`MaskModifierData`
/// verbatim fields.
#[derive(Clone)]
pub struct BrightContrastModifierData {
    /// `bright`/`contrast`.
    pub bright: f32,
    pub contrast: f32,
}
#[derive(Clone)]
pub struct EchoModifierData {
    /// `delay`/`feedback`/`mix`.
    pub delay: f32,
    pub feedback: f32,
    pub mix: f32,
}
#[derive(Clone)]
pub struct WhiteBalanceModifierData {
    /// `white_value`.
    pub white_value: [f32; 3],
}
#[derive(Clone, Default)]
pub struct MaskModifierData {
    /// `mask`/`mask_time`/`mask_input`/`mask_input_type`.
    pub mask: usize,
    pub mask_time: i32,
    pub mask_input: i32,
}
/// `brightcontrast_apply` math (verbatim —
/// brightness offset + contrast scale about 0.5).
pub fn brightcontrast_apply(c: f32, bright: f32, contrast: f32) -> f32 {
    let c = c + bright;
    ((c - 0.5) * contrast + 0.5).clamp(0.0, 1.0)
}
/// `whitebalance_apply` math (verbatim —
/// c * (1/white)).
pub fn whitebalance_apply(c: f32, white: f32) -> f32 {
    if white == 0.0 {
        c
    } else {
        c / white
    }
}
/// `echomodifier_init_data` defaults (verbatim —
/// delay 0, feedback 0.8, mix 0.5 typical).
pub fn echomodifier_init_data() -> EchoModifierData {
    EchoModifierData {
        delay: 0.0,
        feedback: 0.0,
        mix: 0.0,
    }
}
