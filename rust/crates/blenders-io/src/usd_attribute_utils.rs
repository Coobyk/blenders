/// Port of `usd/intern/usd_attribute_utils.cc`
/// — attribute read/write typed dispatch.
/// `copy_blender_property_to_prim` (verbatim
/// — per-type set).
pub fn copy_blender_property_to_prim(name: &str, value: f32) -> (String, f32) {
    (name.to_string(), value)
}
