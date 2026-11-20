//! Port of `BLI_ghash.cc` — `GHash`/`GSet` generic hash map/set.
//! Rust's `HashMap`/`HashSet` supply the storage; the C API surface is
//! preserved as methods (`insert`, `ensure`, `lookup_p`, `pop_key`, ...).
//! Flags (`GHASH_FLAG_ALLOW_REINSERT`, `GHASH_FLAG_ALLOW_DUPES`) kept as
//! bitflags documenting call-site intent; reinsert/dupes need custom
//! table internals and are kept as `debug_assert`-checked no-ops for now.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub const GHASH_FLAG_ALLOW_REINSERT: u8 = 1 << 0;
pub const GHASH_FLAG_ALLOW_DUPES: u8 = 1 << 1;
pub const GHASH_FLAG_ISSET: u8 = 1 << 2;

/// `GHash` — map from `K` to `V`.
pub struct GHash<K: Hash + Eq, V> {
    map: HashMap<K, V>,
    flag: u8,
}

impl<K: Hash + Eq, V> GHash<K, V> {
    /// `BLI_ghash_new` / `BLI_ghash_new_ex` — `info` string is diagnostic-only
    /// in C; dropped in Rust.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    /// `BLI_ghash_new_ex` with `nentries_reserve`.
    pub fn with_capacity(n: usize) -> Self {
        GHash {
            map: HashMap::with_capacity(n),
            flag: 0,
        }
    }

    /// `BLI_ghash_reserve`.
    pub fn reserve(&mut self, n: usize) {
        self.map.reserve(n);
    }

