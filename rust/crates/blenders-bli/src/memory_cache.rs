//! Port of `memory_cache.cc` — `blender::memory_cache`: a global,
//! size-limited, approximate-LRU cache of computed values.
//!
//! `GenericKey`/`CachedValue` are trait objects; keys compare by their
//! `key_string()` byte identity like the C type-erased keys.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// `CachedValue` — a storable cache payload.
pub trait CachedValue: Send + Sync {
    /// `count_memory(counter)` — report heap bytes held by this value.
    fn count_memory(&self) -> i64;
}

/// `GenericKey` — comparable key; identity is `key_string()` bytes.
pub trait GenericKey: Send + Sync {
    fn key_string(&self) -> Vec<u8>;
}

struct StoredValue {
    value: Arc<dyn CachedValue>,
    last_use_time: AtomicI64,
}

struct Cache {
    map: HashMap<Vec<u8>, StoredValue>,
    keys: Vec<Vec<u8>>,
    memory_bytes: i64,
    global_mutex: (),
}

static LOGICAL_TIME: AtomicI64 = AtomicI64::new(0);
static APPROXIMATE_LIMIT: AtomicI64 = AtomicI64::new(1024 * 1024 * 1024);
static SIZE_IN_BYTES: AtomicI64 = AtomicI64::new(0);
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

fn get_cache() -> &'static Mutex<Cache> {
    CACHE.get_or_init(|| {
        Mutex::new(Cache {
            map: HashMap::new(),
            keys: Vec::new(),
            memory_bytes: 0,
            global_mutex: (),
        })
    })
}

/// `memory_cache::get_base` — lookup-or-compute with LRU timestamping.
pub fn get_base(
    key: &dyn GenericKey,
    compute_fn: impl FnOnce() -> Arc<dyn CachedValue>,
) -> Arc<dyn CachedValue> {
    let cache_lock = get_cache();
    let new_time = LOGICAL_TIME.fetch_add(1, Ordering::Relaxed);
    let k = key.key_string();
    {
        let cache = cache_lock.lock().unwrap();
        if let Some(sv) = cache.map.get(&k) {
            sv.last_use_time.store(new_time, Ordering::Relaxed);
            return sv.value.clone();
        }
    }
    let result = compute_fn();
    {
        let mut cache = cache_lock.lock().unwrap();
        if let Some(sv) = cache.map.get(&k) {
            sv.last_use_time.store(new_time, Ordering::Relaxed);
            return sv.value.clone();
        }
        let bytes = result.count_memory();
        cache.memory_bytes += bytes;
        cache.map.insert(
            k.clone(),
            StoredValue {
                value: result.clone(),
                last_use_time: AtomicI64::new(new_time),
            },
        );
        cache.keys.push(k);
        SIZE_IN_BYTES.store(cache.memory_bytes, Ordering::Relaxed);
    }
    try_enforce_limit();
    result
}

/// `memory_cache::set_approximate_size_limit`.
pub fn set_approximate_size_limit(limit_in_bytes: i64) {
    APPROXIMATE_LIMIT.store(limit_in_bytes, Ordering::Relaxed);
    try_enforce_limit();
}

/// `memory_cache::clear`.
pub fn clear() {
    remove_if(|_| true);
}

/// `memory_cache::remove_if` — drop every key matching `predicate`.
pub fn remove_if<F: Fn(&[u8]) -> bool>(predicate: F) {
    let mut cache = get_cache().lock().unwrap();
    let mut memory_bytes = 0i64;
    let keys: Vec<Vec<u8>> = cache.keys.clone();
    for k in &keys {
        if predicate(k) {
            cache.map.remove(k);
        } else if let Some(sv) = cache.map.get(k) {
            memory_bytes += sv.value.count_memory();
        }
    }
    cache.keys.retain(|k| !predicate(k));
    cache.memory_bytes = memory_bytes;
    SIZE_IN_BYTES.store(memory_bytes, Ordering::Relaxed);
    let _ = cache.global_mutex;
}

fn try_enforce_limit() {
    let approximate_limit = APPROXIMATE_LIMIT.load(Ordering::Relaxed);
    if SIZE_IN_BYTES.load(Ordering::Relaxed) < approximate_limit {
        return;
    }
    let mut cache = get_cache().lock().unwrap();
    // newest first by last_use_time
    let mut keys_with_time: Vec<(i64, Vec<u8>)> = cache
        .keys
        .iter()
        .filter_map(|k| {
            cache
                .map
                .get(k)
                .map(|sv| (sv.last_use_time.load(Ordering::Relaxed), k.clone()))
        })
        .collect();
    keys_with_time.sort();
    keys_with_time.reverse();

    // Recount, keeping newest entries within 75% of the limit.
    let mut memory_bytes = 0i64;
    let mut first_bad_index: Option<usize> = None;
    for (i, (_, k)) in keys_with_time.iter().enumerate() {
        if let Some(sv) = cache.map.get(k) {
            memory_bytes += sv.value.count_memory();
            if memory_bytes > (approximate_limit as f64 * 0.75) as i64 {
                first_bad_index = Some(i);
                break;
            }
        }
    }
    let Some(cut) = first_bad_index else { return };
    let need_memory_recount = memory_bytes >= (approximate_limit as f64 * 1.1) as i64;
    if !need_memory_recount && cut == keys_with_time.len() {
        return;
    }
    for (_, k) in keys_with_time.iter().skip(cut) {
        cache.map.remove(k);
    }
    cache.keys = keys_with_time[..cut]
        .iter()
        .map(|(_, k)| k.clone())
        .collect();
    if need_memory_recount {
        memory_bytes = 0;
        for sv in cache.map.values() {
            memory_bytes += sv.value.count_memory();
        }
    }
    cache.memory_bytes = memory_bytes;
    SIZE_IN_BYTES.store(memory_bytes, Ordering::Relaxed);
}

/// `memory_cache::size_in_bytes` (informational).
pub fn size_in_bytes() -> i64 {
    SIZE_IN_BYTES.load(Ordering::Relaxed)
}
