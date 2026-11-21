//! Port of `intern/image_gpu.cc` — GPU-side image textures: UDIM
//! tile packing/mapping, resolution clamping, cache-key lookups.
//! `gpu::Texture`/ImBuf access injected via `GpuTextureOps`.

/// `ImageUDIMTexture`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageUDIMTexture {
    /// Default UDIM texture.
    Default,
    /// Tiled image texture.
    Tiled,
}

/// `ImageCacheKey` (frame + pass + layer).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ImageCacheKey {
    /// `frame`.
    pub frame: i32,
    /// `pass`.
    pub pass: i32,
    /// `layer`.
    pub layer: i32,
}

/// `ImageGPUTextures` result.
#[derive(Clone, Copy, Default)]
pub struct ImageGPUTextures {
    /// Single-image texture handle.
    pub texture: usize,
    /// Tile array texture.
    pub tile_array: usize,
    /// Tile mapping texture.
    pub tile_mapping: usize,
}

/// `is_over_resolution_limit` — `GPU_texture_size_with_limit` injected.
pub fn is_over_resolution_limit(w: i32, h: i32, size_with_limit: &dyn Fn(i32) -> i32) -> bool {
    w > size_with_limit(w) || h > size_with_limit(h)
}

fn power_of_2_min_i(n: i32) -> i32 {
    if n <= 0 {
        return 1;
    }
    let mut p = 1;
    while p * 2 <= n {
        p *= 2;
    }
    p
}

/// `smaller_power_of_2_limit`.
pub fn smaller_power_of_2_limit(num: i32, size_with_limit: &dyn Fn(i32) -> i32) -> i32 {
    power_of_2_min_i(size_with_limit(num))
}

/// `FixedSizeBoxPack`.
#[derive(Clone, Default)]
pub struct FixedSizeBoxPack {
    /// `x`, `y` — placed position.
    pub x: i32,
    /// See `x`.
    pub y: i32,
    /// `w`, `h` — requested size.
    pub w: i32,
    /// See `w`.
    pub h: i32,
}

/// `PackTile` — a tile plus its packing score.
#[derive(Clone, Default)]
pub struct PackTile {
    /// `boxpack`.
    pub boxpack: FixedSizeBoxPack,
    /// `tile` — tile index.
    pub tile: usize,
    /// `pack_score`.
    pub pack_score: f32,
}

/// `compare_packtile` — sort by descending `pack_score`.
pub fn compare_packtile(a: &PackTile, b: &PackTile) -> std::cmp::Ordering {
    b.pack_score
        .partial_cmp(&a.pack_score)
        .unwrap_or(std::cmp::Ordering::Equal)
}

/// Per-tile target size computation from `gpu_texture_create_tile_array`:
/// clamps over-limit dims to the smaller power of 2 and scores.
/// `ibuf_dims(tile)` yields `(w, h, is_bw)`.
pub fn packtile_from_ibuf(w: i32, h: i32, size_with_limit: &dyn Fn(i32) -> i32) -> PackTile {
    let mut b = FixedSizeBoxPack {
        w,
        h,
        ..Default::default()
    };
    if is_over_resolution_limit(b.w, b.h, size_with_limit) {
        b.w = smaller_power_of_2_limit(b.w, size_with_limit);
        b.h = smaller_power_of_2_limit(b.h, size_with_limit);
    }
    let (wf, hf) = (b.w as f32, b.h as f32);
    PackTile {
        boxpack: b,
        tile: 0,
        pack_score: wf.max(hf) / wf.min(hf) * wf * hf,
    }
}

/// `gpu_texture_create_tile_mapping` — builds the 1d-array data
/// (2 vec4 per tile: layer + normalized offset/size).
pub fn tile_mapping_data(
    tile_numbers: &[i32],
    tilearray_layers: &[f32],
    tilearray_offsets: &[[i32; 2]],
    tilearray_sizes: &[[i32; 2]],
    array_w: f32,
    array_h: f32,
) -> Vec<f32> {
    let max_tile = tile_numbers.iter().max().copied().unwrap_or(1001) - 1001;
    let width = (max_tile + 1) as usize;
    let mut data = vec![0.0f32; width * 8];
    for i in 0..width {
        data[4 * i] = -1.0;
    }
    for (k, &tn) in tile_numbers.iter().enumerate() {
        let i = (tn - 1001) as usize;
        data[4 * i] = tilearray_layers[k];
        let ti = 4 * width + 4 * i;
        data[ti] = tilearray_offsets[k][0] as f32 / array_w;
        data[ti + 1] = tilearray_offsets[k][1] as f32 / array_h;
        data[ti + 2] = tilearray_sizes[k][0] as f32 / array_w;
        data[ti + 3] = tilearray_sizes[k][1] as f32 / array_h;
    }
    data
}

/// `BKE_image_sort_tiles` ordering — by `tile_number`.
pub fn sort_tile_numbers(tiles: &mut [i32]) {
    tiles.sort_unstable();
}
