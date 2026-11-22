/// Port of `intern/draw_view_data.cc` —
/// DRWViewData (per-view engine state).
/// `DRWViewData` (verbatim — engine list).
pub struct DrwViewData {
    /// `engines`.
    pub engines: Vec<usize>,
}
