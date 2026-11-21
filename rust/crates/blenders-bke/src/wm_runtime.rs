//! Port of `intern/wm_runtime.cc` — `WindowManagerRuntime`/`WindowRuntime`.
//!
//! `BKE_reports_init`/`BKE_reports_clear` need the report system — PORT_PENDING;
//! the struct is real so `wmWindowManager.runtime` has a home.

/// `WindowManagerRuntime` (BKE_wm_runtime.hh).
#[derive(Default)]
pub struct WindowManagerRuntime {
    /// `ReportList reports` — PORT_PENDING (BKE_report).
    pub reports: usize,
    /// `wm::ReportList` lock-free queue + message store handles.
    pub report_text_lock_free_queue: usize,
    pub saved_reports: usize,
}

/// `WindowRuntime` (BKE_wm_runtime.hh).
#[derive(Default)]
pub struct WindowRuntime {
    /// `Gesture` modal gesture tracker — PORT_PENDING.
    pub gesture: usize,
    /// `char eventstate` etc. handled on the DNA side.
    pub eventstate: usize,
}
