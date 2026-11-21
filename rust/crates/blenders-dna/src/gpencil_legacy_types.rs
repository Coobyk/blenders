//! Port of `DNA_gpencil_legacy_types.h` — legacy grease pencil (v2) ID.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::curve_types::BezTriple;
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;
/// `eGPDspoint_Flag`.
pub type GpdspointFlag = i32;
pub const GP_SPOINT_SELECT: GpdspointFlag = 1 << 0;
pub const GP_SPOINT_TAG: GpdspointFlag = 1 << 1;

/// `eGPDpalettecolor_Flag`.
pub type GpdpalettecolorFlag = i16;
pub const PC_COLOR_HIDE: GpdpalettecolorFlag = 1 << 1;
pub const PC_COLOR_LOCKED: GpdpalettecolorFlag = 1 << 2;
pub const PC_COLOR_ONIONSKIN: GpdpalettecolorFlag = 1 << 3;
pub const PC_COLOR_VOLUMETRIC: GpdpalettecolorFlag = 1 << 4;

/// `eGPDpalette_Flag`.
pub type GpdpaletteFlag = i16;
pub const PL_PALETTE_ACTIVE: GpdpaletteFlag = 1 << 0;

/// `eGPDcurve_point_Flag`.
pub type GpdcurvePointFlag = i32;
pub const GP_CURVE_POINT_SELECT: GpdcurvePointFlag = 1 << 0;

/// `bGPDcurve_Flag`.
pub type BgpdcurveFlag = i16;
pub const GP_CURVE_NEEDS_STROKE_UPDATE: BgpdcurveFlag = 1 << 0;
pub const GP_CURVE_SELECT: BgpdcurveFlag = 1 << 1;

/// `eGPDstroke_Flag`.
pub type GpdstrokeFlag = i16;
pub const GP_STROKE_3DSPACE: GpdstrokeFlag = 1 << 0;
pub const GP_STROKE_2DSPACE: GpdstrokeFlag = 1 << 1;
pub const GP_STROKE_2DIMAGE: GpdstrokeFlag = 1 << 2;
pub const GP_STROKE_SELECT: GpdstrokeFlag = 1 << 3;
pub const GP_STROKE_CYCLIC: GpdstrokeFlag = 1 << 7;
pub const GP_STROKE_NOFILL: GpdstrokeFlag = 1 << 8;
pub const GP_STROKE_NEEDS_CURVE_UPDATE: GpdstrokeFlag = 1 << 9;
pub const GP_STROKE_HELP: GpdstrokeFlag = 1 << 10;
pub const GP_STROKE_COLLIDE: GpdstrokeFlag = 1 << 11;
pub const GP_STROKE_USE_ARROW_START: GpdstrokeFlag = 1 << 12;
pub const GP_STROKE_USE_ARROW_END: GpdstrokeFlag = 1 << 13;
pub const GP_STROKE_TAG: GpdstrokeFlag = 1 << 14;
pub const GP_STROKE_ERASER: GpdstrokeFlag = 1 << 15;

/// `eGPDstroke_Caps`.
pub type GpdstrokeCaps = i16;
pub const GP_STROKE_CAP_ROUND: GpdstrokeCaps = 0;
pub const GP_STROKE_CAP_FLAT: GpdstrokeCaps = 1;
pub const GP_STROKE_CAP_MAX: GpdstrokeCaps = GP_STROKE_CAP_FLAT + 1;

/// `eGPDstroke_Arrowstyle`.
pub type GpdstrokeArrowstyle = i32;
pub const GP_STROKE_ARROWSTYLE_NONE: GpdstrokeArrowstyle = 0;
pub const GP_STROKE_ARROWSTYLE_SEGMENT: GpdstrokeArrowstyle = 2;
pub const GP_STROKE_ARROWSTYLE_OPEN: GpdstrokeArrowstyle = 3;
pub const GP_STROKE_ARROWSTYLE_CLOSED: GpdstrokeArrowstyle = 4;
pub const GP_STROKE_ARROWSTYLE_SQUARE: GpdstrokeArrowstyle = 6;

