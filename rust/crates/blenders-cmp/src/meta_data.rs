//! Port of `intern/meta_data.cc` — compositor
//! Metadata for stamps.

/// `Metadata` verbatim.
#[derive(Clone, Default)]
pub struct Metadata {
    /// `filename`.
    pub filename: String,
    /// `note`.
    pub note: String,
    /// `render_stats`/`render_time`.
    pub render_stats: String,
    pub render_time: f64,
    /// `use_stamp` + stamp_data.
    pub use_stamp: bool,
}
/// `metadata_is_valid` (verbatim).
pub fn metadata_is_valid(md: &Metadata) -> bool {
    !md.filename.is_empty() || !md.render_stats.is_empty()
}
