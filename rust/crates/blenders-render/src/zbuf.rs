//! Port of `intern/zbuf.cc` — edge-function span
//! rasterizer for texture/bake coverage.

/// `ZSPAN_*` verbatim.
pub const ZSPAN_X_EPSILON: f32 = 0.001;
pub const ZSPAN_Y_EPSILON: f32 = 0.002;
pub const ZSPAN_X_OFFSET: f32 = 0.5 + ZSPAN_X_EPSILON;
pub const ZSPAN_Y_OFFSET: f32 = 0.5 + ZSPAN_Y_EPSILON;
pub const FLT_EPSILON: f32 = 1.1920929e-7;

/// `ZSpan` (verbatim — two row spans + min/max y per
/// span + vertex-identity for span completion).
pub struct ZSpan {
    /// `rectx`/`recty`.
    pub rectx: i32,
    pub recty: i32,
    /// `span1`/`span2` x-edges per row.
    pub span1: Vec<f32>,
    pub span2: Vec<f32>,
    /// `miny1`/`maxy1`/`miny2`/`maxy2`.
    pub miny1: i32,
    pub maxy1: i32,
    pub miny2: i32,
    pub maxy2: i32,
    /// `minp1`/`maxp1`/`minp2`/`maxp2` — vertex ids.
    pub minp1: Option<usize>,
    pub maxp1: Option<usize>,
    pub minp2: Option<usize>,
    pub maxp2: Option<usize>,
    /// vertex ys by id (for span-completion compare).
    vert_y: [f32; 3],
    /// vertex xs by id.
    vert_x: [f32; 3],
}
impl ZSpan {
    /// `zbuf_alloc_span`.
    pub fn new(rectx: i32, recty: i32) -> Self {
        Self {
            rectx,
            recty,
            span1: vec![0.0; recty as usize],
            span2: vec![0.0; recty as usize],
            miny1: 0, maxy1: 0, miny2: 0, maxy2: 0,
            minp1: None, maxp1: None, minp2: None, maxp2: None,
            vert_y: [0.0; 3],
            vert_x: [0.0; 3],
        }
    }
    /// `zbuf_init_span`.
    pub fn init_span(&mut self) {
        self.miny1 = self.recty + 1;
        self.miny2 = self.recty + 1;
        self.maxy1 = -1;
        self.maxy2 = -1;
        self.minp1 = None;
        self.maxp1 = None;
        self.minp2 = None;
        self.maxp2 = None;
    }
    /// `zbuf_add_to_span` (verbatim — clip, slope, span
    /// completion by shared vertex).
    pub fn add_to_span(&mut self, v1: usize, v2: usize) {
        let (minv, maxv) = if self.vert_y[v1] < self.vert_y[v2] { (v1, v2) } else { (v2, v1) };
        let minv_y = self.vert_y[minv];
        let maxv_y = self.vert_y[maxv];
        let mut my0 = minv_y.ceil() as i32;
        let mut my2 = maxv_y.floor() as i32;
        if my2 < 0 || my0 >= self.recty {
            return;
        }
        if my2 >= self.recty {
            my2 = self.recty - 1;
        }
        my0 = my0.max(0);
        if my0 > my2 {
            return;
        }
        let xx1 = maxv_y - minv_y;
        let vert_x = self.vert_x;
        let (dx0, mut xs0) = if xx1 > FLT_EPSILON {
            let dx0 = (vert_x[minv] - vert_x[maxv]) / xx1;
            (dx0, dx0 * (minv_y - my2 as f32) + vert_x[minv])
        } else {
            (0.0, vert_x[minv].min(vert_x[maxv]))
        };
        let on_span1 = if self.maxp1.is_none() {
            true
        } else if self.minp1 == Some(maxv) || self.maxp1 == Some(minv) {
            true
        } else {
            false
        };
        if on_span1 {
            if self.minp1.is_none() || self.vert_y[self.minp1.unwrap()] > minv_y {
                self.minp1 = Some(minv);
            }
            if self.maxp1.is_none() || self.vert_y[self.maxp1.unwrap()] < maxv_y {
                self.maxp1 = Some(maxv);
            }
            self.miny1 = my0.min(self.miny1);
            self.maxy1 = my2.max(self.maxy1);
        } else {
            if self.minp2.is_none() || self.vert_y[self.minp2.unwrap()] > minv_y {
                self.minp2 = Some(minv);
            }
            if self.maxp2.is_none() || self.vert_y[self.maxp2.unwrap()] < maxv_y {
                self.maxp2 = Some(maxv);
            }
            self.miny2 = my0.min(self.miny2);
            self.maxy2 = my2.max(self.maxy2);
        }
        for y in (my0..=my2).rev() {
            if on_span1 {
                self.span1[y as usize] = xs0;
            } else {
                self.span2[y as usize] = xs0;
            }
            xs0 += dx0;
        }
    }
}

