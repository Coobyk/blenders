//! Port of `intern/sound_reader_cache.cc` — `SoundReaderCache` +
//! `SoundReaderLease` (Audaspace reader pooling, `max_idle_readers = 2`).
//!
//! `aud::ISound`/`aud::IReader` are `usize` handles until Audaspace lands;
//! the pooling semantics (keep up to 2 idle readers per sound) are faithful.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const MAX_IDLE_READERS: usize = 2;

/// `SoundReaderCacheState` — per-sound idle reader pool.
pub struct SoundReaderCacheState {
    /// `aud::ISound` handle.
    pub sound: usize,
    pub mutex: Mutex<Vec<usize>>, /* idle_readers */
}

impl SoundReaderCacheState {
    pub fn new(sound: usize) -> Self {
        Self { sound, mutex: Mutex::new(Vec::new()) }
    }
    /// `release` — return reader to the idle pool (≤2 kept).
    pub fn release(&self, reader: usize) {
        if reader == 0 {
            return;
        }
        let mut idle = self.mutex.lock().unwrap();
        if idle.len() < MAX_IDLE_READERS {
            idle.push(reader);
        }
    }
}

/// `SoundReaderLease` — RAII reader handle; Drop returns it to the pool.
pub struct SoundReaderLease {
    state: Arc<SoundReaderCacheState>,
    /// `aud::IReader` handle (0 = none).
    pub reader: usize,
}

impl SoundReaderLease {
    pub fn new(state: Arc<SoundReaderCacheState>, reader: usize) -> Self {
        Self { state, reader }
    }
    /// `release` — idempotent.
    pub fn release(&mut self) {
        if self.reader != 0 {
            self.state.release(self.reader);
            self.reader = 0;
        }
    }
}

impl Drop for SoundReaderLease {
    fn drop(&mut self) {
        self.release();
    }
}

/// `SoundReaderCache` — `sound → state` map (`get_readers` returns all idle
/// readers for a sound for cache-invalidation).
#[derive(Default)]
pub struct SoundReaderCache {
    states: HashMap<usize, Arc<SoundReaderCacheState>>,
}

impl SoundReaderCache {
    /// `SoundReaderCache::state_for` — get-or-insert.
    pub fn state_for(&mut self, sound: usize) -> Arc<SoundReaderCacheState> {
        self.states
            .entry(sound)
            .or_insert_with(|| Arc::new(SoundReaderCacheState::new(sound)))
            .clone()
    }

    /// `get_readers` — drain the idle list of one sound (used by
    /// `BKE_sound_reader_free` to close leaked readers).
    pub fn get_readers(&self, sound: usize) -> Vec<usize> {
        match self.states.get(&sound) {
            Some(state) => {
                let mut idle = state.mutex.lock().unwrap();
                std::mem::take(&mut *idle)
            }
            None => Vec::new(),
        }
    }
}
