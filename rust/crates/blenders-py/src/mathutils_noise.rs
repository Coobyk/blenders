/// Port of `mathutils_noise.cc` —
/// `mathutils.noise` (perlin/simplex/worley/
/// turbulence/hetero_terrain/fractal).
/// `noise_random` (verbatim — 0..1 hash).
pub fn noise_random(x: i32, y: i32, z: i32) -> f32 {
    blenders_bli::noise::hash2(
        (x as u32).wrapping_mul(374761393)
            .wrapping_add((y as u32).wrapping_mul(668265263))
            .wrapping_add((z as u32).wrapping_mul(2147483647)),
        0,
    ) as f32 / u32::MAX as f32
}
