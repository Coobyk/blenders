//! Port of `intern/brush.cc` — `brush_defaults`, unified-paint
//! accessors (`use_unified_{size,strength,color}`, `color`/
//! `secondary_color`/`size`/`radius`/`unprojected_size`/`alpha`/
//! `weight`/`input_samples` get/set), `color_jitter_get_settings`,
//! `mask/color_texture_get`, `sample_tex_3d`/`sample_masktex` (tex
//! eval + colorspace injected), `jitter_pos`,
//! `randomize_texture_coords`, `scale_(unprojected_)size`,
//! `calc_curve_factors`/`curve_strength`, `curve_preset`,
//! `common_pressure_curves_init`.
//! `PORT_PENDING`: `brush_init_data`/`copy_data`/`free_data`/
//! `make_local`/`foreach_id`/blend IO/`asset_metadata_ensure`/
//! `BKE_brush_add`/`_delete`/`_duplicate`/`_init_*_settings`/
//! `debug_print_state` (need `Main`/ID-type table/ImBuf); `Brush`
//! `palette`/icon IO.

use blenders_dna::brush_enums::{
    BRUSH_ALPHA_PRESSURE, BRUSH_JITTER_COLOR, BRUSH_LOCK_SIZE, BRUSH_SIZE_PRESSURE,
    BRUSH_USE_UNIFIED_PAINT_ALPHA, BRUSH_USE_UNIFIED_PAINT_COLOR,
    BRUSH_USE_UNIFIED_PAINT_INPUT_SAMPLES, BRUSH_USE_UNIFIED_PAINT_SIZE,
    BRUSH_USE_UNIFIED_PAINT_WEIGHT,
};
use blenders_dna::brush_types::Brush;
use blenders_dna::scene_enums::{UNIFIED_PAINT_BRUSH_LOCK_SIZE, UNIFIED_PAINT_COLOR_JITTER};
use blenders_dna::scene_types::{Paint, UnifiedPaintSettings};
use blenders_dna::texture_types::{
    MTex, MTEX_MAP_MODE_3D, MTEX_MAP_MODE_RANDOM, MTEX_MAP_MODE_STENCIL,
    MTEX_MAP_MODE_TILED, MTEX_MAP_MODE_VIEW,
};
use blenders_math::vector::{Vec2, Vec3};

use crate::paint_runtime::PaintRuntime;

/// `MAX_BRUSH_PIXEL_DIAMETER` (BKE_paint.hh).
pub const MAX_BRUSH_PIXEL_DIAMETER: i32 = 500;
/// `BRUSH_MASK_PRESSURE_CUTOFF`/`_RAMP`.
pub const BRUSH_MASK_PRESSURE_CUTOFF: i32 = 1;
/// `BRUSH_MASK_PRESSURE_RAMP`.
pub const BRUSH_MASK_PRESSURE_RAMP: i32 = 2;
/// `BRUSH_ABSOLUTE_JITTER`.
pub const BRUSH_ABSOLUTE_JITTER: u32 = 1 << 30;
/// `PaintMode::Sculpt` index.
pub const PAINT_MODE_SCULPT: i8 = 0;

/// `eBrushCurvePreset`.
pub const BRUSH_CURVE_CUSTOM: i32 = 0;
/// `BRUSH_CURVE_SHARP`.
pub const BRUSH_CURVE_SHARP: i32 = 1;
/// `BRUSH_CURVE_SMOOTH`.
pub const BRUSH_CURVE_SMOOTH: i32 = 2;
/// `BRUSH_CURVE_SMOOTHER`.
pub const BRUSH_CURVE_SMOOTHER: i32 = 3;
/// `BRUSH_CURVE_SPHERE`.
pub const BRUSH_CURVE_SPHERE: i32 = 4;
/// `BRUSH_CURVE_POW4`.
pub const BRUSH_CURVE_POW4: i32 = 5;
/// `BRUSH_CURVE_INVSQUARE`.
pub const BRUSH_CURVE_INVSQUARE: i32 = 6;

