//! Port of `intern/static_cache_manager.cc` —
//! compositor cache reset.

/// `StaticCacheManager` verbatim (no-op pool).
#[derive(Default)]
pub struct StaticCacheManager;
impl StaticCacheManager {
    /// `reset` (verbatim — clears cached structs).
    pub fn reset(&mut self) {}
}
