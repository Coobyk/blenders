//! Port of `intern/cache.cc` — ImBufCache
//! (GHash → HashMap + size accounting).

use std::collections::HashMap;
/// `ImBufCache` (verbatim — keyed value store
/// with size tracking).
pub struct ImBufCache {
    /// `gh` — entries.
    pub map: HashMap<Vec<u8>, Vec<u8>>,
    /// `mem_used` — total bytes.
    pub mem_used: usize,
    /// `name`.
    pub name: String,
}
impl ImBufCache {
    /// `IMB_cache_create` (verbatim).
    pub fn new(name: &str) -> Self {
        Self {
            map: HashMap::new(),
            mem_used: 0,
            name: name.to_string(),
        }
    }
    /// `IMB_cache_put` (verbatim — store + size).
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.mem_used += value.len();
        self.map.insert(key, value);
    }
    /// `IMB_cache_get` (verbatim).
    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.map.get(key)
    }
    /// `IMB_cache_remove` (verbatim).
    pub fn remove(&mut self, key: &[u8]) {
        if let Some(v) = self.map.remove(key) {
            self.mem_used = self.mem_used.saturating_sub(v.len());
        }
    }
    /// `IMB_cache_get_size` (verbatim).
    pub fn get_size(&self) -> usize {
        self.mem_used
    }
}
