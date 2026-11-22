//! Port of `intern/wm_window_ime_draw.cc` — IME composite
//! preview box layout (mono font, whole-pixel snap, cursor
//! clamp, selection underline); GPU/BLF draws injected.

/// `IMEPreviewLayout` — computed box + underline + selection
/// columns for the caller to draw.
pub struct IMEPreviewLayout {
    /// box rect (xmin, xmax, ymin, ymax).
    pub box_xmin: i32,
    pub box_xmax: i32,
    pub box_ymin: i32,
    pub box_ymax: i32,
    /// `baseline_y`.
    pub baseline_y: i32,
    /// `underline_y`.
    pub underline_y: i32,
    /// `border`.
    pub border: i32,
    /// `char_width`.
    pub char_width: i32,
    /// selection underline columns (start, end) or none.
    pub sel_cols: Option<(i32, i32)>,
}

/// `wm_window_IME_region_preview_draw` layout half
/// (verbatim math; draws injected).
#[allow(clippy::too_many_arguments)]
pub fn wm_window_ime_region_preview_layout(
    region_winx: i32,
    region_winy: i32,
    composite: &str,
    cursor_rect: (i32, i32, i32, i32),
    font_size: i32,
    blf_size: &mut dyn FnMut(i32),
    blf_fixed_width: &dyn Fn() -> i32,
    blf_descender: &dyn Fn() -> i32,
    utf8_offset_to_column: &dyn Fn(&str, i64) -> i32,
    pixelsize: f32,
    select_range: &dyn Fn(&str) -> Option<(i64, i64)>,
) -> IMEPreviewLayout {
    let (cxmin, _cxmax, cymin, cymax) = cursor_rect;
    let str_len = composite.len() as i64;
    let line_height = cymax - cymin;
    (blf_size)(font_size);
    let char_width = (blf_fixed_width)().max(1);
    let column_from_byte_offset = |byte_offset: i64| {
        (utf8_offset_to_column)(composite, byte_offset)
    };
    let columns = column_from_byte_offset(str_len);
    let border = (pixelsize as i32).max(1);
    let box_width = columns * char_width;
    let box_x = cxmin.clamp(0, (region_winx - box_width).max(0));
    let box_y = cymin.clamp(0, (region_winy - line_height).max(0));
    let baseline_y = box_y - (blf_descender)();
    let underline_y = box_y;
    let sel_cols = (select_range)(composite).and_then(|(s, e)| {
        let cs = column_from_byte_offset(s);
        let ce = column_from_byte_offset(e);
        let start = cs.clamp(0, columns);
        let end = ce.clamp(0, columns);
        if end > start {
            Some((start, end))
        } else {
            None
        }
    });
    IMEPreviewLayout {
        box_xmin: box_x,
        box_xmax: box_x + box_width,
        box_ymin: box_y,
        box_ymax: box_y + line_height,
        baseline_y,
        underline_y,
        border,
        char_width,
        sel_cols,
    }
}
