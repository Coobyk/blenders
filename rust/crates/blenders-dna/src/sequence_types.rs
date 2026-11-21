//! Port of `DNA_sequence_types.h` — sequencer strips/effects.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::id::{ID, NIL};
use crate::color_types::{ColorManagedColorspaceSettings, CurveMapping};
use crate::scene_enums::ImageformatViewsformat;
use crate::screen_types::TextboxState;
use crate::small_types::ColorBand;
use crate::vec_types::Rctf;
use blenders_bli::listbase::ListBase;
/// `eStripFlag`.
pub type Stripflag = u32;
pub const SEQ_FLAG_NONE: Stripflag = 0;
pub const SEQ_SELECT: Stripflag = 1 << 0;
pub const SEQ_LEFTSEL: Stripflag = 1 << 1;
pub const SEQ_RIGHTSEL: Stripflag = 1 << 2;
pub const SEQ_DEINTERLACE: Stripflag = 1 << 4;
pub const SEQ_MUTE: Stripflag = 1 << 5;
pub const SEQ_FLAG_TEXT_EDITING_ACTIVE: Stripflag = 1 << 6;
pub const SEQ_REVERSE_FRAMES: Stripflag = 1 << 7;
pub const SEQ_FLIPX: Stripflag = 1 << 11;
pub const SEQ_FLIPY: Stripflag = 1 << 12;
pub const SEQ_MAKE_FLOAT: Stripflag = 1 << 13;
pub const SEQ_LOCK: Stripflag = 1 << 14;
pub const SEQ_USE_PROXY: Stripflag = 1 << 15;
pub const SEQ_AUTO_PLAYBACK_RATE: Stripflag = 1 << 17;
pub const SEQ_SINGLE_FRAME_CONTENT: Stripflag = 1 << 18;
pub const SEQ_SHOW_RETIMING: Stripflag = 1 << 19;
pub const SEQ_MULTIPLY_ALPHA: Stripflag = 1 << 21;
pub const SEQ_USE_EFFECT_DEFAULT_FADE: Stripflag = 1 << 22;
pub const SEQ_AUDIO_VOLUME_ANIMATED: Stripflag = 1 << 24;
pub const SEQ_AUDIO_PITCH_ANIMATED: Stripflag = 1 << 25;
pub const SEQ_AUDIO_PAN_ANIMATED: Stripflag = 1 << 26;
pub const SEQ_AUDIO_DRAW_WAVEFORM: Stripflag = 1 << 27;
pub const SEQ_SCENE_NO_ANNOTATION: Stripflag = 1 << 28;
pub const SEQ_USE_VIEWS: Stripflag = 1 << 29;
pub const SEQ_SCENE_STRIPS: Stripflag = 1 << 30;
pub const SEQ_AUDIO_PITCH_CORRECTION: Stripflag = 1 << 31;
pub const STRIP_ALLSEL: Stripflag = SEQ_SELECT | SEQ_LEFTSEL | SEQ_RIGHTSEL;

/// `eStripProxyStorageFlag`.
pub type Stripproxystorageflag = u8;
pub const SEQ_STORAGE_PROXY_NONE: Stripproxystorageflag = 0;
pub const SEQ_STORAGE_PROXY_CUSTOM_FILE: Stripproxystorageflag = 1 << 1;
pub const SEQ_STORAGE_PROXY_CUSTOM_DIR: Stripproxystorageflag = 1 << 2;

/// `eStripProxyBuildSize`.
pub type Stripproxybuildsize = u16;
pub const SEQ_PROXY_IMAGE_SIZE_NONE: Stripproxybuildsize = 0;
pub const SEQ_PROXY_IMAGE_SIZE_25: Stripproxybuildsize = 1 << 0;
pub const SEQ_PROXY_IMAGE_SIZE_50: Stripproxybuildsize = 1 << 1;
pub const SEQ_PROXY_IMAGE_SIZE_75: Stripproxybuildsize = 1 << 2;
pub const SEQ_PROXY_IMAGE_SIZE_100: Stripproxybuildsize = 1 << 3;

/// `eStripProxyBuildFlag`.
pub type Stripproxybuildflag = u16;
pub const SEQ_PROXY_BUILD_FLAG_NONE: Stripproxybuildflag = 0;
pub const SEQ_PROXY_SKIP_EXISTING: Stripproxybuildflag = 1;

/// `eStripAlphaMode`.
pub type Stripalphamode = u8;
pub const SEQ_ALPHA_STRAIGHT: Stripalphamode = 0;
pub const SEQ_ALPHA_PREMUL: Stripalphamode = 1;

/// `StripType`.
pub type Striptype = u32;
pub const STRIP_TYPE_IMAGE: Striptype = 0;
pub const STRIP_TYPE_META: Striptype = 1;
pub const STRIP_TYPE_SCENE: Striptype = 2;
pub const STRIP_TYPE_MOVIE: Striptype = 3;
pub const STRIP_TYPE_SOUND: Striptype = 4;
pub const STRIP_TYPE_SOUND_HD: Striptype = 5;
pub const STRIP_TYPE_MOVIECLIP: Striptype = 6;
pub const STRIP_TYPE_MASK: Striptype = 7;
pub const STRIP_TYPE_CROSS: Striptype = 8;
pub const STRIP_TYPE_ADD: Striptype = 9;
pub const STRIP_TYPE_SUB: Striptype = 10;
pub const STRIP_TYPE_ALPHAOVER: Striptype = 11;
pub const STRIP_TYPE_ALPHAUNDER: Striptype = 12;
pub const STRIP_TYPE_GAMCROSS: Striptype = 13;
pub const STRIP_TYPE_MUL: Striptype = 14;
pub const STRIP_TYPE_OVERDROP_REMOVED: Striptype = 15;
pub const STRIP_TYPE_COMPOSITOR: Striptype = 16;
pub const STRIP_TYPE_WIPE: Striptype = 25;
pub const STRIP_TYPE_GLOW: Striptype = 26;
pub const STRIP_TYPE_TRANSFORM_LEGACY: Striptype = 27;
pub const STRIP_TYPE_COLOR: Striptype = 28;
pub const STRIP_TYPE_SPEED: Striptype = 29;
pub const STRIP_TYPE_MULTICAM: Striptype = 30;
pub const STRIP_TYPE_ADJUSTMENT: Striptype = 31;
pub const STRIP_TYPE_GAUSSIAN_BLUR: Striptype = 40;
pub const STRIP_TYPE_TEXT: Striptype = 41;
pub const STRIP_TYPE_COLORMIX: Striptype = 42;

