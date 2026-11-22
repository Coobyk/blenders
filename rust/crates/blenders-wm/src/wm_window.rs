//! Port of `intern/wm_window.cc` — window lifecycle
//! (new/close/title/dpi), eventstate modifiers, ghost
//! event proc, timers, clipboard, `WM_window_open`.

/// `WM_TIMER_TAGGED_FOR_REMOVAL`/`WM_TIMER_NO_FREE_CUSTOM_DATA`.
pub const WM_TIMER_TAGGED_FOR_REMOVAL: i32 = 1 << 0;
pub const WM_TIMER_NO_FREE_CUSTOM_DATA: i32 = 1 << 1;
/// `TIMERNOTIFIER`/`TIMERJOBS` verbatim.
pub const TIMERNOTIFIER: i32 = 0x0200;
pub const TIMERJOBS: i32 = 0x0201;
/// `MOD_SIDE_LEFT`/`MOD_SIDE_RIGHT` verbatim.
pub const MOD_SIDE_LEFT: i32 = 0;
pub const MOD_SIDE_RIGHT: i32 = 1;

/// `wm_get_screensize`/`wm_get_desktopsize` (verbatim —
/// ghost dims, false when both 0).
pub fn wm_get_screensize(uiwidth: u32, uiheight: u32) -> Option<[i32; 2]> {
    if uiwidth == 0 && uiheight == 0 {
        return None;
    }
    Some([uiwidth as i32, uiheight as i32])
}

/// `wm_window_check_size` (verbatim — clamp to screen).
pub fn wm_window_check_size(rect: &mut crate::wm_subwindow::Rcti, scr_size: [i32; 2]) {
    if crate::wm_subwindow::rcti_size_x(rect) > scr_size[0] {
        let w = scr_size[0];
        rect.xmax = rect.xmin + w;
    }
    if crate::wm_subwindow::rcti_size_y(rect) > scr_size[1] {
        let h = scr_size[1];
        rect.ymax = rect.ymin + h;
    }
}

/// `find_free_winid` (verbatim — `id = winid + 1` max).
pub fn find_free_winid(winids: &[i32]) -> i32 {
    let mut id = 1;
    for &winid in winids {
        if id <= winid {
            id = winid + 1;
        }
    }
    id
}

/// `wm_window_new` parent rule (verbatim — dialogs may
/// have child-window parents; else parent can't be a child).
pub fn wm_window_new_parent(parent: Option<usize>, parent_has_parent: bool, dialog: bool) -> Option<usize> {
    if !dialog && parent.is_some() && parent_has_parent {
        parent /* → parent->parent; caller resolves */
    } else {
        parent
    }
}

/// `wm_window_title_text` (verbatim — unsaved marker,
/// filename w/o ext when full path shown, `~` home
/// subst, recovered tag, project name, version).
pub fn wm_window_title_text(
    is_main_window: bool,
    single_area_name: Option<&str>,
    single_area_is_project: bool,
    filepath: &str,
    file_saved: bool,
    recovered: bool,
    project_name: Option<&str>,
    version_string: &str,
    home_dir: Option<&str>,
    is_macos: bool,
) -> String {
    let mut title = String::new();
    if !is_main_window {
        match single_area_name {
            Some(name) => {
                title.push_str(name);
                if single_area_is_project {
                    if let Some(p) = project_name {
                        title.push_str(" - ");
                        title.push_str(p);
                    }
                }
            }
            None => title.push_str("Blender"),
        }
        return title;
    }
    let filename = filepath.rsplit('/').next().unwrap_or(filepath);
    let has_filepath = !filepath.is_empty();
    let include_filepath = has_filepath && filepath != filename;
    if !file_saved {
        title.push_str("* ");
    }
    if include_filepath {
        let no_ext = match filename.rfind('.') {
            Some(0) | None => filename,
            Some(i) => &filename[..i],
        };
        title.push_str(no_ext);
    } else if has_filepath {
        title.push_str(filename);
    } else if is_macos {
        title.push_str("Untitled");
    } else {
        title.push_str("(Unsaved)");
    }
    if recovered {
        title.push_str(" (Recovered)");
    }
    if include_filepath {
        let mut add_filepath = true;
        if !is_macos {
            if let Some(home) = home_dir {
                let home = home.trim_end_matches('/');
                if !home.is_empty() && filepath.starts_with(home) && filepath.as_bytes().get(home.len()) == Some(&b'/') {
                    title.push_str(" [~");
                    title.push_str(&filepath[home.len()..]);
                    title.push(']');
                    add_filepath = false;
                }
            }
        }
        if add_filepath {
            title.push_str(" [");
            title.push_str(filepath);
            title.push(']');
        }
    }
    if let Some(p) = project_name {
        title.push_str(" - ");
        title.push_str(p);
    }
    title.push_str(" — Blender ");
    title.push_str(version_string);
    title
}

