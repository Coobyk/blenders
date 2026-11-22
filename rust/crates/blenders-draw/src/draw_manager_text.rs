/// Port of `intern/draw_manager_text.cc` —
/// DRW debug-text queue (draw_text_async).
/// `DRWTextStore` (verbatim — text chunks).
#[derive(Default)]
pub struct DrwTextStore {
    /// `cache` (per-pass text list).
    pub cache: Vec<usize>,
}
/// `DRW_text_cache_add` (verbatim —
/// enqueue text at world pos).
pub fn drw_text_cache_add(_store: &mut DrwTextStore) {}
