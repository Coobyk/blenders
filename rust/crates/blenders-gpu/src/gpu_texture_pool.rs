/// Port of `intern/gpu_texture_pool.cc` —
/// texture freelist per (fmt,size,flags).
/// `GPUTexturePool` (verbatim — pool map).
pub struct GpuTexturePool {
    /// `free` textures by key.
    pub free: std::collections::HashMap<(i32, i32, i32), Vec<usize>>,
}
impl GpuTexturePool {
    /// `texture_acquire` (verbatim — pop or
    /// alloc).
    pub fn acquire(&mut self, fmt: i32, w: i32, h: i32) -> Option<usize> {
        self.free.get_mut(&(fmt, w, h)).and_then(|v| v.pop())
    }
    /// `texture_release` (verbatim — push).
    pub fn release(&mut self, tex: usize, fmt: i32, w: i32, h: i32) {
        self.free.entry((fmt, w, h)).or_default().push(tex);
    }
}
