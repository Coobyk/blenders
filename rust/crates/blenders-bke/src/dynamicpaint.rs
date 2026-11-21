//! Port of `intern/dynamicpaint.cc` — DynamicPaint surface
//! management: unique naming, type updates, sample counts,
//! color blending, bounds math.
//!
//! `PORT_PENDING`: `surfaceGenerateGrid`/`dynamicPaint_
//! initAdjacencyData` (grid + adjacency build needs Mesh/
//! KDTree), `dynamicPaint_step`/`doStep` (wave/paint solver —
//! the heavy per-frame sim, needs `Surface` data layout +
//! effectors), `dynamicPaint_updatePaintData`/`applySurface`/
//! `setInitialColor` (ImBuf/vcol/write-back), brush
//! gathering (`surface_getBrushFlags`), image-sequence IO.

/// `gaussianFactors`/`gaussianTotal` (verbatim).
pub const GAUSSIAN_FACTORS: [f32; 5] = [
    0.9477165633097551,
    0.8011704674018264,
    0.5529162965274175,
    0.3108933298345358,
    0.1423971330292934,
];
/// `gaussianTotal`.
pub const GAUSSIAN_TOTAL: f32 = 3.309425;
/// `neighX`/`neighY` (8 neighbors).
pub const NEIGH_X: [i32; 8] = [1, 1, 0, -1, -1, -1, 0, 1];
/// `neighY`.
pub const NEIGH_Y: [i32; 8] = [0, 1, 1, 1, 0, -1, -1, -1];
/// `neighStraightX`/`neighStraightY`.
pub const NEIGH_STRAIGHT_X: [i32; 8] = [1, 0, -1, 0, 1, -1, -1, 1];
/// `neighStraightY`.
pub const NEIGH_STRAIGHT_Y: [i32; 8] = [0, 1, 0, -1, 1, 1, -1, -1];

/// `eDynamicPaint_SurfaceType` verbatim.
pub const MOD_DPAINT_SURFACE_T_PAINT: i8 = 0;
/// `MOD_DPAINT_SURFACE_T_DISPLACE`.
pub const MOD_DPAINT_SURFACE_T_DISPLACE: i8 = 1;
/// `MOD_DPAINT_SURFACE_T_WEIGHT`.
pub const MOD_DPAINT_SURFACE_T_WEIGHT: i8 = 2;
/// `MOD_DPAINT_SURFACE_T_WAVE`.
pub const MOD_DPAINT_SURFACE_T_WAVE: i8 = 3;
/// `eDynamicPaint_SurfaceFormat` verbatim.
pub const MOD_DPAINT_SURFACE_F_PTEX: i8 = 0;
/// `MOD_DPAINT_SURFACE_F_VERTEX`.
pub const MOD_DPAINT_SURFACE_F_VERTEX: i8 = 1;
/// `MOD_DPAINT_SURFACE_F_IMAGESEQ`.
pub const MOD_DPAINT_SURFACE_F_IMAGESEQ: i8 = 2;
/// `eDynamicPaint_SurfaceFlags` bits used here.
pub const MOD_DPAINT_ANTIALIAS: i32 = 1 << 1;
/// `MOD_DPAINT_WAVE_OPEN_BORDERS`.
pub const MOD_DPAINT_WAVE_OPEN_BORDERS: i32 = 1 << 7;
/// `MOD_DPAINT_OUT1`/`MOD_DPAINT_OUT2`.
pub const MOD_DPAINT_OUT1: i32 = 1 << 9;
/// `MOD_DPAINT_OUT2`.
pub const MOD_DPAINT_OUT2: i32 = 1 << 10;
/// `MOD_DPAINT_DO_GRID`.
pub const MOD_DPAINT_DO_GRID: i32 = 1 << 15;
/// `MOD_DPAINT_ACTIVE`.
pub const MOD_DPAINT_ACTIVE: i32 = 1 << 16;
/// `MOD_DPAINT_EVERY_FRAME`.
pub const MOD_DPAINT_EVERY_FRAME: i32 = 1 << 17;
/// `MOD_DPAINT_HAS_EARLY_LOOP`.
pub const MOD_DPAINT_HAS_EARLY_LOOP: i32 = 1 << 18;

/// `Bounds3D` verbatim.
#[derive(Clone, Copy, Default)]
pub struct Bounds3D {
    /// `min`/`max`.
    pub min: [f32; 3],
    /// `max`.
    pub max: [f32; 3],
    /// `valid`.
    pub valid: bool,
}

