//! Port of `intern/paint.cc` — paint-mode dispatch, cavity
//! curve presets, select/hide tests, palette ops, color
//! jitter.
//!
//! `PORT_PENDING`: `BKE_paint_ensure`/`paint_init_data`/
//! `BKE_paint_copy`/`blend_write/read` (full Paint struct
//! init — needs `ToolSettings` mode table + CurveMapping
//! ownership), asset-reference machinery (`paint_brush_*
//! essentials` — needs `AssetWeakReference`/AssetLibrary),
//! `BKE_paintmode_get_active_from_context` (bContext).

use blenders_dna::color_types::{CurveMapping, CUMA_EXTEND_EXTRAPOLATE};

/// `PaintMode` verbatim (`ePaintMode`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PaintMode {
    /// `Invalid`.
    #[default]
    Invalid = 0,
    /// `Sculpt`.
    Sculpt = 1,
    /// `Vertex`.
    Vertex = 2,
    /// `Weight`.
    Weight = 3,
    /// `Texture2D`.
    Texture2d = 4,
    /// `Texture3D`.
    Texture3d = 5,
    /// `GPencil`.
    Gpencil = 6,
    /// `SculptCurves`.
    SculptCurves = 7,
}
/// `PAINT_MODE_SCULPT` int codes (DNA_scene_types
/// `PaintMode` verbatim numbers).
pub const PAINT_MODE_SCULPT: i8 = 1;
/// `PAINT_MODE_VERTEX`.
pub const PAINT_MODE_VERTEX: i8 = 2;
/// `PAINT_MODE_WEIGHT`.
pub const PAINT_MODE_WEIGHT: i8 = 3;
/// `PAINT_MODE_TEXTURE2D`.
pub const PAINT_MODE_TEXTURE2D: i8 = 4;
/// `PAINT_MODE_TEXTURE3D`.
pub const PAINT_MODE_TEXTURE3D: i8 = 5;
/// `PAINT_MODE_GPENCIL`.
pub const PAINT_MODE_GPENCIL: i8 = 6;
/// `PAINT_MODE_SCULPT_CURVES`.
pub const PAINT_MODE_SCULPT_CURVES: i8 = 7;
/// `PAINT_MODE_INVALID`.
pub const PAINT_MODE_INVALID: i8 = 0;

/// `BKE_paint_object_mode_from_paintmode` (verbatim).
pub fn paint_object_mode_from_paintmode(mode: PaintMode) -> i32 {
    match mode {
        PaintMode::Sculpt => OB_MODE_SCULPT,
        PaintMode::Vertex => OB_MODE_VERTEX_PAINT,
        PaintMode::Weight => OB_MODE_WEIGHT_PAINT,
        PaintMode::Texture2d | PaintMode::Texture3d => OB_MODE_TEXTURE_PAINT,
        PaintMode::SculptCurves => OB_MODE_SCULPT_CURVES,
        PaintMode::Gpencil => OB_MODE_PAINT_GREASE_PENCIL,
        PaintMode::Invalid => OB_MODE_OBJECT,
    }
}
/// `OB_MODE_*` (DNA_object_enums verbatim).
pub const OB_MODE_OBJECT: i32 = 0;
/// `OB_MODE_EDIT`.
pub const OB_MODE_EDIT: i32 = 1 << 0;
/// `OB_MODE_SCULPT`.
pub const OB_MODE_SCULPT: i32 = 1 << 1;
/// `OB_MODE_VERTEX_PAINT`.
pub const OB_MODE_VERTEX_PAINT: i32 = 1 << 2;
/// `OB_MODE_WEIGHT_PAINT`.
pub const OB_MODE_WEIGHT_PAINT: i32 = 1 << 3;
/// `OB_MODE_TEXTURE_PAINT`.
pub const OB_MODE_TEXTURE_PAINT: i32 = 1 << 4;
/// `OB_MODE_PAINT_GREASE_PENCIL`.
pub const OB_MODE_PAINT_GREASE_PENCIL: i32 = 1 << 5;
/// `OB_MODE_EDIT_GPENCIL_LEGACY`.
pub const OB_MODE_EDIT_GPENCIL_LEGACY: i32 = 1 << 6;
/// `OB_MODE_SCULPT_GREASE_PENCIL`.
pub const OB_MODE_SCULPT_GREASE_PENCIL: i32 = 1 << 7;
/// `OB_MODE_WEIGHT_GREASE_PENCIL`.
pub const OB_MODE_WEIGHT_GREASE_PENCIL: i32 = 1 << 8;
/// `OB_MODE_VERTEX_GREASE_PENCIL`.
pub const OB_MODE_VERTEX_GREASE_PENCIL: i32 = 1 << 9;
/// `OB_MODE_SCULPT_CURVES`.
pub const OB_MODE_SCULPT_CURVES: i32 = 1 << 10;
/// `OB_MODE_ALL_MODE_DATA`.
pub const OB_MODE_ALL_MODE_DATA: i32 = 0x7FF;

