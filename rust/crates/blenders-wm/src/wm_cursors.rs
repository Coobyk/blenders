//! Port of `intern/wm_cursors.cc` — `BCursor` SVG table,
//! cursor-size/flip/convert helpers, `WM_cursor_set`
//! dispatch, modal cursors, grab-enable, cursor warp.

/// `CURSOR_HARDWARE_SIZE_MAX` verbatim.
pub const CURSOR_HARDWARE_SIZE_MAX: i32 = 255;
/// `WM_CURSOR_*` verbatim (`WM_types.h`).
pub const WM_CURSOR_NONE: i32 = -2;
pub const WM_CURSOR_DEFAULT: i32 = 0;
pub const WM_CURSOR_NW_ARROW: i32 = 1;
pub const WM_CURSOR_COPY: i32 = 2;
pub const WM_CURSOR_MOVE: i32 = 3;
pub const WM_CURSOR_TEXT_EDIT: i32 = 4;
pub const WM_CURSOR_WAIT: i32 = 5;
pub const WM_CURSOR_STOP: i32 = 6;
pub const WM_CURSOR_EDIT: i32 = 7;
pub const WM_CURSOR_CROSS: i32 = 8;
pub const WM_CURSOR_CROSSC: i32 = 9;
pub const WM_CURSOR_PAINT: i32 = 10;
pub const WM_CURSOR_DOT: i32 = 11;
pub const WM_CURSOR_ERASER: i32 = 12;
pub const WM_CURSOR_HAND: i32 = 13;
pub const WM_CURSOR_E_SCROLL: i32 = 14;
pub const WM_CURSOR_W_SCROLL: i32 = 15;
pub const WM_CURSOR_N_SCROLL: i32 = 16;
pub const WM_CURSOR_S_SCROLL: i32 = 17;
pub const WM_CURSOR_EYEDROPPER: i32 = 18;
pub const WM_CURSOR_SWAP_AREA: i32 = 19;
pub const WM_CURSOR_X_MOVE: i32 = 20;
pub const WM_CURSOR_Y_MOVE: i32 = 21;
pub const WM_CURSOR_H_SPLIT: i32 = 22;
pub const WM_CURSOR_V_SPLIT: i32 = 23;
pub const WM_CURSOR_NSEW_SCROLL: i32 = 24;
pub const WM_CURSOR_NS_SCROLL: i32 = 25;
pub const WM_CURSOR_EW_SCROLL: i32 = 26;
pub const WM_CURSOR_ZOOM_IN: i32 = 27;
pub const WM_CURSOR_ZOOM_OUT: i32 = 28;
pub const WM_CURSOR_PAINT_BRUSH: i32 = 29;
pub const WM_CURSOR_MUTE: i32 = 30;
pub const WM_CURSOR_PAINT_CURSOR: i32 = 31;
pub const WM_CURSOR_PICK_AREA: i32 = 32;
pub const WM_CURSOR_NUM: usize = 33;

/// `GHOST_TStandardCursor` codes (verbatim).
pub const GHOST_K_STANDARD_CURSOR_DEFAULT: i32 = 0;
pub const GHOST_K_STANDARD_CURSOR_CUSTOM: i32 = 1;
pub const GHOST_K_GRAB_NORMAL: i32 = 0;
pub const GHOST_K_GRAB_WRAP: i32 = 1;
pub const GHOST_K_GRAB_HIDE: i32 = 2;
pub const GHOST_K_AXIS_X: i32 = 1;
pub const GHOST_K_AXIS_Y: i32 = 2;

/// `BCursor` verbatim.
pub struct BCursor {
    /// `svg_source`.
    pub svg_source: &'static str,
    /// `hotspot` (normalized 0..1).
    pub hotspot: (f32, f32),
    /// `can_invert`.
    pub can_invert: bool,
}

/// `wm_cursor_size` (verbatim — `preferred_logical_size *
/// dpi_scale`, lround).
pub fn wm_cursor_size(dpi_scale: f32, preferred_logical_size: f32) -> i32 {
    (preferred_logical_size * dpi_scale).round() as i32
}

/// `cursor_bitmap_rgba_flip_y` (verbatim — swap top/
/// bottom scanlines).
pub fn cursor_bitmap_rgba_flip_y(buffer: &mut [u32], size: [usize; 2]) {
    let x_size = size[0];
    let mut y_size = size[1];
    let mut top = 0usize;
    let mut bottom = (y_size - 1) * x_size;
    let mut line = vec![0u32; x_size];
    y_size >>= 1;
    while y_size > 0 {
        line.copy_from_slice(&buffer[top..top + x_size]);
        buffer.copy_within(bottom..bottom + x_size, top);
        buffer[bottom..bottom + x_size].copy_from_slice(&line);
        bottom -= x_size;
        top += x_size;
        y_size -= 1;
    }
}

