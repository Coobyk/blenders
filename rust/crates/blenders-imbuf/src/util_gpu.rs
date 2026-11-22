//! Port of `intern/util_gpu.cc` — ImBuf →
//! GPU texture format selection + update.

/// `gpu::TextureFormat` ids (verbatim subset —
/// uncompressed + BCn).
pub const GPU_RGBA8: i32 = 0;
pub const GPU_RGB8: i32 = 1;
pub const GPU_R8: i32 = 2;
pub const GPU_RGBA16F: i32 = 3;
pub const GPU_RGBA32F: i32 = 4;
/// `imb_is_grayscale_texture_format_compatible`
/// (verbatim — 1ch or equal rgb).
pub fn imb_is_grayscale_format_compatible(channels: i32, rect_float: Option<&[f32]>) -> bool {
    if channels == 1 {
        return true;
    }
    if let Some(buf) = rect_float {
        for px in buf.chunks_exact(4) {
            if px[0] != px[1] || px[1] != px[2] {
                return false;
            }
        }
        return true;
    }
    false
}
/// `imb_gpu_get_format` (verbatim — pick format
// by channels + float).
pub fn imb_gpu_get_format(channels: i32, is_float: bool, is_grayscale: bool) -> i32 {
    if is_grayscale {
        return GPU_R8;
    }
    if is_float {
        return GPU_RGBA32F;
    }
    match channels {
        1 => GPU_R8,
        3 => GPU_RGB8,
        _ => GPU_RGBA8,
    }
}
/// `imb_gpu_get_swizzle` (verbatim — 1ch→"rrr1",
// 3ch→"rgb1", 4ch→"rgba").
pub fn imb_gpu_get_swizzle(channels: i32) -> &'static str {
    match channels {
        1 => "rrr1",
        3 => "rgb1",
        _ => "rgba",
    }
}
/// `imb_gpu_mipmap_modified_chunks_size`
/// (verbatim — chunk dims).
pub fn imb_gpu_mipmap_modified_chunks_size(w: i32, h: i32, chunk: i32) -> (i32, i32) {
    ((w + chunk - 1) / chunk, (h + chunk - 1) / chunk)
}
