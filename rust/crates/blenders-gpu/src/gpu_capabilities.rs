/// Port of `intern/gpu_capabilities.cc` —
/// `GCaps` query API.
/// `GPUCapabilities` (verbatim — backend-
/// reported limits).
#[derive(Default)]
pub struct GpuCapabilities {
    /// `max_texture_size`.
    pub max_texture_size: i32,
    /// `max_texture_3d_size`.
    pub max_texture_3d_size: i32,
    /// `max_buffer_texture_size`.
    pub max_buffer_texture_size: u32,
    /// `max_texture_layers`.
    pub max_texture_layers: i32,
    /// `max_ubos`.
    pub max_ubos: i32,
    /// `max_ssbos`.
    pub max_ssbos: i32,
    /// `max_shader_storage_blocks`.
    pub max_shader_storage_blocks: i32,
    /// `max_work_group_count`[3].
    pub max_work_group_count: [i32; 3],
    /// `max_work_group_size`[3].
    pub max_work_group_size: [i32; 3],
    /// `mem_stats`.
    pub mem_stats_support: bool,
    /// `shader_image_load_store`.
    pub shader_image_load_store: bool,
}
/// `GPU_max_texture_size` (verbatim).
pub fn gpu_max_texture_size(caps: &GpuCapabilities) -> i32 {
    caps.max_texture_size
}
/// `GPU_texture_size_with_limit` (verbatim —
/// min(res, limit)).
pub fn gpu_texture_size_with_limit(caps: &GpuCapabilities, res: i32, glreslimit: i32) -> i32 {
    let size = caps.max_texture_size;
    let reslimit = if glreslimit != 0 { glreslimit.min(size) } else { size };
    reslimit.min(res)
}
/// `GPU_is_safe_texture_size` (verbatim —
/// w*h ≤ max²/4).
pub fn gpu_is_safe_texture_size(caps: &GpuCapabilities, width: i32, height: i32) -> bool {
    let max = caps.max_texture_size as i64;
    (width as i64 * height as i64) <= max * max / 4
}
