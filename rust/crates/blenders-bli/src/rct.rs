//! Port of `rct.cc` — 2D rectangle (`rcti`/`rctf`) utilities.

use blenders_math::vector::Vec2;
use blenders_math::geom::isect_seg_seg_v2;
use blenders_math::matrix::Mat4;

/// `rcti`.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Rcti {
    pub xmin: i32,
    pub xmax: i32,
    pub ymin: i32,
    pub ymax: i32,
}

/// `rctf`.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Rctf {
    pub xmin: f32,
    pub xmax: f32,
    pub ymin: f32,
    pub ymax: f32,
}

/// `BLI_rcti_is_empty`.
pub fn rcti_is_empty(r: &Rcti) -> bool {
    r.xmin >= r.xmax || r.ymin >= r.ymax
}
/// `BLI_rctf_is_empty`.
pub fn rctf_is_empty(r: &Rctf) -> bool {
    r.xmin >= r.xmax || r.ymin >= r.ymax
}
/// `BLI_rcti_isect_x`.
pub fn rcti_isect_x(r: &Rcti, x: i32) -> bool {
    x >= r.xmin && x <= r.xmax
}
/// `BLI_rcti_isect_y`.
pub fn rcti_isect_y(r: &Rcti, y: i32) -> bool {
    y >= r.ymin && y <= r.ymax
}
/// `BLI_rcti_isect_pt`.
pub fn rcti_isect_pt(r: &Rcti, x: i32, y: i32) -> bool {
    rcti_isect_x(r, x) && rcti_isect_y(r, y)
}
/// `BLI_rcti_isect_pt_v`.
pub fn rcti_isect_pt_v(r: &Rcti, xy: [i32; 2]) -> bool {
    rcti_isect_pt(r, xy[0], xy[1])
}
/// `BLI_rctf_isect_x` / `_y` / `_pt` / `_pt_v`.
pub fn rctf_isect_x(r: &Rctf, x: f32) -> bool {
    x >= r.xmin && x <= r.xmax
}
pub fn rctf_isect_y(r: &Rctf, y: f32) -> bool {
    y >= r.ymin && y <= r.ymax
}
pub fn rctf_isect_pt(r: &Rctf, x: f32, y: f32) -> bool {
    rctf_isect_x(r, x) && rctf_isect_y(r, y)
}
pub fn rctf_isect_pt_v(r: &Rctf, xy: Vec2) -> bool {
    rctf_isect_pt(r, xy.x, xy.y)
}

/// `BLI_rcti_length_x` — distance from x to rect horizontally.
pub fn rcti_length_x(r: &Rcti, x: i32) -> i32 {
    if x < r.xmin {
        r.xmin - x
    } else if x > r.xmax {
        x - r.xmax
    } else {
        0
    }
}
/// `BLI_rcti_length_y`.
pub fn rcti_length_y(r: &Rcti, y: i32) -> i32 {
    if y < r.ymin {
        r.ymin - y
    } else if y > r.ymax {
        y - r.ymax
    } else {
        0
    }
}
/// `BLI_rctf_length_x` / `_y`.
pub fn rctf_length_x(r: &Rctf, x: f32) -> f32 {
    if x < r.xmin {
        r.xmin - x
    } else if x > r.xmax {
        x - r.xmax
    } else {
        0.0
    }
}
pub fn rctf_length_y(r: &Rctf, y: f32) -> f32 {
    if y < r.ymin {
        r.ymin - y
    } else if y > r.ymax {
        y - r.ymax
    } else {
        0.0
    }
}

/// `BLI_rctf_inside_rctf` / `BLI_rcti_inside_rcti` — a fully inside b.
pub fn rctf_inside_rctf(a: &Rctf, b: &Rctf) -> bool {
    b.xmin <= a.xmin && b.xmax >= a.xmax && b.ymin <= a.ymin && b.ymax >= a.ymax
}
pub fn rcti_inside_rcti(a: &Rcti, b: &Rcti) -> bool {
    b.xmin <= a.xmin && b.xmax >= a.xmax && b.ymin <= a.ymin && b.ymax >= a.ymax
}

