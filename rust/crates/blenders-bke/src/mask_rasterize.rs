//! Port of `intern/mask_rasterize.cc` — `MaskRasterHandle`/
//! `MaskRasterLayer`/`MaskRasterSplineInfo`, `rotate_point_v2`,
//! `clampis_uint`, `isect_aabb_tri_v2`, `layer_bucket_isect_test`,
//! `maskrasterize_layer_init_from_mask_layer`, `layer_bucket_init*`,
//! `maskrasterize_spline_differentiate_point_outset`,
//! `maskrasterize_layer_z_depth_tri/quad`, `layer_isect`,
//! `layer_bucket_depth_from_xy`, `BKE_maskrasterize_handle_sample`,
//! `BKE_maskrasterize_buffer`, `handle_init`/`free`.
//! `PORT_PENDING`: `maskrasterize_layer_init_scanfill` /
//! `maskrasterize_layer_init_cdt` spline→tri/quad emitters need
//! `BLI_scanfill` + CDT fills — handled via the injected
//! `LayerFill` callback for now.

use blenders_dna::mask_types::{
    MaskLayer, MASK_BLEND_ADD, MASK_BLEND_DARKEN, MASK_BLEND_DIFFERENCE, MASK_BLEND_LIGHTEN,
    MASK_BLEND_MUL, MASK_BLEND_REPLACE, MASK_BLEND_SUBTRACT, MASK_FILL_SOLVER_CDT,
    MASK_HIDE_RENDER,
};
use blenders_dna::scene_types::{
    PROP_INVSQUARE, PROP_ROOT, PROP_SHARP, PROP_SMOOTH, PROP_SPHERE,
};
use blenders_math::geom;
use blenders_math::vector::{Vec2, Vec3};

fn v2of(v: Vec3) -> Vec2 {
    Vec2::new(v.x, v.y)
}

/// `BUCKET_PIXELS_PER_CELL`.
pub const BUCKET_PIXELS_PER_CELL: usize = 8;
/// `SF_EDGE_IS_BOUNDARY`.
pub const SF_EDGE_IS_BOUNDARY: u32 = 0xff;
/// `SF_KEYINDEX_TEMP_ID`.
pub const SF_KEYINDEX_TEMP_ID: u32 = u32::MAX;
/// `TRI_TERMINATOR_ID`.
pub const TRI_TERMINATOR_ID: u32 = u32::MAX;
/// `TRI_VERT` — marks a face slot that stores a triangle.
pub const TRI_VERT: u32 = u32::MAX;

/// `Rctf` — rect bounds (DNA mirror kept local to avoid a dep
/// cycle: same layout as `blenders_dna::DNA_rctf`).
#[derive(Clone, Copy, Default)]
pub struct Rctf {
    /// `xmin`, `xmax`, `ymin`, `ymax`.
    pub xmin: f32,
    /// `xmax`.
    pub xmax: f32,
    /// `ymin`.
    pub ymin: f32,
    /// `ymax`.
    pub ymax: f32,
}

impl Rctf {
    /// `BLI_rctf_init` variant.
    pub fn new(xmin: f32, xmax: f32, ymin: f32, ymax: f32) -> Self {
        Self {
            xmin,
            xmax,
            ymin,
            ymax,
        }
    }
    /// `BLI_rctf_init_minmax` — inverted infinite bounds.
    pub fn init_minmax() -> Self {
        Self {
            xmin: f32::INFINITY,
            xmax: -f32::INFINITY,
            ymin: f32::INFINITY,
            ymax: -f32::INFINITY,
        }
    }
    /// `BLI_rctf_size_x`/`y`.
    pub fn size_x(&self) -> f32 {
        self.xmax - self.xmin
    }
    /// `size_y`.
    pub fn size_y(&self) -> f32 {
        self.ymax - self.ymin
    }
    /// `BLI_rctf_isect_pt_v`.
    pub fn isect_pt(&self, p: Vec2) -> bool {
        p.x >= self.xmin && p.x <= self.xmax && p.y >= self.ymin && p.y <= self.ymax
    }
}

