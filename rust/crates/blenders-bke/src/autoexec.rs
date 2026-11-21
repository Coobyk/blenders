//! Port of `intern/autoexec.cc` — `BKE_autoexec_*` path checks.
//!
//! `U.autoexec_paths` arrives as a slice of `bPathCompare` (ListBase walk is
//! the caller's job); `G.f`/`G.autoexec_override`/`U.flag` arrive as plain
//! flags. Path utils come from `blenders_bli::path_utils`/`fnmatch`.

use blenders_bli::fnmatch::fnmatch;
use blenders_bli::path_utils;
use blenders_dna::userdef_types::{bPathCompare, USER_PATHCMP_GLOB};

/// `AutoExec_Params` (BKE_autoexec.hh).
#[derive(Clone, Copy, Default)]
pub struct AutoExecParams {
    pub skip_overrides: bool,
    pub canonicalize: bool,
    pub strip_filename: bool,
}

fn char_at(s: &str, i: usize) -> u8 {
    s.as_bytes().get(i).copied().unwrap_or(0)
}

/// `autoexec_dirpath_resolve` — `path` untouched, or canonicalized/stripped
/// copy. Returns the resolved dirpath (always an owned String here since the
/// "nothing to do" case is a clone).
fn autoexec_dirpath_resolve(path: &str, canonicalize: bool, strip_filename: bool) -> String {
    if !(canonicalize || strip_filename) {
        return path.to_string();
    }
    let mut dirpath = path.to_string();
    let path_len = dirpath.len();
    let is_dirpath = path_len > 0 && path_utils::slash_is_native_compat(char_at(&dirpath, path_len - 1));
    let do_strip_filename = strip_filename && !is_dirpath;

    if canonicalize {
        path_utils::canonicalize_native(&mut dirpath);
        if !do_strip_filename {
            path_utils::slash_ensure(&mut dirpath);
        }
    }
    if do_strip_filename {
        let base = path_utils::basename(&dirpath);
        dirpath.truncate(dirpath.len() - base.len());
    }
    dirpath
}

/// `BKE_autoexec_match_unchecked` — glob/prefix match against the excluded
/// path list. `paths` is `U.autoexec_paths`.
pub fn bke_autoexec_match_unchecked(
    path: &str,
    canonicalize: bool,
    strip_filename: bool,
    paths: &[bPathCompare],
) -> bool {
    debug_assert!(!path.is_empty());
    let dirpath = autoexec_dirpath_resolve(path, canonicalize, strip_filename);
    let fnmatch_flags = if cfg!(windows) { 4 /* FNM_CASEFOLD */ } else { 0 };
    for path_cmp in paths {
        if char_at(core::str::from_utf8(&path_cmp.path).unwrap_or(""), 0) == 0 {
            continue;
        }
        let pattern = cstr(&path_cmp.path);
        if path_cmp.flag & USER_PATHCMP_GLOB != 0 {
            if fnmatch(pattern.as_bytes(), dirpath.as_bytes(), fnmatch_flags) == 0 {
                return true;
            }
        } else if path_utils::path_ncmp(pattern, &dirpath, pattern.len()) == 0 {
            return true;
        }
    }
    false
}

fn cstr(b: &[u8]) -> &str {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    core::str::from_utf8(&b[..end]).unwrap_or("")
}

/// `BKE_autoexec_match` — `USER_SCRIPT_AUTOEXEC_DISABLE`/`G_FLAG_SCRIPT_AUTOEXEC`
/// pre-checks are the caller's.
pub fn bke_autoexec_match(
    path: &str,
    canonicalize: bool,
    strip_filename: bool,
    paths: &[bPathCompare],
) -> bool {
    bke_autoexec_match_unchecked(path, canonicalize, strip_filename, paths)
}

/// `BKE_autoexec_default_trust_source`.
/// `autoexec_override` is `G.autoexec_override`; `autoexec_disabled` is
/// `U.flag & USER_SCRIPT_AUTOEXEC_DISABLE`.
pub fn bke_autoexec_default_trust_source(
    path: &str,
    params: &AutoExecParams,
    autoexec_override: Option<bool>,
    autoexec_disabled: bool,
    paths: &[bPathCompare],
) -> bool {
    if !params.skip_overrides {
        if let Some(v) = autoexec_override {
            return v;
        }
        if autoexec_disabled {
            return false;
        }
    }
    !bke_autoexec_match_unchecked(path, params.canonicalize, params.strip_filename, paths)
}
