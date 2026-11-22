//! Port of `intern/wm_window_icon.cc` — NanoSVG
//! rasterization of `blender_app_icon_svg` into the GHOST
//! icon buffer + `wm_ghost_icon_generator`.

/// `wm_ghost_icon_generate` (verbatim — parse SVG at 96dpi
/// "px" units, scale-to-fit centered, rasterize RGBA).
pub fn wm_ghost_icon_generate(
    svg_source: &str,
    pixels: &mut [u8],
    icon_size: i32,
    nsvg_parse: &mut dyn FnMut(&str) -> Option<(i32, i32, usize)>,
    nsvg_create_rasterizer: &mut dyn FnMut() -> Option<usize>,
    nsvg_rasterize: &mut dyn FnMut(usize, usize, f32, f32, f32, &mut [u8], i32, i32, i32),
    nsvg_delete_rasterizer: &mut dyn FnMut(usize),
    nsvg_delete: &mut dyn FnMut(usize),
) {
    let buffer_size = (icon_size as usize) * (icon_size as usize) * 4;
    let stride = icon_size * 4;
    let image = (nsvg_parse)(svg_source);
    let (w, h, image_h) = match image {
        Some((w, h, img)) if w != 0 && h != 0 => (w, h, img),
        Some((_, _, img)) => {
            (nsvg_delete)(img);
            pixels[..buffer_size].fill(0);
            return;
        }
        None => {
            pixels[..buffer_size].fill(0);
            return;
        }
    };
    let rast = match (nsvg_create_rasterizer)() {
        Some(r) => r,
        None => {
            (nsvg_delete)(image_h);
            pixels[..buffer_size].fill(0);
            return;
        }
    };
    let scale = icon_size as f32 / (w.max(h)) as f32;
    let offset_x = (icon_size as f32 - w as f32 * scale) * 0.5;
    let offset_y = (icon_size as f32 - h as f32 * scale) * 0.5;
    pixels[..buffer_size].fill(0);
    (nsvg_rasterize)(rast, image_h, offset_x, offset_y, scale, pixels, icon_size, icon_size, stride);
    (nsvg_delete_rasterizer)(rast);
    (nsvg_delete)(image_h);
}