/// `BKE_paint_select_face_test` — `(ob && ob->type ==
/// OB_MESH && ob->mode & OB_MODE_EDIT)`.
pub fn paint_select_face_test(ob_type: i8, ob_mode: i32, has_data: bool) -> bool {
    ob_type == OB_MESH && (ob_mode & OB_MODE_EDIT) != 0 && has_data
}
/// `OB_MESH`.
pub const OB_MESH: i8 = 1;
/// `OB_GREASE_PENCIL`.
pub const OB_GREASE_PENCIL: i8 = 15;
/// `OB_CURVES`.
pub const OB_CURVES: i8 = 16;
/// `OB_MBALL`.
pub const OB_MBALL: i8 = 10;
/// `BKE_paint_select_vert_test` (verbatim — same check).
pub fn paint_select_vert_test(ob_type: i8, ob_mode: i32, has_data: bool) -> bool {
    ob_type == OB_MESH && (ob_mode & OB_MODE_EDIT) != 0 && has_data
}
/// `BKE_paint_select_grease_pencil_test`.
pub fn paint_select_grease_pencil_test(ob_type: i8, ob_mode: i32, has_data: bool) -> bool {
    ob_type == OB_GREASE_PENCIL
        && (ob_mode & (OB_MODE_EDIT | OB_MODE_EDIT_GPENCIL_LEGACY)) != 0
        && has_data
}
/// `BKE_paint_select_elem_test` — either face or vert path.
pub fn paint_select_elem_test(ob_type: i8, ob_mode: i32, has_data: bool) -> bool {
    paint_select_vert_test(ob_type, ob_mode, has_data)
        || paint_select_face_test(ob_type, ob_mode, has_data)
        || paint_select_grease_pencil_test(ob_type, ob_mode, has_data)
}

/// `BKE_paint_always_hide_test` (verbatim).
pub fn paint_always_hide_test(ob_type: i8, ob_mode: i32, has_data: bool) -> bool {
    ob_type == OB_MESH
        && has_data
        && (ob_mode & (OB_MODE_WEIGHT_PAINT | OB_MODE_VERTEX_PAINT)) != 0
}

/// `BKE_paint_cavity_curve_preset` — needs a `CurveMapping`
/// slot; verbatim apply (`curvemap_reset` + flag change).
pub fn paint_cavity_curve_preset(
    cavity_curve: &mut CurveMapping,
    preset: i32,
) {
    cavity_curve.flag &= !CUMA_EXTEND_EXTRAPOLATE;
    cavity_curve.preset = preset;
    let clipr = cavity_curve.clipr;
    crate::colortools::curvemap_reset(
        &mut cavity_curve.cm[0],
        &clipr,
        preset,
        crate::colortools::CurveMapSlopeType::Positive,
    );
    /* `BKE_curvemapping_changed(cumap, false)` — marks
     * table dirty: */
    cavity_curve.changed_timestamp += 1;
}

/// `BRUSH_COLOR_JITTER_USE_*` verbatim.
pub const BRUSH_COLOR_JITTER_USE_HUE_AT_STROKE: i32 = 1 << 0;
/// `BRUSH_COLOR_JITTER_USE_SAT_AT_STROKE`.
pub const BRUSH_COLOR_JITTER_USE_SAT_AT_STROKE: i32 = 1 << 1;
/// `BRUSH_COLOR_JITTER_USE_VAL_AT_STROKE`.
pub const BRUSH_COLOR_JITTER_USE_VAL_AT_STROKE: i32 = 1 << 2;
/// `BRUSH_COLOR_JITTER_USE_HUE_RAND_PRESS`.
pub const BRUSH_COLOR_JITTER_USE_HUE_RAND_PRESS: i32 = 1 << 3;
/// `BRUSH_COLOR_JITTER_USE_SAT_RAND_PRESS`.
pub const BRUSH_COLOR_JITTER_USE_SAT_RAND_PRESS: i32 = 1 << 4;
/// `BRUSH_COLOR_JITTER_USE_VAL_RAND_PRESS`.
pub const BRUSH_COLOR_JITTER_USE_VAL_RAND_PRESS: i32 = 1 << 5;

