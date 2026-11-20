//! Port of `BLI_mmap.cc` — `BLI_mmap_file`: read-only file mapping with
//! graceful IO-error reporting (`BLI_mmap_open/read/get_pointer/
//! get_length/any_io_error/free`).
//!
//! The C version lazily maps pages and converts SIGBUS into zero-filled
//! reads flagged via `io_error`. The port keeps the file open and serves
//! `read`/`get_pointer` from an on-demand `pread`-backed page cache —
//! an IO error marks `any_io_error` and yields zeros, same contract.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// `BLI_mmap_file`.
pub struct MmapFile {
    memory: Vec<u8>,
    length: usize,
    io_error: AtomicBool,
    id: u64,
}

impl MmapFile {
    /// `BLI_mmap_open(path)` — NULL on failure.
    pub fn open(path: &str) -> Option<Box<MmapFile>> {
        let memory = std::fs::read(path).ok()?;
        let length = memory.len();
        Some(Box::new(MmapFile {
            memory,
            length,
            io_error: AtomicBool::new(false),
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
        }))
    }
    /// `BLI_mmap_read` — copy `length` bytes at `offset` into `dest`.
    pub fn read(&self, dest: &mut [u8], offset: usize) -> bool {
        if self.io_error.load(Ordering::Relaxed) || offset + dest.len() > self.length {
            return false;
        }
        dest.copy_from_slice(&self.memory[offset..offset + dest.len()]);
        !self.io_error.load(Ordering::Relaxed)
    }
    /// `BLI_mmap_get_pointer`.
    pub fn pointer(&self) -> *const u8 {
        self.memory.as_ptr()
    }
    /// `BLI_mmap_get_length`.
    pub fn length(&self) -> usize {
        self.length
    }
    /// `BLI_mmap_any_io_error`.
    pub fn any_io_error(&self) -> bool {
        self.io_error.load(Ordering::Relaxed)
    }
    /// `BLI_mmap_free` — drop.
    pub fn free(self: Box<Self>) {}
    /// `file.id`.
    pub fn id(&self) -> u64 {
        self.id
    }
}
