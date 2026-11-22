//! Port of `intern/proxy_job.cc` — proxy build
//! job metadata.

/// `eTaskPriority` verbatim ordinals.
pub const JOB_PRIORITY_HIGH: i32 = 0;
pub const JOB_PRIORITY_NORMAL: i32 = 1;
pub const JOB_PRIORITY_LOW: i32 = 2;
/// `ProxyJob` verbatim fields.
#[derive(Default)]
pub struct ProxyJob {
    /// `stop`/`do_update`/`progress` flags.
    pub stop: bool,
    pub do_update: bool,
    /// `progress`.
    pub progress: f32,
    /// `priority`.
    pub priority: i32,
}
