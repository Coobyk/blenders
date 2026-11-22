//! Port of `intern/wm_window_csd_layout.cc` — client-side
//! decoration layout: body/title/buttons rect computation.

/// `GHOST_CSD_Elem` verbatim (`GHOST_CSD.h`).
#[derive(Clone, Copy, Default)]
pub struct GhostCsdElem {
    /// `type` (`GHOST_TCSD_Type`).
    pub elem_type: i32,
    /// `bounds[0]` x (left..right), `bounds[1]` y
    /// (top..bottom).
    pub bounds: [[i32; 2]; 2],
}

/// `GHOST_TCSD_Type` values (`GHOST_CSD.h`).
pub const GHOST_K_CSD_TYPE_BODY: i32 = 1;
pub const GHOST_K_CSD_TYPE_TITLEBAR: i32 = 2;
pub const GHOST_K_CSD_TYPE_NUM: usize = 16;

/// `GHOST_CSD_Layout` verbatim fields.
pub struct GhostCsdLayout {
    /// `buttons` (ordered type list).
    pub buttons: Vec<i32>,
}

/// `GHOST_CSD_DPI_FRACTIONAL_BASE` verbatim.
pub const GHOST_CSD_DPI_FRACTIONAL_BASE: i32 = 96;

/// `WM_window_csd_fractional_scale_apply` (verbatim).
pub fn wm_window_csd_fractional_scale_apply(value: i32, fractional_scale: [i32; 2]) -> i32 {
    (value * fractional_scale[1]) / fractional_scale[0]
}

/// `WM_window_csd_layout_callback` (verbatim — body, left
/// buttons up to titlebar, right buttons after, titlebar;
/// returns element count).
pub fn wm_window_csd_layout_callback(
    window_size: [i32; 2],
    fractional_scale: [i32; 2],
    window_state: i32,
    csd_layout: Option<&GhostCsdLayout>,
    csd_elems: &mut [GhostCsdElem],
) -> i32 {
    const CSD_TITLE_HEIGHT: i32 = 25;
    let title = wm_window_csd_fractional_scale_apply(CSD_TITLE_HEIGHT, fractional_scale);
    assert!(window_state != 2 /*GHOST_kWindowStateFullScreen*/);
    let mut decor_num = 0usize;
    /* Window contents. */
    csd_elems[decor_num] = GhostCsdElem {
        elem_type: GHOST_K_CSD_TYPE_BODY,
        bounds: [[0, window_size[0]], [title, window_size[1]]],
    };
    decor_num += 1;
    if let Some(csd_layout) = csd_layout {
        let mut button_layout_title_index = 0;
        /* Buttons on the left. */
        {
            let mut button_index = 0;
            for (i, &b) in csd_layout.buttons.iter().enumerate() {
                if b == GHOST_K_CSD_TYPE_TITLEBAR {
                    button_layout_title_index = i;
                    break;
                }
                csd_elems[decor_num] = GhostCsdElem {
                    elem_type: b,
                    bounds: [
                        [title * button_index, title + title * button_index],
                        [0, title],
                    ],
                };
                decor_num += 1;
                button_index += 1;
            }
        }
        /* Buttons on the right. */
        {
            let mut button_index = 0;
            let mut i = csd_layout.buttons.len() as i32 - 1;
            while i > button_layout_title_index as i32 {
                let t = csd_layout.buttons[i as usize];
                csd_elems[decor_num] = GhostCsdElem {
                    elem_type: t,
                    bounds: [
                        [
                            (window_size[0] - title) - title * button_index,
                            window_size[0] - title * button_index,
                        ],
                        [0, title],
                    ],
                };
                decor_num += 1;
                button_index += 1;
                i -= 1;
            }
        }
    }
    /* Title bar. */
    csd_elems[decor_num] = GhostCsdElem {
        elem_type: GHOST_K_CSD_TYPE_TITLEBAR,
        bounds: [[0, window_size[0]], [0, title]],
    };
    decor_num += 1;
    decor_num as i32
}

/// `WM_window_csd_rect_calc` (verbatim — find BODY elem,
/// flip Y into rcti).
pub fn wm_window_csd_rect_calc(
    win_size: (i32, i32),
    fractional_scale: [i32; 2],
    window_state: i32,
    csd_layout: Option<&GhostCsdLayout>,
) -> crate::wm_subwindow::Rcti {
    let mut csd_elems = [GhostCsdElem::default(); GHOST_K_CSD_TYPE_NUM];
    let decor_num = wm_window_csd_layout_callback(
        [win_size.0, win_size.1],
        fractional_scale,
        window_state,
        csd_layout,
        &mut csd_elems,
    );
    let mut elem = None;
    for i in 0..decor_num as usize {
        if csd_elems[i].elem_type == GHOST_K_CSD_TYPE_BODY {
            elem = Some(csd_elems[i]);
            break;
        }
    }
    let elem = match elem {
        Some(e) => e,
        None => {
            return crate::wm_subwindow::Rcti {
                xmin: 0,
                xmax: win_size.0,
                ymin: 0,
                ymax: win_size.1,
            };
        }
    };
    crate::wm_subwindow::Rcti {
        xmin: elem.bounds[0][0],
        xmax: elem.bounds[0][1],
        ymin: win_size.1 - elem.bounds[1][1],
        ymax: win_size.1 - elem.bounds[1][0],
    }
}
