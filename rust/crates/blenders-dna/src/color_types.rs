//! Port of `DNA_color_types.h` — `CurveMapping`/`CurveMap`/`CurveMapPoint`,
//! `Histogram`, `Scopes`, `ColorManaged*Settings` + enums.
#![allow(non_camel_case_types)]
use crate::id::NIL;
use crate::vec_types::Rctf;

pub const CM_RESOL: usize = 32;
pub const CM_TABLE: usize = 256;
pub const CM_TABLEDIV: f32 = 1.0 / 256.0;
pub const CM_TOT: usize = 4;
pub const GPU_SKY_WIDTH: i32 = 512;
pub const GPU_SKY_HEIGHT: i32 = 256;

/// `eCurveMapPoint_Flag`.
pub const CUMA_SELECT: i16 = 1 << 0;
pub const CUMA_HANDLE_VECTOR: i16 = 1 << 1;
pub const CUMA_HANDLE_AUTO_ANIM: i16 = 1 << 2;
pub const CUMA_REMOVE: i16 = 1 << 3;
pub const CUMA_ACTIVE: i16 = 1 << 4;

/// `eCurveMappingFlags`.
pub type CurveMappingFlags = i32;
pub const CUMA_DO_CLIP: CurveMappingFlags = 1 << 0;
pub const CUMA_PREMULLED: CurveMappingFlags = 1 << 1;
pub const CUMA_DRAW_CFRA: CurveMappingFlags = 1 << 2;
pub const CUMA_DRAW_SAMPLE: CurveMappingFlags = 1 << 3;
pub const CUMA_EXTEND_EXTRAPOLATE: CurveMappingFlags = 1 << 4;
pub const CUMA_USE_WRAPPING: CurveMappingFlags = 1 << 5;

/// `eCurveMappingPreset`.
pub type CurveMappingPreset = i32;
pub const CURVE_PRESET_LINE: CurveMappingPreset = 0;
pub const CURVE_PRESET_SHARP: CurveMappingPreset = 1;
pub const CURVE_PRESET_SMOOTH: CurveMappingPreset = 2;
pub const CURVE_PRESET_MAX: CurveMappingPreset = 3;
pub const CURVE_PRESET_MID8: CurveMappingPreset = 4;
pub const CURVE_PRESET_ROUND: CurveMappingPreset = 5;
pub const CURVE_PRESET_ROOT: CurveMappingPreset = 6;
pub const CURVE_PRESET_GAUSS: CurveMappingPreset = 7;
pub const CURVE_PRESET_BELL: CurveMappingPreset = 8;
pub const CURVE_PRESET_CONSTANT_MEDIAN: CurveMappingPreset = 9;

/// `eCurveMappingTone`.
pub type CurveMappingTone = i16;
pub const CURVE_TONE_STANDARD: CurveMappingTone = 0;
pub const CURVE_TONE_FILMLIKE: CurveMappingTone = 2;

/// `eHistogram_Mode`.
pub type HistogramMode = i16;
pub const HISTO_MODE_LUMA: HistogramMode = 0;
pub const HISTO_MODE_RGB: HistogramMode = 1;
pub const HISTO_MODE_R: HistogramMode = 2;
pub const HISTO_MODE_G: HistogramMode = 3;
pub const HISTO_MODE_B: HistogramMode = 4;
pub const HISTO_MODE_ALPHA: HistogramMode = 5;

/// `eHistogram_Flag`.
pub const HISTO_FLAG_LINE: i16 = 1 << 0;
pub const HISTO_FLAG_SAMPLELINE: i16 = 1 << 1;

/// `eScopes_WaveformMode`.
pub type ScopesWaveformMode = i32;
pub const SCOPES_WAVEFRM_LUMA: ScopesWaveformMode = 0;
pub const SCOPES_WAVEFRM_RGB_PARADE: ScopesWaveformMode = 1;
pub const SCOPES_WAVEFRM_YCC_601: ScopesWaveformMode = 2;
pub const SCOPES_WAVEFRM_YCC_709: ScopesWaveformMode = 3;
pub const SCOPES_WAVEFRM_YCC_JPEG: ScopesWaveformMode = 4;
pub const SCOPES_WAVEFRM_RGB: ScopesWaveformMode = 5;

