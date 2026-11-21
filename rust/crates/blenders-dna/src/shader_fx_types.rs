//! Port of `DNA_shader_fx_types.h` — GP shader FX data structs.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::id::NIL;
/// `ShaderFxMode`.
pub type Shaderfxmode = i32;
pub const eShaderFxMode_Realtime: Shaderfxmode = 1 << 0;
pub const eShaderFxMode_Render: Shaderfxmode = 1 << 1;
pub const eShaderFxMode_Editmode: Shaderfxmode = 1 << 2;
pub const eShaderFxMode_Expanded_DEPRECATED: Shaderfxmode = 1 << 3;

/// `ShaderFxFlag`.
pub type Shaderfxflag = i16;
pub const eShaderFxFlag_OverrideLibrary_Local: Shaderfxflag = 1 << 0;

/// `eBlurShaderFx_Flag`.
pub type BlurshaderfxFlag = i32;
pub const FX_BLUR_DOF_MODE: BlurshaderfxFlag = 1 << 0;

/// `ColorizeShaderFxModes`.
pub type Colorizeshaderfxmodes = i32;
pub const eShaderFxColorizeMode_GrayScale: Colorizeshaderfxmodes = 0;
pub const eShaderFxColorizeMode_Sepia: Colorizeshaderfxmodes = 1;
pub const eShaderFxColorizeMode_Duotone: Colorizeshaderfxmodes = 2;
pub const eShaderFxColorizeMode_Custom: Colorizeshaderfxmodes = 3;
pub const eShaderFxColorizeMode_Transparent: Colorizeshaderfxmodes = 4;

/// `eFlipShaderFx_Flag`.
pub type FlipshaderfxFlag = i32;
pub const FX_FLIP_HORIZONTAL: FlipshaderfxFlag = 1 << 0;
pub const FX_FLIP_VERTICAL: FlipshaderfxFlag = 1 << 1;

/// `GlowShaderFxModes`.
pub type Glowshaderfxmodes = i32;
pub const eShaderFxGlowMode_Luminance: Glowshaderfxmodes = 0;
pub const eShaderFxGlowMode_Color: Glowshaderfxmodes = 1;

/// `eGlowShaderFx_Flag`.
pub type GlowshaderfxFlag = i32;
pub const FX_GLOW_USE_ALPHA: GlowshaderfxFlag = 1 << 0;

/// `ePixelShaderFx_Flag`.
pub type PixelshaderfxFlag = i32;
pub const FX_PIXEL_FILTER_NEAREST: PixelshaderfxFlag = 1 << 0;

/// `RimShaderFxModes`.
pub type Rimshaderfxmodes = i32;
pub const eShaderFxRimMode_Normal: Rimshaderfxmodes = 0;
pub const eShaderFxRimMode_Overlay: Rimshaderfxmodes = 1;
pub const eShaderFxRimMode_Add: Rimshaderfxmodes = 2;
pub const eShaderFxRimMode_Subtract: Rimshaderfxmodes = 3;
pub const eShaderFxRimMode_Multiply: Rimshaderfxmodes = 4;
pub const eShaderFxRimMode_Divide: Rimshaderfxmodes = 5;

/// `eShadowShaderFx_Flag`.
pub type ShadowshaderfxFlag = i32;
pub const FX_SHADOW_USE_OBJECT: ShadowshaderfxFlag = 1 << 0;
pub const FX_SHADOW_USE_WAVE: ShadowshaderfxFlag = 1 << 1;

/// `eSwirlShaderFx_Flag`.
pub type SwirlshaderfxFlag = i32;
pub const FX_SWIRL_MAKE_TRANSPARENT: SwirlshaderfxFlag = 1 << 0;

/// `ShaderFxType`.
pub type Shaderfxtype = i32;
pub const eShaderFxType_None: Shaderfxtype = 0;
pub const eShaderFxType_Blur: Shaderfxtype = 1;
pub const eShaderFxType_Flip: Shaderfxtype = 2;
pub const eShaderFxType_Light_deprecated: Shaderfxtype = 3;
pub const eShaderFxType_Pixel: Shaderfxtype = 4;
pub const eShaderFxType_Swirl: Shaderfxtype = 5;
pub const eShaderFxType_Wave: Shaderfxtype = 6;
pub const eShaderFxType_Rim: Shaderfxtype = 7;
pub const eShaderFxType_Colorize: Shaderfxtype = 8;
pub const eShaderFxType_Shadow: Shaderfxtype = 9;
pub const eShaderFxType_Glow: Shaderfxtype = 10;
pub const NUM_SHADER_FX_TYPES: Shaderfxtype = eShaderFxType_Glow + 1;


/// `ShaderFxData` — base FX data (embedded first in each FX subtype).
#[derive(Clone)]
pub struct ShaderFxData {
    pub next: usize,
    pub prev: usize,
    pub type_: i32,
    pub mode: i32,
    pub flag: i16,
    pub ui_expand_flag: i16,
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `char *error`.
    pub error: Option<String>,
}
impl Default for ShaderFxData {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: eShaderFxType_None,
            mode: 0,
            flag: 0,
            ui_expand_flag: 0,
            name: [0; 64],
            error: None,
        }
    }
}