/// `BrushColorJitterSettings`.
#[derive(Clone, Copy)]
pub struct BrushColorJitterSettings {
    /// `color_jitter_flag`.
    pub flag: i32,
    /// `hsv_jitter[0]`.
    pub jitter_hue: f32,
    /// `hsv_jitter[1]`.
    pub jitter_saturation: f32,
    /// `hsv_jitter[2]`.
    pub jitter_value: f32,
    /// `curve_rand_hue`.
    pub curve_rand_hue: usize,
    /// `curve_rand_saturation`.
    pub curve_rand_saturation: usize,
    /// `curve_rand_value`.
    pub curve_rand_value: usize,
}

/// `brush_defaults` — reset the listed members to `Brush::default()`.
pub fn brush_defaults(brush: &mut Brush) {
    let d = Brush::default();
    brush.blend = d.blend;
    brush.flag = d.flag;
    brush.weight = d.weight;
    brush.size = d.size;
    brush.alpha = d.alpha;
    brush.hardness = d.hardness;
    brush.autosmooth_factor = d.autosmooth_factor;
    brush.topology_rake_factor = d.topology_rake_factor;
    brush.crease_pinch_factor = d.crease_pinch_factor;
    brush.normal_radius_factor = d.normal_radius_factor;
    brush.wet_paint_radius_factor = d.wet_paint_radius_factor;
    brush.area_radius_factor = d.area_radius_factor;
    brush.disconnected_distance_max = d.disconnected_distance_max;
    brush.sculpt_plane = d.sculpt_plane;
    brush.plane_offset = d.plane_offset;
    brush.normal_weight = d.normal_weight;
    brush.fill_threshold = d.fill_threshold;
    brush.sampling_flag = d.sampling_flag;
    brush.color = d.color;
    brush.secondary_color = d.secondary_color;
    brush.spacing = d.spacing;
    brush.smooth_stroke_radius = d.smooth_stroke_radius;
    brush.smooth_stroke_factor = d.smooth_stroke_factor;
    brush.rate = d.rate;
    brush.jitter = d.jitter;
    brush.texture_sample_bias = d.texture_sample_bias;
    brush.texture_overlay_alpha = d.texture_overlay_alpha;
    brush.mask_overlay_alpha = d.mask_overlay_alpha;
    brush.cursor_overlay_alpha = d.cursor_overlay_alpha;
    brush.overlay_flags = d.overlay_flags;
    brush.add_col = d.add_col;
    brush.sub_col = d.sub_col;
    brush.stencil_pos = d.stencil_pos;
    brush.stencil_dimension = d.stencil_dimension;
    brush.mtex = d.mtex;
    brush.mask_mtex = d.mask_mtex;
    brush.falloff_shape = d.falloff_shape;
    brush.tip_scale_x = d.tip_scale_x;
    brush.tip_roundness = d.tip_roundness;
}

/// `BKE_brush_curve_preset` — reseed `curve_distance_falloff` via
/// injected reset/alloc.
pub fn brush_curve_preset(
    brush: &mut Brush,
    preset: i32,
    curve_add: &mut dyn FnMut(i32, f32, f32, f32, f32) -> usize,
    curvemap_reset: &mut dyn FnMut(usize, i32),
    curvemapping_changed: &mut dyn FnMut(usize),
    tag_unsaved: &mut dyn FnMut(),
) {
    if brush.curve_distance_falloff == 0 {
        brush.curve_distance_falloff = curve_add(1, 0.0, 0.0, 1.0, 1.0);
    }
    brush.curve_distance_falloff_preset = preset;
    curvemap_reset(brush.curve_distance_falloff, preset);
    curvemapping_changed(brush.curve_distance_falloff);
    tag_unsaved();
}

/// `BKE_brush_mask_texture_get` — Sculpt uses `mtex`.
pub fn mask_texture_get<'a>(brush: &'a Brush, paint_mode_sculpt: bool) -> &'a MTex {
    if paint_mode_sculpt {
        &brush.mtex
    } else {
        &brush.mask_mtex
    }
}

/// `BKE_brush_color_texture_get` — inverse of mask.
pub fn color_texture_get<'a>(brush: &'a Brush, paint_mode_sculpt: bool) -> &'a MTex {
    if paint_mode_sculpt {
        &brush.mask_mtex
    } else {
        &brush.mtex
    }
}