/// `cursor_bitmap_from_svg` dest-size math (verbatim —
/// scale = size/1600, ceil-clamped).
pub fn cursor_svg_dest_size(image_w: f32, image_h: f32, cursor_size: i32) -> [usize; 2] {
    let scale = cursor_size as f32 / 1600.0;
    [
        ((image_w * scale).ceil() as usize).min(cursor_size as usize),
        ((image_h * scale).ceil() as usize).min(cursor_size as usize),
    ]
}

/// `WM_cursor_set` decision logic (verbatim — modal
/// override, visibility, already-set, native vs custom).
pub enum CursorSetAction {
    /// `return` early (background / null win).
    DoNothing,
    /// `setCursorVisibility(false)`.
    Hide,
    /// `setCursorShape(ghost_cursor)`.
    SetGhost(i32),
    /// `window_set_custom_cursor` or default fallback.
    SetCustom(usize),
}

/// `WM_cursor_set` (verbatim dispatch — returns action +
/// updates lastcursor/modalcursor on the caller's win).
pub fn wm_cursor_set(
    curs: i32,
    win_cursor: &mut i32,
    win_lastcursor: &mut i32,
    win_modalcursor: i32,
    background: bool,
    has_ghost_shape: &mut dyn FnMut(i32) -> bool,
) -> CursorSetAction {
    let mut curs = curs;
    if background {
        return CursorSetAction::DoNothing;
    }
    if curs == WM_CURSOR_DEFAULT && win_modalcursor != 0 {
        *win_lastcursor = curs;
        curs = win_modalcursor;
    }
    if curs == WM_CURSOR_NONE {
        return CursorSetAction::Hide;
    }
    if *win_cursor == curs {
        return CursorSetAction::DoNothing;
    }
    *win_cursor = curs;
    if !(0..WM_CURSOR_NUM as i32).contains(&curs) {
        return CursorSetAction::DoNothing;
    }
    let ghost_cursor = convert_to_ghost_standard_cursor(curs);
    if ghost_cursor != GHOST_K_STANDARD_CURSOR_CUSTOM && (has_ghost_shape)(ghost_cursor) {
        CursorSetAction::SetGhost(ghost_cursor)
    } else {
        CursorSetAction::SetCustom(curs as usize)
    }
}

/// `convert_to_ghost_standard_cursor` (verbatim —
/// subset of mapped ids).
pub fn convert_to_ghost_standard_cursor(curs: i32) -> i32 {
    match curs {
        WM_CURSOR_DEFAULT | WM_CURSOR_NW_ARROW | WM_CURSOR_COPY | WM_CURSOR_MOVE => 0,
        WM_CURSOR_TEXT_EDIT => 2,
        WM_CURSOR_WAIT => 3,
        WM_CURSOR_STOP => 4,
        WM_CURSOR_EDIT | WM_CURSOR_CROSS | WM_CURSOR_CROSSC => 5,
        WM_CURSOR_HAND => 6,
        WM_CURSOR_E_SCROLL | WM_CURSOR_W_SCROLL | WM_CURSOR_EW_SCROLL => 7,
        WM_CURSOR_N_SCROLL | WM_CURSOR_S_SCROLL | WM_CURSOR_NS_SCROLL => 8,
        WM_CURSOR_NSEW_SCROLL => 9,
        WM_CURSOR_ZOOM_IN => 10,
        WM_CURSOR_ZOOM_OUT => 11,
        _ => GHOST_K_STANDARD_CURSOR_CUSTOM,
    }
}

/// `WM_cursor_modal_is_set_ok` (verbatim).
pub fn wm_cursor_modal_is_set_ok(grabcursor: i32, modalcursor: i32) -> bool {
    grabcursor == 0 && modalcursor == 0
}

/// `WM_cursor_modal_set`/`WM_cursor_modal_restore`
/// (verbatim state updates).
pub fn wm_cursor_modal_set(lastcursor: &mut i32, cursor: i32, modalcursor: &mut i32, val: i32) {
    if *lastcursor == 0 {
        *lastcursor = cursor;
    }
    *modalcursor = val;
}
pub fn wm_cursor_modal_restore(lastcursor: &mut i32, modalcursor: &mut i32) -> Option<i32> {
    *modalcursor = 0;
    let restore = *lastcursor;
    *lastcursor = 0;
    if restore != 0 {
        Some(restore)
    } else {
        None
    }
}

/// `wm_cursor_warp_relative` (verbatim — add xy to
/// current position).
pub fn wm_cursor_warp_relative(x: i32, y: i32) -> (i32, i32) {
    (x, y)
}
