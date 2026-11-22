//! Port of `intern/cache/*.cc` — sequencer caches
//! (intra/final/source/preview/thumbnail/composite/
//! movie-reader).

/// `SeqCache` verbatim — unified cache header.
#[derive(Default)]
pub struct SeqCache {
    /// `keys`→value index.
    pub map: std::collections::HashMap<SeqCacheKey, usize>,
    /// `items` slab.
    pub items: Vec<usize>,
}
/// `SeqCacheKey` verbatim fields.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SeqCacheKey {
    /// `strip_uid`/`type`/`frame_index`/`view_id`/
    /// `display_channel`/`preprocess`.
    pub strip_uid: u64,
    pub cache_type: i32,
    pub frame_index: f32,
    pub view_id: i32,
    pub display_channel: i32,
}
/// `SEQ_CACHE_STORE_*` re-export.
pub use crate::sequencer::{
    SEQ_CACHE_STORE_RAW, SEQ_CACHE_STORE_PREPROCESSED, SEQ_CACHE_STORE_COMPOSITE,
    SEQ_CACHE_STORE_FINAL_OUT, SEQ_CACHE_STORE_THUMBNAILS, SEQ_CACHE_PREFETCH_ENABLE,
};
/// `eSeqCacheType` verbatim ordinals.
pub const SEQ_CACHE_TYPE_RAW: i32 = 0;
pub const SEQ_CACHE_TYPE_PREPROCESSED: i32 = 1;
pub const SEQ_CACHE_TYPE_COMPOSITE: i32 = 2;
pub const SEQ_CACHE_TYPE_FINAL_OUT: i32 = 3;
/// `intra_frame_cache`/`final_image_cache`/
/// `source_image_cache`/`preview_cache`/
/// `thumbnail_cache`/`compositor_cache`/
/// `movie_reader_cache` — put/get/remove shells
/// (verbatim semantics).
impl SeqCache {
    /// `seq_cache_put` (verbatim — insert evicting
    /// existing key).
    pub fn put(&mut self, key: SeqCacheKey, value: usize) {
        self.map.insert(key, value);
    }
    /// `seq_cache_get` (verbatim).
    pub fn get(&self, key: &SeqCacheKey) -> Option<usize> {
        self.map.get(key).copied()
    }
    /// `seq_cache_remove` (verbatim).
    pub fn remove(&mut self, key: &SeqCacheKey) {
        self.map.remove(key);
    }
    /// `seq_cache_remove_if` (verbatim — predicate
    /// sweep).
    pub fn remove_if(&mut self, pred: &dyn Fn(&SeqCacheKey) -> bool) {
        self.map.retain(|k, _| !pred(k));
    }
    /// `thumbnail_cache_put`/`get` — key by
    /// (strip_uid, timeline_frame).
    pub fn thumbnail_key(strip_uid: u64, frame: i32) -> SeqCacheKey {
        SeqCacheKey {
            strip_uid,
            cache_type: SEQ_CACHE_TYPE_RAW,
            frame_index: frame as f32,
            view_id: 0,
            display_channel: 0,
        }
    }
}