fn rotate(x: &mut f32, y: &mut f32, rotation: f32) {
    if rotation > 0.001 || rotation < -0.001 {
        let angle = y.atan2(*x) + rotation;
        let flen = (*x * *x + *y * *y).sqrt();
        *x = flen * angle.cos();
        *y = flen * angle.sin();
    }
}

/// `BKE_brush_sample_tex_3d` — `texture_evaluate(mtex, co) ->
/// Option<(intensity, rgba)>` injected.
pub fn sample_tex_3d(
    runtime: &PaintRuntime,
    br: &Brush,
    mtex: Option<&MTex>,
    point: Vec3,
    texture_evaluate: &mut dyn FnMut(&MTex, Vec3) -> Option<(f32, [f32; 4])>,
    colorspace_to_linear: &mut dyn FnMut(usize, &mut [f32; 3]),
) -> (f32, [f32; 4]) {
    let mut intensity = 1.0f32;
    let mut rgba = [0.0f32; 4];
    let Some(mtex) = mtex else {
        return (1.0, rgba_linear_fill(1.0));
    };
    if mtex.tex == 0 {
        return (1.0, rgba_linear_fill(1.0));
    }
    let mut hasrgb = false;
    if mtex.brush_map_mode == MTEX_MAP_MODE_3D {
        if let Some((i, r)) = texture_evaluate(mtex, point) {
            intensity = i;
            rgba = r;
            hasrgb = true;
        }
    } else if mtex.brush_map_mode == MTEX_MAP_MODE_STENCIL {
        let rotation = -mtex.rot;
        let mut x = point.x - br.stencil_pos[0];
        let mut y = point.y - br.stencil_pos[1];
        rotate(&mut x, &mut y, rotation);
        if x.abs() > br.stencil_dimension[0] || y.abs() > br.stencil_dimension[1] {
            return (0.0, [0.0; 4]);
        }
        x /= br.stencil_dimension[0];
        y /= br.stencil_dimension[1];
        if let Some((i, r)) = texture_evaluate(mtex, Vec3::new(x, y, 0.0)) {
            intensity = i;
            rgba = r;
            hasrgb = true;
        }
    } else {
        let mut rotation = -mtex.rot;
        let mut x = 0.0f32;
        let mut y = 0.0f32;
        let mut invradius = 1.0f32;
        if mtex.brush_map_mode == MTEX_MAP_MODE_VIEW {
            rotation -= runtime.brush_rotation;
            x = point.x - runtime.tex_mouse[0];
            y = point.y - runtime.tex_mouse[1];
            invradius = 1.0 / runtime.pixel_radius;
        } else if mtex.brush_map_mode == MTEX_MAP_MODE_TILED {
            invradius = 1.0 / runtime.start_pixel_radius;
            x = point.x;
            y = point.y;
        } else if mtex.brush_map_mode == MTEX_MAP_MODE_RANDOM {
            rotation -= runtime.brush_rotation;
            x = point.x - runtime.tex_mouse[0];
            y = point.y - runtime.tex_mouse[1];
            invradius = 1.0 / runtime.pixel_radius;
        }
        x *= invradius;
        y *= invradius;
        rotate(&mut x, &mut y, rotation);
        if let Some((i, r)) = texture_evaluate(mtex, Vec3::new(x, y, 0.0)) {
            intensity = i;
            rgba = r;
            hasrgb = true;
        }
    }
    intensity += br.texture_sample_bias;
    if !hasrgb {
        rgba = rgba_linear_fill(intensity);
    } else if runtime.do_linear_conversion {
        let mut v = [rgba[0], rgba[1], rgba[2]];
        colorspace_to_linear(runtime.colorspace, &mut v);
        rgba[0] = v[0];
        rgba[1] = v[1];
        rgba[2] = v[2];
    }
    (intensity, rgba)
}

fn rgba_linear_fill(v: f32) -> [f32; 4] {
    [v, v, v, 1.0]
}

