/// Port of `bpy_app.cc` — `bpy.app`
/// (version/build info/handlers).
/// `bpy_app` fields (verbatim).
pub struct BpyApp {
    /// `version` tuple.
    pub version: (i32, i32, i32),
    /// `version_string`.
    pub version_string: String,
    /// `binary_path`.
    pub binary_path: String,
    /// `background`.
    pub background: bool,
    /// `factory_startup`.
    pub factory_startup: bool,
    /// `debug_value`.
    pub debug_value: i32,
}
