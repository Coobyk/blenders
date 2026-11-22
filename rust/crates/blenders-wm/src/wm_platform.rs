//! Port of `intern/wm_platform.cc` —
//! `WM_platform_associate_set` (.blend file association:
//! Win32 registry calls, macOS noop, Linux freedesktop via
//! `_bpy_internal.platform.freedesktop`).

/// `WM_platform_associate_set` (verbatim — platform
/// dispatch; r_error_msg out).
pub fn wm_platform_associate_set(
    do_register: bool,
    all_users: bool,
    r_error_msg: &mut Option<String>,
    windows_execute_self: &mut dyn FnMut(&str) -> bool,
    windows_register_blend_extension: &mut dyn FnMut(bool) -> bool,
    windows_unregister_blend_extension: &mut dyn FnMut(bool) -> bool,
    run_freedesktop: &mut dyn FnMut(bool, bool) -> Result<Option<String>, String>,
    is_windows: bool,
    is_macos: bool,
) -> bool {
    let mut result = false;
    *r_error_msg = None;
    if is_windows {
        if all_users {
            if do_register {
                result = (windows_execute_self)("--register-allusers");
            } else {
                result = (windows_execute_self)("--unregister-allusers");
            }
        } else if do_register {
            result = (windows_register_blend_extension)(false);
        } else {
            result = (windows_unregister_blend_extension)(false);
        }
    } else if is_macos {
        /* Pass. */
        let _ = (do_register, all_users);
    } else {
        /* freedesktop register/unregister via bpy. */
        match (run_freedesktop)(do_register, all_users) {
            Ok(v) => {
                result = v.is_none();
                *r_error_msg = v;
            }
            Err(e) => {
                *r_error_msg = Some(e);
            }
        }
    }
    result
}