/// `zspan_rasterize_triangle` (verbatim — 3 edges →
/// spans, edge-function uv walk per row).
pub fn zspan_rasterize_triangle(
    zspan: &mut ZSpan,
    v1_in: [f32; 2],
    v2_in: [f32; 2],
    v3_in: [f32; 2],
    func: &mut dyn FnMut(i32, i32, f32, f32),
) {
    zspan.init_span();
    zspan.vert_x = [v1_in[0] - ZSPAN_X_OFFSET, v2_in[0] - ZSPAN_X_OFFSET, v3_in[0] - ZSPAN_X_OFFSET];
    zspan.vert_y = [v1_in[1] - ZSPAN_Y_OFFSET, v2_in[1] - ZSPAN_Y_OFFSET, v3_in[1] - ZSPAN_Y_OFFSET];
    zspan.add_to_span(0, 1);
    zspan.add_to_span(1, 2);
    zspan.add_to_span(2, 0);
    if zspan.minp2.is_none() || zspan.maxp2.is_none() {
        return;
    }
    let my0 = zspan.miny1.max(zspan.miny2);
    let my2 = zspan.maxy1.min(zspan.maxy2);
    if my2 < my0 {
        return;
    }
    let v1 = [zspan.vert_x[0], zspan.vert_y[0]];
    let v2 = [zspan.vert_x[1], zspan.vert_y[1]];
    let v3 = [zspan.vert_x[2], zspan.vert_y[2]];
    let x1 = v1[0] - v2[0];
    let x2 = v2[0] - v3[0];
    let y1 = v1[1] - v2[1];
    let y2 = v2[1] - v3[1];
    let mut z1 = 1.0f32;
    let mut z2 = 0.0f32;
    let mut x0 = y1 * z2 - z1 * y2;
    let mut y0 = z1 * x2 - x1 * z2;
    let z0 = x1 * y2 - y1 * x2;
    if z0 == 0.0 {
        return;
    }
    let mut xx1 = (x0 * v1[0] + y0 * v1[1]) / z0 + 1.0;
    let uxd = -(x0 as f64) / (z0 as f64);
    let uyd = -(y0 as f64) / (z0 as f64);
    let uy0 = (my2 as f64) * uyd + (xx1 as f64);
    z1 = -1.0;
    z2 = 1.0;
    x0 = y1 * z2 - z1 * y2;
    y0 = z1 * x2 - x1 * z2;
    xx1 = (x0 * v1[0] + y0 * v1[1]) / z0;
    let vxd = -(x0 as f64) / (z0 as f64);
    let vyd = -(y0 as f64) / (z0 as f64);
    let vy0 = (my2 as f64) * vyd + (xx1 as f64);
    let rectx = zspan.rectx;
    let mut i = 0i32;
    for y in (my0..=my2).rev() {
        let s1 = zspan.span1[y as usize];
        let s2 = zspan.span2[y as usize];
        let mut sn1 = s1.min(s2).floor() as i32;
        let mut sn2 = s1.max(s2).floor() as i32;
        sn1 += 1;
        if sn2 >= rectx {
            sn2 = rectx - 1;
        }
        sn1 = sn1.max(0);
        let u = (sn1 as f64) * uxd + uy0 - (i as f64) * uyd;
        let v = (sn1 as f64) * vxd + vy0 - (i as f64) * vyd;
        let mut j = 0i32;
        let mut x = sn1;
        while x <= sn2 {
            func(x, y, (u + (j as f64) * uxd) as f32, (v + (j as f64) * vxd) as f32);
            j += 1;
            x += 1;
        }
        i += 1;
    }
}