/// `isect_segments_i` (file-local in rct.cc).
fn isect_segments_i(v1: [i32; 2], v2: [i32; 2], v3: [i32; 2], v4: [i32; 2]) -> bool {
    let div = ((v2[0] - v1[0]) as f64) * ((v4[1] - v3[1]) as f64)
        - ((v2[1] - v1[1]) as f64) * ((v4[0] - v3[0]) as f64);
    if div == 0.0 {
        return true; // co-linear
    }
    let lambda = ((v1[1] - v3[1]) as f64 * (v4[0] - v3[0]) as f64
        - (v1[0] - v3[0]) as f64 * (v4[1] - v3[1]) as f64)
        / div;
    let mu = ((v1[1] - v3[1]) as f64 * (v2[0] - v1[0]) as f64
        - (v1[0] - v3[0]) as f64 * (v2[1] - v1[1]) as f64)
        / div;
    (0.0..=1.0).contains(&lambda) && (0.0..=1.0).contains(&mu)
}
/// `isect_segments_fl` (file-local).
fn isect_segments_fl(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2], v4: [f32; 2]) -> bool {
    let div = (v2[0] - v1[0]) as f64 * (v4[1] - v3[1]) as f64
        - (v2[1] - v1[1]) as f64 * (v4[0] - v3[0]) as f64;
    if div == 0.0 {
        return true;
    }
    let lambda = ((v1[1] - v3[1]) as f64 * (v4[0] - v3[0]) as f64
        - (v1[0] - v3[0]) as f64 * (v4[1] - v3[1]) as f64)
        / div;
    let mu = ((v1[1] - v3[1]) as f64 * (v2[0] - v1[0]) as f64
        - (v1[0] - v3[0]) as f64 * (v2[1] - v1[1]) as f64)
        / div;
    (0.0..=1.0).contains(&lambda) && (0.0..=1.0).contains(&mu)
}

/// `BLI_rcti_isect_segment`.
pub fn rcti_isect_segment(rect: &Rcti, s1: [i32; 2], s2: [i32; 2]) -> bool {
    if s1[0] < rect.xmin && s2[0] < rect.xmin {
        return false;
    }
    if s1[0] > rect.xmax && s2[0] > rect.xmax {
        return false;
    }
    if s1[1] < rect.ymin && s2[1] < rect.ymin {
        return false;
    }
    if s1[1] > rect.ymax && s2[1] > rect.ymax {
        return false;
    }
    if rcti_isect_pt_v(rect, s1) || rcti_isect_pt_v(rect, s2) {
        return true;
    }
    if isect_segments_i(s1, s2, [rect.xmin, rect.ymin], [rect.xmax, rect.ymax]) {
        return true;
    }
    isect_segments_i(s1, s2, [rect.xmin, rect.ymax], [rect.xmax, rect.ymin])
}
/// `BLI_rctf_isect_segment`.
pub fn rctf_isect_segment(rect: &Rctf, s1: Vec2, s2: Vec2) -> bool {
    let s1a = [s1.x, s1.y];
    let s2a = [s2.x, s2.y];
    if s1a[0] < rect.xmin && s2a[0] < rect.xmin {
        return false;
    }
    if s1a[0] > rect.xmax && s2a[0] > rect.xmax {
        return false;
    }
    if s1a[1] < rect.ymin && s2a[1] < rect.ymin {
        return false;
    }
    if s1a[1] > rect.ymax && s2a[1] > rect.ymax {
        return false;
    }
    if rctf_isect_pt_v(rect, s1) || rctf_isect_pt_v(rect, s2) {
        return true;
    }
    if isect_segments_fl(s1a, s2a, [rect.xmin, rect.ymin], [rect.xmax, rect.ymax]) {
        return true;
    }
    isect_segments_fl(s1a, s2a, [rect.xmin, rect.ymax], [rect.xmax, rect.ymin])
}

/// `BLI_rcti_isect_circle` / `BLI_rctf_isect_circle`.
pub fn rcti_isect_circle(rect: &Rcti, xy: Vec2, radius: f32) -> bool {
    let c = Rctf {
        xmin: rect.xmin as f32,
        xmax: rect.xmax as f32,
        ymin: rect.ymin as f32,
        ymax: rect.ymax as f32,
    };
    rctf_isect_circle(&c, xy, radius)
}
pub fn rctf_isect_circle(rect: &Rctf, xy: Vec2, radius: f32) -> bool {
    let dx = rctf_length_x(rect, xy.x);
    let dy = rctf_length_y(rect, xy.y);
    dx * dx + dy * dy <= radius * radius
}

