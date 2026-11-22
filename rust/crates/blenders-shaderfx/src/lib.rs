//! Port of `source/blender/shader_fx/intern/*.cc` —
//! GPencil shader-effect type registry + init defaults.

/// `eShaderFxType` verbatim ordinals.
pub const E_SHADER_FX_TYPE_BLUR: usize = 0;
pub const E_SHADER_FX_TYPE_COLORIZE: usize = 1;
pub const E_SHADER_FX_TYPE_FLIP: usize = 2;
pub const E_SHADER_FX_TYPE_GLOW: usize = 3;
pub const E_SHADER_FX_TYPE_PIXEL: usize = 4;
pub const E_SHADER_FX_TYPE_RIM: usize = 5;
pub const E_SHADER_FX_TYPE_SHADOW: usize = 6;
pub const E_SHADER_FX_TYPE_SWIRL: usize = 7;
pub const E_SHADER_FX_TYPE_WAVE: usize = 8;
pub const NUM_SHADER_FX_TYPES: usize = 9;
/// `FX_FLIP_HORIZONTAL`/`VERTICAL` verbatim.
pub const FX_FLIP_HORIZONTAL: i32 = 1 << 0;
pub const FX_FLIP_VERTICAL: i32 = 1 << 1;
/// `eShaderFxColorizeMode_GrayScale` verbatim.
pub const E_SHADER_FX_COLORIZE_MODE_GRAYSCALE: i16 = 0;
/// `eShaderFxRimMode_Overlay` verbatim.
pub const E_SHADER_FX_RIM_MODE_OVERLAY: i16 = 2;

/// `FlipShaderFxData` init_data (verbatim —
/// flag |= HORIZONTAL).
pub fn flip_init(flag: &mut i32) {
    *flag |= FX_FLIP_HORIZONTAL;
}
/// `PixelShaderFxData` verbatim init.
pub fn pixel_init() -> ([i32; 2], [f32; 4]) {
    ([5, 5], [0.0, 0.0, 0.0, 0.9])
}
/// `WaveShaderFxData` verbatim init.
pub fn wave_init() -> (f32, f32, f32, i32) {
    (10.0, 20.0, 0.0, 1)
}
/// `BlurShaderFxData` verbatim init.
pub fn blur_init() -> ([f32; 2], i32, f32) {
    ([50.0, 50.0], 8, 0.0)
}
/// `ColorizeShaderFxData` verbatim init.
pub fn colorize_init() -> ([f32; 4], [f32; 4], i16, f32) {
    ([0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0], E_SHADER_FX_COLORIZE_MODE_GRAYSCALE, 0.5)
}
/// `SwirlShaderFxData` verbatim init.
pub fn swirl_init() -> (i32, f32) {
    (100, std::f32::consts::FRAC_PI_2)
}
/// `RimShaderFxData` verbatim init.
pub fn rim_init() -> ([f32; 2], [f32; 3], [f32; 3], i16, [i32; 2], i32) {
    (
        [50.0, -100.0],
        [1.0, 1.0, 0.5],
        [0.0, 0.0, 0.0],
        E_SHADER_FX_RIM_MODE_OVERLAY,
        [0, 0],
        2,
    )
}
/// `GlowShaderFxData` verbatim init.
pub fn glow_init() -> ([f32; 4], [f32; 3], [f32; 2], f32, i32) {
    (
        [0.75, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        [50.0, 50.0],
        0.1,
        8,
    )
}
/// `ShadowShaderFxData` verbatim init.
pub fn shadow_init() -> (f32, [f32; 2], [f32; 2], [f32; 4], f32, f32, f32, i32, [i32; 2], i32) {
    (
        0.0,
        [15.0, 20.0],
        [1.0, 1.0],
        [0.0, 0.0, 0.0, 0.8],
        10.0,
        20.0,
        0.0,
        1,
        [5, 5],
        2,
    )
}

/// `shaderfx_type_init` table (verbatim — 9 slots by
/// eShaderFxType ordinal).
pub fn shaderfx_type_init() -> [&'static str; NUM_SHADER_FX_TYPES] {
    [
        "Blur",
        "Colorize",
        "Flip",
        "Glow",
        "Pixel",
        "Rim",
        "Shadow",
        "Swirl",
        "Wave",
    ]
}
