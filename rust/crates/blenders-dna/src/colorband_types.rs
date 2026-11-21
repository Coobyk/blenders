//! Port of `DNA_colorband_types.h` — `CBData`/`ColorBand` + enums.

pub type eColorBand_ColorMode = i8;
pub const COLBAND_BLEND_RGB: eColorBand_ColorMode = 0;
pub const COLBAND_BLEND_HSV: eColorBand_ColorMode = 1;
pub const COLBAND_BLEND_HSL: eColorBand_ColorMode = 2;

pub type eColorBand_Interp = i8;
pub const COLBAND_INTERP_LINEAR: eColorBand_Interp = 0;
pub const COLBAND_INTERP_EASE: eColorBand_Interp = 1;
pub const COLBAND_INTERP_B_SPLINE: eColorBand_Interp = 2;
pub const COLBAND_INTERP_CARDINAL: eColorBand_Interp = 3;
pub const COLBAND_INTERP_CONSTANT: eColorBand_Interp = 4;

pub type eColorBand_HueInterp = i8;
pub const COLBAND_HUE_NEAR: eColorBand_HueInterp = 0;
pub const COLBAND_HUE_FAR: eColorBand_HueInterp = 1;
pub const COLBAND_HUE_CW: eColorBand_HueInterp = 2;
pub const COLBAND_HUE_CCW: eColorBand_HueInterp = 3;

/// `CBData`.
#[derive(Clone, Copy, Default)]
pub struct CBData {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub pos: f32,
    pub cur: i32,
}

/// `MAXCOLORBAND` — `data[32]`.
pub const MAXCOLORBAND: usize = 32;

/// `ColorBand`.
#[derive(Clone)]
pub struct ColorBand {
    pub tot: i16,
    pub cur: i16,
    pub ipotype: eColorBand_Interp,
    pub ipotype_hue: eColorBand_HueInterp,
    pub color_mode: eColorBand_ColorMode,
    pub data: [CBData; MAXCOLORBAND],
}

impl Default for ColorBand {
    fn default() -> Self {
        Self {
            tot: 0,
            cur: 0,
            ipotype: COLBAND_INTERP_LINEAR,
            ipotype_hue: COLBAND_HUE_NEAR,
            color_mode: COLBAND_BLEND_RGB,
            data: [CBData::default(); MAXCOLORBAND],
        }
    }
}
