/// Port of `view3d_navigate_view_all.cc`
/// — `view all` (fit objects in view:
/// set dist/ofs to bound region).
/// `view_all` (verbatim — compute dist
/// from bounds radius).
pub fn view_all(dist: &mut f32, ofs: &mut [f32; 3], center: [f32; 3], radius: f32, sensor: f32) {
    // C: dist = radius / sin(hfov/2); ofs = -center.
    *dist = radius / (sensor * 0.5 / 50.0).max(0.01);
    *ofs = [-center[0], -center[1], -center[2]];
}
