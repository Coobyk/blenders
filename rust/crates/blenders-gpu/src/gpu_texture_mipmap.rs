/// Port of `intern/gpu_texture_mipmap.cc`
/// — mip-level gen + state.
/// `GPUTexture::mipmaps_get` (verbatim —
/// level count for size).
pub fn gpu_texture_mipmaps_get(w: i32, h: i32, d: i32) -> i32 {
    let max = w.max(h).max(d);
    31 - (max.max(1) as u32).leading_zeros() as i32 + 1
}
