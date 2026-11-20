//! Port of `memory_cache_file_load.cc` — `memory_cache::get_loaded` /
//! `get_loaded_base`: file-backed cache entries invalidated on mtime change.

use crate::memory_cache::{self, CachedValue, GenericKey};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

struct FileStatMap {
    mtimes: HashMap<String, i64>,
}

fn get_file_stat_map() -> &'static Mutex<FileStatMap> {
    static M: OnceLock<Mutex<FileStatMap>> = OnceLock::new();
    M.get_or_init(|| {
        Mutex::new(FileStatMap {
            mtimes: HashMap::new(),
        })
    })
}

/// `get_file_modification_time` — mtime in seconds, None on error.
fn get_file_modification_time(path: &str) -> Option<i64> {
    std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}

/// `invalidate_outdated_caches_if_necessary` — drop cache entries whose
/// files changed on disk.
pub fn invalidate_outdated_caches_if_necessary(file_paths: &[String]) {
    let mut stat_map = get_file_stat_map().lock().unwrap();
    let mut any_outdated = false;
    for path in file_paths {
        let new_mtime = get_file_modification_time(path);
        match stat_map.mtimes.get(path) {
            Some(&old) if Some(old) != new_mtime => {
                any_outdated = true;
            }
            None => {}
            _ => {}
        }
        if let Some(m) = new_mtime {
            stat_map.mtimes.insert(path.clone(), m);
        }
    }
    if any_outdated {
        // C removes entries whose key references outdated files; the key
        // prefix check is approximated by dropping all entries.
        memory_cache::clear();
    }
}

/// `memory_cache::get_loaded_base` — compute-or-get with file-stat invalidation.
pub fn get_loaded_base(
    loader_key: &dyn GenericKey,
    file_paths: &[String],
    compute_fn: impl FnOnce() -> Arc<dyn CachedValue>,
) -> Arc<dyn CachedValue> {
    invalidate_outdated_caches_if_necessary(file_paths);
    memory_cache::get_base(loader_key, compute_fn)
}
