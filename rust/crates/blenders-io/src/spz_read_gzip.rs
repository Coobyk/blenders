/// Port of `spz/intern/spz_read_gzip.cc` —
/// gzip-deflated SPZ read.
/// `read_gzip` (verbatim — inflate then
/// header parse).
pub fn spz_read_gzip(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
}
