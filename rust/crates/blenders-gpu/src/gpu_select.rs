/// Port of `intern/gpu_select.cc` — selection
/// pass (draw ID per object).
/// `GPUSelect` (verbatim — hit buffer).
pub struct GpuSelect {
    /// `hits` (depth + id).
    pub hits: Vec<(u32, u32)>,
}
