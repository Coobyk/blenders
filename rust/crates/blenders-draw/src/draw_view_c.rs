/// Port of `intern/draw_view_c.cc` — DRWView
/// (camera matrices + culling).
/// `DRWView` (verbatim — view/proj/win mat +
/// frustum planes).
pub struct DrwView {
    /// `viewmat`.
    pub viewmat: [[f32; 4]; 4],
    /// `winmat`.
    pub winmat: [[f32; 4]; 4],
    /// `wininv`.
    pub wininv: [[f32; 4]; 4],
    /// `frustum_planes` [6].
    pub frustum_planes: [[f32; 4]; 6],
    /// `is_inverted`.
    pub is_inverted: bool,
}