/// `eGPDframe_Flag`.
pub type GpdframeFlag = i16;
pub const GP_FRAME_PAINT: GpdframeFlag = 1 << 0;
pub const GP_FRAME_SELECT: GpdframeFlag = 1 << 1;
pub const GP_FRAME_LRT_CLEARED: GpdframeFlag = 1 << 2;

/// `ebGPDlayer_Mask_Flag`.
pub type BgpdlayerMaskFlag = i16;
pub const GP_MASK_HIDE: BgpdlayerMaskFlag = 1 << 0;
pub const GP_MASK_INVERT: BgpdlayerMaskFlag = 1 << 1;

/// `eGPDlayer_Flag`.
pub type GpdlayerFlag = i16;
pub const GP_LAYER_HIDE: GpdlayerFlag = 1 << 0;
pub const GP_LAYER_LOCKED: GpdlayerFlag = 1 << 1;
pub const GP_LAYER_ACTIVE: GpdlayerFlag = 1 << 2;
pub const GP_LAYER_DRAWDEBUG: GpdlayerFlag = 1 << 3;
pub const GP_LAYER_SOLO_MODE: GpdlayerFlag = 1 << 4;
pub const GP_LAYER_SELECT: GpdlayerFlag = 1 << 5;
pub const GP_LAYER_FRAMELOCK: GpdlayerFlag = 1 << 6;
pub const GP_LAYER_NO_XRAY: GpdlayerFlag = 1 << 7;
pub const GP_LAYER_VOLUMETRIC: GpdlayerFlag = 1 << 10;
pub const GP_LAYER_USE_LIGHTS: GpdlayerFlag = 1 << 11;
pub const GP_LAYER_UNLOCK_COLOR: GpdlayerFlag = 1 << 12;
pub const GP_LAYER_USE_MASK: GpdlayerFlag = 1 << 13;
pub const GP_LAYER_IS_RULER: GpdlayerFlag = 1 << 14;
pub const GP_LAYER_DISABLE_MASKS_IN_VIEWLAYER: GpdlayerFlag = 1 << 15;

/// `eGPDlayer_OnionFlag`.
pub type GpdlayerOnionflag = i16;
pub const GP_LAYER_ONIONSKIN: GpdlayerOnionflag = 1 << 0;
pub const GP_LAYER_ONIONSKIN_CUSTOM_COLOR: GpdlayerOnionflag = 1 << 1;

/// `eGPLayerBlendModes`.
pub type Gplayerblendmodes = i32;
pub const eGplBlendMode_Regular: Gplayerblendmodes = 0;
pub const eGplBlendMode_HardLight: Gplayerblendmodes = 1;
pub const eGplBlendMode_Add: Gplayerblendmodes = 2;
pub const eGplBlendMode_Subtract: Gplayerblendmodes = 3;
pub const eGplBlendMode_Multiply: Gplayerblendmodes = 4;
pub const eGplBlendMode_Divide: Gplayerblendmodes = 5;

/// `eGPdata_Flag`.
pub type GpdataFlag = i32;
pub const GP_DATA_ANNOTATIONS: GpdataFlag = 1 << 0;
pub const GP_DATA_DISPINFO: GpdataFlag = 1 << 1;
pub const GP_DATA_EXPAND: GpdataFlag = 1 << 2;
pub const GP_DATA_VIEWALIGN: GpdataFlag = 1 << 4;
pub const GP_DATA_DEPTH_VIEW: GpdataFlag = 1 << 5;
pub const GP_DATA_DEPTH_STROKE: GpdataFlag = 1 << 6;
pub const GP_DATA_DEPTH_STROKE_ENDPOINTS: GpdataFlag = 1 << 7;
pub const GP_DATA_STROKE_EDITMODE: GpdataFlag = 1 << 8;
pub const GP_DATA_SHOW_ONIONSKINS: GpdataFlag = 1 << 9;
pub const GP_DATA_CACHE_IS_DIRTY: GpdataFlag = 1 << 11;
pub const GP_DATA_STROKE_PAINTMODE: GpdataFlag = 1 << 12;
pub const GP_DATA_STROKE_SCULPTMODE: GpdataFlag = 1 << 13;
pub const GP_DATA_STROKE_WEIGHTMODE: GpdataFlag = 1 << 14;
pub const GP_DATA_STROKE_KEEPTHICKNESS: GpdataFlag = 1 << 15;
pub const GP_DATA_STROKE_MULTIEDIT: GpdataFlag = 1 << 16;
pub const GP_DATA_STROKE_VERTEXMODE: GpdataFlag = 1 << 18;
pub const GP_DATA_AUTOLOCK_LAYERS: GpdataFlag = 1 << 20;
pub const GP_DATA_CURVE_EDIT_MODE: GpdataFlag = 1 << 21;
pub const GP_DATA_CURVE_ADAPTIVE_RESOLUTION: GpdataFlag = 1 << 22;

