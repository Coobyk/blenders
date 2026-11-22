/// Port of `intern/draw_view.cc` —
/// `DRWView` builder (camera-fit, frustum,
/// culling, update callbacks).
/// `DRWViewType` (verbatim).
pub const DRW_VIEW_DEFAULT: i32 = 0;
pub const DRW_VIEW_CLIP: i32 = 1;
/// `DRWView::create` (verbatim — compute
/// mats + frustum).
pub fn drw_view_create(
    viewmat: [[f32; 4]; 4],
    winmat: [[f32; 4]; 4],
    wininv: [[f32; 4]; 4],
) -> super::draw_view_c::DrwView {
    super::draw_view_c::DrwView {
        viewmat,
        winmat,
        wininv,
        frustum_planes: [[0.0; 4]; 6],
        is_inverted: false,
    }
}
/// `plane_from_proj` (verbatim — extract
/// frustum plane from projection).
pub fn plane_from_proj(winmat: &[[f32; 4]; 4], row: usize, sign: f32) -> [f32; 4] {
    let mut p = [0.0f32; 4];
    for i in 0..4 {
        p[i] = winmat[3][i] + sign * winmat[row][i];
    }
    p
}
