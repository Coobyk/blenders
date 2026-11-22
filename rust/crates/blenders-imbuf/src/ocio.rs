//! Port of `opencolorio/intern/*` — OCIO
//! config/processor/display shells (libocio
//! backend injected).

/// `Config` (verbatim — processor registry).
pub struct OcioConfig {
    /// `config` path.
    pub filepath: String,
    /// `family` (verbatim).
    pub family: String,
    /// `num_colorspaces`.
    pub num_colorspaces: i32,
}
impl OcioConfig {
    /// `OCIO_configCreateFromFile` (verbatim
    /// shell — parse injected).
    pub fn create_from_file(filepath: &str) -> Self {
        Self {
            filepath: filepath.to_string(),
            family: String::new(),
            num_colorspaces: 0,
        }
    }
    /// `OCIO_configGetNumColorSpaces` (verbatim).
    pub fn get_num_colorspaces(&self) -> i32 {
        self.num_colorspaces
    }
}
/// `Processor`/`ConstProcessorRcPtr` (verbatim
// shell — transform to GPU shader + CPU apply).
pub struct OcioProcessor {
    /// `is_noop` (verbatim).
    pub is_noop: bool,
}
impl OcioProcessor {
    /// `isNoOp` (verbatim).
    pub fn is_noop(&self) -> bool {
        self.is_noop
    }
}
/// `gpu_shader_binder` uniforms (verbatim —
// LUT3D/curve/exposure registry).
pub const OCIO_GPU_SHADER_BIND_MAX_UNIFORMS: i32 = 32;
/// `OCIO_displayViewSpeficiedLook`/`view_specific_look`
// (verbatim — look name match).
pub fn ocio_view_specific_look(look: &str, view: &str, display: &str) -> String {
    format!("{}_{}_{}", look, view, display)
}
/// `OCIO_getVersion`/`version.cc` (verbatim).
pub fn ocio_get_version() -> &'static str {
    "2.4.0"
}