/// `BKE_brush_sample_masktex` — mask variant using `_sec` rotation
/// + `mask_tex_mouse` + `size_pressure_value`.
pub fn sample_masktex(
    runtime: &PaintRuntime,
    br: &Brush,
    point: Vec2,
    texture_evaluate: &mut dyn FnMut(&MTex, Vec3) -> Option<(f32, [f32; 4])>,
) -> f32 {
    let mtex = &br.mask_mtex;
    if mtex.tex == 0 {
        return 1.0;
    }
    let mut intensity = 1.0f32;
    if mtex.brush_map_mode == MTEX_MAP_MODE_STENCIL {
        let rotation = -mtex.rot;
        let mut x = point.x - br.mask_stencil_pos[0];
        let mut y = point.y - br.mask_stencil_pos[1];
        rotate(&mut x, &mut y, rotation);
        if x.abs() > br.mask_stencil_dimension[0]
            || y.abs() > br.mask_stencil_dimension[1]
        {
            return 0.0;
        }
        x /= br.mask_stencil_dimension[0];
        y /= br.mask_stencil_dimension[1];
        if let Some((i, _)) = texture_evaluate(mtex, Vec3::new(x, y, 0.0)) {
            intensity = i;
        }
    } else {
        let mut rotation = -mtex.rot;
        let mut x = 0.0f32;
        let mut y = 0.0f32;
        let mut invradius = 1.0f32;
        if mtex.brush_map_mode == MTEX_MAP_MODE_VIEW {
            rotation -= runtime.brush_rotation_sec;
            x = point.x - runtime.mask_tex_mouse[0];
            y = point.y - runtime.mask_tex_mouse[1];
            invradius = 1.0 / runtime.pixel_radius;
        } else if mtex.brush_map_mode == MTEX_MAP_MODE_TILED {
            invradius = 1.0 / runtime.start_pixel_radius;
            x = point.x;
            y = point.y;
        } else if mtex.brush_map_mode == MTEX_MAP_MODE_RANDOM {
            rotation -= runtime.brush_rotation_sec;
            x = point.x - runtime.mask_tex_mouse[0];
            y = point.y - runtime.mask_tex_mouse[1];
            invradius = 1.0 / runtime.pixel_radius;
        }
        x *= invradius;
        y *= invradius;
        rotate(&mut x, &mut y, rotation);
        if let Some((i, _)) = texture_evaluate(mtex, Vec3::new(x, y, 0.0)) {
            intensity = i;
        }
    }
    intensity = intensity.clamp(0.0, 1.0);
    match br.mask_pressure {
        BRUSH_MASK_PRESSURE_CUTOFF => {
            if 1.0 - intensity < runtime.size_pressure_value {
                1.0
            } else {
                0.0
            }
        }
        BRUSH_MASK_PRESSURE_RAMP => {
            runtime.size_pressure_value + intensity * (1.0 - runtime.size_pressure_value)
        }
        _ => intensity,
    }
}

/* Unified settings. */

/// `BKE_brush_use_unified_size`.
pub fn use_unified_size(brush: Option<&Brush>) -> bool {
    let Some(b) = brush else {
        return true;
    };
    b.unified_paint_flags & BRUSH_USE_UNIFIED_PAINT_SIZE != 0
}
/// `BKE_brush_use_unified_strength`.
pub fn use_unified_strength(brush: Option<&Brush>) -> bool {
    let Some(b) = brush else {
        return true;
    };
    b.unified_paint_flags & BRUSH_USE_UNIFIED_PAINT_ALPHA != 0
}
/// `BKE_brush_use_unified_color`.
pub fn use_unified_color(brush: Option<&Brush>) -> bool {
    let Some(b) = brush else {
        return true;
    };
    b.unified_paint_flags & BRUSH_USE_UNIFIED_PAINT_COLOR != 0
}

/// `BKE_brush_color_get`.
pub fn color_get<'a>(paint: &'a Paint, brush: &'a Brush) -> [f32; 3] {
    if use_unified_color(Some(brush)) {
        paint.unified_paint_settings.color
    } else {
        brush.color
    }
}

