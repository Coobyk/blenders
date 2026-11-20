//! Port of `filereader_gzip.cc` — `GzipReader`: streaming gzip inflate
//! over an inner `FileReader` (zlib `inflateInit2(16+MAX_WBITS)`).

use crate::filereader_file::FileReader;

/// `BLI_filereader_new_gzip` — non-seekable gzip reader wrapping `base`.
pub fn filereader_new_gzip(base: Box<dyn FileReader>) -> Box<dyn FileReader> {
    Box::new(GzipReader {
        base,
        decoder_state: None,
        in_buf: vec![0u8; 256 * 2014],
        offset: 0,
    })
}

struct GzipReader {
    base: Box<dyn FileReader>,
    /// flate2 decoder built on first read (stream starts at offset 0).
    decoder_state: Option<flate2::Decompress>,
    in_buf: Vec<u8>,
    offset: i64,
}

impl FileReader for GzipReader {
    fn read(&mut self, buffer: &mut [u8]) -> i64 {
        // Inflate loop: fill input when empty, stop on stream end.
        if self.decoder_state.is_none() {
            self.decoder_state = Some(flate2::Decompress::new(true)); // gzip wbits
        }
        let dec = self.decoder_state.as_mut().unwrap();
        let mut produced = 0usize;
        let mut in_data: Vec<u8> = Vec::new();
        let mut in_pos = 0usize;
        while produced < buffer.len() {
            if in_pos >= in_data.len() {
                let n = self.base.read(&mut self.in_buf);
                if n <= 0 {
                    break;
                }
                in_data = self.in_buf[..n as usize].to_vec();
                in_pos = 0;
            }
            let before_in = dec.total_in();
            let before_out = dec.total_out();
            let res = dec.decompress(
                &in_data[in_pos..],
                &mut buffer[produced..],
                flate2::FlushDecompress::None,
            );
            let cin = (dec.total_in() - before_in) as usize;
            let cout = (dec.total_out() - before_out) as usize;
            in_pos += cin;
            produced += cout;
            match res {
                Ok(flate2::Status::Ok) | Ok(flate2::Status::BufError) => {
                    if cin == 0 && cout == 0 {
                        break;
                    }
                }
                Ok(flate2::Status::StreamEnd) => break,
                Err(_) => break,
            }
        }
        self.offset += produced as i64;
        produced as i64
    }
    fn seek(&mut self, _offset: i64, _whence: i32) -> i64 {
        // gzip streams are not seekable (C returns -1 / doesn't set seek).
        -1
    }
    fn offset(&self) -> i64 {
        self.offset
    }
    fn close(mut self: Box<Self>) {
        self.base.close();
    }
}