/// `eStripMovieClipFlag`.
pub type Stripmovieclipflag = u32;
pub const SEQ_MOVIECLIP_NONE: Stripmovieclipflag = 0;
pub const SEQ_MOVIECLIP_RENDER_UNDISTORTED: Stripmovieclipflag = 1 << 0;
pub const SEQ_MOVIECLIP_RENDER_STABILIZED: Stripmovieclipflag = 1 << 1;

/// `StripBlendMode`.
pub type Stripblendmode = u32;
pub const STRIP_BLEND_REPLACE: Stripblendmode = 0;
pub const STRIP_BLEND_CROSS: Stripblendmode = 8;
pub const STRIP_BLEND_ADD: Stripblendmode = 9;
pub const STRIP_BLEND_SUB: Stripblendmode = 10;
pub const STRIP_BLEND_ALPHAOVER: Stripblendmode = 11;
pub const STRIP_BLEND_ALPHAUNDER: Stripblendmode = 12;
pub const STRIP_BLEND_GAMCROSS: Stripblendmode = 13;
pub const STRIP_BLEND_MUL: Stripblendmode = 14;
pub const STRIP_BLEND_OVERDROP_REMOVED: Stripblendmode = 15;
pub const STRIP_BLEND_SCREEN: Stripblendmode = 43;
pub const STRIP_BLEND_LIGHTEN: Stripblendmode = 44;
pub const STRIP_BLEND_DODGE: Stripblendmode = 45;
pub const STRIP_BLEND_DARKEN: Stripblendmode = 46;
pub const STRIP_BLEND_COLOR_BURN: Stripblendmode = 47;
pub const STRIP_BLEND_LINEAR_BURN: Stripblendmode = 48;
pub const STRIP_BLEND_OVERLAY: Stripblendmode = 49;
pub const STRIP_BLEND_HARD_LIGHT: Stripblendmode = 50;
pub const STRIP_BLEND_SOFT_LIGHT: Stripblendmode = 51;
pub const STRIP_BLEND_PIN_LIGHT: Stripblendmode = 52;
pub const STRIP_BLEND_LIN_LIGHT: Stripblendmode = 53;
pub const STRIP_BLEND_VIVID_LIGHT: Stripblendmode = 54;
pub const STRIP_BLEND_HUE: Stripblendmode = 55;
pub const STRIP_BLEND_SATURATION: Stripblendmode = 56;
pub const STRIP_BLEND_VALUE: Stripblendmode = 57;
pub const STRIP_BLEND_BLEND_COLOR: Stripblendmode = 58;
pub const STRIP_BLEND_DIFFERENCE: Stripblendmode = 59;
pub const STRIP_BLEND_EXCLUSION: Stripblendmode = 60;

/// `StripColorTag`.
pub type Stripcolortag = i8;
pub const STRIP_COLOR_NONE: Stripcolortag = -1;
pub const STRIP_COLOR_01: Stripcolortag = STRIP_COLOR_NONE + 1;
pub const STRIP_COLOR_02: Stripcolortag = STRIP_COLOR_01 + 1;
pub const STRIP_COLOR_03: Stripcolortag = STRIP_COLOR_02 + 1;
pub const STRIP_COLOR_04: Stripcolortag = STRIP_COLOR_03 + 1;
pub const STRIP_COLOR_05: Stripcolortag = STRIP_COLOR_04 + 1;
pub const STRIP_COLOR_06: Stripcolortag = STRIP_COLOR_05 + 1;
pub const STRIP_COLOR_07: Stripcolortag = STRIP_COLOR_06 + 1;
pub const STRIP_COLOR_08: Stripcolortag = STRIP_COLOR_07 + 1;
pub const STRIP_COLOR_09: Stripcolortag = STRIP_COLOR_08 + 1;
pub const STRIP_COLOR_TOT: Stripcolortag = STRIP_COLOR_09 + 1;

/// `eStripTransformFilter`.
pub type Striptransformfilter = i32;
pub const SEQ_TRANSFORM_FILTER_AUTO: Striptransformfilter = -1;
pub const SEQ_TRANSFORM_FILTER_NEAREST: Striptransformfilter = 0;
pub const SEQ_TRANSFORM_FILTER_BILINEAR: Striptransformfilter = 1;
pub const SEQ_TRANSFORM_FILTER_BOX: Striptransformfilter = 2;
pub const SEQ_TRANSFORM_FILTER_CUBIC_BSPLINE: Striptransformfilter = 3;
pub const SEQ_TRANSFORM_FILTER_CUBIC_MITCHELL: Striptransformfilter = 4;

/// `eSeqChannelFlag`.
pub type Seqchannelflag = u32;
pub const SEQ_CHANNEL_NONE: Seqchannelflag = 0;
pub const SEQ_CHANNEL_LOCK: Seqchannelflag = 1 << 0;
pub const SEQ_CHANNEL_MUTE: Seqchannelflag = 1 << 1;

/// `eModColorBalanceMethod`.
pub type Modcolorbalancemethod = u32;
pub const SEQ_COLOR_BALANCE_METHOD_LIFTGAMMAGAIN: Modcolorbalancemethod = 0;
pub const SEQ_COLOR_BALANCE_METHOD_SLOPEOFFSETPOWER: Modcolorbalancemethod = 1;

/// `eModColorBalanceInverseFlag`.
pub type Modcolorbalanceinverseflag = u32;
pub const SEQ_COLOR_BALANCE_INVERSE_NONE: Modcolorbalanceinverseflag = 0;
pub const SEQ_COLOR_BALANCE_INVERSE_GAIN: Modcolorbalanceinverseflag = 1 << 0;
pub const SEQ_COLOR_BALANCE_INVERSE_GAMMA: Modcolorbalanceinverseflag = 1 << 1;
pub const SEQ_COLOR_BALANCE_INVERSE_LIFT: Modcolorbalanceinverseflag = 1 << 2;
pub const SEQ_COLOR_BALANCE_INVERSE_SLOPE: Modcolorbalanceinverseflag = 1 << 3;
pub const SEQ_COLOR_BALANCE_INVERSE_OFFSET: Modcolorbalanceinverseflag = 1 << 4;
pub const SEQ_COLOR_BALANCE_INVERSE_POWER: Modcolorbalanceinverseflag = 1 << 5;

