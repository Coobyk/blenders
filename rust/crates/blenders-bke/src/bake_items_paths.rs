//! Port of `intern/bake_items_paths.cc` — bake meta-file naming & discovery.

/// `SubFrame` — frame with sub-frame fraction (f32 in C++).
pub type SubFrame = f32;

/// `MetaFile` (BKE_bake_items_paths.hh).
pub struct MetaFile {
    pub frame: SubFrame,
    pub path: String,
}

/// `BakePath`.
pub struct BakePath {
    pub bake_dir: String,
    pub meta_dir: String,
    pub blobs_dir: String,
}

impl BakePath {
    /// `BakePath::from_single_root` — `<root>/meta`, `<root>/blobs`.
    pub fn from_single_root(root_dir: &str) -> Self {
        Self {
            bake_dir: root_dir.to_string(),
            meta_dir: blenders_bli::path_utils::join(&[root_dir, "meta"]),
            blobs_dir: blenders_bli::path_utils::join(&[root_dir, "blobs"]),
        }
    }
}

/// `frame_to_file_name` — `%011.5f` with `.` → `_`.
pub fn frame_to_file_name(frame: SubFrame) -> String {
    format!("{:011.5}", frame).replace('.', "_")
}

/// `file_name_to_frame` — `_` → `.` then `std::stof` (leading float parse).
pub fn file_name_to_frame(file_name: &str) -> Option<SubFrame> {
    let s = file_name.replace('_', ".");
    // `stof` accepts a leading float and ignores the rest (e.g. ".json").
    let mut end = 0;
    for (i, c) in s.char_indices() {
        if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E' {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    s[..end].parse::<f32>().ok()
}

/// `find_sorted_meta_files` — all `*.json` in `meta_dir` whose stem parses as
/// a frame, sorted by frame.
pub fn find_sorted_meta_files(meta_dir: &str) -> Vec<MetaFile> {
    let mut meta_files = Vec::new();
    if let Ok(rd) = std::fs::read_dir(meta_dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if !path.to_string_lossy().ends_with(".json") {
                continue;
            }
            let Some(name) = path.file_name().map(|s| s.to_string_lossy().to_string()) else {
                continue;
            };
            let Some(frame) = file_name_to_frame(&name) else {
                continue;
            };
            meta_files.push(MetaFile {
                frame,
                path: path.to_string_lossy().to_string(),
            });
        }
    }
    meta_files.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    meta_files
}