/// `BLI_rctf_union` / `BLI_rcti_union`.
pub fn rctf_union(a: &mut Rctf, b: &Rctf) {
    a.xmin = a.xmin.min(b.xmin);
    a.xmax = a.xmax.max(b.xmax);
    a.ymin = a.ymin.min(b.ymin);
    a.ymax = a.ymax.max(b.ymax);
}
/// `BLI_rctf_union_x` / `_y`.
pub fn rctf_union_x(r: &mut Rctf, x: f32) {
    r.xmin = r.xmin.min(x);
    r.xmax = r.xmax.max(x);
}
pub fn rctf_union_y(r: &mut Rctf, y: f32) {
    r.ymin = r.ymin.min(y);
    r.ymax = r.ymax.max(y);
}
pub fn rcti_union(a: &mut Rcti, b: &Rcti) {
    a.xmin = a.xmin.min(b.xmin);
    a.xmax = a.xmax.max(b.xmax);
    a.ymin = a.ymin.min(b.ymin);
    a.ymax = a.ymax.max(b.ymax);
}

/// `BLI_rctf_init` / `BLI_rcti_init`.
pub fn rctf_init(xmin: f32, xmax: f32, ymin: f32, ymax: f32) -> Rctf {
    Rctf { xmin, xmax, ymin, ymax }
}
pub fn rcti_init(xmin: i32, xmax: i32, ymin: i32, ymax: i32) -> Rcti {
    Rcti { xmin, xmax, ymin, ymax }
}

/// `BLI_rctf_is_valid` / `BLI_rcti_is_valid`.
pub fn rctf_is_valid(r: &Rctf) -> bool {
    r.xmin <= r.xmax && r.ymin <= r.ymax
}
pub fn rcti_is_valid(r: &Rcti) -> bool {
    r.xmin <= r.xmax && r.ymin <= r.ymax
}
/// `BLI_rctf_sanitize` / `BLI_rcti_sanitize`.
pub fn rctf_sanitize(r: &mut Rctf) {
    if r.xmin > r.xmax {
        std::mem::swap(&mut r.xmin, &mut r.xmax);
    }
    if r.ymin > r.ymax {
        std::mem::swap(&mut r.ymin, &mut r.ymax);
    }
}
pub fn rcti_sanitize(r: &mut Rcti) {
    if r.xmin > r.xmax {
        std::mem::swap(&mut r.xmin, &mut r.xmax);
    }
    if r.ymin > r.ymax {
        std::mem::swap(&mut r.ymin, &mut r.ymax);
    }
}

/// `BLI_rctf_init_pt_radius` / `BLI_rcti_init_pt_radius`.
pub fn rctf_init_pt_radius(xy: Vec2, size: f32) -> Rctf {
    Rctf {
        xmin: xy.x - size,
        xmax: xy.x + size,
        ymin: xy.y - size,
        ymax: xy.y + size,
    }
}
pub fn rcti_init_pt_radius(xy: [i32; 2], size: i32) -> Rcti {
    Rcti {
        xmin: xy[0] - size,
        xmax: xy[0] + size,
        ymin: xy[1] - size,
        ymax: xy[1] + size,
    }
}

/// `BLI_rcti_init_minmax` / `BLI_rctf_init_minmax` — inverted extremes.
pub fn rcti_init_minmax() -> Rcti {
    Rcti {
        xmin: i32::MAX,
        xmax: i32::MIN,
        ymin: i32::MAX,
        ymax: i32::MIN,
    }
}
pub fn rctf_init_minmax() -> Rctf {
    Rctf {
        xmin: f32::MAX,
        xmax: -f32::MAX,
        ymin: f32::MAX,
        ymax: -f32::MAX,
    }
}

/// `BLI_rcti_do_minmax_v` / `BLI_rcti_do_minmax_rcti` / `BLI_rctf_do_minmax_v`.
pub fn rcti_do_minmax_v(r: &mut Rcti, xy: [i32; 2]) {
    r.xmin = r.xmin.min(xy[0]);
    r.xmax = r.xmax.max(xy[0]);
    r.ymin = r.ymin.min(xy[1]);
    r.ymax = r.ymax.max(xy[1]);
}
pub fn rcti_do_minmax_rcti(r: &mut Rcti, o: &Rcti) {
    r.xmin = r.xmin.min(o.xmin);
    r.xmax = r.xmax.max(o.xmax);
    r.ymin = r.ymin.min(o.ymin);
    r.ymax = r.ymax.max(o.ymax);
}
pub fn rctf_do_minmax_v(r: &mut Rctf, xy: Vec2) {
    r.xmin = r.xmin.min(xy.x);
    r.xmax = r.xmax.max(xy.x);
    r.ymin = r.ymin.min(xy.y);
    r.ymax = r.ymax.max(xy.y);
}