/// `BKE_brush_color_jitter_get_settings`.
pub fn color_jitter_get_settings(
    paint: &Paint,
    brush: &Brush,
) -> Option<BrushColorJitterSettings> {
    if use_unified_color(Some(brush)) {
        if paint.unified_paint_settings.flag & UNIFIED_PAINT_COLOR_JITTER == 0 {
            return None;
        }
        let s = &paint.unified_paint_settings;
        return Some(BrushColorJitterSettings {
            flag: s.color_jitter_flag,
            jitter_hue: s.hsv_jitter[0],
            jitter_saturation: s.hsv_jitter[1],
            jitter_value: s.hsv_jitter[2],
            curve_rand_hue: s.curve_rand_hue,
            curve_rand_saturation: s.curve_rand_saturation,
            curve_rand_value: s.curve_rand_value,
        });
    }
    if brush.flag2 & BRUSH_JITTER_COLOR == 0 {
        return None;
    }
    Some(BrushColorJitterSettings {
        flag: brush.color_jitter_flag,
        jitter_hue: brush.hsv_jitter[0],
        jitter_saturation: brush.hsv_jitter[1],
        jitter_value: brush.hsv_jitter[2],
        curve_rand_hue: brush.curve_rand_hue,
        curve_rand_saturation: brush.curve_rand_saturation,
        curve_rand_value: brush.curve_rand_value,
    })
}

/// `BKE_brush_secondary_color_get`.
pub fn secondary_color_get(paint: &Paint, brush: &Brush) -> [f32; 3] {
    if use_unified_color(Some(brush)) {
        paint.unified_paint_settings.secondary_color
    } else {
        brush.secondary_color
    }
}

/// `BKE_brush_color_set`.
pub fn color_set(
    paint: &mut Paint,
    brush: &mut Brush,
    color: [f32; 3],
    tag_unsaved: &mut dyn FnMut(),
) {
    if use_unified_color(Some(brush)) {
        let ups = &mut paint.unified_paint_settings;
        ups.color = color;
        color_sync_legacy_ups(ups);
    } else {
        brush.color = color;
        tag_unsaved();
        color_sync_legacy_brush(brush);
    }
}

/// `BKE_brush_color_sync_legacy(Brush*)` — linear → sRGB legacy.
pub fn color_sync_legacy_brush(brush: &mut Brush) {
    brush.rgb = srgb_encode_v3(&brush.color);
    brush.secondary_rgb = srgb_encode_v3(&brush.secondary_color);
}
/// `BKE_brush_color_sync_legacy(UnifiedPaintSettings*)`.
pub fn color_sync_legacy_ups(ups: &mut UnifiedPaintSettings) {
    ups.rgb = srgb_encode_v3(&ups.color);
    ups.secondary_rgb = srgb_encode_v3(&ups.secondary_color);
}

fn srgb_encode_v3(c: &[f32; 3]) -> [f32; 3] {
    c.map(|v| {
        if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    })
}