/// `eSeqRetimingKeyFlag`.
pub type Seqretimingkeyflag = u32;
pub const SEQ_RETIMING_FLAG_NONE: Seqretimingkeyflag = 0;
pub const SEQ_SPEED_TRANSITION_IN: Seqretimingkeyflag = 1 << 0;
pub const SEQ_SPEED_TRANSITION_OUT: Seqretimingkeyflag = 1 << 1;
pub const SEQ_FREEZE_FRAME_IN: Seqretimingkeyflag = 1 << 2;
pub const SEQ_FREEZE_FRAME_OUT: Seqretimingkeyflag = 1 << 3;
pub const SEQ_KEY_SELECTED: Seqretimingkeyflag = 1 << 4;

/// `eEditingOverlayFrameFlag`.
pub type Editingoverlayframeflag = u32;
pub const SEQ_EDIT_OVERLAY_FRAME_NONE: Editingoverlayframeflag = 0;
pub const SEQ_EDIT_OVERLAY_FRAME_SHOW: Editingoverlayframeflag = 1;
pub const SEQ_EDIT_OVERLAY_FRAME_ABS: Editingoverlayframeflag = 2;

/// `eEditingShowMissingMediaFlag`.
pub type Editingshowmissingmediaflag = u32;
pub const SEQ_EDIT_SHOW_NONE: Editingshowmissingmediaflag = 0;
pub const SEQ_EDIT_SHOW_MISSING_MEDIA: Editingshowmissingmediaflag = 1 << 0;

/// `eEditingProxyStorageMode`.
pub type Editingproxystoragemode = u32;
pub const SEQ_EDIT_PROXY_NONE: Editingproxystoragemode = 0;
pub const SEQ_EDIT_PROXY_DIR_STORAGE: Editingproxystoragemode = 1;

/// `eEditingCacheFlag`.
pub type Editingcacheflag = u32;
pub const SEQ_CACHE_NONE: Editingcacheflag = 0;
pub const SEQ_CACHE_STORE_RAW: Editingcacheflag = 1 << 0;
pub const SEQ_CACHE_UNUSED_1: Editingcacheflag = 1 << 1;
pub const SEQ_CACHE_UNUSED_2: Editingcacheflag = 1 << 2;
pub const SEQ_CACHE_STORE_FINAL_OUT: Editingcacheflag = 1 << 3;
pub const SEQ_CACHE_ALL_TYPES: Editingcacheflag = SEQ_CACHE_STORE_RAW | SEQ_CACHE_STORE_FINAL_OUT;
pub const SEQ_CACHE_UNUSED_4: Editingcacheflag = 1 << 4;
pub const SEQ_CACHE_UNUSED_5: Editingcacheflag = 1 << 5;
pub const SEQ_CACHE_UNUSED_6: Editingcacheflag = 1 << 6;
pub const SEQ_CACHE_UNUSED_7: Editingcacheflag = 1 << 7;
pub const SEQ_CACHE_UNUSED_8: Editingcacheflag = 1 << 8;
pub const SEQ_CACHE_UNUSED_9: Editingcacheflag = 1 << 9;
pub const SEQ_CACHE_PREFETCH_ENABLE: Editingcacheflag = 1 << 10;
pub const SEQ_CACHE_UNUSED_11: Editingcacheflag = 1 << 11;

/// `eEffectWipeType`.
pub type Effectwipetype = u16;
pub const SEQ_WIPE_SINGLE: Effectwipetype = 0;
pub const SEQ_WIPE_DOUBLE: Effectwipetype = SEQ_WIPE_SINGLE + 1;
pub const SEQ_WIPE_IRIS: Effectwipetype = SEQ_WIPE_DOUBLE + 1;
pub const SEQ_WIPE_CLOCK: Effectwipetype = SEQ_WIPE_IRIS + 1;

/// `eEffectSpeedControlFlags`.
pub type Effectspeedcontrolflags = u32;
pub const SEQ_SPEED_NONE: Effectspeedcontrolflags = 0;
pub const SEQ_SPEED_UNUSED_2: Effectspeedcontrolflags = 1 << 0;
pub const SEQ_SPEED_UNUSED_1: Effectspeedcontrolflags = 1 << 1;
pub const SEQ_SPEED_UNUSED_3: Effectspeedcontrolflags = 1 << 2;
pub const SEQ_SPEED_USE_INTERPOLATION: Effectspeedcontrolflags = 1 << 3;

/// `eEffectSpeedControlType`.
pub type Effectspeedcontroltype = u32;
pub const SEQ_SPEED_STRETCH: Effectspeedcontroltype = 0;
pub const SEQ_SPEED_MULTIPLY: Effectspeedcontroltype = 1;
pub const SEQ_SPEED_LENGTH: Effectspeedcontroltype = 2;
pub const SEQ_SPEED_FRAME_NUMBER: Effectspeedcontroltype = 3;

/// `eEffectTextFlags`.
pub type Effecttextflags = u8;
pub const SEQ_TEXT_NONE: Effecttextflags = 0;
pub const SEQ_TEXT_SHADOW: Effecttextflags = 1 << 0;
pub const SEQ_TEXT_BOX: Effecttextflags = 1 << 1;
pub const SEQ_TEXT_BOLD: Effecttextflags = 1 << 2;
pub const SEQ_TEXT_ITALIC: Effecttextflags = 1 << 3;
pub const SEQ_TEXT_OUTLINE: Effecttextflags = 1 << 4;
pub const SEQ_TEXT_USE_ABSOLUTE_LINE_SPACING: Effecttextflags = 1 << 5;

/// `eEffectTextAlignX`.
pub type Effecttextalignx = u8;
pub const SEQ_TEXT_ALIGN_X_LEFT: Effecttextalignx = 0;
pub const SEQ_TEXT_ALIGN_X_CENTER: Effecttextalignx = 1;
pub const SEQ_TEXT_ALIGN_X_RIGHT: Effecttextalignx = 2;

/// `eEffectTextAnchorX`.
pub type Effecttextanchorx = u8;
pub const SEQ_TEXT_ANCHOR_X_LEFT: Effecttextanchorx = 0;
pub const SEQ_TEXT_ANCHOR_X_CENTER: Effecttextanchorx = 1;
pub const SEQ_TEXT_ANCHOR_X_RIGHT: Effecttextanchorx = 2;

/// `eEffectTextAnchorY`.
pub type Effecttextanchory = u8;
pub const SEQ_TEXT_ANCHOR_Y_TOP: Effecttextanchory = 0;
pub const SEQ_TEXT_ANCHOR_Y_CENTER: Effecttextanchory = 1;
pub const SEQ_TEXT_ANCHOR_Y_BOTTOM: Effecttextanchory = 2;

