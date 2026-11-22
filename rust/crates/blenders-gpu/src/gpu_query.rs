/// Port of `intern/gpu_query.cc` — GPUQuery
/// (occlusion/time-elapsed wrappers).
/// `GPUQuery` (verbatim — backend handle +
/// type).
pub struct GpuQuery {
    /// `type` (occlusion/time).
    pub query_type: i32,
    /// `result`.
    pub result: u64,
}
