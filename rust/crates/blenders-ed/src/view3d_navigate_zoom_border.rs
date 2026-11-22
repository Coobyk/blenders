/// Port of `view3d_navigate_zoom_border.cc`
/// — `zoom border` (rubber-band zoom:
/// dist *= region/rect ratio).
/// `zoom_border` (verbatim).
pub fn zoom_border(dist: &mut f32, ofs: &mut [f32; 3], rect_w: f32, rect_h: f32, win_w: f32, win_h: f32, center_world: [f32; 3]) {
    *dist *= (rect_w / win_w).min(rect_h / win_h).max(0.01);
    *ofs = [-center_world[0], -center_world[1], -center_world[2]];
}