/// `eModTonemapType`.
pub type Modtonemaptype = u32;
pub const SEQ_TONEMAP_RH_SIMPLE: Modtonemaptype = 0;
pub const SEQ_TONEMAP_RD_PHOTORECEPTOR: Modtonemaptype = 1;

/// `ePitchMode`.
pub type Pitchmode = u32;
pub const PITCH_MODE_SEMITONES: Pitchmode = 0;
pub const PITCH_MODE_RATIO: Pitchmode = 1;

/// `ePitchQuality`.
pub type Pitchquality = u32;
pub const PITCH_QUALITY_HIGH: Pitchquality = 0;
pub const PITCH_QUALITY_FAST: Pitchquality = 1;
pub const PITCH_QUALITY_CONSISTENT: Pitchquality = 2;

/// `eStripModifierType`.
pub type Stripmodifiertype = u32;
pub const eSeqModifierType_None: Stripmodifiertype = 0;
pub const eSeqModifierType_ColorBalance: Stripmodifiertype = 1;
pub const eSeqModifierType_Curves: Stripmodifiertype = 2;
pub const eSeqModifierType_HueCorrect: Stripmodifiertype = 3;
pub const eSeqModifierType_BrightContrast: Stripmodifiertype = 4;
pub const eSeqModifierType_Mask: Stripmodifiertype = 5;
pub const eSeqModifierType_WhiteBalance: Stripmodifiertype = 6;
pub const eSeqModifierType_Tonemap: Stripmodifiertype = 7;
pub const eSeqModifierType_SoundEqualizer: Stripmodifiertype = 8;
pub const eSeqModifierType_Compositor: Stripmodifiertype = 9;
pub const eSeqModifierType_Pitch: Stripmodifiertype = 10;
pub const eSeqModifierType_Echo: Stripmodifiertype = 11;
pub const NUM_STRIP_MODIFIER_TYPES: Stripmodifiertype = eSeqModifierType_Echo + 1;

/// `eStripModifierFlag`.
pub type Stripmodifierflag = u32;
pub const STRIP_MODIFIER_FLAG_NONE: Stripmodifierflag = 0;
pub const STRIP_MODIFIER_FLAG_MUTE: Stripmodifierflag = 1 << 0;
pub const STRIP_MODIFIER_FLAG_EXPANDED: Stripmodifierflag = 1 << 1;
pub const STRIP_MODIFIER_FLAG_ACTIVE: Stripmodifierflag = 1 << 2;
pub const STRIP_MODIFIER_FLAG_SHOW_PREVIEW: Stripmodifierflag = 1 << 3;

/// `eModMaskInput`.
pub type Modmaskinput = u32;
pub const STRIP_MASK_INPUT_STRIP: Modmaskinput = 0;
pub const STRIP_MASK_INPUT_ID: Modmaskinput = 1;

/// `eModMaskTime`.
pub type Modmasktime = u32;
pub const STRIP_MASK_TIME_RELATIVE: Modmasktime = 0;
pub const STRIP_MASK_TIME_ABSOLUTE: Modmasktime = 1;

/// `SequencerCompositorModifierFlag`.
pub type Sequencercompositormodifierflag = u8;
pub const SEQ_COMP_MOD_NONE: Sequencercompositormodifierflag = 0;
pub const SEQ_COMP_MOD_HIDE_DATABLOCK_SELECTOR: Sequencercompositormodifierflag = 1 << 0;

/// `StripElem`.
#[derive(Clone)]
pub struct StripElem {
    /// `char filename[256]`
    pub filename: [u8; 256],
    /// `int orig_width`
    pub orig_width: i32,
    /// `int orig_height`
    pub orig_height: i32,
    /// `float orig_fps`
    pub orig_fps: f32,
}
impl Default for StripElem {
    fn default() -> Self {
        Self {
            filename: [0; 256],
            orig_width: 0,
            orig_height: 0,
            orig_fps: 0.0,
        }
    }
}

/// `StripCrop`.
#[derive(Clone)]
pub struct StripCrop {
    /// `int top`
    pub top: i32,
    /// `int bottom`
    pub bottom: i32,
    /// `int left`
    pub left: i32,
    /// `int right`
    pub right: i32,
}
impl Default for StripCrop {
    fn default() -> Self {
        Self {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0,
        }
    }
}

/// `StripTransform`.
#[derive(Clone)]
pub struct StripTransform {
    /// `float xofs`
    pub xofs: f32,
    /// `float yofs`
    pub yofs: f32,
    /// `float scale_x`
    pub scale_x: f32,
    /// `float scale_y`
    pub scale_y: f32,
    /// `float rotation`
    pub rotation: f32,
    /// `float origin[2]`
    pub origin: [f32; 2],
    /// `eStripTransformFilter filter`
    pub filter: Striptransformfilter,
}
impl Default for StripTransform {
    fn default() -> Self {
        Self {
            xofs: 0.0,
            yofs: 0.0,
            scale_x: 0.0,
            scale_y: 0.0,
            rotation: 0.0,
            origin: [0.0; 2],
            filter: SEQ_TRANSFORM_FILTER_AUTO,
        }
    }
}

/// `StripColorBalance`.
#[derive(Clone)]
pub struct StripColorBalance {
    /// `eModColorBalanceMethod method`
    pub method: Modcolorbalancemethod,
    /// `float lift[3]`
    pub lift: [f32; 3],
    /// `float gamma[3]`
    pub gamma: [f32; 3],
    /// `float gain[3]`
    pub gain: [f32; 3],
    /// `float slope[3]`
    pub slope: [f32; 3],
    /// `float offset[3]`
    pub offset: [f32; 3],
    /// `float power[3]`
    pub power: [f32; 3],
    /// `eModColorBalanceInverseFlag flag`
    pub flag: Modcolorbalanceinverseflag,
}
impl Default for StripColorBalance {
    fn default() -> Self {
        Self {
            method: SEQ_COLOR_BALANCE_METHOD_LIFTGAMMAGAIN,
            lift: [0.0; 3],
            gamma: [0.0; 3],
            gain: [0.0; 3],
            slope: [0.0; 3],
            offset: [0.0; 3],
            power: [0.0; 3],
            flag: SEQ_COLOR_BALANCE_INVERSE_NONE,
        }
    }
}

