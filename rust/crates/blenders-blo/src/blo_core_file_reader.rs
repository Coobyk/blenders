//! Port of `intern/blo_core_file_reader.cc` — file
//! reader sniffing (uncompressed vs gzip).

/// `BLO_file_reader_uncompressed` sniff result
/// (verbatim — first 7 bytes "BLENDER" → raw blend;
/// else needs decompressor).
#[derive(PartialEq)]
pub enum FileSniff {
    /// `memcmp(first, "BLENDER", 7) == 0` → blend.
    UncompressedBlend,
    /// not a raw blend — try decompressors.
    NotBlend,
    /// file smaller than 7 bytes.
    TooSmall,
}
/// `BLO_file_reader_uncompressed` (verbatim).
pub fn blo_file_reader_sniff(first_bytes: &[u8]) -> FileSniff {
    if first_bytes.len() < 7 {
        return FileSniff::TooSmall;
    }
    if &first_bytes[..7] == b"BLENDER" {
        return FileSniff::UncompressedBlend;
    }
    FileSniff::NotBlend
}
