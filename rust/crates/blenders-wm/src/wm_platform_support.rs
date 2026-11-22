//! Port of `intern/wm_platform_support.cc` — GPU platform
//! support checks: approval-file read/append, docs link
//! build, dialog decision.

/// `WM_PLATFORM_SUPPORT_TEXT_SIZE` verbatim.
pub const WM_PLATFORM_SUPPORT_TEXT_SIZE: usize = 1024;
/// `BLENDER_PLATFORM_SUPPORT_FILE` verbatim.
pub const BLENDER_PLATFORM_SUPPORT_FILE: &str = ".blender_platform_support";

/// `GPU_SUPPORT_LEVEL_*` verbatim (`GPU_platform.hh`).
pub const GPU_SUPPORT_LEVEL_SUPPORTED: i32 = 0;
pub const GPU_SUPPORT_LEVEL_LIMITED: i32 = 1;
pub const GPU_SUPPORT_LEVEL_UNSUPPORTED: i32 = 2;

/// `wm_platform_support_check_approval` (verbatim — read
/// lines for key; append when missing+update; factory
/// startup / missing cfg → false).
pub fn wm_platform_support_check_approval(
    factory_startup: bool,
    cfgdir: Option<String>,
    platform_support_key: &str,
    update: bool,
    read_lines: &dyn Fn(&str) -> Vec<String>,
    append_line: &mut dyn FnMut(&str, &str),
) -> bool {
    if factory_startup {
        return false;
    }
    let cfgdir = match cfgdir {
        Some(d) => d,
        None => return false,
    };
    let filepath = format!("{}/{}", cfgdir, BLENDER_PLATFORM_SUPPORT_FILE);
    let lines = (read_lines)(&filepath);
    let mut result = false;
    for line in &lines {
        if line == platform_support_key {
            result = true;
            break;
        }
    }
    if !result && update {
        (append_line)(&filepath, platform_support_key);
    }
    result
}

/// `wm_platform_support_create_link` (verbatim — os dir +
/// vendor page).
pub fn wm_platform_support_create_link(
    os: &str,
    gpu_type_matches: &dyn Fn(i32) -> bool,
    gpu_device_intel: i32,
    gpu_device_nvidia: i32,
    gpu_device_ati: i32,
) -> String {
    let mut link = String::from("https://docs.blender.org/manual/en/dev/troubleshooting/gpu/");
    match os {
        "windows" => link.push_str("windows/"),
        "apple" => link.push_str("apple/"),
        _ => link.push_str("linux/"),
    }
    if (gpu_type_matches)(gpu_device_intel) {
        link.push_str("intel.html");
    } else if (gpu_type_matches)(gpu_device_nvidia) {
        link.push_str("nvidia.html");
    } else if (gpu_type_matches)(gpu_device_ati) {
        link.push_str("amd.html");
    } else {
        link.push_str("unknown.html");
    }
    link
}

/// `SupportCheckView` — inputs for
/// `WM_platform_support_perform_checks`.
pub struct SupportCheckView {
    /// `GPU_platform_support_level`.
    pub support_level: i32,
    /// `GPU_platform_support_level_key`.
    pub platform_key: String,
    /// `GPU_platform_gpu_name`.
    pub gpu_name: String,
    /// `GPU_backend_get_type != GPU_BACKEND_NONE`.
    pub backend_detected: bool,
    /// `G.factory_startup`/`G.background`/`G.debug&G_DEBUG`.
    pub factory_startup: bool,
    pub background: bool,
    pub debug: bool,
}

/// `WM_platform_support_perform_checks` (verbatim —
/// approval gate → link + dialog option + message →
/// show/log).
pub fn wm_platform_support_perform_checks(
    view: &SupportCheckView,
    check_approval: &mut dyn FnMut(&str, bool) -> bool,
    create_link: &mut dyn FnMut() -> String,
    clog_info: &mut dyn FnMut(&str),
    show_message_box: &mut dyn FnMut(&str, &str, &str, &str, &str, i32),
) -> bool {
    let mut result = true;
    if view.support_level != GPU_SUPPORT_LEVEL_UNSUPPORTED
        && !view.factory_startup
        && (check_approval)(&view.platform_key, !view.background)
    {
        return result;
    }
    let show_message = view.support_level == GPU_SUPPORT_LEVEL_LIMITED
        || view.support_level == GPU_SUPPORT_LEVEL_UNSUPPORTED;
    let show_continue =
        view.backend_detected && view.support_level != GPU_SUPPORT_LEVEL_UNSUPPORTED;
    let link = if view.backend_detected {
        (create_link)()
    } else {
        String::new()
    };
    let mut dialog_options = 0;
    let mut title = String::new();
    let mut message = String::new();
    match view.support_level {
        GPU_SUPPORT_LEVEL_LIMITED => {
            title = "Blender - Limited Platform Support".to_string();
            message = format!(
                "Your graphics card or driver version has limited support. It may work, \
                 but with issues.\n \nNewer graphics drivers might be available with \
                 better Blender compatibility.\n \nGraphics card:\n{}",
                view.gpu_name
            );
            dialog_options = 1 /*GHOST_DialogWarning*/;
        }
        GPU_SUPPORT_LEVEL_UNSUPPORTED => {
            title = "Blender - Platform Unsupported".to_string();
            message = format!(
                "Your graphics card or driver version is not supported.\n \nNewer \
                 graphics drivers might be available with better Blender compatibility.\n \
                 \nGraphics card:\n{}\n \n",
                view.gpu_name
            );
            if !show_continue {
                message.push_str("Blender will now close.");
                dialog_options = 2 /*GHOST_DialogError*/;
                result = false;
            }
        }
        _ => {}
    }
    if show_message {
        (clog_info)(&format!("{}\n\n{}\n{}\n", title, message, link));
    }
    if view.background {
        result = true;
    } else if show_message {
        (show_message_box)(
            &title,
            &message,
            "Find Latest Drivers",
            if show_continue { "Continue Anyway" } else { "Exit" },
            &link,
            dialog_options,
        );
    }
    result
}
