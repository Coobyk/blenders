/// Port of `view3d_navigate_view_center_cursor.cc`
/// — `view center cursor` (set `ofs` to
/// cursor pos).
/// `view_center_cursor` (verbatim —
/// ofs = -cursor).
pub fn view_center_cursor(ofs: &mut [f32; 3], cursor: [f32; 3]) {
    *ofs = [-cursor[0], -cursor[1], -cursor[2]];
}
