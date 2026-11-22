/// Port of `intern/gpu_work_in_flight.cc` —
/// `GPUWorkInFlight` fence tracking.
/// `GPUWorkInFlight` (verbatim — fence id).
pub struct GpuWorkInFlight {
    /// `fence`.
    pub fence: u64,
}