/// `StripProxy`.
#[derive(Clone)]
pub struct StripProxy {
    /// `char dirpath[768]`
    pub dirpath: [u8; 768],
    /// `char filename[256]`
    pub filename: [u8; 256],
    /// `MovieReader *anim`
    pub anim: usize,
    /// `short quality`
    pub quality: i16,
    /// `eStripProxyBuildFlag build_flags`
    pub build_flags: Stripproxybuildflag,
    /// `eStripProxyStorageFlag storage`
    pub storage: Stripproxystorageflag,
}
impl Default for StripProxy {
    fn default() -> Self {
        Self {
            dirpath: [0; 768],
            filename: [0; 256],
            anim: NIL,
            quality: 0,
            build_flags: SEQ_PROXY_BUILD_FLAG_NONE,
            storage: SEQ_STORAGE_PROXY_NONE,
        }
    }
}

/// `StripData`.
#[derive(Clone)]
pub struct StripData {
    /// `StripData *next`
    pub next: usize,
    /// `StripData *prev`
    pub prev: usize,
    /// `StripElem *stripdata`
    pub stripdata: usize,
    /// `int stripdata_num`
    pub stripdata_num: i32,
    /// `char dirpath[768]`
    pub dirpath: [u8; 768],
    /// `StripProxy *proxy`
    pub proxy: usize,
    /// `StripCrop *crop`
    pub crop: usize,
    /// `StripTransform *transform`
    pub transform: usize,
    /// `StripColorBalance *color_balance_legacy`
    pub color_balance_legacy: usize,
    pub colorspace_settings: ColorManagedColorspaceSettings,
}
impl Default for StripData {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            stripdata: NIL,
            stripdata_num: 0,
            dirpath: [0; 768],
            proxy: NIL,
            crop: NIL,
            transform: NIL,
            color_balance_legacy: NIL,
            colorspace_settings: Default::default(),
        }
    }
}

/// `SeqRetimingKey`.
#[derive(Clone)]
pub struct SeqRetimingKey {
    /// `double strip_frame_index`
    pub strip_frame_index: f64,
    /// `eSeqRetimingKeyFlag flag`
    pub flag: Seqretimingkeyflag,
    /// `float retiming_factor`
    pub retiming_factor: f32,
    /// `double original_strip_frame_index`
    pub original_strip_frame_index: f64,
    /// `float original_retiming_factor`
    pub original_retiming_factor: f32,
}
impl Default for SeqRetimingKey {
    fn default() -> Self {
        Self {
            strip_frame_index: 0.0,
            flag: SEQ_RETIMING_FLAG_NONE,
            retiming_factor: 0.0,
            original_strip_frame_index: 0.0,
            original_retiming_factor: 0.0,
        }
    }
}

/// `Strip`.
#[derive(Clone)]
pub struct Strip {
    /// `Strip *next`
    pub next: usize,
    /// `Strip *prev`
    pub prev: usize,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `eStripFlag flag`
    pub flag: Stripflag,
    /// `StripType type`
    pub type_: Striptype,
    /// `int len`
    pub len: i32,
    /// `float start`
    pub start: f32,
    /// `float startofs`
    pub startofs: f32,
    /// `float endofs`
    pub endofs: f32,
    /// `float startstill_legacy`
    pub startstill_legacy: f32,
    /// `float endstill_legacy`
    pub endstill_legacy: f32,
    /// `int channel`
    pub channel: i32,
    /// `int startdisp`
    pub startdisp: i32,
    /// `int enddisp`
    pub enddisp: i32,
    /// `float sat`
    pub sat: f32,
    /// `float mul`
    pub mul: f32,
    /// `short streamindex`
    pub streamindex: i16,
    /// `int multicam_source`
    pub multicam_source: i32,
    /// `eStripMovieClipFlag clip_flag`
    pub clip_flag: Stripmovieclipflag,
    /// `StripData *data`
    pub data: usize,
    /// `Scene *scene`
    pub scene: usize,
    /// `Object *scene_camera`
    pub scene_camera: usize,
    /// `char *scene_view_layer_name`
    pub scene_view_layer_name: usize,
    /// `MovieClip *clip`
    pub clip: usize,
    /// `Mask *mask`
    pub mask: usize,
    /// `float effect_fader`
    pub effect_fader: f32,
    /// `float speed_fader_legacy`
    pub speed_fader_legacy: f32,
    /// `Strip *input1`
    pub input1: usize,
    /// `Strip *input2`
    pub input2: usize,
    /// `ListBaseT<Strip>`
    pub seqbase: ListBase,
    /// `bSound *sound`
    pub sound: usize,
    /// `float volume`
    pub volume: f32,
    /// `float pitch_legacy`
    pub pitch_legacy: f32,
    /// `float pan`
    pub pan: f32,
    /// `float strobe`
    pub strobe: f32,
    /// `float sound_offset`
    pub sound_offset: f32,
    /// `void *effectdata`
    pub effectdata: usize,
    /// `int anim_startofs`
    pub anim_startofs: i32,
    /// `int anim_endofs`
    pub anim_endofs: i32,
    /// `StripBlendMode blend_mode`
    pub blend_mode: Stripblendmode,
    /// `float blend_opacity`
    pub blend_opacity: f32,
    /// `StripColorTag color_tag`
    pub color_tag: Stripcolortag,
    /// `eStripAlphaMode alpha_mode`
    pub alpha_mode: Stripalphamode,
    /// `int sfra`
    pub sfra: i32,
    /// `eImageFormat_ViewsFormat views_format`
    pub views_format: ImageformatViewsformat,
    /// `Stereo3dFormat *stereo3d_format`
    pub stereo3d_format: usize,
    /// `IDProperty *prop`
    pub prop: usize,
    /// `IDProperty *system_properties`
    pub system_properties: usize,
    /// `float media_playback_rate`
    pub media_playback_rate: f32,
    /// `float speed_factor`
    pub speed_factor: f32,
    /// `SeqRetimingKey *retiming_keys`
    pub retiming_keys: usize,
    /// `int retiming_keys_num`
    pub retiming_keys_num: i32,
    /// `seq::StripRuntime *runtime`
    pub runtime: usize,
}
impl Default for Strip {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            flag: SEQ_FLAG_NONE,
            type_: STRIP_TYPE_IMAGE,
            len: 0,
            start: 0.0,
            startofs: 0.0,
            endofs: 0.0,
            startstill_legacy: 0.0,
            endstill_legacy: 0.0,
            channel: 0,
            startdisp: 0,
            enddisp: 0,
            sat: 0.0,
            mul: 0.0,
            streamindex: 0,
            multicam_source: 0,
            clip_flag: SEQ_MOVIECLIP_NONE,
            data: NIL,
            scene: NIL,
            scene_camera: NIL,
            scene_view_layer_name: NIL,
            clip: NIL,
            mask: NIL,
            effect_fader: 0.0,
            speed_fader_legacy: 0.0,
            input1: NIL,
            input2: NIL,
            seqbase: ListBase::default(),
            sound: NIL,
            volume: 0.0,
            pitch_legacy: 0.0,
            pan: 0.0,
            strobe: 0.0,
            sound_offset: 0.0,
            effectdata: NIL,
            anim_startofs: 0,
            anim_endofs: 0,
            blend_mode: STRIP_BLEND_REPLACE,
            blend_opacity: 0.0,
            color_tag: STRIP_COLOR_NONE,
            alpha_mode: SEQ_ALPHA_STRAIGHT,
            sfra: 0,
            views_format: Default::default(),
            stereo3d_format: NIL,
            prop: NIL,
            system_properties: NIL,
            media_playback_rate: 0.0,
            speed_factor: 0.0,
            retiming_keys: NIL,
            retiming_keys_num: 0,
            runtime: NIL,
        }
    }
}