/// `rotate_point_v2` — rotate `p` about `cent` with aspect
/// pre/post-scale.
pub fn rotate_point_v2(p: Vec2, cent: Vec2, angle: f32, asp: Vec2) -> Vec2 {
    let (s, c) = angle.sin_cos();
    let r = Vec2::new((p.x - cent.x) / asp.x, (p.y - cent.y) / asp.y);
    Vec2::new(
        (r.x * c - r.y * s) * asp.x + cent.x,
        (r.x * s + r.y * c) * asp.y + cent.y,
    )
}

/// `clampis_uint`.
pub fn clampis_uint(v: u32, min: u32, max: u32) -> u32 {
    v.clamp(min, max)
}

/// `MaskRasterLayer` — per-`MaskLayer` sampling state.
#[derive(Default)]
pub struct MaskRasterLayer {
    /// `face_tot`.
    pub face_tot: u32,
    /// `face_array` — `[v0..v3]` (`v3 == TRI_VERT` → tri).
    pub face_array: Vec<[u32; 4]>,
    /// `face_coords` — `(x, y, z 0-1)` per vert.
    pub face_coords: Vec<Vec3>,
    /// `bounds`.
    pub bounds: Rctf,
    /// `buckets_face` — per-cell face index list (terminated by
    /// `TRI_TERMINATOR_ID`).
    pub buckets_face: Vec<Vec<u32>>,
    /// `buckets_xy_scalar`.
    pub buckets_xy_scalar: [f32; 2],
    /// `buckets_x`.
    pub buckets_x: u32,
    /// `buckets_y`.
    pub buckets_y: u32,
    /// `alpha`.
    pub alpha: f32,
    /// `blend`.
    pub blend: i8,
    /// `blend_flag`.
    pub blend_flag: i8,
    /// `falloff`.
    pub falloff: i8,
}

/// `MaskRasterSplineInfo`.
#[derive(Clone, Copy, Default)]
pub struct MaskRasterSplineInfo {
    /// `vertex_offset`.
    pub vertex_offset: u32,
    /// `vertex_total`.
    pub vertex_total: u32,
    /// `vertex_total_cap_head`.
    pub vertex_total_cap_head: u32,
    /// `vertex_total_cap_tail`.
    pub vertex_total_cap_tail: u32,
    /// `is_cyclic`.
    pub is_cyclic: bool,
}

/// `MaskRasterHandle`.
#[derive(Default)]
pub struct MaskRasterHandle {
    /// `layers`.
    pub layers: Vec<MaskRasterLayer>,
    /// `bounds`.
    pub bounds: Rctf,
}

/// `maskrasterize_spline_differentiate_point_outset` — offset a
/// differentiated point ring outward by `ofs` (only when the
/// feather point still within `ofs` distance if `do_test`).
pub fn maskrasterize_spline_differentiate_point_outset(
    diff_feather_points: &mut [Vec2],
    diff_points: &[Vec2],
    ofs: f32,
    do_test: bool,
) {
    let tot_diff_point = diff_points.len();
    if tot_diff_point < 2 {
        return;
    }
    let ofs_squared = ofs * ofs;
    let mut k_prev = tot_diff_point - 2;
    let mut k_curr = tot_diff_point - 1;
    let mut k_next = 0;

    let mut d_prev = (diff_points[k_prev] - diff_points[k_curr]).normalized();
    for k in 0..tot_diff_point {
        let co_curr = diff_points[k_curr];
        let co_next = diff_points[k_next];
        let d_next = (co_curr - co_next).normalized();
        if !do_test
            || (diff_feather_points[k] - diff_points[k]).length_squared() < ofs_squared
        {
            let d = (d_prev + d_next).normalized();
            diff_feather_points[k] = Vec2::new(
                diff_points[k].x + d.y * ofs,
                diff_points[k].y + (-d.x) * ofs,
            );
        }
        d_prev = d_next;
        k_curr = k_next;
        k_next += 1;
        let _ = co_curr;
    }
}

