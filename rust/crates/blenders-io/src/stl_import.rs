/// Port of `stl/importer/stl_import.cc` —
/// STL import entry (sniff ascii/binary).
/// `stl_import` (verbatim — ascii if starts
/// with "solid" and lacks binary length match).
pub fn stl_import_is_ascii(data: &[u8]) -> bool {
    if data.len() >= 5 && &data[..5] == b"solid" {
        // ASCII "solid" file won't have valid binary length.
        if data.len() >= 84 {
            let count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]) as usize;
            let expected = 84 + count * 50;
            if expected != data.len() {
                return true;
            }
        } else {
            return true;
        }
    }
    false
}
