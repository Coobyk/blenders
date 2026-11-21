//! Port of `intern/view3d_runtime.cc`.
//!
//! `RegionView3DRuntime` is a plain-runtime struct; `region_view3d_lock_flags_get`
//! is ported with the runtime's viewlock passed in (the runtime pointer is a
//! `usize` in the DNA port).

/// `RegionView3DRuntime` (BKE_view3d_runtime.hh) — view lock + eval log slot.
#[derive(Default)]
pub struct RegionView3DRuntime {
    /// `eRegionView3D_ViewLock viewlock`.
    pub viewlock: i32,
    /// `nodes::geo_eval_log::GeoModifierLog *eval_log` — PORT_PENDING.
    pub eval_log: usize,
}

/// `region_view3d_lock_flags_get(rv3d)` — `rv3d->viewlock | rv3d->runtime->viewlock`.
/// Pass `rv3d.runtime`'s viewlock as `runtime_viewlock`.
pub fn region_view3d_lock_flags_get(viewlock: i32, runtime_viewlock: i32) -> i32 {
    viewlock | runtime_viewlock
}