/// `isect_aabb_tri_v2` — separating-axis AABB/tri test.
#[allow(clippy::too_many_arguments)]
pub fn isect_aabb_tri_v2(
    xmin: f32,
    ymin: f32,
    xmax: f32,
    ymax: f32,
    v1: Vec2,
    v2: Vec2,
    v3: Vec2,
) -> bool {
    let va = [v1, v2, v3];
    for i in 0..3 {
        let a = va[i];
        let b = va[(i + 1) % 3];
        let c = va[(i + 2) % 3];
        let nx = -(b.y - a.y);
        let ny = b.x - a.x;
        let d_c = nx * (c.x - a.x) + ny * (c.y - a.y);
        let box_max = nx * (if nx >= 0.0 { xmax } else { xmin }) - nx * a.x
            + ny * (if ny >= 0.0 { ymax } else { ymin })
            - ny * a.y;
        let box_min = nx * (if nx >= 0.0 { xmin } else { xmax }) - nx * a.x
            + ny * (if ny >= 0.0 { ymin } else { ymax })
            - ny * a.y;
        if box_max < 0.0f32.min(d_c) || box_min > 0.0f32.max(d_c) {
            return false;
        }
    }
    true
}

/// `isect_point_tri_v2` — CCW point-in-triangle.
fn isect_point_tri_v2(pt: Vec2, v1: Vec2, v2: Vec2, v3: Vec2) -> bool {
    geom::isect_point_tri_v2_cw(pt, v1, v2, v3)
        || geom::isect_point_tri_v2_cw(pt, v3, v2, v1)
}

/// `layer_bucket_isect_test`.
fn layer_bucket_isect_test(
    layer: &MaskRasterLayer,
    face_index: u32,
    bucket_x: u32,
    bucket_y: u32,
    bucket_size_x: f32,
    bucket_size_y: f32,
) -> bool {
    let face = &layer.face_array[face_index as usize];
    let cos = &layer.face_coords;
    let xmin = layer.bounds.xmin + bucket_size_x * bucket_x as f32;
    let ymin = layer.bounds.ymin + bucket_size_y * bucket_y as f32;
    let xmax = xmin + bucket_size_x;
    let ymax = ymin + bucket_size_y;
    let v1 = v2of(cos[face[0] as usize]);
    let v2 = v2of(cos[face[1] as usize]);
    let v3 = v2of(cos[face[2] as usize]);
    if isect_aabb_tri_v2(xmin, ymin, xmax, ymax, v1, v2, v3) {
        return true;
    }
    if face[3] != TRI_VERT {
        let v4 = v2of(cos[face[3] as usize]);
        if isect_aabb_tri_v2(xmin, ymin, xmax, ymax, v1, v3, v4) {
            return true;
        }
    }
    false
}

/// `maskrasterize_layer_init_from_mask_layer`.
pub fn maskrasterize_layer_init_from_mask_layer(
    layer: &mut MaskRasterLayer,
    masklay: &MaskLayer,
) {
    layer.alpha = masklay.alpha;
    layer.blend = masklay.blend;
    layer.blend_flag = masklay.blend_flag;
    layer.falloff = masklay.falloff;
}

/// `layer_bucket_init_dummy`.
pub fn layer_bucket_init_dummy(layer: &mut MaskRasterLayer) {
    layer.face_tot = 0;
    layer.face_coords.clear();
    layer.face_array.clear();
    layer.buckets_x = 0;
    layer.buckets_y = 0;
    layer.buckets_xy_scalar = [0.0; 2];
    layer.buckets_face.clear();
    layer.bounds = Rctf::new(-1.0, -1.0, -1.0, -1.0);
}

