/// Port of `view3d_navigate_view_dolly.cc`
/// — viewport dolly (move camera along
/// forward, dist unchanged).
/// `viewdolly_exec` core (verbatim).
pub fn view_dolly(ofs: &mut [f32; 3], forward: [f32; 3], delta: f32) {
    ofs[0] += forward[0] * delta;
    ofs[1] += forward[1] * delta;
    ofs[2] += forward[2] * delta;
}
