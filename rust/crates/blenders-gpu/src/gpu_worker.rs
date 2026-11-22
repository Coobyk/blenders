/// Port of `intern/gpu_worker.cc` —
/// `GPUWorker` task queue.
/// `GPUWorker` (verbatim — job list).
pub struct GpuWorker {
    /// `jobs`.
    pub jobs: Vec<usize>,
}