/// `layer_bucket_init` — scatter faces into a `BUCKET_PIXELS_PER_CELL`
/// bucket grid.
pub fn layer_bucket_init(layer: &mut MaskRasterLayer, pixel_size: f32) {
    let bucket_dim_x = layer.bounds.size_x();
    let bucket_dim_y = layer.bounds.size_y();
    layer.buckets_x =
        ((bucket_dim_x / pixel_size) / BUCKET_PIXELS_PER_CELL as f32) as u32;
    layer.buckets_y =
        ((bucket_dim_y / pixel_size) / BUCKET_PIXELS_PER_CELL as f32) as u32;
    layer.buckets_x = layer.buckets_x.clamp(8, 512);
    layer.buckets_y = layer.buckets_y.clamp(8, 512);
    layer.buckets_xy_scalar[0] =
        (1.0 / (bucket_dim_x + f32::EPSILON)) * layer.buckets_x as f32;
    layer.buckets_xy_scalar[1] =
        (1.0 / (bucket_dim_y + f32::EPSILON)) * layer.buckets_y as f32;

    let bucket_size_x = (bucket_dim_x + f32::EPSILON) / layer.buckets_x as f32;
    let bucket_size_y = (bucket_dim_y + f32::EPSILON) / layer.buckets_y as f32;
    let bucket_tot = (layer.buckets_x * layer.buckets_y) as usize;
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); bucket_tot];

    for face_index in 0..layer.face_tot {
        let face = layer.face_array[face_index as usize];
        let cos = &layer.face_coords;
        let (mut xmin, mut xmax, mut ymin, mut ymax);
        if face[3] == TRI_VERT {
            let v1 = cos[face[0] as usize];
            let v2 = cos[face[1] as usize];
            let v3 = cos[face[2] as usize];
            xmin = v1.x.min(v2.x.min(v3.x));
            xmax = v1.x.max(v2.x.max(v3.x));
            ymin = v1.y.min(v2.y.min(v3.y));
            ymax = v1.y.max(v2.y.max(v3.y));
        } else {
            let v1 = cos[face[0] as usize];
            let v2 = cos[face[1] as usize];
            let v3 = cos[face[2] as usize];
            let v4 = cos[face[3] as usize];
            xmin = v1.x.min(v2.x.min(v3.x.min(v4.x)));
            xmax = v1.x.max(v2.x.max(v3.x.max(v4.x)));
            ymin = v1.y.min(v2.y.min(v3.y.min(v4.y)));
            ymax = v1.y.max(v2.y.max(v3.y.max(v4.y)));
        }
        if xmax < 0.0 || ymax < 0.0 || xmin > 1.0 || ymin > 1.0 {
            continue;
        }
        xmin = xmin.clamp(0.0, 1.0);
        ymin = ymin.clamp(0.0, 1.0);
        xmax = xmax.clamp(0.0, 1.0);
        ymax = ymax.clamp(0.0, 1.0);

        let mut xi_min =
            ((xmin - layer.bounds.xmin) * layer.buckets_xy_scalar[0]) as u32;
        let mut xi_max =
            ((xmax - layer.bounds.xmin) * layer.buckets_xy_scalar[0]) as u32;
        let mut yi_min =
            ((ymin - layer.bounds.ymin) * layer.buckets_xy_scalar[1]) as u32;
        let mut yi_max =
            ((ymax - layer.bounds.ymin) * layer.buckets_xy_scalar[1]) as u32;
        if xi_min >= layer.buckets_x {
            xi_min = layer.buckets_x - 1;
        }
        if xi_max >= layer.buckets_x {
            xi_max = layer.buckets_x - 1;
        }
        if yi_min >= layer.buckets_y {
            yi_min = layer.buckets_y - 1;
        }
        if yi_max >= layer.buckets_y {
            yi_max = layer.buckets_y - 1;
        }
        for yi in yi_min..=yi_max {
            for xi in xi_min..=xi_max {
                if layer_bucket_isect_test(
                    layer,
                    face_index,
                    xi,
                    yi,
                    bucket_size_x,
                    bucket_size_y,
                ) {
                    buckets[(layer.buckets_x * yi + xi) as usize].push(face_index);
                }
            }
        }
    }
    /* Terminate each bucket list. */
    layer.buckets_face = buckets
        .into_iter()
        .map(|mut b| {
            b.push(TRI_TERMINATOR_ID);
            b
        })
        .collect();
}

