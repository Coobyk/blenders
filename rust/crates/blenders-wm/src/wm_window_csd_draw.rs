//! Port of `intern/wm_window_csd_draw.cc` — CSD titlebar
//! draw layout (Y-flip, centered title, button circles +
//! icons) and rounded-corner mask radii.

use crate::wm_subwindow::Rcti;
use crate::wm_window_csd_layout::{
    wm_window_csd_fractional_scale_apply, wm_window_csd_layout_callback, GhostCsdElem,
    GhostCsdLayout, GHOST_K_CSD_TYPE_BODY, GHOST_K_CSD_TYPE_NUM, GHOST_K_CSD_TYPE_TITLEBAR,
};

/// `GHOST_kCSDTypeButton*` types (verbatim ordering).
pub const GHOST_K_CSD_TYPE_BUTTON_CLOSE: i32 = 3;
pub const GHOST_K_CSD_TYPE_BUTTON_MAXIMIZE: i32 = 4;
pub const GHOST_K_CSD_TYPE_BUTTON_MINIMIZE: i32 = 5;
pub const GHOST_K_CSD_TYPE_BUTTON_MENU: i32 = 6;
/// `WM_WINDOW_CSD_CORNER_RADIUS` verbatim.
pub const WM_WINDOW_CSD_CORNER_RADIUS: i32 = 6;
/// `ICON_DEFAULT_HEIGHT` verbatim.
pub const ICON_DEFAULT_HEIGHT: i32 = 16;
/// `GHOST_kWindowStateMaximized`.
pub const GHOST_K_WINDOW_STATE_MAXIMIZED: i32 = 3;
/// `GHOST_kWindowTiled*` flags.
pub const GHOST_K_WINDOW_TILED_LEFT: i32 = 1 << 0;
pub const GHOST_K_WINDOW_TILED_RIGHT: i32 = 1 << 1;
pub const GHOST_K_WINDOW_TILED_TOP: i32 = 1 << 2;
pub const GHOST_K_WINDOW_TILED_BOTTOM: i32 = 1 << 3;
/// `GHOST_kWindowStateNormal`.
pub const GHOST_K_WINDOW_STATE_NORMAL: i32 = 0;

/// `BLI_rcti_cent_x`/`cent_y`.
pub fn rcti_cent_x(r: &Rcti) -> i32 {
    (r.xmin + r.xmax) / 2
}
/// cent_y.
pub fn rcti_cent_y(r: &Rcti) -> i32 {
    (r.ymin + r.ymax) / 2
}

/// `CsdDrawLayout` — Y-flipped type-indexed elems +
/// title rect for the draw to consume.
pub struct CsdDrawLayout {
    /// `csd_elems` (type-indexed, Y-flipped bounds).
    pub elems: [GhostCsdElem; GHOST_K_CSD_TYPE_NUM],
    /// `title_rect`.
    pub title_rect: Rcti,
    /// `rect_size_y`.
    pub rect_size_y: i32,
}

/// `WM_window_csd_draw_titlebar_ex` layout half (verbatim
/// — layout callback, flip Y into type-indexed array,
/// title rect).
pub fn wm_window_csd_draw_titlebar_layout(
    win_size: [i32; 2],
    fractional_scale: [i32; 2],
    win_state: i32,
    csd_layout: Option<&GhostCsdLayout>,
) -> Option<CsdDrawLayout> {
    let mut csd_elems_orig = [GhostCsdElem::default(); GHOST_K_CSD_TYPE_NUM];
    let csd_elems_num = wm_window_csd_layout_callback(
        win_size,
        fractional_scale,
        win_state,
        csd_layout,
        &mut csd_elems_orig,
    );
    if csd_elems_num <= 0 {
        return None;
    }
    let mut csd_elems = [GhostCsdElem {
        elem_type: GHOST_K_CSD_TYPE_BODY,
        bounds: [[0, 0], [0, 0]],
    }; GHOST_K_CSD_TYPE_NUM];
    for i in 0..csd_elems_num as usize {
        let slot = csd_elems_orig[i].elem_type as usize;
        let mut elem = csd_elems_orig[i];
        elem.bounds[1][0] = win_size[1] - elem.bounds[1][0];
        elem.bounds[1][1] = win_size[1] - elem.bounds[1][1];
        elem.bounds[1].swap(0, 1);
        csd_elems[slot] = elem;
    }
    assert!(csd_elems[GHOST_K_CSD_TYPE_TITLEBAR as usize].elem_type == GHOST_K_CSD_TYPE_TITLEBAR);
    let tb = csd_elems[GHOST_K_CSD_TYPE_TITLEBAR as usize];
    let title_rect = Rcti {
        xmin: tb.bounds[0][0],
        xmax: tb.bounds[0][1],
        ymin: tb.bounds[1][0],
        ymax: tb.bounds[1][1],
    };
    let rect_size_y = crate::wm_subwindow::rcti_size_y(&title_rect);
    Some(CsdDrawLayout {
        elems: csd_elems,
        title_rect,
        rect_size_y,
    })
}

