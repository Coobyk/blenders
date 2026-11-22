//! Port of `intern/strip_add.cc` — LoadData +
//! add_*_strip constructors.

/// `SEQ_LOAD_SET_VIEW_TRANSFORM` verbatim.
pub const SEQ_LOAD_SET_VIEW_TRANSFORM: i32 = 1 << 0;
/// `SEQ_LOAD_SOUND_CACHE`/`SEQ_LOAD_SOUND_MONO`.
pub const SEQ_LOAD_SOUND_CACHE: i32 = 1 << 1;
pub const SEQ_LOAD_SOUND_MONO: i32 = 1 << 2;
/// `LoadData` verbatim fields.
#[derive(Clone, Default)]
pub struct LoadData {
    /// `name`/`path`.
    pub name: String,
    pub path: String,
    /// `start_frame`/`channel`/`end_frame`/`flags`.
    pub start_frame: i32,
    pub channel: i32,
    pub end_frame: i32,
    pub flags: i32,
    /// `scene`/`clip`/`mask` handles.
    pub scene: usize,
    pub clip: usize,
    pub mask: usize,
    /// `load_method`/`fit_method`/`set_view_transform`.
    pub load_method: i32,
    pub fit_method: i32,
    /// `strips`/`images`/`views` use arrays.
    pub allow_duplicate_file: bool,
}
/// `add_load_data_init` (verbatim).
pub fn add_load_data_init(name: &str, path: &str, start_frame: i32, channel: i32) -> LoadData {
    LoadData {
        name: name.to_string(),
        path: path.to_string(),
        start_frame,
        channel,
        ..Default::default()
    }
}
/// `default_strip_length` (verbatim — U.sequencer_
/// default_strip_length * fps, min 1).
pub fn default_strip_length(sequencer_default_strip_length: f64, scene_fps: f64) -> i32 {
    (sequencer_default_strip_length * scene_fps).round().max(1.0) as i32
}
/// `strip_add_set_name` (verbatim — override name
/// else id name / type name / "Image").
pub fn strip_add_set_name(
    load_name: &str,
    strip_type: i32,
    is_effect: bool,
    scene_name: &str,
    clip_name: &str,
    mask_name: &str,
) -> String {
    if !load_name.is_empty() {
        return crate::strip_edit::edit_strip_name_set(load_name);
    }
    match strip_type {
        crate::utils::STRIP_TYPE_SCENE => crate::strip_edit::edit_strip_name_set(scene_name),
        crate::utils::STRIP_TYPE_MOVIECLIP => crate::strip_edit::edit_strip_name_set(clip_name),
        crate::utils::STRIP_TYPE_MASK => crate::strip_edit::edit_strip_name_set(mask_name),
        _ if is_effect => crate::strip_edit::edit_strip_name_set(
            &crate::utils::strip_give_name(strip_type, is_effect, ""),
        ),
        _ => crate::strip_edit::edit_strip_name_set(load_name),
    }
}
/// `strip_add_set_view_transform` (verbatim — when
/// flag + byte colorspace → default view transform).
pub fn strip_add_set_view_transform(
    flags: i32,
    strip_colorspace: &str,
    role_colorspace_byte: &str,
    default_view_transform: &str,
) -> Option<String> {
    if (flags & SEQ_LOAD_SET_VIEW_TRANSFORM) != 0 && strip_colorspace == role_colorspace_byte {
        return Some(default_view_transform.to_string());
    }
    None
}
