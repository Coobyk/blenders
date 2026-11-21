//! Port of `intern/grease_pencil_convert_legacy.cc` —
//! legacy `bGPD*` → `GreasePencil` conversion helpers:
//! used vertex-group filtering, UV/layer/texture matrices,
//! modifier field copies, fcurve thickness conversion.
//!
//! `PORT_PENDING`: `legacy_gpencil_frame_to_grease_pencil_
//! drawing` (needs `bGPDframe`/`Drawing` full layouts +
//! attribute init), `legacy_object_modifier_*` per-type
//! converters (need `ModifierData` variants + `bGPDmodifier`
//! types), `conversion_data`/`grease_pencil_conversion_`
//! `bmain` orchestration, `sanitize_annotations`.

use blenders_math::Vec3;

/// `bGPDspoint` verbatim.
#[derive(Clone, Default)]
pub struct BGpdSPoint {
    /// `x`,`y`,`z` (position).
    pub x: f32,
    /// `y`.
    pub y: f32,
    /// `z`.
    pub z: f32,
    /// `pressure`.
    pub pressure: f32,
    /// `strength`.
    pub strength: f32,
    /// `time`.
    pub time: f32,
    /// `flag`.
    pub flag: i32,
    /// `uv_fac`.
    pub uv_fac: f32,
    /// `uv_rot`.
    pub uv_rot: f32,
}

/// `bGPDstroke` (verbatim fields used here).
#[derive(Clone, Default)]
pub struct BGpdStroke {
    /// `points` — `bGPDspoint` array.
    pub points: Vec<BGpdSPoint>,
    /// `totpoints`.
    pub totpoints: i32,
    /// `flag`.
    pub flag: i32,
    /// `dvert` handle (`MDeformVert` indices).
    pub dvert: usize,
    /// `uv_translation`/`uv_rotation`/`uv_scale`.
    pub uv_translation: [f32; 2],
    /// `uv_rotation`.
    pub uv_rotation: f32,
    /// `uv_scale`.
    pub uv_scale: [f32; 2],
}

/// `bGPDframe` (verbatim fields used here).
#[derive(Default)]
pub struct BGpdFrame {
    /// `strokes`.
    pub strokes: Vec<BGpdStroke>,
    /// `framenum`.
    pub framenum: i32,
}

/// `find_used_vertex_groups` (verbatim — keep only groups
/// referenced by stroke `dvert`s, build `old→new` index map).
/// Returns `(used_groups, old_index→new_index)` where
/// `used_groups` are the `bDeformGroup` handles kept.
pub fn find_used_vertex_groups(
    gpf: &BGpdFrame,
    dverts: &[Vec<(i32, f32)>], /* per-stroke: (def_nr, weight) */
    vertex_group_names: &[usize],
) -> (Vec<usize>, Vec<i32>) {
    let num_groups = vertex_group_names.len();
    let mut is_used = vec![false; num_groups];
    for (i, stroke) in gpf.strokes.iter().enumerate() {
        if stroke.dvert == usize::MAX || i >= dverts.len() {
            continue;
        }
        for &(def_nr, _w) in &dverts[i] {
            if def_nr < 0 || def_nr as usize >= num_groups {
                continue;
            }
            is_used[def_nr as usize] = true;
        }
    }
    let mut used = Vec::new();
    let mut indices = vec![-1i32; num_groups];
    let mut new_i = 0i32;
    for (old_i, &group) in vertex_group_names.iter().enumerate() {
        if !is_used[old_i] {
            continue;
        }
        indices[old_i] = new_i;
        new_i += 1;
        used.push(group);
    }
    (used, indices)
}

/* `float3x2`/`float4x2`/`float4x3` — fixed-size matrices
 * (`row-major`, rows = first index). */

/// `float3x2` — 3×2 (rows 3, cols 2)? In Blender `float3x2`
/// is a 3-column × 2-row matrix stored as `[c][r]`. Here we
/// keep it `[[f32; 2]; 3]` (cols of vec2).
pub type Float3x2 = [[f32; 2]; 3];
/// `float4x2` — 4 cols of vec2.
pub type Float4x2 = [[f32; 2]; 4];
/// `float4x3` — 4 cols of vec3.
pub type Float4x3 = [[f32; 3]; 4];
/// `float2x2`.
pub type Float2x2 = [[f32; 2]; 2];
/// `float2x4` — 2 cols of vec4.
pub type Float2x4 = [[f32; 4]; 2];

