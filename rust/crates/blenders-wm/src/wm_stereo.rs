//! Port of `intern/wm_stereo.cc` — stereo-3d display
//! formats (sidebyside/topbottom/anaglyph/interlace
//! draws), enabled checks, mouse offset, `WM_OT_stereo3d`
//! operator helpers.

/// `S3D_DISPLAY_*` verbatim (`DNA_scene_types.h`).
pub const S3D_DISPLAY_ANAGLYPH: i32 = 0;
pub const S3D_DISPLAY_INTERLACE: i32 = 1;
pub const S3D_DISPLAY_SIDEBYSIDE: i32 = 2;
pub const S3D_DISPLAY_TOPBOTTOM: i32 = 3;
pub const S3D_DISPLAY_PAGEFLIP: i32 = 4;
/// `S3D_SIDEBYSIDE_CROSSEYED`/`S3D_INTERLACE_SWAP`.
pub const S3D_SIDEBYSIDE_CROSSEYED: i32 = 1 << 0;
pub const S3D_INTERLACE_SWAP: i32 = 1 << 1;
/// `STEREO_LEFT_ID`/`RIGHT`.
pub const STEREO_LEFT_ID: i32 = 0;

/// `wm_stereo3d_draw_sidebyside` quad math (verbatim —
/// soffx by view/cross-eyed + half-pixel uv offset).
pub fn wm_stereo3d_draw_sidebyside_verts(
    flag: i32,
    view: i32,
    win_size: (i32, i32),
) -> (i32, [((f32, f32), (f32, f32)); 4]) {
    let cross_eyed = (flag & S3D_SIDEBYSIDE_CROSSEYED) != 0;
    let mut soffx = win_size.0 / 2;
    if view == STEREO_LEFT_ID {
        if !cross_eyed {
            soffx = 0;
        }
    } else if cross_eyed {
        soffx = 0;
    }
    let halfx = crate::wm_subwindow::GLA_PIXEL_OFS / win_size.0 as f32;
    let halfy = crate::wm_subwindow::GLA_PIXEL_OFS / win_size.1 as f32;
    let verts = [
        ((halfx, halfy), (soffx as f32, 0.0)),
        ((1.0 + halfx, halfy), (soffx as f32 + win_size.0 as f32 * 0.5, 0.0)),
        (
            (1.0 + halfx, 1.0 + halfy),
            (soffx as f32 + win_size.0 as f32 * 0.5, win_size.1 as f32),
        ),
        ((halfx, 1.0 + halfy), (soffx as f32, win_size.1 as f32)),
    ];
    (soffx, verts)
}

/// `wm_stereo3d_draw_topbottom` quad math (verbatim).
pub fn wm_stereo3d_draw_topbottom_verts(
    view: i32,
    win_size: (i32, i32),
) -> (i32, [((f32, f32), (f32, f32)); 4]) {
    let mut soffy = win_size.1 / 2;
    if view == STEREO_LEFT_ID {
        soffy = 0;
    }
    let halfx = crate::wm_subwindow::GLA_PIXEL_OFS / win_size.0 as f32;
    let halfy = crate::wm_subwindow::GLA_PIXEL_OFS / win_size.1 as f32;
    let verts = [
        ((halfx, halfy), (0.0, soffy as f32)),
        ((1.0 + halfx, halfy), (win_size.0 as f32, soffy as f32)),
        (
            (1.0 + halfx, 1.0 + halfy),
            (win_size.0 as f32, soffy as f32 + win_size.1 as f32 * 0.5),
        ),
        ((halfx, 1.0 + halfy), (0.0, soffy as f32 + win_size.1 as f32 * 0.5)),
    ];
    (soffy, verts)
}

/// `wm_stereo3d_is_fullscreen_required` (verbatim).
pub fn wm_stereo3d_is_fullscreen_required(stereo_display: i32) -> bool {
    stereo_display == S3D_DISPLAY_SIDEBYSIDE || stereo_display == S3D_DISPLAY_TOPBOTTOM
}

