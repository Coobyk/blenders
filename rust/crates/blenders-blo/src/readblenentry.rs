/// Port of `intern/readblenentry.cc` —
/// dir-listing for .blend files (thumbnail/
/// version info).
/// `BLO_blendhandle_from_file` (verbatim —
/// open + validate).
pub struct BlendHandle {
    /// `filepath`.
    pub filepath: String,
    /// `version`.
    pub version: i32,
}