/// Title text placement (verbatim — active/dimmed color,
/// centered x, descender+offset y).
pub fn csd_title_text_pos(
    title_rect: &Rcti,
    rect_size_y: i32,
    title_width: i32,
    title_decender: i32,
    title_height_max: i32,
) -> (f32, f32) {
    let px_offset = -crate::wm_subwindow::GLA_PIXEL_OFS;
    let offset_y = if rect_size_y > title_height_max {
        (rect_size_y - title_height_max) / 2
    } else {
        0
    };
    (
        (title_rect.xmin + (rcti_cent_x(title_rect) - (title_width / 2))) as f32 + px_offset,
        (title_decender + offset_y) as f32 + px_offset,
    )
}

/// `csd_title_color` (verbatim — inactive halves color).
pub fn csd_title_color(
    is_active: bool,
    border_color: Option<[u8; 3]>,
    text_color: [u8; 3],
    alpha: f32,
) -> [u8; 4] {
    if !is_active {
        if let Some(bc) = border_color {
            [
                ((bc[0] as i32 + text_color[0] as i32) / 2) as u8,
                ((bc[1] as i32 + text_color[1] as i32) / 2) as u8,
                ((bc[2] as i32 + text_color[2] as i32) / 2) as u8,
                (255.0 * alpha) as u8,
            ]
        } else {
            [text_color[0] / 2, text_color[1] / 2, text_color[2] / 2, (255.0 * alpha) as u8]
        }
    } else {
        [text_color[0], text_color[1], text_color[2], (255.0 * alpha) as u8]
    }
}

/// Button circle rect + icon pos (verbatim — bounds minus
/// title offset, radius, center, icon top-left).
pub fn csd_button_rect(
    elems: &[GhostCsdElem; GHOST_K_CSD_TYPE_NUM],
    ty: i32,
    title_rect: &Rcti,
) -> Option<Rcti> {
    let e = &elems[ty as usize];
    if e.elem_type == GHOST_K_CSD_TYPE_BODY {
        return None;
    }
    Some(Rcti {
        xmin: e.bounds[0][0] - title_rect.xmin,
        xmax: e.bounds[0][1] - title_rect.xmin,
        ymin: e.bounds[1][0] - title_rect.ymin,
        ymax: e.bounds[1][1] - title_rect.ymin,
    })
}

/// `csd_button_circle` params (verbatim).
pub fn csd_button_circle(butrect: &Rcti, button_margin: i32) -> (i32, i32, i32) {
    let but_radius = (crate::wm_subwindow::rcti_size_x(butrect) / 2) - button_margin;
    (rcti_cent_x(butrect), rcti_cent_y(butrect), but_radius)
}

/// Button types + icons order (verbatim — maximize icon
/// swaps when maximized).
pub fn csd_button_icons(win_state: i32) -> ([i32; 4], [i32; 4]) {
    const ICON_X: i32 = 32;
    const ICON_AREA_DOCK: i32 = 202;
    const ICON_CHECKBOX_DEHLT: i32 = 197;
    const ICON_DOT: i32 = 66;
    const ICON_BLENDER: i32 = 35;
    (
        [
            GHOST_K_CSD_TYPE_BUTTON_CLOSE,
            GHOST_K_CSD_TYPE_BUTTON_MAXIMIZE,
            GHOST_K_CSD_TYPE_BUTTON_MINIMIZE,
            GHOST_K_CSD_TYPE_BUTTON_MENU,
        ],
        [
            ICON_X,
            if win_state == GHOST_K_WINDOW_STATE_MAXIMIZED {
                ICON_AREA_DOCK
            } else {
                ICON_CHECKBOX_DEHLT
            },
            ICON_DOT,
            ICON_BLENDER,
        ],
    )
}

/// `WM_window_csd_draw_corner_mask` radii (verbatim —
/// tiled edges zero out their corners; state-normal gate).
pub fn wm_window_csd_corner_radii(
    window_state: i32,
    fractional_scale: [i32; 2],
    tiled_edges: i32,
) -> Option<[f32; 4]> {
    let radius = if window_state == GHOST_K_WINDOW_STATE_NORMAL {
        wm_window_csd_fractional_scale_apply(WM_WINDOW_CSD_CORNER_RADIUS, fractional_scale)
    } else {
        0
    };
    if radius <= 0 {
        return None;
    }
    let corner_tiled_edges = [
        GHOST_K_WINDOW_TILED_RIGHT | GHOST_K_WINDOW_TILED_TOP,
        GHOST_K_WINDOW_TILED_LEFT | GHOST_K_WINDOW_TILED_TOP,
        GHOST_K_WINDOW_TILED_LEFT | GHOST_K_WINDOW_TILED_BOTTOM,
        GHOST_K_WINDOW_TILED_RIGHT | GHOST_K_WINDOW_TILED_BOTTOM,
    ];
    let mut radii = [0.0f32; 4];
    for corner in 0..4 {
        radii[corner] = if (tiled_edges & corner_tiled_edges[corner]) != 0 {
            0.0
        } else {
            radius as f32
        };
    }
    if radii.iter().all(|&v| v == 0.0) {
        return None;
    }
    Some(radii)
}
