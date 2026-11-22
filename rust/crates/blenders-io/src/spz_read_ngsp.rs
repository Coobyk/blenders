/// Port of `spz/intern/spz_read_ngsp.cc` —
/// ngsp-variant SPZ read.
/// `read_ngsp` (verbatim — v2 header).
pub fn spz_read_ngsp(data: &[u8]) -> bool {
    data.len() >= 4 && u32::from_le_bytes([data[0], data[1], data[2], data[3]]) == 0x5053474e
}
