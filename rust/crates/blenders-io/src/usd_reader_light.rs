/// Port of `usd/intern/usd_reader_light.cc`
/// — Lux light → Light data.
/// `read_light` (verbatim — type token).
pub fn usd_read_light(light_type: &str) -> i32 {
    match light_type {
        "SunLight" | "DistantLight" => 1,
        "CylinderLight" => 2,
        "RectLight" => 3,
        _ => 0,
    }
}