/// `eGPD_OnionFlag`.
pub type GpdOnionflag = i32;
pub const GP_ONION_GHOST_PREVCOL: GpdOnionflag = 1 << 0;
pub const GP_ONION_GHOST_NEXTCOL: GpdOnionflag = 1 << 1;
pub const GP_ONION_GHOST_ALWAYS: GpdOnionflag = 1 << 2;
pub const GP_ONION_FADE: GpdOnionflag = 1 << 3;
pub const GP_ONION_LOOP: GpdOnionflag = 1 << 4;

/// `eGP_OnionModes`.
pub type GpOnionmodes = i32;
pub const GP_ONION_MODE_ABSOLUTE: GpOnionmodes = 0;
pub const GP_ONION_MODE_RELATIVE: GpOnionmodes = 1;
pub const GP_ONION_MODE_SELECTED: GpOnionmodes = 2;

/// `eGP_DrawMode`.
pub type GpDrawmode = i16;
pub const GP_DRAWMODE_2D: GpDrawmode = 0;
pub const GP_DRAWMODE_3D: GpDrawmode = 1;

/// `bGPDspoint`.
#[derive(Clone)]
pub struct bGPDspoint {
    /// `float x`
    pub x: f32,
    /// `float y`
    pub y: f32,
    /// `float z`
    pub z: f32,
    /// `float pressure`
    pub pressure: f32,
    /// `float strength`
    pub strength: f32,
    /// `float time`
    pub time: f32,
    /// `eGPDspoint_Flag flag`
    pub flag: GpdspointFlag,
    /// `float uv_fac`
    pub uv_fac: f32,
    /// `float uv_rot`
    pub uv_rot: f32,
    /// `float uv_fill[2]`
    pub uv_fill: [f32; 2],
    /// `float vert_color[4]`
    pub vert_color: [f32; 4],
}
impl Default for bGPDspoint {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            pressure: 0.0,
            strength: 0.0,
            time: 0.0,
            flag: Default::default(),
            uv_fac: 0.0,
            uv_rot: 0.0,
            uv_fill: [0.0; 2],
            vert_color: [0.0; 4],
        }
    }
}

/// `bGPDtriangle`.
#[derive(Clone)]
pub struct bGPDtriangle {
    /// `unsigned_int verts[3]`
    pub verts: [u32; 3],
}
impl Default for bGPDtriangle {
    fn default() -> Self {
        Self {
            verts: [0; 3],
        }
    }
}

/// `bGPDpalettecolor`.
#[derive(Clone)]
pub struct bGPDpalettecolor {
    /// `bGPDpalettecolor *next`
    pub next: usize,
    /// `bGPDpalettecolor *prev`
    pub prev: usize,
    /// `char info[64]`
    pub info: [u8; 64],
    /// `float color[4]`
    pub color: [f32; 4],
    /// `float fill[4]`
    pub fill: [f32; 4],
    /// `eGPDpalettecolor_Flag flag`
    pub flag: GpdpalettecolorFlag,
}
impl Default for bGPDpalettecolor {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            info: [0; 64],
            color: [0.0; 4],
            fill: [0.0; 4],
            flag: Default::default(),
        }
    }
}

/// `bGPDpalette`.
#[derive(Clone)]
pub struct bGPDpalette {
    /// `bGPDpalette *next`
    pub next: usize,
    /// `bGPDpalette *prev`
    pub prev: usize,
    /// `ListBaseT<bGPDpalettecolor>`
    pub colors: ListBase,
    /// `char info[64]`
    pub info: [u8; 64],
    /// `eGPDpalette_Flag flag`
    pub flag: GpdpaletteFlag,
}
impl Default for bGPDpalette {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            colors: ListBase::default(),
            info: [0; 64],
            flag: Default::default(),
        }
    }
}

