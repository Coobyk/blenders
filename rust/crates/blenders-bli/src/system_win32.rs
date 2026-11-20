//! Port of `system_win32.cc` — Windows crash-handler / backtrace subsystem.
//!
//! PORT_PENDING: `_WIN32`-only upstream (DbgHelp stack walks, minidump dialog).
//! Non-Windows targets get no-op stubs; `BLI_system_backtrace_with_os_info`
//! prints nothing on POSIX (upstream's POSIX backtrace lives in system.cc).

/// `BLI_system_backtrace_with_os_info` — Windows SEH backtrace; no-op on POSIX.
pub fn system_backtrace_with_os_info(_os_info: *const std::ffi::c_void) {}

/// `BLI_windows_exception_print_message`.
pub fn windows_exception_print_message(_os_info: *const std::ffi::c_void) {}

/// `BLI_windows_exception_show_dialog` — returns the crash-log path it used, if any.
pub fn windows_exception_show_dialog(
    _filepath_crashlog: &str,
    _os_info: *const std::ffi::c_void,
) -> Option<String> {
    None
}
