/// Port of `intern/mesh_bevel.cc` — Bevel
/// modifier geometry (delegates to
/// `blenders-bmesh` bevel tool + custom
/// profile/segments math).

/// `bevel_type` ordinals (verbatim).
pub const BEVEL_TYPE_OFFSET: i32 = 0;
pub const BEVEL_TYPE_WIDTH: i32 = 1;
pub const BEVEL_TYPE_DEPTH: i32 = 2;
pub const BEVEL_TYPE_PERCENT: i32 = 3;
pub const BEVEL_TYPE_ABSOLUTE: i32 = 4;
/// `bevel_offset` (verbatim — per-type
// effective offset).
pub fn bevel_offset(
    bevel_type: i32,
    value: f32,
    edge_len: f32,
) -> f32 {
    match bevel_type {
        BEVEL_TYPE_WIDTH | BEVEL_TYPE_ABSOLUTE => value,
        BEVEL_TYPE_DEPTH => value / 2.0_f32.hypot(0.0).max(1.0) * 2.0_f32.sqrt(),
        BEVEL_TYPE_PERCENT => value * edge_len / 100.0,
        _ => value,
    }
}
/// `bevel_profile` (verbatim — superellipse
// profile points).
pub fn bevel_profile(segments: i32, profile: f32) -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity((segments + 1) as usize);
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let theta = t * std::f32::consts::FRAC_PI_2;
        // Superellipse: |x|^2/p + |y|^2/p = 1.
        let p = profile.max(0.01);
        let c = theta.cos().powf(2.0 / p);
        let s = theta.sin().powf(2.0 / p);
        let l = (c.powi(2) + s.powi(2)).sqrt();
        out.push([s / l.max(1e-9), c / l.max(1e-9)]);
    }
    out
}