/// `MetaStack`.
#[derive(Clone)]
pub struct MetaStack {
    /// `MetaStack *next`
    pub next: usize,
    /// `MetaStack *prev`
    pub prev: usize,
    /// `Strip *old_strip`
    pub old_strip: usize,
    /// `Strip *parent_strip`
    pub parent_strip: usize,
    /// `int disp_range[2]`
    pub disp_range: [i32; 2],
}
impl Default for MetaStack {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            old_strip: NIL,
            parent_strip: NIL,
            disp_range: [0; 2],
        }
    }
}

/// `SeqTimelineChannel`.
#[derive(Clone)]
pub struct SeqTimelineChannel {
    /// `SeqTimelineChannel *next`
    pub next: usize,
    /// `SeqTimelineChannel *prev`
    pub prev: usize,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `int index`
    pub index: i32,
    /// `eSeqChannelFlag flag`
    pub flag: Seqchannelflag,
}
impl Default for SeqTimelineChannel {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            index: 0,
            flag: SEQ_CHANNEL_NONE,
        }
    }
}

/// `StripConnection`.
#[derive(Clone)]
pub struct StripConnection {
    /// `StripConnection *next`
    pub next: usize,
    /// `StripConnection *prev`
    pub prev: usize,
    /// `Strip *strip_ref`
    pub strip_ref: usize,
}
impl Default for StripConnection {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            strip_ref: NIL,
        }
    }
}

/// `Editing`.
#[derive(Clone)]
pub struct Editing {
    /// `Strip *current_meta_strip`
    pub current_meta_strip: usize,
    /// `ListBaseT<Strip>`
    pub seqbase: ListBase,
    /// `ListBaseT<MetaStack>`
    pub metastack: ListBase,
    /// `ListBaseT<SeqTimelineChannel>`
    pub channels: ListBase,
    /// `Strip *act_strip`
    pub act_strip: usize,
    /// `char proxy_dir[1024]`
    pub proxy_dir: [u8; 1024],
    /// `eEditingProxyStorageMode proxy_storage`
    pub proxy_storage: Editingproxystoragemode,
    /// `int overlay_frame_ofs`
    pub overlay_frame_ofs: i32,
    /// `int overlay_frame_abs`
    pub overlay_frame_abs: i32,
    /// `eEditingOverlayFrameFlag overlay_frame_flag`
    pub overlay_frame_flag: Editingoverlayframeflag,
    /// `rctf overlay_frame_rect`
    pub overlay_frame_rect: Rctf,
    /// `eEditingShowMissingMediaFlag show_missing_media_flag`
    pub show_missing_media_flag: Editingshowmissingmediaflag,
    /// `eEditingCacheFlag cache_flag`
    pub cache_flag: Editingcacheflag,
    /// `seq::EditingRuntime *runtime`
    pub runtime: usize,
}
impl Default for Editing {
    fn default() -> Self {
        Self {
            current_meta_strip: NIL,
            seqbase: ListBase::default(),
            metastack: ListBase::default(),
            channels: ListBase::default(),
            act_strip: NIL,
            proxy_dir: [0; 1024],
            proxy_storage: SEQ_EDIT_PROXY_NONE,
            overlay_frame_ofs: 0,
            overlay_frame_abs: 0,
            overlay_frame_flag: SEQ_EDIT_OVERLAY_FRAME_NONE,
            overlay_frame_rect: Default::default(),
            show_missing_media_flag: SEQ_EDIT_SHOW_NONE,
            cache_flag: SEQ_CACHE_NONE,
            runtime: NIL,
        }
    }
}

/// `WipeVars`.
#[derive(Clone)]
pub struct WipeVars {
    /// `float edgeWidth`
    pub edge_width: f32,
    /// `float angle`
    pub angle: f32,
    /// `short forward`
    pub forward: i16,
    /// `eEffectWipeType wipetype`
    pub wipetype: Effectwipetype,
}
impl Default for WipeVars {
    fn default() -> Self {
        Self {
            edge_width: 0.0,
            angle: 0.0,
            forward: 0,
            wipetype: SEQ_WIPE_SINGLE,
        }
    }
}

/// `GlowVars`.
#[derive(Clone)]
pub struct GlowVars {
    /// `float fMini`
    pub f_mini: f32,
    /// `float fClamp`
    pub f_clamp: f32,
    /// `float fBoost`
    pub f_boost: f32,
    /// `float dDist`
    pub d_dist: f32,
    /// `int dQuality`
    pub d_quality: i32,
    /// `int bNoComp`
    pub b_no_comp: i32,
}
impl Default for GlowVars {
    fn default() -> Self {
        Self {
            f_mini: 0.0,
            f_clamp: 0.0,
            f_boost: 0.0,
            d_dist: 0.0,
            d_quality: 0,
            b_no_comp: 0,
        }
    }
}

/// `TransformVarsLegacy`.
#[derive(Clone)]
pub struct TransformVarsLegacy {
    /// `float ScalexIni`
    pub scalex_ini: f32,
    /// `float ScaleyIni`
    pub scaley_ini: f32,
    /// `float xIni`
    pub x_ini: f32,
    /// `float yIni`
    pub y_ini: f32,
    /// `float rotIni`
    pub rot_ini: f32,
    /// `int percent`
    pub percent: i32,
    /// `int interpolation`
    pub interpolation: i32,
    /// `int uniform_scale`
    pub uniform_scale: i32,
}
impl Default for TransformVarsLegacy {
    fn default() -> Self {
        Self {
            scalex_ini: Default::default(),
            scaley_ini: Default::default(),
            x_ini: 0.0,
            y_ini: 0.0,
            rot_ini: 0.0,
            percent: 0,
            interpolation: 0,
            uniform_scale: 0,
        }
    }
}

