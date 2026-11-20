//! Port of `filereader_zstd.cc` — `ZstdReader`: streaming zstd reader
//! with optional seek-table support (seekable zstd format).

use crate::filereader_file::{FileReader, SEEK_CUR, SEEK_END, SEEK_SET};

struct SeekTable {
    compressed_ofs: Vec<usize>,
    uncompressed_ofs: Vec<usize>,
    cached_content: Option<Vec<u8>>,
    cached_frame: i64,
}

/// `BLI_filereader_new_zstd` — wraps `base`; parses the seek table
/// (magic 0x8F92EAB1 at EOF) when present for random access.
pub fn filereader_new_zstd(mut base: Box<dyn FileReader>) -> Box<dyn FileReader> {
    let seek = read_seek_table(base.as_mut());
    Box::new(ZstdReader {
        base,
        seek,
        in_buf: vec![0u8; 256 * 1024],
        in_pos: 0,
        in_len: 0,
        decoder: zstd::bulk::Decompressor::new().ok(),
        offset: 0,
    })
}

struct ZstdReader {
    base: Box<dyn FileReader>,
    seek: Option<SeekTable>,
    in_buf: Vec<u8>,
    in_pos: usize,
    in_len: usize,
    decoder: Option<zstd::bulk::Decompressor<'static>>,
    offset: i64,
}

fn read_u32(base: &mut dyn FileReader) -> Option<u32> {
    let mut b = [0u8; 4];
    if base.read(&mut b) != 4 {
        return None;
    }
    Some(u32::from_le_bytes(b))
}

/// `zstd_read_seek_table` — parse the seek-table footer (magic
/// 0x8F92EAB1, flags byte, frames_num, per-frame sizes after a
/// 0x184D2A5E skippable frame header).
fn read_seek_table(base: &mut dyn FileReader) -> Option<SeekTable> {
    if base.seek(-4, SEEK_END) < 13 {
        return None;
    }
    if read_u32(base)? != 0x8F92EAB1 {
        return None;
    }
    let mut flags = [0u8; 1];
    if base.seek(-5, SEEK_END) < 0 || base.read(&mut flags) != 1 {
        return None;
    }
    let has_checksums = flags[0] & 0x80 != 0;
    if flags[0] & 0x60 != 0 {
        return None;
    }
    if base.seek(-9, SEEK_END) < 0 {
        return None;
    }
    let frames_num = read_u32(base)? as usize;

    let expected_frame_length = frames_num * if has_checksums { 12 } else { 8 } + 9;
    let frame_start_ofs = 8 + expected_frame_length;
    let seek_frame_start = base.seek(-(frame_start_ofs as i64), SEEK_END);
    if seek_frame_start < (frames_num * 8) as i64 {
        return None;
    }
    if read_u32(base)? != 0x184D2A5E {
        return None;
    }
    if read_u32(base)? != expected_frame_length as u32 {
        return None;
    }

    let mut compressed_ofs = vec![0usize; frames_num + 1];
    let mut uncompressed_ofs = vec![0usize; frames_num + 1];
    let mut compressed = 0usize;
    let mut uncompressed = 0usize;
    for i in 0..frames_num {
        let csize = read_u32(base)? as usize;
        let usize_ = read_u32(base)? as usize;
        if has_checksums && base.seek(4, SEEK_CUR) < 0 {
            return None;
        }
        compressed_ofs[i] = compressed;
        uncompressed_ofs[i] = uncompressed;
        compressed += csize;
        uncompressed += usize_;
    }
    compressed_ofs[frames_num] = compressed;
    uncompressed_ofs[frames_num] = uncompressed;

    if seek_frame_start != compressed as i64 || base.seek(seek_frame_start, SEEK_SET) < 0 {
        return None;
    }
    Some(SeekTable {
        compressed_ofs,
        uncompressed_ofs,
        cached_content: None,
        cached_frame: -1,
    })
}

/// `zstd_frame_from_pos` — binary search the frame containing `pos`.
fn frame_from_pos(seek: &SeekTable, pos: usize) -> i64 {
    let n = seek.compressed_ofs.len() - 1;
    if pos >= seek.uncompressed_ofs[n] {
        return -1;
    }
    let mut low = 0usize;
    let mut high = n;
    while low + 1 < high {
        let mid = low + ((high - low) >> 1);
        if seek.uncompressed_ofs[mid] <= pos {
            low = mid;
        } else {
            high = mid;
        }
    }
    low as i64
}