/// `BLI_rctf_transform_pt_v`.
pub fn rctf_transform_pt_v(dst: &Rctf, src: &Rctf, xy_src: Vec2) -> Vec2 {
    Vec2::new(
        dst.xmin + (dst.xmax - dst.xmin) * ((xy_src.x - src.xmin) / (src.xmax - src.xmin)),
        dst.ymin + (dst.ymax - dst.ymin) * ((xy_src.y - src.ymin) / (src.ymax - src.ymin)),
    )
}

/// `BLI_rctf_transform_calc_m4_pivot_min_ex` — scale+translate Mat4.
pub fn rctf_transform_calc_m4_pivot_min_ex(
    dst: &Rctf, src: &Rctf, matrix: &mut Mat4, x: usize, y: usize,
) {
    debug_assert!(x < 3 && y < 3);
    *matrix = Mat4::IDENTITY;
    matrix.cols[x][x] = rctf_size_x(src) / rctf_size_x(dst);
    matrix.cols[y][y] = rctf_size_y(src) / rctf_size_y(dst);
    matrix.cols[3][x] = (src.xmin - dst.xmin) * matrix.cols[x][x];
    matrix.cols[3][y] = (src.ymin - dst.ymin) * matrix.cols[y][y];
}
/// `BLI_rctf_transform_calc_m4_pivot_min`.
pub fn rctf_transform_calc_m4_pivot_min(dst: &Rctf, src: &Rctf, matrix: &mut Mat4) {
    rctf_transform_calc_m4_pivot_min_ex(dst, src, matrix, 0, 1);
}

/// `BLI_rcti_translate` / `BLI_rctf_translate`.
pub fn rcti_translate(r: &mut Rcti, x: i32, y: i32) {
    r.xmin += x;
    r.xmax += x;
    r.ymin += y;
    r.ymax += y;
}
pub fn rctf_translate(r: &mut Rctf, x: f32, y: f32) {
    r.xmin += x;
    r.xmax += x;
    r.ymin += y;
    r.ymax += y;
}
/// `BLI_rcti_mul` / `BLI_rctf_mul`.
pub fn rcti_mul(r: &mut Rcti, factor: i32) {
    r.xmin *= factor;
    r.xmax *= factor;
    r.ymin *= factor;
    r.ymax *= factor;
}
pub fn rctf_mul(r: &mut Rctf, factor: f32) {
    r.xmin *= factor;
    r.xmax *= factor;
    r.ymin *= factor;
    r.ymax *= factor;
}
/// `BLI_rcti_recenter` / `BLI_rctf_recenter`.
pub fn rcti_recenter(r: &mut Rcti, x: i32, y: i32) {
    let dx = rcti_size_x(r) / 2;
    let dy = rcti_size_y(r) / 2;
    *r = Rcti {
        xmin: x - dx,
        xmax: x + dx,
        ymin: y - dy,
        ymax: y + dy,
    };
}
pub fn rctf_recenter(r: &mut Rctf, x: f32, y: f32) {
    let dx = rctf_size_x(r) / 2.0;
    let dy = rctf_size_y(r) / 2.0;
    *r = Rctf {
        xmin: x - dx,
        xmax: x + dx,
        ymin: y - dy,
        ymax: y + dy,
    };
}
/// `BLI_rcti_resize_x` / `_y` / `BLI_rcti_resize`.
pub fn rcti_resize_x(r: &mut Rcti, x: i32) {
    let c = r.xmin + (r.xmax - r.xmin) / 2;
    r.xmin = c - x / 2;
    r.xmax = c + x - x / 2;
}
pub fn rcti_resize_y(r: &mut Rcti, y: i32) {
    let c = r.ymin + (r.ymax - r.ymin) / 2;
    r.ymin = c - y / 2;
    r.ymax = c + y - y / 2;
}
pub fn rcti_resize(r: &mut Rcti, x: i32, y: i32) {
    rcti_resize_x(r, x);
    rcti_resize_y(r, y);
}
/// `BLI_rcti_pad` / `BLI_rctf_pad`.
pub fn rcti_pad(r: &mut Rcti, pad_x: i32, pad_y: i32) {
    r.xmin -= pad_x;
    r.xmax += pad_x;
    r.ymin -= pad_y;
    r.ymax += pad_y;
}
pub fn rctf_pad(r: &mut Rctf, pad_x: f32, pad_y: f32) {
    r.xmin -= pad_x;
    r.xmax += pad_x;
    r.ymin -= pad_y;
    r.ymax += pad_y;
}
/// `BLI_rctf_resize_x` / `_y` / `BLI_rctf_resize`.
pub fn rctf_resize_x(r: &mut Rctf, x: f32) {
    let c = (r.xmin + r.xmax) / 2.0;
    r.xmin = c - x / 2.0;
    r.xmax = c + x / 2.0;
}
pub fn rctf_resize_y(r: &mut Rctf, y: f32) {
    let c = (r.ymin + r.ymax) / 2.0;
    r.ymin = c - y / 2.0;
    r.ymax = c + y / 2.0;
}
pub fn rctf_resize(r: &mut Rctf, x: f32, y: f32) {
    rctf_resize_x(r, x);
    rctf_resize_y(r, y);
}
/// `BLI_rcti_scale` / `BLI_rctf_scale`.
pub fn rcti_scale(r: &mut Rcti, scale: f32) {
    let c = [(r.xmin + r.xmax) as f32 / 2.0, (r.ymin + r.ymax) as f32 / 2.0];
    let hx = (r.xmax - r.xmin) as f32 / 2.0 * scale;
    let hy = (r.ymax - r.ymin) as f32 / 2.0 * scale;
    r.xmin = (c[0] - hx) as i32;
    r.xmax = (c[0] + hx) as i32;
    r.ymin = (c[1] - hy) as i32;
    r.ymax = (c[1] + hy) as i32;
}
pub fn rctf_scale(r: &mut Rctf, scale: f32) {
    let cx = (r.xmin + r.xmax) / 2.0;
    let cy = (r.ymin + r.ymax) / 2.0;
    let hx = (r.xmax - r.xmin) / 2.0 * scale;
    let hy = (r.ymax - r.ymin) / 2.0 * scale;
    r.xmin = cx - hx;
    r.xmax = cx + hx;
    r.ymin = cy - hy;
    r.ymax = cy + hy;
}

