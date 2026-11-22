//! Port of `intern/render_context.cc` — RenderContext
//! + FileOutput (render-result backing for file
//! outputs).

/// `R_IMF_*` verbatim.
pub const R_IMF_FLAG_PREVIEW_JPG: i32 = 1;
pub const R_IMF_IMTYPE_OPENEXR: i32 = 8;
pub const R_IMF_IMTYPE_MULTILAYER: i32 = 22;
/// `ImColorMode` verbatim ordinals.
pub const IM_COLOR_MODE_BW: i32 = 0;
pub const IM_COLOR_MODE_RGB: i32 = 1;
pub const IM_COLOR_MODE_RGBA: i32 = 2;
/// `ImageFormatData` verbatim subset.
#[derive(Clone, Default)]
pub struct ImageFormatData {
    /// `imtype`.
    pub imtype: i32,
    /// `flag`.
    pub flag: i32,
}
/// `FileOutput` verbatim fields.
#[derive(Default)]
pub struct FileOutput {
    /// `path_`.
    pub path: String,
    /// `format_`.
    pub format: ImageFormatData,
    /// `save_as_render_`.
    pub save_as_render: bool,
    /// `render_result_` (rect + layer/view lists).
    pub rectx: i32,
    pub recty: i32,
    /// `meta_data_`.
    pub meta_data: Vec<(String, String)>,
    /// `views`/`layers`/`passes` handles.
    pub views: Vec<String>,
    pub passes: Vec<String>,
}
impl FileOutput {
    /// `FileOutput` ctor (verbatim — size, dummy ppm,
    /// single layer, no preview jpg).
    pub fn new(path: &str, format: ImageFormatData, size: [i32; 2], save_as_render: bool) -> Self {
        let mut f = FileOutput {
            path: path.to_string(),
            format,
            save_as_render,
            rectx: size[0],
            recty: size[1],
            ..Default::default()
        };
        f.format.flag &= !R_IMF_FLAG_PREVIEW_JPG;
        f
    }
    /// `add_view` (verbatim — EXR only for empty
    /// views; color mode by channels).
    pub fn add_view(&mut self, view_name: &str) {
        debug_assert!(matches!(self.format.imtype, R_IMF_IMTYPE_OPENEXR | R_IMF_IMTYPE_MULTILAYER) || self.format.imtype == 0);
        self.views.push(view_name.to_string());
    }
    /// `add_meta_data` (verbatim).
    pub fn add_meta_data(&mut self, key: &str, value: &str) {
        self.meta_data.push((key.to_string(), value.to_string()));
    }
    /// `color_mode_from_channels` (verbatim — 1→BW,
    /// 3→RGB, else RGBA).
    pub fn color_mode_from_channels(channels: i64) -> i32 {
        match channels {
            1 => IM_COLOR_MODE_BW,
            3 => IM_COLOR_MODE_RGB,
            _ => IM_COLOR_MODE_RGBA,
        }
    }
}
/// `RenderContext` verbatim — `file_outputs_` map.
#[derive(Default)]
pub struct RenderContext {
    /// `file_outputs_` by path.
    pub file_outputs: std::collections::HashMap<String, FileOutput>,
}
impl RenderContext {
    /// `get_file_output` (verbatim — lookup-or-add).
    pub fn get_file_output(
        &mut self,
        path: &str,
        format: ImageFormatData,
        size: [i32; 2],
        save_as_render: bool,
    ) -> &mut FileOutput {
        self.file_outputs
            .entry(path.to_string())
            .or_insert_with(|| FileOutput::new(path, format, size, save_as_render))
    }
}
