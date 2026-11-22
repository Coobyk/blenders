/// Port of `view3d_navigate_view_pan.cc`
/// — viewport pan (`viewpan` op: translate
/// `ofs` in view plane).
/// `viewpan_exec` core (verbatim — ofs
/// shift by window delta → world).
pub fn view_pan(ofs: &mut [f32; 3], dx_world: f32, dy_world: f32) {
    ofs[0] -= dx_world;
    ofs[1] -= dy_world;
}