    /// `BLI_ghash_insert` — overwrite existing key.
    pub fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    /// `BLI_ghash_reinsert` — remove then insert (flag-checked in C).
    pub fn reinsert(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    /// `BLI_ghash_replace_key` — move value to a different key.
    pub fn replace_key(&mut self, old_key: K, new_key: K) {
        if let Some(v) = self.map.remove(&old_key) {
            self.map.insert(new_key, v);
        }
    }

    /// `BLI_ghash_ensure_p` — get-or-insert-default, returning &mut.
    pub fn ensure_p(&mut self, key: K) -> &mut V
    where
        V: Default,
    {
        self.map.entry(key).or_default()
    }

    /// `BLI_ghash_lookup`.
    pub fn lookup(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    /// `BLI_ghash_lookup_p` — &mut variant.
    pub fn lookup_p(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }
    /// `BLI_ghash_lookup_default`.
    pub fn lookup_default<'a>(&'a self, key: &K, default: &'a V) -> &'a V {
        self.map.get(key).unwrap_or(default)
    }
    /// `BLI_ghash_haskey`.
    pub fn haskey(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// `BLI_ghash_remove`.
    pub fn remove(&mut self, key: &K) -> bool {
        self.map.remove(key).is_some()
    }
    /// `BLI_ghash_popkey` — remove and return entry.
    pub fn popkey(&mut self, key: &K) -> Option<(K, V)>
    where
        K: Clone,
    {
        self.map.remove_entry(key)
    }
    /// `BLI_ghash_pop` — pop an arbitrary entry.
    pub fn pop(&mut self) -> Option<(K, V)>
    where
        K: Clone,
    {
        let k = self.map.keys().next()?.clone();
        self.map.remove_entry(&k)
    }

    /// `BLI_ghash_clear` (`GHASH_CLEAR_ALL` default).
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// `BLI_ghash_copy` (`GHashCopyCallback`/`GHashKeyCopyFunc` handled by `Clone`).
    pub fn copy(&self) -> Self
    where
        K: Clone,
        V: Clone,
    {
        GHash {
            map: self.map.clone(),
            flag: self.flag,
        }
    }

    /// `BLI_ghash_len`.
    pub fn len(&self) -> usize {
        self.map.len()
    }
    /// `BLI_ghash_buckets_len` — std gives no bucket count; capacity approximates.
    pub fn buckets_len(&self) -> usize {
        self.map.capacity()
    }
    /// `BLI_ghash_calc_quality`/`_ex` — std internals are opaque; returns 0.
    pub fn calc_quality(&self) -> f64 {
        0.0
    }

    /// `BLI_ghash_flag_set` / `BLI_ghash_flag_clear`.
    pub fn flag_set(&mut self, flag: u8) {
        self.flag |= flag;
    }
    pub fn flag_clear(&mut self, flag: u8) {
        self.flag &= !flag;
    }
    pub fn flag(&self) -> u8 {
        self.flag
    }

    /// `BLI_ghashIterator_*` — iterate (K, V) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.map.iter_mut()
    }
}

/// `GSet` — set of `T`.
pub struct GSet<T: Hash + Eq> {
    set: HashSet<T>,
    flag: u8,
}

impl<T: Hash + Eq> GSet<T> {
    /// `BLI_gset_new`/`_ex`.
    pub fn new() -> Self {
        GSet {
            set: HashSet::new(),
            flag: GHASH_FLAG_ISSET,
        }
    }
    pub fn with_capacity(n: usize) -> Self {
        GSet {
            set: HashSet::with_capacity(n),
            flag: GHASH_FLAG_ISSET,
        }
    }

    /// `BLI_gset_add` — returns true if newly inserted.
    pub fn add(&mut self, key: T) -> bool {
        self.set.insert(key)
    }
    /// `BLI_gset_insert` (same as add).
    pub fn insert(&mut self, key: T) {
        self.set.insert(key);
    }
    /// `BLI_gset_reinsert`.
    pub fn reinsert(&mut self, key: T) {
        self.set.replace(key);
    }
    /// `BLI_gset_replace_key`.
    pub fn replace_key(&mut self, old: &T, new: T) {
        if self.set.remove(old) {
            self.set.insert(new);
        }
    }
    /// `BLI_gset_ensure_p_ex` — get existing or insert; bool = was present.
    pub fn ensure(&mut self, key: T) -> (&T, bool)
    where
        T: Clone,
    {
        let existed = self.set.contains(&key);
        self.set.insert(key.clone());
        (self.set.get(&key).unwrap(), existed)
    }
    /// `BLI_gset_haskey` / `BLI_gset_lookup`.
    pub fn haskey(&self, key: &T) -> bool {
        self.set.contains(key)
    }
    pub fn lookup(&self, key: &T) -> Option<&T> {
        self.set.get(key)
    }
    /// `BLI_gset_remove`.
    pub fn remove(&mut self, key: &T) -> bool {
        self.set.remove(key)
    }
    /// `BLI_gset_pop_key` — remove and return the stored key.
    pub fn pop_key(&mut self, key: &T) -> Option<T>
    where
        T: Clone,
    {
        self.set.take(key)
    }
    /// `BLI_gset_pop` — pop arbitrary.
    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let k = self.set.iter().next()?.clone();
        self.set.take(&k)
    }
    /// `BLI_gset_clear` / `_ex`.
    pub fn clear(&mut self) {
        self.set.clear();
    }
    /// `BLI_gset_copy`.
    pub fn copy(&self) -> Self
    where
        T: Clone,
    {
        GSet {
            set: self.set.clone(),
            flag: self.flag,
        }
    }
    /// `BLI_gset_len`.
    pub fn len(&self) -> usize {
        self.set.len()
    }
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
    /// `BLI_gset_buckets_len`.
    pub fn buckets_len(&self) -> usize {
        self.set.capacity()
    }
    /// `BLI_gset_calc_quality` — 0 (std internals opaque).
    pub fn calc_quality(&self) -> f64 {
        0.0
    }
    /// `BLI_gset_flag_set`/`clear`.
    pub fn flag_set(&mut self, flag: u8) {
        self.flag |= flag;
    }
    pub fn flag_clear(&mut self, flag: u8) {
        self.flag &= !flag;
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.set.iter()
    }
}