/// `WM_stereo3d_enabled` (verbatim — fullscreen-required
/// → ghost state; else screen stereo check).
pub fn wm_stereo3d_enabled(
    display_mode: i32,
    skip_stereo3d_check: bool,
    ghost_is_fullscreen: bool,
    screen_stereo3d_required: bool,
) -> bool {
    if wm_stereo3d_is_fullscreen_required(display_mode) {
        return ghost_is_fullscreen;
    }
    if !skip_stereo3d_check && !screen_stereo3d_required {
        return false;
    }
    if wm_stereo3d_is_fullscreen_required(display_mode) {
        return ghost_is_fullscreen;
    }
    true
}

/// `wm_stereo3d_mouse_offset_apply` (verbatim — halve
/// and double over the split axis).
pub fn wm_stereo3d_mouse_offset_apply(
    display_mode: i32,
    win_native_size: (i32, i32),
    r_mouse_xy: &mut [i32; 2],
) {
    if display_mode == S3D_DISPLAY_SIDEBYSIDE {
        let half_x = win_native_size.0 / 2;
        if r_mouse_xy[0] > half_x {
            r_mouse_xy[0] -= half_x;
        }
        r_mouse_xy[0] *= 2;
    } else if display_mode == S3D_DISPLAY_TOPBOTTOM {
        let half_y = win_native_size.1 / 2;
        if r_mouse_xy[1] > half_y {
            r_mouse_xy[1] -= half_y;
        }
        r_mouse_xy[1] *= 2;
    }
}

/// `Stereo3dFormat` verbatim fields (`DNA_scene_types.h`).
#[derive(Clone, Copy, Default)]
pub struct Stereo3dFormat {
    /// `display_mode`/`anaglyph_type`/`interlace_type`.
    pub display_mode: i32,
    pub anaglyph_type: i32,
    pub interlace_type: i32,
    /// `flag`.
    pub flag: i32,
}

/// `wm_stereo3d_set_properties` (verbatim — enum/bool
/// props → format).
pub fn wm_stereo3d_set_properties(
    s3d: &mut Stereo3dFormat,
    prop_is_set: &dyn Fn(&str) -> bool,
    prop_enum_get: &dyn Fn(&str) -> i32,
    prop_bool_get: &dyn Fn(&str) -> bool,
) -> bool {
    let mut is_set = false;
    if (prop_is_set)("display_mode") {
        s3d.display_mode = (prop_enum_get)("display_mode");
        is_set = true;
    }
    if (prop_is_set)("anaglyph_type") {
        s3d.anaglyph_type = (prop_enum_get)("anaglyph_type");
        is_set = true;
    }
    if (prop_is_set)("interlace_type") {
        s3d.interlace_type = (prop_enum_get)("interlace_type");
        is_set = true;
    }
    if (prop_is_set)("use_interlace_swap") {
        if (prop_bool_get)("use_interlace_swap") {
            s3d.flag |= S3D_INTERLACE_SWAP;
        } else {
            s3d.flag &= !S3D_INTERLACE_SWAP;
        }
        is_set = true;
    }
    if (prop_is_set)("use_sidebyside_crosseyed") {
        if (prop_bool_get)("use_sidebyside_crosseyed") {
            s3d.flag |= S3D_SIDEBYSIDE_CROSSEYED;
        } else {
            s3d.flag &= !S3D_SIDEBYSIDE_CROSSEYED;
        }
        is_set = true;
    }
    is_set
}

/// `wm_stereo3d_set_draw` field list (verbatim — per-mode
/// props shown in the popup).
pub fn wm_stereo3d_set_draw_props(display_mode: i32) -> Vec<&'static str> {
    let mut props = vec!["display_mode"];
    match display_mode {
        S3D_DISPLAY_ANAGLYPH => props.push("anaglyph_type"),
        S3D_DISPLAY_INTERLACE => {
            props.push("interlace_type");
            props.push("use_interlace_swap");
        }
        S3D_DISPLAY_SIDEBYSIDE => props.push("use_sidebyside_crosseyed"),
        _ => {}
    }
    props
}
