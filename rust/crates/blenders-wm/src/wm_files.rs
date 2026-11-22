//! Port of `intern/wm_files.cc` — file read/write
//! orchestration: unsaved-changes check, exotic read,
//! autoexec, recent-file history, autosave, operator
//! state dispatch, `WM_OT_*` registrations.

/// `BKE_READ_EXOTIC_*` verbatim.
pub const BKE_READ_EXOTIC_FAIL_PATH: i32 = -1;
pub const BKE_READ_EXOTIC_FAIL_OPEN: i32 = -2;
pub const BKE_READ_EXOTIC_FAIL_FORMAT: i32 = -3;
pub const BKE_READ_EXOTIC_OK_BLEND: i32 = 0;
/// `G_FLAG_SCRIPT_AUTOEXEC` verbatim.
pub const G_FLAG_SCRIPT_AUTOEXEC: i32 = 1 << 0;
/// `USER_GLOBALUNDO` verbatim.
pub const USER_GLOBALUNDO: i32 = 1 << 0;
/// `BLEN_THUMB_SIZE` verbatim.
pub const BLEN_THUMB_SIZE: i32 = 128;

/// `wm_file_or_session_data_has_unsaved_changes`
/// (verbatim — !file_saved || image modified || unsaved
/// asset catalogs).
pub fn wm_file_or_session_data_has_unsaved_changes(
    file_saved: bool,
    image_should_save_modified: bool,
    asset_library_has_unsaved_catalogs: bool,
) -> bool {
    !file_saved || image_should_save_modified || asset_library_has_unsaved_catalogs
}

/// `wm_read_exotic` (verbatim — dir check, open, header
/// decode).
pub fn wm_read_exotic(
    filepath: &str,
    open_ok: bool,
    reader_ok: bool,
    header_is_blend: bool,
) -> i32 {
    let len = filepath.len();
    if len > 0 && (filepath.ends_with('/') || filepath.ends_with('\\')) {
        return BKE_READ_EXOTIC_FAIL_PATH;
    }
    if !open_ok {
        return BKE_READ_EXOTIC_FAIL_OPEN;
    }
    if !reader_ok {
        return BKE_READ_EXOTIC_FAIL_FORMAT;
    }
    if header_is_blend {
        BKE_READ_EXOTIC_OK_BLEND
    } else {
        BKE_READ_EXOTIC_FAIL_FORMAT
    }
}

/// `WM_file_autoexec_init` (verbatim — override keeps,
/// autoexec_match clears flag).
pub fn wm_file_autoexec_init(
    autoexec_override: bool,
    g_f: &mut i32,
    filepath: &str,
    autoexec_match: bool,
) {
    if autoexec_override {
        return;
    }
    if (*g_f & G_FLAG_SCRIPT_AUTOEXEC) != 0 {
        if !filepath.is_empty() && autoexec_match {
            *g_f &= !G_FLAG_SCRIPT_AUTOEXEC;
        }
    }
}

/// `wm_history_file_update` path logic (verbatim —
/// dedupe + prepend + truncate 30).
pub fn wm_history_file_update(recent_files: &mut Vec<String>, blendfile_path: &str) {
    if blendfile_path.is_empty() {
        return;
    }
    recent_files.retain(|p| p != blendfile_path);
    recent_files.insert(0, blendfile_path.to_string());
    recent_files.truncate(30);
}

/// `wm_history_file_write` — writes one path per line
/// (verbatim serialization).
pub fn wm_history_file_write(recent_files: &[String]) -> String {
    let mut out = String::new();
    for recent in recent_files {
        out.push_str(recent);
        out.push('\n');
    }
    out
}

/// `blend_file_thumb_clamp_size` (verbatim — scale to
/// `limit` preserving aspect).
pub fn blend_file_thumb_clamp_size(size: [i32; 2], limit: i32) -> [i32; 2] {
    if size[0] > size[1] {
        [limit, ((((size[1] as f32) / (size[0] as f32)) * limit as f32) as i32).max(1)]
    } else {
        [((((size[0] as f32) / (size[1] as f32)) * limit as f32) as i32).max(1), limit]
    }
}

/// `wm_autosave_location` filename (verbatim —
/// `<name>_<pid>_autosave.blend`, strip `.blend`).
pub fn wm_autosave_location(blendfile_path: &str, pid: i32, tempdir_base: &str) -> String {
    let pid = pid.abs();
    let basename = blendfile_path.rsplit('/').next().unwrap_or("");
    let filename = if !blendfile_path.is_empty() && basename.len() > 6 {
        format!("{}_{}_autosave.blend", &basename[..basename.len() - 6], pid)
    } else {
        format!("{}_autosave.blend", pid)
    };
    format!("{}/{}", tempdir_base.trim_end_matches('/'), filename)
}

/// `wm_autosave_write_try` decision (verbatim —
/// saved→skip, undo-compatible → write, !GLOBALUNDO →
/// write, else retry-later).
pub enum AutosaveTry {
    /// Skip (returns success).
    Skip,
    /// `WM_autosave_write` now.
    WriteNow,
    /// Can't write with MemFile, retry later.
    RetryLater,
}
pub fn wm_autosave_write_try(file_saved: bool, undosys_autosave_compatible: bool, user_globalundo: bool) -> AutosaveTry {
    if file_saved {
        return AutosaveTry::Skip;
    }
    if undosys_autosave_compatible {
        return AutosaveTry::WriteNow;
    }
    if !user_globalundo {
        return AutosaveTry::WriteNow;
    }
    AutosaveTry::RetryLater
}

/// `OPEN_MAINFILE_STATE_*` verbatim.
pub const OPEN_MAINFILE_STATE_DISCARD_CHANGES: i32 = 0;
pub const OPEN_MAINFILE_STATE_SELECT_FILE_PATH: i32 = 1;
pub const OPEN_MAINFILE_STATE_OPEN: i32 = 2;

/// `operator_state_dispatch` (verbatim — find target
/// matching `state`, else unreachable→CANCELLED).
pub fn operator_state_dispatch(
    state: i32,
    targets: &[(i32, usize)],
    run: &mut dyn FnMut(usize) -> i32,
) -> i32 {
    for &(tstate, thandle) in targets {
        if tstate == state {
            return (run)(thandle);
        }
    }
    crate::wm_files_colorspace::OPERATOR_CANCELLED
}
