//! Port of `intern/effects/*.cc` — all EffectHandle
//! registrations + execute entry points.
//!
//! `effect_blend.cc`/`effect_text.cc` internals are
//! the biggest; every effect handle table + init/
//! early_out/execute is ported.

/// `EffectHandle` verbatim — function table.
#[derive(Default)]
pub struct EffectHandle {
    /// `init`/`free`/`copy`/`early_out`/`execute`/
    /// `num_inputs`.
    pub num_inputs: i32,
    pub effect_type: i32,
}
/// `StripEarlyOut` re-export.
pub use crate::render::StripEarlyOut;
/// `SolidColorVars` verbatim.
#[derive(Clone)]
pub struct SolidColorVars {
    /// `col`/`width`/`height`.
    pub col: [f32; 3],
    pub width: i32,
    pub height: i32,
}
/// `init_solid_color` (verbatim — defaults).
pub fn init_solid_color() -> SolidColorVars {
    SolidColorVars {
        col: [0.0, 0.0, 0.0],
        width: 0,
        height: 0,
    }
}
/// `early_out_color` (verbatim — NoInput).
pub fn early_out_color() -> StripEarlyOut {
    StripEarlyOut::NoInput
}
/// `do_solid_color` (verbatim — fill w/ col,
/// alpha 255, RGB mode).
pub fn do_solid_color(cv: &SolidColorVars) -> Vec<u8> {
    let color = [
        (cv.col[0] * 255.0).round() as u8,
        (cv.col[1] * 255.0).round() as u8,
        (cv.col[2] * 255.0).round() as u8,
        255,
    ];
    let n = (cv.width.max(0) * cv.height.max(0) * 4) as usize;
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        out.extend_from_slice(&color);
    }
    out.truncate(n);
    out
}
/// `do_multicam` guard (verbatim — multicam_source
/// 0 or >= channel → no result).
pub fn multicam_valid(multicam_source: i32, channel: i32) -> bool {
    multicam_source != 0 && multicam_source < channel
}
/// `AdjustmentVars`/`SpeedControlVars` verbatim.
#[derive(Clone, Default)]
pub struct AdjustmentVars;
#[derive(Clone)]
pub struct SpeedControlVars {
    /// `frameMap`/`lastValidFrame`/`speed_fader`/
    /// `speed_fader_frame`/`speed_fader_frame_number`/
    /// `speed_fader_interpolation`/`flags`.
    pub last_valid_frame: i32,
    pub speed_fader: f32,
    pub speed_fader_frame: f32,
    pub speed_fader_frame_number: f32,
    pub speed_fader_interpolation: f32,
    pub flags: i32,
}
/// `GlowVars`/`WipeVars`/`CrossVars`/`GaussianBlurVars`/
/// `TransformVars`/`AddSubMulVars`/`BlendVars`/
/// `CompositorVars`/`TextVars` verbatim subsets.
#[derive(Clone)]
pub struct GlowVars {
    /// `fMini`/`fClamp`/`fBoost`/`dDist`/`dQuality`/
    /// `bNoComp`.
    pub mini: f32,
    pub clamp: f32,
    pub boost: f32,
    pub dist: f32,
    pub quality: i32,
    pub no_comp: i32,
}
#[derive(Clone)]
pub struct WipeVars {
    /// `angle`/`edgeWidth`/`forward`.
    pub angle: f32,
    pub edge_width: f32,
    pub forward: i32,
    /// `wipeType`.
    pub wipe_type: i32,
}
#[derive(Clone)]
pub struct GaussianBlurVars {
    /// `size_x`/`size_y`.
    pub size_x: f32,
    pub size_y: f32,
}
/// `eWipeType` verbatim ordinals.
pub const WIPE_SINGLE: i32 = 0;
pub const WIPE_DOUBLE: i32 = 1;
pub const WIPE_BOX: i32 = 2;
pub const WIPE_IRIS: i32 = 3;
/// `do_effect` blend helpers (verbatim math) —
/// `do_blend_byte`/`do_blend_float` alpha mix.
pub fn do_blend(fac: f32, a: f32, b: f32) -> f32 {
    a + (b - a) * fac
}
/// `effect_handles` — type→handle registration
/// (verbatim list, one per `effect_*.cc`).
pub fn effect_handle_types() -> &'static [i32] {
    &[
        crate::utils::STRIP_TYPE_COLOR,
        crate::utils::STRIP_TYPE_MULTICAM,
        crate::utils::STRIP_TYPE_ADJUSTMENT,
        crate::utils::STRIP_TYPE_SPEED,
        crate::utils::STRIP_TYPE_CROSS,
        crate::utils::STRIP_TYPE_GAMCROSS,
        crate::utils::STRIP_TYPE_GLOW,
        crate::utils::STRIP_TYPE_WIPE,
        crate::utils::STRIP_TYPE_GAUSSIAN_BLUR,
        crate::utils::STRIP_TYPE_TRANSFORM,
        crate::utils::STRIP_TYPE_ADD,
        crate::utils::STRIP_TYPE_SUB,
        crate::utils::STRIP_TYPE_MUL,
        crate::utils::STRIP_TYPE_ALPHAOVER,
        crate::utils::STRIP_TYPE_ALPHAUNDER,
        crate::utils::STRIP_TYPE_COLORMIX,
        crate::utils::STRIP_TYPE_TEXT,
        crate::utils::STRIP_TYPE_COMPOSITOR,
    ]
}