/// `SolidColorVars`.
#[derive(Clone)]
pub struct SolidColorVars {
    /// `float col[3]`
    pub col: [f32; 3],
    /// `int width`
    pub width: i32,
    /// `int height`
    pub height: i32,
}
impl Default for SolidColorVars {
    fn default() -> Self {
        Self {
            col: [0.0; 3],
            width: Default::default(),
            height: Default::default(),
        }
    }
}

/// `SpeedControlVars`.
#[derive(Clone)]
pub struct SpeedControlVars {
    /// `float *frameMap`
    pub frame_map: usize,
    /// `float globalSpeed_legacy`
    pub global_speed_legacy: f32,
    /// `eEffectSpeedControlFlags flags`
    pub flags: Effectspeedcontrolflags,
    /// `eEffectSpeedControlType speed_control_type`
    pub speed_control_type: Effectspeedcontroltype,
    /// `float speed_fader`
    pub speed_fader: f32,
    /// `float speed_fader_length`
    pub speed_fader_length: f32,
    /// `float speed_fader_frame_number`
    pub speed_fader_frame_number: f32,
}
impl Default for SpeedControlVars {
    fn default() -> Self {
        Self {
            frame_map: NIL,
            global_speed_legacy: 0.0,
            flags: SEQ_SPEED_NONE,
            speed_control_type: SEQ_SPEED_STRETCH,
            speed_fader: 0.0,
            speed_fader_length: 0.0,
            speed_fader_frame_number: 0.0,
        }
    }
}

/// `GaussianBlurVars`.
#[derive(Clone)]
pub struct GaussianBlurVars {
    /// `float size_x`
    pub size_x: f32,
    /// `float size_y`
    pub size_y: f32,
}
impl Default for GaussianBlurVars {
    fn default() -> Self {
        Self {
            size_x: 0.0,
            size_y: 0.0,
        }
    }
}

/// `TextVars`.
#[derive(Clone)]
pub struct TextVars {
    /// `char *text_ptr`
    pub text_ptr: usize,
    /// `int text_len_bytes`
    pub text_len_bytes: i32,
    /// `VFont *text_font`
    pub text_font: usize,
    /// `int text_blf_id`
    pub text_blf_id: i32,
    /// `float text_size`
    pub text_size: f32,
    /// `float space_line`
    pub space_line: f32,
    /// `float abs_space_line`
    pub abs_space_line: f32,
    /// `float color[4]`
    pub color: [f32; 4],
    /// `float shadow_color[4]`
    pub shadow_color: [f32; 4],
    /// `float box_color[4]`
    pub box_color: [f32; 4],
    /// `float outline_color[4]`
    pub outline_color: [f32; 4],
    /// `float loc[2]`
    pub loc: [f32; 2],
    /// `float wrap_width`
    pub wrap_width: f32,
    /// `float box_margin`
    pub box_margin: f32,
    /// `float box_roundness`
    pub box_roundness: f32,
    /// `float shadow_angle`
    pub shadow_angle: f32,
    /// `float shadow_offset`
    pub shadow_offset: f32,
    /// `float shadow_blur`
    pub shadow_blur: f32,
    /// `float outline_width`
    pub outline_width: f32,
    /// `eEffectTextFlags flag`
    pub flag: Effecttextflags,
    /// `eEffectTextAlignX align`
    pub align: Effecttextalignx,
    /// `int cursor_offset`
    pub cursor_offset: i32,
    /// `int selection_start_offset`
    pub selection_start_offset: i32,
    /// `int selection_end_offset`
    pub selection_end_offset: i32,
    /// `char align_y_legacy`
    pub align_y_legacy: i8,
    /// `eEffectTextAnchorX anchor_x`
    pub anchor_x: Effecttextanchorx,
    /// `eEffectTextAnchorY anchor_y`
    pub anchor_y: Effecttextanchory,
    /// `seq::TextVarsRuntime *runtime`
    pub runtime: usize,
    /// `char text_legacy[512]`
    pub text_legacy: [u8; 512],
    pub textbox_state: TextboxState,
}
impl Default for TextVars {
    fn default() -> Self {
        Self {
            text_ptr: NIL,
            text_len_bytes: 0,
            text_font: NIL,
            text_blf_id: 0,
            text_size: 0.0,
            space_line: 1.0,
            abs_space_line: 1.0,
            color: [0.0; 4],
            shadow_color: [0.0; 4],
            box_color: [0.0; 4],
            outline_color: [0.0; 4],
            loc: [0.0; 2],
            wrap_width: 0.0,
            box_margin: 0.0,
            box_roundness: 0.0,
            shadow_angle: 0.0,
            shadow_offset: 0.0,
            shadow_blur: 0.0,
            outline_width: 0.0,
            flag: SEQ_TEXT_NONE,
            align: SEQ_TEXT_ALIGN_X_LEFT,
            cursor_offset: 0,
            selection_start_offset: 0,
            selection_end_offset: 0,
            align_y_legacy: 0,
            anchor_x: SEQ_TEXT_ANCHOR_X_LEFT,
            anchor_y: SEQ_TEXT_ANCHOR_Y_TOP,
            runtime: NIL,
            text_legacy: [0; 512],
            textbox_state: Default::default(),
        }
    }
}

/// `ColorMixVars`.
#[derive(Clone)]
pub struct ColorMixVars {
    /// `StripBlendMode blend_effect`
    pub blend_effect: Stripblendmode,
    /// `float factor`
    pub factor: f32,
}
impl Default for ColorMixVars {
    fn default() -> Self {
        Self {
            blend_effect: STRIP_BLEND_REPLACE,
            factor: 0.0,
        }
    }
}

/// `CompositorEffectVars`.
#[derive(Clone)]
pub struct CompositorEffectVars {
    /// `bNodeTree *node_group`
    pub node_group: usize,
    /// `IDProperty *system_properties`
    pub system_properties: usize,
}
impl Default for CompositorEffectVars {
    fn default() -> Self {
        Self {
            node_group: NIL,
            system_properties: NIL,
        }
    }
}