/// `BLI_rctf_pad_y` — pad so the rect's fraction of `boundary_size` expands.
pub fn rctf_pad_y(r: &mut Rctf, boundary_size: f32, pad_min: f32, pad_max: f32) {
    debug_assert!(pad_max >= 0.0 && pad_min >= 0.0 && boundary_size > 0.0);
    let total_pad = pad_max + pad_min;
    if total_pad == 0.0 {
        return;
    }
    let local_pad = rctf_size_y(r) / (boundary_size - total_pad);
    r.ymax += local_pad * pad_max;
    r.ymin -= local_pad * pad_min;
}

/// `BLI_rctf_interp`.
pub fn rctf_interp(r: &mut Rctf, a: &Rctf, b: &Rctf, fac: f32) {
    let ifac = 1.0 - fac;
    r.xmin = a.xmin * ifac + b.xmin * fac;
    r.xmax = a.xmax * ifac + b.xmax * fac;
    r.ymin = a.ymin * ifac + b.ymin * fac;
    r.ymax = a.ymax * ifac + b.ymax * fac;
}

/// `BLI_rctf_clamp_pt_v` / `BLI_rcti_clamp_pt_v` — clamp point into rect.
pub fn rctf_clamp_pt_v(r: &Rctf, xy: &mut Vec2) -> bool {
    let mut changed = false;
    if xy.x < r.xmin {
        xy.x = r.xmin;
        changed = true;
    }
    if xy.x > r.xmax {
        xy.x = r.xmax;
        changed = true;
    }
    if xy.y < r.ymin {
        xy.y = r.ymin;
        changed = true;
    }
    if xy.y > r.ymax {
        xy.y = r.ymax;
        changed = true;
    }
    changed
}
pub fn rcti_clamp_pt_v(r: &Rcti, xy: &mut [i32; 2]) -> bool {
    let mut changed = false;
    if xy[0] < r.xmin {
        xy[0] = r.xmin;
        changed = true;
    }
    if xy[0] > r.xmax {
        xy[0] = r.xmax;
        changed = true;
    }
    if xy[1] < r.ymin {
        xy[1] = r.ymin;
        changed = true;
    }
    if xy[1] > r.ymax {
        xy[1] = r.ymax;
        changed = true;
    }
    changed
}

