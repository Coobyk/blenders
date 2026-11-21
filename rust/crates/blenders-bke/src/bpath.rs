//! Port of `intern/bpath.cc` — `BKE_bpath_*` file-path traversal API.
//!
//! The per-ID dispatch (`BKE_bpath_foreach_path_id`) needs `IDTypeInfo`
//! `foreach_path` callbacks + `DEG` — caller injects an `IdsIterator`
//! closure; `Main`/`ID` are `usize` handles. All path machinery is
//! verbatim.

use blenders_bli::path_utils;
use blenders_bli::storage::is_file;

pub const FILE_MAX: usize = 1024;
const MAX_DIR_RECURSE: i32 = 20;
/// `FILESIZE_INVALID_DIRECTORY`.
pub const FILESIZE_INVALID_DIRECTORY: i64 = -1;

/// `eBPathForeachFlag` (BKE_bpath.hh).
pub const BKE_BPATH_FOREACH_PATH_ABSOLUTE: u16 = 1 << 0;
pub const BKE_BPATH_FOREACH_PATH_SKIP_LINKED: u16 = 1 << 1;
pub const BKE_BPATH_FOREACH_PATH_SKIP_PACKED: u16 = 1 << 2;
pub const BKE_BPATH_FOREACH_PATH_RESOLVE_TOKEN: u16 = 1 << 3;
pub const BKE_BPATH_FOREACH_PATH_SKIP_MULTIFILE: u16 = 1 << 8;
pub const BKE_BPATH_FOREACH_PATH_RELOAD_EDITED: u16 = 1 << 9;
pub const BKE_BPATH_FOREACH_PATH_EXPAND_TOKENS: u16 = 1 << 10;
pub const BKE_BPATH_FOREACH_PATH_EXPAND_SEQUENCES: u16 = 1 << 11;
pub const BKE_BPATH_FOREACH_PATH_EXPAND_CACHES: u16 = 1 << 12;
/// `BKE_BPATH_TRAVERSE_SKIP_WEAK_REFERENCES`.
pub const BKE_BPATH_TRAVERSE_SKIP_WEAK_REFERENCES: u16 = 1 << 14;

/// `BPathSummary`.
#[derive(Clone, Copy, Default)]
pub struct BPathSummary {
    pub count_total: i32,
    pub count_changed: i32,
    pub count_failed: i32,
}

/// `BKE_bpath_summary_report` — returns the report string (ReportList
/// is not ported; caller logs at warning level when `count_failed > 0`).
pub fn bke_bpath_summary_report(summary: &BPathSummary) -> String {
    format!(
        "Total files {} | Changed {} | Failed {}",
        summary.count_total, summary.count_changed, summary.count_failed
    )
}

/// `BPathForeachPathData` — callback is `FnMut(&mut BPathForeachPathData,
/// path_dst: &mut String, path_src: &str) -> bool` injected at each call
/// site (stored here behind `FnMut` boxing like the C member).
pub struct BPathForeachPathData<'a> {
    pub bmain: usize,
    pub flag: u16,
    /// Only set when `BKE_BPATH_FOREACH_PATH_ABSOLUTE`.
    pub absolute_base_path: Option<String>,
    pub owner_id: usize,
    pub is_path_modified: bool,
    pub is_expanded: bool,
    pub is_cache: bool,
    pub is_readonly: bool,
    pub user_data: usize,
    callback: Option<Box<dyn FnMut(&mut BPathForeachPathData, &mut String, &str) -> bool + 'a>>,
}

impl<'a> BPathForeachPathData<'a> {
    /// Set the traversal callback (`callback_function` in C).
    pub fn set_callback<F>(&mut self, f: F)
    where
        F: FnMut(&mut BPathForeachPathData, &mut String, &str) -> bool + 'a,
    {
        self.callback = Some(Box::new(f));
    }

    /// Take+call+restore so the callback can receive `self` (as C does).
    fn call_callback(&mut self, path_dst: &mut String, path_src: &str) -> bool {
        let mut cb = match self.callback.take() {
            Some(cb) => cb,
            None => return false,
        };
        let result = cb(self, path_dst, path_src);
        self.callback = Some(cb);
        result
    }
    /// `BKE_bpath_foreach_path_id` — PORT_PENDING: `IDTypeInfo::foreach_path`
    /// + `node_tree_from_id` + `DEG_id_tag_update`. Caller supplies an
    /// `id_foreach_path` closure per ID instead.
    pub fn foreach_path_id<F>(&mut self, id: usize, mut id_foreach_path: F)
    where
        F: FnMut(&mut BPathForeachPathData, usize),
    {
        if (self.flag & BKE_BPATH_FOREACH_PATH_SKIP_LINKED) != 0 {
            /* ID_IS_LINKED check is the caller's (needs ID flags). */
        }
        self.owner_id = id;
        self.is_path_modified = false;
        self.is_expanded = false;
        self.is_cache = false;
        self.is_readonly = false;
        id_foreach_path(self, id);
    }

    /// `BKE_bpath_foreach_path_fixed_process`.
    pub fn foreach_path_fixed_process(&mut self, path: &mut String) -> bool {
        let mut path_src = path.clone();
        if let Some(base) = &self.absolute_base_path {
            path_utils::abs(&mut path_src, base);
        }
        /* so functions can check old value */
        let mut path_dst = path.clone();
        if self.call_callback(&mut path_dst, &path_src) {
            *path = path_dst;
            self.is_path_modified = true;
            return true;
        }
        false
    }