/// `ShaderFxData_Runtime`.
#[derive(Clone, Default)]
pub struct ShaderFxDataRuntime {
    pub loc: [f32; 3],
    /// `DRWShadingGroup *fx_sh`, `*fx_sh_b`, `*fx_sh_c`.
    pub fx_sh: usize,
    pub fx_sh_b: usize,
    pub fx_sh_c: usize,
}

/// `BlurShaderFxData`.
#[derive(Clone, Default)]
pub struct BlurShaderFxData {
    pub shaderfx: ShaderFxData,
    pub radius: [f32; 2],
    pub flag: i32,
    pub samples: i32,
    pub rotation: f32,
    pub runtime: ShaderFxDataRuntime,
}

/// `ColorizeShaderFxData`.
#[derive(Clone)]
pub struct ColorizeShaderFxData {
    pub shaderfx: ShaderFxData,
    pub mode: i32,
    pub low_color: [f32; 4],
    pub high_color: [f32; 4],
    pub factor: f32,
    pub flag: i32,
    pub runtime: ShaderFxDataRuntime,
}
impl Default for ColorizeShaderFxData {
    fn default() -> Self {
        Self {
            shaderfx: ShaderFxData::default(),
            mode: eShaderFxColorizeMode_GrayScale,
            low_color: [0.0; 4],
            high_color: [0.0; 4],
            factor: 0.0,
            flag: 0,
            runtime: ShaderFxDataRuntime::default(),
        }
    }
}

/// `FlipShaderFxData`.
#[derive(Clone, Default)]
pub struct FlipShaderFxData {
    pub shaderfx: ShaderFxData,
    pub flag: i32,
    pub flipmode: i32,
    pub runtime: ShaderFxDataRuntime,
}

/// `GlowShaderFxData`.
#[derive(Clone)]
pub struct GlowShaderFxData {
    pub shaderfx: ShaderFxData,
    pub glow_color: [f32; 4],
    pub select_color: [f32; 3],
    pub threshold: f32,
    pub flag: i32,
    pub mode: i32,
    pub blur: [f32; 2],
    pub samples: i32,
    pub rotation: f32,
    pub blend_mode: i32,
    pub runtime: ShaderFxDataRuntime,
}
impl Default for GlowShaderFxData {
    fn default() -> Self {
        Self {
            shaderfx: ShaderFxData::default(),
            glow_color: [0.0; 4],
            select_color: [0.0; 3],
            threshold: 0.0,
            flag: 0,
            mode: eShaderFxGlowMode_Luminance,
            blur: [0.0; 2],
            samples: 0,
            rotation: 0.0,
            blend_mode: 0,
            runtime: ShaderFxDataRuntime::default(),
        }
    }
}

/// `PixelShaderFxData`.
#[derive(Clone, Default)]
pub struct PixelShaderFxData {
    pub shaderfx: ShaderFxData,
    pub size: [i32; 3],
    pub flag: i32,
    pub rgba: [f32; 4],
    pub runtime: ShaderFxDataRuntime,
}

/// `RimShaderFxData`.
#[derive(Clone)]
pub struct RimShaderFxData {
    pub shaderfx: ShaderFxData,
    pub offset: [i32; 2],
    pub flag: i32,
    pub rim_rgb: [f32; 3],
    pub mask_rgb: [f32; 3],
    pub mode: i32,
    pub blur: [i32; 2],
    pub samples: i32,
    pub runtime: ShaderFxDataRuntime,
}
impl Default for RimShaderFxData {
    fn default() -> Self {
        Self {
            shaderfx: ShaderFxData::default(),
            offset: [0; 2],
            flag: 0,
            rim_rgb: [0.0; 3],
            mask_rgb: [0.0; 3],
            mode: eShaderFxRimMode_Normal,
            blur: [0; 2],
            samples: 0,
            runtime: ShaderFxDataRuntime::default(),
        }
    }
}

/// `ShadowShaderFxData`.
#[derive(Clone, Default)]
pub struct ShadowShaderFxData {
    pub shaderfx: ShaderFxData,
    /// `Object *object`.
    pub object: usize,
    pub offset: [i32; 2],
    pub flag: i32,
    pub shadow_rgba: [f32; 4],
    pub amplitude: f32,
    pub period: f32,
    pub phase: f32,
    pub orientation: i32,
    pub scale: [f32; 2],
    pub rotation: f32,
    pub blur: [i32; 2],
    pub samples: i32,
    pub runtime: ShaderFxDataRuntime,
}

/// `SwirlShaderFxData`.
#[derive(Clone, Default)]
pub struct SwirlShaderFxData {
    pub shaderfx: ShaderFxData,
    /// `Object *object`.
    pub object: usize,
    pub flag: i32,
    pub radius: i32,
    pub angle: f32,
    pub transparent: i32,
    pub runtime: ShaderFxDataRuntime,
}

/// `WaveShaderFxData`.
#[derive(Clone, Default)]
pub struct WaveShaderFxData {
    pub shaderfx: ShaderFxData,
    pub amplitude: f32,
    pub period: f32,
    pub phase: f32,
    pub orientation: i32,
    pub flag: i32,
    pub runtime: ShaderFxDataRuntime,
}