/// `BLI_rctf_clamp` — clamp rect inside bounds; returns (changed, offset).
pub fn rctf_clamp(rect: &mut Rctf, bounds: &Rctf) -> (bool, Vec2) {
    let mut changed = false;
    let mut r_xy = Vec2::ZERO;
    let w = rect.xmax - rect.xmin;
    let h = rect.ymax - rect.ymin;
    if rect.xmin < bounds.xmin {
        r_xy.x = bounds.xmin - rect.xmin;
        changed = true;
    }
    if rect.xmin + w > bounds.xmax {
        r_xy.x = bounds.xmax - rect.xmin - w;
        changed = true;
    }
    if rect.ymin < bounds.ymin {
        r_xy.y = bounds.ymin - rect.ymin;
        changed = true;
    }
    if rect.ymin + h > bounds.ymax {
        r_xy.y = bounds.ymax - rect.ymin - h;
        changed = true;
    }
    rctf_translate(rect, r_xy.x, r_xy.y);
    (changed, r_xy)
}
/// `BLI_rcti_clamp`.
pub fn rcti_clamp(rect: &mut Rcti, bounds: &Rcti) -> (bool, [i32; 2]) {
    let mut changed = false;
    let mut r_xy = [0; 2];
    let w = rect.xmax - rect.xmin;
    let h = rect.ymax - rect.ymin;
    if rect.xmin < bounds.xmin {
        r_xy[0] = bounds.xmin - rect.xmin;
        changed = true;
    }
    if rect.xmin + w > bounds.xmax {
        r_xy[0] = bounds.xmax - rect.xmin - w;
        changed = true;
    }
    if rect.ymin < bounds.ymin {
        r_xy[1] = bounds.ymin - rect.ymin;
        changed = true;
    }
    if rect.ymin + h > bounds.ymax {
        r_xy[1] = bounds.ymax - rect.ymin - h;
        changed = true;
    }
    rcti_translate(rect, r_xy[0], r_xy[1]);
    (changed, r_xy)
}

/// `BLI_rctf_compare` / `BLI_rcti_compare`.
pub fn rctf_compare(a: &Rctf, b: &Rctf, limit: f32) -> bool {
    (a.xmin - b.xmin).abs() <= limit
        && (a.xmax - b.xmax).abs() <= limit
        && (a.ymin - b.ymin).abs() <= limit
        && (a.ymax - b.ymax).abs() <= limit
}
pub fn rcti_compare(a: &Rcti, b: &Rcti) -> bool {
    a == b
}

/// `BLI_rctf_isect` / `BLI_rcti_isect` — intersect two rects.
pub fn rctf_isect(src1: &Rctf, src2: &Rctf) -> Option<Rctf> {
    let xmin = src1.xmin.max(src2.xmin);
    let xmax = src1.xmax.min(src2.xmax);
    let ymin = src1.ymin.max(src2.ymin);
    let ymax = src1.ymax.min(src2.ymax);
    if xmin <= xmax && ymin <= ymax {
        Some(Rctf { xmin, xmax, ymin, ymax })
    } else {
        None
    }
}
pub fn rcti_isect(src1: &Rcti, src2: &Rcti) -> Option<Rcti> {
    let xmin = src1.xmin.max(src2.xmin);
    let xmax = src1.xmax.min(src2.xmax);
    let ymin = src1.ymin.max(src2.ymin);
    let ymax = src1.ymax.min(src2.ymax);
    if xmin <= xmax && ymin <= ymax {
        Some(Rcti { xmin, xmax, ymin, ymax })
    } else {
        None
    }
}

/// `BLI_rctf_isect_rect_x` / `_y` — overlapping x/y range.
pub fn rctf_isect_rect_x(s1: &Rctf, s2: &Rctf) -> Option<[f32; 2]> {
    let lo = s1.xmin.max(s2.xmin);
    let hi = s1.xmax.min(s2.xmax);
    if lo < hi { Some([lo, hi]) } else { None }
}
pub fn rctf_isect_rect_y(s1: &Rctf, s2: &Rctf) -> Option<[f32; 2]> {
    let lo = s1.ymin.max(s2.ymin);
    let hi = s1.ymax.min(s2.ymax);
    if lo < hi { Some([lo, hi]) } else { None }
}
pub fn rcti_isect_rect_x(s1: &Rcti, s2: &Rcti) -> Option<[i32; 2]> {
    let lo = s1.xmin.max(s2.xmin);
    let hi = s1.xmax.min(s2.xmax);
    if lo < hi { Some([lo, hi]) } else { None }
}
pub fn rcti_isect_rect_y(s1: &Rcti, s2: &Rcti) -> Option<[i32; 2]> {
    let lo = s1.ymin.max(s2.ymin);
    let hi = s1.ymax.min(s2.ymax);
    if lo < hi { Some([lo, hi]) } else { None }
}

