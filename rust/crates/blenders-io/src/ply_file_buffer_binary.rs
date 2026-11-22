/// Port of `ply/exporter/ply_file_buffer_binary.cc`
/// — little-endian writer.
pub fn ply_write_binary_f32(v: f32) -> [u8; 4] {
    v.to_le_bytes()
}
