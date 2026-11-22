//! Port of `intern/wm_subwindow.cc` — viewport/scissor
//! setup + ortho projection helpers (`wmViewport`,
//! `wmPartialViewport`, `wmWindowViewport(_ex|Title)`,
//! `wmOrtho2*`).

/// `GLA_PIXEL_OFS` verbatim (`GPU_viewport` offset).
pub const GLA_PIXEL_OFS: f32 = 0.375;

/// `rcti` verbatim (`BLI_rect.h`).
#[derive(Clone, Copy, Default)]
pub struct Rcti {
    pub xmin: i32,
    pub xmax: i32,
    pub ymin: i32,
    pub ymax: i32,
}

/// `BLI_rcti_size_x`/`size_y` (verbatim — exclusive).
pub fn rcti_size_x(r: &Rcti) -> i32 {
    r.xmax - r.xmin
}
/// `BLI_rcti_size_y`.
pub fn rcti_size_y(r: &Rcti) -> i32 {
    r.ymax - r.ymin
}
/// `BLI_rcti_isect` (verbatim — clip a to b).
pub fn rcti_isect(a: &Rcti, b: &Rcti) -> Option<Rcti> {
    let xmin = a.xmin.max(b.xmin);
    let ymin = a.ymin.max(b.ymin);
    let xmax = a.xmax.min(b.xmax);
    let ymax = a.ymax.min(b.ymax);
    if xmax < xmin || ymax < ymin {
        None
    } else {
        Some(Rcti { xmin, xmax, ymin, ymax })
    }
}

/// `GpuView` — GPU/scissor/matrix calls injected.
pub struct GpuView<'a> {
    /// `GPU_viewport`.
    pub viewport: &'a mut dyn FnMut(i32, i32, i32, i32),
    /// `GPU_scissor`.
    pub scissor: &'a mut dyn FnMut(i32, i32, i32, i32),
    /// `GPU_matrix_ortho_set`.
    pub matrix_ortho_set: &'a mut dyn FnMut(f32, f32, f32, f32, f32, f32),
    /// `GPU_matrix_identity_set`.
    pub matrix_identity_set: &'a mut dyn FnMut(),
}

/// `wmViewport` (verbatim — viewport+scissor on winrct +
/// pixelspace ortho + identity).
pub fn wm_viewport(winrct: &Rcti, gpu: &mut GpuView) {
    let width = rcti_size_x(winrct) + 1;
    let height = rcti_size_y(winrct) + 1;
    (gpu.viewport)(winrct.xmin, winrct.ymin, width, height);
    (gpu.scissor)(winrct.xmin, winrct.ymin, width, height);
    wm_ortho2_pixelspace(width as f32, height as f32, gpu);
    (gpu.matrix_identity_set)();
}

/// `wmPartialViewport` (verbatim — full or isect'd rect,
/// 1px scissor pad on full).
pub fn wm_partial_viewport(
    drawrct: &mut Rcti,
    winrct: &Rcti,
    partialrct: &Rcti,
    gpu: &mut GpuView,
) {
    let scissor_pad;
    if partialrct.xmin == partialrct.xmax {
        *drawrct = *winrct;
        scissor_pad = true;
    } else {
        *drawrct = rcti_isect(winrct, partialrct).unwrap_or_default();
        scissor_pad = false;
    }
    let x = drawrct.xmin - winrct.xmin;
    let y = drawrct.ymin - winrct.ymin;
    let width = rcti_size_x(winrct) + 1;
    let height = rcti_size_y(winrct) + 1;
    let mut scissor_width = rcti_size_x(drawrct);
    let mut scissor_height = rcti_size_y(drawrct);
    if scissor_pad {
        scissor_width += 1;
        scissor_height += 1;
    }
    (gpu.viewport)(0, 0, width, height);
    (gpu.scissor)(x, y, scissor_width, scissor_height);
    wm_ortho2_pixelspace(width as f32, height as f32, gpu);
    (gpu.matrix_identity_set)();
}

/// `wmWindowViewport_ex` (verbatim — native pixel size +
/// offset ortho).
pub fn wm_window_viewport_ex(win_size: (i32, i32), offset: f32, gpu: &mut GpuView) {
    (gpu.viewport)(0, 0, win_size.0, win_size.1);
    (gpu.scissor)(0, 0, win_size.0, win_size.1);
    wm_ortho2_offset(win_size.0 as f32, win_size.1 as f32, offset, gpu);
    (gpu.matrix_identity_set)();
}

/// `wmWindowViewport` (verbatim — `-GLA_PIXEL_OFS`).
pub fn wm_window_viewport(win_size: (i32, i32), gpu: &mut GpuView) {
    wm_window_viewport_ex(win_size, -GLA_PIXEL_OFS, gpu);
}

/// `wmWindowViewportTitle_ex` (verbatim).
pub fn wm_window_viewport_title_ex(rect: &Rcti, offset: f32, gpu: &mut GpuView) {
    (gpu.viewport)(rect.xmin, rect.ymin, rect.xmax, rect.ymax);
    (gpu.scissor)(rect.xmin, rect.ymin, rect.xmax, rect.ymax);
    wm_ortho2_offset(rect.xmax as f32, rect.ymax as f32, offset, gpu);
    (gpu.matrix_identity_set)();
}

/// `wmWindowViewportTitle` (verbatim).
pub fn wm_window_viewport_title(rect: &Rcti, gpu: &mut GpuView) {
    wm_window_viewport_title_ex(rect, -GLA_PIXEL_OFS, gpu);
}

/// `wmOrtho2` (verbatim — degenerate-guarded ortho set).
pub fn wm_ortho2(x1: f32, x2: f32, y1: f32, y2: f32, gpu: &mut GpuView) {
    let mut x2 = x2;
    let mut y2 = y2;
    if x2 == x1 {
        x2 += 1.0;
    }
    if y2 == y1 {
        y2 += 1.0;
    }
    (gpu.matrix_ortho_set)(x1, x2, y1, y2, -100.0, 100.0);
}

/// `wmOrtho2_offset` (verbatim).
pub fn wm_ortho2_offset(x: f32, y: f32, ofs: f32, gpu: &mut GpuView) {
    wm_ortho2(ofs, x + ofs, ofs, y + ofs, gpu);
}

/// `wmOrtho2_region_pixelspace` (verbatim — `-0.01`).
pub fn wm_ortho2_region_pixelspace(winx: i32, winy: i32, gpu: &mut GpuView) {
    wm_ortho2_offset(winx as f32, winy as f32, -0.01, gpu);
}

/// `wmOrtho2_pixelspace` (verbatim — `-GLA_PIXEL_OFS`).
pub fn wm_ortho2_pixelspace(x: f32, y: f32, gpu: &mut GpuView) {
    wm_ortho2_offset(x, y, -GLA_PIXEL_OFS, gpu);
}
