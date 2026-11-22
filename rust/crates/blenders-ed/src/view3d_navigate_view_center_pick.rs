/// Port of `view3d_navigate_view_center_pick.cc`
/// — `view center pick` (set `ofs` to
/// picked point).
/// `view_center_pick` (verbatim —
/// ofs = -hit).
pub fn view_center_pick(ofs: &mut [f32; 3], hit: [f32; 3]) {
    *ofs = [-hit[0], -hit[1], -hit[2]];
}
