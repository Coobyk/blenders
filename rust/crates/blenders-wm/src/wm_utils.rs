//! Port of `intern/wm_utils.cc` — generic callback and
//! user-data lifetime helpers.

/// `wmGenericCallback` verbatim (`WM_types.h`).
pub struct WmGenericCallback {
    /// `exec` callback.
    pub exec: usize,
    /// `free_user_data` callback (0 = none).
    pub free_user_data: usize,
    /// `user_data`.
    pub user_data: usize,
}

/// `WM_generic_callback_free` (verbatim).
pub fn wm_generic_callback_free(callback: &mut WmGenericCallback) {
    if callback.free_user_data != 0 {
        /* `callback->free_user_data(callback->user_data)` */
        let _ = callback.user_data;
    }
    /* `MEM_delete(callback)` — Rust drop. */
}

/// `do_nothing` (verbatim — steal target).
pub fn do_nothing() {}

/// `WM_generic_callback_steal` (verbatim — clone + neuter
/// original so the move owns the callbacks).
pub fn wm_generic_callback_steal(callback: &mut WmGenericCallback) -> WmGenericCallback {
    let new_callback = WmGenericCallback {
        exec: callback.exec,
        free_user_data: callback.free_user_data,
        user_data: callback.user_data,
    };
    callback.exec = do_nothing as usize;
    callback.free_user_data = 0;
    callback.user_data = 0;
    new_callback
}

/// `wmGenericUserData` verbatim (`WM_types.h`).
pub struct WmGenericUserData {
    /// `data`.
    pub data: usize,
    /// `use_free`.
    pub use_free: bool,
    /// `free_fn` (0 = default delete).
    pub free_fn: usize,
}

/// `WM_generic_user_data_free` (verbatim).
pub fn wm_generic_user_data_free(wm_userdata: &mut WmGenericUserData) {
    if wm_userdata.data != 0 && wm_userdata.use_free {
        /* `free_fn(data)` or `MEM_delete_void` — caller
         * side frees via injected fn. */
        let _ = wm_userdata.free_fn;
        wm_userdata.data = 0;
    }
}
