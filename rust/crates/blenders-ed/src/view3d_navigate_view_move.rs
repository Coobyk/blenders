/// Port of `view3d_navigate_view_move.cc`
/// — viewport move (dolly+pan combined).
/// `viewmove_exec` core (verbatim).
pub fn view_move(ofs: &mut [f32; 3], d: [f32; 3]) {
    ofs[0] += d[0];
    ofs[1] += d[1];
    ofs[2] += d[2];
}
