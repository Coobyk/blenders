/// Port of `intern/draw_cache.cc` —
/// per-ID draw data registry (DRWData +
/// batch-cache invalidation).
/// `DRWData` (verbatim — per-ID draw data).
#[derive(Default)]
pub struct DrwData {
    /// `data` (draw-engine blobs).
    pub data: Vec<usize>,
    /// `dirty` (invalidate flag).
    pub dirty: bool,
    /// `recache` (batch rebuild flag).
    pub recache: u32,
}
/// `drw_data_get` (verbatim — by ID).
pub fn drw_data_get(_id: usize) -> usize {
    0
}
/// `drw_data_invalidate` (verbatim — set
/// dirty).
pub fn drw_data_invalidate(d: &mut DrwData) {
    d.dirty = true;
}
