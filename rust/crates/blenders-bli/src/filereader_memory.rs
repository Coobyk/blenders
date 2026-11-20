//! Port of `filereader_memory.cc` — `BLI_filereader_new_memory` and
//! `BLI_filereader_new_mmap`: `FileReader` over an in-memory buffer and
//! over a mapped file.

use crate::filereader_file::{FileReader, SEEK_CUR, SEEK_END, SEEK_SET};

/// `MemoryReader` — `FileReader` over a byte slice.
pub struct MemoryReader {
    data: Vec<u8>,
    offset: i64,
}

/// `BLI_filereader_new_memory(data, size)` — copies `data` (C borrowed).
pub fn filereader_new_memory(data: &[u8]) -> Box<dyn FileReader> {
    Box::new(MemoryReader {
        data: data.to_vec(),
        offset: 0,
    })
}

/// `BLI_filereader_new_mmap(mmap_file)` — read via `MmapFile`.
pub fn filereader_new_mmap(file: &crate::mmap::MmapFile) -> Box<dyn FileReader> {
    let mut data = vec![0u8; file.length()];
    let _ = file.read(&mut data, 0);
    Box::new(MemoryReader { data, offset: 0 })
}

impl FileReader for MemoryReader {
    fn read(&mut self, buffer: &mut [u8]) -> i64 {
        let avail = (self.data.len() as i64 - self.offset).max(0) as usize;
        let n = avail.min(buffer.len());
        buffer[..n].copy_from_slice(&self.data[self.offset as usize..self.offset as usize + n]);
        self.offset += n as i64;
        n as i64
    }
    fn seek(&mut self, offset: i64, whence: i32) -> i64 {
        let new = match whence {
            SEEK_SET => offset,
            SEEK_CUR => self.offset + offset,
            SEEK_END => self.data.len() as i64 + offset,
            _ => return -1,
        };
        if new < 0 {
            return -1;
        }
        self.offset = new.min(self.data.len() as i64);
        self.offset
    }
    fn offset(&self) -> i64 {
        self.offset
    }
}