/// `bGPDcurve_point`.
#[derive(Clone)]
pub struct bGPDcurve_point {
    pub bezt: BezTriple,
    /// `float pressure`
    pub pressure: f32,
    /// `float strength`
    pub strength: f32,
    /// `int point_index`
    pub point_index: i32,
    /// `eGPDcurve_point_Flag flag`
    pub flag: GpdcurvePointFlag,
    /// `float uv_fac`
    pub uv_fac: f32,
    /// `float uv_rot`
    pub uv_rot: f32,
    /// `float uv_fill[2]`
    pub uv_fill: [f32; 2],
    /// `float vert_color[4]`
    pub vert_color: [f32; 4],
}
impl Default for bGPDcurve_point {
    fn default() -> Self {
        Self {
            bezt: Default::default(),
            pressure: 0.0,
            strength: 0.0,
            point_index: 0,
            flag: Default::default(),
            uv_fac: 0.0,
            uv_rot: 0.0,
            uv_fill: [0.0; 2],
            vert_color: [0.0; 4],
        }
    }
}

/// `bGPDcurve`.
#[derive(Clone)]
pub struct bGPDcurve {
    /// `bGPDcurve_point *curve_points`
    pub curve_points: usize,
    /// `int tot_curve_points`
    pub tot_curve_points: i32,
    /// `bGPDcurve_Flag flag`
    pub flag: BgpdcurveFlag,
}
impl Default for bGPDcurve {
    fn default() -> Self {
        Self {
            curve_points: NIL,
            tot_curve_points: 0,
            flag: Default::default(),
        }
    }
}

/// `bGPDstroke_Runtime`.
#[derive(Clone)]
pub struct bGPDstroke_Runtime {
    /// `char tmp_layerinfo[128]`
    pub tmp_layerinfo: [u8; 128],
    /// `float multi_frame_falloff`
    pub multi_frame_falloff: f32,
    /// `int stroke_start`
    pub stroke_start: i32,
    /// `int fill_start`
    pub fill_start: i32,
    /// `int vertex_start`
    pub vertex_start: i32,
    /// `int curve_start`
    pub curve_start: i32,
    /// `bGPDstroke *gps_orig`
    pub gps_orig: usize,
    /// `void *_pad2`
    pub pad2: usize,
}
impl Default for bGPDstroke_Runtime {
    fn default() -> Self {
        Self {
            tmp_layerinfo: [0; 128],
            multi_frame_falloff: 0.0,
            stroke_start: 0,
            fill_start: 0,
            vertex_start: 0,
            curve_start: 0,
            gps_orig: NIL,
            pad2: NIL,
        }
    }
}

/// `bGPDstroke`.
#[derive(Clone)]
pub struct bGPDstroke {
    /// `bGPDstroke *next`
    pub next: usize,
    /// `bGPDstroke *prev`
    pub prev: usize,
    /// `bGPDspoint *points`
    pub points: usize,
    /// `bGPDtriangle *triangles`
    pub triangles: usize,
    /// `int totpoints`
    pub totpoints: i32,
    /// `int tot_triangles`
    pub tot_triangles: i32,
    /// `short thickness`
    pub thickness: i16,
    /// `GpdstrokeFlag flag`
    pub flag: GpdstrokeFlag,
    /// `double inittime`
    pub inittime: f64,
    /// `char colorname[128]`
    pub colorname: [u8; 128],
    /// `int mat_nr`
    pub mat_nr: i32,
    /// `eGPDstroke_Caps caps[2]`
    pub caps: [GpdstrokeCaps; 2],
    /// `float hardness`
    pub hardness: f32,
    /// `float aspect_ratio[2]`
    pub aspect_ratio: [f32; 2],
    /// `float fill_opacity_fac`
    pub fill_opacity_fac: f32,
    /// `float uv_rotation`
    pub uv_rotation: f32,
    /// `float uv_translation[2]`
    pub uv_translation: [f32; 2],
    /// `float uv_scale`
    pub uv_scale: f32,
    /// `int select_index`
    pub select_index: i32,
    /// `MDeformVert *dvert`
    pub dvert: usize,
    /// `void *_pad3`
    pub pad3: usize,
    /// `float vert_color_fill[4]`
    pub vert_color_fill: [f32; 4],
    /// `bGPDcurve *editcurve`
    pub editcurve: usize,
    pub runtime: bGPDstroke_Runtime,
    /// `void *_pad5`
    pub pad5: usize,
}
impl Default for bGPDstroke {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            points: NIL,
            triangles: NIL,
            totpoints: 0,
            tot_triangles: 0,
            thickness: 0,
            flag: Default::default(),
            inittime: 0.0,
            colorname: [0; 128],
            mat_nr: 0,
            caps: [0; 2],
            hardness: 0.0,
            aspect_ratio: [0.0; 2],
            fill_opacity_fac: 0.0,
            uv_rotation: 0.0,
            uv_translation: [0.0; 2],
            uv_scale: 0.0,
            select_index: 0,
            dvert: NIL,
            pad3: NIL,
            vert_color_fill: [0.0; 4],
            editcurve: NIL,
            runtime: Default::default(),
            pad5: NIL,
        }
    }
}

