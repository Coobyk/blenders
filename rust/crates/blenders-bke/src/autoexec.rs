//! Port of `intern/autoexec.cc` — `BKE_autoexec_match*` trusted-path
//! checks against `U.autoexec_paths`.
//!
//! `bPathCompare` entries are `(path, flag)`; the userdef/G.flags gates
//! come in as bools (PORT_PENDING `U`/`G` globals).

use blenders_bli::fnmatch::{fnmatch_str, FNM_CASEFOLD};
use blenders_bli::path_utils;
use blenders_dna::userdef_types::USER_PATHCMP_GLOB;

/// `autoexec_dirpath_resolve` — canonicalize and/or strip the filename.
fn autoexec_dirpath_resolve(path: &str, canonicalize: bool, strip_filename: bool) -> String {
    if !(canonicalize || strip_filename) {
        return path.to_string();
    }
    let mut dirpath = path.to_string();
    let is_dirpath = !dirpath.is_empty()
        && path_utils::slash_is_native_compat(dirpath.as_bytes()[dirpath.len() - 1]);
    let do_strip_filename = strip_filename && !is_dirpath;
    if canonicalize {
        path_utils::canonicalize_native(&mut dirpath);
        if !do_strip_filename {
            path_utils::slash_ensure(&mut dirpath);
        }
    }
    if do_strip_filename {
        /* Truncating at the file name leaves the trailing slash. */
        if let Some(base) = dirpath.rfind(|c| c == '/' || c == '\\') {
            dirpath.truncate(base + 1);
        } else {
            dirpath.clear();
        }
    }
    dirpath
}

/// `BKE_autoexec_match_unchecked` — `autoexec_paths` is the
/// `ListBase bPathCompare` flattened to `(path, flag)`.
pub fn bke_autoexec_match_unchecked(
    path: &str,
    canonicalize: bool,
    strip_filename: bool,
    autoexec_paths: &[(String, u32)],
) -> bool {
    debug_assert!(!path.is_empty());
    let dirpath = autoexec_dirpath_resolve(path, canonicalize, strip_filename);
    #[cfg(windows)]
    let fnmatch_flags = FNM_CASEFOLD;
    #[cfg(not(windows))]
    let fnmatch_flags = 0;
    for (path_cmp, flag) in autoexec_paths {
        if path_cmp.is_empty() {
            /* `if (path_cmp->path[0] == '\0') continue` */
        } else if flag & USER_PATHCMP_GLOB as u32 != 0 {
            if fnmatch_str(path_cmp, &dirpath, fnmatch_flags) {
                return true;
            }
        } else if path_utils::path_ncmp(path_cmp, &dirpath, path_cmp.len()) == 0 {
            return true;
        }
    }
    false
}

/// `BKE_autoexec_match` — call gate asserted in C
/// (`USER_SCRIPT_AUTOEXEC_DISABLE`/`G_FLAG_SCRIPT_AUTOEXEC`); caller
/// enforces it.
pub fn bke_autoexec_match(
    path: &str,
    canonicalize: bool,
    strip_filename: bool,
    autoexec_paths: &[(String, u32)],
) -> bool {
    bke_autoexec_match_unchecked(path, canonicalize, strip_filename, autoexec_paths)
}

/// `AutoExec_Params` (BKE_autoexec.hh).
#[derive(Clone, Copy, Default)]
pub struct AutoExecParams {
    pub canonicalize: bool,
    pub strip_filename: bool,
    pub skip_overrides: bool,
}

/// `BKE_autoexec_default_trust_source` — `autoexec_override` is
/// `G.autoexec_override` (`std::optional<bool>`), `script_autoexec_disable`
/// the userpref flag check.
pub fn bke_autoexec_default_trust_source(
    path: &str,
    params: &AutoExecParams,
    autoexec_override: Option<bool>,
    script_autoexec_disable: bool,
    autoexec_paths: &[(String, u32)],
) -> bool {
    if !params.skip_overrides {
        if let Some(over) = autoexec_override {
            return over;
        }
        if script_autoexec_disable {
            return false;
        }
    }
    !bke_autoexec_match_unchecked(path, params.canonicalize, params.strip_filename, autoexec_paths)
}
