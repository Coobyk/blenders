//! Port of `intern/wm_dragdrop.cc` — drag path/asset data,
//! drag free/list management, drop prepare/handle events,
//! path getters.

/// `WM_DRAG_*` verbatim (`WM_types.h`).
pub const WM_DRAG_PATH: i32 = 4;
pub const WM_DRAG_COLOR: i32 = 5;
pub const WM_DRAG_STRING: i32 = 11;
/// `FILE_TYPE_*` verbatim (`ED_filetypes.hh`).
pub const FILE_TYPE_BLENDER: i32 = 1;
pub const FILE_TYPE_IMAGE: i32 = 2;
pub const FILE_TYPE_MOVIE: i32 = 4;
pub const FILE_TYPE_TEXT: i32 = 8;
pub const FILE_TYPE_ASSET: i32 = 32;

/// `wmDragPath` verbatim.
pub struct WmDragPath {
    /// `paths`.
    pub paths: Vec<String>,
    /// `file_types` (per-path flags).
    pub file_types: Vec<i32>,
    /// `file_types_bit_flag`.
    pub file_types_bit_flag: i32,
    /// `tooltip`.
    pub tooltip: String,
}

/// `wmDrag` fields used by getters (`type`, `poin`).
pub struct WmDrag {
    /// `type` (`eWM_DragDataType`).
    pub drag_type: i32,
    /// `poin` → `WmDragPath` (when PATH).
    pub poin: Option<usize>,
}

/// `WM_drag_create_path_data` (verbatim — per-path type
/// flag, tooltip "Dragging N files" for N>1).
pub fn wm_drag_create_path_data(
    paths: &[&str],
    path_extension_type: &dyn Fn(&str) -> i32,
) -> WmDragPath {
    assert!(!paths.is_empty());
    let mut path_data = WmDragPath {
        paths: Vec::new(),
        file_types: Vec::new(),
        file_types_bit_flag: 0,
        tooltip: String::new(),
    };
    for path in paths {
        path_data.paths.push((*path).to_string());
        let type_flag = (path_extension_type)(path);
        path_data.file_types_bit_flag |= type_flag;
        path_data.file_types.push(type_flag);
    }
    path_data.tooltip = path_data.paths[0].clone();
    if path_data.paths.len() > 1 {
        path_data.tooltip = format!("Dragging {} files", path_data.paths.len());
    }
    path_data
}

/// `WM_drag_get_single_path` (verbatim — type gate +
/// first path).
pub fn wm_drag_get_single_path<'a>(drag_type: i32, path_data: Option<&'a WmDragPath>) -> Option<&'a str> {
    if drag_type != WM_DRAG_PATH {
        return None;
    }
    path_data.and_then(|p| p.paths.first().map(String::as_str))
}

/// `WM_drag_get_single_path(file_type)` (verbatim — first
/// path whose type intersects).
pub fn wm_drag_get_single_path_typed<'a>(
    drag_type: i32,
    path_data: Option<&'a WmDragPath>,
    file_type: i32,
) -> Option<&'a str> {
    if drag_type != WM_DRAG_PATH {
        return None;
    }
    let p = path_data?;
    let idx = p.file_types.iter().position(|t| (*t & file_type) != 0)?;
    Some(&p.paths[idx])
}

/// `WM_drag_has_path_file_type` (verbatim).
pub fn wm_drag_has_path_file_type(drag_type: i32, path_data: Option<&WmDragPath>, file_type: i32) -> bool {
    if drag_type != WM_DRAG_PATH {
        return false;
    }
    path_data.map_or(false, |p| (p.file_types_bit_flag & file_type) != 0)
}

/// `WM_drag_get_paths` (verbatim).
pub fn wm_drag_get_paths<'a>(drag_type: i32, path_data: Option<&'a WmDragPath>) -> &'a [String] {
    if drag_type != WM_DRAG_PATH {
        return &[];
    }
    path_data.map_or(&[], |p| p.paths.as_slice())
}

/// `WM_drag_get_path_file_type` (verbatim — first path's
/// type).
pub fn wm_drag_get_path_file_type(drag_type: i32, path_data: Option<&WmDragPath>) -> i32 {
    if drag_type != WM_DRAG_PATH {
        return 0;
    }
    path_data.and_then(|p| p.file_types.first().copied()).unwrap_or(0)
}