/// `bGPDframe_Runtime`.
#[derive(Clone)]
pub struct bGPDframe_Runtime {
    /// `int frameid`
    pub frameid: i32,
    /// `int onion_id`
    pub onion_id: i32,
}
impl Default for bGPDframe_Runtime {
    fn default() -> Self {
        Self {
            frameid: 0,
            onion_id: 0,
        }
    }
}

/// `bGPDframe`.
#[derive(Clone)]
pub struct bGPDframe {
    /// `bGPDframe *next`
    pub next: usize,
    /// `bGPDframe *prev`
    pub prev: usize,
    /// `ListBaseT<bGPDstroke>`
    pub strokes: ListBase,
    /// `int framenum`
    pub framenum: i32,
    /// `eGPDframe_Flag flag`
    pub flag: GpdframeFlag,
    /// `short key_type`
    pub key_type: i16,
    pub runtime: bGPDframe_Runtime,
}
impl Default for bGPDframe {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            strokes: ListBase::default(),
            framenum: 0,
            flag: Default::default(),
            key_type: 0,
            runtime: Default::default(),
        }
    }
}

/// `bGPDlayer_Mask`.
#[derive(Clone)]
pub struct bGPDlayer_Mask {
    /// `bGPDlayer_Mask *next`
    pub next: usize,
    /// `bGPDlayer_Mask *prev`
    pub prev: usize,
    /// `char name[128]`
    pub name: [u8; 128],
    /// `ebGPDlayer_Mask_Flag flag`
    pub flag: BgpdlayerMaskFlag,
    /// `short sort_index`
    pub sort_index: i16,
}
impl Default for bGPDlayer_Mask {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 128],
            flag: Default::default(),
            sort_index: 0,
        }
    }
}

/// `bGPDlayer_Runtime`.
#[derive(Clone)]
pub struct bGPDlayer_Runtime {
    /// `int icon_id`
    pub icon_id: i32,
}
impl Default for bGPDlayer_Runtime {
    fn default() -> Self {
        Self {
            icon_id: 0,
        }
    }
}

