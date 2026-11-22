/// Port of `usd/intern/usd_light_convert.cc`
/// — Blender light → USD Lux tokens.
/// `light_to_usd_type` (verbatim — POINT/
/// SPOT/SUN/AREA).
pub fn usd_light_to_type(light_type: i32) -> &'static str {
    match light_type {
        0 => "SphereLight",
        1 => "SunLight",
        2 => "CylinderLight",
        3 => "RectLight",
        _ => "SphereLight",
    }
}