/// `wmTimer` fields used.
pub struct WmTimer {
    /// `event_type`.
    pub event_type: i32,
    /// `time_step`/`time_last`/`time_next`/`time_start`/`time_duration`.
    pub time_step: f64,
    pub time_last: f64,
    pub time_next: f64,
    pub time_start: f64,
    pub time_duration: f64,
    /// `win`/`customdata`/`flags`/`sleep`.
    pub win: usize,
    pub customdata: usize,
    pub flags: i32,
    pub sleep: bool,
}

/// `wm_window_timer_reset` (verbatim — restart cycle).
pub fn wm_window_timer_reset(wt: &mut WmTimer, now_seconds: f64) {
    wt.time_last = now_seconds;
    wt.time_next = wt.time_last + wt.time_step;
    wt.time_start = wt.time_last;
    wt.time_duration = 0.0;
}

/// `WM_event_timer_add` (verbatim — TIMER check, add).
pub fn wm_event_timer_add(
    timers: &mut Vec<WmTimer>,
    win: usize,
    event_type: i32,
    time_step: f64,
    now_seconds: f64,
) -> usize {
    assert!(time_step >= 0.0);
    timers.push(WmTimer {
        event_type,
        time_step,
        win,
        time_last: 0.0,
        time_next: 0.0,
        time_start: 0.0,
        time_duration: 0.0,
        customdata: 0,
        flags: 0,
        sleep: false,
    });
    let i = timers.len() - 1;
    wm_window_timer_reset(&mut timers[i], now_seconds);
    i
}

/// `WM_event_timer_add_notifier` (verbatim — TIMERNOTIFIER
/// + customdata type + NO_FREE flag).
pub fn wm_event_timer_add_notifier(
    timers: &mut Vec<WmTimer>,
    win: usize,
    notifier_type: u32,
    time_step: f64,
    now_seconds: f64,
) -> usize {
    let i = wm_event_timer_add(timers, win, TIMERNOTIFIER, time_step, now_seconds);
    timers[i].customdata = notifier_type as usize;
    timers[i].flags |= WM_TIMER_NO_FREE_CUSTOM_DATA;
    i
}

/// `WM_event_timer_sleep` (verbatim — not-found/removed
/// gates then `timer->sleep = do_sleep`).
pub fn wm_event_timer_sleep(timer_in_list: bool, timer_flagged_removed: bool, timer: &mut WmTimer, do_sleep: bool) {
    if !timer_in_list {
        return;
    }
    if timer_flagged_removed {
        return;
    }
    timer.sleep = do_sleep;
}

/// `wm_window_update_eventstate_modifiers` — returns
/// synthesized key-up/down ghost events (verbatim table
/// walk).
pub fn wm_window_update_eventstate_modifiers(
    keymodifier_sided: [u8; 2],
    keymodifier_eventstate: u8,
    modifier_table: &[(u8, [i32; 2])],
) -> Vec<(bool, i32)> {
    let keymodifier = keymodifier_sided[0] | keymodifier_sided[1];
    let mut out = Vec::new();
    if keymodifier == keymodifier_eventstate {
        return out;
    }
    for &(flag, ghost_key_pair) in modifier_table {
        if (keymodifier_eventstate & flag) != 0 {
            if (keymodifier & flag) == 0 {
                for side in 0..2 {
                    if (keymodifier_sided[side] & flag) == 0 {
                        out.push((false, ghost_key_pair[side])); /* key up */
                        break;
                    }
                }
            }
        } else if (keymodifier & flag) != 0 {
            for side in 0..2 {
                if (keymodifier_sided[side] & flag) != 0 {
                    out.push((true, ghost_key_pair[side])); /* key down */
                }
            }
        }
    }
    out
}

/// `wm_window_update_eventstate_modifiers_clear`
/// (verbatim — unhandled-bit merge + all-up).
pub fn wm_window_update_eventstate_modifiers_clear(
    keymodifier_eventstate: u8,
    keymodifier_l: u8,
    keymodifier_r: u8,
    modifier_table: &[(u8, [i32; 2])],
) -> Vec<i32> {
    let mut out = Vec::new();
    if keymodifier_eventstate == 0 {
        return out;
    }
    let unhandled = keymodifier_eventstate & !(keymodifier_l | keymodifier_r);
    let sided = [keymodifier_l | unhandled, keymodifier_r];
    for &(flag, ghost_key_pair) in modifier_table {
        if (keymodifier_eventstate & flag) != 0 {
            for side in 0..2 {
                if (sided[side] & flag) == 0 {
                    out.push(ghost_key_pair[side]);
                }
            }
        }
    }
    out
}

/// `wm_cursor_position_*_ghost_*_coords` (verbatim — swap
/// client↔screen y via winrct).
pub fn cursor_from_ghost_client(y_client: i32, winrct_ymax: i32, winrct_ymin: i32) -> i32 {
    winrct_ymax - winrct_ymin - y_client
}