/// `boundsIntersect` (verbatim — axis 0..1? no: `i=2;i--`
/// iterates 1,0 — i.e. x,y only! verbatim XY check).
pub fn bounds_intersect(b1: &Bounds3D, b2: &Bounds3D) -> bool {
    if !b1.valid || !b2.valid {
        return false;
    }
    for i in (0..2).rev() {
        if !(b1.min[i] <= b2.max[i] && b1.max[i] >= b2.min[i]) {
            return false;
        }
    }
    true
}
/// `boundsIntersectDist` (verbatim).
pub fn bounds_intersect_dist(b1: &Bounds3D, b2: &Bounds3D, dist: f32) -> bool {
    if !b1.valid || !b2.valid {
        return false;
    }
    for i in (0..2).rev() {
        if !(b1.min[i] <= (b2.max[i] + dist) && b1.max[i] >= (b2.min[i] - dist)) {
            return false;
        }
    }
    true
}
/// `boundIntersectPoint` (verbatim).
pub fn bound_intersect_point(b: &Bounds3D, point: [f32; 3], radius: f32) -> bool {
    if !b.valid {
        return false;
    }
    for i in (0..2).rev() {
        if !(b.min[i] <= (point[i] + radius) && b.max[i] >= (point[i] - radius)) {
            return false;
        }
    }
    true
}
/// `boundInsert` (verbatim).
pub fn bound_insert(b: &mut Bounds3D, point: [f32; 3]) {
    if !b.valid {
        b.min = point;
        b.max = point;
        b.valid = true;
        return;
    }
    for i in 0..3 {
        b.min[i] = b.min[i].min(point[i]);
        b.max[i] = b.max[i].max(point[i]);
    }
}
/// `getSurfaceDimension`.
pub fn get_surface_dimension(mb: &Bounds3D) -> f32 {
    (mb.max[0] - mb.min[0])
        .max(mb.max[1] - mb.min[1])
        .max(mb.max[2] - mb.min[2])
}

/// `DynamicPaintSurface` (verbatim DNA fields, handles).
#[derive(Clone, Default)]
pub struct DynamicPaintSurface {
    /// `canvas` parent handle.
    pub canvas: usize,
    /// `format` (`eDynamicPaint_SurfaceFormat`).
    pub format: i8,
    /// `type` (`eDynamicPaint_SurfaceType`).
    pub type_: i8,
    /// `disp_format`, `image_fileformat`, `effect_ui`,
    /// `init_color_type`.
    pub disp_format: i8,
    /// `image_fileformat`.
    pub image_fileformat: i8,
    /// `effect_ui`.
    pub effect_ui: i8,
    /// `init_color_type`.
    pub init_color_type: i8,
    /// `flags`.
    pub flags: i32,
    /// `effect`, `image_output_path`, `output_name[64]`,
    /// `output_name2[64]`.
    pub effect: i32,
    /// `image_output_path`.
    pub image_output_path: String,
    /// `output_name`.
    pub output_name: String,
    /// `output_name2`.
    pub output_name2: String,
    /// `uv_layer_name`.
    pub uv_layer_name: String,
    /// `init_color`, `init_texture`, `init_texture_type`.
    pub init_color: [f32; 4],
    /// `init_texture` handle.
    pub init_texture: usize,
    /// `init_texture_type`.
    pub init_texture_type: i16,
    /// `disp_damp`/`diss_speed`/`color_dry_threshold`/`wave_*
    /// `/gravity`/`influence_scale`/`radius_scale`/`depth_
    /// clamp`/`frame_start`/`frame_end`/`substeps`.
    pub disp_damp: f32,
    /// `diss_speed`.
    pub diss_speed: f32,
    /// `color_dry_threshold`.
    pub color_dry_threshold: f32,
    /// `wave_speed`.
    pub wave_speed: f32,
    /// `wave_timescale`.
    pub wave_timescale: f32,
    /// `wave_spring`.
    pub wave_spring: f32,
    /// `wave_smoothing`.
    pub wave_smoothing: f32,
    /// `wave_damping`.
    pub wave_damping: f32,
    /// `gravity`.
    pub gravity: f32,
    /// `influence_scale`.
    pub influence_scale: f32,
    /// `radius_scale`.
    pub radius_scale: f32,
    /// `depth_clamp`.
    pub depth_clamp: f32,
    /// `frame_start`.
    pub frame_start: i32,
    /// `frame_end`.
    pub frame_end: i32,
    /// `substeps`.
    pub substeps: i32,
    /// `steps`/`current_frame` bookkeeping.
    pub steps: i32,
    /// `current_frame`.
    pub current_frame: i32,
    /// `data` handle (`PaintSurfaceData`).
    pub data: usize,
    /// `name`.
    pub name: String,
    /// `next`/`prev` — index arena handled by caller Vec.
    pub runtime: usize,
}

/// `DynamicPaintCanvasSettings` (verbatim).
#[derive(Default)]
pub struct DynamicPaintCanvasSettings {
    /// `pmd` handle.
    pub pmd: usize,
    /// `surfaces` — `Vec<DynamicPaintSurface>`.
    pub surfaces: Vec<DynamicPaintSurface>,
    /// `active_sur` index.
    pub active_sur: i32,
    /// `flags`.
    pub flags: i32,
    /// `error`.
    pub error: String,
}

/// `get_activeSurface` (verbatim).
pub fn get_active_surface(canvas: &DynamicPaintCanvasSettings) -> Option<usize> {
    let i = canvas.active_sur;
    if i >= 0 && (i as usize) < canvas.surfaces.len() {
        Some(i as usize)
    } else {
        None
    }
}