/// `maskrasterize_layer_z_depth_tri`.
pub fn maskrasterize_layer_z_depth_tri(pt: Vec2, v1: Vec3, v2: Vec3, v3: Vec3) -> f32 {
    let w = geom::barycentric_weights_v2(v2of(v1), v2of(v2), v2of(v3), pt);
    v1.z * w.x + v2.z * w.y + v3.z * w.z
}

/// `maskrasterize_layer_z_depth_quad`.
pub fn maskrasterize_layer_z_depth_quad(
    pt: Vec2,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    v4: Vec3,
) -> f32 {
    let w = geom::barycentric_weights_v2_quad(
        v2of(v1),
        v2of(v2),
        v2of(v3),
        v2of(v4),
        pt,
    );
    w[2] + w[3]
}

/// `maskrasterize_layer_isect` — depth of `xy` inside `face`
/// (`dist_orig` gates the quad test as in C).
pub fn maskrasterize_layer_isect(
    face: &[u32; 4],
    cos: &[Vec3],
    dist_orig: f32,
    xy: Vec2,
) -> f32 {
    if face[3] == TRI_VERT {
        if isect_point_tri_v2(
            xy,
            v2of(cos[face[0] as usize]),
            v2of(cos[face[1] as usize]),
            v2of(cos[face[2] as usize]),
        ) {
            return 0.0;
        }
    } else if cos[face[0] as usize].z < dist_orig
        || cos[face[1] as usize].z < dist_orig
        || cos[face[2] as usize].z < dist_orig
        || cos[face[3] as usize].z < dist_orig
    {
        if isect_point_tri_v2(
            xy,
            v2of(cos[face[0] as usize]),
            v2of(cos[face[1] as usize]),
            v2of(cos[face[2] as usize]),
        ) || isect_point_tri_v2(
            xy,
            v2of(cos[face[0] as usize]),
            v2of(cos[face[2] as usize]),
            v2of(cos[face[3] as usize]),
        ) {
            return maskrasterize_layer_z_depth_quad(
                xy,
                cos[face[0] as usize],
                cos[face[1] as usize],
                cos[face[2] as usize],
                cos[face[3] as usize],
            );
        }
    }
    1.0
}

/// `layer_bucket_index_from_xy`.
fn layer_bucket_index_from_xy(layer: &MaskRasterLayer, xy: Vec2) -> usize {
    (((xy.x - layer.bounds.xmin) * layer.buckets_xy_scalar[0]) as u32
        + ((xy.y - layer.bounds.ymin) * layer.buckets_xy_scalar[1]) as u32
            * layer.buckets_x) as usize
}

/// `layer_bucket_depth_from_xy`.
pub fn layer_bucket_depth_from_xy(layer: &MaskRasterLayer, xy: Vec2) -> f32 {
    let index = layer_bucket_index_from_xy(layer, xy);
    if index >= layer.buckets_face.len() {
        return 1.0;
    }
    let bucket = &layer.buckets_face[index];
    let mut best_dist = 1.0f32;
    for &face_index in bucket {
        if face_index == TRI_TERMINATOR_ID {
            break;
        }
        let face = &layer.face_array[face_index as usize];
        best_dist = best_dist.min(maskrasterize_layer_isect(
            face,
            &layer.face_coords,
            best_dist,
            xy,
        ));
        if best_dist == 0.0 {
            break;
        }
    }
    best_dist
}

