//! Port of `winstuff.cc` + `winstuff_dir.cc` + `winstuff_registration.cc` —
//! Windows-only helpers (registry, file associations, store-install detection,
//! QoS, dirent shim).
//!
//! PORT_PENDING: upstream compiles these only under `_WIN32`. The Rust port
//! targets Unix; every function is `#[cfg(windows)]`-pending and the non-Windows
//! build exposes `false`/`None` stubs matching the POSIX fallbacks upstream
//! provides elsewhere (e.g. `BLI_file_external_operation_*` -> false).

/// `BLI_windows_get_executable_dir`.
#[cfg(windows)]
pub fn windows_get_executable_dir() -> Option<String> {
    None // PORT_PENDING: GetModuleFileNameW.
}
#[cfg(not(windows))]
pub fn windows_get_executable_dir() -> Option<String> {
    None
}

/// `BLI_windows_is_store_install`.
pub fn windows_is_store_install() -> bool {
    false
}

/// `BLI_windows_register_blend_extension` / `BLI_windows_unregister_blend_extension`.
pub fn windows_register_blend_extension(_all_users: bool) -> bool {
    cfg!(windows) && false
}
pub fn windows_unregister_blend_extension(_all_users: bool) -> bool {
    cfg!(windows) && false
}

/// `BLI_windows_external_operation_supported` / `_execute` — registry-backed on Windows.
pub fn windows_external_operation_supported(_filepath: &str, _operation: &str) -> bool {
    false
}
pub fn windows_external_operation_execute(_filepath: &str, _operation: &str) -> bool {
    false
}

/// `BLI_windows_execute_self` — ShellExecuteEx "runas". Returns true on success.
pub fn windows_execute_self(
    _parameters: &str,
    _run_as_admin: bool,
    _wait: bool,
    _detect_microsoft_terminal: bool,
    _minimize_console: bool,
) -> bool {
    false
}

/// `BLI_windows_get_default_root_dir` — e.g. "C:\\". POSIX -> "/".
pub fn windows_get_default_root_dir() -> &'static str {
    #[cfg(windows)]
    {
        "C:\\"
    }
    #[cfg(not(windows))]
    {
        "/"
    }
}

/// `BLI_windows_get_directx_driver_version` / `_intel_driver_info`.
pub fn windows_get_directx_driver_version(_device_substring: &[u16]) -> Option<(u32, u32, u32, u32)> {
    None
}
pub fn windows_get_directx_intel_driver_info(_device_luid: u64) -> Option<(u32, u32, u32)> {
    None
}

/// `BLI_windows_is_build_version_greater_or_equal`.
pub fn windows_is_build_version_greater_or_equal(_major: u32, _minor: u32, _build: u32) -> bool {
    false
}

/// `QoSMode` / `QoSPrecedence` / `BLI_windows_process_set_qos`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QoSMode {
    Background,
    Utility,
    Multimedia,
    Audio,
    High,
    Display,
    Realtime,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QoSPrecedence {
    Normal,
    High,
}
pub fn windows_process_set_qos(_qos_mode: QoSMode, _qos_precedence: QoSPrecedence) {}

/// `BLI_windows_update_pinned_launcher` (winstuff_registration.cc).
pub fn windows_update_pinned_launcher(_launcher_path: &str) -> bool {
    false
}