/// `BrushColorJitterSettings` (verbatim).
pub struct BrushColorJitterSettings {
    /// `flag`.
    pub flag: i32,
    /// `hue`, `saturation`, `value` jitter amounts.
    pub hue: f32,
    /// `saturation`.
    pub saturation: f32,
    /// `value`.
    pub value: f32,
    /// `curve_hue_jitter`/`curve_sat_jitter`/`curve_val_jitter`
    /// — CurveMapping handles (usize → CurveMapping index).
    pub curve_hue_jitter: usize,
    /// `curve_sat_jitter`.
    pub curve_sat_jitter: usize,
    /// `curve_val_jitter`.
    pub curve_val_jitter: usize,
}

/// `BKE_paint_randomize_color` (verbatim; `noise::perlin` on
/// `float2`, `CurveMapping` evaluates via injected closure).
pub fn paint_randomize_color(
    color_jitter: &BrushColorJitterSettings,
    initial_hsv_jitter: [f32; 3],
    distance: f32,
    pressure: f32,
    color: [f32; 3],
    eval_curve: &mut dyn FnMut(usize, f32) -> f32,
) -> [f32; 3] {
    const NOISE_SCALE: f32 = 1.0 / 20.0;
    let perlin2 = |a: f32, b: f32| {
        blenders_bli::noise::perlin(blenders_math::Vec2::new(a, b))
    };
    let random_hue = if (color_jitter.flag & BRUSH_COLOR_JITTER_USE_HUE_AT_STROKE) != 0 {
        initial_hsv_jitter[0]
    } else {
        perlin2(distance * NOISE_SCALE, initial_hsv_jitter[0] * 100.0)
    };
    let random_sat = if (color_jitter.flag & BRUSH_COLOR_JITTER_USE_SAT_AT_STROKE) != 0 {
        initial_hsv_jitter[1]
    } else {
        perlin2(distance * NOISE_SCALE, initial_hsv_jitter[1] * 100.0)
    };
    let random_val = if (color_jitter.flag & BRUSH_COLOR_JITTER_USE_VAL_AT_STROKE) != 0 {
        initial_hsv_jitter[2]
    } else {
        perlin2(distance * NOISE_SCALE, initial_hsv_jitter[2] * 100.0)
    };
    let mut hue_jitter_scale = color_jitter.hue;
    if (color_jitter.flag & BRUSH_COLOR_JITTER_USE_HUE_RAND_PRESS) != 0 {
        hue_jitter_scale *= eval_curve(color_jitter.curve_hue_jitter, pressure);
    }
    let mut sat_jitter_scale = color_jitter.saturation;
    if (color_jitter.flag & BRUSH_COLOR_JITTER_USE_SAT_RAND_PRESS) != 0 {
        sat_jitter_scale *= eval_curve(color_jitter.curve_sat_jitter, pressure);
    }
    let mut val_jitter_scale = color_jitter.value;
    if (color_jitter.flag & BRUSH_COLOR_JITTER_USE_VAL_RAND_PRESS) != 0 {
        val_jitter_scale *= eval_curve(color_jitter.curve_val_jitter, pressure);
    }
    let mut hsv = {
        let (h, s, v) =
            blenders_math::color::rgb_to_hsv(color[0], color[1], color[2]);
        [h, s, v]
    };
    hsv[0] += blenders_math::base::interpf(0.5, random_hue, hue_jitter_scale) - 0.5;
    if hsv[0] > 1.0 {
        hsv[0] -= 1.0;
    } else if hsv[0] < 0.0 {
        hsv[0] += 1.0;
    }
    hsv[1] *= blenders_math::base::interpf(1.0, random_sat * 2.0, sat_jitter_scale);
    hsv[2] *= blenders_math::base::interpf(1.0, random_val * 2.0, val_jitter_scale);
    blenders_math::color::hsv_to_rgb(hsv[0], hsv[1], hsv[2])
}

/// `BKE_palette_color_set` — `PaletteColor.rgb = rgb`.
pub fn palette_color_set(color: &mut [f32; 3], rgb: [f32; 3]) {
    *color = rgb;
}
/// `BKE_palette_clear` — drop all colors.
pub fn palette_clear(colors: &mut Vec<[f32; 3]>) {
    colors.clear();
}
/// `BKE_palette_color_remove`.
pub fn palette_color_remove(colors: &mut Vec<[f32; 3]>, color_index: usize) {
    if color_index < colors.len() {
        colors.remove(color_index);
    }
}