/// `bGPDlayer`.
#[derive(Clone)]
pub struct bGPDlayer {
    /// `bGPDlayer *next`
    pub next: usize,
    /// `bGPDlayer *prev`
    pub prev: usize,
    /// `ListBaseT<bGPDframe>`
    pub frames: ListBase,
    /// `bGPDframe *actframe`
    pub actframe: usize,
    /// `eGPDlayer_Flag flag`
    pub flag: GpdlayerFlag,
    /// `eGPDlayer_OnionFlag onion_flag`
    pub onion_flag: GpdlayerOnionflag,
    /// `float color[4]`
    pub color: [f32; 4],
    /// `float fill[4]`
    pub fill: [f32; 4],
    /// `char info[128]`
    pub info: [u8; 128],
    /// `short thickness`
    pub thickness: i16,
    /// `short pass_index`
    pub pass_index: i16,
    /// `Object *parent`
    pub parent: usize,
    /// `float inverse[4][4]`
    pub inverse: [[f32; 4]; 4],
    /// `char parsubstr[64]`
    pub parsubstr: [u8; 64],
    /// `short partype`
    pub partype: i16,
    /// `short line_change`
    pub line_change: i16,
    /// `float tintcolor[4]`
    pub tintcolor: [f32; 4],
    /// `float opacity`
    pub opacity: f32,
    /// `char viewlayername[64]`
    pub viewlayername: [u8; 64],
    /// `eGPLayerBlendModes blend_mode`
    pub blend_mode: Gplayerblendmodes,
    /// `float vertex_paint_opacity`
    pub vertex_paint_opacity: f32,
    /// `short gstep`
    pub gstep: i16,
    /// `short gstep_next`
    pub gstep_next: i16,
    /// `float gcolor_prev[3]`
    pub gcolor_prev: [f32; 3],
    /// `float gcolor_next[3]`
    pub gcolor_next: [f32; 3],
    /// `ListBaseT<bGPDlayer_Mask>`
    pub mask_layers: ListBase,
    /// `int act_mask`
    pub act_mask: i32,
    /// `float location[3]`
    pub location: [f32; 3],
    /// `float rotation[3]`
    pub rotation: [f32; 3],
    /// `float scale[3]`
    pub scale: [f32; 3],
    /// `float layer_mat[4][4]`
    pub layer_mat: [[f32; 4]; 4],
    /// `float layer_invmat[4][4]`
    pub layer_invmat: [[f32; 4]; 4],
    pub runtime: bGPDlayer_Runtime,
}
impl Default for bGPDlayer {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            frames: ListBase::default(),
            actframe: NIL,
            flag: Default::default(),
            onion_flag: Default::default(),
            color: [0.0; 4],
            fill: [0.0; 4],
            info: [0; 128],
            thickness: 0,
            pass_index: 0,
            parent: NIL,
            inverse: [[0.0; 4]; 4],
            parsubstr: [0; 64],
            partype: 0,
            line_change: 0,
            tintcolor: [0.0; 4],
            opacity: 0.0,
            viewlayername: [0; 64],
            blend_mode: eGplBlendMode_Regular,
            vertex_paint_opacity: 0.0,
            gstep: 0,
            gstep_next: 0,
            gcolor_prev: [0.0; 3],
            gcolor_next: [0.0; 3],
            mask_layers: ListBase::default(),
            act_mask: 0,
            location: [0.0; 3],
            rotation: [0.0; 3],
            scale: [0.0; 3],
            layer_mat: [[0.0; 4]; 4],
            layer_invmat: [[0.0; 4]; 4],
            runtime: Default::default(),
        }
    }
}

/// `bGPdata_Runtime`.
#[derive(Clone)]
pub struct bGPdata_Runtime {
    /// `tGPspoint *sbuffer`
    pub sbuffer: usize,
    /// `short playing`
    pub playing: i16,
    /// `short matid`
    pub matid: i16,
    /// `GpdstrokeFlag sbuffer_sflag`
    pub sbuffer_sflag: GpdstrokeFlag,
    /// `int sbuffer_used`
    pub sbuffer_used: i32,
    /// `int sbuffer_size`
    pub sbuffer_size: i32,
    /// `float vert_color_fill[4]`
    pub vert_color_fill: [f32; 4],
    /// `float fill_opacity_fac`
    pub fill_opacity_fac: f32,
    /// `float arrow_start[8]`
    pub arrow_start: [f32; 8],
    /// `float arrow_end[8]`
    pub arrow_end: [f32; 8],
    /// `eGPDstroke_Arrowstyle arrow_start_style`
    pub arrow_start_style: GpdstrokeArrowstyle,
    /// `eGPDstroke_Arrowstyle arrow_end_style`
    pub arrow_end_style: GpdstrokeArrowstyle,
}
impl Default for bGPdata_Runtime {
    fn default() -> Self {
        Self {
            sbuffer: NIL,
            playing: 0,
            matid: 0,
            sbuffer_sflag: 0,
            sbuffer_used: 0,
            sbuffer_size: 0,
            vert_color_fill: [0.0; 4],
            fill_opacity_fac: 0.0,
            arrow_start: [0.0; 8],
            arrow_end: [0.0; 8],
            arrow_start_style: GP_STROKE_ARROWSTYLE_NONE,
            arrow_end_style: GP_STROKE_ARROWSTYLE_NONE,
        }
    }
}

