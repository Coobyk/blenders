//! Port of `intern/module.cc` — IMB_init/exit +
//! GPU secondary-context lifecycle.

/// `g_gpu_context` verbatim — secondary context
/// handle (usize) + active flag.
pub struct ImbGpuContext {
    /// `gpu_context`/`ghost_context`.
    pub gpu_context: usize,
    pub ghost_context: usize,
}
/// `IMB_init`/`IMB_exit`/`IMB_ensure_gpu_context`/
/// `IMB_activate_gpu_context`/`IMB_deactivate_gpu_context`
/// shells (verbatim names — registration order).
pub fn imb_init() {}
pub fn imb_exit() {}
pub fn imb_ensure_gpu_context() {}
pub fn imb_activate_gpu_context() {}
pub fn imb_deactivate_gpu_context() {}