/// `StripModifierData`.
#[derive(Clone)]
pub struct StripModifierData {
    /// `StripModifierData *next`
    pub next: usize,
    /// `StripModifierData *prev`
    pub prev: usize,
    /// `eStripModifierType type`
    pub type_: Stripmodifiertype,
    /// `eStripModifierFlag flag`
    pub flag: Stripmodifierflag,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `eModMaskInput mask_input_type`
    pub mask_input_type: Modmaskinput,
    /// `eModMaskTime mask_time`
    pub mask_time: Modmasktime,
    /// `Strip *mask_strip`
    pub mask_strip: usize,
    /// `Mask *mask_id`
    pub mask_id: usize,
    /// `int persistent_uid`
    pub persistent_uid: i32,
    /// `uint16_t layout_panel_open_flag`
    pub layout_panel_open_flag: u16,
    /// `uint16_t ui_expand_flag`
    pub ui_expand_flag: u16,
    /// `IDProperty *system_properties`
    pub system_properties: usize,
    /// `blender::seq::StripModifierDataRuntime *runtime`
    pub runtime: usize,
}
impl Default for StripModifierData {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: eSeqModifierType_None,
            flag: STRIP_MODIFIER_FLAG_NONE,
            name: [0; 64],
            mask_input_type: STRIP_MASK_INPUT_STRIP,
            mask_time: STRIP_MASK_TIME_RELATIVE,
            mask_strip: NIL,
            mask_id: NIL,
            persistent_uid: 0,
            layout_panel_open_flag: 0,
            ui_expand_flag: 0,
            system_properties: NIL,
            runtime: NIL,
        }
    }
}

/// `ColorBalanceModifierData`.
#[derive(Clone)]
pub struct ColorBalanceModifierData {
    pub modifier: StripModifierData,
    pub color_balance: StripColorBalance,
    /// `float color_multiply`
    pub color_multiply: f32,
}
impl Default for ColorBalanceModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            color_balance: Default::default(),
            color_multiply: 0.0,
        }
    }
}

/// `CurvesModifierData`.
#[derive(Clone)]
pub struct CurvesModifierData {
    pub modifier: StripModifierData,
    pub curve_mapping: CurveMapping,
}
impl Default for CurvesModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve_mapping: Default::default(),
        }
    }
}

/// `HueCorrectModifierData`.
#[derive(Clone)]
pub struct HueCorrectModifierData {
    pub modifier: StripModifierData,
    pub curve_mapping: CurveMapping,
}
impl Default for HueCorrectModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            curve_mapping: Default::default(),
        }
    }
}

/// `BrightContrastModifierData`.
#[derive(Clone)]
pub struct BrightContrastModifierData {
    pub modifier: StripModifierData,
    /// `float bright`
    pub bright: f32,
    /// `float contrast`
    pub contrast: f32,
}
impl Default for BrightContrastModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            bright: 0.0,
            contrast: 0.0,
        }
    }
}

/// `SequencerMaskModifierData`.
#[derive(Clone)]
pub struct SequencerMaskModifierData {
    pub modifier: StripModifierData,
}
impl Default for SequencerMaskModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
        }
    }
}

/// `WhiteBalanceModifierData`.
#[derive(Clone)]
pub struct WhiteBalanceModifierData {
    pub modifier: StripModifierData,
    /// `float white_value[3]`
    pub white_value: [f32; 3],
}
impl Default for WhiteBalanceModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            white_value: [0.0; 3],
        }
    }
}

/// `SequencerTonemapModifierData`.
#[derive(Clone)]
pub struct SequencerTonemapModifierData {
    pub modifier: StripModifierData,
    /// `float key`
    pub key: f32,
    /// `float offset`
    pub offset: f32,
    /// `float gamma`
    pub gamma: f32,
    /// `float intensity`
    pub intensity: f32,
    /// `float contrast`
    pub contrast: f32,
    /// `float adaptation`
    pub adaptation: f32,
    /// `float correction`
    pub correction: f32,
    /// `eModTonemapType type`
    pub type_: Modtonemaptype,
}
impl Default for SequencerTonemapModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            key: 0.18,
            offset: 1.0,
            gamma: 1.0,
            intensity: 0.0,
            contrast: 0.0,
            adaptation: 1.0,
            correction: 0.0,
            type_: SEQ_TONEMAP_RD_PHOTORECEPTOR,
        }
    }
}

/// `SequencerCompositorModifierData`.
#[derive(Clone)]
pub struct SequencerCompositorModifierData {
    pub modifier: StripModifierData,
    /// `SequencerCompositorModifierFlag flag`
    pub flag: Sequencercompositormodifierflag,
    /// `bNodeTree *node_group`
    pub node_group: usize,
}
impl Default for SequencerCompositorModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            flag: SEQ_COMP_MOD_NONE,
            node_group: NIL,
        }
    }
}

/// `EQCurveMappingData`.
#[derive(Clone)]
pub struct EQCurveMappingData {
    /// `EQCurveMappingData *next`
    pub next: usize,
    /// `EQCurveMappingData *prev`
    pub prev: usize,
    pub curve_mapping: CurveMapping,
}
impl Default for EQCurveMappingData {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            curve_mapping: Default::default(),
        }
    }
}

/// `SoundEqualizerModifierData`.
#[derive(Clone)]
pub struct SoundEqualizerModifierData {
    pub modifier: StripModifierData,
    /// `ListBaseT<EQCurveMappingData>`
    pub graphics: ListBase,
}
impl Default for SoundEqualizerModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            graphics: ListBase::default(),
        }
    }
}

/// `PitchModifierData`.
#[derive(Clone)]
pub struct PitchModifierData {
    pub modifier: StripModifierData,
    /// `ePitchMode mode`
    pub mode: Pitchmode,
    /// `int semitones`
    pub semitones: i32,
    /// `int cents`
    pub cents: i32,
    /// `float ratio`
    pub ratio: f32,
    /// `char preserve_formant`
    pub preserve_formant: i8,
    /// `ePitchQuality quality`
    pub quality: Pitchquality,
}
impl Default for PitchModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            mode: PITCH_MODE_SEMITONES,
            semitones: 0,
            cents: 0,
            ratio: 0.0,
            preserve_formant: 0,
            quality: PITCH_QUALITY_HIGH,
        }
    }
}

/// `EchoModifierData`.
#[derive(Clone)]
pub struct EchoModifierData {
    pub modifier: StripModifierData,
    /// `float delay`
    pub delay: f32,
    /// `float feedback`
    pub feedback: f32,
    /// `float mix`
    pub mix: f32,
}
impl Default for EchoModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            delay: 0.0,
            feedback: 0.0,
            mix: 0.0,
        }
    }
}