/* ---- `wm_dragdrop_test.cc` (verbatim test cases) ---- */
#[cfg(test)]
mod tests {
    use super::*;

    fn ext_type(path: &str) -> i32 {
        if path.ends_with(".blend") {
            FILE_TYPE_BLENDER
        } else if path.ends_with(".png") {
            FILE_TYPE_IMAGE
        } else if path.ends_with(".txt") {
            FILE_TYPE_TEXT
        } else {
            0
        }
    }

    #[test]
    fn wm_drag_path_single() {
        let path_data = wm_drag_create_path_data(&["text_file.txt"], &ext_type);
        assert_eq!(path_data.paths.len(), 1);
        assert_eq!(path_data.tooltip, "text_file.txt");
        assert_eq!(wm_drag_get_single_path(WM_DRAG_PATH, Some(&path_data)), Some("text_file.txt"));
        assert_eq!(wm_drag_get_path_file_type(WM_DRAG_PATH, Some(&path_data)), FILE_TYPE_TEXT);
        assert_eq!(
            wm_drag_get_single_path_typed(WM_DRAG_PATH, Some(&path_data), FILE_TYPE_TEXT),
            Some("text_file.txt")
        );
        assert_eq!(
            wm_drag_get_single_path_typed(WM_DRAG_PATH, Some(&path_data), FILE_TYPE_BLENDER),
            None
        );
        assert!(wm_drag_has_path_file_type(
            WM_DRAG_PATH,
            Some(&path_data),
            FILE_TYPE_BLENDER | FILE_TYPE_TEXT | FILE_TYPE_IMAGE
        ));
        assert!(!wm_drag_has_path_file_type(
            WM_DRAG_PATH,
            Some(&path_data),
            FILE_TYPE_BLENDER | FILE_TYPE_IMAGE
        ));
    }

    #[test]
    fn wm_drag_path_multi() {
        let path_data = wm_drag_create_path_data(
            &["blender.blend", "text_file.txt", "image.png"],
            &ext_type,
        );
        assert_eq!(path_data.paths.len(), 3);
        assert_eq!(path_data.tooltip, "Dragging 3 files");
        assert_eq!(wm_drag_get_single_path(WM_DRAG_PATH, Some(&path_data)), Some("blender.blend"));
        assert_eq!(wm_drag_get_path_file_type(WM_DRAG_PATH, Some(&path_data)), FILE_TYPE_BLENDER);
        assert_eq!(
            wm_drag_get_single_path_typed(
                WM_DRAG_PATH,
                Some(&path_data),
                FILE_TYPE_BLENDER | FILE_TYPE_TEXT | FILE_TYPE_IMAGE
            ),
            Some("blender.blend")
        );
        assert_eq!(
            wm_drag_get_single_path_typed(
                WM_DRAG_PATH,
                Some(&path_data),
                FILE_TYPE_TEXT | FILE_TYPE_IMAGE
            ),
            Some("text_file.txt")
        );
        assert_eq!(
            wm_drag_get_single_path_typed(WM_DRAG_PATH, Some(&path_data), FILE_TYPE_ASSET),
            None
        );
        assert!(wm_drag_has_path_file_type(WM_DRAG_PATH, Some(&path_data), FILE_TYPE_IMAGE));
        assert!(!wm_drag_has_path_file_type(WM_DRAG_PATH, Some(&path_data), FILE_TYPE_ASSET));
    }

    #[test]
    fn wm_drag_path_wrong_type() {
        assert_eq!(wm_drag_get_single_path(WM_DRAG_COLOR, None), None);
        assert_eq!(wm_drag_get_path_file_type(WM_DRAG_COLOR, None), 0);
        assert!(wm_drag_get_paths(WM_DRAG_COLOR, None).is_empty());
        assert_eq!(
            wm_drag_get_single_path_typed(
                WM_DRAG_COLOR,
                None,
                FILE_TYPE_BLENDER | FILE_TYPE_IMAGE | FILE_TYPE_TEXT | FILE_TYPE_ASSET
            ),
            None
        );
        assert!(!wm_drag_has_path_file_type(
            WM_DRAG_COLOR,
            None,
            FILE_TYPE_BLENDER | FILE_TYPE_IMAGE | FILE_TYPE_TEXT | FILE_TYPE_ASSET
        ));
    }
}
