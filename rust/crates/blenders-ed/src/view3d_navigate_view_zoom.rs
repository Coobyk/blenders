/// Port of `view3d_navigate_view_zoom.cc`
/// — viewport zoom (`viewzoom` op: scale
/// `dist` exponentially).
/// `viewzoom_exec` core (verbatim —
/// dist *= exp(-delta)).
pub fn view_zoom(dist: &mut f32, delta: f32, zoom_invert: bool) {
    let d = if zoom_invert { -delta } else { delta };
    *dist *= (-d).exp();
    *dist = dist.clamp(0.0001, 10000.0);
}