/// `BKE_brush_size_set`.
pub fn size_set(
    paint: &mut Paint,
    brush: &mut Brush,
    size: i32,
    tag_unsaved: &mut dyn FnMut(),
) {
    let size = size.clamp(1, MAX_BRUSH_PIXEL_DIAMETER);
    if use_unified_size(Some(brush)) {
        paint.unified_paint_settings.size = size;
    } else {
        brush.size = size;
        tag_unsaved();
    }
}
/// `BKE_brush_size_get`.
pub fn size_get(paint: &Paint, brush: &Brush) -> i32 {
    if use_unified_size(Some(brush)) {
        paint.unified_paint_settings.size
    } else {
        brush.size
    }
}
/// `BKE_brush_radius_get`.
pub fn radius_get(paint: &Paint, brush: &Brush) -> f32 {
    size_get(paint, brush) as f32 / 2.0
}
/// `BKE_brush_use_locked_size`.
pub fn use_locked_size(paint: &Paint, brush: &Brush) -> bool {
    if brush.unified_paint_flags & BRUSH_USE_UNIFIED_PAINT_SIZE != 0 {
        paint.unified_paint_settings.flag & UNIFIED_PAINT_BRUSH_LOCK_SIZE != 0
    } else {
        brush.flag & BRUSH_LOCK_SIZE != 0
    }
}
/// `BKE_brush_use_size_pressure`.
pub fn use_size_pressure(brush: &Brush) -> bool {
    brush.flag & BRUSH_SIZE_PRESSURE != 0
}
/// `BKE_brush_use_alpha_pressure`.
pub fn use_alpha_pressure(brush: &Brush) -> bool {
    brush.flag & BRUSH_ALPHA_PRESSURE != 0
}
/// `BKE_brush_unprojected_size_set`.
pub fn unprojected_size_set(
    paint: &mut Paint,
    brush: &mut Brush,
    unprojected_size: f32,
    tag_unsaved: &mut dyn FnMut(),
) {
    if use_unified_size(Some(brush)) {
        paint.unified_paint_settings.unprojected_size = unprojected_size;
    } else {
        brush.unprojected_size = unprojected_size;
        tag_unsaved();
    }
}
/// `BKE_brush_unprojected_size_get`.
pub fn unprojected_size_get(paint: &Paint, brush: &Brush) -> f32 {
    if use_unified_size(Some(brush)) {
        paint.unified_paint_settings.unprojected_size
    } else {
        brush.unprojected_size
    }
}
/// `BKE_brush_unprojected_radius_get`.
pub fn unprojected_radius_get(paint: &Paint, brush: &Brush) -> f32 {
    unprojected_size_get(paint, brush) / 2.0
}
/// `BKE_brush_scale_unprojected_size`.
pub fn scale_unprojected_size(
    unprojected_size: &mut f32,
    new_brush_size: i32,
    old_brush_size: i32,
) {
    let mut scale = new_brush_size as f32;
    if old_brush_size != 0 {
        scale /= old_brush_size as f32;
    }
    *unprojected_size *= scale;
}
/// `BKE_brush_scale_size` — note C divides by `new_unprojected_size`
/// (verbatim quirk).
pub fn scale_size(
    r_brush_size: &mut i32,
    new_unprojected_size: f32,
    old_unprojected_size: f32,
) {
    let mut scale = new_unprojected_size;
    if old_unprojected_size != 0.0 {
        scale /= new_unprojected_size;
    }
    *r_brush_size = (*r_brush_size as f32 * scale) as i32;
}
/// `BKE_brush_alpha_set`.
pub fn alpha_set(
    paint: &mut Paint,
    brush: &mut Brush,
    alpha: f32,
    tag_unsaved: &mut dyn FnMut(),
) {
    if use_unified_strength(Some(brush)) {
        paint.unified_paint_settings.alpha = alpha;
    } else {
        brush.alpha = alpha;
        tag_unsaved();
    }
}
/// `BKE_brush_alpha_get`.
pub fn alpha_get(paint: &Paint, brush: &Brush) -> f32 {
    if use_unified_strength(Some(brush)) {
        paint.unified_paint_settings.alpha
    } else {
        brush.alpha
    }
}
/// `BKE_brush_weight_get`.
pub fn weight_get(paint: &Paint, brush: &Brush) -> f32 {
    if brush.unified_paint_flags & BRUSH_USE_UNIFIED_PAINT_WEIGHT != 0 {
        paint.unified_paint_settings.weight
    } else {
        brush.weight
    }
}
/// `BKE_brush_weight_set`.
pub fn weight_set(
    paint: &mut Paint,
    brush: &mut Brush,
    value: f32,
    tag_unsaved: &mut dyn FnMut(),
) {
    if brush.unified_paint_flags & BRUSH_USE_UNIFIED_PAINT_WEIGHT != 0 {
        paint.unified_paint_settings.weight = value;
    } else {
        brush.weight = value;
        tag_unsaved();
    }
}
/// `BKE_brush_input_samples_get`.
pub fn input_samples_get(paint: &Paint, brush: &Brush) -> i32 {
    if brush.unified_paint_flags & BRUSH_USE_UNIFIED_PAINT_INPUT_SAMPLES != 0 {
        paint.unified_paint_settings.input_samples
    } else {
        brush.input_samples
    }
}
/// `BKE_brush_input_samples_set`.
pub fn input_samples_set(
    paint: &mut Paint,
    brush: &mut Brush,
    value: i32,
    tag_unsaved: &mut dyn FnMut(),
) {
    if brush.unified_paint_flags & BRUSH_USE_UNIFIED_PAINT_INPUT_SAMPLES != 0 {
        paint.unified_paint_settings.input_samples = value;
    } else {
        brush.input_samples = value;
        tag_unsaved();
    }
}