fn float3x2_identity() -> Float3x2 {
    [[1.0, 0.0], [0.0, 1.0], [0.0, 0.0]]
}
fn float4x2_identity() -> Float4x2 {
    [[1.0, 0.0], [0.0, 1.0], [0.0, 0.0], [0.0, 0.0]]
}
fn mul_2x2_by_3x2(m: Float2x2, a: Float3x2) -> Float3x2 {
    let mut r = [[0f32; 2]; 3];
    for c in 0..3 {
        for row in 0..2 {
            r[c][row] = m[0][row] * a[c][0] + m[1][row] * a[c][1];
        }
    }
    r
}
fn mul_3x2_by_4x3(a: Float3x2, b: Float4x3) -> Float4x2 {
    /* (3x2) * (4x3): output is 4x2 — col j of result =
     * a[0]*b[j][0] + a[1]*b[j][1] + a[2]*b[j][2] + trans. */
    let mut r = [[0f32; 2]; 4];
    for j in 0..4 {
        r[j][0] = a[0][0] * b[j][0] + a[1][0] * b[j][1] + a[2][0] * b[j][2];
        r[j][1] = a[0][1] * b[j][0] + a[1][1] * b[j][1] + a[2][1] * b[j][2];
    }
    r
}
fn transpose_2x4(a: Float2x4) -> Float4x2 {
    let mut r = [[0f32; 2]; 4];
    for c in 0..4 {
        for row in 0..2 {
            r[c][row] = a[row][c];
        }
    }
    r
}

/// `get_legacy_stroke_to_texture_matrix` (verbatim).
pub fn get_legacy_stroke_to_texture_matrix(
    uv_translation: [f32; 2],
    uv_rotation: f32,
    uv_scale: [f32; 2],
) -> Float3x2 {
    let minv = [-1.0f32, -1.0];
    let maxv = [1.0f32, 1.0];
    let center = [0.5f32, 0.5];
    let uv_scale_inv = [
        if uv_scale[0] == 0.0 { 0.0 } else { 1.0 / uv_scale[0] },
        if uv_scale[1] == 0.0 { 0.0 } else { 1.0 / uv_scale[1] },
    ];
    let diagonal = [maxv[0] - minv[0], maxv[1] - minv[1]];
    let (sin, cos) = (uv_rotation.sin(), uv_rotation.cos());
    let rotation: Float2x2 = [[cos, -sin], [sin, cos]];

    let mut m = float3x2_identity();
    /* Bounding-box rescale. */
    m[2][0] -= minv[0];
    m[2][1] -= minv[1];
    m = mul_2x2_by_3x2([[1.0 / diagonal[0], 0.0], [0.0, 1.0 / diagonal[1]]], m);
    /* Translation. */
    m[2][0] += uv_translation[0];
    m[2][1] += uv_translation[1];
    /* Rotation about center. */
    m[2][0] -= center[0];
    m[2][1] -= center[1];
    m = mul_2x2_by_3x2(rotation, m);
    m[2][0] += center[0];
    m[2][1] += center[1];
    /* Scale. */
    m = mul_2x2_by_3x2([[uv_scale_inv[0], 0.0], [0.0, uv_scale_inv[1]]], m);
    m
}

/// `get_legacy_layer_to_stroke_matrix` (verbatim).
pub fn get_legacy_layer_to_stroke_matrix(points: &[BGpdSPoint]) -> Float4x2 {
    let totpoints = points.len();
    if totpoints < 2 {
        return float4x2_identity();
    }
    let p0 = &points[0];
    let p1 = &points[1];
    let p3 = &points[(totpoints as f32 * 0.75) as usize];
    let pt0 = Vec3::new(p0.x, p0.y, p0.z);
    let pt1 = Vec3::new(p1.x, p1.y, p1.z);
    let pt3 = Vec3::new(p3.x, p3.y, p3.z);
    let local_x = (pt1 - pt0).normalize();
    let local_3 = if totpoints == 2 {
        pt3 * 0.001 - pt0
    } else {
        pt3 - pt0
    };
    let normal = local_x.cross(local_3);
    let local_y = normal.cross(local_x).normalize();
    /* `transpose(float2x4(vec4(local_x, -dot(pt0, local_x)),
     * vec4(local_y, -dot(pt0, local_y))))` */
    let m2x4: Float2x4 = [
        [local_x.x, local_x.y, local_x.z, -pt0.dot(local_x)],
        [local_y.x, local_y.y, local_y.z, -pt0.dot(local_y)],
    ];
    transpose_2x4(m2x4)
}

/// `get_legacy_texture_matrix` (verbatim).
pub fn get_legacy_texture_matrix(gps: &BGpdStroke) -> Float4x2 {
    let texture_matrix = get_legacy_stroke_to_texture_matrix(
        gps.uv_translation,
        gps.uv_rotation,
        gps.uv_scale,
    );
    let strokemat = get_legacy_layer_to_stroke_matrix(&gps.points);
    let mut strokemat4x3: Float4x3 = [[0.0; 3]; 4];
    for j in 0..4 {
        strokemat4x3[j][0] = strokemat[j][0];
        strokemat4x3[j][1] = strokemat[j][1];
    }
    strokemat4x3[2][2] = 0.0;
    strokemat4x3[3][2] = 1.0;
    mul_3x2_by_4x3(texture_matrix, strokemat4x3)
}

/// `fcurve_convert_thickness_cb` — legacy thickness curve
/// → new `thickness` anim channel name/scale (verbatim
/// rename logic).
pub fn fcurve_convert_thickness_cb(rna_path: &mut String) {
    /* `gps->thickness` → `point.thickness` anim path */
    *rna_path = "thickness".to_string();
}
