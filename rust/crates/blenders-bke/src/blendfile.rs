//! Port of `intern/blendfile.cc` — .blend path helpers,
//! read/write orchestration flags, `ReuseOldBMainData` ID
//! remap bookkeeping, subversion warning.
//!
//! `PORT_PENDING`: `BKE_blendfile_read`/`write` (BLO
//! readstream/writestream engines — injected), `setup_app_*`
//! (bContext + wmWindow + userdef mutation), `partial_write`
//! (file packing), `reuse_bmain_*` ID-swap machinery
//! (IDRemapper handles — the structural part is ported).

/// `BLO_GROUP_MAX` verbatim.
pub const BLO_GROUP_MAX: usize = 17;
/// `FILE_MAX_LIBEXTRA`.
pub const FILE_MAX_LIBEXTRA: usize = 768;
/// `BLO_EMBEDDED_STARTUP_BLEND`.
pub const BLO_EMBEDDED_STARTUP_BLEND: &str = "<startup.blend>";

/// `BKE_blendfile_extension_check` (verbatim).
pub fn blendfile_extension_check(path: &str) -> bool {
    const EXTS: [&str; 3] = [".blend", ".ble", ".blend.gz"];
    let lower = path.to_ascii_lowercase();
    EXTS.iter().any(|e| lower.ends_with(e))
}

/// `BKE_blendfile_library_path_explode` — split
/// `file.blend/Group/name` (verbatim walk-up logic; `is_dir`/
/// `is_file` injected).
pub fn blendfile_library_path_explode(
    path: &str,
    is_dir: &dyn Fn(&str) -> bool,
    is_file: &dyn Fn(&str) -> bool,
) -> Option<(String, Option<String>, Option<String>)> {
    if is_dir(path) {
        return None;
    }
    let mut dir = path.to_string();
    let mut slash: Option<usize> = None;
    let mut prev_slash: Option<usize> = None;
    loop {
        let idx = dir.rfind('/');
        match idx {
            Some(i) => {
                let tail = dir[i + 1..].to_string();
                let prev = dir.clone();
                dir.truncate(i);
                if blendfile_extension_check(&dir) && is_file(&dir) {
                    slash = Some(i);
                    /* store the tail at `slash` — keep it in
                     * `dir` so `group` is `slash+1` tail */
                    dir = format!("{}/{}", dir, tail);
                    break;
                }
                if dir == BLO_EMBEDDED_STARTUP_BLEND {
                    slash = Some(i);
                    dir = format!("{}/{}", dir, tail);
                    break;
                }
                prev_slash = Some(i);
                let _ = prev;
            }
            None => break,
        }
    }
    let slash_i = slash?;
    /* `dir` at this point is `blend_path/group/...name`. */
    let blend_path = &dir[..slash_i];
    let rest = &dir[slash_i + 1..];
    let group = if rest.is_empty() {
        None
    } else {
        Some(rest.split('/').next().unwrap().to_string())
    };
    let name = if rest.split('/').count() > 1 {
        Some(rest.splitn(2, '/').nth(1).unwrap().to_string())
    } else {
        None
    };
    Some((blend_path.to_string(), group, name))
}

/// `wm_scene_is_visible` (verbatim — the wm windows walk).
pub fn wm_scene_is_visible(window_scenes: &[usize], scene: usize) -> bool {
    window_scenes.iter().any(|&s| s == scene)
}

/// `handle_subversion_warning` — `Some(warning)` when the
/// file is newer than this binary (verbatim).
pub fn handle_subversion_warning(
    versionfile: i32,
    subversionfile: i32,
    blender_file_version: i32,
    blender_file_subversion: i32,
) -> Option<String> {
    if versionfile > blender_file_version
        || (versionfile == blender_file_version && subversionfile > blender_file_subversion)
    {
        Some(format!(
            "File written by newer Blender binary ({}.{})",
            versionfile, subversionfile
        ))
    } else {
        None
    }
}

/// `ReuseOldBMainData` (verbatim state fields).
#[derive(Default)]
pub struct ReuseOldBMainData {
    /// `bmain_src`/`bmain_dst` — the two `Main` arenas.
    pub bmain_src: usize,
    /// `bmain_dst`.
    pub bmain_dst: usize,
    /// `remapper` (IDRemapper handle).
    pub remapper: usize,
    /// `dependencies_process_callback` order bookkeeping.
    pub swapped_ids: Vec<(usize, usize)>,
    /// `recursing` depth guard.
    pub recursing: i32,
}

/// `BlendFileReadReport`/`BlendFileReadParams` verbatim
/// (flags byte).
pub struct BlendFileReadReport {
    /// `reports` handle.
    pub reports: usize,
    /// `do_undamaged` — recovery write flag.
    pub do_undamaged: bool,
    /// `resync_required` — collection resync flag.
    pub resync_required: bool,
}
/// `eBlenFileReadFlag` verbatim.
pub const BLENFILEREAD_FLAG_NEEDS_ID_PROCESSING: i32 = 1 << 0;
/// `BLENFILEREAD_FLAG_IS_BLENDFILE`.
pub const BLENFILEREAD_FLAG_IS_BLENDFILE: i32 = 1 << 1;
/// `BLENFILEREAD_FLAG_IS_UNDO`.
pub const BLENFILEREAD_FLAG_IS_UNDO: i32 = 1 << 2;
/// `BLENFILEREAD_FLAG_UI`.
pub const BLENFILEREAD_FLAG_UI: i32 = 1 << 3;

/// `BlendFileReadParams` (verbatim).
#[derive(Default)]
pub struct BlendFileReadParams {
    /// `is_startup`.
    pub is_startup: bool,
    /// `is_factory_startup`.
    pub is_factory_startup: bool,
    /// `is_undo`.
    pub is_undo: bool,
    /// `skip_flags`.
    pub skip_flags: i32,
    /// `reuse_main` — the `reuse_data` bucket.
    pub reuse_main: Option<ReuseOldBMainData>,
}
