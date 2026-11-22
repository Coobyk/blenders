//! Port of `intern/texture_margin.cc` — UV-coverage
//! map + Dijkstra flood margin fill.

use crate::zbuf::{zspan_rasterize_conservative_wireframe, zspan_rasterize_triangle, ZSpan};

/// `TextureMarginMap::directions`/`distances` verbatim.
pub const DIRECTIONS: [[i32; 2]; 8] = [
    [1, 0],
    [1, 1],
    [0, 1],
    [-1, 1],
    [-1, 0],
    [-1, -1],
    [0, -1],
    [1, -1],
];
pub const DISTANCES: [i32; 8] = [2, 3, 2, 3, 2, 3, 2, 3];

/// Dijkstra pixel pack helpers (verbatim).
pub fn pack_dijkstra_pixel(dist: u32, dir: u32) -> u32 {
    0x8000_0000 + (dist << 4) + dir
}
pub fn dijkstra_pixel_get_distance(dp: u32) -> u32 {
    (dp ^ 0x8000_0000) >> 4
}
pub fn dijkstra_pixel_get_direction(dp: u32) -> u32 {
    dp & 0xF
}
pub fn is_dijkstra_pixel(dp: u32) -> bool {
    (dp & 0x8000_0000) != 0
}
pub fn dijkstra_pixel_is_unset(dp: u32) -> bool {
    dp == 0xFFFF_FFFF
}

/// `TextureMarginMap` (verbatim — pixel_data u32 grid
/// + rasterized face coverage + flood margin).
pub struct TextureMarginMap {
    /// `w_`/`h_`.
    pub w: i32,
    pub h: i32,
    /// `pixel_data_`.
    pub pixel_data: Vec<u32>,
    /// `zspan_`.
    pub zspan: ZSpan,
    /// `uv_offset_`.
    pub uv_offset: [f32; 2],
    /// `value_to_store_`/mask state for raster callbacks.
    pub value_to_store: u32,
    /// `write_mask_`.
    pub write_mask: bool,
}
impl TextureMarginMap {
    /// Ctor (verbatim — 0xFFFFFFFF fill + span alloc).
    pub fn new(w: i32, h: i32, uv_offset: [f32; 2]) -> Self {
        Self {
            w,
            h,
            pixel_data: vec![0xFFFF_FFFF; (w * h) as usize],
            zspan: ZSpan::new(w, h),
            uv_offset,
            value_to_store: 0,
            write_mask: false,
        }
    }
    /// `set_pixel`.
    pub fn set_pixel(&mut self, x: i32, y: i32, value: u32) {
        self.pixel_data[(y * self.w + x) as usize] = value;
    }
    /// `get_pixel` (verbatim — OOB → unset).
    pub fn get_pixel(&self, x: i32, y: i32) -> u32 {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return 0xFFFF_FFFF;
        }
        self.pixel_data[(y * self.w + x) as usize]
    }
    /// `rasterize_tri` + `zscan_store_pixel` (verbatim —
    /// mask-aware value write via zspan).
    pub fn rasterize_tri(
        &mut self,
        v1: [f32; 2],
        v2: [f32; 2],
        v3: [f32; 2],
        value: u32,
        mut mask: Option<&mut Vec<u8>>,
        writemask: bool,
        fill: bool,
    ) {
        self.value_to_store = value;
        self.write_mask = writemask;
        let w = self.w;
        let value_to_store = self.value_to_store;
        let write_mask = self.write_mask;
        let pixel_data = &mut self.pixel_data;
        let mut store = |x: i32, y: i32, _u: f32, _v: f32| {
            if let Some(m) = mask.as_deref() {
                if write_mask {
                    pixel_data[(y * w + x) as usize] = value_to_store;
                    let _ = m;
                } else if m[(y * w + x) as usize] != 0 {
                    pixel_data[(y * w + x) as usize] = value_to_store;
                }
            } else {
                pixel_data[(y * w + x) as usize] = value_to_store;
            }
            if write_mask {
                if let Some(m) = mask.as_deref_mut() {
                    m[(y * w + x) as usize] = 1;
                }
            }
        };
        if fill {
            let (s1, s2, rx, ry) =
                (self.zspan.span1.clone(), self.zspan.span2.clone(), self.zspan.rectx, self.zspan.recty);
            let mut zs = ZSpan { span1: s1, span2: s2, rectx: rx, recty: ry, ..ZSpan::new(rx, ry) };
            zspan_rasterize_triangle(&mut zs, v1, v2, v3, &mut store);
            self.zspan = zs;
        } else {
            zspan_rasterize_conservative_wireframe(&self.zspan, v1, v2, v3, &mut store);
        }
    }
    /// `grow_dijkstra` seed pass (verbatim — unset
    /// pixel adjacent to set → pack dist+dir, seed heap).
    pub fn grow_dijkstra_seed(&self) -> Vec<(i32, i32, i32)> {
        let mut seeds = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                if dijkstra_pixel_is_unset(self.get_pixel(x, y)) {
                    for i in 0..8 {
                        let xx = x - DIRECTIONS[i][0];
                        let yy = y - DIRECTIONS[i][1];
                        if xx >= 0 && xx < self.w && yy >= 0 && yy < self.h {
                            let p = self.get_pixel(xx, yy);
                            if !is_dijkstra_pixel(p) && !dijkstra_pixel_is_unset(p) {
                                seeds.push((DISTANCES[i], x, y));
                                break;
                            }
                        }
                    }
                }
            }
        }
        seeds
    }
}