/// `eScopes_VecscopeMode`.
pub type ScopesVecscopeMode = i32;
pub const SCOPES_VECSCOPE_RGB: ScopesVecscopeMode = 0;
pub const SCOPES_VECSCOPE_LUMA: ScopesVecscopeMode = 1;

/// `eColorManageDisplay_Emulation`.
pub type ColorManageDisplayEmulation = i8;
pub const COLORMANAGE_DISPLAY_EMULATION_AUTO: ColorManageDisplayEmulation = 0;
pub const COLORMANAGE_DISPLAY_EMULATION_OFF: ColorManageDisplayEmulation = 1;

/// `eColorManageView_Flag`.
pub type ColorManageViewFlag = i32;
pub const COLORMANAGE_VIEW_USE_CURVES: ColorManageViewFlag = 1 << 0;
pub const COLORMANAGE_VIEW_USE_DEPRECATED: ColorManageViewFlag = 1 << 1;
pub const COLORMANAGE_VIEW_USE_WHITE_BALANCE: ColorManageViewFlag = 1 << 2;
pub const COLORMANAGE_VIEW_ONLY_VIEW_LOOK: ColorManageViewFlag = 1 << 3;

/// `CurveMapPoint`.
#[derive(Clone, Copy, Default)]
pub struct CurveMapPoint {
    pub x: f32,
    pub y: f32,
    pub flag: i16,
    pub shorty: i16,
}

/// `CurveMap`.
#[derive(Clone)]
pub struct CurveMap {
    pub totpoint: i16,
    /// DNA_DEPRECATED `flag`.
    pub flag: i16,
    pub range: f32,
    pub mintable: f32,
    pub maxtable: f32,
    pub ext_in: [f32; 2],
    pub ext_out: [f32; 2],
    /// `CurveMapPoint *curve` → Vec.
    pub curve: Vec<CurveMapPoint>,
    /// `CurveMapPoint *table` → Vec.
    pub table: Vec<CurveMapPoint>,
    /// `CurveMapPoint *premultable` → Vec.
    pub premultable: Vec<CurveMapPoint>,
    pub premul_ext_in: [f32; 2],
    pub premul_ext_out: [f32; 2],
    pub default_handle_type: i16,
}
impl Default for CurveMap {
    fn default() -> Self {
        Self {
            totpoint: 0,
            flag: 0,
            range: 0.0,
            mintable: 0.0,
            maxtable: 0.0,
            ext_in: [0.0; 2],
            ext_out: [0.0; 2],
            curve: Vec::new(),
            table: Vec::new(),
            premultable: Vec::new(),
            premul_ext_in: [0.0; 2],
            premul_ext_out: [0.0; 2],
            default_handle_type: 0,
        }
    }
}

/// `CurveMapping`.
#[derive(Clone)]
pub struct CurveMapping {
    pub flag: CurveMappingFlags,
    pub cur: i32,
    pub preset: CurveMappingPreset,
    pub changed_timestamp: i32,
    pub curr: Rctf,
    pub clipr: Rctf,
    /// `CurveMap cm[4]`.
    pub cm: [CurveMap; 4],
    pub black: [f32; 3],
    pub white: [f32; 3],
    pub bwmul: [f32; 3],
    pub sample: [f32; 3],
    pub tone: CurveMappingTone,
}
impl Default for CurveMapping {
    fn default() -> Self {
        Self {
            flag: 0,
            cur: 0,
            preset: CURVE_PRESET_LINE,
            changed_timestamp: 0,
            curr: Rctf::default(),
            clipr: Rctf::default(),
            cm: Default::default(),
            black: [0.0; 3],
            white: [0.0; 3],
            bwmul: [0.0; 3],
            sample: [0.0; 3],
            tone: CURVE_TONE_STANDARD,
        }
    }
}