impl ZstdReader {
    /// `zstd_ensure_cache` — decompress frame `frame` into the cache.
    /// The seek table is moved out to satisfy the borrow checker.
    fn ensure_cache(&mut self, frame: usize) -> bool {
        let mut seek = self.seek.take().unwrap();
        if seek.cached_frame == frame as i64 {
            self.seek = Some(seek);
            return true;
        }
        let cofs = seek.compressed_ofs[frame];
        let usize_ = seek.uncompressed_ofs[frame + 1] - seek.uncompressed_ofs[frame];
        let csize = seek.compressed_ofs[frame + 1] - seek.compressed_ofs[frame];
        let ok = (|| {
            if self.base.seek(cofs as i64, SEEK_SET) < 0 {
                return None;
            }
            let mut compressed = vec![0u8; csize];
            if self.base.read(&mut compressed) < csize as i64 {
                return None;
            }
            zstd::bulk::decompress(&compressed, usize_).ok()
        })();
        match ok {
            Some(data) => {
                seek.cached_frame = frame as i64;
                seek.cached_content = Some(data);
                self.seek = Some(seek);
                true
            }
            None => {
                self.seek = Some(seek);
                false
            }
        }
    }

    /// `zstd_read_seekable`.
    fn read_seekable(&mut self, buffer: &mut [u8]) -> i64 {
        let end_offset = self.offset + buffer.len() as i64;
        let mut read_len = 0usize;
        while self.offset < end_offset {
            let frame = {
                let seek = self.seek.as_ref().unwrap();
                frame_from_pos(seek, self.offset as usize)
            };
            if frame < 0 {
                break;
            }
            let frame = frame as usize;
            let offset_in_frame = {
                let seek = self.seek.as_ref().unwrap();
                self.offset as usize - seek.uncompressed_ofs[frame]
            };
            if !self.ensure_cache(frame) {
                break;
            }
            let (frame_end, data_len) = {
                let seek = self.seek.as_ref().unwrap();
                (
                    (seek.uncompressed_ofs[frame + 1] as i64).min(end_offset),
                    seek.cached_content.as_ref().map(|c| c.len()).unwrap_or(0),
                )
            };
            let frame_read_len = (frame_end - self.offset) as usize;
            let data = self.seek.as_ref().unwrap().cached_content.as_ref().unwrap();
            let copy_len = frame_read_len.min(data_len - offset_in_frame);
            buffer[read_len..read_len + copy_len]
                .copy_from_slice(&data[offset_in_frame..offset_in_frame + copy_len]);
            read_len += copy_len;
            self.offset = frame_end;
        }
        read_len as i64
    }

    /// `zstd_read` — streaming decompress (no seek table).
    fn read_stream(&mut self, buffer: &mut [u8]) -> i64 {
        let mut produced = 0usize;
        while produced < buffer.len() {
            if self.in_pos >= self.in_len {
                self.in_pos = 0;
                let n = self.base.read(&mut self.in_buf);
                if n <= 0 {
                    break;
                }
                self.in_len = n as usize;
            }
            // Stream one chunk through a fresh decoder each time is
            // wrong; keep a persistent StreamingDecompression — the
            // `zstd` crate's stream Decoder wants Read; emulate with a
            // buffered source.
            let chunk = self.in_buf[self.in_pos..self.in_len].to_vec();
            self.in_pos = self.in_len;
            match self
                .decoder
                .as_mut()
                .and_then(|d| d.decompress(&chunk, buffer.len() - produced).ok())
            {
                Some(d) => {
                    let copy = d.len().min(buffer.len() - produced);
                    buffer[produced..produced + copy].copy_from_slice(&d[..copy]);
                    produced += copy;
                }
                None => break,
            }
        }
        self.offset += produced as i64;
        produced as i64
    }
}

impl FileReader for ZstdReader {
    fn read(&mut self, buffer: &mut [u8]) -> i64 {
        if self.seek.is_some() {
            self.read_seekable(buffer)
        } else {
            self.read_stream(buffer)
        }
    }
    fn seek(&mut self, offset: i64, whence: i32) -> i64 {
        let Some(seek) = &self.seek else {
            return -1;
        };
        let total = *seek.uncompressed_ofs.last().unwrap() as i64;
        let new_pos = match whence {
            SEEK_SET => offset,
            SEEK_END => total + offset,
            _ => self.offset + offset,
        };
        if new_pos < 0 || new_pos > total {
            return -1;
        }
        self.offset = new_pos;
        self.offset
    }
    fn offset(&self) -> i64 {
        self.offset
    }
    fn close(mut self: Box<Self>) {
        self.base.close();
    }
}