/// `BKE_brush_jitter_pos` — rng floats injected.
pub fn jitter_pos(
    paint: &Paint,
    brush: &Brush,
    pos: Vec2,
    rng: &mut dyn FnMut() -> f32,
) -> Vec2 {
    let mut rand_pos;
    loop {
        let x = rng() - 0.5;
        let y = rng() - 0.5;
        if x * x + y * y <= 0.25 {
            rand_pos = Vec2::new(x, y);
            break;
        }
    }
    let (diameter, spread) = if brush.flag & BRUSH_ABSOLUTE_JITTER != 0 {
        (2 * brush.jitter_absolute, 1.0f32)
    } else {
        ((2.0 * radius_get(paint, brush)) as i32, brush.jitter)
    };
    Vec2::new(
        pos.x + 2.0 * rand_pos.x * diameter as f32 * spread,
        pos.y + 2.0 * rand_pos.y * diameter as f32 * spread,
    )
}

/// `BKE_brush_randomize_texture_coords`.
pub fn randomize_texture_coords(runtime: &mut PaintRuntime, mask: bool, rng: &mut dyn FnMut() -> f32) {
    if mask {
        runtime.mask_tex_mouse[0] = rng() * runtime.pixel_radius;
        runtime.mask_tex_mouse[1] = rng() * runtime.pixel_radius;
    } else {
        runtime.tex_mouse[0] = rng() * runtime.pixel_radius;
        runtime.tex_mouse[1] = rng() * runtime.pixel_radius;
    }
}

/// `common_pressure_curves_init` — `curve_rand_*`/`curve_preset`
/// defaults via injected CurveMapping allocs.
pub fn common_pressure_curves_init(
    gp: &mut blenders_dna::brush_types::BrushGpencilSettings,
    curve_add: &mut dyn FnMut(i32, f32, f32, f32, f32) -> usize,
    curvemap_preset: &mut dyn FnMut(usize, i32),
) {
    const CURVE_PRESET_GAUSS: i32 = 5;
    gp.curve_rand_pressure = curve_add(1, 0.0, 0.0, 1.0, 1.0);
    curvemap_preset(gp.curve_rand_pressure, CURVE_PRESET_GAUSS);
    gp.curve_rand_strength = curve_add(1, 0.0, 0.0, 1.0, 1.0);
    curvemap_preset(gp.curve_rand_strength, CURVE_PRESET_GAUSS);
    gp.curve_rand_uv = curve_add(1, 0.0, 0.0, 1.0, 1.0);
    curvemap_preset(gp.curve_rand_uv, CURVE_PRESET_GAUSS);
}

/// `BKE_brush_calc_curve_factors`.
pub fn calc_curve_factors(
    preset: i32,
    cumap_eval: &dyn Fn(f32) -> f32,
    distances: &[f32],
    brush_radius: f32,
    factors: &mut [f32],
) {
    let radius_rcp = 1.0 / brush_radius;
    for (i, &distance) in distances.iter().enumerate() {
        if distance >= brush_radius {
            factors[i] = 0.0;
            continue;
        }
        let factor = 1.0 - distance * radius_rcp;
        factors[i] *= match preset {
            BRUSH_CURVE_CUSTOM => cumap_eval(distance * radius_rcp),
            BRUSH_CURVE_SHARP => factor * factor,
            BRUSH_CURVE_SMOOTH => 3.0 * factor * factor - 2.0 * factor * factor * factor,
            BRUSH_CURVE_SMOOTHER => {
                /* `smootherstep`: 6f⁵−15f⁴+10f³. */
                factor * factor * factor * (factor * (factor * 6.0 - 15.0) + 10.0)
            }
            BRUSH_CURVE_SPHERE => (2.0 * factor - factor * factor).sqrt(),
            BRUSH_CURVE_POW4 => factor * factor * factor * factor,
            BRUSH_CURVE_INVSQUARE => factor * (2.0 - factor),
            _ => factor,
        };
    }
}

/// `BKE_brush_curve_strength` — single-distance variant.
pub fn curve_strength(
    preset: i32,
    cumap_eval: &dyn Fn(f32) -> f32,
    distance: f32,
    brush_radius: f32,
) -> f32 {
    let mut factor = 1.0f32;
    let mut factors = [factor];
    calc_curve_factors(preset, cumap_eval, &[distance], brush_radius, &mut factors);
    factor = factors[0];
    factor
}
