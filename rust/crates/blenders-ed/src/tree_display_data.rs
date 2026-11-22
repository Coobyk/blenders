/// Port of `space_outliner/tree/tree_display_data.cc`
/// — outliner display-data storage.
/// `TreeDisplayData` (verbatim — per-element
/// state).
pub struct TreeDisplayData {
    /// `drawn_elements` (visible row list).
    pub drawn_elements: Vec<usize>,
}