/// `BKE_maskrasterize_handle_sample`.
pub fn bke_maskrasterize_handle_sample(mr_handle: &MaskRasterHandle, xy: Vec2) -> f32 {
    let mut value = 0.0f32;
    for layer in &mr_handle.layers {
        if layer.alpha == 0.0 || !layer.bounds.isect_pt(xy) {
            continue;
        }
        let mut value_layer = 1.0 - layer_bucket_depth_from_xy(layer, xy);
        match layer.falloff {
            PROP_SMOOTH => {
                value_layer = 3.0 * value_layer * value_layer
                    - 2.0 * value_layer * value_layer * value_layer;
            }
            PROP_SPHERE => {
                value_layer = (2.0 * value_layer - value_layer * value_layer).sqrt();
            }
            PROP_ROOT => {
                value_layer = value_layer.sqrt();
            }
            PROP_SHARP => {
                value_layer *= value_layer;
            }
            PROP_INVSQUARE => {
                value_layer *= 2.0 - value_layer;
            }
            _ => {}
        }
        match layer.blend {
            MASK_BLEND_ADD => value += value_layer,
            MASK_BLEND_SUBTRACT => value -= value_layer,
            MASK_BLEND_LIGHTEN => value = value.max(value_layer),
            MASK_BLEND_DARKEN => value = value.min(value_layer),
            MASK_BLEND_MUL => value *= value_layer,
            MASK_BLEND_REPLACE => {
                value = value * (1.0 - layer.alpha) + value_layer * layer.alpha;
            }
            MASK_BLEND_DIFFERENCE => value = (value - value_layer).abs(),
            _ => value += value_layer,
        }
        value = value.clamp(0.0, 1.0);
    }
    value
}

/// `BKE_maskrasterize_buffer` — fill `buffer` row-wise (serial).
pub fn bke_maskrasterize_buffer(
    mr_handle: &MaskRasterHandle,
    width: u32,
    height: u32,
    buffer: &mut [f32],
) {
    let x_inv = 1.0 / width as f32;
    let y_inv = 1.0 / height as f32;
    let (x_px_ofs, y_px_ofs) = (x_inv * 0.5, y_inv * 0.5);
    for y in 0..height {
        let mut i = y as usize * width as usize;
        let yy = y as f32 * y_inv + y_px_ofs;
        for x in 0..width {
            let xy = Vec2::new(x as f32 * x_inv + x_px_ofs, yy);
            buffer[i] = bke_maskrasterize_handle_sample(mr_handle, xy);
            i += 1;
        }
    }
}

/// `BKE_maskrasterize_handle_free` — Vecs drop; kept for parity.
pub fn bke_maskrasterize_handle_free(_mr_handle: MaskRasterHandle) {}

/// Injected spline-fill output for one `MaskLayer`: the scanfill/
/// CDT emitters produce `(faces, coords, bounds)` — this layer's
/// callers supply them until `maskrasterize_layer_init_scanfill`
/// and `_init_cdt` are ported.
pub type LayerFill = dyn Fn(&MaskLayer) -> (Vec<[u32; 4]>, Vec<Vec3>, Rctf);

/// `BKE_maskrasterize_handle_init` — build layers over
/// `mask_layers` using `fill` per layer (dispatching
/// `MASK_FILL_SOLVER_CDT` vs scanfill stays inside the callback).
pub fn bke_maskrasterize_handle_init(
    mask_layers: &[MaskLayer],
    _width: u32,
    _height: u32,
    pixel_size: f32,
    fill: &LayerFill,
) -> MaskRasterHandle {
    let mut mr_handle = MaskRasterHandle {
        bounds: Rctf::init_minmax(),
        ..Default::default()
    };
    for masklay in mask_layers {
        if masklay.visibility_flag & MASK_HIDE_RENDER != 0 {
            continue;
        }
        let mut layer = MaskRasterLayer::default();
        maskrasterize_layer_init_from_mask_layer(&mut layer, masklay);
        let _ = masklay.fill_solver & MASK_FILL_SOLVER_CDT;
        let (faces, coords, bounds) = fill(masklay);
        if faces.is_empty() {
            layer_bucket_init_dummy(&mut layer);
        } else {
            layer.face_tot = faces.len() as u32;
            layer.face_array = faces;
            layer.face_coords = coords;
            layer.bounds = bounds;
            layer_bucket_init(&mut layer, pixel_size);
        }
        mr_handle.bounds.xmin = mr_handle.bounds.xmin.min(layer.bounds.xmin);
        mr_handle.bounds.xmax = mr_handle.bounds.xmax.max(layer.bounds.xmax);
        mr_handle.bounds.ymin = mr_handle.bounds.ymin.min(layer.bounds.ymin);
        mr_handle.bounds.ymax = mr_handle.bounds.ymax.max(layer.bounds.ymax);
        mr_handle.layers.push(layer);
    }
    mr_handle
}