/// `dynamicPaint_surfaceNumOfPoints` (verbatim; `verts_num`
/// injected).
pub fn dynamicpaint_surface_num_of_points(format: i8, verts_num: usize) -> usize {
    if format == MOD_DPAINT_SURFACE_F_PTEX {
        return 0;
    }
    if format == MOD_DPAINT_SURFACE_F_VERTEX {
        return verts_num;
    }
    0
}

/// `surface_duplicateOutputExists` (verbatim — walk sibling
/// surfaces comparing `output_name`/`output_name2`).
pub fn surface_duplicate_output_exists(
    surfaces: &[DynamicPaintSurface],
    t_index: usize,
    type_: i8,
    format: i8,
    name: &str,
) -> bool {
    for (i, s) in surfaces.iter().enumerate() {
        if i == t_index || s.type_ != type_ || s.format != format {
            continue;
        }
        if (!s.output_name.is_empty() && name == s.output_name)
            || (!s.output_name2.is_empty() && name == s.output_name2)
        {
            return true;
        }
    }
    false
}

/// `surface_duplicateNameExists` (verbatim).
pub fn surface_duplicate_name_exists(
    surfaces: &[DynamicPaintSurface],
    t_index: usize,
    name: &str,
) -> bool {
    surfaces
        .iter()
        .enumerate()
        .any(|(i, s)| i != t_index && s.name == name)
}

/// `dynamicPaintSurface_updateType` (verbatim — `output_*
    /// name` presets by surface type/format).
pub fn dynamicpaint_surface_update_type(surface: &mut DynamicPaintSurface) {
    let (mut name_prefix, mut name_suffix_1, mut name_suffix_2) = ("", "", "");
    if surface.format == MOD_DPAINT_SURFACE_F_IMAGESEQ {
        surface.output_name.clear();
        surface.output_name2.clear();
        surface.flags |= MOD_DPAINT_ANTIALIAS;
        surface.depth_clamp = 1.0;
    } else {
        name_prefix = "dp_";
        surface.flags &= !MOD_DPAINT_ANTIALIAS;
        surface.depth_clamp = 0.0;
    }
    match surface.type_ {
        MOD_DPAINT_SURFACE_T_PAINT => {
            name_suffix_1 = "paintmap";
            name_suffix_2 = "wetmap";
        }
        MOD_DPAINT_SURFACE_T_DISPLACE => {
            name_suffix_1 = "displace";
            name_suffix_2 = "displace";
        }
        MOD_DPAINT_SURFACE_T_WEIGHT => {
            name_suffix_1 = "weight";
            name_suffix_2 = "weight";
        }
        MOD_DPAINT_SURFACE_T_WAVE => {
            name_suffix_1 = "wave";
            name_suffix_2 = "wave";
        }
        _ => {}
    }
    surface.output_name = format!("{}{}", name_prefix, name_suffix_1);
    surface.output_name2 = format!("{}{}", name_prefix, name_suffix_2);
}

/// `surface_totalSamples` (verbatim; `adj_data.total_
/// targets` injected as `Option<i32>`).
pub fn surface_total_samples(
    format: i8,
    flags: i32,
    total_points: i32,
    adj_total_targets: Option<i32>,
) -> i32 {
    if format == MOD_DPAINT_SURFACE_F_IMAGESEQ && (flags & MOD_DPAINT_ANTIALIAS) != 0 {
        return total_points * 5;
    }
    if format == MOD_DPAINT_SURFACE_F_VERTEX && (flags & MOD_DPAINT_ANTIALIAS) != 0 {
        if let Some(t) = adj_total_targets {
            return total_points + t;
        }
    }
    total_points
}

/// `blendColors` (verbatim — non-premultiplied blend).
pub fn blend_colors(t_color: [f32; 3], t_alpha: f32, s_color: [f32; 3], s_alpha: f32) -> [f32; 4] {
    let i_alpha = 1.0 - s_alpha;
    let f_alpha = t_alpha * i_alpha + s_alpha;
    let mut result = [0.0; 4];
    if f_alpha != 0.0 {
        for i in 0..3 {
            result[i] = (t_color[i] * t_alpha * i_alpha + s_color[i] * s_alpha) / f_alpha;
        }
    } else {
        result[..3].copy_from_slice(&t_color);
    }
    result[3] = f_alpha;
    result
}

/// `mixColors` (verbatim).
pub fn mix_colors(
    a_color: &mut [f32; 3],
    a_weight: f32,
    b_color: [f32; 3],
    b_weight: f32,
    ratio: f32,
) -> f32 {
    if b_weight == 0.0 {
        return a_weight * (1.0 - ratio);
    }
    if a_weight == 0.0 {
        *a_color = b_color;
        return b_weight * ratio;
    }
    let weight_ratio = b_weight / (a_weight + b_weight);
    let factor = if ratio <= 0.5 {
        weight_ratio * (ratio * 2.0)
    } else {
        let r = ratio * 2.0 - 1.0;
        weight_ratio * (1.0 - r) + r
    };
    for i in 0..3 {
        a_color[i] = a_color[i] * (1.0 - factor) + b_color[i] * factor;
    }
    (1.0 - factor) * a_weight + factor * b_weight
}
