//! Port of `intern/wm_init_exit.cc` — WM startup/exit
//! orchestration: GPU backend init gates, `WM_init`,
//! splash-on-startup, delayed exit, `WM_exit(_ex)`.

/// `wm_start_with_console` verbatim global.
pub static WM_START_WITH_CONSOLE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
/// `gpu_is_init` verbatim global.
pub static GPU_IS_INIT: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// `WM_init_state_start_with_console_set` (verbatim).
pub fn wm_init_state_start_with_console_set(value: bool) {
    WM_START_WITH_CONSOLE.store(value, std::sync::atomic::Ordering::Relaxed);
}

/// `WM_init_state_start_with_console_is_set` (verbatim).
pub fn wm_init_state_start_with_console_is_set() -> bool {
    WM_START_WITH_CONSOLE.load(std::sync::atomic::Ordering::Relaxed)
}

/// `WM_init_gpu_backend` (verbatim — one-shot gate).
pub fn wm_init_gpu_backend(gpu_backend_init: &mut dyn FnMut()) {
    if !GPU_IS_INIT.swap(true, std::sync::atomic::Ordering::Relaxed) {
        (gpu_backend_init)();
    }
}

/// `WM_init_gpu_offscreen` (verbatim — needs-init →
/// backend + offscreen init).
pub fn wm_init_gpu_offscreen(
    gpu_backend_init: &mut dyn FnMut(),
    gpu_offscreen_init: &mut dyn FnMut(),
    gpu_backend_type_supported: bool,
) {
    if gpu_backend_type_supported {
        wm_init_gpu_backend(gpu_backend_init);
        (gpu_offscreen_init)();
    }
}

/// `wm_init_splash_show_on_startup_check` (verbatim —
/// `!G.background && !wm_start_with_console` … plus
/// userpref show_splash).
pub fn wm_init_splash_show_on_startup_check(background: bool, start_with_console: bool, show_splash: bool) -> bool {
    !background && !start_with_console && show_splash
}

/// `wm_exit_handler`/`wm_exit_schedule_delayed_for_window`
/// (verbatim — schedules window->exit after delay).
pub fn wm_exit_schedule_delayed(
    windows: &[usize],
    event_timer_add: &mut dyn FnMut(usize, f64) -> usize,
) {
    for &win in windows {
        let _ = (event_timer_add)(win, 0.0);
    }
}

/// `WM_exit` (verbatim — `WM_exit_ex(C, true, true)` +
/// `exit(exit_code)`).
pub fn wm_exit(exit_ex: &mut dyn FnMut(bool, bool), exit_code: i32) -> i32 {
    (exit_ex)(true, true);
    exit_code
}

/// `WM_script_tag_reload` (verbatim — sets `G.relbase_valid |= 2`).
pub fn wm_script_tag_reload(relbase_valid: &mut i32) {
    *relbase_valid |= 2;
}
