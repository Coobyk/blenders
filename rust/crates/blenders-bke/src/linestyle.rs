//! Port of `intern/linestyle.cc` — `BKE_linestyle_init` defaults,
//! the 4 modifier lists (color/alpha/thickness/geometry) modeled as
//! typed enum `Vec`s, `new_modifier`/`add_to_modifier_list`/
//! `*_modifier_add`/`_copy`/`_remove`/`_move`,
//! `modifier_list_color_ramps`/`path_to_color_ramp`.
//! `PORT_PENDING`: `linestyle_{init,copy,free,foreach_id,foreach_
//! working_space_color}`/`blend_write`/`blend_read_data`/`direct_
//! link_*` (need `Main`/ID table/blend IO); `BKE_linestyle_new`/
//! `active_from_view_layer`.

use std::collections::HashSet;

use blenders_dna::linestyle_types::*;
use blenders_dna::texture_types::TEX_PR_TEXTURE;

/// `LS_MODIFIER_ENABLED | LS_MODIFIER_EXPANDED`.
pub const LS_MODIFIER_ENABLED: i32 = 1;
/// `EXPANDED`.
pub const LS_MODIFIER_EXPANDED: i32 = 2;
/// `LS_VALUE_BLEND`.
pub const LS_VALUE_BLEND: i32 = 1;
/// `LIB_ID_CREATE_NO_USER_REFCOUNT`.
pub const LIB_ID_CREATE_NO_USER_REFCOUNT: i32 = 1 << 10;
/// `LS_MODIFIER_MATERIAL_LINE` / `_LINE_A`.
pub const LS_MODIFIER_MATERIAL_LINE: i32 = 0;
/// `LINE_A`.
pub const LS_MODIFIER_MATERIAL_LINE_A: i32 = 1;
/// `LS_THICKNESS_ASYMMETRIC`.
pub const LS_THICKNESS_ASYMMETRIC: i32 = 1;
/// `LS_MODIFIER_SPATIAL_NOISE_SMOOTH`.
pub const LS_MODIFIER_SPATIAL_NOISE_SMOOTH: i32 = 1;
/// `LS_MODIFIER_SPATIAL_NOISE_PURERANDOM`.
pub const LS_MODIFIER_SPATIAL_NOISE_PURERANDOM: i32 = 2;
/// `LS_MODIFIER_BLUEPRINT_CIRCLES`.
pub const LS_MODIFIER_BLUEPRINT_CIRCLES: i32 = 1;
/// `LS_MODIFIER_2D_TRANSFORM_PIVOT_CENTER`.
pub const LS_MODIFIER_2D_TRANSFORM_PIVOT_CENTER: i32 = 0;

/// `modifier_name[]` — UI names per `eLineStyleModifier_Type`.
pub const MODIFIER_NAME: &[&str] = &[
    "Unknown",
    "Along Stroke",
    "Distance from Camera",
    "Distance from Object",
    "Material",
    "Sampling",
    "Bezier Curve",
    "Sinus Displacement",
    "Spatial Noise",
    "Perlin Noise 1D",
    "Perlin Noise 2D",
    "Backbone Stretcher",
    "Tip Remover",
    "Calligraphy",
    "Polygonalization",
    "Guiding Lines",
    "Blueprint",
    "2D Offset",
    "2D Transform",
    "Tangent",
    "Noise",
    "Crease Angle",
    "Simplification",
    "Curvature 3D",
];

/// `eLineStyleModifier_Type` values (verbatim C order).
pub const LS_MODIFIER_ALONG_STROKE: i16 = 1;
/// `DISTANCE_FROM_CAMERA`.
pub const LS_MODIFIER_DISTANCE_FROM_CAMERA: i16 = 2;
/// `DISTANCE_FROM_OBJECT`.
pub const LS_MODIFIER_DISTANCE_FROM_OBJECT: i16 = 3;
/// `MATERIAL`.
pub const LS_MODIFIER_MATERIAL: i16 = 4;
/// `SAMPLING`.
pub const LS_MODIFIER_SAMPLING: i16 = 5;
/// `BEZIER_CURVE`.
pub const LS_MODIFIER_BEZIER_CURVE: i16 = 6;
/// `SINUS_DISPLACEMENT`.
pub const LS_MODIFIER_SINUS_DISPLACEMENT: i16 = 7;
/// `SPATIAL_NOISE`.
pub const LS_MODIFIER_SPATIAL_NOISE: i16 = 8;
/// `PERLIN_NOISE_1D`.
pub const LS_MODIFIER_PERLIN_NOISE_1D: i16 = 9;
/// `PERLIN_NOISE_2D`.
pub const LS_MODIFIER_PERLIN_NOISE_2D: i16 = 10;
/// `BACKBONE_STRETCHER`.
pub const LS_MODIFIER_BACKBONE_STRETCHER: i16 = 11;
/// `TIP_REMOVER`.
pub const LS_MODIFIER_TIP_REMOVER: i16 = 12;
/// `CALLIGRAPHY`.
pub const LS_MODIFIER_CALLIGRAPHY: i16 = 13;
/// `POLYGONIZATION`.
pub const LS_MODIFIER_POLYGONIZATION: i16 = 14;
/// `GUIDING_LINES`.
pub const LS_MODIFIER_GUIDING_LINES: i16 = 15;
/// `BLUEPRINT`.
pub const LS_MODIFIER_BLUEPRINT: i16 = 16;
/// `2D_OFFSET`.
pub const LS_MODIFIER_2D_OFFSET: i16 = 17;
/// `2D_TRANSFORM`.
pub const LS_MODIFIER_2D_TRANSFORM: i16 = 18;
/// `TANGENT`.
pub const LS_MODIFIER_TANGENT: i16 = 19;
/// `NOISE`.
pub const LS_MODIFIER_NOISE: i16 = 20;
/// `CREASE_ANGLE`.
pub const LS_MODIFIER_CREASE_ANGLE: i16 = 21;
/// `SIMPLIFICATION`.
pub const LS_MODIFIER_SIMPLIFICATION: i16 = 22;
/// `CURVATURE_3D`.
pub const LS_MODIFIER_CURVATURE_3D: i16 = 23;

