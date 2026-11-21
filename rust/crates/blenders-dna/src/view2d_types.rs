//! Port of `DNA_view2d_types.h` — `View2D`.
#![allow(non_camel_case_types)]
use crate::id::NIL;
use crate::vec_types::{Rctf, Rcti};

/// `eView2D_KeepZoom`.
pub const V2D_LIMITZOOM: i16 = 1 << 0;
pub const V2D_KEEPASPECT: i16 = 1 << 1;
pub const V2D_KEEPZOOM: i16 = 1 << 2;
pub const V2D_LOCKZOOM_X: i16 = 1 << 8;
pub const V2D_LOCKZOOM_Y: i16 = 1 << 9;

/// `eView2D_KeepOfs`.
pub const V2D_LOCKOFS_X: i16 = 1 << 1;
pub const V2D_LOCKOFS_Y: i16 = 1 << 2;
pub const V2D_KEEPOFS_X: i16 = 1 << 3;
pub const V2D_KEEPOFS_Y: i16 = 1 << 4;

/// `eView2D_KeepTot`.
pub const V2D_KEEPTOT_FREE: i16 = 0;
pub const V2D_KEEPTOT_BOUNDS: i16 = 1;
pub const V2D_KEEPTOT_STRICT: i16 = 2;

/// `eView2D_Flag`.
pub const V2D_VIEWSYNC_SCREEN_TIME: i16 = 1 << 0;
pub const V2D_VIEWSYNC_AREA_VERTICAL: i16 = 1 << 1;
pub const V2D_PIXELOFS_X: i16 = 1 << 2;
pub const V2D_PIXELOFS_Y: i16 = 1 << 3;
pub const V2D_IS_NAVIGATING: i16 = 1 << 9;
pub const V2D_IS_INIT: i16 = 1 << 10;
pub const V2D_SNAP_TO_PAGESIZE_Y: i16 = 1 << 11;
pub const V2D_ZOOM_IGNORE_KEEPOFS: i16 = 1 << 12;

/// `eView2D_Scroll`.
pub const V2D_SCROLL_LEFT: i16 = 1 << 0;
pub const V2D_SCROLL_RIGHT: i16 = 1 << 1;
pub const V2D_SCROLL_VERTICAL: i16 = V2D_SCROLL_LEFT | V2D_SCROLL_RIGHT;
pub const V2D_SCROLL_TOP: i16 = 1 << 2;
pub const V2D_SCROLL_BOTTOM: i16 = 1 << 3;
pub const V2D_SCROLL_HORIZONTAL: i16 = V2D_SCROLL_TOP | V2D_SCROLL_BOTTOM;
pub const V2D_SCROLL_VERTICAL_HANDLES: i16 = 1 << 5;
pub const V2D_SCROLL_HORIZONTAL_HANDLES: i16 = 1 << 6;
pub const V2D_SCROLL_VERTICAL_HIDE: i16 = 1 << 7;
pub const V2D_SCROLL_HORIZONTAL_HIDE: i16 = 1 << 8;
pub const V2D_SCROLL_VERTICAL_FULLR: i16 = 1 << 9;
pub const V2D_SCROLL_HORIZONTAL_FULLR: i16 = 1 << 10;

/// `eView2D_ScrollUI`.
pub const V2D_SCROLL_H_ACTIVE: i16 = 1 << 0;
pub const V2D_SCROLL_V_ACTIVE: i16 = 1 << 1;

/// `eView2D_Align`.
pub const V2D_ALIGN_FREE: i16 = 0;
pub const V2D_ALIGN_NO_POS_X: i16 = 1 << 0;
pub const V2D_ALIGN_NO_NEG_X: i16 = 1 << 1;
pub const V2D_ALIGN_NO_POS_Y: i16 = 1 << 2;
pub const V2D_ALIGN_NO_NEG_Y: i16 = 1 << 3;

/// `View2D`.
#[derive(Clone)]
pub struct View2D {
    pub tot: Rctf,
    pub cur: Rctf,
    pub vert: Rcti,
    pub hor: Rcti,
    pub mask: Rcti,
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub minzoom: f32,
    pub maxzoom: f32,
    pub scroll: i16,
    pub scroll_ui: i16,
    pub keeptot: i16,
    pub keepzoom: i16,
    pub keepofs: i16,
    pub flag: i16,
    pub align: i16,
    pub winx: i16,
    pub winy: i16,
    pub oldwinx: i16,
    pub oldwiny: i16,
    pub around: i16,
    pub alpha_vert: i8,
    pub alpha_hor: i8,
    pub page_size_y: f32,
    /// `ui::SmoothView2DStore *sms`.
    pub sms: usize,
    /// `wmTimer *smooth_timer`.
    pub smooth_timer: usize,
}
impl Default for View2D {
    fn default() -> Self {
        Self {
            tot: Rctf::default(),
            cur: Rctf::default(),
            vert: Rcti::default(),
            hor: Rcti::default(),
            mask: Rcti::default(),
            min: [0.0; 2],
            max: [0.0; 2],
            minzoom: 0.0,
            maxzoom: 0.0,
            scroll: 0,
            scroll_ui: 0,
            keeptot: 0,
            keepzoom: 0,
            keepofs: 0,
            flag: 0,
            align: 0,
            winx: 0,
            winy: 0,
            oldwinx: 0,
            oldwiny: 0,
            around: 0,
            alpha_vert: 0,
            alpha_hor: 0,
            page_size_y: 0.0,
            sms: NIL,
            smooth_timer: NIL,
        }
    }
}