    /// `BKE_bpath_foreach_path_readonly_process`.
    pub fn foreach_path_readonly_process(&mut self, path: &str) {
        let mut path_src = path.to_string();
        if let Some(base) = &self.absolute_base_path {
            path_utils::abs(&mut path_src, base);
        }
        /* Any modification to path_dst will be discarded. */
        let mut path_dst = path_src.clone();
        self.is_readonly = true;
        let changed = self.call_callback(&mut path_dst, &path_src);
        self.is_readonly = false;
        debug_assert!(!changed, "read-only path edited in bpath foreach");
    }

    /// `BKE_bpath_foreach_path_dirfile_fixed_process` — returns the new
    /// `(dir, file)` split when the callback modified the path.
    pub fn foreach_path_dirfile_fixed_process(
        &mut self,
        path_dir: &str,
        path_file: &str,
    ) -> Option<(String, String)> {
        let mut path_src = path_utils::join(&[path_dir, path_file]);
        let mut path_dst = path_src.clone();
        if let Some(base) = &self.absolute_base_path {
            path_utils::abs(&mut path_src, base);
        }
        if self.call_callback(&mut path_dst, &path_src) {
            self.is_path_modified = true;
            return Some(path_utils::split_dir_file(&path_dst));
        }
        None
    }

    /// `BKE_bpath_foreach_path_allocated_process` — allocated (owned)
    /// path string; callback may replace it.
    pub fn foreach_path_allocated_process(&mut self, path: &mut Option<String>) -> bool {
        let mut path_src = path.clone().unwrap_or_default();
        if let Some(base) = &self.absolute_base_path {
            path_utils::abs(&mut path_src, base);
        }
        let mut path_dst = String::new();
        if self.call_callback(&mut path_dst, &path_src) {
            *path = Some(path_dst);
            self.is_path_modified = true;
            return true;
        }
        false
    }
}

/// `BKE_bpath_sequence_filepaths_foreach` — every existing file matching
/// `<head><digits><tail>` under `abs_filepath`'s directory.
pub fn bke_bpath_sequence_filepaths_foreach<F: FnMut(&str)>(abs_filepath: &str, mut callback: F) {
    let (dirname, filename) = path_utils::split_dir_file(abs_filepath);
    let (head, tail, digits_len) = match path_utils::sequence_decode(&filename) {
        Some((_pic, head_len, tail_len, digits_len)) => (
            filename[..head_len].to_string(),
            filename[filename.len() - tail_len..].to_string(),
            digits_len,
        ),
        None => (String::new(), String::new(), 0),
    };
    if digits_len == 0 {
        if is_file(abs_filepath) {
            callback(abs_filepath);
        }
        return;
    }
    let head_len = head.len();
    let tail_len = tail.len();
    let expected_len = head_len + digits_len + tail_len;
    for entry in blenders_bli::filelist::builddir(&dirname) {
        if entry.type_bits & 0o170000 != 0o100000 {
            /* !S_IFREG */
            continue;
        }
        let name = &entry.relname;
        if name.len() != expected_len
            || !name.starts_with(&head)
            || !blenders_bli::string::str_endswith(name, &tail)
        {
            continue;
        }
        if name.as_bytes()[head_len..head_len + digits_len]
            .iter()
            .all(|b| b.is_ascii_digit())
        {
            callback(&entry.path);
        }
    }
}

/// `missing_files_find__recursive` — depth-limited search for the biggest
/// file named `filename_src`; returns `(path, size)` of the best match.
pub fn missing_files_find_recursive(
    search_directory: &str,
    filename_src: &str,
    r_filesize: &mut i64,
    r_recurse_depth: &mut i32,
) -> Option<String> {
    let mut found: Option<String> = None;
    let dir = match std::fs::read_dir(search_directory) {
        Ok(d) => d,
        Err(_) => return None,
    };
    for de in dir.flatten() {
        let name = de.file_name().to_string_lossy().into_owned();
        if name == "." || name == ".." {
            continue;
        }
        let path = path_utils::join(&[search_directory, &name]);
        let md = match de.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if md.is_file() {
            if name == filename_src {
                let size = md.len() as i64;
                if size > 0 && size > *r_filesize {
                    found = Some(path);
                    *r_filesize = size;
                }
            }
        } else if md.is_dir() && *r_recurse_depth <= MAX_DIR_RECURSE {
            *r_recurse_depth += 1;
            if let Some(p) =
                missing_files_find_recursive(&path, filename_src, r_filesize, r_recurse_depth)
            {
                found = Some(p);
            }
            *r_recurse_depth -= 1;
        }
    }
    found
}

/// `PathStore` — backup/restore of every ID filepath (`bpath_list_*`).
#[derive(Default)]
pub struct PathStore {
    pub entries: Vec<(usize, String, String)>, /* id, slot-name, path */
}

/// `bpath_absolute_relative_convert` — `path_src`→`path_dst` for
/// absolute⇄relative conversion (shared by the two foreach callbacks).
pub fn bpath_absolute_relative_convert(
    path_src: &str,
    basepath: &str,
    to_relative: bool,
) -> Option<String> {
    if path_src.is_empty() {
        return None;
    }
    let mut path = path_src.to_string();
    if to_relative {
        path_utils::rel(&mut path, basepath);
    } else {
        path_utils::abs(&mut path, basepath);
    }
    Some(path)
}
