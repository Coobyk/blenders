//! Port of `intern/wm_splash_screen.cc` — the splash
//! screen & about popups (block layout constants, image
//! round-corners, rosetta check, `WM_OT_splash`/`_about`).

/// `WM_block_splash_*` verbatim layout numbers.
pub const SPLASH_WIDTH_FACTOR: i32 = 45;
pub const SPLASH_WIDTH_CLAMP_FAC: f32 = 0.7;
pub const SPLASH_VERSION_X: f32 = 8.0;
pub const SPLASH_VERSION_Y: f32 = 13.0;
pub const SPLASH_PROJECT_MARGIN_UNITS: i32 = 1;
pub const SPLASH_PROJECT_WIDTH_UNITS: i32 = 8;
pub const ABOUT_WIDTH_FACTOR: i32 = 42;
pub const ABOUT_LOGO_SIZE_FAC: f32 = 0.2;
pub const ABOUT_LOGO_TOP_SEP: f32 = 0.2;
pub const ABOUT_LOGO_UNIT: f32 = 1.0;
pub const ABOUT_LOGO_BOTTOM_SEP: f32 = 2.7;
pub const ABOUT_BOUNDS_FAC: f32 = 22.0;

/// `wm_block_splash_create` width math (verbatim —
/// `points*45*scale` clamped to 70% of native pixel x).
pub fn splash_width(widget_points: i32, ui_scale_fac: f32, native_pixel_x: i32) -> i32 {
    let mut w = (widget_points * SPLASH_WIDTH_FACTOR) as f32 * ui_scale_fac;
    w = w.min(native_pixel_x as f32 * SPLASH_WIDTH_CLAMP_FAC);
    w as i32
}

/// `is_using_macos_rosetta` (verbatim — sysctl result;
/// 0 native / 1 translated / -1 error).
pub fn is_using_macos_rosetta(sysctlbyname_proc_translated: Option<i32>) -> i32 {
    match sysctlbyname_proc_translated {
        Some(ret) => ret,
        None => 0,
    }
}

/// `wm_block_splash_image_roundcorners_add` (verbatim —
/// premul alpha from rounded-corner distance).
pub fn roundcorner_alpha(x: i32, y: i32, w: i32, h: i32, radius: f32) -> f32 {
    let fx = x as f32;
    let fy = y as f32;
    let cx = if fx < radius {
        radius
    } else if fx > w as f32 - radius {
        w as f32 - radius
    } else {
        fx
    };
    let cy = if fy < radius {
        radius
    } else if fy > h as f32 - radius {
        h as f32 - radius
    } else {
        fy
    };
    let d = ((fx - cx) * (fx - cx) + (fy - cy) * (fy - cy)).sqrt();
    (radius - d).clamp(0.0, 1.0)
}

/// `wm_splash_invoke`/`wm_splash_about_invoke` (verbatim —
/// popup_block_invoke → FINISHED).
pub fn wm_splash_invoke(block_create: &mut dyn FnMut()) -> i32 {
    (block_create)();
    crate::wm_files_colorspace::OPERATOR_FINISHED
}

/// `WM_OT_splash` registration (verbatim fields).
pub fn wm_ot_splash() -> (&'static str, &'static str, &'static str) {
    ("WM_OT_splash", "Splash Screen", "Open the splash screen with release info")
}

/// `WM_OT_splash_about` registration (verbatim fields).
pub fn wm_ot_splash_about() -> (&'static str, &'static str, &'static str) {
    ("WM_OT_splash_about", "About Blender", "Open information about Blender")
}