/// `bGPgrid`.
#[derive(Clone)]
pub struct bGPgrid {
    /// `float color[3]`
    pub color: [f32; 3],
    /// `float scale[2]`
    pub scale: [f32; 2],
    /// `float offset[2]`
    pub offset: [f32; 2],
    /// `int lines`
    pub lines: i32,
}
impl Default for bGPgrid {
    fn default() -> Self {
        Self {
            color: [0.0; 3],
            scale: [0.0; 2],
            offset: [0.0; 2],
            lines: 0,
        }
    }
}

/// `bGPdata`.
#[derive(Clone)]
pub struct bGPdata {
    pub id: ID,
    /// `AnimData *adt`
    pub adt: usize,
    /// `ListBaseT<bGPDlayer>`
    pub layers: ListBase,
    /// `eGPdata_Flag flag`
    pub flag: GpdataFlag,
    /// `int curve_edit_resolution`
    pub curve_edit_resolution: i32,
    /// `float curve_edit_threshold`
    pub curve_edit_threshold: f32,
    /// `float curve_edit_corner_angle`
    pub curve_edit_corner_angle: f32,
    /// `ListBaseT<bGPDpalette>`
    pub palettes: ListBase,
    /// `ListBaseT<bDeformGroup>`
    pub vertex_group_names: ListBase,
    /// `float pixfactor`
    pub pixfactor: f32,
    /// `float line_color[4]`
    pub line_color: [f32; 4],
    /// `float onion_factor`
    pub onion_factor: f32,
    /// `eGP_OnionModes onion_mode`
    pub onion_mode: GpOnionmodes,
    /// `eGPD_OnionFlag onion_flag`
    pub onion_flag: GpdOnionflag,
    /// `short gstep`
    pub gstep: i16,
    /// `short gstep_next`
    pub gstep_next: i16,
    /// `float gcolor_prev[3]`
    pub gcolor_prev: [f32; 3],
    /// `float gcolor_next[3]`
    pub gcolor_next: [f32; 3],
    /// `float zdepth_offset`
    pub zdepth_offset: f32,
    /// `Material *mat`
    pub mat: usize,
    /// `short totcol`
    pub totcol: i16,
    /// `short totlayer`
    pub totlayer: i16,
    /// `short totframe`
    pub totframe: i16,
    /// `int totstroke`
    pub totstroke: i32,
    /// `int totpoint`
    pub totpoint: i32,
    /// `eGP_DrawMode draw_mode`
    pub draw_mode: GpDrawmode,
    /// `short onion_keytype`
    pub onion_keytype: i16,
    /// `int select_last_index`
    pub select_last_index: i32,
    /// `int vertex_group_active_index`
    pub vertex_group_active_index: i32,
    pub grid: bGPgrid,
    pub runtime: bGPdata_Runtime,
}
impl Default for bGPdata {
    fn default() -> Self {
        Self {
            id: Default::default(),
            adt: NIL,
            layers: ListBase::default(),
            flag: Default::default(),
            curve_edit_resolution: 0,
            curve_edit_threshold: 0.0,
            curve_edit_corner_angle: 0.0,
            palettes: ListBase::default(),
            vertex_group_names: ListBase::default(),
            pixfactor: 0.0,
            line_color: [0.0; 4],
            onion_factor: 0.0,
            onion_mode: GP_ONION_MODE_ABSOLUTE,
            onion_flag: Default::default(),
            gstep: 0,
            gstep_next: 0,
            gcolor_prev: [0.0; 3],
            gcolor_next: [0.0; 3],
            zdepth_offset: 0.0,
            mat: NIL,
            totcol: 0,
            totlayer: 0,
            totframe: 0,
            totstroke: 0,
            totpoint: 0,
            draw_mode: GP_DRAWMODE_2D,
            onion_keytype: 0,
            select_last_index: 0,
            vertex_group_active_index: 0,
            grid: Default::default(),
            runtime: Default::default(),
        }
    }
}