fn deg2rad(deg: f32) -> f32 {
    deg * (std::f32::consts::PI / 180.0)
}

fn new_modifier(name: Option<&str>, type_: i16) -> LineStyleModifier {
    let mut m = LineStyleModifier {
        type_: type_ as i32,
        influence: 1.0,
        flags: LS_MODIFIER_ENABLED | LS_MODIFIER_EXPANDED,
        ..Default::default()
    };
    let name = name.unwrap_or_else(|| {
        MODIFIER_NAME
            .get(type_ as usize)
            .copied()
            .unwrap_or("Unknown")
    });
    let b = name.as_bytes();
    let n = b.len().min(63);
    m.name[..n].copy_from_slice(&b[..n]);
    m
}

fn uniquename(lb: &[LineStyleModifier], m: &mut LineStyleModifier, type_: usize) {
    let base = String::from_utf8_lossy(
        &m.name[..m.name.iter().position(|&c| c == 0).unwrap_or(64)],
    )
    .to_string();
    let base = if base.is_empty() {
        MODIFIER_NAME.get(type_).copied().unwrap_or("Unknown").to_string()
    } else {
        base
    };
    let names: HashSet<String> = lb
        .iter()
        .map(|o| {
            String::from_utf8_lossy(
                &o.name[..o.name.iter().position(|&c| c == 0).unwrap_or(64)],
            )
            .to_string()
        })
        .collect();
    if !names.contains(&base) {
        return;
    }
    for i in 1..1000 {
        let candidate = format!("{}.{:03}", base.trim_end_matches(|c| c == '.'), i);
        if !names.contains(&candidate) {
            let b = candidate.as_bytes();
            let n = b.len().min(63);
            m.name = [0u8; 64];
            m.name[..n].copy_from_slice(&b[..n]);
            return;
        }
    }
}

/// `add_to_modifier_list`.
fn add_to_modifier_list(lb: &mut Vec<LineStyleModifier>, mut m: LineStyleModifier) {
    let t = m.type_ as usize;
    uniquename(lb, &mut m, t);
    lb.push(m);
}

/* Color modifiers — stored untyped (the C file allocates typed
 * subclasses; here each entry carries its typed payload). */

/// `BKE_linestyle_color_modifier_add` — per-type defaults verbatim;
/// `colorband_add` injected.
pub fn color_modifier_add(
    lb: &mut Vec<LineStyleModifier>,
    name: Option<&str>,
    type_: i16,
    colorband_add: &mut dyn FnMut() -> usize,
    ramp_handle: &mut dyn FnMut(usize, usize),
) -> Option<usize> {
    let mut m = new_modifier(name, type_);
    m.blend = LS_VALUE_BLEND;
    match type_ {
        LS_MODIFIER_ALONG_STROKE | LS_MODIFIER_TANGENT | LS_MODIFIER_NOISE => {}
        LS_MODIFIER_DISTANCE_FROM_CAMERA | LS_MODIFIER_DISTANCE_FROM_OBJECT
        | LS_MODIFIER_MATERIAL | LS_MODIFIER_CREASE_ANGLE | LS_MODIFIER_CURVATURE_3D => {}
        _ => return None,
    }
    add_to_modifier_list(lb, m);
    let idx = lb.len() - 1;
    match type_ {
        LS_MODIFIER_ALONG_STROKE | LS_MODIFIER_TANGENT => {
            ramp_handle(idx, colorband_add());
        }
        LS_MODIFIER_NOISE => {
            ramp_handle(idx, colorband_add());
        }
        _ => {
            ramp_handle(idx, colorband_add());
        }
    }
    Some(idx)
}