/// `BLI_rcti_rctf_copy`/`_floor`/`_round`/`BLI_rctf_rcti_copy`.
pub fn rcti_rctf_copy(src: &Rctf) -> Rcti {
    Rcti {
        xmin: src.xmin as i32,
        xmax: src.xmax as i32 + 1,
        ymin: src.ymin as i32,
        ymax: src.ymax as i32 + 1,
    }
}
pub fn rcti_rctf_copy_floor(src: &Rctf) -> Rcti {
    Rcti {
        xmin: src.xmin.floor() as i32,
        xmax: src.xmax.floor() as i32,
        ymin: src.ymin.floor() as i32,
        ymax: src.ymax.floor() as i32,
    }
}
pub fn rcti_rctf_copy_round(src: &Rctf) -> Rcti {
    Rcti {
        xmin: (src.xmin + 0.5) as i32,
        xmax: (src.xmax + 0.5) as i32,
        ymin: (src.ymin + 0.5) as i32,
        ymax: (src.ymax + 0.5) as i32,
    }
}
pub fn rctf_rcti_copy(src: &Rcti) -> Rctf {
    Rctf {
        xmin: src.xmin as f32,
        xmax: src.xmax as f32,
        ymin: src.ymin as f32,
        ymax: src.ymax as f32,
    }
}

/// `print_rctf` / `print_rcti`.
pub fn print_rctf(s: &str, r: &Rctf) {
    println!("{}: xmin %.3f xmax %.3f ymin %.3f ymax %.3f (%.3f x %.3f)\n", s,);
    let _ = r;
}
pub fn print_rcti(s: &str, r: &Rcti) {
    println!(
        "{}: xmin {} xmax {} ymin {} ymax {} ({} x {})",
        s,
        r.xmin,
        r.xmax,
        r.ymin,
        r.ymax,
        r.xmax - r.xmin,
        r.ymax - r.ymin
    );
}

/// `BLI_rctf_rotate_expand` — grow rect to cover `src` rotated about its center.
pub fn rctf_rotate_expand(dst: &mut Rctf, src: &Rctf, angle: f32) {
    let (s, c) = angle.sin_cos();
    let cent = [rctf_cent_x(src), rctf_cent_y(src)];
    let mut corner_max = [0.0f32; 2];
    let corner = [src.xmax - cent[0], src.ymax - cent[1]];
    let rot = [corner[0] * c + corner[1] * s, corner[1] * c - corner[0] * s];
    corner_max[0] = rot[0].abs();
    corner_max[1] = rot[1].abs();
    let corner = [corner[0], -corner[1]];
    let rot = [corner[0] * c + corner[1] * s, corner[1] * c - corner[0] * s];
    corner_max[0] = corner_max[0].max(rot[0].abs());
    corner_max[1] = corner_max[1].max(rot[1].abs());
    dst.xmin = cent[0] - corner_max[0];
    dst.xmax = cent[0] + corner_max[0];
    dst.ymin = cent[1] - corner_max[1];
    dst.ymax = cent[1] + corner_max[1];
}

/// `BLI_rctf_rotate_expand_around` — rotate about arbitrary pivot.
pub fn rctf_rotate_expand_around(dst: &mut Rctf, src: &Rctf, pivot: Vec2, angle: f32) {
    let cent = [rctf_cent_x(src) - pivot.x, rctf_cent_y(src) - pivot.y];
    let (sa, ca) = angle.sin_cos();
    let cent_rot = [cent[0] * ca - cent[1] * sa, cent[0] * sa + cent[1] * ca];
    rctf_rotate_expand(dst, src, angle);
    rctf_translate(dst, cent_rot[0] - cent[0], cent_rot[1] - cent[1]);
}

