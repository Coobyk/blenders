/// Port of `intern/randomize.cc` — Randomize
/// Geometry (per-point random offset/rotation/
/// scale).
use blenders_bli::noise::hash2;
/// `randomize_positions` (verbatim — sample
/// uniform field per index).
pub fn randomize_positions(
    positions: &mut [[f32; 3]],
    delta_min: [f32; 3],
    delta_max: [f32; 3],
    seed: i32,
) {
    for (i, p) in positions.iter_mut().enumerate() {
        for c in 0..3 {
            let t = hash2(seed as u32, (i * 3 + c) as u32) as f32 / u32::MAX as f32;
            p[c] += delta_min[c] + (delta_max[c] - delta_min[c]) * t;
        }
    }
}
/// `randomize_scale` (verbatim — per-point
/// uniform scale about center).
pub fn randomize_scale(
    positions: &mut [[f32; 3]],
    center: [f32; 3],
    scale_min: f32,
    scale_max: f32,
    seed: i32,
) {
    for (i, p) in positions.iter_mut().enumerate() {
        let t = hash2(seed as u32, i as u32) as f32 / u32::MAX as f32;
        let s = scale_min + (scale_max - scale_min) * t;
        for c in 0..3 {
            p[c] = center[c] + (p[c] - center[c]) * s;
        }
    }
}