/// `zspan_rasterize_conservative_line` (verbatim —
/// per-scanline x-range + w-interpolated uv).
pub fn zspan_rasterize_conservative_line(
    zspan: &ZSpan,
    v1_in: [f32; 2],
    v2_in: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    func: &mut dyn FnMut(i32, i32, f32, f32),
) {
    let v1 = [v1_in[0] - ZSPAN_X_OFFSET, v1_in[1] - ZSPAN_Y_OFFSET];
    let v2 = [v2_in[0] - ZSPAN_X_OFFSET, v2_in[1] - ZSPAN_Y_OFFSET];
    let miny = v1[1].min(v2[1]);
    let maxy = v1[1].max(v2[1]);
    let mut y0 = (miny + 0.5).floor() as i32;
    let mut y1 = (maxy + 0.5).floor() as i32;
    y0 = y0.max(0);
    if y1 >= zspan.recty {
        y1 = zspan.recty - 1;
    }
    if y0 > y1 {
        return;
    }
    let mut dir = [v2[0] - v1[0], v2[1] - v1[1]];
    let len_sqr = dir[0] * dir[0] + dir[1] * dir[1];
    if len_sqr > FLT_EPSILON {
        dir[0] /= len_sqr;
        dir[1] /= len_sqr;
    }
    let x_miny = if v1[1] < v2[1] { v1[0] } else { v2[0] };
    let x_maxy = if v1[1] < v2[1] { v2[0] } else { v1[0] };
    let inv_dy = if (maxy - miny).abs() > FLT_EPSILON { 1.0 / (maxy - miny) } else { 0.0 };
    while y0 <= y1 {
        let ta = ((y0 as f32 - 0.5).clamp(miny, maxy) - miny) * inv_dy;
        let tb = (maxy - (y0 as f32 + 0.5).clamp(miny, maxy)) * inv_dy;
        let xa = x_miny + (x_maxy - x_miny) * ta;
        let xb = x_miny + (x_maxy - x_miny) * (1.0 - tb);
        let minx = xa.min(xb);
        let maxx = xa.max(xb);
        let mut x0 = (minx + 0.5).floor() as i32;
        let mut x1 = (maxx + 0.5).floor() as i32;
        x0 = x0.max(0);
        if x1 >= zspan.rectx {
            x1 = zspan.rectx - 1;
        }
        if x0 <= x1 {
            while x0 <= x1 {
                let mut w = (x0 as f32 - v1[0]) * dir[0] + (y0 as f32 - v1[1]) * dir[1];
                w = w.clamp(0.0, 1.0);
                let u = (1.0 - w) * uv1[0] + w * uv2[0];
                let v = (1.0 - w) * uv1[1] + w * uv2[1];
                func(x0, y0, u, v);
                x0 += 1;
            }
        }
        y0 += 1;
    }
}

/// `zspan_rasterize_conservative_wireframe` (verbatim —
/// 3 lines, barycentric corner uvs).
pub fn zspan_rasterize_conservative_wireframe(
    zspan: &ZSpan,
    v1: [f32; 2],
    v2: [f32; 2],
    v3: [f32; 2],
    func: &mut dyn FnMut(i32, i32, f32, f32),
) {
    let uv1 = [1.0, 0.0];
    let uv2 = [0.0, 1.0];
    let uv3 = [0.0, 0.0];
    zspan_rasterize_conservative_line(zspan, v1, v2, uv1, uv2, func);
    zspan_rasterize_conservative_line(zspan, v2, v3, uv2, uv3, func);
    zspan_rasterize_conservative_line(zspan, v3, v1, uv3, uv1, func);
}