/// `BKE_linestyle_color_modifier_copy`.
pub fn color_modifier_copy(
    lb: &mut Vec<LineStyleModifier>,
    src_idx: usize,
    flag: i32,
    colorband_copy: &mut dyn FnMut(usize) -> usize,
    ramp_handle: &mut dyn FnMut(usize, usize),
    id_us_plus: &mut dyn FnMut(usize),
) -> Option<usize> {
    let src = lb.get(src_idx)?;
    let name = {
        let n = src.name.iter().position(|&c| c == 0).unwrap_or(64);
        String::from_utf8_lossy(&src.name[..n]).to_string()
    };
    let mut m = new_modifier(Some(&name), src.type_ as i16);
    m.influence = src.influence;
    m.flags = src.flags;
    m.blend = src.blend;
    let _ = flag;
    let _ = id_us_plus;
    add_to_modifier_list(lb, m);
    let idx = lb.len() - 1;
    ramp_handle(idx, colorband_copy(idx));
    Some(idx)
}

/// `BKE_linestyle_*_modifier_remove` — generic remove.
pub fn modifier_remove(lb: &mut Vec<LineStyleModifier>, index: usize) -> i32 {
    if index >= lb.len() {
        return -1;
    }
    lb.remove(index);
    0
}

/// `BKE_linestyle_*_modifier_move` — `BLI_listbase_link_move`.
pub fn modifier_move(lb: &mut Vec<LineStyleModifier>, index: usize, direction: i32) -> bool {
    if index >= lb.len() {
        return false;
    }
    let new_index = index as i32 + direction;
    if new_index < 0 || new_index >= lb.len() as i32 {
        return false;
    }
    lb.swap(index, new_index as usize);
    true
}

/// `BKE_linestyle_modifier_list_color_ramps` — collects the color
/// ramps of the AlongStroke/DistanceFromCamera/DistanceFromObject/
/// Material color modifiers (handle list via `ramp_of`).
pub fn modifier_list_color_ramps(
    lb: &[LineStyleModifier],
    ramp_of: &dyn Fn(usize) -> usize,
    out: &mut Vec<usize>,
) {
    out.clear();
    for (i, m) in lb.iter().enumerate() {
        match m.type_ as i16 {
            LS_MODIFIER_ALONG_STROKE
            | LS_MODIFIER_DISTANCE_FROM_CAMERA
            | LS_MODIFIER_DISTANCE_FROM_OBJECT
            | LS_MODIFIER_MATERIAL => {
                out.push(ramp_of(i));
            }
            _ => {}
        }
    }
}

/// `BKE_linestyle_path_to_color_ramp` — RNA path string of the
/// modifier owning `color_ramp`, or None.
pub fn path_to_color_ramp(
    lb: &[LineStyleModifier],
    color_ramp: usize,
    ramp_of: &dyn Fn(usize) -> usize,
) -> Option<String> {
    for (i, m) in lb.iter().enumerate() {
        match m.type_ as i16 {
            LS_MODIFIER_ALONG_STROKE
            | LS_MODIFIER_DISTANCE_FROM_CAMERA
            | LS_MODIFIER_DISTANCE_FROM_OBJECT
            | LS_MODIFIER_MATERIAL => {
                if ramp_of(i) == color_ramp {
                    let n = m.name.iter().position(|&c| c == 0).unwrap_or(64);
                    let name = String::from_utf8_lossy(&m.name[..n]).to_string();
                    return Some(format!("color_modifiers[\"{name}\"].color_ramp"));
                }
            }
            _ => {}
        }
    }
    None
}

/// `BKE_linestyle_init` — the `INIT_DEFAULT_STRUCT_AFTER` values
/// verbatim from `DNA_linestyle_types.h` member initializers.
/// The Sampling geometry modifier the C init appends is created by
/// the caller through `color/alpha/thickness/geometry` modifier
/// storage (modifier lists live outside `FreestyleLineStyle` here).
pub fn linestyle_init(ls: &mut FreestyleLineStyle) {
    ls.r = 0.0;
    ls.g = 0.0;
    ls.b = 0.0;
    ls.alpha = 1.0;
    ls.thickness = 3.0;
    ls.thickness_position = LS_THICKNESS_CENTER;
    ls.thickness_ratio = 0.5;
    ls.flag = LS_SAME_OBJECT | LS_NO_SORTING | LS_TEXTURE;
    ls.caps = LS_CAPS_BUTT;
    ls.chaining = LS_CHAINING_PLAIN;
    ls.rounds = 3;
    ls.split_length = 100.0;
    ls.min_angle = 0.0;
    ls.max_angle = 0.0;
    ls.min_length = 0.0;
    ls.max_length = 10000.0;
    ls.chain_count = 10;
    ls.split_dash1 = 0;
    ls.split_gap1 = 0;
    ls.split_dash2 = 0;
    ls.split_gap2 = 0;
    ls.split_dash3 = 0;
    ls.split_gap3 = 0;
    ls.sort_key = LS_SORT_KEY_DISTANCE_FROM_CAMERA;
    ls.integration_type = LS_INTEGRATION_MEAN;
    ls.texstep = 1.0;
    ls.texact = 0;
    ls.pr_texture = TEX_PR_TEXTURE;
    ls.use_nodes = 0;
    ls.dash1 = 0;
    ls.gap1 = 0;
    ls.dash2 = 0;
    ls.gap2 = 0;
    ls.dash3 = 0;
    ls.gap3 = 0;
    ls.panel = LS_PANEL_STROKES;
}
