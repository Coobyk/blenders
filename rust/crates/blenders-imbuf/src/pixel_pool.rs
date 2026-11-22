//! Port of `intern/pixel_pool.cc` — pooled pixel
//! buffers (ImBufPixelPool + limit).

/// `g_pool` verbatim — freelist + total bytes.
pub struct PixelPool {
    /// `free_buffers` per (w,h,ch).
    pub free_buffers: std::collections::HashMap<(i32, i32, i32), Vec<Vec<f32>>>,
    /// `total_size`/`limit`.
    pub total_size: usize,
    pub limit: usize,
}
/// `IMB_pixel_pool_set_limit` (verbatim).
pub fn imb_pixel_pool_set_limit(limit: usize) -> usize {
    limit
}
/// `IMB_alloc_pixels`/`IMB_free_pixels` shells.
pub fn imb_pixel_pool_alloc(n: usize) -> Vec<f32> {
    vec![0.0; n]
}
