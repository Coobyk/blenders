//! Port of `intern/icons_rasterize.cc` — `BKE_icon_geom_rasterize` +
//! `BKE_icon_geom_invert_lightness`. `ImBuf` → `IconImage` (packed u32 RGBA);
//! `IMB_scale` → box downscale.

use blenders_bli::bitmap_draw_2d::bitmap_draw_2d_tri_v2i;
use blenders_math::vector::Vec2;
use blenders_math::geom::barycentric_weights_v2_clamped;

/// `Icon_Geom` (BKE_icons.hh) — packed vertex data for icon triangles.
pub struct IconGeom<'a> {
    /// `coords_len` — number of triangles.
    pub coords_len: usize,
    /// `coords` — `uchar[3*coords_len][2]` positions (0..256 space).
    pub coords: &'a [[u8; 2]],
    /// `colors` — u32 per vertex (little-endian RGBA bytes).
    pub colors: &'a [u32],
}

/// Raster output (`ImBuf` byte-data equivalent).
pub struct IconImage {
    pub w: usize,
    pub h: usize,
    /// Pixels `rect[y*w+x]` as packed u32 (byte order = little-endian RGBA).
    pub rect: Vec<u32>,
}

struct RasterData<'a> {
    rect: &'a mut [u32],
    w: usize,
    color: [u32; 3],
    pt_fl: [Vec2; 3],
    color_u: [[u32; 4]; 3],
    smooth: bool,
}

fn tri_fill(data: &mut RasterData, x: i32, x_end: i32, y: i32) {
    if !data.smooth {
        let col = data.color[0];
        for xi in x..x_end {
            data.rect[(y as usize) * data.w + xi as usize] = col;
        }
    } else {
        let mut pt = Vec2::new(x as f32, y as f32);
        for xi in x..x_end {
            let w3 = barycentric_weights_v2_clamped(data.pt_fl[0], data.pt_fl[1], data.pt_fl[2], pt);
            let mut col_u = [0u32; 4];
            for c in 0..3 {
                for ch in 0..4 {
                    col_u[ch] += data.color_u[c][ch] * (w3[c] * 255.0) as u32;
                }
            }
            data.rect[(y as usize) * data.w + xi as usize] =
                (col_u[0] / 255 & 0xff) | ((col_u[1] / 255 & 0xff) << 8)
                    | ((col_u[2] / 255 & 0xff) << 16) | ((col_u[3] / 255 & 0xff) << 24);
            pt.x += 1.0;
        }
    }
}

/// Box downscale (`IMB_scale` Box filter).
fn box_downscale(src: &[u32], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<u32> {
    let mut out = vec![0u32; dw * dh];
    for y in 0..dh {
        for x in 0..dw {
            let x0 = x * sw / dw;
            let x1 = ((x + 1) * sw / dw).max(x0 + 1).min(sw);
            let y0 = y * sh / dh;
            let y1 = ((y + 1) * sh / dh).max(y0 + 1).min(sh);
            let mut acc = [0u64; 4];
            let mut n = 0u64;
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let p = src[sy * sw + sx];
                    for ch in 0..4 {
                        acc[ch] += ((p >> (ch * 8)) & 0xff) as u64;
                    }
                    n += 1;
                }
            }
            let mut px = 0u32;
            for ch in 0..4 {
                px |= (((acc[ch] / n) & 0xff) as u32) << (ch * 8);
            }
            out[y * dw + x] = px;
        }
    }
    out
}

/// `BKE_icon_geom_rasterize`.
pub fn bke_icon_geom_rasterize(geom: &IconGeom, size_x: usize, size_y: usize) -> IconImage {
    let rect_w = usize::max(256, size_x * 2);
    let rect_h = usize::max(256, size_y * 2);
    let mut rect = vec![0u32; rect_w * rect_h];

    let use_scale = rect_w != 256 || rect_h != 256;
    let scale = [rect_w as f32 / 256.0, rect_h as f32 / 256.0];

    for t in 0..geom.coords_len {
        let pos = &geom.coords[t * 3..t * 3 + 3];
        let col = &geom.colors[t * 3..t * 3 + 3];
        let mut pt = [[0i32; 2]; 3];
        for i in 0..3 {
            if use_scale {
                pt[i] = [
                    (pos[i][0] as f32 * scale[0]) as i32,
                    (pos[i][1] as f32 * scale[1]) as i32,
                ];
            } else {
                pt[i] = [pos[i][0] as i32, pos[i][1] as i32];
            }
        }
        let smooth = !(col[0] == col[1] && col[0] == col[2]);
        let mut color_u = [[0u32; 4]; 3];
        let mut pt_fl = [Vec2::ZERO; 3];
        if smooth {
            for i in 0..3 {
                pt_fl[i] = Vec2::new(pt[i][0] as f32, pt[i][1] as f32);
                for ch in 0..4 {
                    color_u[i][ch] = (col[i] >> (ch * 8)) & 0xff;
                }
            }
        }
        {
            let mut data = RasterData {
                rect: &mut rect,
                w: rect_w,
                color: [col[0], col[1], col[2]],
                pt_fl,
                color_u,
                smooth,
            };
            bitmap_draw_2d_tri_v2i(pt[0], pt[1], pt[2], |x, x_end, y| {
                if y >= 0 && (y as usize) < rect_h {
                    tri_fill(&mut data, x.max(0), x_end.min(rect_w as i32), y);
                }
            });
        }
    }

    IconImage {
        w: size_x,
        h: size_y,
        rect: box_downscale(&rect, rect_w, rect_h, size_x, size_y),
    }
}

/// `BKE_icon_geom_invert_lightness` — invert perceived lightness
/// (`srgb_to_linearrgb` via `srgb_to_linearrgb_v3_v3v3` gamma path);
/// operates on `geom->colors` in place.
pub fn bke_icon_geom_invert_lightness(colors: &mut [u32]) {
    for c in colors.iter_mut() {
        let r = (*c & 0xff) as f32 / 255.0;
        let g = ((*c >> 8) & 0xff) as f32 / 255.0;
        let b = ((*c >> 16) & 0xff) as f32 / 255.0;
        let a = (*c >> 24) & 0xff;
        let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let l2 = 1.0 - l;
        let scale = if l > 1e-4 { l2 / l } else { 1.0 };
        let nr = (r * scale * 255.0).clamp(0.0, 255.0) as u32;
        let ng = (g * scale * 255.0).clamp(0.0, 255.0) as u32;
        let nb = (b * scale * 255.0).clamp(0.0, 255.0) as u32;
        *c = nr | (ng << 8) | (nb << 16) | (a << 24);
    }
}
