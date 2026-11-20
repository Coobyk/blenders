//! Port of `filereader_file.cc` — `BLI_filereader_new_file`: a `FileReader`
//! over a raw file descriptor.

use std::io::{Read, Seek, SeekFrom};

/// `FileReader` — the vtable ported as a trait.
pub trait FileReader: Send {
    /// `reader.read(buffer)` — returns bytes read or -1 on error.
    fn read(&mut self, buffer: &mut [u8]) -> i64;
    /// `reader.seek(offset, whence)` — new absolute offset.
    fn seek(&mut self, offset: i64, whence: i32) -> i64;
    /// `reader.offset`.
    fn offset(&self) -> i64;
    /// `reader.close` — drop.
    fn close(self: Box<Self>) {}
}

/// `SEEK_SET`/`SEEK_CUR`/`SEEK_END` constants.
pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;

/// `RawFileReader` — `FileReader` over `std::fs::File`.
pub struct RawFileReader {
    file: std::fs::File,
    offset: i64,
}

/// `BLI_filereader_new_file` — wrap an opened `File`.
pub fn filereader_new_file(file: std::fs::File) -> Box<dyn FileReader> {
    Box::new(RawFileReader { file, offset: 0 })
}

impl FileReader for RawFileReader {
    fn read(&mut self, buffer: &mut [u8]) -> i64 {
        match self.file.read(buffer) {
            Ok(n) => {
                self.offset += n as i64;
                n as i64
            }
            Err(_) => -1,
        }
    }
    fn seek(&mut self, offset: i64, whence: i32) -> i64 {
        let pos = match whence {
            SEEK_SET => SeekFrom::Start(offset as u64),
            SEEK_CUR => SeekFrom::Current(offset),
            SEEK_END => SeekFrom::End(offset),
            _ => return -1,
        };
        match self.file.seek(pos) {
            Ok(n) => {
                self.offset = n as i64;
                n as i64
            }
            Err(_) => -1,
        }
    }
    fn offset(&self) -> i64 {
        self.offset
    }
}
