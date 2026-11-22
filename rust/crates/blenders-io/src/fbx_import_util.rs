/// Port of `fbx/importer/fbx_import_util.cc`
/// — FBX unit/axis conv.
/// `fbx_convert_scale` (verbatim — cm→m).
pub fn fbx_convert_scale(unit_scale_cm: f32) -> f32 {
    unit_scale_cm / 100.0
}