/// `Histogram`.
#[derive(Clone)]
pub struct Histogram {
    pub channels: i32,
    pub x_resolution: i32,
    pub data_luma: [f32; 256],
    pub data_r: [f32; 256],
    pub data_g: [f32; 256],
    pub data_b: [f32; 256],
    pub data_a: [f32; 256],
    pub xmax: f32,
    pub ymax: f32,
    pub mode: HistogramMode,
    pub flag: i16,
    pub height: i32,
    /// `float co[2][2]` — min/max range coords.
    pub co: [[f32; 2]; 2],
}
impl Default for Histogram {
    fn default() -> Self {
        Self {
            channels: 0,
            x_resolution: 0,
            data_luma: [0.0; 256],
            data_r: [0.0; 256],
            data_g: [0.0; 256],
            data_b: [0.0; 256],
            data_a: [0.0; 256],
            xmax: 0.0,
            ymax: 0.0,
            mode: 0,
            flag: 0,
            height: 0,
            co: [[0.0; 2]; 2],
        }
    }
}

/// `Scopes`.
#[derive(Clone)]
pub struct Scopes {
    pub ok: i32,
    pub sample_full: i32,
    pub sample_lines: i32,
    pub wavefrm_mode: ScopesWaveformMode,
    pub vecscope_mode: ScopesVecscopeMode,
    pub wavefrm_height: i32,
    pub vecscope_height: i32,
    pub waveform_tot: i32,
    pub accuracy: f32,
    pub wavefrm_alpha: f32,
    pub wavefrm_yfac: f32,
    pub vecscope_alpha: f32,
    /// `float minmax[3][2]`.
    pub minmax: [[f32; 2]; 3],
    pub hist: Histogram,
    /// `float *waveform_1/2/3`, `*vecscope`, `*vecscope_rgb` → Vec.
    pub waveform_1: Vec<f32>,
    pub waveform_2: Vec<f32>,
    pub waveform_3: Vec<f32>,
    pub vecscope: Vec<f32>,
    pub vecscope_rgb: Vec<f32>,
}
impl Default for Scopes {
    fn default() -> Self {
        Self {
            ok: 0,
            sample_full: 0,
            sample_lines: 0,
            wavefrm_mode: 0,
            vecscope_mode: 0,
            wavefrm_height: 0,
            vecscope_height: 0,
            waveform_tot: 0,
            accuracy: 0.0,
            wavefrm_alpha: 0.0,
            wavefrm_yfac: 0.0,
            vecscope_alpha: 0.0,
            minmax: [[0.0; 2]; 3],
            hist: Histogram::default(),
            waveform_1: Vec::new(),
            waveform_2: Vec::new(),
            waveform_3: Vec::new(),
            vecscope: Vec::new(),
            vecscope_rgb: Vec::new(),
        }
    }
}

/// `ColorManagedViewSettings`.
#[derive(Clone)]
pub struct ColorManagedViewSettings {
    pub flag: ColorManageViewFlag,
    /// `char look[64]`.
    pub look: [u8; 64],
    /// `char view_transform[64]`.
    pub view_transform: [u8; 64],
    pub exposure: f32,
    pub gamma: f32,
    pub temperature: f32,
    pub tint: f32,
    /// `CurveMapping *curve_mapping`.
    pub curve_mapping: usize,
}
impl Default for ColorManagedViewSettings {
    fn default() -> Self {
        Self {
            flag: 0,
            look: [0; 64],
            view_transform: [0; 64],
            exposure: 0.0,
            gamma: 0.0,
            temperature: 0.0,
            tint: 0.0,
            curve_mapping: NIL,
        }
    }
}

/// `ColorManagedDisplaySettings`.
#[derive(Clone)]
pub struct ColorManagedDisplaySettings {
    /// `char display_device[64]`.
    pub display_device: [u8; 64],
    pub emulation: ColorManageDisplayEmulation,
}
impl Default for ColorManagedDisplaySettings {
    fn default() -> Self {
        Self {
            display_device: [0; 64],
            emulation: 0,
        }
    }
}

/// `ColorManagedColorspaceSettings`.
#[derive(Clone)]
pub struct ColorManagedColorspaceSettings {
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `char interop_id[64]`.
    pub interop_id: [u8; 64],
}
impl Default for ColorManagedColorspaceSettings {
    fn default() -> Self {
        Self {
            name: [0; 64],
            interop_id: [0; 64],
        }
    }
}