/// `BLI_rctf_clamp_segment` — clip segment endpoints into rect.
pub fn rctf_clamp_segment(rect: &Rctf, s1: &mut Vec2, s2: &mut Vec2) -> bool {
    let p1_inside = rctf_isect_pt_v(rect, *s1);
    let p2_inside = rctf_isect_pt_v(rect, *s2);
    if p1_inside && p2_inside {
        return true;
    }
    let lines: [(Vec2, Vec2); 4] = [
        (Vec2::new(rect.xmin, rect.ymax), Vec2::new(rect.xmax, rect.ymax)),
        (Vec2::new(rect.xmin, rect.ymin), Vec2::new(rect.xmax, rect.ymin)),
        (Vec2::new(rect.xmin, rect.ymin), Vec2::new(rect.xmin, rect.ymax)),
        (Vec2::new(rect.xmax, rect.ymin), Vec2::new(rect.xmax, rect.ymax)),
    ];
    if p1_inside && !p2_inside {
        for &(a, b) in &lines {
            if let Some(ix) = seg_seg_point(*s1, *s2, a, b) {
                *s2 = ix;
            }
        }
        return true;
    }
    if !p1_inside && p2_inside {
        for &(a, b) in &lines {
            if let Some(ix) = seg_seg_point(*s1, *s2, a, b) {
                *s1 = ix;
            }
        }
        return true;
    }
    for &(a, b) in &lines {
        match seg_seg_point(*s1, *s2, a, b) {
            Some(ix) => *s1 = ix,
            None => return false,
        }
    }
    for &(a, b) in &lines {
        match seg_seg_point(*s2, *s1, a, b) {
            Some(ix) => *s2 = ix,
            None => return false,
        }
    }
    true
}

fn seg_seg_point(s1: Vec2, s2: Vec2, l1: Vec2, l2: Vec2) -> Option<Vec2> {
    match isect_seg_seg_v2(s1, s2, l1, l2) {
        blenders_math::geom::Isect::Exact | blenders_math::geom::Isect::Cross => {
            // recompute point at intersection of the lines
            let d = (s2 - s1).cross(l2 - l1);
            if d == 0.0 {
                return Some(s1);
            }
            let t = (l1 - s1).cross(l2 - l1) / d;
            Some(s1 + (s2 - s1) * t)
        }
        _ => None,
    }
}

// ---- sizes / centers (from BLI_rect.h inline API) ----

/// `BLI_rctf_size_x`/`y`, `BLI_rcti_size_x`/`y`, `BLI_rctf_cent_x`/`y`, `BLI_rcti_cent_x`/`y`, `BLI_rcti_cent`, `BLI_rcti_size`, `BLI_rcti_cent_x_fl`.
pub fn rctf_size_x(r: &Rctf) -> f32 {
    r.xmax - r.xmin
}
pub fn rctf_size_y(r: &Rctf) -> f32 {
    r.ymax - r.ymin
}
pub fn rcti_size_x(r: &Rcti) -> i32 {
    r.xmax - r.xmin
}
pub fn rcti_size_y(r: &Rcti) -> i32 {
    r.ymax - r.ymin
}
pub fn rctf_cent_x(r: &Rctf) -> f32 {
    (r.xmin + r.xmax) * 0.5
}
pub fn rctf_cent_y(r: &Rctf) -> f32 {
    (r.ymin + r.ymax) * 0.5
}
pub fn rcti_cent_x(r: &Rcti) -> i32 {
    (r.xmin + r.xmax) / 2
}
pub fn rcti_cent_y(r: &Rcti) -> i32 {
    (r.ymin + r.ymax) / 2
}
pub fn rcti_cent_x_fl(r: &Rcti) -> f32 {
    (r.xmin + r.xmax) as f32 * 0.5
}
pub fn rcti_cent_y_fl(r: &Rcti) -> f32 {
    (r.ymin + r.ymax) as f32 * 0.5
}
/// `BLI_rcti_area` / `BLI_rctf_area`.
pub fn rcti_area(r: &Rcti) -> i64 {
    rcti_size_x(r) as i64 * rcti_size_y(r) as i64
}
pub fn rctf_area(r: &Rctf) -> f32 {
    rctf_size_x(r) * rctf_size_y(r)
}
/// `BLI_rctf_length` — diagonal length.
pub fn rctf_length(r: &Rctf, bound: f32) -> f32 {
    let d = (r.xmin - bound).abs().max((r.xmax - bound).abs());
    let d2 = (r.ymin - bound).abs().max((r.ymax - bound).abs());
    (d * d + d2 * d2).sqrt()
}
