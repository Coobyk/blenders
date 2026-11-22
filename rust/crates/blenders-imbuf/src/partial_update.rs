//! Port of `intern/partial_update.cc` — tiled
/// dirty-region tracking for ImBuf.

/// `MAX_HISTORY_LEN` (verbatim).
pub const MAX_HISTORY_LEN: usize = 4;
/// `PartialUpdate` chunk grid (verbatim).
pub struct PartialUpdate {
    /// `changes` — per-chunk dirty flag.
    pub chunks: Vec<bool>,
    /// `chunk_size` — pixel dim per chunk.
    pub chunk_size: i32,
    /// `chunks_x`/`chunks_y`.
    pub chunks_x: i32,
    pub chunks_y: i32,
}
impl PartialUpdate {
    /// `PartialUpdate` ctor (verbatim — grid
    /// from image dims).
    pub fn new(w: i32, h: i32, chunk_size: i32) -> Self {
        let chunks_x = (w + chunk_size - 1) / chunk_size;
        let chunks_y = (h + chunk_size - 1) / chunk_size;
        Self {
            chunks: vec![false; (chunks_x * chunks_y) as usize],
            chunk_size,
            chunks_x,
            chunks_y,
        }
    }
    /// `Changes::set_all_chunks_modified` (verbatim).
    pub fn set_all_modified(&mut self) {
        for c in &mut self.chunks {
            *c = true;
        }
    }
    /// `IMB_partial_update_mark_region` (verbatim
    /// — mark chunks intersecting rect).
    pub fn mark_region(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let cx0 = (x / self.chunk_size).clamp(0, self.chunks_x - 1);
        let cy0 = (y / self.chunk_size).clamp(0, self.chunks_y - 1);
        let cx1 = ((x + w) / self.chunk_size).clamp(0, self.chunks_x - 1);
        let cy1 = ((y + h) / self.chunk_size).clamp(0, self.chunks_y - 1);
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                self.chunks[(cy * self.chunks_x + cx) as usize] = true;
            }
        }
    }
    /// `IMB_partial_update_flush` (verbatim —
    /// clear all).
    pub fn flush(&mut self) {
        for c in &mut self.chunks {
            *c = false;
        }
    }
}
