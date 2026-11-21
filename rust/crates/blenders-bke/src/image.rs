//! Port of `intern/image.cc` — `Image` tile/UDIM math,
//! image-cache key hashing, alpha-mode from extension,
//! buffer bookkeeping.
//!
//! `PORT_PENDING`: `image_acquire_ibuf`/`image_load_image_
//! file` (ImBuf + OIIO decode — injected), `image_blend_*
//! read/write` (BlendWriter), `image_foreach_*` callbacks,
//! `BKE_image_get_render_result`/`result_rect` (RenderResult
//! layers), `BKE_image_signal`/`stamp`/`scale` (ImBuf
//! channels), `image_colorspace_from_imbuf` (colorspace),
//! `image_multilayer` (GPUTexture), `BKE_image_get_tile`
//! tile-list management.

/// `ImageCacheKey` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ImageCacheKey {
    /// `udim_type`.
    pub udim_type: i32,
    /// `frame`.
    pub frame: i32,
    /// `tile_number`.
    pub tile_number: i32,
    /// `view`.
    pub view: i32,
    /// `multi_index`.
    pub multi_index: i32,
}

/// `imagecache_hashhash` — same 5-field hash (verbatim).
pub fn imagecache_hashhash(key: &ImageCacheKey) -> u32 {
    /* `get_default_hash` over the 5 fields. */
    let mut h: u64 = 0xcbf29ce484222325;
    for &v in &[key.udim_type, key.frame, key.tile_number, key.view, key.multi_index] {
        h ^= v as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h as u32
}
/// `imagecache_keydata` — `(framenr, proxy=NONE, render_
/// flags=0)` (verbatim).
pub fn imagecache_keydata(key: &ImageCacheKey) -> (i32, i32, i32) {
    (key.frame, IMB_PROXY_NONE, 0)
}
/// `IMB_PROXY_NONE`.
pub const IMB_PROXY_NONE: i32 = 0;

/// `image_get_tile_number_from_iuser` (verbatim).
pub fn image_get_tile_number_from_iuser(first_tile_number: i32, iuser_tile: Option<i32>) -> i32 {
    iuser_tile.unwrap_or(first_tile_number)
}

/// `BKE_image_get_tile` — tile-number → list index
/// (verbatim; `0`/`1001` alias to first tile).
pub fn image_get_tile_index(tiles: &[i32], tile_number: i32, is_tiled: bool, udim_max: i32) -> Option<usize> {
    if tile_number == 0 || tile_number == 1001 {
        return (!tiles.is_empty()).then_some(0);
    }
    if !is_tiled || tile_number < 1001 || tile_number > udim_max {
        return None;
    }
    tiles.iter().position(|&t| t == tile_number)
}
/// `IMA_UDIM_MAX`.
pub const IMA_UDIM_MAX: i32 = 2000;
/// `IMA_SRC_TILED`.
pub const IMA_SRC_TILED: i8 = 6;

/// `BKE_image_get_tile_from_pos` (verbatim — returns
/// `(tile_number, udim_offset)` or `None`).
pub fn image_get_tile_from_pos(
    is_tiled: bool,
    uv: [f32; 2],
    has_tile: &dyn Fn(i32) -> bool,
) -> Option<(i32, [f32; 2])> {
    if !is_tiled || uv[0] < 0.0 || uv[1] < 0.0 || uv[0] >= 10.0 {
        return None;
    }
    let ix = uv[0] as i32;
    let iy = uv[1] as i32;
    let tile_number = 1001 + 10 * iy + ix;
    if !has_tile(tile_number) {
        return None;
    }
    Some((tile_number, [ix as f32, iy as f32]))
}

/// `BKE_image_get_tile_uv` (verbatim).
pub fn image_get_tile_uv(is_tiled: bool, tile_number: i32) -> [f32; 2] {
    if !is_tiled {
        return [0.0; 2];
    }
    let tile_index = tile_number - 1001;
    [
        (tile_index % 10) as f32,
        (tile_index / 10) as f32,
    ]
}

/// `distance_to_unit_interval` (verbatim).
pub fn distance_to_unit_interval(x: f32) -> f32 {
    if x <= 0.0 {
        -x
    } else if x <= 1.0 {
        0.0
    } else {
        x - 1.0
    }
}
/// `distance_squared_to_udim` (verbatim).
pub fn distance_squared_to_udim(co: [f32; 2], udim: [f32; 2]) -> f32 {
    let dx = distance_to_unit_interval(co[0] - udim[0]);
    let dy = distance_to_unit_interval(co[1] - udim[1]);
    dx * dx + dy * dy
}
/// `nearest_udim_tile_tie_break` (verbatim).
pub fn nearest_udim_tile_tie_break(
    best_dist_sq: f32,
    best_uv: [f32; 2],
    dist_sq: f32,
    uv: [f32; 2],
) -> bool {
    if best_dist_sq == dist_sq {
        if best_uv[0] == uv[0] {
            return uv[1] > best_uv[1];
        }
        return uv[0] > best_uv[0];
    }
    dist_sq < best_dist_sq
}

/// `BKE_image_find_nearest_tile_with_offset` (verbatim;
/// `tile_numbers`/`tile_uvs` are the parallel tile table).
pub fn image_find_nearest_tile_with_offset(
    is_tiled: bool,
    co: [f32; 2],
    tiles: &[(i32, [f32; 2])],
) -> (i32, [f32; 2]) {
    let mut r_uv_offset = [0.0f32; 2];
    let mut tile_number_best = -1;
    if !is_tiled {
        return (tile_number_best, r_uv_offset);
    }
    let mut dist_best_sq = f32::MAX;
    for &(tile_number, uv_offset) in tiles {
        let dist_sq = distance_squared_to_udim(co, uv_offset);
        if dist_sq == 0.0 {
            if co[0].floor() == uv_offset[0] && co[1].floor() == uv_offset[1] {
                r_uv_offset = uv_offset;
                return (tile_number, r_uv_offset);
            }
        }
        if nearest_udim_tile_tie_break(dist_best_sq, r_uv_offset, dist_sq, uv_offset) {
            dist_best_sq = dist_sq;
            r_uv_offset = uv_offset;
            tile_number_best = tile_number;
        }
    }
    (tile_number_best, r_uv_offset)
}
/// `BKE_image_find_nearest_tile`.
pub fn image_find_nearest_tile(is_tiled: bool, co: [f32; 2], tiles: &[(i32, [f32; 2])]) -> i32 {
    image_find_nearest_tile_with_offset(is_tiled, co, tiles).0
}

/// `IMA_ALPHA_*` verbatim.
pub const IMA_ALPHA_IGNORE: i8 = 0;
/// `IMA_ALPHA_STRAIGHT`.
pub const IMA_ALPHA_STRAIGHT: i8 = 1;
/// `IMA_ALPHA_PREMUL`.
pub const IMA_ALPHA_PREMUL: i8 = 2;
/// `IMA_ALPHA_CHANNEL_PACKED`.
pub const IMA_ALPHA_CHANNEL_PACKED: i8 = 3;

/// `BKE_image_alpha_mode_from_extension_ex` (verbatim).
pub fn image_alpha_mode_from_extension_ex(filepath: &str) -> i8 {
    let lower = filepath.to_ascii_lowercase();
    if [".exr", ".cin", ".dpx", ".hdr"]
        .iter()
        .any(|e| lower.ends_with(e))
    {
        IMA_ALPHA_PREMUL
    } else {
        IMA_ALPHA_STRAIGHT
    }
}
